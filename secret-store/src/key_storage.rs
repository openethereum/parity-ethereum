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

use std::collections::BTreeMap;
use std::sync::Arc;
use serde_json;
use tiny_keccak::Keccak;
use ethereum_types::{H256, Address};
use crypto::publickey::{Secret, Public};
use kvdb::KeyValueDB;
use types::{Error, ServerKeyId, NodeId};
use serialization::{SerializablePublic, SerializableSecret, SerializableH256, SerializableAddress};

/// Key of version value.
const DB_META_KEY_VERSION: &'static [u8; 7] = b"version";
/// Current db version.
const CURRENT_VERSION: u8 = 3;

/// Encrypted key share, stored by key storage on the single key server.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DocumentKeyShare {
	/// Author of the entry.
	pub author: Address,
	/// Decryption threshold (at least threshold + 1 nodes are required to decrypt data).
	pub threshold: usize,
	/// Server public key.
	pub public: Public,
	/// Common (shared) encryption point.
	pub common_point: Option<Public>,
	/// Encrypted point.
	pub encrypted_point: Option<Public>,
	/// Key share versions.
	pub versions: Vec<DocumentKeyShareVersion>,
}

/// Versioned portion of document key share.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentKeyShareVersion {
	/// Version hash (Keccak(time + id_numbers)).
	pub hash: H256,
	/// Nodes ids numbers.
	pub id_numbers: BTreeMap<NodeId, Secret>,
	/// Node secret share.
	pub secret_share: Secret,
}

/// Document encryption keys storage
pub trait KeyStorage: Send + Sync {
	/// Insert document encryption key
	fn insert(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error>;
	/// Update document encryption key
	fn update(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error>;
	/// Get document encryption key
	fn get(&self, document: &ServerKeyId) -> Result<Option<DocumentKeyShare>, Error>;
	/// Remove document encryption key
	fn remove(&self, document: &ServerKeyId) -> Result<(), Error>;
	/// Clears the database
	fn clear(&self) -> Result<(), Error>;
	/// Check if storage contains document encryption key
	fn contains(&self, document: &ServerKeyId) -> bool;
	/// Iterate through storage
	fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=(ServerKeyId, DocumentKeyShare)> + 'a>;
}

/// Persistent document encryption keys storage
pub struct PersistentKeyStorage {
	db: Arc<dyn KeyValueDB>,
}

/// Persistent document encryption keys storage iterator
pub struct PersistentKeyStorageIterator<'a> {
	iter: Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>,
}

/// V3 of encrypted key share, as it is stored by key storage on the single key server.
#[derive(Serialize, Deserialize)]
struct SerializableDocumentKeyShareV3 {
	/// Author of the entry.
	pub author: SerializableAddress,
	/// Decryption threshold (at least threshold + 1 nodes are required to decrypt data).
	pub threshold: usize,
	/// Server public.
	pub public: SerializablePublic,
	/// Common (shared) encryption point.
	pub common_point: Option<SerializablePublic>,
	/// Encrypted point.
	pub encrypted_point: Option<SerializablePublic>,
	/// Versions.
	pub versions: Vec<SerializableDocumentKeyShareVersionV3>
}

/// V3 of encrypted key share version, as it is stored by key storage on the single key server.
#[derive(Serialize, Deserialize)]
struct SerializableDocumentKeyShareVersionV3 {
	/// Version hash.
	pub hash: SerializableH256,
	/// Nodes ids numbers.
	pub id_numbers: BTreeMap<SerializablePublic, SerializableSecret>,
	/// Node secret share.
	pub secret_share: SerializableSecret,
}

impl PersistentKeyStorage {
	/// Create new persistent document encryption keys storage
	pub fn new(db: Arc<dyn KeyValueDB>) -> Result<Self, Error> {
		let db = upgrade_db(db)?;

		Ok(PersistentKeyStorage {
			db: db,
		})
	}
}

fn upgrade_db(db: Arc<dyn KeyValueDB>) -> Result<Arc<dyn KeyValueDB>, Error> {
	let version = db.get(None, DB_META_KEY_VERSION)?;
	let version = version.and_then(|v| v.get(0).cloned());
	match version {
		None => {
			let mut batch = db.transaction();
			batch.put(None, DB_META_KEY_VERSION, &[CURRENT_VERSION]);
			db.write(batch)?;
			Ok(db)
		},
		Some(CURRENT_VERSION) => Ok(db),
		_ => Err(Error::Database(format!("unsupported SecretStore database version: {:?}", version))),
	}
}

impl KeyStorage for PersistentKeyStorage {
	fn insert(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error> {
		let key: SerializableDocumentKeyShareV3 = key.into();
		let key = serde_json::to_vec(&key).map_err(|e| Error::Database(e.to_string()))?;
		let mut batch = self.db.transaction();
		batch.put(None, document.as_bytes(), &key);
		self.db.write(batch).map_err(Into::into)
	}

	fn update(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error> {
		self.insert(document, key)
	}

	fn get(&self, document: &ServerKeyId) -> Result<Option<DocumentKeyShare>, Error> {
		self.db.get(None, document.as_bytes())
			.map_err(|e| Error::Database(e.to_string()))
			.and_then(|key| match key {
				None => Ok(None),
				Some(key) => serde_json::from_slice::<SerializableDocumentKeyShareV3>(&key)
					.map_err(|e| Error::Database(e.to_string()))
					.map(Into::into)
					.map(Some),
			})
	}

	fn remove(&self, document: &ServerKeyId) -> Result<(), Error> {
		let mut batch = self.db.transaction();
		batch.delete(None, document.as_bytes());
		self.db.write(batch).map_err(Into::into)
	}

	fn clear(&self) -> Result<(), Error> {
		let mut batch = self.db.transaction();
		for (key, _) in self.iter() {
			batch.delete(None, key.as_bytes());
		}
		self.db.write(batch)
			.map_err(|e| Error::Database(e.to_string()))
	}

	fn contains(&self, document: &ServerKeyId) -> bool {
		self.db.get(None, document.as_bytes())
			.map(|k| k.is_some())
			.unwrap_or(false)
	}

	fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=(ServerKeyId, DocumentKeyShare)> + 'a> {
		Box::new(PersistentKeyStorageIterator {
			iter: self.db.iter(None),
		})
	}
}

impl<'a> Iterator for PersistentKeyStorageIterator<'a> {
	type Item = (ServerKeyId, DocumentKeyShare);

	fn next(&mut self) -> Option<(ServerKeyId, DocumentKeyShare)> {
		self.iter.as_mut().next()
			.and_then(|(db_key, db_val)| serde_json::from_slice::<SerializableDocumentKeyShareV3>(&db_val)
					  .ok()
					  .map(|key| (ServerKeyId::from_slice(&*db_key), key.into())))
	}
}

impl DocumentKeyShare {
	/// Get last version reference.
	#[cfg(test)]
	pub fn last_version(&self) -> Result<&DocumentKeyShareVersion, Error> {
		self.versions.iter().rev()
			.nth(0)
			.ok_or_else(|| Error::Database("key version is not found".into()))
	}

	/// Get given version reference.
	pub fn version(&self, version: &H256) -> Result<&DocumentKeyShareVersion, Error> {
		self.versions.iter().rev()
			.find(|v| &v.hash == version)
			.ok_or_else(|| Error::Database("key version is not found".into()))
	}
}

impl DocumentKeyShareVersion {
	/// Create new version
	pub fn new(id_numbers: BTreeMap<NodeId, Secret>, secret_share: Secret) -> Self {
		DocumentKeyShareVersion {
			hash: Self::data_hash(id_numbers.iter().map(|(k, v)| (k.as_bytes(), v.as_bytes()))),
			id_numbers: id_numbers,
			secret_share: secret_share,
		}
	}

	/// Calculate hash of given version data.
	pub fn data_hash<'a, I>(id_numbers: I) -> H256 where I: Iterator<Item=(&'a [u8], &'a [u8])> {
		let mut nodes_keccak = Keccak::new_keccak256();

		for (node, node_number) in id_numbers {
			nodes_keccak.update(node);
			nodes_keccak.update(node_number);
		}

		let mut nodes_keccak_value = [0u8; 32];
		nodes_keccak.finalize(&mut nodes_keccak_value);

		nodes_keccak_value.into()
	}
}

impl From<DocumentKeyShare> for SerializableDocumentKeyShareV3 {
	fn from(key: DocumentKeyShare) -> Self {
		SerializableDocumentKeyShareV3 {
			author: key.author.into(),
			threshold: key.threshold,
			public: key.public.into(),
			common_point: key.common_point.map(Into::into),
			encrypted_point: key.encrypted_point.map(Into::into),
			versions: key.versions.into_iter().map(Into::into).collect(),
		}
	}
}

impl From<DocumentKeyShareVersion> for SerializableDocumentKeyShareVersionV3 {
	fn from(version: DocumentKeyShareVersion) -> Self {
		SerializableDocumentKeyShareVersionV3 {
			hash: version.hash.into(),
			id_numbers: version.id_numbers.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
			secret_share: version.secret_share.into(),
		}
	}
}

impl From<SerializableDocumentKeyShareV3> for DocumentKeyShare {
	fn from(key: SerializableDocumentKeyShareV3) -> Self {
		DocumentKeyShare {
			author: key.author.into(),
			threshold: key.threshold,
			public: key.public.into(),
			common_point: key.common_point.map(Into::into),
			encrypted_point: key.encrypted_point.map(Into::into),
			versions: key.versions.into_iter()
				.map(|v| DocumentKeyShareVersion {
					hash: v.hash.into(),
					id_numbers: v.id_numbers.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
					secret_share: v.secret_share.into(),
				})
				.collect(),
		}
	}
}

#[cfg(test)]
pub mod tests {
	extern crate tempdir;

	use std::collections::HashMap;
	use std::sync::Arc;
	use parking_lot::RwLock;
	use self::tempdir::TempDir;
	use crypto::publickey::{Random, Generator, Public};
	use kvdb_rocksdb::Database;
	use types::{Error, ServerKeyId};
	use super::{KeyStorage, PersistentKeyStorage, DocumentKeyShare, DocumentKeyShareVersion};

	/// In-memory document encryption keys storage
	#[derive(Default)]
	pub struct DummyKeyStorage {
		keys: RwLock<HashMap<ServerKeyId, DocumentKeyShare>>,
	}

	impl KeyStorage for DummyKeyStorage {
		fn insert(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error> {
			self.keys.write().insert(document, key);
			Ok(())
		}

		fn update(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error> {
			self.keys.write().insert(document, key);
			Ok(())
		}

		fn get(&self, document: &ServerKeyId) -> Result<Option<DocumentKeyShare>, Error> {
			Ok(self.keys.read().get(document).cloned())
		}

		fn remove(&self, document: &ServerKeyId) -> Result<(), Error> {
			self.keys.write().remove(document);
			Ok(())
		}

		fn clear(&self) -> Result<(), Error> {
			self.keys.write().clear();
			Ok(())
		}

		fn contains(&self, document: &ServerKeyId) -> bool {
			self.keys.read().contains_key(document)
		}

		fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=(ServerKeyId, DocumentKeyShare)> + 'a> {
			Box::new(self.keys.read().clone().into_iter())
		}
	}

	#[test]
	fn persistent_key_storage() {
		let tempdir = TempDir::new("").unwrap();
		let key1 = ServerKeyId::from_low_u64_be(1);
		let value1 = DocumentKeyShare {
			author: Default::default(),
			threshold: 100,
			public: Public::default(),
			common_point: Some(Random.generate().unwrap().public().clone()),
			encrypted_point: Some(Random.generate().unwrap().public().clone()),
			versions: vec![DocumentKeyShareVersion {
				hash: Default::default(),
				id_numbers: vec![
					(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone())
				].into_iter().collect(),
				secret_share: Random.generate().unwrap().secret().clone(),
			}],
		};
		let key2 = ServerKeyId::from_low_u64_be(2);
		let value2 = DocumentKeyShare {
			author: Default::default(),
			threshold: 200,
			public: Public::default(),
			common_point: Some(Random.generate().unwrap().public().clone()),
			encrypted_point: Some(Random.generate().unwrap().public().clone()),
			versions: vec![DocumentKeyShareVersion {
				hash: Default::default(),
				id_numbers: vec![
					(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone())
				].into_iter().collect(),
				secret_share: Random.generate().unwrap().secret().clone(),
			}],
		};
		let key3 = ServerKeyId::from_low_u64_be(3);

		let db = Database::open_default(&tempdir.path().display().to_string()).unwrap();

		let key_storage = PersistentKeyStorage::new(Arc::new(db)).unwrap();
		key_storage.insert(key1.clone(), value1.clone()).unwrap();
		key_storage.insert(key2.clone(), value2.clone()).unwrap();
		assert_eq!(key_storage.get(&key1), Ok(Some(value1.clone())));
		assert_eq!(key_storage.get(&key2), Ok(Some(value2.clone())));
		assert_eq!(key_storage.get(&key3), Ok(None));
		drop(key_storage);

		let db = Database::open_default(&tempdir.path().display().to_string()).unwrap();

		let key_storage = PersistentKeyStorage::new(Arc::new(db)).unwrap();
		assert_eq!(key_storage.get(&key1), Ok(Some(value1)));
		assert_eq!(key_storage.get(&key2), Ok(Some(value2)));
		assert_eq!(key_storage.get(&key3), Ok(None));
	}
}
