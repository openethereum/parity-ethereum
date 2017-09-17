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
use types::all::{Error, ServiceConfiguration, ServerKeyId, NodeId};
use serialization::{SerializablePublic, SerializableSecret};

/// Key of version value.
const DB_META_KEY_VERSION: &'static [u8; 7] = b"version";

/// Encrypted key share, stored by key storage on the single key server.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentKeyShare {
	/// Author of the entry.
	pub author: Public,
	/// Decryption threshold (at least threshold + 1 nodes are required to decrypt data).
	pub threshold: usize,
	/// Nodes ids numbers.
	pub id_numbers: BTreeMap<NodeId, Secret>,
	/// Node secret share.
	pub secret_share: Secret,
	/// Common (shared) encryption point.
	pub common_point: Option<Public>,
	/// Encrypted point.
	pub encrypted_point: Option<Public>,
}

/// Document encryption keys storage
pub trait KeyStorage: Send + Sync {
	/// Insert document encryption key
	fn insert(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error>;
	/// Update document encryption key
	fn update(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error>;
	/// Get document encryption key
	fn get(&self, document: &ServerKeyId) -> Result<DocumentKeyShare, Error>;
	/// Check if storage contains document encryption key
	fn contains(&self, document: &ServerKeyId) -> bool;
}

/// Persistent document encryption keys storage
pub struct PersistentKeyStorage {
	db: Database,
}

/// V0 of encrypted key share, as it is stored by key storage on the single key server.
#[derive(Serialize, Deserialize)]
struct SerializableDocumentKeyShareV0 {
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

/// V1 of encrypted key share, as it is stored by key storage on the single key server.
#[derive(Serialize, Deserialize)]
struct SerializableDocumentKeyShareV1 {
	/// Authore of the entry.
	pub author: SerializablePublic,
	/// Decryption threshold (at least threshold + 1 nodes are required to decrypt data).
	pub threshold: usize,
	/// Nodes ids numbers.
	pub id_numbers: BTreeMap<SerializablePublic, SerializableSecret>,
	/// Node secret share.
	pub secret_share: SerializableSecret,
	/// Common (shared) encryption point.
	pub common_point: Option<SerializablePublic>,
	/// Encrypted point.
	pub encrypted_point: Option<SerializablePublic>,
}

impl PersistentKeyStorage {
	/// Create new persistent document encryption keys storage
	pub fn new(config: &ServiceConfiguration) -> Result<Self, Error> {
		let mut db_path = PathBuf::from(&config.data_path);
		db_path.push("db");
		let db_path = db_path.to_str().ok_or(Error::Database("Invalid secretstore path".to_owned()))?;

		let db = Database::open_default(&db_path).map_err(Error::Database)?;
		let db = upgrade_db(db)?;

		Ok(PersistentKeyStorage {
			db: db,
		})
	}
}

fn upgrade_db(db: Database) -> Result<Database, Error> {
	let version = db.get(None, DB_META_KEY_VERSION).map_err(Error::Database)?;
	let version = version.and_then(|v| v.get(0).cloned()).unwrap_or(0);
	match version {
		0 => {
			let mut batch = db.transaction();
			batch.put(None, DB_META_KEY_VERSION, &[1]);
			for (db_key, db_value) in db.iter(None).into_iter().flat_map(|inner| inner) {
				let v0_key = serde_json::from_slice::<SerializableDocumentKeyShareV0>(&db_value).map_err(|e| Error::Database(e.to_string()))?;
				let v1_key = SerializableDocumentKeyShareV1 {
					// author is used in separate generation + encrypt sessions.
					// in v0 there have been only simultaneous GenEnc sessions.
					author: Public::default().into(), 
					threshold: v0_key.threshold,
					id_numbers: v0_key.id_numbers,
					secret_share: v0_key.secret_share,
					common_point: Some(v0_key.common_point),
					encrypted_point: Some(v0_key.encrypted_point),
				};
				let db_value = serde_json::to_vec(&v1_key).map_err(|e| Error::Database(e.to_string()))?;
				batch.put(None, &*db_key, &*db_value);
			}
			db.write(batch).map_err(Error::Database)?;
			Ok(db)
		},
		1 => Ok(db),
		_ => Err(Error::Database(format!("unsupported SecretStore database version:? {}", version))),
	}
}

impl KeyStorage for PersistentKeyStorage {
	fn insert(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error> {
		let key: SerializableDocumentKeyShareV1 = key.into();
		let key = serde_json::to_vec(&key).map_err(|e| Error::Database(e.to_string()))?;
		let mut batch = self.db.transaction();
		batch.put(None, &document, &key);
		self.db.write(batch).map_err(Error::Database)
	}

	fn update(&self, document: ServerKeyId, key: DocumentKeyShare) -> Result<(), Error> {
		self.insert(document, key)
	}

	fn get(&self, document: &ServerKeyId) -> Result<DocumentKeyShare, Error> {
		self.db.get(None, document)
			.map_err(Error::Database)?
			.ok_or(Error::DocumentNotFound)
			.map(|key| key.into_vec())
			.and_then(|key| serde_json::from_slice::<SerializableDocumentKeyShareV1>(&key).map_err(|e| Error::Database(e.to_string())))
			.map(Into::into)
	}

	fn contains(&self, document: &ServerKeyId) -> bool {
		self.db.get(None, document)
			.map(|k| k.is_some())
			.unwrap_or(false)
	}
}

impl From<DocumentKeyShare> for SerializableDocumentKeyShareV1 {
	fn from(key: DocumentKeyShare) -> Self {
		SerializableDocumentKeyShareV1 {
			author: key.author.into(),
			threshold: key.threshold,
			id_numbers: key.id_numbers.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
			secret_share: key.secret_share.into(),
			common_point: key.common_point.map(Into::into),
			encrypted_point: key.encrypted_point.map(Into::into),
		}
	}
}

impl From<SerializableDocumentKeyShareV1> for DocumentKeyShare {
	fn from(key: SerializableDocumentKeyShareV1) -> Self {
		DocumentKeyShare {
			author: key.author.into(),
			threshold: key.threshold,
			id_numbers: key.id_numbers.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
			secret_share: key.secret_share.into(),
			common_point: key.common_point.map(Into::into),
			encrypted_point: key.encrypted_point.map(Into::into),
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::{BTreeMap, HashMap};
	use parking_lot::RwLock;
	use serde_json;
	use devtools::RandomTempPath;
	use ethkey::{Random, Generator, Public, Secret};
	use util::Database;
	use types::all::{Error, NodeAddress, ServiceConfiguration, ClusterConfiguration, ServerKeyId};
	use super::{DB_META_KEY_VERSION, KeyStorage, PersistentKeyStorage, DocumentKeyShare,
		SerializableDocumentKeyShareV0, SerializableDocumentKeyShareV1, upgrade_db};

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

		fn get(&self, document: &ServerKeyId) -> Result<DocumentKeyShare, Error> {
			self.keys.read().get(document).cloned().ok_or(Error::DocumentNotFound)
		}

		fn contains(&self, document: &ServerKeyId) -> bool {
			self.keys.read().contains_key(document)
		}
	}

	#[test]
	fn persistent_key_storage() {
		let path = RandomTempPath::create_dir();
		let config = ServiceConfiguration {
			listener_address: None,
			acl_check_enabled: true,
			data_path: path.as_str().to_owned(),
			cluster_config: ClusterConfiguration {
				threads: 1,
				listener_address: NodeAddress {
					address: "0.0.0.0".to_owned(),
					port: 8083,
				},
				nodes: BTreeMap::new(),
				allow_connecting_to_higher_nodes: false,
			},
		};
		
		let key1 = ServerKeyId::from(1);
		let value1 = DocumentKeyShare {
			author: Public::default(),
			threshold: 100,
			id_numbers: vec![
				(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone())
			].into_iter().collect(),
			secret_share: Random.generate().unwrap().secret().clone(),
			common_point: Some(Random.generate().unwrap().public().clone()),
			encrypted_point: Some(Random.generate().unwrap().public().clone()),
		};
		let key2 = ServerKeyId::from(2);
		let value2 = DocumentKeyShare {
			author: Public::default(),
			threshold: 200,
			id_numbers: vec![
				(Random.generate().unwrap().public().clone(), Random.generate().unwrap().secret().clone())
			].into_iter().collect(),
			secret_share: Random.generate().unwrap().secret().clone(),
			common_point: Some(Random.generate().unwrap().public().clone()),
			encrypted_point: Some(Random.generate().unwrap().public().clone()),
		};
		let key3 = ServerKeyId::from(3);

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

	#[test]
	fn upgrade_db_0_to_1() {
		let db_path = RandomTempPath::create_dir();
		let db = Database::open_default(db_path.as_str()).unwrap();

		// prepare v0 database
		{
			let key = serde_json::to_vec(&SerializableDocumentKeyShareV0 {
				threshold: 777,
				id_numbers: vec![(
					"b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8".into(),
					"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse::<Secret>().unwrap().into(),
				)].into_iter().collect(),
				secret_share: "00125d85a05e5e63e214cb60fe63f132eec8a103aa29266b7e6e6c5b7597230b".parse::<Secret>().unwrap().into(),
				common_point: "99e82b163b062d55a64085bacfd407bb55f194ba5fb7a1af9c34b84435455520f1372e0e650a4f91aed0058cb823f62146ccb5599c8d13372c300dea866b69fc".into(),
				encrypted_point: "7e05df9dd077ec21ed4bc45c9fe9e0a43d65fa4be540630de615ced5e95cf5c3003035eb713317237d7667feeeb64335525158f5f7411f67aca9645169ea554c".into(),
			}).unwrap();
			let mut batch = db.transaction();
			batch.put(None, &[7], &key);
			db.write(batch).unwrap();
		}

		// upgrade database
		let db = upgrade_db(db).unwrap();

		// check upgrade
		assert_eq!(db.get(None, DB_META_KEY_VERSION).unwrap().unwrap()[0], 1);
		let key = serde_json::from_slice::<SerializableDocumentKeyShareV1>(&db.get(None, &[7]).unwrap().map(|key| key.to_vec()).unwrap()).unwrap();
		assert_eq!(Public::default(), key.author.clone().into());
		assert_eq!(777, key.threshold);
		assert_eq!(vec![(
			"b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8".parse::<Public>().unwrap(),
			"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse::<Secret>().unwrap(),
		)], key.id_numbers.clone().into_iter().map(|(k, v)| (k.into(), v.into())).collect::<Vec<(Public, Secret)>>());
		assert_eq!("00125d85a05e5e63e214cb60fe63f132eec8a103aa29266b7e6e6c5b7597230b".parse::<Secret>().unwrap(), key.secret_share.into());
		assert_eq!(Some("99e82b163b062d55a64085bacfd407bb55f194ba5fb7a1af9c34b84435455520f1372e0e650a4f91aed0058cb823f62146ccb5599c8d13372c300dea866b69fc".parse::<Public>().unwrap()), key.common_point.clone().map(Into::into));
		assert_eq!(Some("7e05df9dd077ec21ed4bc45c9fe9e0a43d65fa4be540630de615ced5e95cf5c3003035eb713317237d7667feeeb64335525158f5f7411f67aca9645169ea554c".parse::<Public>().unwrap()), key.encrypted_point.clone().map(Into::into));
	}
}
