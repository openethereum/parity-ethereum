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

extern crate rand;
extern crate ethstore;

mod util;

use ethstore::{EthStore, SimpleSecretStore, SecretVaultRef, StoreAccountRef};
use ethstore::ethkey::{Random, Generator, Secret, KeyPair, verify_address};
use ethstore::accounts_dir::RootDiskDirectory;
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
	assert_eq!(store.accounts().unwrap().len(), 0);
	assert!(store.insert_account(SecretVaultRef::Root, random_secret(), "").is_ok());
	assert_eq!(store.accounts().unwrap().len(), 1);
	assert!(store.insert_account(SecretVaultRef::Root, random_secret(), "").is_ok());
	assert_eq!(store.accounts().unwrap().len(), 2);
}

#[test]
fn secret_store_sign() {
	let dir = TransientDir::create().unwrap();
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert!(store.insert_account(SecretVaultRef::Root, random_secret(), "").is_ok());
	let accounts = store.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	assert!(store.sign(&accounts[0], "", &Default::default()).is_ok());
	assert!(store.sign(&accounts[0], "1", &Default::default()).is_err());
}

#[test]
fn secret_store_change_password() {
	let dir = TransientDir::create().unwrap();
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert!(store.insert_account(SecretVaultRef::Root, random_secret(), "").is_ok());
	let accounts = store.accounts().unwrap();
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
	assert!(store.insert_account(SecretVaultRef::Root, random_secret(), "").is_ok());
	let accounts = store.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	assert!(store.remove_account(&accounts[0], "").is_ok());
	assert_eq!(store.accounts().unwrap().len(), 0);
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

fn ciphertext_path() -> &'static str {
	match ::std::fs::metadata("ethstore") {
		Ok(_) => "ethstore/tests/res/ciphertext",
		Err(_) => "tests/res/ciphertext",
	}
}

#[test]
fn secret_store_laod_geth_files() {
	let dir = RootDiskDirectory::at(test_path());
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert_eq!(store.accounts().unwrap(), vec![
		StoreAccountRef::root("3f49624084b67849c7b4e805c5988c21a430f9d9".into()),
		StoreAccountRef::root("5ba4dcf897e97c2bdf8315b9ef26c13c085988cf".into()),
		StoreAccountRef::root("63121b431a52f8043c16fcf0d1df9cb7b5f66649".into()),
	]);
}

#[test]
fn secret_store_load_pat_files() {
	let dir = RootDiskDirectory::at(pat_path());
	let store = EthStore::open(Box::new(dir)).unwrap();
	assert_eq!(store.accounts().unwrap(), vec![
		StoreAccountRef::root("3f49624084b67849c7b4e805c5988c21a430f9d9".into()),
		StoreAccountRef::root("5ba4dcf897e97c2bdf8315b9ef26c13c085988cf".into()),
	]);
}

#[test]
fn test_decrypting_files_with_short_ciphertext() {
	// 31e9d1e6d844bd3a536800ef8d8be6a9975db509, 30
	let kp1 = KeyPair::from_secret("000081c29e8142bb6a81bef5a92bda7a8328a5c85bb2f9542e76f9b0f94fc018".parse().unwrap()).unwrap();
	// d1e64e5480bfaf733ba7d48712decb8227797a4e , 31
	let kp2 = KeyPair::from_secret("00fa7b3db73dc7dfdf8c5fbdb796d741e4488628c41fc4febd9160a866ba0f35".parse().unwrap()).unwrap();
	let dir = RootDiskDirectory::at(ciphertext_path());
	let store = EthStore::open(Box::new(dir)).unwrap();
	let accounts = store.accounts().unwrap();
	assert_eq!(accounts, vec![
		StoreAccountRef::root("31e9d1e6d844bd3a536800ef8d8be6a9975db509".into()),
		StoreAccountRef::root("d1e64e5480bfaf733ba7d48712decb8227797a4e".into()),
	]);

	let message = Default::default();

	let s1 = store.sign(&accounts[0], "foo", &message).unwrap();
	let s2 = store.sign(&accounts[1], "foo", &message).unwrap();
	assert!(verify_address(&accounts[0].address, &s1, &message).unwrap());
	assert!(verify_address(&kp1.address(), &s1, &message).unwrap());
	assert!(verify_address(&kp2.address(), &s2, &message).unwrap());
}
