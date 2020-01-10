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

//! Tests for account state encoding and decoding

use std::collections::HashSet;

use account_db::{AccountDB, AccountDBMut};
use common_types::{
	basic_account::BasicAccount,
	snapshot::Progress
};
use ethcore::test_helpers::get_temp_state_db;
use ethereum_types::{H256, Address};
use hash_db::{HashDB, EMPTY_PREFIX};
use keccak_hash::{KECCAK_EMPTY, KECCAK_NULL_RLP, keccak};
use parking_lot::RwLock;
use rlp::Rlp;
use snapshot::test_helpers::{ACC_EMPTY, to_fat_rlps, from_fat_rlp};

use crate::helpers::fill_storage;

#[test]
fn encoding_basic() {
	let mut db = get_temp_state_db();
	let addr = Address::random();

	let account = BasicAccount {
		nonce: 50.into(),
		balance: 123456789.into(),
		storage_root: KECCAK_NULL_RLP,
		code_hash: KECCAK_EMPTY,
		code_version: 0.into(),
	};

	let thin_rlp = ::rlp::encode(&account);
	assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp).unwrap(), account);
	let p = RwLock::new(Progress::new());
	let fat_rlps = to_fat_rlps(&keccak(&addr), &account, &AccountDB::from_hash(db.as_hash_db(), keccak(addr)), &mut Default::default(), usize::max_value(), usize::max_value(), &p).unwrap();
	let fat_rlp = Rlp::new(&fat_rlps[0]).at(1).unwrap();
	assert_eq!(from_fat_rlp(&mut AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr)), fat_rlp, H256::zero()).unwrap().0, account);
}

#[test]
fn encoding_version() {
	let mut db = get_temp_state_db();
	let addr = Address::random();

	let account = BasicAccount {
		nonce: 50.into(),
		balance: 123456789.into(),
		storage_root: KECCAK_NULL_RLP,
		code_hash: KECCAK_EMPTY,
		code_version: 1.into(),
	};

	let thin_rlp = ::rlp::encode(&account);
	assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp).unwrap(), account);
	let p = RwLock::new(Progress::new());
	let fat_rlps = to_fat_rlps(&keccak(&addr), &account, &AccountDB::from_hash(db.as_hash_db(), keccak(addr)), &mut Default::default(), usize::max_value(), usize::max_value(), &p).unwrap();
	let fat_rlp = Rlp::new(&fat_rlps[0]).at(1).unwrap();
	assert_eq!(from_fat_rlp(&mut AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr)), fat_rlp, H256::zero()).unwrap().0, account);
}

#[test]
fn encoding_storage() {
	let mut db = get_temp_state_db();
	let addr = Address::random();

	let account = {
		let acct_db = AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr));
		let mut root = KECCAK_NULL_RLP;
		fill_storage(acct_db, &mut root, &mut H256::zero());
		BasicAccount {
			nonce: 25.into(),
			balance: 987654321.into(),
			storage_root: root,
			code_hash: KECCAK_EMPTY,
			code_version: 0.into(),
		}
	};

	let thin_rlp = ::rlp::encode(&account);
	assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp).unwrap(), account);

	let p = RwLock::new(Progress::new());

	let fat_rlp = to_fat_rlps(&keccak(&addr), &account, &AccountDB::from_hash(db.as_hash_db(), keccak(addr)), &mut Default::default(), usize::max_value(), usize::max_value(), &p).unwrap();
	let fat_rlp = Rlp::new(&fat_rlp[0]).at(1).unwrap();
	assert_eq!(from_fat_rlp(&mut AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr)), fat_rlp, H256::zero()).unwrap().0, account);
}

#[test]
fn encoding_storage_split() {
	let mut db = get_temp_state_db();
	let addr = Address::random();

	let account = {
		let acct_db = AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr));
		let mut root = KECCAK_NULL_RLP;
		fill_storage(acct_db, &mut root, &mut H256::zero());
		BasicAccount {
			nonce: 25.into(),
			balance: 987654321.into(),
			storage_root: root,
			code_hash: KECCAK_EMPTY,
			code_version: 0.into(),
		}
	};

	let thin_rlp = ::rlp::encode(&account);
	assert_eq!(::rlp::decode::<BasicAccount>(&thin_rlp).unwrap(), account);

	let p = RwLock::new(Progress::new());
	let fat_rlps = to_fat_rlps(&keccak(addr), &account, &AccountDB::from_hash(db.as_hash_db(), keccak(addr)), &mut Default::default(), 500, 1000, &p).unwrap();
	let mut root = KECCAK_NULL_RLP;
	let mut restored_account = None;
	for rlp in fat_rlps {
		let fat_rlp = Rlp::new(&rlp).at(1).unwrap();
		restored_account = Some(from_fat_rlp(&mut AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr)), fat_rlp, root).unwrap().0);
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
		let mut acct_db = AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr1));
		acct_db.insert(EMPTY_PREFIX, b"this is definitely code")
	};

	{
		let mut acct_db = AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr2));
		acct_db.emplace(code_hash.clone(), EMPTY_PREFIX, b"this is definitely code".to_vec());
	}

	let account1 = BasicAccount {
		nonce: 50.into(),
		balance: 123456789.into(),
		storage_root: KECCAK_NULL_RLP,
		code_hash,
		code_version: 0.into(),
	};

	let account2 = BasicAccount {
		nonce: 400.into(),
		balance: 98765432123456789usize.into(),
		storage_root: KECCAK_NULL_RLP,
		code_hash,
		code_version: 0.into(),
	};

	let mut used_code = HashSet::new();
	let p1 = RwLock::new(Progress::new());
	let p2 = RwLock::new(Progress::new());
	let fat_rlp1 = to_fat_rlps(&keccak(&addr1), &account1, &AccountDB::from_hash(db.as_hash_db(), keccak(addr1)), &mut used_code, usize::max_value(), usize::max_value(), &p1).unwrap();
	let fat_rlp2 = to_fat_rlps(&keccak(&addr2), &account2, &AccountDB::from_hash(db.as_hash_db(), keccak(addr2)), &mut used_code, usize::max_value(), usize::max_value(), &p2).unwrap();
	assert_eq!(used_code.len(), 1);

	let fat_rlp1 = Rlp::new(&fat_rlp1[0]).at(1).unwrap();
	let fat_rlp2 = Rlp::new(&fat_rlp2[0]).at(1).unwrap();

	let (acc, maybe_code) = from_fat_rlp(&mut AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr2)), fat_rlp2, H256::zero()).unwrap();
	assert!(maybe_code.is_none());
	assert_eq!(acc, account2);

	let (acc, maybe_code) = from_fat_rlp(&mut AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(addr1)), fat_rlp1, H256::zero()).unwrap();
	assert_eq!(maybe_code, Some(b"this is definitely code".to_vec()));
	assert_eq!(acc, account1);
}

#[test]
fn encoding_empty_acc() {
	let mut db = get_temp_state_db();
	assert_eq!(from_fat_rlp(&mut AccountDBMut::from_hash(db.as_hash_db_mut(), keccak(Address::zero())), Rlp::new(&::rlp::NULL_RLP), H256::zero()).unwrap(), (ACC_EMPTY, None));
}
