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

		let mut mac = Hmac::new(::rcrypto::sha2::Sha256::new(), password.as_bytes());
		let mut derived_key = vec![0u8; KEY_LENGTH as usize];
		pbkdf2(&mut mac, &salt.as_slice(), KEY_ITERATIONS, &mut derived_key);
		let key = &derived_key[KEY_LENGTH_AES as usize..KEY_LENGTH as usize];

		let mut cipher_text = vec![0u8; value.as_slice().len()];
		crypto::aes::encrypt(&key, &iv.as_slice(), &value.as_slice(), &mut cipher_text);
		key_file.crypto.cipher_text = cipher_text;

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
