// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! Request types, verification, and verification errors.

use ethcore::encoded;
use ethcore::receipt::Receipt;

use rlp::{RlpStream, Stream, UntrustedRlp, View};
use util::{Address, Bytes, HashDB, H256};
use util::memorydb::MemoryDB;
use util::sha3::Hashable;
use util::trie::{Trie, TrieDB, TrieError};

use super::Account as BasicAccount;

/// Errors in verification.
#[derive(Debug, PartialEq)]
pub enum Error {
	/// RLP decoder error.
	Decoder(::rlp::DecoderError),
	/// Trie lookup error (result of bad proof)
	Trie(TrieError),
	/// Bad inclusion proof
	BadProof,
	/// Wrong header number.
	WrongNumber(u64, u64),
	/// Wrong header hash.
	WrongHash(H256, H256),
	/// Wrong trie root.
	WrongTrieRoot(H256, H256),
}

impl From<::rlp::DecoderError> for Error {
	fn from(err: ::rlp::DecoderError) -> Self {
		Error::Decoder(err)
	}
}

impl From<Box<TrieError>> for Error {
	fn from(err: Box<TrieError>) -> Self {
		Error::Trie(*err)
	}
}

/// Request for a header by number.
pub struct HeaderByNumber {
	/// The header's number.
	pub num: u64,
	/// The root of the CHT containing this header.
	pub cht_root: H256,
}

impl HeaderByNumber {
	/// Check a response with a header and cht proof.
	pub fn check_response(&self, header: &[u8], proof: &[Bytes]) -> Result<encoded::Header, Error> {
		use util::trie::{Trie, TrieDB};

		// check the proof
		let mut db = MemoryDB::new();

		for node in proof { db.insert(&node[..]); }
		let key = ::rlp::encode(&self.num);

		let expected_hash: H256 = match TrieDB::new(&db, &self.cht_root).and_then(|t| t.get(&*key))? {
			Some(val) => ::rlp::decode(&val),
			None => return Err(Error::BadProof)
		};

		// and compare the hash to the found header.
		let found_hash = header.sha3();
		match expected_hash == found_hash {
			true => Ok(encoded::Header::new(header.to_vec())),
			false => Err(Error::WrongHash(expected_hash, found_hash)),
		}
	}
}

/// Request for a header by hash.
pub struct HeaderByHash(pub H256);

impl HeaderByHash {
	/// Check a response for the header.
	pub fn check_response(&self, header: &[u8]) -> Result<encoded::Header, Error> {
		let hash = header.sha3();
		match hash == self.0 {
			true => Ok(encoded::Header::new(header.to_vec())),
			false => Err(Error::WrongHash(self.0, hash)),
		}
	}
}

/// Request for a block, with header and precomputed hash.
pub struct Body {
	/// The block's header.
	pub header: encoded::Header,
	/// The block's hash.
	pub hash: H256,
}

impl Body {
	/// Check a response for this block body.
	pub fn check_response(&self, body: &[u8]) -> Result<encoded::Block, Error> {
		let body_view = UntrustedRlp::new(&body);

		// check the integrity of the the body against the header
		let tx_root = ::util::triehash::ordered_trie_root(body_view.at(0)?.iter().map(|r| r.as_raw().to_vec()));
		if tx_root != self.header.transactions_root() {
			return Err(Error::WrongTrieRoot(self.header.transactions_root(), tx_root));
		}

		let uncles_hash = body_view.at(1)?.as_raw().sha3();
		if uncles_hash != self.header.uncles_hash() {
			return Err(Error::WrongHash(self.header.uncles_hash(), uncles_hash));
		}

		// concatenate the header and the body.
		let mut stream = RlpStream::new_list(3);
		stream.append_raw(self.header.rlp().as_raw(), 1);
		stream.append_raw(body, 2);

		Ok(encoded::Block::new(stream.out()))
	}
}

/// Request for a block's receipts with header for verification.
pub struct BlockReceipts(pub encoded::Header);

impl BlockReceipts {
	/// Check a response with receipts against the stored header.
	pub fn check_response(&self, receipts: &[Receipt]) -> Result<Vec<Receipt>, Error> {
		let receipts_root = self.0.receipts_root();
		let found_root = ::util::triehash::ordered_trie_root(receipts.iter().map(|r| ::rlp::encode(r).to_vec()));

		match receipts_root == found_root {
			true => Ok(receipts.to_vec()),
			false => Err(Error::WrongTrieRoot(receipts_root, found_root)),
		}
	}
}

/// Request for an account structure.
pub struct Account {
	/// Header for verification.
	pub header: encoded::Header,
	/// Address requested.
	pub address: Address,
}

impl Account {
	/// Check a response with an account against the stored header.
	pub fn check_response(&self, proof: &[Bytes]) -> Result<BasicAccount, Error> {
		let state_root = self.header.state_root();

		let mut db = MemoryDB::new();
		for node in proof { db.insert(&node[..]); }

		match TrieDB::new(&db, &state_root).and_then(|t| t.get(&self.address.sha3()))? {
			Some(val) => {
				let rlp = UntrustedRlp::new(&val);
				Ok(BasicAccount {
					nonce: rlp.val_at(0)?,
					balance: rlp.val_at(1)?,
					storage_root: rlp.val_at(2)?,
					code_hash: rlp.val_at(3)?,
				})
			},
			None => Err(Error::BadProof)
		}
	}
}
