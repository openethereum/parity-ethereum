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

//! Snapshot creation, restoration, and network service.
//!
//! Documentation of the format can be found at
//! https://wiki.parity.io/Warp-Sync-Snapshot-Format

use std::collections::{HashMap, HashSet};
use std::cmp;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use hash::{keccak, KECCAK_NULL_RLP, KECCAK_EMPTY};

use account_db::{AccountDB, AccountDBMut};
use blockchain::{BlockChain, BlockProvider};
use types::{
	ids::BlockId,
	header::Header,
	errors::{SnapshotError as Error, EthcoreError},
	snapshot::Progress,
};
use ethereum_types::{H256, U256};
use hash_db::HashDB;
use keccak_hasher::KeccakHasher;
use snappy;
use bytes::Bytes;
use parking_lot::Mutex;
use journaldb::{self, Algorithm, JournalDB};
use kvdb::{KeyValueDB, DBValue};
use trie::{Trie, TrieMut};
use ethtrie::{TrieDB, TrieDBMut};
use rlp::{RlpStream, Rlp};
use bloom_journal::Bloom;
use num_cpus;
use types::snapshot::ManifestData;

use self::io::SnapshotWriter;

use super::state_db::StateDB;
use account_state::Account as StateAccount;
use engine::Engine;

use crossbeam_utils::thread;
use rand::{Rng, rngs::OsRng};

pub use self::consensus::*;
pub use self::service::{SnapshotClient, Service, DatabaseRestore};
pub use self::traits::{SnapshotService, SnapshotComponents, Rebuilder};
pub use self::watcher::Watcher;
pub use types::basic_account::BasicAccount;

pub mod io;
pub mod service;

mod account;
mod block;
mod consensus;
mod watcher;

#[cfg(test)]
mod tests;

mod traits;

// Try to have chunks be around 4MB (before compression)
const PREFERRED_CHUNK_SIZE: usize = 4 * 1024 * 1024;

// Maximal chunk size (decompressed)
// Snappy::decompressed_len estimation may sometimes yield results greater
// than PREFERRED_CHUNK_SIZE so allow some threshold here.
const MAX_CHUNK_SIZE: usize = PREFERRED_CHUNK_SIZE / 4 * 5;

// Minimum supported state chunk version.
const MIN_SUPPORTED_STATE_CHUNK_VERSION: u64 = 1;
// current state chunk version.
const STATE_CHUNK_VERSION: u64 = 2;
/// number of snapshot subparts, must be a power of 2 in [1; 256]
const SNAPSHOT_SUBPARTS: usize = 16;
/// Maximum number of snapshot subparts (must be a multiple of `SNAPSHOT_SUBPARTS`)
const MAX_SNAPSHOT_SUBPARTS: usize = 256;

/// Configuration for the Snapshot service
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotConfiguration {
	/// If `true`, no periodic snapshots will be created
	pub no_periodic: bool,
	/// Number of threads for creating snapshots
	pub processing_threads: usize,
}

impl Default for SnapshotConfiguration {
	fn default() -> Self {
		SnapshotConfiguration {
			no_periodic: false,
			processing_threads: ::std::cmp::max(1, num_cpus::get_physical() / 2),
		}
	}
}

/// Take a snapshot using the given blockchain, starting block hash, and database, writing into the given writer.
pub fn take_snapshot<W: SnapshotWriter + Send>(
	chunker: Box<dyn SnapshotComponents>,
	chain: &BlockChain,
	block_hash: H256,
	state_db: &dyn HashDB<KeccakHasher, DBValue>,
	writer: W,
	p: &Progress,
	processing_threads: usize,
) -> Result<(), Error> {
	let start_header = chain.block_header_data(&block_hash)
		.ok_or_else(|| Error::InvalidStartingBlock(BlockId::Hash(block_hash)))?;
	let state_root = start_header.state_root();
	let block_number = start_header.number();

	info!("Taking snapshot starting at block {}", block_number);

	let version = chunker.current_version();
	let writer = Mutex::new(writer);
	let (state_hashes, block_hashes) = thread::scope(|scope| -> Result<(Vec<H256>, Vec<H256>), Error> {
		let writer = &writer;
		let block_guard = scope.spawn(move |_| {
			chunk_secondary(chunker, chain, block_hash, writer, p)
		});

		// The number of threads must be between 1 and SNAPSHOT_SUBPARTS
		assert!(processing_threads >= 1, "Cannot use less than 1 threads for creating snapshots");
		let num_threads: usize = cmp::min(processing_threads, SNAPSHOT_SUBPARTS);
		info!(target: "snapshot", "Using {} threads for Snapshot creation.", num_threads);

		let mut state_guards = Vec::with_capacity(num_threads as usize);

		for thread_idx in 0..num_threads {
			let state_guard = scope.spawn(move |_| -> Result<Vec<H256>, Error> {
				let mut chunk_hashes = Vec::new();

				for part in (thread_idx..SNAPSHOT_SUBPARTS).step_by(num_threads) {
					debug!(target: "snapshot", "Chunking part {} in thread {}", part, thread_idx);
					let mut hashes = chunk_state(state_db, &state_root, writer, p, Some(part), thread_idx)?;
					chunk_hashes.append(&mut hashes);
				}

				Ok(chunk_hashes)
			});
			state_guards.push(state_guard);
		}

		let block_hashes = block_guard.join().expect("Sub-thread never panics; qed")?;
		let mut state_hashes = Vec::new();

		for guard in state_guards {
			let part_state_hashes = guard.join().expect("Sub-thread never panics; qed")?;
			state_hashes.extend(part_state_hashes);
		}

		debug!(target: "snapshot", "Took a snapshot of {} accounts", p.accounts.load(Ordering::SeqCst));
		Ok((state_hashes, block_hashes))
	}).expect("Sub-thread never panics; qed")?;

	info!(target: "snapshot", "produced {} state chunks and {} block chunks.", state_hashes.len(), block_hashes.len());

	let manifest_data = ManifestData {
		version,
		state_hashes,
		block_hashes,
		state_root,
		block_number,
		block_hash,
	};

	writer.into_inner().finish(manifest_data)?;

	p.done.store(true, Ordering::SeqCst);

	Ok(())
}

/// Create and write out all secondary chunks to disk, returning a vector of all
/// the hashes of secondary chunks created.
///
/// Secondary chunks are engine-specific, but they intend to corroborate the state data
/// in the state chunks.
/// Returns a list of chunk hashes, with the first having the blocks furthest from the genesis.
pub fn chunk_secondary<'a>(
	mut chunker: Box<dyn SnapshotComponents>,
	chain: &'a BlockChain,
	start_hash: H256,
	writer: &Mutex<dyn SnapshotWriter + 'a>,
	progress: &'a Progress
) -> Result<Vec<H256>, Error> {
	let mut chunk_hashes = Vec::new();
	let mut snappy_buffer = vec![0; snappy::max_compressed_len(PREFERRED_CHUNK_SIZE)];

	{
		let mut chunk_sink = |raw_data: &[u8]| {
			let compressed_size = snappy::compress_into(raw_data, &mut snappy_buffer);
			let compressed = &snappy_buffer[..compressed_size];
			let hash = keccak(&compressed);
			let size = compressed.len();

			writer.lock().write_block_chunk(hash, compressed)?;
			trace!(target: "snapshot", "wrote secondary chunk. hash: {:x}, size: {}, uncompressed size: {}",
				hash, size, raw_data.len());

			progress.size.fetch_add(size as u64, Ordering::SeqCst);
			chunk_hashes.push(hash);
			Ok(())
		};

		chunker.chunk_all(
			chain,
			start_hash,
			&mut chunk_sink,
			progress,
			PREFERRED_CHUNK_SIZE,
		)?;
	}

	Ok(chunk_hashes)
}

/// State trie chunker.
struct StateChunker<'a> {
	hashes: Vec<H256>,
	rlps: Vec<Bytes>,
	cur_size: usize,
	snappy_buffer: Vec<u8>,
	writer: &'a Mutex<dyn SnapshotWriter + 'a>,
	progress: &'a Progress,
	thread_idx: usize,
}

impl<'a> StateChunker<'a> {
	// Push a key, value pair to be encoded.
	//
	// If the buffer is greater than the desired chunk size,
	// this will write out the data to disk.
	fn push(&mut self, data: Bytes) -> Result<(), Error> {
		self.cur_size += data.len();
		self.rlps.push(data);
		Ok(())
	}

	// Write out the buffer to disk, pushing the created chunk's hash to
	// the list.
	fn write_chunk(&mut self) -> Result<(), Error> {
		let num_entries = self.rlps.len();
		let mut stream = RlpStream::new_list(num_entries);
		for rlp in self.rlps.drain(..) {
			stream.append_raw(&rlp, 1);
		}

		let raw_data = stream.out();

		let compressed_size = snappy::compress_into(&raw_data, &mut self.snappy_buffer);
		let compressed = &self.snappy_buffer[..compressed_size];
		let hash = keccak(&compressed);

		self.writer.lock().write_state_chunk(hash, compressed)?;
		trace!(target: "snapshot", "Thread {} wrote state chunk. size: {}, uncompressed size: {}", self.thread_idx, compressed_size, raw_data.len());

		self.progress.accounts.fetch_add(num_entries, Ordering::SeqCst);
		self.progress.size.fetch_add(compressed_size as u64, Ordering::SeqCst);

		self.hashes.push(hash);
		self.cur_size = 0;

		Ok(())
	}

	// Get current chunk size.
	fn chunk_size(&self) -> usize {
		self.cur_size
	}
}

/// Walk the given state database starting from the given root,
/// creating chunks and writing them out.
/// `part` is a number between 0 and 15, which describe which part of
/// the tree should be chunked.
///
/// Returns a list of hashes of chunks created, or any error it may
/// have encountered.
pub fn chunk_state<'a>(
	db: &dyn HashDB<KeccakHasher, DBValue>,
	root: &H256,
	writer: &Mutex<dyn SnapshotWriter + 'a>,
	progress: &'a Progress,
	part: Option<usize>,
	thread_idx: usize,
) -> Result<Vec<H256>, Error> {
	let account_trie = TrieDB::new(&db, &root)?;

	let mut chunker = StateChunker {
		hashes: Vec::new(),
		rlps: Vec::new(),
		cur_size: 0,
		snappy_buffer: vec![0; snappy::max_compressed_len(PREFERRED_CHUNK_SIZE)],
		writer,
		progress,
		thread_idx,
	};

	let mut used_code = HashSet::new();

	// account_key here is the address' hash.
	let mut account_iter = account_trie.iter()?;

	let mut seek_to = None;

	if let Some(part) = part {
		assert!(part < 16, "Wrong chunk state part number (must be <16) in snapshot creation.");

		let part_offset = MAX_SNAPSHOT_SUBPARTS / SNAPSHOT_SUBPARTS;
		let mut seek_from = vec![0; 32];
		seek_from[0] = (part * part_offset) as u8;
		account_iter.seek(&seek_from)?;

		// Set the upper-bound, except for the last part
		if part < SNAPSHOT_SUBPARTS - 1 {
			seek_to = Some(((part + 1) * part_offset) as u8)
		}
	}

	for item in account_iter {
		let (account_key, account_data) = item?;
		let account_key_hash = H256::from_slice(&account_key);

		if seek_to.map_or(false, |seek_to| account_key[0] >= seek_to) {
			break;
		}

		let account = ::rlp::decode(&*account_data)?;
		let account_db = AccountDB::from_hash(db, account_key_hash);

		let fat_rlps = account::to_fat_rlps(&account_key_hash, &account, &account_db, &mut used_code, PREFERRED_CHUNK_SIZE - chunker.chunk_size(), PREFERRED_CHUNK_SIZE, progress)?;
		for (i, fat_rlp) in fat_rlps.into_iter().enumerate() {
			if i > 0 {
				chunker.write_chunk()?;
			}
			chunker.push(fat_rlp)?;
		}
	}

	if chunker.cur_size != 0 {
		chunker.write_chunk()?;
	}

	Ok(chunker.hashes)
}

/// Used to rebuild the state trie piece by piece.
pub struct StateRebuilder {
	db: Box<dyn JournalDB>,
	state_root: H256,
	known_code: HashMap<H256, H256>, // code hashes mapped to first account with this code.
	missing_code: HashMap<H256, Vec<H256>>, // maps code hashes to lists of accounts missing that code.
	bloom: Bloom,
	known_storage_roots: HashMap<H256, H256>, // maps account hashes to last known storage root. Only filled for last account per chunk.
}

impl StateRebuilder {
	/// Create a new state rebuilder to write into the given backing DB.
	pub fn new(db: Arc<dyn KeyValueDB>, pruning: Algorithm) -> Self {
		StateRebuilder {
			db: journaldb::new(db.clone(), pruning, ::db::COL_STATE),
			state_root: KECCAK_NULL_RLP,
			known_code: HashMap::new(),
			missing_code: HashMap::new(),
			bloom: StateDB::load_bloom(&*db),
			known_storage_roots: HashMap::new(),
		}
	}

	/// Feed an uncompressed state chunk into the rebuilder.
	pub fn feed(&mut self, chunk: &[u8], flag: &AtomicBool) -> Result<(), EthcoreError> {
		let rlp = Rlp::new(chunk);
		let empty_rlp = StateAccount::new_basic(U256::zero(), U256::zero()).rlp();
		let mut pairs = Vec::with_capacity(rlp.item_count()?);

		// initialize the pairs vector with empty values so we have slots to write into.
		pairs.resize(rlp.item_count()?, (H256::zero(), Vec::new()));

		let status = rebuild_accounts(
			self.db.as_hash_db_mut(),
			rlp,
			&mut pairs,
			&self.known_code,
			&mut self.known_storage_roots,
			flag
		)?;

		for (addr_hash, code_hash) in status.missing_code {
			self.missing_code.entry(code_hash).or_insert_with(Vec::new).push(addr_hash);
		}

		// patch up all missing code. must be done after collecting all new missing code entries.
		for (code_hash, code, first_with) in status.new_code {
			for addr_hash in self.missing_code.remove(&code_hash).unwrap_or_else(Vec::new) {
				let mut db = AccountDBMut::from_hash(self.db.as_hash_db_mut(), addr_hash);
				db.emplace(code_hash, hash_db::EMPTY_PREFIX, DBValue::from_slice(&code));
			}

			self.known_code.insert(code_hash, first_with);
		}

		let backing = self.db.backing().clone();

		// batch trie writes
		{
			let mut account_trie = if self.state_root != KECCAK_NULL_RLP {
				TrieDBMut::from_existing(self.db.as_hash_db_mut(), &mut self.state_root)?
			} else {
				TrieDBMut::new(self.db.as_hash_db_mut(), &mut self.state_root)
			};

			for (hash, thin_rlp) in pairs {
				if !flag.load(Ordering::SeqCst) { return Err(Error::RestorationAborted.into()) }

				if &thin_rlp[..] != &empty_rlp[..] {
					self.bloom.set(hash.as_bytes());
				}
				account_trie.insert(hash.as_bytes(), &thin_rlp)?;
			}
		}

		let bloom_journal = self.bloom.drain_journal();
		let mut batch = backing.transaction();
		StateDB::commit_bloom(&mut batch, bloom_journal)?;
		self.db.inject(&mut batch)?;
		backing.write_buffered(batch);
		trace!(target: "snapshot", "current state root: {:?}", self.state_root);
		Ok(())
	}

	/// Finalize the restoration. Check for accounts missing code and make a dummy
	/// journal entry.
	/// Once all chunks have been fed, there should be nothing missing.
	pub fn finalize(mut self, era: u64, id: H256) -> Result<Box<dyn JournalDB>, EthcoreError> {
		let missing = self.missing_code.keys().cloned().collect::<Vec<_>>();
		if !missing.is_empty() { return Err(Error::MissingCode(missing).into()) }

		let mut batch = self.db.backing().transaction();
		self.db.journal_under(&mut batch, era, &id)?;
		self.db.backing().write_buffered(batch);

		Ok(self.db)
	}

	/// Get the state root of the rebuilder.
	pub fn state_root(&self) -> H256 { self.state_root }
}

#[derive(Default)]
struct RebuiltStatus {
	// new code that's become available. (code_hash, code, addr_hash)
	new_code: Vec<(H256, Bytes, H256)>,
	missing_code: Vec<(H256, H256)>, // accounts that are missing code.
}

// rebuild a set of accounts and their storage.
// returns a status detailing newly-loaded code and accounts missing code.
fn rebuild_accounts(
	db: &mut dyn HashDB<KeccakHasher, DBValue>,
	account_fat_rlps: Rlp,
	out_chunk: &mut [(H256, Bytes)],
	known_code: &HashMap<H256, H256>,
	known_storage_roots: &mut HashMap<H256, H256>,
	abort_flag: &AtomicBool,
) -> Result<RebuiltStatus, EthcoreError> {
	let mut status = RebuiltStatus::default();
	for (account_rlp, out) in account_fat_rlps.into_iter().zip(out_chunk.iter_mut()) {
		if !abort_flag.load(Ordering::SeqCst) { return Err(Error::RestorationAborted.into()) }

		let hash: H256 = account_rlp.val_at(0)?;
		let fat_rlp = account_rlp.at(1)?;

		let thin_rlp = {

			// fill out the storage trie and code while decoding.
			let (acc, maybe_code) = {
				let mut acct_db = AccountDBMut::from_hash(db, hash);
				let storage_root = known_storage_roots.get(&hash).cloned().unwrap_or_default();
				account::from_fat_rlp(&mut acct_db, fat_rlp, storage_root)?
			};

			let code_hash = acc.code_hash.clone();
			match maybe_code {
				// new inline code
				Some(code) => status.new_code.push((code_hash, code, hash)),
				None => {
					if code_hash != KECCAK_EMPTY {
						// see if this code has already been included inline
						match known_code.get(&code_hash) {
							Some(&first_with) => {
								// if so, load it from the database.
								let code = AccountDB::from_hash(db, first_with)
									.get(&code_hash, hash_db::EMPTY_PREFIX)
									.ok_or_else(|| Error::MissingCode(vec![first_with]))?;

								// and write it again under a different mangled key
								AccountDBMut::from_hash(db, hash).emplace(code_hash, hash_db::EMPTY_PREFIX, code);
							}
							// if not, queue it up to be filled later
							None => status.missing_code.push((hash, code_hash)),
						}
					}
				}
			}

			::rlp::encode(&acc)
		};

		*out = (hash, thin_rlp);
	}
	if let Some(&(ref hash, ref rlp)) = out_chunk.iter().last() {
		known_storage_roots.insert(*hash, ::rlp::decode::<BasicAccount>(rlp)?.storage_root);
	}
	if let Some(&(ref hash, ref rlp)) = out_chunk.iter().next() {
		known_storage_roots.insert(*hash, ::rlp::decode::<BasicAccount>(rlp)?.storage_root);
	}
	Ok(status)
}

/// Proportion of blocks which we will verify `PoW` for.
const POW_VERIFY_RATE: f32 = 0.02;

/// Verify an old block with the given header, engine, blockchain, body. If `always` is set, it will perform
/// the fullest verification possible. If not, it will take a random sample to determine whether it will
/// do heavy or light verification.
pub fn verify_old_block(rng: &mut OsRng, header: &Header, engine: &dyn Engine, chain: &BlockChain, always: bool) -> Result<(), EthcoreError> {
	engine.verify_block_basic(header)?;

	if always || rng.gen::<f32>() <= POW_VERIFY_RATE {
		engine.verify_block_unordered(header)?;
		match chain.block_header_data(header.parent_hash()) {
			Some(parent) => engine.verify_block_family(header, &parent.decode()?).map_err(Into::into),
			None => Ok(()),
		}
	} else {
		Ok(())
	}
}
