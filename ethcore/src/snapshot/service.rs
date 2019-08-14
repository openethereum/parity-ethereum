// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Snapshot network service implementation.

use std::collections::HashSet;
use std::io::{self, Read, ErrorKind};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::cmp;

use super::{ManifestData, StateRebuilder, Rebuilder, RestorationStatus, SnapshotService, MAX_CHUNK_SIZE};
use super::io::{SnapshotReader, LooseReader, SnapshotWriter, LooseWriter};

use blockchain::{BlockChain, BlockChainDB, BlockChainDBHandler};
use client::{BlockChainClient, Client, ChainInfo, ClientIoMessage};
use client_traits::BlockInfo;
use engines::Engine;
use hash::keccak;
use types::{
	errors::{EthcoreError as Error, SnapshotError, SnapshotError::UnlinkedAncientBlockChain},
	ids::BlockId,
};

use io::IoChannel;

use ethereum_types::H256;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use bytes::Bytes;
use journaldb::Algorithm;
use kvdb::DBTransaction;
use snappy;

/// Helper for removing directories in case of error.
struct Guard(bool, PathBuf);

impl Guard {
	fn new(path: PathBuf) -> Self { Guard(true, path) }

	#[cfg(test)]
	fn benign() -> Self { Guard(false, PathBuf::default()) }

	fn disarm(mut self) { self.0 = false }
}

impl Drop for Guard {
	fn drop(&mut self) {
		if self.0 {
			let _ = fs::remove_dir_all(&self.1);
		}
	}
}

/// External database restoration handler
pub trait DatabaseRestore: Send + Sync {
	/// Restart with a new backend. Takes ownership of passed database and moves it to a new location.
	fn restore_db(&self, new_db: &str) -> Result<(), Error>;
}

/// State restoration manager.
struct Restoration {
	manifest: ManifestData,
	state_chunks_left: HashSet<H256>,
	block_chunks_left: HashSet<H256>,
	state: StateRebuilder,
	secondary: Box<dyn Rebuilder>,
	writer: Option<LooseWriter>,
	snappy_buffer: Bytes,
	final_state_root: H256,
	guard: Guard,
	db: Arc<dyn BlockChainDB>,
}

struct RestorationParams<'a> {
	manifest: ManifestData, // manifest to base restoration on.
	pruning: Algorithm, // pruning algorithm for the database.
	db: Arc<dyn BlockChainDB>, // database
	writer: Option<LooseWriter>, // writer for recovered snapshot.
	genesis: &'a [u8], // genesis block of the chain.
	guard: Guard, // guard for the restoration directory.
	engine: &'a dyn Engine,
}

impl Restoration {
	// make a new restoration using the given parameters.
	fn new(params: RestorationParams) -> Result<Self, Error> {
		let manifest = params.manifest;

		let state_chunks = manifest.state_hashes.iter().cloned().collect();
		let block_chunks = manifest.block_hashes.iter().cloned().collect();

		let raw_db = params.db;

		let chain = BlockChain::new(Default::default(), params.genesis, raw_db.clone());
		let components = params.engine.snapshot_components()
			.ok_or_else(|| ::snapshot::Error::SnapshotsUnsupported)?;

		let secondary = components.rebuilder(chain, raw_db.clone(), &manifest)?;

		let final_state_root = manifest.state_root.clone();

		Ok(Restoration {
			manifest,
			state_chunks_left: state_chunks,
			block_chunks_left: block_chunks,
			state: StateRebuilder::new(raw_db.key_value().clone(), params.pruning),
			secondary,
			writer: params.writer,
			snappy_buffer: Vec::new(),
			final_state_root,
			guard: params.guard,
			db: raw_db,
		})
	}

	// feeds a state chunk, aborts early if `flag` becomes false.
	fn feed_state(&mut self, hash: H256, chunk: &[u8], flag: &AtomicBool) -> Result<(), Error> {
		if self.state_chunks_left.contains(&hash) {
			let expected_len = snappy::decompressed_len(chunk)?;
			if expected_len > MAX_CHUNK_SIZE {
				trace!(target: "snapshot", "Discarding large chunk: {} vs {}", expected_len, MAX_CHUNK_SIZE);
				return Err(::snapshot::Error::ChunkTooLarge.into());
			}
			let len = snappy::decompress_into(chunk, &mut self.snappy_buffer)?;

			self.state.feed(&self.snappy_buffer[..len], flag)?;

			if let Some(ref mut writer) = self.writer.as_mut() {
				writer.write_state_chunk(hash, chunk)?;
			}

			self.state_chunks_left.remove(&hash);
		}

		Ok(())
	}

	// feeds a block chunk
	fn feed_blocks(&mut self, hash: H256, chunk: &[u8], engine: &dyn Engine, flag: &AtomicBool) -> Result<(), Error> {
		if self.block_chunks_left.contains(&hash) {
			let expected_len = snappy::decompressed_len(chunk)?;
			if expected_len > MAX_CHUNK_SIZE {
				trace!(target: "snapshot", "Discarding large chunk: {} vs {}", expected_len, MAX_CHUNK_SIZE);
				return Err(::snapshot::Error::ChunkTooLarge.into());
			}
			let len = snappy::decompress_into(chunk, &mut self.snappy_buffer)?;

			self.secondary.feed(&self.snappy_buffer[..len], engine, flag)?;
			if let Some(ref mut writer) = self.writer.as_mut() {
				 writer.write_block_chunk(hash, chunk)?;
			}

			self.block_chunks_left.remove(&hash);
		}

		Ok(())
	}

	// finish up restoration.
	fn finalize(mut self) -> Result<(), Error> {
		use trie::TrieError;

		if !self.is_done() { return Ok(()) }

		// verify final state root.
		let root = self.state.state_root();
		if root != self.final_state_root {
			warn!("Final restored state has wrong state root: expected {:?}, got {:?}", self.final_state_root, root);
			return Err(TrieError::InvalidStateRoot(root).into());
		}

		// check for missing code.
		self.state.finalize(self.manifest.block_number, self.manifest.block_hash)?;

		// connect out-of-order chunks and verify chain integrity.
		self.secondary.finalize()?;

		if let Some(writer) = self.writer {
			writer.finish(self.manifest)?;
		}

		self.guard.disarm();
		trace!(target: "snapshot", "restoration finalised correctly");
		Ok(())
	}

	// is everything done?
	fn is_done(&self) -> bool {
		self.block_chunks_left.is_empty() && self.state_chunks_left.is_empty()
	}
}

/// Type alias for client io channel.
pub type Channel = IoChannel<ClientIoMessage>;

/// Trait alias for the Client Service used
pub trait SnapshotClient: BlockChainClient + BlockInfo + DatabaseRestore {}

/// Snapshot service parameters.
pub struct ServiceParams {
	/// The consensus engine this is built on.
	pub engine: Arc<dyn Engine>,
	/// The chain's genesis block.
	pub genesis_block: Bytes,
	/// State pruning algorithm.
	pub pruning: Algorithm,
	/// Handler for opening a restoration DB.
	pub restoration_db_handler: Box<dyn BlockChainDBHandler>,
	/// Async IO channel for sending messages.
	pub channel: Channel,
	/// The directory to put snapshots in.
	/// Usually "<chain hash>/snapshot"
	pub snapshot_root: PathBuf,
	/// A handle for database restoration.
	pub client: Arc<dyn SnapshotClient>,
}

/// `SnapshotService` implementation.
/// This controls taking snapshots and restoring from them.
pub struct Service {
	restoration: Mutex<Option<Restoration>>,
	restoration_db_handler: Box<dyn BlockChainDBHandler>,
	snapshot_root: PathBuf,
	io_channel: Mutex<Channel>,
	pruning: Algorithm,
	status: Mutex<RestorationStatus>,
	reader: RwLock<Option<LooseReader>>,
	engine: Arc<dyn Engine>,
	genesis_block: Bytes,
	state_chunks: AtomicUsize,
	block_chunks: AtomicUsize,
	client: Arc<dyn SnapshotClient>,
	progress: super::Progress,
	taking_snapshot: AtomicBool,
	restoring_snapshot: AtomicBool,
}

impl Service {
	/// Create a new snapshot service from the given parameters.
	pub fn new(params: ServiceParams) -> Result<Self, Error> {
		let mut service = Service {
			restoration: Mutex::new(None),
			restoration_db_handler: params.restoration_db_handler,
			snapshot_root: params.snapshot_root,
			io_channel: Mutex::new(params.channel),
			pruning: params.pruning,
			status: Mutex::new(RestorationStatus::Inactive),
			reader: RwLock::new(None),
			engine: params.engine,
			genesis_block: params.genesis_block,
			state_chunks: AtomicUsize::new(0),
			block_chunks: AtomicUsize::new(0),
			client: params.client,
			progress: Default::default(),
			taking_snapshot: AtomicBool::new(false),
			restoring_snapshot: AtomicBool::new(false),
		};

		// create the root snapshot dir if it doesn't exist.
		if let Err(e) = fs::create_dir_all(&service.snapshot_root) {
			if e.kind() != ErrorKind::AlreadyExists {
				return Err(e.into())
			}
		}

		// delete the temporary restoration DB dir if it does exist.
		if let Err(e) = fs::remove_dir_all(service.restoration_db()) {
			if e.kind() != ErrorKind::NotFound {
				return Err(e.into())
			}
		}

		// delete the temporary snapshot dir if it does exist.
		if let Err(e) = fs::remove_dir_all(service.temp_snapshot_dir()) {
			if e.kind() != ErrorKind::NotFound {
				return Err(e.into())
			}
		}

		let reader = LooseReader::new(service.snapshot_dir()).ok();
		*service.reader.get_mut() = reader;

		Ok(service)
	}

	// get the current snapshot dir.
	fn snapshot_dir(&self) -> PathBuf {
		let mut dir = self.snapshot_root.clone();
		dir.push("current");
		dir
	}

	// get the temporary snapshot dir.
	fn temp_snapshot_dir(&self) -> PathBuf {
		let mut dir = self.snapshot_root.clone();
		dir.push("in_progress");
		dir
	}

	// get the restoration directory.
	fn restoration_dir(&self) -> PathBuf {
		let mut dir = self.snapshot_root.clone();
		dir.push("restoration");
		dir
	}

	// restoration db path.
	fn restoration_db(&self) -> PathBuf {
		let mut dir = self.restoration_dir();
		dir.push("db");
		dir
	}

	// temporary snapshot recovery path.
	fn temp_recovery_dir(&self) -> PathBuf {
		let mut dir = self.restoration_dir();
		dir.push("temp");
		dir
	}

	// previous snapshot chunks path.
	fn prev_chunks_dir(&self) -> PathBuf {
		let mut dir = self.snapshot_root.clone();
		dir.push("prev_chunks");
		dir
	}

	// Migrate the blocks in the current DB into the new chain
	fn migrate_blocks(&self) -> Result<usize, Error> {
		// Count the number of migrated blocks
		let mut count = 0;
		let rest_db = self.restoration_db();

		let cur_chain_info = self.client.chain_info();

		let next_db = self.restoration_db_handler.open(&rest_db)?;
		let next_chain = BlockChain::new(Default::default(), &[], next_db.clone());
		let next_chain_info = next_chain.chain_info();

		// The old database looks like this:
		// [genesis, best_ancient_block] ... [first_block, best_block]
		// If we are fully synced neither `best_ancient_block` nor `first_block` is set, and we can
		// assume that the whole range from [genesis, best_block] is imported.
		// The new database only contains the tip of the chain ([new_first_block, new_best_block]),
		// so the useful set of blocks is defined as:
		// [0 ... min(new.first_block, best_ancient_block or best_block)]
		//
		// If, for whatever reason, the old db does not have ancient blocks (i.e.
		// `best_ancient_block` is `None` AND a non-zero `first_block`), such that the old db looks
		// like [old_first_block..old_best_block] (which may or may not partially overlap with
		// [new_first_block..new_best_block]) we do the conservative thing and do not migrate the
		// old blocks.
		let find_range = || -> Option<(H256, H256)> {
			// In theory, if the current best_block is > new first_block (i.e. ranges overlap)
			// we could salvage them but what if there's been a re-org at the boundary and the two
			// chains do not match anymore? We'd have to check the existing blocks carefully.
			if cur_chain_info.ancient_block_number.is_none() && cur_chain_info.first_block_number.unwrap_or(0) > 0 {
				info!(target: "blockchain", "blocks in the current DB do not stretch back to genesis; can't salvage them into the new DB. In current DB, first block: #{:?}/{:?}, best block: #{:?}/{:?}",
					cur_chain_info.first_block_number, cur_chain_info.first_block_hash,
					cur_chain_info.best_block_number, cur_chain_info.best_block_hash);
				return None;
			}
			let next_available_from = next_chain_info.first_block_number?;
			let cur_available_to = cur_chain_info.ancient_block_number.unwrap_or(cur_chain_info.best_block_number);

			let highest_block_num = cmp::min(next_available_from.saturating_sub(1), cur_available_to);

			if highest_block_num == 0 {
				return None;
			}

			trace!(target: "snapshot", "Trying to import ancient blocks until {}. First block in new chain=#{}, first block in old chain=#{:?}, best block in old chain=#{}",
				highest_block_num, next_available_from, cur_chain_info.first_block_number, cur_chain_info.best_block_number);

			// Here we start from the highest block number and go backward to 0,
			// thus starting at `highest_block_num` and targeting `0`.
			let target_hash = self.client.block_hash(BlockId::Number(0))?;
			let start_hash = self.client.block_hash(BlockId::Number(highest_block_num))?;

			Some((start_hash, target_hash))
		};

		let (start_hash, target_hash) = match find_range() {
			Some(x) => x,
			None => return Ok(0),
		};

		let mut batch = DBTransaction::new();
		let mut parent_hash = start_hash;
		while parent_hash != target_hash {
			// Early return if restoration is aborted
			if !self.restoring_snapshot.load(Ordering::SeqCst) {
				return Ok(count);
			}

			let block = self.client.block(BlockId::Hash(parent_hash)).ok_or_else(|| {
				error!(target: "snapshot", "migrate_blocks: did not find block from parent_hash={:#x} (start_hash={:#x})", parent_hash, start_hash);
				UnlinkedAncientBlockChain(parent_hash)
			})?;
			parent_hash = block.parent_hash();

			let block_number = block.number();
			let block_receipts = self.client.block_receipts(&block.hash());
			let parent_total_difficulty = self.client.block_total_difficulty(BlockId::Hash(parent_hash));

			match (block_receipts, parent_total_difficulty) {
				(Some(block_receipts), Some(parent_total_difficulty)) => {
					let block_receipts = block_receipts.receipts;

					next_chain.insert_unordered_block(&mut batch, block, block_receipts, Some(parent_total_difficulty), false, true);
					count += 1;
				},
				_ => {
					// We couldn't reach the targeted hash
					error!(target: "snapshot", "migrate_blocks: failed to find receipts and parent total difficulty; cannot reach the target_hash ({:#x}). Block #{}, parent_hash={:#x}, parent_total_difficulty={:?}, start_hash={:#x}, ancient_block_number={:?}, best_block_number={:?}",
						target_hash, block_number, parent_hash, parent_total_difficulty,
						start_hash, cur_chain_info.ancient_block_number, cur_chain_info.best_block_number,
					);
					return Err(UnlinkedAncientBlockChain(parent_hash).into());
				},
			}

			// Writing changes to DB and logging every now and then
			if block_number % 1_000 == 0 {
				next_db.key_value().write_buffered(batch);
				next_chain.commit();
				next_db.key_value().flush().expect("DB flush failed.");
				batch = DBTransaction::new();
			}

			if block_number % 10_000 == 0 {
				info!(target: "snapshot", "Block restoration at #{}", block_number);
			}
		}

		// Final commit to the DB
		next_db.key_value().write_buffered(batch);
		next_chain.commit();
		next_db.key_value().flush().expect("DB flush failed.");

		// Update best ancient block in the Next Chain
		next_chain.update_best_ancient_block(&start_hash);
		Ok(count)
	}

	/// Get a reference to the snapshot reader.
	pub fn reader(&self) -> RwLockReadGuard<Option<LooseReader>> {
		self.reader.read()
	}

	/// Tick the snapshot service. This will log any active snapshot
	/// being taken.
	pub fn tick(&self) {
		if self.progress.done() || !self.taking_snapshot.load(Ordering::SeqCst) { return }

		let p = &self.progress;
		info!("Snapshot: {} accounts {} blocks {} bytes", p.accounts(), p.blocks(), p.size());
	}

	/// Take a snapshot at the block with the given number.
	/// calling this while a restoration is in progress or vice versa
	/// will lead to a race condition where the first one to finish will
	/// have their produced snapshot overwritten.
	pub fn take_snapshot(&self, client: &Client, num: u64) -> Result<(), Error> {
		if self.taking_snapshot.compare_and_swap(false, true, Ordering::SeqCst) {
			info!("Skipping snapshot at #{} as another one is currently in-progress.", num);
			return Ok(());
		}

		info!("Taking snapshot at #{}", num);
		self.progress.reset();

		let temp_dir = self.temp_snapshot_dir();
		let snapshot_dir = self.snapshot_dir();

		let _ = fs::remove_dir_all(&temp_dir);

		let writer = LooseWriter::new(temp_dir.clone())?;

		let guard = Guard::new(temp_dir.clone());
		let res = client.take_snapshot(writer, BlockId::Number(num), &self.progress);
		self.taking_snapshot.store(false, Ordering::SeqCst);
		if let Err(e) = res {
			if client.chain_info().best_block_number >= num + client.pruning_history() {
				// The state we were snapshotting was pruned before we could finish.
				info!("Periodic snapshot failed: block state pruned. Run with a longer `--pruning-history` or with `--no-periodic-snapshot`");
				return Err(e);
			} else {
				return Err(e);
			}
		}

		info!("Finished taking snapshot at #{}", num);

		let mut reader = self.reader.write();

		// destroy the old snapshot reader.
		*reader = None;

		if snapshot_dir.exists() {
			fs::remove_dir_all(&snapshot_dir)?;
		}

		fs::rename(temp_dir, &snapshot_dir)?;

		*reader = Some(LooseReader::new(snapshot_dir)?);

		guard.disarm();
		Ok(())
	}

	/// Initialize the restoration synchronously.
	/// The recover flag indicates whether to recover the restored snapshot.
	pub fn init_restore(&self, manifest: ManifestData, recover: bool) -> Result<(), Error> {
		let mut res = self.restoration.lock();

		let rest_dir = self.restoration_dir();
		let rest_db = self.restoration_db();
		let recovery_temp = self.temp_recovery_dir();
		let prev_chunks = self.prev_chunks_dir();

		// delete and restore the restoration dir.
		if let Err(e) = fs::remove_dir_all(&prev_chunks) {
			match e.kind() {
				ErrorKind::NotFound => {},
				_ => return Err(e.into()),
			}
		}

		// Move the previous recovery temp directory
		// to `prev_chunks` to be able to restart restoring
		// with previously downloaded blocks
		// This step is optional, so don't fail on error
		fs::rename(&recovery_temp, &prev_chunks).ok();

		self.state_chunks.store(0, Ordering::SeqCst);
		self.block_chunks.store(0, Ordering::SeqCst);

		// tear down existing restoration.
		*res = None;

		// delete and restore the restoration dir.
		if let Err(e) = fs::remove_dir_all(&rest_dir) {
			match e.kind() {
				ErrorKind::NotFound => {},
				_ => return Err(e.into()),
			}
		}

		*self.status.lock() = RestorationStatus::Initializing {
			chunks_done: 0,
			state_chunks: manifest.state_hashes.len() as u32,
			block_chunks: manifest.block_hashes.len() as u32,
		};

		fs::create_dir_all(&rest_dir)?;

		// make new restoration.
		let writer = match recover {
			true => Some(LooseWriter::new(recovery_temp)?),
			false => None
		};

		let params = RestorationParams {
			manifest: manifest.clone(),
			pruning: self.pruning,
			db: self.restoration_db_handler.open(&rest_db)?,
			writer,
			genesis: &self.genesis_block,
			guard: Guard::new(rest_db),
			engine: &*self.engine,
		};

		let state_chunks = manifest.state_hashes.len();
		let block_chunks = manifest.block_hashes.len();

		*res = Some(Restoration::new(params)?);

		self.restoring_snapshot.store(true, Ordering::SeqCst);

		// Import previous chunks, continue if it fails
		self.import_prev_chunks(&mut res, manifest).ok();

		// It could be that the restoration failed or completed in the meanwhile
		let mut restoration_status = self.status.lock();
		if let RestorationStatus::Initializing { .. } = *restoration_status {
			*restoration_status = RestorationStatus::Ongoing {
				state_chunks: state_chunks as u32,
				block_chunks: block_chunks as u32,
				state_chunks_done: self.state_chunks.load(Ordering::SeqCst) as u32,
				block_chunks_done: self.block_chunks.load(Ordering::SeqCst) as u32,
			};
		}

		Ok(())
	}

	/// Import the previous chunks into the current restoration
	fn import_prev_chunks(&self, restoration: &mut Option<Restoration>, manifest: ManifestData) -> Result<(), Error> {
		let prev_chunks = self.prev_chunks_dir();

		// Restore previous snapshot chunks
		let files = fs::read_dir(prev_chunks.as_path())?;
		let mut num_temp_chunks = 0;

		for prev_chunk_file in files {
			// Don't go over all the files if the restoration has been aborted
			if !self.restoring_snapshot.load(Ordering::SeqCst) {
				trace!(target:"snapshot", "Aborting importing previous chunks");
				return Ok(());
			}
			// Import the chunk, don't fail and continue if one fails
			match self.import_prev_chunk(restoration, &manifest, prev_chunk_file) {
				Ok(true) => num_temp_chunks += 1,
				Err(e) => trace!(target: "snapshot", "Error importing chunk: {:?}", e),
				_ => (),
			}
		}

		trace!(target:"snapshot", "Imported {} previous chunks", num_temp_chunks);

		// Remove the prev temp directory
		fs::remove_dir_all(&prev_chunks)?;

		Ok(())
	}

	/// Import a previous chunk at the given path. Returns whether the block was imported or not
	fn import_prev_chunk(&self, restoration: &mut Option<Restoration>, manifest: &ManifestData, file: io::Result<fs::DirEntry>) -> Result<bool, Error> {
		let file = file?;
		let path = file.path();

		let mut file = File::open(path.clone())?;
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer)?;

		let hash = keccak(&buffer);

		let is_state = if manifest.block_hashes.contains(&hash) {
			false
		} else if manifest.state_hashes.contains(&hash) {
			true
		} else {
			return Ok(false);
		};

		self.feed_chunk_with_restoration(restoration, hash, &buffer, is_state)?;

		trace!(target: "snapshot", "Fed chunk {:?}", hash);

		Ok(true)
	}

	// finalize the restoration. this accepts an already-locked
	// restoration as an argument -- so acquiring it again _will_
	// lead to deadlock.
	fn finalize_restoration(&self, rest: &mut Option<Restoration>) -> Result<(), Error> {
		trace!(target: "snapshot", "finalizing restoration");
		*self.status.lock() = RestorationStatus::Finalizing;

		let recover = rest.as_ref().map_or(false, |rest| rest.writer.is_some());

		// destroy the restoration before replacing databases and snapshot.
		rest.take()
			.map(|r| r.finalize())
			.unwrap_or(Ok(()))?;

		let migrated_blocks = self.migrate_blocks()?;
		info!(target: "snapshot", "Migrated {} ancient blocks", migrated_blocks);

		// replace the Client's database with the new one (restart the Client).
		self.client.restore_db(&*self.restoration_db().to_string_lossy())?;

		if recover {
			let mut reader = self.reader.write();
			*reader = None; // destroy the old reader if it existed.

			let snapshot_dir = self.snapshot_dir();

			if snapshot_dir.exists() {
				trace!(target: "snapshot", "removing old snapshot dir at {}", snapshot_dir.to_string_lossy());
				fs::remove_dir_all(&snapshot_dir)?;
			}

			trace!(target: "snapshot", "copying restored snapshot files over");
			fs::rename(self.temp_recovery_dir(), &snapshot_dir)?;

			*reader = Some(LooseReader::new(snapshot_dir)?);
		}

		let _ = fs::remove_dir_all(self.restoration_dir());
		*self.status.lock() = RestorationStatus::Inactive;

		Ok(())
	}

	/// Feed a chunk of either kind (block or state). no-op if no restoration or status is wrong.
	fn feed_chunk(&self, hash: H256, chunk: &[u8], is_state: bool) {
		// TODO: be able to process block chunks and state chunks at same time?
		let r = {
			let mut restoration = self.restoration.lock();
			self.feed_chunk_with_restoration(&mut restoration, hash, chunk, is_state)
		};
		match r {
			Ok(()) |
			Err(Error::Snapshot(SnapshotError::RestorationAborted)) => (),
			Err(e) => {
				// TODO: after this we're sometimes deadlocked
				warn!("Encountered error during snapshot restoration: {}", e);
				self.abort_restore();
				if let Some(mut status) = self.status.try_lock_for(std::time::Duration::from_millis(10)) {
					*status = RestorationStatus::Failed;
				}
				let _ = fs::remove_dir_all(self.restoration_dir());
			}
		}
	}

	/// Feed a chunk with the Restoration
	fn feed_chunk_with_restoration(&self, restoration: &mut Option<Restoration>, hash: H256, chunk: &[u8], is_state: bool) -> Result<(), Error> {
		let (result, db) = {
			match self.status() {
				RestorationStatus::Inactive | RestorationStatus::Failed | RestorationStatus::Finalizing => {
					trace!(target: "snapshot", "Tried to restore chunk {:x} while inactive, failed or finalizing", hash);
					return Ok(());
				},
				RestorationStatus::Ongoing { .. } | RestorationStatus::Initializing { .. } => {
					let (res, db) = {
						let rest = match *restoration {
							Some(ref mut r) => r,
							None => return Ok(()),
						};

						(match is_state {
							true => rest.feed_state(hash, chunk, &self.restoring_snapshot),
							false => rest.feed_blocks(hash, chunk, &*self.engine, &self.restoring_snapshot),
						}.map(|_| rest.is_done()), rest.db.clone())
					};

					let res = match res {
						Ok(is_done) => {
							match is_state {
								true => self.state_chunks.fetch_add(1, Ordering::SeqCst),
								false => self.block_chunks.fetch_add(1, Ordering::SeqCst),
							};

							match is_done {
								true => {
									db.key_value().flush()?;
									drop(db);
									return self.finalize_restoration(&mut *restoration);
								},
								false => Ok(())
							}
						}
						other => other.map(drop),
					};
					(res, db)
				}
			}
		};

		result?;
		db.key_value().flush()?;
		Ok(())
	}

	/// Feed a state chunk to be processed synchronously.
	pub fn feed_state_chunk(&self, hash: H256, chunk: &[u8]) {
		self.feed_chunk(hash, chunk, true);
	}

	/// Feed a block chunk to be processed synchronously.
	pub fn feed_block_chunk(&self, hash: H256, chunk: &[u8]) {
		self.feed_chunk(hash, chunk, false);
	}
}

impl SnapshotService for Service {
	fn manifest(&self) -> Option<ManifestData> {
		self.reader.read().as_ref().map(|r| r.manifest().clone())
	}

	fn supported_versions(&self) -> Option<(u64, u64)> {
		self.engine.snapshot_components()
			.map(|c| (c.min_supported_version(), c.current_version()))
	}

	fn chunk(&self, hash: H256) -> Option<Bytes> {
		self.reader.read().as_ref().and_then(|r| r.chunk(hash).ok())
	}

	fn completed_chunks(&self) -> Option<Vec<H256>> {
		let restoration = self.restoration.lock();

		match *restoration {
			Some(ref restoration) => {
				let completed_chunks = restoration.manifest.block_hashes
					.iter()
					.filter(|h| !restoration.block_chunks_left.contains(h))
					.chain(
						restoration.manifest.state_hashes
							.iter()
							.filter(|h| !restoration.state_chunks_left.contains(h))
					)
					.map(|h| *h)
					.collect();

				Some(completed_chunks)
			},
			None => None,
		}
	}

	fn status(&self) -> RestorationStatus {
		let mut cur_status = self.status.lock();

		match *cur_status {
			RestorationStatus::Initializing { ref mut chunks_done, .. } => {
				*chunks_done = self.state_chunks.load(Ordering::SeqCst) as u32 +
					self.block_chunks.load(Ordering::SeqCst) as u32;
			}
			RestorationStatus::Ongoing { ref mut state_chunks_done, ref mut block_chunks_done, .. } => {
				*state_chunks_done = self.state_chunks.load(Ordering::SeqCst) as u32;
				*block_chunks_done = self.block_chunks.load(Ordering::SeqCst) as u32;
			},
			_ => (),
		}

		cur_status.clone()
	}

	fn begin_restore(&self, manifest: ManifestData) {
		if let Err(e) = self.io_channel.lock().send(ClientIoMessage::BeginRestoration(manifest)) {
			trace!("Error sending snapshot service message: {:?}", e);
		}
	}

	fn abort_restore(&self) {
		trace!(target: "snapshot", "Aborting restore");
		self.restoring_snapshot.store(false, Ordering::SeqCst);
		*self.restoration.lock() = None;
		*self.status.lock() = RestorationStatus::Inactive;
	}

	fn restore_state_chunk(&self, hash: H256, chunk: Bytes) {
		if let Err(e) = self.io_channel.lock().send(ClientIoMessage::FeedStateChunk(hash, chunk)) {
			trace!("Error sending snapshot service message: {:?}", e);
		}
	}

	fn restore_block_chunk(&self, hash: H256, chunk: Bytes) {
		if let Err(e) = self.io_channel.lock().send(ClientIoMessage::FeedBlockChunk(hash, chunk)) {
			trace!("Error sending snapshot service message: {:?}", e);
		}
	}

	fn abort_snapshot(&self) {
		if self.taking_snapshot.load(Ordering::SeqCst) {
			trace!(target: "snapshot", "Aborting snapshot – Snapshot under way");
			self.progress.abort.store(true, Ordering::SeqCst);
		}
	}

	fn shutdown(&self) {
		trace!(target: "snapshot", "Shut down SnapshotService");
		self.abort_restore();
		trace!(target: "snapshot", "Shut down SnapshotService - restore aborted");
		self.abort_snapshot();
		trace!(target: "snapshot", "Shut down SnapshotService - snapshot aborted");
	}
}

impl Drop for Service {
	fn drop(&mut self) {
		trace!(target: "shutdown", "Dropping Service");
		self.abort_restore();
		trace!(target: "shutdown", "Dropping Service - restore aborted");
		self.abort_snapshot();
		trace!(target: "shutdown", "Dropping Service - snapshot aborted");
	}
}

#[cfg(test)]
mod tests {
	use client::ClientIoMessage;
	use io::{IoService};
	use crate::spec;
	use journaldb::Algorithm;
	use snapshot::{ManifestData, RestorationStatus, SnapshotService};
	use super::*;
	use tempdir::TempDir;
	use test_helpers::{generate_dummy_client_with_spec_and_data, restoration_db_handler};

	#[test]
	fn sends_async_messages() {
		let gas_prices = vec![1.into(), 2.into(), 3.into(), 999.into()];
		let client = generate_dummy_client_with_spec_and_data(spec::new_null, 400, 5, &gas_prices);
		let service = IoService::<ClientIoMessage>::start().unwrap();
		let spec = spec::new_test();

		let tempdir = TempDir::new("").unwrap();
		let dir = tempdir.path().join("snapshot");

		let snapshot_params = ServiceParams {
			engine: spec.engine.clone(),
			genesis_block: spec.genesis_block(),
			restoration_db_handler: restoration_db_handler(Default::default()),
			pruning: Algorithm::Archive,
			channel: service.channel(),
			snapshot_root: dir,
			client: client,
		};

		let service = Service::new(snapshot_params).unwrap();

		assert!(service.manifest().is_none());
		assert!(service.chunk(Default::default()).is_none());
		assert_eq!(service.status(), RestorationStatus::Inactive);

		let manifest = ManifestData {
			version: 2,
			state_hashes: vec![],
			block_hashes: vec![],
			state_root: Default::default(),
			block_number: 0,
			block_hash: Default::default(),
		};

		service.begin_restore(manifest);
		service.abort_restore();
		service.restore_state_chunk(Default::default(), vec![]);
		service.restore_block_chunk(Default::default(), vec![]);
	}

	#[test]
	fn cannot_finish_with_invalid_chunks() {
		use ethereum_types::H256;
		use kvdb_rocksdb::DatabaseConfig;

		let spec = spec::new_test();
		let tempdir = TempDir::new("").unwrap();

		let state_hashes: Vec<_> = (0..5).map(|_| H256::random()).collect();
		let block_hashes: Vec<_> = (0..5).map(|_| H256::random()).collect();
		let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);
		let gb = spec.genesis_block();
		let flag = ::std::sync::atomic::AtomicBool::new(true);

		let params = RestorationParams {
			manifest: ManifestData {
				version: 2,
				state_hashes: state_hashes.clone(),
				block_hashes: block_hashes.clone(),
				state_root: H256::zero(),
				block_number: 100000,
				block_hash: H256::zero(),
			},
			pruning: Algorithm::Archive,
			db: restoration_db_handler(db_config).open(&tempdir.path().to_owned()).unwrap(),
			writer: None,
			genesis: &gb,
			guard: Guard::benign(),
			engine: &*spec.engine.clone(),
		};

		let mut restoration = Restoration::new(params).unwrap();
		let definitely_bad_chunk = [1, 2, 3, 4, 5];

		for hash in state_hashes {
			assert!(restoration.feed_state(hash, &definitely_bad_chunk, &flag).is_err());
			assert!(!restoration.is_done());
		}

		for hash in block_hashes {
			assert!(restoration.feed_blocks(hash, &definitely_bad_chunk, &*spec.engine, &flag).is_err());
			assert!(!restoration.is_done());
		}
	}
}
