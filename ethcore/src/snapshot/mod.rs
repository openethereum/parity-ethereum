// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::collections::VecDeque;

use account_db::{AccountDB, AccountDBMut};
use blockchain::BlockChain;
use client::BlockChainClient;
use error::Error;
use engine::Engine;
use ids::BlockID;
use views::{BlockView, HeaderView};

use util::{Bytes, Hashable, HashDB, JournalDB, snappy, TrieDB, TrieDBMut, TrieMut};
use util::hash::{FixedHash, H256};
use util::rlp::{DecoderError, RlpStream, Stream, UntrustedRlp, View};

use self::account::Account;
use self::block::AbridgedBlock;
use self::io::SnapshotWriter;

use crossbeam::{scope, ScopedJoinHandle};
use rand::{Rng, OsRng};

pub use self::service::Service;

pub mod io;
pub mod service;

mod account;
mod block;

// Try to have chunks be around 16MB (before compression)
const PREFERRED_CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Statuses for restorations.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum RestorationStatus {
	///	No restoration.
	Inactive,
	/// Ongoing restoration. Number of blocks restored and number of state chunks restored.
	Ongoing(u64, usize),
	/// Failed restoration.
	Failed,
}

/// The interface for a snapshot network service.
/// This handles:
///    - restoration of snapshots to temporary databases.
///    - responding to queries for snapshot manifests and chunks
pub trait SnapshotService {
	/// Query the most recent manifest data.
	fn manifest(&self) -> Option<ManifestData>;

	/// Get raw chunk for a given hash.
	fn chunk(&self, hash: H256) -> Option<Bytes>;

	/// Ask the snapshot service for the restoration status.
	fn status(&self) -> RestorationStatus;

	/// Begin snapshot restoration.
	/// If restoration in-progress, this will reset it.
	/// From this point on, any previous snapshot may become unavailable.
	/// Returns true if successful, false otherwise.
	fn begin_restore(&self, manifest: ManifestData) -> bool;

	/// Feed a raw state chunk to the service to be processed asynchronously.
	/// no-op if not currently restoring.
	fn restore_state_chunk(&self, hash: H256, chunk: Bytes);

	/// Feed a raw block chunk to the service to be processed asynchronously.
	/// no-op if currently restoring.
	fn restore_block_chunk(&self, hash: H256, chunk: Bytes);
}

/// Take a snapshot using the given client and database, writing into the given writer.
pub fn take_snapshot<W: SnapshotWriter>(client: &BlockChainClient, state_db: &HashDB, mut writer: W) -> Result<(), Error> {
	let chain_info = client.chain_info();

	let genesis_hash = chain_info.genesis_hash;
	let best_header_raw = client.best_block_header();
	let best_header = HeaderView::new(&best_header_raw);
	let state_root = best_header.state_root();

	info!("Taking snapshot starting at block {}", best_header.number());

	let state_hashes = try!(chunk_state(state_db, &state_root, &mut writer));
	let block_hashes = try!(chunk_blocks(client, best_header.hash(), genesis_hash, &mut writer));

	info!("produced {} state chunks and {} block chunks.", state_hashes.len(), block_hashes.len());

	let manifest_data = ManifestData {
		state_hashes: state_hashes,
		block_hashes: block_hashes.into_iter().rev().collect(),
		state_root: state_root,
		block_number: chain_info.best_block_number,
		block_hash: chain_info.best_block_hash,
	};

	try!(writer.finish(manifest_data));

	Ok(())
}

/// Used to build block chunks.
struct BlockChunker<'a> {
	client: &'a BlockChainClient,
	// block, receipt rlp pairs.
	rlps: VecDeque<Bytes>,
	current_hash: H256,
	hashes: Vec<H256>,
	snappy_buffer: Vec<u8>,
	writer: &'a mut SnapshotWriter,
}

impl<'a> BlockChunker<'a> {
	// Repeatedly fill the buffers and writes out chunks, moving backwards from starting block hash.
	// Loops until we reach the genesis, and writes out the remainder.
	fn chunk_all(&mut self, genesis_hash: H256) -> Result<(), Error> {
		let mut loaded_size = 0;

		while self.current_hash != genesis_hash {
			let block = self.client.block(BlockID::Hash(self.current_hash))
				.expect("started from the head of chain and walking backwards; client stores full chain; qed");
			let view = BlockView::new(&block);
			let abridged_rlp = AbridgedBlock::from_block_view(&view).into_inner();

			let receipts = self.client.block_receipts(&self.current_hash)
				.expect("started from head of chain and walking backwards; client stores full chain; qed");

			let pair = {
				let mut pair_stream = RlpStream::new_list(2);
				pair_stream.append_raw(&abridged_rlp, 1).append(&receipts);
				pair_stream.out()
			};

			let new_loaded_size = loaded_size + pair.len();

			// cut off the chunk if too large
			if new_loaded_size > PREFERRED_CHUNK_SIZE {
				let header = view.header_view();
				let parent_hash = header.parent_hash();

				try!(self.write_chunk(parent_hash));
				loaded_size = pair.len();
			} else {
				loaded_size = new_loaded_size;
			}

			self.rlps.push_front(pair);
			self.current_hash = view.header_view().parent_hash();
		}

		if loaded_size != 0 {
			// we don't store the genesis block, so once we get to this point,
			// the "first" block will be number 1.
			try!(self.write_chunk(genesis_hash));
		}

		Ok(())
	}

	// write out the data in the buffers to a chunk on disk
	//
	// we preface each chunk with the parent of the first block's details.
	fn write_chunk(&mut self, parent_hash: H256) -> Result<(), Error> {
		trace!(target: "snapshot", "prepared block chunk with {} blocks", self.rlps.len());
		let parent_id = BlockID::Hash(parent_hash);
		let parent_header_bytes = self.client.block_header(parent_id.clone())
			.expect("parent hash either obtained from other block header or is genesis; qed");

		let parent_header = HeaderView::new(&parent_header_bytes);
		let parent_number = parent_header.number();

		let mut rlp_stream = RlpStream::new_list(2 + self.rlps.len());
		rlp_stream.append(&parent_number).append(&parent_hash);

		for pair in self.rlps.drain(..) {
			rlp_stream.append_raw(&pair, 1);
		}

		let raw_data = rlp_stream.out();

		let size = snappy::compress_into(&raw_data, &mut self.snappy_buffer);
		let compressed = &self.snappy_buffer[..size];
		let hash = compressed.sha3();

		try!(self.writer.write_block_chunk(hash, compressed));
		trace!(target: "snapshot", "wrote block chunk. hash: {}, size: {}, uncompressed size: {}", hash.hex(), size, raw_data.len());

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
pub fn chunk_blocks(client: &BlockChainClient, best_block_hash: H256, genesis_hash: H256, writer: &mut SnapshotWriter) -> Result<Vec<H256>, Error> {
	let mut chunker = BlockChunker {
		client: client,
		rlps: VecDeque::new(),
		current_hash: best_block_hash,
		hashes: Vec::new(),
		snappy_buffer: vec![0; snappy::max_compressed_len(PREFERRED_CHUNK_SIZE)],
		writer: writer,
	};

	try!(chunker.chunk_all(genesis_hash));

	Ok(chunker.hashes)
}

/// State trie chunker.
struct StateChunker<'a> {
	hashes: Vec<H256>,
	rlps: Vec<Bytes>,
	cur_size: usize,
	snappy_buffer: Vec<u8>,
	writer: &'a mut SnapshotWriter,
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
			try!(self.write_chunk());
		}

		self.cur_size += pair.len();
		self.rlps.push(pair);

		Ok(())
	}

	// Write out the buffer to disk, pushing the created chunk's hash to
	// the list.
	fn write_chunk(&mut self) -> Result<(), Error> {
		let mut stream = RlpStream::new_list(self.rlps.len());
		for rlp in self.rlps.drain(..) {
			stream.append_raw(&rlp, 1);
		}

		let raw_data = stream.out();

		let compressed_size = snappy::compress_into(&raw_data, &mut self.snappy_buffer);
		let compressed = &self.snappy_buffer[..compressed_size];
		let hash = compressed.sha3();

		try!(self.writer.write_state_chunk(hash, compressed));
		trace!(target: "snapshot", "wrote state chunk. size: {}, uncompressed size: {}", compressed_size, raw_data.len());

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
pub fn chunk_state(db: &HashDB, root: &H256, writer: &mut SnapshotWriter) -> Result<Vec<H256>, Error> {
	let account_trie = try!(TrieDB::new(db, &root));

	let mut chunker = StateChunker {
		hashes: Vec::new(),
		rlps: Vec::new(),
		cur_size: 0,
		snappy_buffer: vec![0; snappy::max_compressed_len(PREFERRED_CHUNK_SIZE)],
		writer: writer,
	};

	trace!(target: "snapshot", "beginning state chunking");

	// account_key here is the address' hash.
	for (account_key, account_data) in account_trie.iter() {
		let account = Account::from_thin_rlp(account_data);
		let account_key_hash = H256::from_slice(&account_key);

		let account_db = AccountDB::from_hash(db, account_key_hash);

		let fat_rlp = try!(account.to_fat_rlp(&account_db));
		try!(chunker.push(account_key, fat_rlp));
	}

	if chunker.cur_size != 0 {
		try!(chunker.write_chunk());
	}

	Ok(chunker.hashes)
}

/// Manifest data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestData {
	/// List of state chunk hashes.
	pub state_hashes: Vec<H256>,
	/// List of block chunk hashes.
	pub block_hashes: Vec<H256>,
	/// The final, expected state root.
	pub state_root: H256,
	/// Block number this snapshot was taken at.
	pub block_number: u64,
	/// Block hash this snapshot was taken at.
	pub block_hash: H256,
}

impl ManifestData {
	/// Encode the manifest data to rlp.
	pub fn to_rlp(&self) -> Bytes {
		let mut stream = RlpStream::new_list(5);
		stream.append(&self.state_hashes);
		stream.append(&self.block_hashes);
		stream.append(&self.state_root);
		stream.append(&self.block_number);
		stream.append(&self.block_hash);

		stream.out()
	}

	/// Try to restore manifest data from raw bytes, interpreted as RLP.
	pub fn from_rlp(raw: &[u8]) -> Result<Self, DecoderError> {
		let decoder = UntrustedRlp::new(raw);

		let state_hashes: Vec<H256> = try!(decoder.val_at(0));
		let block_hashes: Vec<H256> = try!(decoder.val_at(1));
		let state_root: H256 = try!(decoder.val_at(2));
		let block_number: u64 = try!(decoder.val_at(3));
		let block_hash: H256 = try!(decoder.val_at(4));

		Ok(ManifestData {
			state_hashes: state_hashes,
			block_hashes: block_hashes,
			state_root: state_root,
			block_number: block_number,
			block_hash: block_hash,
		})
	}
}

/// Used to rebuild the state trie piece by piece.
pub struct StateRebuilder {
	db: Box<JournalDB>,
	state_root: H256,
}

impl StateRebuilder {
	/// Create a new state rebuilder to write into the given backing DB.
	pub fn new(db: Box<JournalDB>) -> Self {
		StateRebuilder {
			db: db,
			state_root: H256::zero(),
		}
	}

	/// Feed an uncompressed state chunk into the rebuilder.
	pub fn feed(&mut self, chunk: &[u8]) -> Result<(), Error> {
		let rlp = UntrustedRlp::new(chunk);
		let account_fat_rlps: Vec<_> = rlp.iter().map(|r| r.as_raw()).collect();
		let mut pairs = Vec::with_capacity(rlp.item_count());

		// initialize the pairs vector with empty values so we have slots to write into.
		for _ in 0..rlp.item_count() {
			pairs.push((H256::new(), Vec::new()));
		}

		let chunk_size = account_fat_rlps.len() / ::num_cpus::get();

		// build account tries in parallel.
		try!(scope(|scope| {
			let mut handles = Vec::new();
			for (account_chunk, out_pairs_chunk) in account_fat_rlps.chunks(chunk_size).zip(pairs.chunks_mut(chunk_size)) {
				let mut db = self.db.boxed_clone();
				let handle: ScopedJoinHandle<Result<(), Error>> = scope.spawn(move || {
					try!(rebuild_account_trie(db.as_hashdb_mut(), account_chunk, out_pairs_chunk));

					// commit the db changes we made in this thread.
					try!(db.commit(0, &H256::zero(), None));

					Ok(())
				});

				handles.push(handle);
			}

			// see if we got any errors.
			for handle in handles {
				try!(handle.join());
			}

			Ok::<_, Error>(())
		}));

		// batch trie writes
		{
			let mut account_trie = if self.state_root != H256::zero() {
				try!(TrieDBMut::from_existing(self.db.as_hashdb_mut(), &mut self.state_root))
			} else {
				TrieDBMut::new(self.db.as_hashdb_mut(), &mut self.state_root)
			};

			for (hash, thin_rlp) in pairs {
				account_trie.insert(&hash, &thin_rlp);
			}
		}

		try!(self.db.commit(0, &H256::zero(), None));
		Ok(())
	}

	/// Get the state root of the rebuilder.
	pub fn state_root(&self) -> H256 { self.state_root }
}

fn rebuild_account_trie(db: &mut HashDB, account_chunk: &[&[u8]], out_chunk: &mut [(H256, Bytes)]) -> Result<(), Error> {
	for (account_pair, out) in account_chunk.into_iter().zip(out_chunk) {
		let account_rlp = UntrustedRlp::new(account_pair);

		let hash: H256 = try!(account_rlp.val_at(0));
		let fat_rlp = try!(account_rlp.at(1));

		let thin_rlp = {
			let mut acct_db = AccountDBMut::from_hash(db.as_hashdb_mut(), hash);

			// fill out the storage trie and code while decoding.
			let acc = try!(Account::from_fat_rlp(&mut acct_db, fat_rlp));

			acc.to_thin_rlp()
		};

		*out = (hash, thin_rlp);
	}
	Ok(())
}

/// Proportion of blocks which we will verify PoW for.
const POW_VERIFY_RATE: f32 = 0.02;

/// Rebuilds the blockchain from chunks.
///
/// Does basic verification for all blocks, but PoW verification for some.
/// Blocks must be fed in-order.
pub struct BlockRebuilder {
	chain: BlockChain,
	rng: OsRng,
	parent_hash: H256,
	cur_number: u64,
}

impl BlockRebuilder {
	/// Create a new BlockRebuilder.
	pub fn new(chain: BlockChain) -> Result<Self, Error> {
		Ok(BlockRebuilder {
			chain: chain,
			rng: try!(OsRng::new()),
			parent_hash: H256::new(),
			cur_number: 0,
		})
	}

	/// Feed the rebuilder an uncompressed block chunk.
	/// Returns the number of blocks fed or any errors.
	pub fn feed(&mut self, chunk: &[u8], engine: &Engine) -> Result<u64, Error> {
		let rlp = UntrustedRlp::new(chunk);
		let item_count = rlp.item_count();

		trace!(target: "snapshot", "restoring block chunk with {} blocks.", item_count - 2);

		// todo: assert here that these values are consistent with chunks being in order.
		self.cur_number = try!(rlp.val_at::<u64>(0)) + 1;
		self.parent_hash = try!(rlp.val_at::<H256>(1));

		for idx in 2..item_count {
			try!(self.process_rlp_pair(try!(rlp.at(idx)), engine));
		}

		Ok(item_count as u64 - 2)
	}

	fn process_rlp_pair(&mut self, pair: UntrustedRlp, engine: &Engine) -> Result<(), Error> {
		use basic_types::Seal::With;

		trace!(target: "snapshot", "processing block pair");
		let abridged_rlp = try!(pair.at(0)).as_raw().to_owned();
		let abridged_block = AbridgedBlock::from_raw(abridged_rlp);
		let receipts = try!(pair.val_at(1));
		let block = try!(abridged_block.to_block(self.parent_hash, self.cur_number));
		let block_bytes = block.rlp_bytes(With);


		if self.rng.gen::<f32>() <= POW_VERIFY_RATE {
			trace!(target: "snapshot", "verifying PoW");
			try!(engine.verify_block_seal(&block.header))
		} else {
			trace!(target: "snapshot", "verifying basic");
			try!(engine.verify_block_basic(&block.header, Some(&block_bytes)));
		}

		self.chain.insert_canon_block(&block_bytes, receipts);

		self.parent_hash = BlockView::new(&block_bytes).hash();
		self.cur_number += 1;

		Ok(())
	}
}