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

use std::path::PathBuf;
use util::Database;
use types::all::{Error, ServiceConfiguration, DocumentAddress, DocumentKey};

/// Document encryption keys storage
pub trait KeyStorage: Send + Sync {
	/// Insert document encryption key
	fn insert(&self, document: DocumentAddress, key: DocumentKey) -> Result<(), Error>;
	/// Get document encryption key
	fn get(&self, document: &DocumentAddress) -> Result<DocumentKey, Error>;
}

/// Persistent document encryption keys storage
pub struct PersistentKeyStorage {
	db: Database,
}

impl PersistentKeyStorage {
	/// Create new persistent document encryption keys storage
	pub fn new(config: &ServiceConfiguration) -> Result<Self, Error> {
		let mut db_path = PathBuf::from(&config.data_path);
		db_path.push("db");
		let db_path = db_path.to_str().ok_or(Error::Database("Invalid secretstore path".to_owned()))?;

		Ok(PersistentKeyStorage {
			db: Database::open_default(&db_path).map_err(Error::Database)?,
		})
	}
}

impl KeyStorage for PersistentKeyStorage {
	fn insert(&self, document: DocumentAddress, key: DocumentKey) -> Result<(), Error> {
		let mut batch = self.db.transaction();
		batch.put(None, &document, &key);
		self.db.write(batch).map_err(Error::Database)
	}

	fn get(&self, document: &DocumentAddress) -> Result<DocumentKey, Error> {
		self.db.get(None, document)
			.map_err(Error::Database)?
			.ok_or(Error::DocumentNotFound)
			.map(|key| key.to_vec())
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::HashMap;
	use parking_lot::RwLock;
	use devtools::RandomTempPath;
	use super::super::types::all::{Error, ServiceConfiguration, DocumentAddress, DocumentKey};
	use super::{KeyStorage, PersistentKeyStorage};

	#[derive(Default)]
	/// In-memory document encryption keys storage
	pub struct DummyKeyStorage {
		keys: RwLock<HashMap<DocumentAddress, DocumentKey>>,
	}

	impl KeyStorage for DummyKeyStorage {
		fn insert(&self, document: DocumentAddress, key: DocumentKey) -> Result<(), Error> {
			self.keys.write().insert(document, key);
			Ok(())
		}

		fn get(&self, document: &DocumentAddress) -> Result<DocumentKey, Error> {
			self.keys.read().get(document).cloned().ok_or(Error::DocumentNotFound)
		}
	}

	#[test]
	fn persistent_key_storage() {
		let path = RandomTempPath::create_dir();
		let config = ServiceConfiguration {
			listener_addr: "0.0.0.0".to_owned(),
			listener_port: 8082,
			data_path: path.as_str().to_owned(),
		};
		
		let key1 = DocumentAddress::from(1);
		let value1: DocumentKey = vec![0x77, 0x88];
		let key2 = DocumentAddress::from(2);
		let value2: DocumentKey = vec![0x11, 0x22];
		let key3 = DocumentAddress::from(3);

		let key_storage = PersistentKeyStorage::new(&config).unwrap();
		key_storage.insert(key1.clone(), value1.clone()).unwrap();
		key_storage.insert(key2.clone(), value2.clone()).unwrap();
		assert_eq!(key_storage.get(&key1), Ok(value1.clone()));
		assert_eq!(key_storage.get(&key2), Ok(value2.clone()));
		assert_eq!(key_storage.get(&key3), Err(Error::DocumentNotFound));
		drop(key_storage);

		let key_storage = PersistentKeyStorage::new(&config).unwrap();
		assert_eq!(key_storage.get(&key1), Ok(value1));
		assert_eq!(key_storage.get(&key2), Ok(value2));
		assert_eq!(key_storage.get(&key3), Err(Error::DocumentNotFound));
	}
}
