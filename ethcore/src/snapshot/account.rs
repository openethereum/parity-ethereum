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

//! Account state encoding and decoding

use account_db::{AccountDB, AccountDBMut};
use basic_account::BasicAccount;
use snapshot::Error;

use util::{U256, FixedHash, H256, Bytes, HashDB, SHA3_EMPTY, SHA3_NULL_RLP};
use util::trie::{TrieDB, Trie};
use rlp::{RlpStream, Stream, UntrustedRlp, View};

use std::collections::HashSet;

// An empty account -- these are replaced with RLP null data for a space optimization.
const ACC_EMPTY: BasicAccount = BasicAccount {
	nonce: U256([0, 0, 0, 0]),
	balance: U256([0, 0, 0, 0]),
	storage_root: SHA3_NULL_RLP,
	code_hash: SHA3_EMPTY,
};

// whether an encoded account has code and how it is referred to.
#[repr(u8)]
enum CodeState {
	// the account has no code.
	Empty = 0,
	// raw code is encoded.
	Inline = 1,
	// the code is referred to by hash.
	Hash = 2,
}

impl CodeState {
	fn from(x: u8) -> Result<Self, Error> {
		match x {
			0 => Ok(CodeState::Empty),
			1 => Ok(CodeState::Inline),
			2 => Ok(CodeState::Hash),
			_ => Err(Error::UnrecognizedCodeState(x))
		}
	}

	fn raw(self) -> u8 {
		self as u8
	}
}

// walk the account's storage trie, returning an RLP item containing the
// account properties and the storage.
pub fn to_fat_rlp(acc: &BasicAccount, acct_db: &AccountDB, used_code: &mut HashSet<H256>) -> Result<Bytes, Error> {
	if acc == &ACC_EMPTY {
		return Ok(::rlp::NULL_RLP.to_vec());
	}

	let db = TrieDB::new(acct_db, &acc.storage_root)?;

	let mut pairs = Vec::new();

	for item in db.iter()? {
		let (k, v) = item?;
		pairs.push((k, v));
	}

	let mut stream = RlpStream::new_list(pairs.len());

	for (k, v) in pairs {
		stream.begin_list(2).append(&k).append(&&*v);
	}

	let pairs_rlp = stream.out();

	let mut account_stream = RlpStream::new_list(5);
	account_stream.append(&acc.nonce)
				  .append(&acc.balance);

	// [has_code, code_hash].
	if acc.code_hash == SHA3_EMPTY {
		account_stream.append(&CodeState::Empty.raw()).append_empty_data();
	} else if used_code.contains(&acc.code_hash) {
		account_stream.append(&CodeState::Hash.raw()).append(&acc.code_hash);
	} else {
		match acct_db.get(&acc.code_hash) {
			Some(c) => {
				used_code.insert(acc.code_hash.clone());
				account_stream.append(&CodeState::Inline.raw()).append(&&*c);
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
// returns the account structure along with its newly recovered code,
// if it exists.
pub fn from_fat_rlp(
	acct_db: &mut AccountDBMut,
	rlp: UntrustedRlp,
) -> Result<(BasicAccount, Option<Bytes>), Error> {
	use util::{TrieDBMut, TrieMut};

	// check for special case of empty account.
	if rlp.is_empty() {
		return Ok((ACC_EMPTY, None));
	}

	let nonce = rlp.val_at(0)?;
	let balance = rlp.val_at(1)?;
	let code_state: CodeState = {
		let raw: u8 = rlp.val_at(2)?;
		CodeState::from(raw)?
	};

	// load the code if it exists.
	let (code_hash, new_code) = match code_state {
		CodeState::Empty => (SHA3_EMPTY, None),
		CodeState::Inline => {
			let code: Bytes = rlp.val_at(3)?;
			let code_hash = acct_db.insert(&code);

			(code_hash, Some(code))
		}
		CodeState::Hash => {
			let code_hash = rlp.val_at(3)?;

			(code_hash, None)
		}
	};

	let mut storage_root = H256::zero();

	{
		let mut storage_trie = TrieDBMut::new(acct_db, &mut storage_root);
		let pairs = rlp.at(4)?;
		for pair_rlp in pairs.iter() {
			let k: Bytes  = pair_rlp.val_at(0)?;
			let v: Bytes = pair_rlp.val_at(1)?;

			storage_trie.insert(&k, &v)?;
		}
	}

	let acc = BasicAccount {
		nonce: nonce,
		balance: balance,
		storage_root: storage_root,
		code_hash: code_hash,
	};

	Ok((acc, new_code))
}

#[cfg(test)]
mod tests {
	use account_db::{AccountDB, AccountDBMut};
	use basic_account::BasicAccount;
	use tests::helpers::get_temp_state_db;
	use snapshot::tests::helpers::fill_storage;

	use util::sha3::{SHA3_EMPTY, SHA3_NULL_RLP};
	use util::{Address, FixedHash, H256, HashDB, DBValue};
	use rlp::{UntrustedRlp, View};

	use std::collections::HashSet;

	use super::{ACC_EMPTY, to_fat_rlp, from_fat_rlp};

	#[test]
	fn encoding_basic() {
		let mut db = get_temp_state_db();
		let addr = Address::random();

		let account = BasicAccount {
			nonce: 50.into(),
			balance: 123456789.into(),
			storage_root: SHA3_NULL_RLP,
			code_hash: SHA3_EMPTY,
		};

		let thin_rlp = ::rlp::encode(&account);
		assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp), account);

		let fat_rlp = to_fat_rlp(&account, &AccountDB::new(db.as_hashdb(), &addr), &mut Default::default()).unwrap();
		let fat_rlp = UntrustedRlp::new(&fat_rlp);
		assert_eq!(from_fat_rlp(&mut AccountDBMut::new(db.as_hashdb_mut(), &addr), fat_rlp).unwrap().0, account);
	}

	#[test]
	fn encoding_storage() {
		let mut db = get_temp_state_db();
		let addr = Address::random();

		let account = {
			let acct_db = AccountDBMut::new(db.as_hashdb_mut(), &addr);
			let mut root = SHA3_NULL_RLP;
			fill_storage(acct_db, &mut root, &mut H256::zero());
			BasicAccount {
				nonce: 25.into(),
				balance: 987654321.into(),
				storage_root: root,
				code_hash: SHA3_EMPTY,
			}
		};

		let thin_rlp = ::rlp::encode(&account);
		assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp), account);

		let fat_rlp = to_fat_rlp(&account, &AccountDB::new(db.as_hashdb(), &addr), &mut Default::default()).unwrap();
		let fat_rlp = UntrustedRlp::new(&fat_rlp);
		assert_eq!(from_fat_rlp(&mut AccountDBMut::new(db.as_hashdb_mut(), &addr), fat_rlp).unwrap().0, account);
	}

	#[test]
	fn encoding_code() {
		let mut db = get_temp_state_db();

		let addr1 = Address::random();
		let addr2 = Address::random();

		let code_hash = {
			let mut acct_db = AccountDBMut::new(db.as_hashdb_mut(), &addr1);
			acct_db.insert(b"this is definitely code")
		};

		{
			let mut acct_db = AccountDBMut::new(db.as_hashdb_mut(), &addr2);
			acct_db.emplace(code_hash.clone(), DBValue::from_slice(b"this is definitely code"));
		}

		let account1 = BasicAccount {
			nonce: 50.into(),
			balance: 123456789.into(),
			storage_root: SHA3_NULL_RLP,
			code_hash: code_hash,
		};

		let account2 = BasicAccount {
			nonce: 400.into(),
			balance: 98765432123456789usize.into(),
			storage_root: SHA3_NULL_RLP,
			code_hash: code_hash,
		};

		let mut used_code = HashSet::new();

		let fat_rlp1 = to_fat_rlp(&account1, &AccountDB::new(db.as_hashdb(), &addr1), &mut used_code).unwrap();
		let fat_rlp2 = to_fat_rlp(&account2, &AccountDB::new(db.as_hashdb(), &addr2), &mut used_code).unwrap();
		assert_eq!(used_code.len(), 1);

		let fat_rlp1 = UntrustedRlp::new(&fat_rlp1);
		let fat_rlp2 = UntrustedRlp::new(&fat_rlp2);

		let (acc, maybe_code) = from_fat_rlp(&mut AccountDBMut::new(db.as_hashdb_mut(), &addr2), fat_rlp2).unwrap();
		assert!(maybe_code.is_none());
		assert_eq!(acc, account2);

		let (acc, maybe_code) = from_fat_rlp(&mut AccountDBMut::new(db.as_hashdb_mut(), &addr1), fat_rlp1).unwrap();
		assert_eq!(maybe_code, Some(b"this is definitely code".to_vec()));
		assert_eq!(acc, account1);
	}

	#[test]
	fn encoding_empty_acc() {
		let mut db = get_temp_state_db();
		let mut used_code = HashSet::new();

		assert_eq!(to_fat_rlp(&ACC_EMPTY, &AccountDB::new(db.as_hashdb(), &Address::default()), &mut used_code).unwrap(), ::rlp::NULL_RLP.to_vec());
		assert_eq!(from_fat_rlp(&mut AccountDBMut::new(db.as_hashdb_mut(), &Address::default()), UntrustedRlp::new(&::rlp::NULL_RLP)).unwrap(), (ACC_EMPTY, None));
	}
}
