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

use std::thread;
use std::sync::Arc;
use std::sync::mpsc;
use futures::{self, Future};
use parking_lot::Mutex;
use tokio_core::reactor::Core;
use ethcrypto;
use ethkey;
use super::acl_storage::AclStorage;
use super::key_storage::KeyStorage;
use key_server_cluster::ClusterCore;
use traits::KeyServer;
use types::all::{Error, RequestSignature, DocumentAddress, DocumentEncryptedKey, ClusterConfiguration};
use key_server_cluster::{ClusterClient, ClusterConfiguration as NetClusterConfiguration};

/// Secret store key server implementation
pub struct KeyServerImpl<T: AclStorage, U: KeyStorage> {
	acl_storage: T,
	key_storage: U,
	data: Arc<Mutex<KeyServerCore>>,
}

unsafe impl<T, U> Send for KeyServerImpl<T, U> where T: AclStorage, U: KeyStorage {}
unsafe impl<T, U> Sync for KeyServerImpl<T, U> where T: AclStorage, U: KeyStorage {}


/// Secret store key server data.
pub struct KeyServerCore {
	close: Option<futures::Complete<()>>,
	_handle: Option<thread::JoinHandle<()>>,
	cluster: Option<Arc<ClusterClient>>,
}

impl<T, U> KeyServerImpl<T, U> where T: AclStorage, U: KeyStorage {
	/// Create new key server instance
	pub fn new(config: &ClusterConfiguration, acl_storage: T, key_storage: U) -> Result<Self, Error> {
		Ok(KeyServerImpl {
			acl_storage: acl_storage,
			key_storage: key_storage,
			data: Arc::new(Mutex::new(KeyServerCore::new(config)?)),
		})
	}

	#[cfg(test)]
	/// Create new key server instance without network communications
	pub fn new_no_cluster(acl_storage: T, key_storage: U) -> Result<Self, Error> {
		Ok(KeyServerImpl {
			acl_storage: acl_storage,
			key_storage: key_storage,
			data: Arc::new(Mutex::new(KeyServerCore::new_no_cluster()?)),
		})
	}


	#[cfg(test)]
	/// Get cluster client reference.
	pub fn cluster(&self) -> Arc<ClusterClient> {
		self.data.lock().cluster.clone()
			.expect("cluster can be None in test cfg only; test cfg is for correct tests; qed")
	}
}

impl<T, U> KeyServer for KeyServerImpl<T, U> where T: AclStorage, U: KeyStorage {
	fn generate_document_key(&self, signature: &RequestSignature, document: &DocumentAddress, threshold: usize) -> Result<DocumentEncryptedKey, Error> {
		// recover requestor' public key from signature
		let public = ethkey::recover(signature, document)
			.map_err(|_| Error::BadSignature)?;

		// generate document key
		let data = self.data.lock();
		let encryption_session = data.cluster.as_ref().expect("cluster can be None in test cfg only; test cfg is for correct tests; qed")
			.new_encryption_session(document.clone(), threshold)?;
		let document_key = encryption_session.wait()?;

		// encrypt document key with requestor public key
		let document_key = ethcrypto::ecies::encrypt_single_message(&public, &document_key)
			.map_err(|err| Error::Internal(format!("Error encrypting document key: {}", err)))?;
		Ok(document_key)
	}

	fn document_key(&self, signature: &RequestSignature, document: &DocumentAddress) -> Result<DocumentEncryptedKey, Error> {
		// recover requestor' public key from signature
		let public = ethkey::recover(signature, document)
			.map_err(|_| Error::BadSignature)?;

		// check that requestor has access to the document
		if !self.acl_storage.check(&public, document)? {
			return Err(Error::AccessDenied);
		}

		// read unencrypted document key
		let document_key = self.key_storage.get(document)?;
		// encrypt document key with requestor public key
		let document_key = ethcrypto::ecies::encrypt_single_message(&public, &document_key)
			.map_err(|err| Error::Internal(format!("Error encrypting document key: {}", err)))?;
		Ok(document_key)
	}
}

impl KeyServerCore {
	pub fn new(config: &ClusterConfiguration) -> Result<Self, Error> {
		let config = NetClusterConfiguration {
			threads: config.threads,
			self_key_pair: ethkey::KeyPair::from_secret_slice(&config.self_private)?,
			listen_address: (config.listener_address.address.clone(), config.listener_address.port),
			nodes: config.nodes.iter()
				.map(|(node_id, node_address)| (node_id.clone(), (node_address.address.clone(), node_address.port)))
				.collect(),
			allow_connecting_to_higher_nodes: config.allow_connecting_to_higher_nodes,
			encryption_config: config.encryption_config.clone(),
		};

		let (stop, stopped) = futures::oneshot();
		let (tx, rx) = mpsc::channel();
		let handle = thread::spawn(move || {
			let mut el = Core::new().expect("Creating an event loop should not fail.");
			let cluster = ClusterCore::new(el.handle(), config);
			let cluster_client = cluster.and_then(|c| c.run().map(|_| c.client()));
			tx.send(cluster_client).expect("Rx is blocking upper thread.");
			let _ = el.run(futures::empty().select(stopped));
		});
		let cluster = rx.recv().expect("tx is transfered to a newly spawned thread.")?;
 
		Ok(KeyServerCore {
			close: Some(stop),
			_handle: Some(handle),
			cluster: Some(cluster),
		})
	}

	#[cfg(test)]
	pub fn new_no_cluster() -> Result<Self, Error> {
		Ok(KeyServerCore {
			close: None,
			_handle: None,
			cluster: None,
		})
	}
}

impl Drop for KeyServerCore {
	fn drop(&mut self) {
		self.close.take().map(|v| v.complete(()));
	}
}

#[cfg(test)]
mod tests {
	use std::time;
	use std::str::FromStr;
	use ethcrypto;
	use ethkey::{self, Secret};
	use acl_storage::DummyAclStorage;
	use key_storage::KeyStorage;
	use key_storage::tests::DummyKeyStorage;
	use types::all::{ClusterConfiguration, NodeAddress, EncryptionConfiguration};
	use super::super::{Error, RequestSignature, DocumentAddress};
	use super::{KeyServer, KeyServerImpl};

	const DOCUMENT1: &'static str = "0000000000000000000000000000000000000000000000000000000000000001";
	const DOCUMENT2: &'static str = "0000000000000000000000000000000000000000000000000000000000000002";
	const KEY1: &'static str = "key1";
	const PRIVATE1: &'static str = "03055e18a8434dcc9061cc1b81c4ef84dc7cf4574d755e52cdcf0c8898b25b11";
	const PUBLIC2: &'static str = "dfe62f56bb05fbd85b485bac749f3410309e24b352bac082468ce151e9ddb94fa7b5b730027fe1c7c5f3d5927621d269f91aceb5caa3c7fe944677a22f88a318";
	const PRIVATE2: &'static str = "0eb3816f4f705fa0fd952fb27b71b8c0606f09f4743b5b65cbc375bd569632f2";

	fn create_key_server() -> KeyServerImpl<DummyAclStorage, DummyKeyStorage> {
		let acl_storage = DummyAclStorage::default();
		let key_storage = DummyKeyStorage::default();
		key_storage.insert(DOCUMENT1.into(), KEY1.into()).unwrap();
		acl_storage.prohibit(PUBLIC2.into(), DOCUMENT1.into());
		KeyServerImpl::new_no_cluster(acl_storage, key_storage).unwrap()
	}

	fn make_signature(secret: &str, document: &'static str) -> RequestSignature {
		let secret = Secret::from_str(secret).unwrap();
		let document: DocumentAddress = document.into();
		ethkey::sign(&secret, &document).unwrap()
	}

	#[test]
	fn document_key_succeeds() {
		let key_server = create_key_server();
		let signature = make_signature(PRIVATE1, DOCUMENT1);
		let document_key = key_server.document_key(&signature, &DOCUMENT1.into()).unwrap();
		let document_key = ethcrypto::ecies::decrypt_single_message(&Secret::from_str(PRIVATE1).unwrap(), &document_key);
		assert_eq!(document_key, Ok(KEY1.into()));
	}

	#[test]
	fn document_key_fails_when_bad_signature() {
		let key_server = create_key_server();
		let signature = RequestSignature::default();
		let document_key = key_server.document_key(&signature, &DOCUMENT1.into());
		assert_eq!(document_key, Err(Error::BadSignature));
	}

	#[test]
	fn document_key_fails_when_acl_check_fails() {
		let key_server = create_key_server();
		let signature = make_signature(PRIVATE2, DOCUMENT1);
		let document_key = key_server.document_key(&signature, &DOCUMENT1.into());
		assert_eq!(document_key, Err(Error::AccessDenied));
	}

	#[test]
	fn document_key_fails_when_document_not_found() {
		let key_server = create_key_server();
		let signature = make_signature(PRIVATE1, DOCUMENT2);
		let document_key = key_server.document_key(&signature, &DOCUMENT2.into());
		assert_eq!(document_key, Err(Error::DocumentNotFound));
	}

	#[test]
	fn document_key_generation_works_over_network() {
		let key_pairs = vec![
			ethkey::KeyPair::from_secret(ethkey::Secret::from_str("6c26a76e9b31048d170873a791401c7e799a11f0cefc0171cc31a49800967509").unwrap()).unwrap(),
			ethkey::KeyPair::from_secret(ethkey::Secret::from_str("7e94018b3731afdb3b4e6f4c3e179475640166da12e1d1b0c7d80729b1a5b452").unwrap()).unwrap(),
			ethkey::KeyPair::from_secret(ethkey::Secret::from_str("5ab6ed2a52c33142380032c39a03a86b12eacb3fa4b53bc16d84f51318156f8c").unwrap()).unwrap(),
		];
		let mut config = ClusterConfiguration {
				threads: 4,
				self_private: (***key_pairs[0].secret()).into(),
				listener_address: NodeAddress {
					address: "127.0.0.1".into(),
					port: 6000,
				},
				nodes: key_pairs.iter().enumerate().map(|(i, kp)| (kp.public().clone(),
					NodeAddress {
						address: "127.0.0.1".into(),
						port: 6000 + (i as u16),
					})).collect(),
				allow_connecting_to_higher_nodes: false,
				encryption_config: EncryptionConfiguration {
					key_check_timeout_ms: 10,
				},
			};

		let key_server1 = KeyServerImpl::new(&config, DummyAclStorage::default(), DummyKeyStorage::default()).unwrap();

		config.self_private = (***key_pairs[1].secret()).into();
		config.listener_address.port = 6001;
		let _key_server2 = KeyServerImpl::new(&config, DummyAclStorage::default(), DummyKeyStorage::default()).unwrap();

		config.self_private = (***key_pairs[2].secret()).into();
		config.listener_address.port = 6002;
		let _key_server3 = KeyServerImpl::new(&config, DummyAclStorage::default(), DummyKeyStorage::default()).unwrap();

		// wait until connections areestablished (TODO: get rid of timeout)
		let start = time::Instant::now();
		loop {
			if key_server1.cluster().cluster_state().connected.len() == 2 {
				break;
			}
			if time::Instant::now() - start > time::Duration::from_millis(300) {
				panic!("connections are not established in 300ms");
			}
		}

		let signature = make_signature(PRIVATE1, DOCUMENT1);
		key_server1.generate_document_key(&signature, &DOCUMENT1.into(), 1).unwrap();
	}
}
