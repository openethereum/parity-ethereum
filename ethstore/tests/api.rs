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

extern crate rand;
extern crate ethstore;

mod util;

use std::str::FromStr;
use ethstore::{SecretStore, EthStore};
use ethstore::ethkey::{Random, Generator, Secret, Address};
use ethstore::dir::DiskDirectory;
use util::TransientDir;

#[test]
fn secret_store_create() {
	let dir = TransientDir::create().unwrap();
	let _ = EthStore::open(Box::new(dir)).unwrap();
}

#[test]
#[should_panic]
fn secret_store_open_not_existing() {
	let dir = TransientDir::open();
	let _ = EthStore::open(Box::new(dir)).unwrap();
}

fn random_secret() -> Secret {
	Random.generate().unwrap().secret().clone()
}

#[test]
fn secret_store_create_account() {
	let dir = TransientDir::create().unwrap();
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert_eq!(store.accounts().len(), 0);
	assert!(store.insert_account(random_secret(), "").is_ok());
	assert_eq!(store.accounts().len(), 1);
	assert!(store.insert_account(random_secret(), "").is_ok());
	assert_eq!(store.accounts().len(), 2);
}

#[test]
fn secret_store_sign() {
	let dir = TransientDir::create().unwrap();
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert!(store.insert_account(random_secret(), "").is_ok());
	let accounts = store.accounts();
	assert_eq!(accounts.len(), 1);
	assert!(store.sign(&accounts[0], "", &Default::default()).is_ok());
	assert!(store.sign(&accounts[0], "1", &Default::default()).is_err());
}

#[test]
fn secret_store_change_password() {
	let dir = TransientDir::create().unwrap();
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert!(store.insert_account(random_secret(), "").is_ok());
	let accounts = store.accounts();
	assert_eq!(accounts.len(), 1);
	assert!(store.sign(&accounts[0], "", &Default::default()).is_ok());
	assert!(store.change_password(&accounts[0], "", "1").is_ok());
	assert!(store.sign(&accounts[0], "", &Default::default()).is_err());
	assert!(store.sign(&accounts[0], "1", &Default::default()).is_ok());
}

#[test]
fn secret_store_remove_account() {
	let dir = TransientDir::create().unwrap();
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert!(store.insert_account(random_secret(), "").is_ok());
	let accounts = store.accounts();
	assert_eq!(accounts.len(), 1);
	assert!(store.remove_account(&accounts[0], "").is_ok());
	assert_eq!(store.accounts().len(), 0);
	assert!(store.remove_account(&accounts[0], "").is_err());
}

fn test_path() -> &'static str {
	match ::std::fs::metadata("ethstore") {
		Ok(_) => "ethstore/tests/res/geth_keystore",
 		Err(_) => "tests/res/geth_keystore",
	}
}

fn pat_path() -> &'static str {
	match ::std::fs::metadata("ethstore") {
		Ok(_) => "ethstore/tests/res/pat",
		Err(_) => "tests/res/pat",
	}
}

#[test]
fn secret_store_laod_geth_files() {
	let dir = DiskDirectory::at(test_path());
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert_eq!(store.accounts(), vec![
		Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap(),
		Address::from_str("5ba4dcf897e97c2bdf8315b9ef26c13c085988cf").unwrap(),
		Address::from_str("63121b431a52f8043c16fcf0d1df9cb7b5f66649").unwrap(),
	]);
}

#[test]
fn secret_store_load_pat_files() {
	let dir = DiskDirectory::at(pat_path());
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert_eq!(store.accounts(), vec![
		Address::from_str("3f49624084b67849c7b4e805c5988c21a430f9d9").unwrap(),
		Address::from_str("5ba4dcf897e97c2bdf8315b9ef26c13c085988cf").unwrap(),
	]);
}


