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

//! Generic Encryptor

use keys::directory::*;
use common::*;
use rcrypto::pbkdf2::*;
use rcrypto::aes;
use rcrypto::hmac::*;
use crypto;

const KEY_LENGTH: u32 = 32;
const KEY_ITERATIONS: u32 = 4096;
const KEY_LENGTH_AES: u32 = KEY_LENGTH/2;

const KEY_LENGTH_USIZE: usize = KEY_LENGTH as usize;
const KEY_LENGTH_AES_USIZE: usize = KEY_LENGTH_AES as usize;

pub trait EncryptedHashMap<Key: Hash + Eq> {
	// Returns existing value for the key, if any
	fn get<Value: Populatable + Default + BytesConvertable>(&self, key: &Key, password: &str) -> Option<Value>;
	// Insert new encrypted key-value and returns previous if there was any
	fn insert<Value: Populatable + Default + BytesConvertable>(&mut self, key: Key, value: Value, password: &str) -> Option<Value>;
	// Removes key-value by key and returns the removed one, if any exists and password was provided
	fn remove<Value: Populatable + Default + BytesConvertable> (&mut self, key: &Key, password: Option<&str>) -> Option<Value>;
	// Deletes key-value by key and returns if the key-value existed
	fn delete(&mut self, key: &Key) -> bool {
		self.remove::<&[u8]>(key, None).is_some()
	}
}

pub struct SecretStore {
	directory: KeyDirectory
}

impl SecretStore {
	fn new() -> SecretStore {
		let mut path = ::std::env::home_dir().expect("Failed to get home dir");
		path.push(".keys");
		SecretStore {
			directory: KeyDirectory::new(&path)
		}
	}

	#[cfg(test)]
	fn new_test(path: &::tests::helpers::RandomTempPath) -> SecretStore {
		SecretStore {
			directory: KeyDirectory::new(path.as_path())
		}
	}
}

fn derive_key_iterations(password: &str, salt: &H256, c: u32) -> (Bytes, Bytes) {
	let mut h_mac = Hmac::new(::rcrypto::sha2::Sha256::new(), password.as_bytes());
	let mut derived_key = vec![0u8; KEY_LENGTH_USIZE];
	pbkdf2(&mut h_mac, &salt.as_slice(), c, &mut derived_key);
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES_USIZE];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES_USIZE..KEY_LENGTH_USIZE];
	(derived_right_bits.to_vec(), derived_left_bits.to_vec())
}

fn derive_key(password: &str, salt: &H256) -> (Bytes, Bytes) {
	derive_key_iterations(password, salt, KEY_ITERATIONS)
}

fn derive_mac(derived_left_bits: &[u8], cipher_text: &[u8]) -> Bytes {
	let mut mac = vec![0u8; KEY_LENGTH_AES_USIZE + cipher_text.len()];
	mac[0..KEY_LENGTH_AES_USIZE].clone_from_slice(derived_left_bits);
	mac[KEY_LENGTH_AES_USIZE..cipher_text.len()+KEY_LENGTH_AES_USIZE].clone_from_slice(cipher_text);
	mac
}

impl EncryptedHashMap<H128> for SecretStore {
	fn get<Value: Populatable + Default + BytesConvertable>(&self, key: &H128, password: &str) -> Option<Value> {
		match self.directory.get(key) {
			Some(key_file) => {
				let mut instance = Value::default();
				instance.populate_raw(&key_file.crypto.cipher_text);
				Some(instance)
			},
			None => None
		}

	}

	fn insert<Value: Populatable + Default + BytesConvertable>(&mut self, key: H128, value: Value, password: &str) -> Option<Value> {
		let previous = if let Some(_) = self.directory.get(&key) { self.get(&key, password) } else { None };

		let salt = H256::random();
		let iv = H128::random();
		let mut key_file = KeyFileContent::new(KeyFileCrypto::new_pbkdf2(vec![], iv.clone(), salt.clone(), KEY_ITERATIONS, KEY_LENGTH));

		let (derived_left_bits, derived_right_bits) = derive_key(password, &salt);

		let mut cipher_text = vec![0u8; value.as_slice().len()];
		crypto::aes::encrypt(&derived_left_bits, &iv.as_slice(), &value.as_slice(), &mut cipher_text);
		key_file.crypto.cipher_text = cipher_text.clone();

		key_file.crypto.mac = derive_mac(&derived_right_bits, &cipher_text).sha3();

		previous
	}

	fn remove<Value: Populatable + Default + BytesConvertable>(&mut self, key: &H128, password: Option<&str>) -> Option<Value> {
		let previous = match (self.directory.get(&key), password) {
			(Some(_), Some(pass)) => self.get(&key, pass),
			(_, _) => None
		};
		previous
	}

}

#[cfg(test)]
mod vector_tests {
	use super::{derive_key,derive_mac,derive_key_iterations};
	use common::*;


	#[test]
	fn mac_vector() {
		let password = "testpassword";
		let salt = H256::from_str("ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd").unwrap();
		let cipher_text = FromHex::from_hex("5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46").unwrap();
		let iterations = 262144u32;

		let (derived_left_bits, derived_right_bits) = derive_key_iterations(password, &salt, iterations);
		assert_eq!("f06d69cdc7da0faffb1008270bca38f5", derived_left_bits.to_hex());
		assert_eq!("e31891a3a773950e6d0fea48a7188551", derived_right_bits.to_hex());

		let mut mac_body = derive_mac(&derived_right_bits, &cipher_text);
		assert_eq!("e31891a3a773950e6d0fea48a71885515318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46", mac_body.to_hex());

		let mac = mac_body.sha3();
		assert_eq!("517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2", format!("{:?}", mac));
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use tests::helpers::*;
	use common::*;

	#[test]
	fn secret_store_insert() {
		let temp = RandomTempPath::create_dir();
		let mut sstore = SecretStore::new_test(&temp);

		sstore.insert(H128::random(), "Cat".to_owned(), "pass");
	}



}
