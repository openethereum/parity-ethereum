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

//! Pv64 snapshot creation helpers.

// Try to have chunks be around 16MB (before compression)
const PREFERRED_CHUNK_SIZE: usize = 16 * 1024 * 1024;

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use account::Account;
use client::BlockChainClient;
use error::Error;
use ids::BlockID;
use views::BlockView;

use util::{Bytes, Hashable};
use util::hash::H256;
use util::{HashDB, TrieDB};
use util::rlp::{DecoderError, Stream, RlpStream, UntrustedRlp, View};

/// Used to build block chunks.
pub struct BlockChunker<'a> {
	client: &'a BlockChainClient,
	// block, receipt rlp pairs.
	rlps: VecDeque<(Bytes, Bytes)>,
	genesis_hash: H256,
	current_hash: H256,
}

impl<'a> BlockChunker<'a> {
	/// Create a new BlockChunker given a client and the genesis hash.
	pub fn new(client: &'a BlockChainClient, best_block_hash: H256, genesis_hash: H256) -> Self {
		// Todo [rob]: find a way to reuse rlp allocations
		BlockChunker {
			client: client,
			rlps: VecDeque::new(),
			genesis_hash: genesis_hash,
			current_hash: best_block_hash,
		}
	}

	// Try to fill the buffers, moving backwards from current block hash.
	// This will return true if it created a block chunk, false otherwise.
	fn fill_buffers(&mut self) -> bool {
		let mut loaded_size = 0;
		let mut blocks_loaded = 0;

		while loaded_size < PREFERRED_CHUNK_SIZE && self.current_hash != self.genesis_hash {

			// skip compression for now
			let block = self.client.block(BlockID::Hash(self.current_hash)).unwrap();
			let receipts = self.client.block_receipts(&self.current_hash).unwrap();

			let new_loaded_size = loaded_size + (block.len() + receipts.len());

			// cut off the chunk if too large
			if new_loaded_size > PREFERRED_CHUNK_SIZE {
				break;
			} else {
				loaded_size = new_loaded_size;
			}

			self.current_hash = BlockView::new(&block).header_view().parent_hash();

			self.rlps.push_front((block, receipts));
			blocks_loaded += 1;
		}

		if blocks_loaded > 0 {
			trace!(target: "pv64_snapshot", "prepared block chunk with {} blocks", blocks_loaded);
		}

		loaded_size != 0
	}

	// write out the data in the buffers to a chunk on disk
	fn write_chunk(&mut self, path: &Path) -> H256 {
		// Todo [rob]: compress raw data, put parent hash and block number into chunk.
		let mut rlp_stream = RlpStream::new_list(self.rlps.len());
		for (block, receipts) in self.rlps.drain(..) {
			rlp_stream.begin_list(2).append(&block).append(&receipts);
		}

		let raw_data = rlp_stream.out();
		let hash = raw_data.sha3();

		trace!(target: "pv64_snapshot", "writing block chunk. hash: {},  size: {} bytes", hash.hex(), raw_data.len());

		let mut file_path = path.to_owned();
		file_path.push(hash.hex());

		let mut file = File::create(file_path).unwrap();
		file.write_all(&raw_data).unwrap();

		hash
	}

	/// Create and write out all block chunks to disk, returning a vector of all
	/// the hashes of block chunks created.
	///
	/// The path parameter is the directory to store the block chunks in.
	/// This function assumes the directory exists already.
	pub fn chunk_all(mut self, path: &Path) -> Vec<H256> {
		let mut chunk_hashes = Vec::new();

		while self.fill_buffers() {
			chunk_hashes.push(self.write_chunk(path));
		}

		chunk_hashes
	}
}

/// State trie chunker.
pub struct StateChunker<'a> {
	hashes: Vec<H256>,
	rlps: Vec<Bytes>,
	cur_size: usize,
	hash_db: &'a HashDB,
	snapshot_path: &'a Path,
}

impl<'a> StateChunker<'a> {
	// Push a key, value pair to be encoded.
	//
	// If the buffer is greater than the desired chunk size,
	// this will write out the data to disk.
	fn push(&mut self, key: Bytes, value: Bytes) {
		let pair = RlpStream::new_list(2).append(&key).append(&value).out();

		if self.cur_size + pair.len() >= PREFERRED_CHUNK_SIZE {
			self.write_chunk();
		}

		self.rlps.push(pair);
	}

	// Write out the buffer to disk, pushing the created chunk's hash to
	// the list.
	fn write_chunk(&mut self) {
		let mut bytes = RlpStream::new().append(&&self.rlps[..]).out();

		self.rlps.clear();

		let hash = bytes.sha3();

		let mut path = self.snapshot_path.to_owned();
		path.push(hash.hex());

		let mut file = File::create(path).unwrap();
		file.write_all(&bytes).unwrap();

		self.hashes.push(hash);
		self.cur_size = 0;
	}

	// walk an account's storage trie, pushing all the key, value pairs.
	fn walk_account(&mut self, account: Account) -> Result<(), Error> {
		let storage_view = try!(TrieDB::new(self.hash_db, &account.storage_root));

		for (storage_key, storage_val) in storage_view.iter() {
			self.push(storage_key, storage_val);
		}
	}

	/// Walk the given state database starting from the given root,
	/// creating chunks and writing them out.
	///
	/// Returns a list of hashes created, or any error it may
	/// have encountered.
	///
	/// The data in the chunks is just a list of key, value pairs.
	pub fn chunk_all(db: &'a HashDB, root: &'a H256, path: &'a Path) -> Result<Vec<H256>, Error> {
		let account_view = try!(TrieDB::new(db, &root));

		let mut chunker = StateChunker {
			hashes: Vec::new(),
			rlps: Vec::new(),
			cur_size: 0,
			hash_db: db,
			snapshot_path: path,
		};

		for (account_key, account_data) in account_view.iter() {
			// todo [rob]: reformat account data to new RLP structure.
			let account = Account::from_rlp(account_data);
			chunker.push(account_key, account_data.to_owned());

			try!(chunker.walk_account(account_data));
		}

		chunker.hashes
	}
}

/// Manifest data.
pub struct ManifestData {
	/// List of state chunk hashes.
	pub state_hashes: Vec<H256>,
	/// List of block chunk hashes.
	pub block_hashes: Vec<H256>,
	/// The final, expected state root.
	pub state_root: H256,
}

impl ManifestData {
	/// Encode the manifest data to.
	pub fn to_rlp(self) -> Bytes {
		let mut stream = RlpStream::new_list(3);
		stream.append(&self.state_hashes);
		stream.append(&self.block_hashes);
		stream.append(&self.state_root);

		stream.out()
	}

	/// Try to restore manifest data from raw bytes interpreted as RLP.
	pub fn from_rlp(raw: &[u8]) -> Result<Self, DecoderError> {
		let decoder = UntrustedRlp::new(raw);

		let state_hashes: Vec<H256> = try!(try!(decoder.at(0)).as_val());
		let block_hashes: Vec<H256> = try!(try!(decoder.at(1)).as_val());
		let state_root: H256 = try!(try!(decoder.at(2)).as_val());

		Ok(ManifestData {
			state_hashes: state_hashes,
			block_hashes: block_hashes,
			state_root: state_root,
		})
	}
}