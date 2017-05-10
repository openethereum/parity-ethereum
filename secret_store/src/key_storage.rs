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
use std::collections::BTreeMap;
use serde_json;
use ethkey::{Secret, Public};
use util::Database;
use types::all::{Error, ServiceConfiguration, DocumentAddress, NodeId};
use serialization::{SerializablePublic, SerializableSecret};

#[derive(Debug, Clone, PartialEq)]
/// Encrypted key share, stored by key storage on the single key server.
pub struct DocumentKeyShare {
	/// Decryption threshold (at least threshold + 1 nodes are required to decrypt data).
	pub threshold: usize,
	/// Nodes ids numbers.
	pub id_numbers: BTreeMap<NodeId, Secret>,
	/// Node secret share.
	pub secret_share: Secret,
	/// Common (shared) encryption point.
	pub common_point: Public,
	/// Encrypted point.
	pub encrypted_point: Public,
}

/// Document encryption keys storage
pub trait KeyStorage: Send + Sync {
	/// Insert document encryption key
	fn insert(&self, document: DocumentAddress, key: DocumentKeyShare) -> Result<(), Error>;
	/// Get document encryption key
	fn get(&self, document: &DocumentAddress) -> Result<DocumentKeyShare, Error>;
	/// Check if storage contains document encryption key
	fn contains(&self, document: &DocumentAddress) -> bool;
}

/// Persistent document encryption keys storage
pub struct PersistentKeyStorage {
	db: Database,
}

#[derive(Serialize, Deserialize)]
/// Encrypted key share, as it is stored by key storage on the single key server.
struct SerializableDocumentKeyShare {
	/// Decryption threshold (at least threshold + 1 nodes are required to decrypt data).
	pub threshold: usize,
	/// Nodes ids numbers.
	pub id_numbers: BTreeMap<SerializablePublic, SerializableSecret>,
	/// Node secret share.
	pub secret_share: SerializableSecret,
	/// Common (shared) encryption point.
	pub common_point: SerializablePublic,
	/// Encrypted point.
	pub encrypted_point: SerializablePublic,
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
	fn insert(&self, document: DocumentAddress, key: DocumentKeyShare) -> Result<(), Error> {
		let key: SerializableDocumentKeyShare = key.into();
		let key = serde_json::to_vec(&key).map_err(|e| Error::Database(e.to_string()))?;
		let mut batch = self.db.transaction();
		batch.put(None, &document, &key);
		self.db.write(batch).map_err(Error::Database)
	}

	fn get(&self, document: &DocumentAddress) -> Result<DocumentKeyShare, Error> {
		self.db.get(None, document)
			.map_err(Error::Database)?
			.ok_or(Error::DocumentNotFound)
			.map(|key| key.to_vec())
			.and_then(|key| serde_json::from_slice::<SerializableDocumentKeyShare>(&key).map_err(|e| Error::Database(e.to_string())))
			.map(Into::into)
	}

	fn contains(&self, document: &DocumentAddress) -> bool {
		self.db.get(None, document)
			.map(|k| k.is_some())
			.unwrap_or(false)
	}
}

impl From<DocumentKeyShare> for SerializableDocumentKeyShare {
	fn from(key: DocumentKeyShare) -> Self {
		SerializableDocumentKeyShare {
			threshold: key.threshold,
			id_numbers: key.id_numbers.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
			secret_share: key.secret_share.into(),
			common_point: key.common_point.into(),
			encrypted_point: key.encrypted_point.into(),
		}
	}
}

impl From<SerializableDocumentKeyShare> for DocumentKeyShare {
	fn from(key: SerializableDocumentKeyShare) -> Self {
		DocumentKeyShare {
			threshold: key.threshold,
			id_numbers: key.id_numbers.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
			secret_share: key.secret_share.into(),
			common_point: key.common_point.into(),
			encrypted_point: key.encrypted_point.into(),
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::{BTreeMap, HashMap};
	use parking_lot::RwLock;
	use devtools::RandomTempPath;
	use ethkey::{Random, Generator};
	use super::super::types::all::{Error, NodeAddress, ServiceConfiguration, ClusterConfiguration, DocumentAddress};
	use super::{KeyStorage, PersistentKeyStorage, DocumentKeyShare};

	#[derive(Default)]
	/// In-memory document encryption keys storage
	pub struct DummyKeyStorage {
		keys: RwLock<HashMap<DocumentAddress, DocumentKeyShare>>,
	}

	impl KeyStorage for DummyKeyStorage {
		fn insert(&self, document: DocumentAddress, key: DocumentKeyShare) -> Result<(), Error> {
			self.keys.write().insert(document, key);
			Ok(())
		}

		fn get(&self, document: &DocumentAddress) -> Result<DocumentKeyShare, Error> {
			self.keys.read().get(document).cloned().ok_or(Error::DocumentNotFound)
		}

		fn contains(&self, document: &DocumentAddress) -> bool {
			self.keys.read().contains_key(document)
		}
	}

	#[test]
	fn persistent_key_storage() {
		let path = RandomTempPath::create_dir();
		let config = ServiceConfiguration {
			listener_address: NodeAddress {
				address: "0.0.0.0".to_owned(),
				port: 8082,
			},
			data_path: path.as_str().to_owned(),
			cluster_config: ClusterConfiguration {
				threads: 1,
				self_private: (**Random.generate().unwrap().secret().clone()).into(),
				listener_address: NodeAddress {
					address: "0.0.0.0".to_owned(),
					port: 8083,
				},
				nodes: BTreeMap::new(),
				allow_connecting_to_higher_nodes: false,
			},
		};
		
		let key1 = DocumentAddress::from(1);
		let value1 = DocumentKeyShare {
			threshold: 100,
			id_numbers: vec![
				(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone())
			].into_iter().collect(),
			secret_share: Random.generate().unwrap().secret().clone(),
			common_point: Random.generate().unwrap().public().clone(),
			encrypted_point: Random.generate().unwrap().public().clone(),
		};
		let key2 = DocumentAddress::from(2);
		let value2 = DocumentKeyShare {
			threshold: 200,
			id_numbers: vec![
				(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone())
			].into_iter().collect(),
			secret_share: Random.generate().unwrap().secret().clone(),
			common_point: Random.generate().unwrap().public().clone(),
			encrypted_point: Random.generate().unwrap().public().clone(),
		};
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
