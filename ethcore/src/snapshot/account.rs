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

//! Account state encoding and decoding

use account_db::{AccountDB, AccountDBMut};
use client::BlockChainClient;
use error::Error;

use util::{Bytes, HashDB, SHA3_EMPTY, TrieDB};
use util::hash::{FixedHash, H256};
use util::numbers::U256;
use util::rlp::{DecoderError, Rlp, RlpStream, Stream, UntrustedRlp, View};

// An alternate account structure from ::account::Account.
pub struct Account {
	nonce: U256,
	balance: U256,
	storage_root: H256,
	code_hash: H256,
}

impl Account {
	// decode the account from rlp.
	pub fn from_thin_rlp(rlp: &[u8]) -> Self {
		let r: Rlp = Rlp::new(rlp);

		Account {
			nonce: r.val_at(0),
			balance: r.val_at(1),
			storage_root: r.val_at(2),
			code_hash: r.val_at(3),
		}
	}

	// encode the account to a standard rlp.
	pub fn to_thin_rlp(&self) -> Bytes {
		let mut stream = RlpStream::new_list(4);
		stream
			.append(&self.nonce)
			.append(&self.balance)
			.append(&self.storage_root)
			.append(&self.code_hash);

		stream.out()
	}

	// walk the account's storage trie, returning an RLP item containing the
	// account properties and the storage.
	pub fn to_fat_rlp(&self, acct_db: &AccountDB, addr_hash: H256) -> Result<Bytes, Error> {
		let db = try!(TrieDB::new(acct_db, &self.storage_root));

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
					  .append(&self.balance);

		// [has_code, code_hash].
		if self.code_hash == SHA3_EMPTY {
			account_stream.append(&false).append_empty_data();
		} else {
			match acct_db.lookup(&self.code_hash) {
				Some(c) => {
					account_stream.append(&true).append(&c);
				}
				None => {
					warn!("code lookup failed for account with address hash {}, code hash {}", addr_hash, self.code_hash);
					account_stream.append(&false).append_empty_data();
				}
			}
		}

		account_stream.append_raw(&pairs_rlp, 1);

		Ok(account_stream.out())
	}

	// decode a fat rlp, and rebuild the storage trie as we go.
	pub fn from_fat_rlp(acct_db: &mut AccountDBMut, rlp: UntrustedRlp) -> Result<Self, DecoderError> {
		use util::{TrieDBMut, TrieMut};

		let nonce = try!(rlp.val_at(0));
		let balance = try!(rlp.val_at(1));
		let code_hash = if try!(rlp.val_at(2)) {
			let code: Bytes = try!(rlp.val_at(3));
			acct_db.insert(&code)
		} else {
			SHA3_EMPTY
		};

		let mut storage_root = H256::zero();

		{
			let mut storage_trie = TrieDBMut::new(acct_db, &mut storage_root);
			let pairs = try!(rlp.at(4));
			for pair_rlp in pairs.iter() {
				let k: Bytes  = try!(pair_rlp.val_at(0));
				let v: Bytes = try!(pair_rlp.val_at(1));

				storage_trie.insert(&k, &v);
			}
		}
		Ok(Account {
			nonce: nonce,
			balance: balance,
			storage_root: storage_root,
			code_hash: code_hash,
		})
	}
}