// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Snapshot creation, restoration, and network service.
//!
//! Documentation of the format can be found at
//! https://github.com/ethcore/parity/wiki/%22PV64%22-Snapshot-Format

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use account_db::{AccountDB, AccountDBMut};
use blockchain::{BlockChain, BlockProvider};
use engines::Engine;
use header::Header;
use ids::BlockId;
use views::BlockView;

use util::{Bytes, Hashable, HashDB, DBValue, snappy, U256, Uint};
use util::Mutex;
use util::hash::{FixedHash, H256};
use util::journaldb::{self, Algorithm, JournalDB};
use util::kvdb::Database;
use util::trie::{TrieDB, TrieDBMut, Trie, TrieMut};
use util::sha3::SHA3_NULL_RLP;
use rlp::{RlpStream, Stream, UntrustedRlp, View};
use bloom_journal::Bloom;

use self::block::AbridgedBlock;
use self::io::SnapshotWriter;

use super::state_db::StateDB;
use super::state::Account as StateAccount;

use crossbeam::scope;
use rand::{Rng, OsRng};

pub use self::error::Error;

pub use self::service::{Service, DatabaseRestore};
pub use self::traits::SnapshotService;
pub use self::watcher::Watcher;
pub use types::snapshot_manifest::ManifestData;
pub use types::restoration_status::RestorationStatus;

pub mod io;
pub mod service;

mod account;
mod block;
mod error;
mod watcher;

#[cfg(test)]
mod tests;

/// IPC interfaces
#[cfg(feature="ipc")]
pub mod remote {
	pub use super::traits::RemoteSnapshotService;
}

mod traits {
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/snapshot_service_trait.rs"));
}

// Try to have chunks be around 4MB (before compression)
const PREFERRED_CHUNK_SIZE: usize = 4 * 1024 * 1024;

// How many blocks to include in a snapshot, starting from the head of the chain.
const SNAPSHOT_BLOCKS: u64 = 30000;

/// A progress indicator for snapshots.
#[derive(Debug, Default)]
pub struct Progress {
	accounts: AtomicUsize,
	blocks: AtomicUsize,
	size: AtomicUsize, // Todo [rob] use Atomicu64 when it stabilizes.
	done: AtomicBool,
}

impl Progress {
	/// Reset the progress.
	pub fn reset(&self) {
		self.accounts.store(0, Ordering::Release);
		self.blocks.store(0, Ordering::Release);
		self.size.store(0, Ordering::Release);

		// atomic fence here to ensure the others are written first?
		// logs might very rarely get polluted if not.
		self.done.store(false, Ordering::Release);
	}

	/// Get the number of accounts snapshotted thus far.
	pub fn accounts(&self) -> usize { self.accounts.load(Ordering::Acquire) }

	/// Get the number of blocks snapshotted thus far.
	pub fn blocks(&self) -> usize { self.blocks.load(Ordering::Acquire) }

	/// Get the written size of the snapshot in bytes.
	pub fn size(&self) -> usize { self.size.load(Ordering::Acquire) }

	/// Whether the snapshot is complete.
	pub fn done(&self) -> bool  { self.done.load(Ordering::Acquire) }

}
/// Take a snapshot using the given blockchain, starting block hash, and database, writing into the given writer.
pub fn take_snapshot<W: SnapshotWriter + Send>(
	chain: &BlockChain,
	block_at: H256,
	state_db: &HashDB,
	writer: W,
	p: &Progress
) -> Result<(), Error> {
	let start_header = chain.block_header(&block_at)
		.ok_or(Error::InvalidStartingBlock(BlockId::Hash(block_at)))?;
	let state_root = start_header.state_root();
	let number = start_header.number();

	info!("Taking snapshot starting at block {}", number);

	let writer = Mutex::new(writer);
	let (state_hashes, block_hashes) = scope(|scope| {
		let block_guard = scope.spawn(|| chunk_blocks(chain, block_at, &writer, p));
		let state_res = chunk_state(state_db, state_root, &writer, p);

		state_res.and_then(|state_hashes| {
			block_guard.join().map(|block_hashes| (state_hashes, block_hashes))
		})
	})?;

	info!("produced {} state chunks and {} block chunks.", state_hashes.len(), block_hashes.len());

	let manifest_data = ManifestData {
		state_hashes: state_hashes,
		block_hashes: block_hashes,
		state_root: *state_root,
		block_number: number,
		block_hash: block_at,
	};

	writer.into_inner().finish(manifest_data)?;

	p.done.store(true, Ordering::SeqCst);

	Ok(())
}

/// Used to build block chunks.
struct BlockChunker<'a> {
	chain: &'a BlockChain,
	// block, receipt rlp pairs.
	rlps: VecDeque<Bytes>,
	current_hash: H256,
	hashes: Vec<H256>,
	snappy_buffer: Vec<u8>,
	writer: &'a Mutex<SnapshotWriter + 'a>,
	progress: &'a Progress,
}

impl<'a> BlockChunker<'a> {
	// Repeatedly fill the buffers and writes out chunks, moving backwards from starting block hash.
	// Loops until we reach the first desired block, and writes out the remainder.
	fn chunk_all(&mut self) -> Result<(), Error> {
		let mut loaded_size = 0;
		let mut last = self.current_hash;

		let genesis_hash = self.chain.genesis_hash();

		for _ in 0..SNAPSHOT_BLOCKS {
			if self.current_hash == genesis_hash { break }

			let (block, receipts) = self.chain.block(&self.current_hash)
				.and_then(|b| self.chain.block_receipts(&self.current_hash).map(|r| (b, r)))
				.ok_or(Error::BlockNotFound(self.current_hash))?;

			let abridged_rlp = AbridgedBlock::from_block_view(&block.view()).into_inner();

			let pair = {
				let mut pair_stream = RlpStream::new_list(2);
				pair_stream.append_raw(&abridged_rlp, 1).append(&receipts);
				pair_stream.out()
			};

			let new_loaded_size = loaded_size + pair.len();

			// cut off the chunk if too large.

			if new_loaded_size > PREFERRED_CHUNK_SIZE && !self.rlps.is_empty() {
				self.write_chunk(last)?;
				loaded_size = pair.len();
			} else {
				loaded_size = new_loaded_size;
			}

			self.rlps.push_front(pair);

			last = self.current_hash;
			self.current_hash = block.header_view().parent_hash();
		}

		if loaded_size != 0 {
			self.write_chunk(last)?;
		}

		Ok(())
	}

	// write out the data in the buffers to a chunk on disk
	//
	// we preface each chunk with the parent of the first block's details,
	// obtained from the details of the last block written.
	fn write_chunk(&mut self, last: H256) -> Result<(), Error> {
		trace!(target: "snapshot", "prepared block chunk with {} blocks", self.rlps.len());

		let (last_header, last_details) = self.chain.block_header(&last)
			.and_then(|n| self.chain.block_details(&last).map(|d| (n, d)))
			.ok_or(Error::BlockNotFound(last))?;

		let parent_number = last_header.number() - 1;
		let parent_hash = last_header.parent_hash();
		let parent_total_difficulty = last_details.total_difficulty - *last_header.difficulty();

		trace!(target: "snapshot", "parent last written block: {}", parent_hash);

		let num_entries = self.rlps.len();
		let mut rlp_stream = RlpStream::new_list(3 + num_entries);
		rlp_stream.append(&parent_number).append(parent_hash).append(&parent_total_difficulty);

		for pair in self.rlps.drain(..) {
			rlp_stream.append_raw(&pair, 1);
		}

		let raw_data = rlp_stream.out();

		let size = snappy::compress_into(&raw_data, &mut self.snappy_buffer);
		let compressed = &self.snappy_buffer[..size];
		let hash = compressed.sha3();

		self.writer.lock().write_block_chunk(hash, compressed)?;
		trace!(target: "snapshot", "wrote block chunk. hash: {}, size: {}, uncompressed size: {}", hash.hex(), size, raw_data.len());

		self.progress.size.fetch_add(size, Ordering::SeqCst);
		self.progress.blocks.fetch_add(num_entries, Ordering::SeqCst);

		self.hashes.push(hash);
		Ok(())
	}
}

/// Create and write out all block chunks to disk, returning a vector of all
/// the hashes of block chunks created.
///
/// The path parameter is the directory to store the block chunks in.
/// This function assumes the directory exists already.
/// Returns a list of chunk hashes, with the first having the blocks furthest from the genesis.
pub fn chunk_blocks<'a>(chain: &'a BlockChain, start_hash: H256, writer: &Mutex<SnapshotWriter + 'a>, progress: &'a Progress) -> Result<Vec<H256>, Error> {
	let mut chunker = BlockChunker {
		chain: chain,
		rlps: VecDeque::new(),
		current_hash: start_hash,
		hashes: Vec::new(),
		snappy_buffer: vec![0; snappy::max_compressed_len(PREFERRED_CHUNK_SIZE)],
		writer: writer,
		progress: progress,
	};

	chunker.chunk_all()?;

	Ok(chunker.hashes)
}

/// State trie chunker.
struct StateChunker<'a> {
	hashes: Vec<H256>,
	rlps: Vec<Bytes>,
	cur_size: usize,
	snappy_buffer: Vec<u8>,
	writer: &'a Mutex<SnapshotWriter + 'a>,
	progress: &'a Progress,
}

impl<'a> StateChunker<'a> {
	// Push a key, value pair to be encoded.
	//
	// If the buffer is greater than the desired chunk size,
	// this will write out the data to disk.
	fn push(&mut self, account_hash: Bytes, data: Bytes) -> Result<(), Error> {
		let pair = {
			let mut stream = RlpStream::new_list(2);
			stream.append(&account_hash).append_raw(&data, 1);
			stream.out()
		};

		if self.cur_size + pair.len() >= PREFERRED_CHUNK_SIZE {
			self.write_chunk()?;
		}

		self.cur_size += pair.len();
		self.rlps.push(pair);

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
		let hash = compressed.sha3();

		self.writer.lock().write_state_chunk(hash, compressed)?;
		trace!(target: "snapshot", "wrote state chunk. size: {}, uncompressed size: {}", compressed_size, raw_data.len());

		self.progress.accounts.fetch_add(num_entries, Ordering::SeqCst);
		self.progress.size.fetch_add(compressed_size, Ordering::SeqCst);

		self.hashes.push(hash);
		self.cur_size = 0;

		Ok(())
	}
}

/// Walk the given state database starting from the given root,
/// creating chunks and writing them out.
///
/// Returns a list of hashes of chunks created, or any error it may
/// have encountered.
pub fn chunk_state<'a>(db: &HashDB, root: &H256, writer: &Mutex<SnapshotWriter + 'a>, progress: &'a Progress) -> Result<Vec<H256>, Error> {
	let account_trie = TrieDB::new(db, &root)?;

	let mut chunker = StateChunker {
		hashes: Vec::new(),
		rlps: Vec::new(),
		cur_size: 0,
		snappy_buffer: vec![0; snappy::max_compressed_len(PREFERRED_CHUNK_SIZE)],
		writer: writer,
		progress: progress,
	};

	let mut used_code = HashSet::new();

	// account_key here is the address' hash.
	for item in account_trie.iter()? {
		let (account_key, account_data) = item?;
		let account = ::rlp::decode(&*account_data);
		let account_key_hash = H256::from_slice(&account_key);

		let account_db = AccountDB::from_hash(db, account_key_hash);

		let fat_rlp = account::to_fat_rlp(&account, &account_db, &mut used_code)?;
		chunker.push(account_key, fat_rlp)?;
	}

	if chunker.cur_size != 0 {
		chunker.write_chunk()?;
	}

	Ok(chunker.hashes)
}

/// Used to rebuild the state trie piece by piece.
pub struct StateRebuilder {
	db: Box<JournalDB>,
	state_root: H256,
	known_code: HashMap<H256, H256>, // code hashes mapped to first account with this code.
	missing_code: HashMap<H256, Vec<H256>>, // maps code hashes to lists of accounts missing that code.
	bloom: Bloom,
}

impl StateRebuilder {
	/// Create a new state rebuilder to write into the given backing DB.
	pub fn new(db: Arc<Database>, pruning: Algorithm) -> Self {
		StateRebuilder {
			db: journaldb::new(db.clone(), pruning, ::db::COL_STATE),
			state_root: SHA3_NULL_RLP,
			known_code: HashMap::new(),
			missing_code: HashMap::new(),
			bloom: StateDB::load_bloom(&*db),
		}
	}

	/// Feed an uncompressed state chunk into the rebuilder.
	pub fn feed(&mut self, chunk: &[u8], flag: &AtomicBool) -> Result<(), ::error::Error> {
		let rlp = UntrustedRlp::new(chunk);
		let empty_rlp = StateAccount::new_basic(U256::zero(), U256::zero()).rlp();
		let mut pairs = Vec::with_capacity(rlp.item_count());

		// initialize the pairs vector with empty values so we have slots to write into.
		pairs.resize(rlp.item_count(), (H256::new(), Vec::new()));

		let status = rebuild_accounts(
			self.db.as_hashdb_mut(),
			rlp,
			&mut pairs,
			&self.known_code,
			flag
		)?;

		for (addr_hash, code_hash) in status.missing_code {
			self.missing_code.entry(code_hash).or_insert_with(Vec::new).push(addr_hash);
		}

		// patch up all missing code. must be done after collecting all new missing code entries.
		for (code_hash, code, first_with) in status.new_code {
			for addr_hash in self.missing_code.remove(&code_hash).unwrap_or_else(Vec::new) {
				let mut db = AccountDBMut::from_hash(self.db.as_hashdb_mut(), addr_hash);
				db.emplace(code_hash, DBValue::from_slice(&code));
			}

			self.known_code.insert(code_hash, first_with);
		}

		let backing = self.db.backing().clone();

		// batch trie writes
		{
			let mut account_trie = if self.state_root != SHA3_NULL_RLP {
				TrieDBMut::from_existing(self.db.as_hashdb_mut(), &mut self.state_root)?
			} else {
				TrieDBMut::new(self.db.as_hashdb_mut(), &mut self.state_root)
			};

			for (hash, thin_rlp) in pairs {
				if !flag.load(Ordering::SeqCst) { return Err(Error::RestorationAborted.into()) }

				if &thin_rlp[..] != &empty_rlp[..] {
					self.bloom.set(&*hash);
				}
				account_trie.insert(&hash, &thin_rlp)?;
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

	/// Check for accounts missing code. Once all chunks have been fed, there should
	/// be none.
	pub fn check_missing(self) -> Result<(), Error> {
		let missing = self.missing_code.keys().cloned().collect::<Vec<_>>();
		match missing.is_empty() {
			true => Ok(()),
			false => Err(Error::MissingCode(missing)),
		}
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
	db: &mut HashDB,
	account_fat_rlps: UntrustedRlp,
	out_chunk: &mut [(H256, Bytes)],
	known_code: &HashMap<H256, H256>,
	abort_flag: &AtomicBool,
) -> Result<RebuiltStatus, ::error::Error> {
	let mut status = RebuiltStatus::default();
	for (account_rlp, out) in account_fat_rlps.into_iter().zip(out_chunk) {
		if !abort_flag.load(Ordering::SeqCst) { return Err(Error::RestorationAborted.into()) }

		let hash: H256 = account_rlp.val_at(0)?;
		let fat_rlp = account_rlp.at(1)?;

		let thin_rlp = {

			// fill out the storage trie and code while decoding.
			let (acc, maybe_code) = {
				let mut acct_db = AccountDBMut::from_hash(db, hash);
				account::from_fat_rlp(&mut acct_db, fat_rlp)?
			};

			let code_hash = acc.code_hash.clone();
			match maybe_code {
				// new inline code
				Some(code) => status.new_code.push((code_hash, code, hash)),
				None => {
					if code_hash != ::util::SHA3_EMPTY {
						// see if this code has already been included inline
						match known_code.get(&code_hash) {
							Some(&first_with) => {
								// if so, load it from the database.
								let code = AccountDB::from_hash(db, first_with)
									.get(&code_hash)
									.ok_or_else(|| Error::MissingCode(vec![first_with]))?;

								// and write it again under a different mangled key
								AccountDBMut::from_hash(db, hash).emplace(code_hash, code);
							}
							// if not, queue it up to be filled later
							None => status.missing_code.push((hash, code_hash)),
						}
					}
				}
			}

			::rlp::encode(&acc).to_vec()
		};

		*out = (hash, thin_rlp);
	}
	Ok(status)
}

/// Proportion of blocks which we will verify `PoW` for.
const POW_VERIFY_RATE: f32 = 0.02;

/// Verify an old block with the given header, engine, blockchain, body. If `always` is set, it will perform
/// the fullest verification possible. If not, it will take a random sample to determine whether it will
/// do heavy or light verification.
pub fn verify_old_block(rng: &mut OsRng, header: &Header, engine: &Engine, chain: &BlockChain, body: Option<&[u8]>, always: bool) -> Result<(), ::error::Error> {
	if always || rng.gen::<f32>() <= POW_VERIFY_RATE {
		match chain.block_header(header.parent_hash()) {
			Some(parent) => engine.verify_block_family(header, &parent, body),
			None => engine.verify_block_seal(header),
		}
	} else {
		engine.verify_block_basic(header, body)
	}
}

/// Rebuilds the blockchain from chunks.
///
/// Does basic verification for all blocks, but `PoW` verification for some.
/// Blocks must be fed in-order.
///
/// The first block in every chunk is disconnected from the last block in the
/// chunk before it, as chunks may be submitted out-of-order.
///
/// After all chunks have been submitted, we "glue" the chunks together.
pub struct BlockRebuilder {
	chain: BlockChain,
	db: Arc<Database>,
	rng: OsRng,
	disconnected: Vec<(u64, H256)>,
	best_number: u64,
	best_hash: H256,
	best_root: H256,
	fed_blocks: u64,
}

impl BlockRebuilder {
	/// Create a new BlockRebuilder.
	pub fn new(chain: BlockChain, db: Arc<Database>, manifest: &ManifestData) -> Result<Self, ::error::Error> {
		Ok(BlockRebuilder {
			chain: chain,
			db: db,
			rng: OsRng::new()?,
			disconnected: Vec::new(),
			best_number: manifest.block_number,
			best_hash: manifest.block_hash,
			best_root: manifest.state_root,
			fed_blocks: 0,
		})
	}

	/// Feed the rebuilder an uncompressed block chunk.
	/// Returns the number of blocks fed or any errors.
	pub fn feed(&mut self, chunk: &[u8], engine: &Engine, abort_flag: &AtomicBool) -> Result<u64, ::error::Error> {
		use basic_types::Seal::With;
		use util::U256;
		use util::triehash::ordered_trie_root;

		let rlp = UntrustedRlp::new(chunk);
		let item_count = rlp.item_count();
		let num_blocks = (item_count - 3) as u64;

		trace!(target: "snapshot", "restoring block chunk with {} blocks.", item_count - 3);

		if self.fed_blocks + num_blocks > SNAPSHOT_BLOCKS {
			return Err(Error::TooManyBlocks(SNAPSHOT_BLOCKS, self.fed_blocks).into())
		}

		// todo: assert here that these values are consistent with chunks being in order.
		let mut cur_number = rlp.val_at::<u64>(0)? + 1;
		let mut parent_hash = rlp.val_at::<H256>(1)?;
		let parent_total_difficulty = rlp.val_at::<U256>(2)?;

		for idx in 3..item_count {
			if !abort_flag.load(Ordering::SeqCst) { return Err(Error::RestorationAborted.into()) }

			let pair = rlp.at(idx)?;
			let abridged_rlp = pair.at(0)?.as_raw().to_owned();
			let abridged_block = AbridgedBlock::from_raw(abridged_rlp);
			let receipts: Vec<::receipt::Receipt> = pair.val_at(1)?;
			let receipts_root = ordered_trie_root(
				pair.at(1)?.iter().map(|r| r.as_raw().to_owned())
			);

			let block = abridged_block.to_block(parent_hash, cur_number, receipts_root)?;
			let block_bytes = block.rlp_bytes(With);
			let is_best = cur_number == self.best_number;

			if is_best {
				if block.header.hash() != self.best_hash {
					return Err(Error::WrongBlockHash(cur_number, self.best_hash, block.header.hash()).into())
				}

				if block.header.state_root() != &self.best_root {
					return Err(Error::WrongStateRoot(self.best_root, *block.header.state_root()).into())
				}
			}

			verify_old_block(
				&mut self.rng,
				&block.header,
				engine,
				&self.chain,
				Some(&block_bytes),
				is_best
			)?;

			let mut batch = self.db.transaction();

			// special-case the first block in each chunk.
			if idx == 3 {
				if self.chain.insert_unordered_block(&mut batch, &block_bytes, receipts, Some(parent_total_difficulty), is_best, false) {
					self.disconnected.push((cur_number, block.header.hash()));
				}
			} else {
				self.chain.insert_unordered_block(&mut batch, &block_bytes, receipts, None, is_best, false);
			}
			self.db.write_buffered(batch);
			self.chain.commit();

			parent_hash = BlockView::new(&block_bytes).hash();
			cur_number += 1;
		}

		self.fed_blocks += num_blocks;

		Ok(num_blocks)
	}

	/// Glue together any disconnected chunks and check that the chain is complete.
	pub fn finalize(self, canonical: HashMap<u64, H256>) -> Result<(), Error> {
		let mut batch = self.db.transaction();

		for (first_num, first_hash) in self.disconnected {
			let parent_num = first_num - 1;

			// check if the parent is even in the chain.
			// since we don't restore every single block in the chain,
			// the first block of the first chunks has nothing to connect to.
			if let Some(parent_hash) = self.chain.block_hash(parent_num) {
				// if so, add the child to it.
				self.chain.add_child(&mut batch, parent_hash, first_hash);
			}
		}
		self.db.write_buffered(batch);

		let best_number = self.best_number;
		for num in (0..self.fed_blocks).map(|x| best_number - x) {

			let hash = self.chain.block_hash(num).ok_or(Error::IncompleteChain)?;

			if let Some(canon_hash) = canonical.get(&num).cloned() {
				if canon_hash != hash {
					return Err(Error::WrongBlockHash(num, canon_hash, hash));
				}
			}
		}

		Ok(())
	}
}
