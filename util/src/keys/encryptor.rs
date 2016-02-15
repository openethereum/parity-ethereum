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

pub trait EncryptedHashMap<Key: Hash + Eq> {
	// Returns existing value for the key, if any
	fn get<Value: Populatable + Default>(&self, key: &Key, password: &str) -> Option<Value>;
	// Insert new encrypted key-value and returns previous if there was any
	fn insert<Value: Populatable + Default>(&mut self, key: Key, value: Value, password: &str) -> Option<Value>;
	// Removes key-value by key and returns the removed one, if any exists and password was provided
	fn remove<Value: Populatable + Default> (&mut self, key: &Key, password: Option<&str>) -> Option<Value>;
	// Deletes key-value by key and returns if the key-value existed
	fn delete(&mut self, key: &Key) -> bool {
		self.remove::<()>(key, None).is_some()
	}
}

pub struct SecretStore {
	directory: KeyDirectory
}

impl EncryptedHashMap<H128> for SecretStore {
	fn get<Value: Populatable + Default>(&self, key: &H128, password: &str) -> Option<Value> {
		match self.directory.get(key) {
			Some(key_file) => {
				let mut instance = Value::default();
				instance.populate_raw(&key_file.crypto.cipher_text);
				Some(instance)
			},
			None => None
		}

	}

	fn insert<Value: Populatable + Default>(&mut self, key: H128, value: Value, password: &str) -> Option<Value>{
		let previous = if let Some(key_file) = self.directory.get(&key) { self.get(&key, password) } else { None };
		previous
	}

	fn remove<Value: Populatable + Default>(&mut self, key: &H128, password: Option<&str>) -> Option<Value> {
		let previous = match (self.directory.get(&key), password) {
			(Some(key_file), Some(pass)) => self.get(&key, pass),
			(_, _) => None
		};
		previous
	}

}
