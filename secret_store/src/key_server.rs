// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::collections::BTreeSet;
use std::sync::Arc;
use parking_lot::Mutex;
use crypto::DEFAULT_MAC;
use ethkey::crypto;
use parity_runtime::Executor;
use super::acl_storage::AclStorage;
use super::key_storage::KeyStorage;
use super::key_server_set::KeyServerSet;
use key_server_cluster::{math, ClusterCore};
use traits::{AdminSessionsServer, ServerKeyGenerator, DocumentKeyServer, MessageSigner, KeyServer, NodeKeyPair};
use types::{Error, Public, RequestSignature, Requester, ServerKeyId, EncryptedDocumentKey, EncryptedDocumentKeyShadow,
	ClusterConfiguration, MessageHash, EncryptedMessageSignature, NodeId};
use key_server_cluster::{ClusterClient, ClusterConfiguration as NetClusterConfiguration};

/// Secret store key server implementation
pub struct KeyServerImpl {
	data: Arc<Mutex<KeyServerCore>>,
}

/// Secret store key server data.
pub struct KeyServerCore {
	cluster: Arc<ClusterClient>,
}

impl KeyServerImpl {
	/// Create new key server instance
	pub fn new(config: &ClusterConfiguration, key_server_set: Arc<KeyServerSet>, self_key_pair: Arc<NodeKeyPair>,
		acl_storage: Arc<AclStorage>, key_storage: Arc<KeyStorage>, executor: Executor) -> Result<Self, Error>
	{
		Ok(KeyServerImpl {
			data: Arc::new(Mutex::new(KeyServerCore::new(config, key_server_set, self_key_pair, acl_storage, key_storage, executor)?)),
		})
	}

	/// Get cluster client reference.
	pub fn cluster(&self) -> Arc<ClusterClient> {
		self.data.lock().cluster.clone()
	}
}

impl KeyServer for KeyServerImpl {}

impl AdminSessionsServer for KeyServerImpl {
	fn change_servers_set(&self, old_set_signature: RequestSignature, new_set_signature: RequestSignature, new_servers_set: BTreeSet<NodeId>) -> Result<(), Error> {
		let servers_set_change_session = self.data.lock().cluster
			.new_servers_set_change_session(None, None, new_servers_set, old_set_signature, new_set_signature)?;
		servers_set_change_session.as_servers_set_change()
			.expect("new_servers_set_change_session creates servers_set_change_session; qed")
			.wait().map_err(Into::into)
	}
}

impl ServerKeyGenerator for KeyServerImpl {
	fn generate_key(&self, key_id: &ServerKeyId, author: &Requester, threshold: usize) -> Result<Public, Error> {
		// recover requestor' public key from signature
		let address = author.address(key_id).map_err(Error::InsufficientRequesterData)?;

		// generate server key
		let generation_session = self.data.lock().cluster.new_generation_session(key_id.clone(), None, address, threshold)?;
		generation_session.wait(None)
			.expect("when wait is called without timeout it always returns Some; qed")
			.map_err(Into::into)
	}
}

impl DocumentKeyServer for KeyServerImpl {
	fn store_document_key(&self, key_id: &ServerKeyId, author: &Requester, common_point: Public, encrypted_document_key: Public) -> Result<(), Error> {
		// store encrypted key
		let encryption_session = self.data.lock().cluster.new_encryption_session(key_id.clone(),
			author.clone(), common_point, encrypted_document_key)?;
		encryption_session.wait(None).map_err(Into::into)
	}

	fn generate_document_key(&self, key_id: &ServerKeyId, author: &Requester, threshold: usize) -> Result<EncryptedDocumentKey, Error> {
		// recover requestor' public key from signature
		let public = author.public(key_id).map_err(Error::InsufficientRequesterData)?;

		// generate server key
		let server_key = self.generate_key(key_id, author, threshold)?;

		// generate random document key
		let document_key = math::generate_random_point()?;
		let encrypted_document_key = math::encrypt_secret(&document_key, &server_key)?;

		// store document key in the storage
		self.store_document_key(key_id, author, encrypted_document_key.common_point, encrypted_document_key.encrypted_point)?;

		// encrypt document key with requestor public key
		let document_key = crypto::ecies::encrypt(&public, &DEFAULT_MAC, &document_key)
			.map_err(|err| Error::Internal(format!("Error encrypting document key: {}", err)))?;
		Ok(document_key)
	}

	fn restore_document_key(&self, key_id: &ServerKeyId, requester: &Requester) -> Result<EncryptedDocumentKey, Error> {
		// recover requestor' public key from signature
		let public = requester.public(key_id).map_err(Error::InsufficientRequesterData)?;

		// decrypt document key
		let decryption_session = self.data.lock().cluster.new_decryption_session(key_id.clone(),
			None, requester.clone(), None, false, false)?;
		let document_key = decryption_session.wait(None)
			.expect("when wait is called without timeout it always returns Some; qed")?
			.decrypted_secret;

		// encrypt document key with requestor public key
		let document_key = crypto::ecies::encrypt(&public, &DEFAULT_MAC, &document_key)
			.map_err(|err| Error::Internal(format!("Error encrypting document key: {}", err)))?;
		Ok(document_key)
	}

	fn restore_document_key_shadow(&self, key_id: &ServerKeyId, requester: &Requester) -> Result<EncryptedDocumentKeyShadow, Error> {
		let decryption_session = self.data.lock().cluster.new_decryption_session(key_id.clone(),
			None, requester.clone(), None, true, false)?;
		decryption_session.wait(None)
			.expect("when wait is called without timeout it always returns Some; qed")
			.map_err(Into::into)
	}
}

impl MessageSigner for KeyServerImpl {
	fn sign_message_schnorr(&self, key_id: &ServerKeyId, requester: &Requester, message: MessageHash) -> Result<EncryptedMessageSignature, Error> {
		// recover requestor' public key from signature
		let public = requester.public(key_id).map_err(Error::InsufficientRequesterData)?;

		// sign message
		let signing_session = self.data.lock().cluster.new_schnorr_signing_session(key_id.clone(),
			requester.clone().into(), None, message)?;
		let message_signature = signing_session.wait()?;

		// compose two message signature components into single one
		let mut combined_signature = [0; 64];
		combined_signature[..32].clone_from_slice(&**message_signature.0);
		combined_signature[32..].clone_from_slice(&**message_signature.1);

		// encrypt combined signature with requestor public key
		let message_signature = crypto::ecies::encrypt(&public, &DEFAULT_MAC, &combined_signature)
			.map_err(|err| Error::Internal(format!("Error encrypting message signature: {}", err)))?;
		Ok(message_signature)
	}

	fn sign_message_ecdsa(&self, key_id: &ServerKeyId, requester: &Requester, message: MessageHash) -> Result<EncryptedMessageSignature, Error> {
		// recover requestor' public key from signature
		let public = requester.public(key_id).map_err(Error::InsufficientRequesterData)?;

		// sign message
		let signing_session = self.data.lock().cluster.new_ecdsa_signing_session(key_id.clone(),
			requester.clone().into(), None, message)?;
		let message_signature = signing_session.wait()?;

		// encrypt combined signature with requestor public key
		let message_signature = crypto::ecies::encrypt(&public, &DEFAULT_MAC, &*message_signature)
			.map_err(|err| Error::Internal(format!("Error encrypting message signature: {}", err)))?;
		Ok(message_signature)
	}
}

impl KeyServerCore {
	pub fn new(config: &ClusterConfiguration, key_server_set: Arc<KeyServerSet>, self_key_pair: Arc<NodeKeyPair>,
		acl_storage: Arc<AclStorage>, key_storage: Arc<KeyStorage>, executor: Executor) -> Result<Self, Error>
	{
		let config = NetClusterConfiguration {
			self_key_pair: self_key_pair.clone(),
			listen_address: (config.listener_address.address.clone(), config.listener_address.port),
			key_server_set: key_server_set,
			allow_connecting_to_higher_nodes: config.allow_connecting_to_higher_nodes,
			acl_storage: acl_storage,
			key_storage: key_storage,
			admin_public: config.admin_public.clone(),
			auto_migrate_enabled: config.auto_migrate_enabled,
		};

		let cluster = ClusterCore::new(executor, config)
			.and_then(|c| c.run().map(|_| c.client()))
			.map_err(|err| Error::from(err))?;

		Ok(KeyServerCore {
			cluster,
		})
	}
}

#[cfg(test)]
pub mod tests {
	use std::collections::BTreeSet;
	use std::time;
	use std::sync::Arc;
	use std::net::SocketAddr;
	use std::collections::BTreeMap;
	use crypto::DEFAULT_MAC;
	use ethkey::{self, crypto, Secret, Random, Generator, verify_public};
	use acl_storage::DummyAclStorage;
	use key_storage::KeyStorage;
	use key_storage::tests::DummyKeyStorage;
	use node_key_pair::PlainNodeKeyPair;
	use key_server_set::tests::MapKeyServerSet;
	use key_server_cluster::math;
	use ethereum_types::{H256, H520};
	use parity_runtime::Runtime;
	use types::{Error, Public, ClusterConfiguration, NodeAddress, RequestSignature, ServerKeyId,
		EncryptedDocumentKey, EncryptedDocumentKeyShadow, MessageHash, EncryptedMessageSignature,
		Requester, NodeId};
	use traits::{AdminSessionsServer, ServerKeyGenerator, DocumentKeyServer, MessageSigner, KeyServer};
	use super::KeyServerImpl;

	#[derive(Default)]
	pub struct DummyKeyServer;

	impl KeyServer for DummyKeyServer {}

	impl AdminSessionsServer for DummyKeyServer {
		fn change_servers_set(&self, _old_set_signature: RequestSignature, _new_set_signature: RequestSignature, _new_servers_set: BTreeSet<NodeId>) -> Result<(), Error> {
			unimplemented!("test-only")
		}
	}

	impl ServerKeyGenerator for DummyKeyServer {
		fn generate_key(&self, _key_id: &ServerKeyId, _author: &Requester, _threshold: usize) -> Result<Public, Error> {
			unimplemented!("test-only")
		}
	}

	impl DocumentKeyServer for DummyKeyServer {
		fn store_document_key(&self, _key_id: &ServerKeyId, _author: &Requester, _common_point: Public, _encrypted_document_key: Public) -> Result<(), Error> {
			unimplemented!("test-only")
		}

		fn generate_document_key(&self, _key_id: &ServerKeyId, _author: &Requester, _threshold: usize) -> Result<EncryptedDocumentKey, Error> {
			unimplemented!("test-only")
		}

		fn restore_document_key(&self, _key_id: &ServerKeyId, _requester: &Requester) -> Result<EncryptedDocumentKey, Error> {
			unimplemented!("test-only")
		}

		fn restore_document_key_shadow(&self, _key_id: &ServerKeyId, _requester: &Requester) -> Result<EncryptedDocumentKeyShadow, Error> {
			unimplemented!("test-only")
		}
	}

	impl MessageSigner for DummyKeyServer {
		fn sign_message_schnorr(&self, _key_id: &ServerKeyId, _requester: &Requester, _message: MessageHash) -> Result<EncryptedMessageSignature, Error> {
			unimplemented!("test-only")
		}

		fn sign_message_ecdsa(&self, _key_id: &ServerKeyId, _requester: &Requester, _message: MessageHash) -> Result<EncryptedMessageSignature, Error> {
			unimplemented!("test-only")
		}
	}

	fn make_key_servers(start_port: u16, num_nodes: usize) -> (Vec<KeyServerImpl>, Vec<Arc<DummyKeyStorage>>, Runtime) {
		let key_pairs: Vec<_> = (0..num_nodes).map(|_| Random.generate().unwrap()).collect();
		let configs: Vec<_> = (0..num_nodes).map(|i| ClusterConfiguration {
				listener_address: NodeAddress {
					address: "127.0.0.1".into(),
					port: start_port + (i as u16),
				},
				nodes: key_pairs.iter().enumerate().map(|(j, kp)| (kp.public().clone(),
					NodeAddress {
						address: "127.0.0.1".into(),
						port: start_port + (j as u16),
					})).collect(),
				key_server_set_contract_address: None,
				allow_connecting_to_higher_nodes: false,
				admin_public: None,
				auto_migrate_enabled: false,
			}).collect();
		let key_servers_set: BTreeMap<Public, SocketAddr> = configs[0].nodes.iter()
			.map(|(k, a)| (k.clone(), format!("{}:{}", a.address, a.port).parse().unwrap()))
			.collect();
		let key_storages = (0..num_nodes).map(|_| Arc::new(DummyKeyStorage::default())).collect::<Vec<_>>();
		let runtime = Runtime::with_thread_count(4);
		let key_servers: Vec<_> = configs.into_iter().enumerate().map(|(i, cfg)|
			KeyServerImpl::new(&cfg, Arc::new(MapKeyServerSet::new(false, key_servers_set.clone())),
				Arc::new(PlainNodeKeyPair::new(key_pairs[i].clone())),
				Arc::new(DummyAclStorage::default()),
				key_storages[i].clone(), runtime.executor()).unwrap()
		).collect();

		// wait until connections are established. It is fast => do not bother with events here
		let start = time::Instant::now();
		let mut tried_reconnections = false;
		loop {
			if key_servers.iter().all(|ks| ks.cluster().cluster_state().connected.len() == num_nodes - 1) {
				break;
			}

			let old_tried_reconnections = tried_reconnections;
			let mut fully_connected = true;
			for key_server in &key_servers {
				if key_server.cluster().cluster_state().connected.len() != num_nodes - 1 {
					fully_connected = false;
					if !old_tried_reconnections {
						tried_reconnections = true;
						key_server.cluster().connect();
					}
				}
			}
			if fully_connected {
				break;
			}
			if time::Instant::now() - start > time::Duration::from_millis(3000) {
				panic!("connections are not established in 3000ms");
			}
		}

		(key_servers, key_storages, runtime)
	}

	#[test]
	fn document_key_generation_and_retrievement_works_over_network_with_single_node() {
		//::logger::init_log();
		let (key_servers, _, runtime) = make_key_servers(6070, 1);

		// generate document key
		let threshold = 0;
		let document = Random.generate().unwrap().secret().clone();
		let secret = Random.generate().unwrap().secret().clone();
		let signature = ethkey::sign(&secret, &document).unwrap();
		let generated_key = key_servers[0].generate_document_key(&document, &signature.clone().into(), threshold).unwrap();
		let generated_key = crypto::ecies::decrypt(&secret, &DEFAULT_MAC, &generated_key).unwrap();

		// now let's try to retrieve key back
		for key_server in key_servers.iter() {
			let retrieved_key = key_server.restore_document_key(&document, &signature.clone().into()).unwrap();
			let retrieved_key = crypto::ecies::decrypt(&secret, &DEFAULT_MAC, &retrieved_key).unwrap();
			assert_eq!(retrieved_key, generated_key);
		}
		drop(runtime);
	}

	#[test]
	fn document_key_generation_and_retrievement_works_over_network_with_3_nodes() {
		//::logger::init_log();
		let (key_servers, key_storages, runtime) = make_key_servers(6080, 3);

		let test_cases = [0, 1, 2];
		for threshold in &test_cases {
			// generate document key
			let document = Random.generate().unwrap().secret().clone();
			let secret = Random.generate().unwrap().secret().clone();
			let signature = ethkey::sign(&secret, &document).unwrap();
			let generated_key = key_servers[0].generate_document_key(&document, &signature.clone().into(), *threshold).unwrap();
			let generated_key = crypto::ecies::decrypt(&secret, &DEFAULT_MAC, &generated_key).unwrap();

			// now let's try to retrieve key back
			for (i, key_server) in key_servers.iter().enumerate() {
				let retrieved_key = key_server.restore_document_key(&document, &signature.clone().into()).unwrap();
				let retrieved_key = crypto::ecies::decrypt(&secret, &DEFAULT_MAC, &retrieved_key).unwrap();
				assert_eq!(retrieved_key, generated_key);

				let key_share = key_storages[i].get(&document).unwrap().unwrap();
				assert!(key_share.common_point.is_some());
				assert!(key_share.encrypted_point.is_some());
			}
		}
		drop(runtime);
	}

	#[test]
	fn server_key_generation_and_storing_document_key_works_over_network_with_3_nodes() {
		//::logger::init_log();
		let (key_servers, _, runtime) = make_key_servers(6090, 3);

		let test_cases = [0, 1, 2];
		for threshold in &test_cases {
			// generate server key
			let server_key_id = Random.generate().unwrap().secret().clone();
			let requestor_secret = Random.generate().unwrap().secret().clone();
			let signature = ethkey::sign(&requestor_secret, &server_key_id).unwrap();
			let server_public = key_servers[0].generate_key(&server_key_id, &signature.clone().into(), *threshold).unwrap();

			// generate document key (this is done by KS client so that document key is unknown to any KS)
			let generated_key = Random.generate().unwrap().public().clone();
			let encrypted_document_key = math::encrypt_secret(&generated_key, &server_public).unwrap();

			// store document key
			key_servers[0].store_document_key(&server_key_id, &signature.clone().into(),
				encrypted_document_key.common_point, encrypted_document_key.encrypted_point).unwrap();

			// now let's try to retrieve key back
			for key_server in key_servers.iter() {
				let retrieved_key = key_server.restore_document_key(&server_key_id, &signature.clone().into()).unwrap();
				let retrieved_key = crypto::ecies::decrypt(&requestor_secret, &DEFAULT_MAC, &retrieved_key).unwrap();
				let retrieved_key = Public::from_slice(&retrieved_key);
				assert_eq!(retrieved_key, generated_key);
			}
		}
		drop(runtime);
	}

	#[test]
	fn server_key_generation_and_message_signing_works_over_network_with_3_nodes() {
		//::logger::init_log();
		let (key_servers, _, runtime) = make_key_servers(6100, 3);

		let test_cases = [0, 1, 2];
		for threshold in &test_cases {
			// generate server key
			let server_key_id = Random.generate().unwrap().secret().clone();
			let requestor_secret = Random.generate().unwrap().secret().clone();
			let signature = ethkey::sign(&requestor_secret, &server_key_id).unwrap();
			let server_public = key_servers[0].generate_key(&server_key_id, &signature.clone().into(), *threshold).unwrap();

			// sign message
			let message_hash = H256::from(42);
			let combined_signature = key_servers[0].sign_message_schnorr(&server_key_id, &signature.into(), message_hash.clone()).unwrap();
			let combined_signature = crypto::ecies::decrypt(&requestor_secret, &DEFAULT_MAC, &combined_signature).unwrap();
			let signature_c = Secret::from_slice(&combined_signature[..32]).unwrap();
			let signature_s = Secret::from_slice(&combined_signature[32..]).unwrap();

			// check signature
			assert_eq!(math::verify_schnorr_signature(&server_public, &(signature_c, signature_s), &message_hash), Ok(true));
		}
		drop(runtime);
	}

	#[test]
	fn decryption_session_is_delegated_when_node_does_not_have_key_share() {
		//::logger::init_log();
		let (key_servers, _, runtime) = make_key_servers(6110, 3);

		// generate document key
		let threshold = 0;
		let document = Random.generate().unwrap().secret().clone();
		let secret = Random.generate().unwrap().secret().clone();
		let signature = ethkey::sign(&secret, &document).unwrap();
		let generated_key = key_servers[0].generate_document_key(&document, &signature.clone().into(), threshold).unwrap();
		let generated_key = crypto::ecies::decrypt(&secret, &DEFAULT_MAC, &generated_key).unwrap();

		// remove key from node0
		key_servers[0].cluster().key_storage().remove(&document).unwrap();

		// now let's try to retrieve key back by requesting it from node0, so that session must be delegated
		let retrieved_key = key_servers[0].restore_document_key(&document, &signature.into()).unwrap();
		let retrieved_key = crypto::ecies::decrypt(&secret, &DEFAULT_MAC, &retrieved_key).unwrap();
		assert_eq!(retrieved_key, generated_key);
		drop(runtime);
	}

	#[test]
	fn schnorr_signing_session_is_delegated_when_node_does_not_have_key_share() {
		//::logger::init_log();
		let (key_servers, _, runtime) = make_key_servers(6114, 3);
		let threshold = 1;

		// generate server key
		let server_key_id = Random.generate().unwrap().secret().clone();
		let requestor_secret = Random.generate().unwrap().secret().clone();
		let signature = ethkey::sign(&requestor_secret, &server_key_id).unwrap();
		let server_public = key_servers[0].generate_key(&server_key_id, &signature.clone().into(), threshold).unwrap();

		// remove key from node0
		key_servers[0].cluster().key_storage().remove(&server_key_id).unwrap();

		// sign message
		let message_hash = H256::from(42);
		let combined_signature = key_servers[0].sign_message_schnorr(&server_key_id, &signature.into(), message_hash.clone()).unwrap();
		let combined_signature = crypto::ecies::decrypt(&requestor_secret, &DEFAULT_MAC, &combined_signature).unwrap();
		let signature_c = Secret::from_slice(&combined_signature[..32]).unwrap();
		let signature_s = Secret::from_slice(&combined_signature[32..]).unwrap();

		// check signature
		assert_eq!(math::verify_schnorr_signature(&server_public, &(signature_c, signature_s), &message_hash), Ok(true));
		drop(runtime);
	}

	#[test]
	fn ecdsa_signing_session_is_delegated_when_node_does_not_have_key_share() {
		//::logger::init_log();
		let (key_servers, _, runtime) = make_key_servers(6117, 4);
		let threshold = 1;

		// generate server key
		let server_key_id = Random.generate().unwrap().secret().clone();
		let requestor_secret = Random.generate().unwrap().secret().clone();
		let signature = ethkey::sign(&requestor_secret, &server_key_id).unwrap();
		let server_public = key_servers[0].generate_key(&server_key_id, &signature.clone().into(), threshold).unwrap();

		// remove key from node0
		key_servers[0].cluster().key_storage().remove(&server_key_id).unwrap();

		// sign message
		let message_hash = H256::random();
		let signature = key_servers[0].sign_message_ecdsa(&server_key_id, &signature.into(), message_hash.clone()).unwrap();
		let signature = crypto::ecies::decrypt(&requestor_secret, &DEFAULT_MAC, &signature).unwrap();
		let signature: H520 = signature[0..65].into();

		// check signature
		assert!(verify_public(&server_public, &signature.into(), &message_hash).unwrap());
		drop(runtime);
	}

	#[test]
	fn servers_set_change_session_works_over_network() {
		// TODO [Test]
	}
}
