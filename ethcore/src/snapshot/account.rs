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
use error::Error;

use util::{Bytes, HashDB, SHA3_EMPTY, TrieDB};
use util::hash::{FixedHash, H256};
use util::numbers::U256;
use util::rlp::{DecoderError, Rlp, RlpStream, Stream, UntrustedRlp, View};

// An alternate account structure from ::account::Account.
#[derive(PartialEq, Clone, Debug)]
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
	pub fn to_fat_rlp(&self, acct_db: &AccountDB) -> Result<Bytes, Error> {
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
			match acct_db.get(&self.code_hash) {
				Some(c) => {
					account_stream.append(&true).append(&c);
				}
				None => {
					warn!("code lookup failed during snapshot");
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

#[cfg(test)]
mod tests {
	use account_db::{AccountDB, AccountDBMut};
	use tests::helpers::get_temp_journal_db;

	use util::{SHA3_NULL_RLP, SHA3_EMPTY};
	use util::hash::{Address, FixedHash, H256};
	use util::rlp::{UntrustedRlp, View};
	use util::trie::{Alphabet, StandardMap, SecTrieDBMut, TrieMut, ValueMode};

	use super::Account;

	fn fill_storage(mut db: AccountDBMut) -> H256 {
		let map = StandardMap {
			alphabet: Alphabet::All,
			min_key: 6,
			journal_key: 6,
			value_mode: ValueMode::Random,
			count: 100
		};

		let mut root = H256::new();
		{
			let mut trie = SecTrieDBMut::new(&mut db, &mut root);
			for (k, v) in map.make() {
				trie.insert(&k, &v);
			}
		}
		root
	}

	#[test]
	fn encoding_basic() {
		let mut db = get_temp_journal_db();
		let mut db = &mut **db;
		let addr = Address::random();

		let account = Account {
			nonce: 50.into(),
			balance: 123456789.into(),
			storage_root: SHA3_NULL_RLP,
			code_hash: SHA3_EMPTY,
		};

		let thin_rlp = account.to_thin_rlp();
		assert_eq!(Account::from_thin_rlp(&thin_rlp), account);

		let fat_rlp = account.to_fat_rlp(&AccountDB::new(db.as_hashdb(), &addr)).unwrap();
		let fat_rlp = UntrustedRlp::new(&fat_rlp);
		assert_eq!(Account::from_fat_rlp(&mut AccountDBMut::new(db.as_hashdb_mut(), &addr), fat_rlp).unwrap(), account);
	}

	#[test]
	fn encoding_storage() {
		let mut db = get_temp_journal_db();
		let mut db = &mut **db;
		let addr = Address::random();

		let root = fill_storage(AccountDBMut::new(db.as_hashdb_mut(), &addr));
		let account = Account {
			nonce: 25.into(),
			balance: 987654321.into(),
			storage_root: root,
			code_hash: SHA3_EMPTY,
		};

		let thin_rlp = account.to_thin_rlp();
		assert_eq!(Account::from_thin_rlp(&thin_rlp), account);

		let fat_rlp = account.to_fat_rlp(&AccountDB::new(db.as_hashdb(), &addr)).unwrap();
		let fat_rlp = UntrustedRlp::new(&fat_rlp);
		assert_eq!(Account::from_fat_rlp(&mut AccountDBMut::new(db.as_hashdb_mut(), &addr), fat_rlp).unwrap(), account);
	}
}