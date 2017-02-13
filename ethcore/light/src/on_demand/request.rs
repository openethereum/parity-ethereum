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

//! Request types, verification, and verification errors.

use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::receipt::Receipt;

use rlp::{RlpStream, Stream, UntrustedRlp, View};
use util::{Address, Bytes, HashDB, H256, U256};
use util::memorydb::MemoryDB;
use util::sha3::Hashable;
use util::trie::{Trie, TrieDB, TrieError};

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
	/// Wrong hash.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderByNumber {
	/// The header's number.
	num: u64,
	/// The cht number for the given block number.
	cht_num: u64,
	/// The root of the CHT containing this header.
	cht_root: H256,
}

impl HeaderByNumber {
	/// Construct a new header-by-number request. Fails if the given number is 0.
	/// Provide the expected CHT root to compare against.
	pub fn new(num: u64, cht_root: H256) -> Option<Self> {
		::cht::block_to_cht_number(num).map(|cht_num| HeaderByNumber {
			num: num,
			cht_num: cht_num,
			cht_root: cht_root,
		})
	}

	/// Access the requested block number.
	pub fn num(&self) -> u64 { self.num }

	/// Access the CHT number.
	pub fn cht_num(&self) -> u64 { self.cht_num }

	/// Access the expected CHT root.
	pub fn cht_root(&self) -> H256 { self.cht_root }

	/// Check a response with a header and cht proof.
	pub fn check_response(&self, header: &[u8], proof: &[Bytes]) -> Result<(encoded::Header, U256), Error> {
		let (expected_hash, td) = match ::cht::check_proof(proof, self.num, self.cht_root) {
			Some((expected_hash, td)) => (expected_hash, td),
			None => return Err(Error::BadProof),
		};

		// and compare the hash to the found header.
		let found_hash = header.sha3();
		match expected_hash == found_hash {
			true => Ok((encoded::Header::new(header.to_vec()), td)),
			false => Err(Error::WrongHash(expected_hash, found_hash)),
		}
	}
}

/// Request for a header by hash.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body {
	/// The block's header.
	pub header: encoded::Header,
	/// The block's hash.
	pub hash: H256,
}

impl Body {
	/// Create a request for a block body from a given header.
	pub fn new(header: encoded::Header) -> Self {
		let hash = header.hash();
		Body {
			header: header,
			hash: hash,
		}
	}

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Request for account code.
pub struct Code {
	/// Block hash, number pair.
	pub block_id: (H256, u64),
	/// Address requested.
	pub address: Address,
	/// Account's code hash.
	pub code_hash: H256,
}

impl Code {
	/// Check a response with code against the code hash.
	pub fn check_response(&self, code: &[u8]) -> Result<(), Error> {
		let found_hash = code.sha3();
		if found_hash == self.code_hash {
			Ok(())
		} else {
			Err(Error::WrongHash(self.code_hash, found_hash))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use util::{MemoryDB, Address, H256, FixedHash};
	use util::trie::{Trie, TrieMut, SecTrieDB, SecTrieDBMut};
	use util::trie::recorder::Recorder;

	use ethcore::client::{BlockChainClient, TestBlockChainClient, EachBlockWith};
	use ethcore::header::Header;
	use ethcore::encoded;
	use ethcore::receipt::Receipt;

	#[test]
	fn no_invalid_header_by_number() {
		assert!(HeaderByNumber::new(0, Default::default()).is_none())
	}

	#[test]
	fn check_header_by_number() {
		use ::cht;

		let test_client = TestBlockChainClient::new();
		test_client.add_blocks(10500, EachBlockWith::Nothing);

		let cht = {
			let fetcher = |id| {
				let hdr = test_client.block_header(id).unwrap();
				let td = test_client.block_total_difficulty(id).unwrap();
				Some(cht::BlockInfo {
					hash: hdr.hash(),
					parent_hash: hdr.parent_hash(),
					total_difficulty: td,
				})
			};

			cht::build(cht::block_to_cht_number(10_000).unwrap(), fetcher).unwrap()
		};

		let proof = cht.prove(10_000, 0).unwrap().unwrap();
		let req = HeaderByNumber::new(10_000, cht.root()).unwrap();

		let raw_header = test_client.block_header(::ethcore::ids::BlockId::Number(10_000)).unwrap();

		assert!(req.check_response(&raw_header.into_inner(), &proof[..]).is_ok());
	}

	#[test]
	fn check_header_by_hash() {
		let mut header = Header::new();
		header.set_number(10_000);
		header.set_extra_data(b"test_header".to_vec());
		let hash = header.hash();
		let raw_header = ::rlp::encode(&header);

		assert!(HeaderByHash(hash).check_response(&*raw_header).is_ok())
	}

	#[test]
	fn check_body() {
		use rlp::{RlpStream, Stream};

		let header = Header::new();
		let mut body_stream = RlpStream::new_list(2);
		body_stream.begin_list(0).begin_list(0);

		let req = Body {
			header: encoded::Header::new(::rlp::encode(&header).to_vec()),
			hash: header.hash(),
		};

		assert!(req.check_response(&*body_stream.drain()).is_ok())
	}

	#[test]
	fn check_receipts() {
		let receipts = (0..5).map(|_| Receipt {
			state_root: Some(H256::random()),
			gas_used: 21_000u64.into(),
			log_bloom: Default::default(),
			logs: Vec::new(),
		}).collect::<Vec<_>>();

		let mut header = Header::new();
		let receipts_root = ::util::triehash::ordered_trie_root(
			receipts.iter().map(|x| ::rlp::encode(x).to_vec())
		);

		header.set_receipts_root(receipts_root);

		let req = BlockReceipts(encoded::Header::new(::rlp::encode(&header).to_vec()));

		assert!(req.check_response(&receipts).is_ok())
	}

	#[test]
	fn check_state_proof() {
		use rlp::{RlpStream, Stream};

		let mut root = H256::default();
		let mut db = MemoryDB::new();
		let mut header = Header::new();
		header.set_number(123_456);
		header.set_extra_data(b"test_header".to_vec());

		let addr = Address::random();
		let rand_acc = || {
			let mut stream = RlpStream::new_list(4);
			stream.append(&2u64)
				.append(&100_000_000u64)
				.append(&H256::random())
				.append(&H256::random());

			stream.out()
		};
		{
			let mut trie = SecTrieDBMut::new(&mut db, &mut root);
			for _ in 0..100 {
				let address = Address::random();
				trie.insert(&*address, &rand_acc()).unwrap();
			}

			trie.insert(&*addr, &rand_acc()).unwrap();
		}

		let proof = {
			let trie = SecTrieDB::new(&db, &root).unwrap();
			let mut recorder = Recorder::new();

			trie.get_with(&*addr, &mut recorder).unwrap().unwrap();

			recorder.drain().into_iter().map(|r| r.data).collect::<Vec<_>>()
		};

		header.set_state_root(root.clone());

		let req = Account {
			header: encoded::Header::new(::rlp::encode(&header).to_vec()),
			address: addr,
		};

		assert!(req.check_response(&proof[..]).is_ok());
	}

	#[test]
	fn check_code() {
		let code = vec![1u8; 256];
		let req = Code {
			block_id: (Default::default(), 2),
			address: Default::default(),
			code_hash: ::util::Hashable::sha3(&code),
		};

		assert!(req.check_response(&code).is_ok());
		assert!(req.check_response(&[]).is_err());
	}
}
