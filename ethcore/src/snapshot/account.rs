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

//! Account state encoding and decoding

use account_db::{AccountDB, AccountDBMut};
use types::basic_account::BasicAccount;
use bytes::Bytes;
use ethereum_types::{H256, U256};
use ethtrie::{TrieDB, TrieDBMut};
use hash::{KECCAK_EMPTY, KECCAK_NULL_RLP};
use hash_db::HashDB;
use rlp::{RlpStream, Rlp};
use snapshot::Error;
use std::collections::HashSet;
use trie::{Trie, TrieMut};

// An empty account -- these were replaced with RLP null data for a space optimization in v1.
const ACC_EMPTY: BasicAccount = BasicAccount {
	nonce: U256([0, 0, 0, 0]),
	balance: U256([0, 0, 0, 0]),
	storage_root: KECCAK_NULL_RLP,
	code_hash: KECCAK_EMPTY,
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

// walk the account's storage trie, returning a vector of RLP items containing the
// account address hash, account properties and the storage. Each item contains at most `max_storage_items`
// storage records split according to snapshot format definition.
pub fn to_fat_rlps(account_hash: &H256, acc: &BasicAccount, acct_db: &AccountDB, used_code: &mut HashSet<H256>, first_chunk_size: usize, max_chunk_size: usize) -> Result<Vec<Bytes>, Error> {
	let db = &(acct_db as &HashDB<_,_>);
	let db = TrieDB::new(db, &acc.storage_root)?;
	let mut chunks = Vec::new();
	let mut db_iter = db.iter()?;
	let mut target_chunk_size = first_chunk_size;
	let mut account_stream = RlpStream::new_list(2);
	let mut leftover: Option<Vec<u8>> = None;
	loop {
		account_stream.append(account_hash);
		account_stream.begin_list(5);

		account_stream.append(&acc.nonce)
					.append(&acc.balance);

		// [has_code, code_hash].
		if acc.code_hash == KECCAK_EMPTY {
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

		account_stream.begin_unbounded_list();
		if account_stream.len() > target_chunk_size {
			// account does not fit, push an empty record to mark a new chunk
			target_chunk_size = max_chunk_size;
			chunks.push(Vec::new());
		}

		if let Some(pair) = leftover.take() {
			if !account_stream.append_raw_checked(&pair, 1, target_chunk_size) {
				return Err(Error::ChunkTooSmall);
			}
		}

		loop {
			match db_iter.next() {
				Some(Ok((k, v))) => {
					let pair = {
						let mut stream = RlpStream::new_list(2);
						stream.append(&k).append(&&*v);
						stream.drain()
					};
					if !account_stream.append_raw_checked(&pair, 1, target_chunk_size) {
						account_stream.complete_unbounded_list();
						let stream = ::std::mem::replace(&mut account_stream, RlpStream::new_list(2));
						chunks.push(stream.out());
						target_chunk_size = max_chunk_size;
						leftover = Some(pair);
						break;
					}
				},
				Some(Err(e)) => {
					return Err(e.into());
				},
				None => {
					account_stream.complete_unbounded_list();
					let stream = ::std::mem::replace(&mut account_stream, RlpStream::new_list(2));
					chunks.push(stream.out());
					return Ok(chunks);
				}
			}

		}
	}
}

// decode a fat rlp, and rebuild the storage trie as we go.
// returns the account structure along with its newly recovered code,
// if it exists.
pub fn from_fat_rlp(
	acct_db: &mut AccountDBMut,
	rlp: Rlp,
	mut storage_root: H256,
) -> Result<(BasicAccount, Option<Bytes>), Error> {

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
		CodeState::Empty => (KECCAK_EMPTY, None),
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

	{
		let mut storage_trie = if storage_root.is_zero() {
			TrieDBMut::new(acct_db, &mut storage_root)
		} else {
			TrieDBMut::from_existing(acct_db, &mut storage_root)?
		};
		let pairs = rlp.at(4)?;
		for pair_rlp in pairs.iter() {
			let k: Bytes = pair_rlp.val_at(0)?;
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
	use types::basic_account::BasicAccount;
	use test_helpers::get_temp_state_db;
	use snapshot::tests::helpers::fill_storage;

	use hash::{KECCAK_EMPTY, KECCAK_NULL_RLP, keccak};
	use ethereum_types::{H256, Address};
	use hash_db::HashDB;
	use kvdb::DBValue;
	use rlp::Rlp;

	use std::collections::HashSet;

	use super::{ACC_EMPTY, to_fat_rlps, from_fat_rlp};

	#[test]
	fn encoding_basic() {
		let mut db = get_temp_state_db();
		let addr = Address::random();

		let account = BasicAccount {
			nonce: 50.into(),
			balance: 123456789.into(),
			storage_root: KECCAK_NULL_RLP,
			code_hash: KECCAK_EMPTY,
		};

		let thin_rlp = ::rlp::encode(&account);
		assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp).unwrap(), account);

		let fat_rlps = to_fat_rlps(&keccak(&addr), &account, &AccountDB::new(db.as_hash_db(), &addr), &mut Default::default(), usize::max_value(), usize::max_value()).unwrap();
		let fat_rlp = Rlp::new(&fat_rlps[0]).at(1).unwrap();
		assert_eq!(from_fat_rlp(&mut AccountDBMut::new(db.as_hash_db_mut(), &addr), fat_rlp, H256::zero()).unwrap().0, account);
	}

	#[test]
	fn encoding_storage() {
		let mut db = get_temp_state_db();
		let addr = Address::random();

		let account = {
			let acct_db = AccountDBMut::new(db.as_hash_db_mut(), &addr);
			let mut root = KECCAK_NULL_RLP;
			fill_storage(acct_db, &mut root, &mut H256::zero());
			BasicAccount {
				nonce: 25.into(),
				balance: 987654321.into(),
				storage_root: root,
				code_hash: KECCAK_EMPTY,
			}
		};

		let thin_rlp = ::rlp::encode(&account);
		assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp).unwrap(), account);

		let fat_rlp = to_fat_rlps(&keccak(&addr), &account, &AccountDB::new(db.as_hash_db(), &addr), &mut Default::default(), usize::max_value(), usize::max_value()).unwrap();
		let fat_rlp = Rlp::new(&fat_rlp[0]).at(1).unwrap();
		assert_eq!(from_fat_rlp(&mut AccountDBMut::new(db.as_hash_db_mut(), &addr), fat_rlp, H256::zero()).unwrap().0, account);
	}

	#[test]
	fn encoding_storage_split() {
		let mut db = get_temp_state_db();
		let addr = Address::random();

		let account = {
			let acct_db = AccountDBMut::new(db.as_hash_db_mut(), &addr);
			let mut root = KECCAK_NULL_RLP;
			fill_storage(acct_db, &mut root, &mut H256::zero());
			BasicAccount {
				nonce: 25.into(),
				balance: 987654321.into(),
				storage_root: root,
				code_hash: KECCAK_EMPTY,
			}
		};

		let thin_rlp = ::rlp::encode(&account);
		assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp).unwrap(), account);

		let fat_rlps = to_fat_rlps(&keccak(addr), &account, &AccountDB::new(db.as_hash_db(), &addr), &mut Default::default(), 500, 1000).unwrap();
		let mut root = KECCAK_NULL_RLP;
		let mut restored_account = None;
		for rlp in fat_rlps {
			let fat_rlp = Rlp::new(&rlp).at(1).unwrap();
			restored_account = Some(from_fat_rlp(&mut AccountDBMut::new(db.as_hash_db_mut(), &addr), fat_rlp, root).unwrap().0);
			root = restored_account.as_ref().unwrap().storage_root.clone();
		}
		assert_eq!(restored_account, Some(account));
	}

	#[test]
	fn encoding_code() {
		let mut db = get_temp_state_db();

		let addr1 = Address::random();
		let addr2 = Address::random();

		let code_hash = {
			let mut acct_db = AccountDBMut::new(db.as_hash_db_mut(), &addr1);
			acct_db.insert(b"this is definitely code")
		};

		{
			let mut acct_db = AccountDBMut::new(db.as_hash_db_mut(), &addr2);
			acct_db.emplace(code_hash.clone(), DBValue::from_slice(b"this is definitely code"));
		}

		let account1 = BasicAccount {
			nonce: 50.into(),
			balance: 123456789.into(),
			storage_root: KECCAK_NULL_RLP,
			code_hash: code_hash,
		};

		let account2 = BasicAccount {
			nonce: 400.into(),
			balance: 98765432123456789usize.into(),
			storage_root: KECCAK_NULL_RLP,
			code_hash: code_hash,
		};

		let mut used_code = HashSet::new();

		let fat_rlp1 = to_fat_rlps(&keccak(&addr1), &account1, &AccountDB::new(db.as_hash_db(), &addr1), &mut used_code, usize::max_value(), usize::max_value()).unwrap();
		let fat_rlp2 = to_fat_rlps(&keccak(&addr2), &account2, &AccountDB::new(db.as_hash_db(), &addr2), &mut used_code, usize::max_value(), usize::max_value()).unwrap();
		assert_eq!(used_code.len(), 1);

		let fat_rlp1 = Rlp::new(&fat_rlp1[0]).at(1).unwrap();
		let fat_rlp2 = Rlp::new(&fat_rlp2[0]).at(1).unwrap();

		let (acc, maybe_code) = from_fat_rlp(&mut AccountDBMut::new(db.as_hash_db_mut(), &addr2), fat_rlp2, H256::zero()).unwrap();
		assert!(maybe_code.is_none());
		assert_eq!(acc, account2);

		let (acc, maybe_code) = from_fat_rlp(&mut AccountDBMut::new(db.as_hash_db_mut(), &addr1), fat_rlp1, H256::zero()).unwrap();
		assert_eq!(maybe_code, Some(b"this is definitely code".to_vec()));
		assert_eq!(acc, account1);
	}

	#[test]
	fn encoding_empty_acc() {
		let mut db = get_temp_state_db();
		assert_eq!(from_fat_rlp(&mut AccountDBMut::new(db.as_hash_db_mut(), &Address::default()), Rlp::new(&::rlp::NULL_RLP), H256::zero()).unwrap(), (ACC_EMPTY, None));
	}
}
