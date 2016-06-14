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

//! Snapshot creation helpers.

// Try to have chunks be around 16MB (before compression)
const PREFERRED_CHUNK_SIZE: usize = 16 * 1024 * 1024;

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use account_db::AccountDB;
use client::BlockChainClient;
use error::Error;
use ids::BlockID;
use views::BlockView;

use util::{Bytes, Hashable, HashDB, TrieDB};
use util::hash::{FixedHash, H256};
use util::numbers::U256;
use util::rlp::{DecoderError, Rlp, RlpStream, Stream, SHA3_NULL_RLP, UntrustedRlp, View};
/// Used to build block chunks.
struct BlockChunker<'a> {
	client: &'a BlockChainClient,
	// block, receipt rlp pairs.
	rlps: VecDeque<Bytes>,
	genesis_hash: H256,
	current_hash: H256,
	hashes: Vec<H256>,
}

impl<'a> BlockChunker<'a> {
	// Try to fill the buffers, moving backwards from current block hash.
	// This will return true if it created a block chunk, false otherwise.
	fn fill_buffers(&mut self) -> bool {
		let mut loaded_size = 0;
		let mut blocks_loaded = 0;

		while loaded_size < PREFERRED_CHUNK_SIZE && self.current_hash != self.genesis_hash {

			// skip compression for now
			let block = self.client.block(BlockID::Hash(self.current_hash)).unwrap();
			let receipts = self.client.block_receipts(&self.current_hash).unwrap();

			let pair = {
				let mut pair_stream = RlpStream::new_list(2);
				pair_stream.append(&block).append(&receipts);
				pair_stream.out()
			};

			let new_loaded_size = loaded_size + pair.len();

			// cut off the chunk if too large
			if new_loaded_size > PREFERRED_CHUNK_SIZE {
				break;
			} else {
				loaded_size = new_loaded_size;
			}

			self.rlps.push_front(pair);
			self.current_hash = BlockView::new(&block).header_view().parent_hash();
			blocks_loaded += 1;
		}

		if blocks_loaded > 0 {
			trace!(target: "snapshot", "prepared block chunk with {} blocks", blocks_loaded);
		}

		loaded_size != 0
	}

	// write out the data in the buffers to a chunk on disk
	fn write_chunk(&mut self, path: &Path) -> Result<(), Error> {
		// Todo [rob]: compress raw data, put parent hash and block number into chunk.
		let mut rlp_stream = RlpStream::new_list(self.rlps.len());
		for pair in self.rlps.drain(..) {
			rlp_stream.append(&pair);
		}

		let raw_data = rlp_stream.out();
		let hash = raw_data.sha3();

		trace!(target: "snapshot", "writing block chunk. hash: {},  size: {} bytes", hash.hex(), raw_data.len());

		let mut file_path = path.to_owned();
		file_path.push(hash.hex());

		let mut file = try!(File::create(file_path));
		try!(file.write_all(&raw_data));

		self.hashes.push(hash);
		Ok(())
	}
}

/// Create and write out all block chunks to disk, returning a vector of all
/// the hashes of block chunks created.
///
/// The path parameter is the directory to store the block chunks in.
/// This function assumes the directory exists already.
pub fn chunk_blocks(client: &BlockChainClient, best_block_hash: H256, genesis_hash: H256, path: &Path) -> Result<Vec<H256>, Error> {
	let mut chunker = BlockChunker {
		client: client,
		rlps: VecDeque::new(),
		genesis_hash: genesis_hash,
		current_hash: best_block_hash,
		hashes: Vec::new(),
	};

	while chunker.fill_buffers() {
		try!(chunker.write_chunk(path));
	}
	if chunker.rlps.len() != 0 {
		try!(chunker.write_chunk(path));
	}
	Ok(chunker.hashes)
}

/// State trie chunker.
struct StateChunker<'a> {
	hashes: Vec<H256>,
	rlps: Vec<Bytes>,
	cur_size: usize,
	snapshot_path: &'a Path,
}

impl<'a> StateChunker<'a> {
	// Push a key, value pair to be encoded.
	//
	// If the buffer is greater than the desired chunk size,
	// this will write out the data to disk.
	fn push(&mut self, key: Bytes, value: Bytes) -> Result<(), Error> {
		let pair = {
			let mut stream = RlpStream::new_list(2);
			stream.append(&key).append(&value);
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
		trace!(target: "snapshot", "writing state chunk. uncompressed size: {}", self.cur_size);

		let bytes = {
			let mut stream = RlpStream::new();
			stream.append(&&self.rlps[..]);
			stream.out()
		};

		self.rlps.clear();

		let hash = bytes.sha3();

		let mut path = self.snapshot_path.to_owned();
		path.push(hash.hex());

		let mut file = try!(File::create(path));
		try!(file.write_all(&bytes));

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
pub fn chunk_state(db: &HashDB, root: &H256, path: &Path) -> Result<Vec<H256>, Error> {
	let account_view = try!(TrieDB::new(db, &root));

	let mut chunker = StateChunker {
		hashes: Vec::new(),
		rlps: Vec::new(),
		cur_size: 0,
		snapshot_path: path,
	};

	trace!(target: "snapshot", "beginning state chunking");

	// account_key here is the address' hash.
	for (account_key, account_data) in account_view.iter() {
		let account = AccountReader::from_thin_rlp(account_data);
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

// An alternate account structure, only used for reading the storage values
// out of the account as opposed to writing any.
struct AccountReader {
	nonce: U256,
	balance: U256,
	storage_root: H256,
	code_hash: H256,
}

impl AccountReader {
	// deserialize the account from rlp.
	fn from_thin_rlp(rlp: &[u8]) -> Self {
		let r: Rlp = Rlp::new(rlp);

		AccountReader {
			nonce: r.val_at(0),
			balance: r.val_at(1),
			storage_root: r.val_at(2),
			code_hash: r.val_at(3),
		}
	}

	// walk the account's storage trie, returning an RLP item containing the
	// account properties and the storage.
	fn to_fat_rlp(&self, hash_db: &HashDB) -> Result<Bytes, Error> {
		let db = try!(TrieDB::new(hash_db, &self.storage_root));

		let mut pairs = Vec::new();

		for (k, v) in db.iter() {
			pairs.push((k, v));
		}

		let mut stream = RlpStream::new_list(pairs.len());

		for (k, v) in pairs {
			stream.begin_list(2).append(&k).append(&v);
		}

		let pairs_rlp = stream.out();

		let mut account_stream = RlpStream::new_list(5);
		account_stream.append(&self.nonce)
					  .append(&self.balance)
					  .append(&self.storage_root);

		account_stream.begin_list(2);
		if self.code_hash == SHA3_NULL_RLP {
			account_stream.append(&true).append(&hash_db.get(&self.code_hash).unwrap());
		} else {
			account_stream.append(&false).append_empty_data();
		}

		account_stream.append(&pairs_rlp);

		Ok(account_stream.out())
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