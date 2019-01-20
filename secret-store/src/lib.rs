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

extern crate byteorder;
extern crate common_types;
extern crate ethabi;
extern crate ethcore;
extern crate ethcore_call_contract as call_contract;
extern crate ethcore_sync as sync;
extern crate ethereum_types;
extern crate ethkey;
extern crate hyper;
extern crate keccak_hash as hash;
extern crate kvdb;
extern crate parity_bytes as bytes;
extern crate parity_crypto as crypto;
extern crate parity_runtime;
extern crate parking_lot;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate tiny_keccak;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_service;
extern crate url;

#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
extern crate kvdb_rocksdb;

mod key_server_cluster;
mod types;
mod helpers;

mod traits;
mod acl_storage;
mod key_server;
mod key_storage;
mod serialization;
mod key_server_set;
mod node_key_pair;
mod listener;
mod trusted_client;

use std::sync::Arc;
use kvdb::KeyValueDB;
use ethcore::client::Client;
use ethcore::miner::Miner;
use sync::SyncProvider;
use parity_runtime::Executor;

pub use types::{ServerKeyId, EncryptedDocumentKey, RequestSignature, Public,
	Error, NodeAddress, ContractAddress, ServiceConfiguration, ClusterConfiguration};
pub use traits::{NodeKeyPair, KeyServer};
pub use self::node_key_pair::{PlainNodeKeyPair, KeyStoreNodeKeyPair};

/// Start new key server instance
pub fn start(client: Arc<Client>, sync: Arc<SyncProvider>, miner: Arc<Miner>, self_key_pair: Arc<NodeKeyPair>, mut config: ServiceConfiguration,
	db: Arc<KeyValueDB>, executor: Executor) -> Result<Box<KeyServer>, Error>
{
	let trusted_client = trusted_client::TrustedClient::new(self_key_pair.clone(), client.clone(), sync, miner);
	let acl_storage: Arc<acl_storage::AclStorage> = match config.acl_check_contract_address.take() {
		Some(acl_check_contract_address) => acl_storage::OnChainAclStorage::new(trusted_client.clone(), acl_check_contract_address)?,
		None => Arc::new(acl_storage::DummyAclStorage::default()),
	};

	let key_server_set = key_server_set::OnChainKeyServerSet::new(trusted_client.clone(), config.cluster_config.key_server_set_contract_address.take(),
		self_key_pair.clone(), config.cluster_config.auto_migrate_enabled, config.cluster_config.nodes.clone())?;
	let key_storage = Arc::new(key_storage::PersistentKeyStorage::new(db)?);
	let key_server = Arc::new(key_server::KeyServerImpl::new(&config.cluster_config, key_server_set.clone(), self_key_pair.clone(),
		acl_storage.clone(), key_storage.clone(), executor.clone())?);
	let cluster = key_server.cluster();
	let key_server: Arc<KeyServer> = key_server;

	// prepare HTTP listener
	let http_listener = match config.listener_address {
		Some(listener_address) => Some(listener::http_listener::KeyServerHttpListener::start(listener_address, Arc::downgrade(&key_server), executor)?),
		None => None,
	};

	// prepare service contract listeners
	let create_service_contract = |address, name, api_mask|
		Arc::new(listener::service_contract::OnChainServiceContract::new(
			api_mask,
			trusted_client.clone(),
			name,
			address,
			self_key_pair.clone()));

	let mut contracts: Vec<Arc<listener::service_contract::ServiceContract>> = Vec::new();
	config.service_contract_address.map(|address|
		create_service_contract(address,
			listener::service_contract::SERVICE_CONTRACT_REGISTRY_NAME.to_owned(),
			listener::ApiMask::all()))
		.map(|l| contracts.push(l));
	config.service_contract_srv_gen_address.map(|address|
		create_service_contract(address,
			listener::service_contract::SRV_KEY_GEN_SERVICE_CONTRACT_REGISTRY_NAME.to_owned(),
			listener::ApiMask { server_key_generation_requests: true, ..Default::default() }))
		.map(|l| contracts.push(l));
	config.service_contract_srv_retr_address.map(|address|
		create_service_contract(address,
			listener::service_contract::SRV_KEY_RETR_SERVICE_CONTRACT_REGISTRY_NAME.to_owned(),
			listener::ApiMask { server_key_retrieval_requests: true, ..Default::default() }))
		.map(|l| contracts.push(l));
	config.service_contract_doc_store_address.map(|address|
		create_service_contract(address,
			listener::service_contract::DOC_KEY_STORE_SERVICE_CONTRACT_REGISTRY_NAME.to_owned(),
			listener::ApiMask { document_key_store_requests: true, ..Default::default() }))
		.map(|l| contracts.push(l));
	config.service_contract_doc_sretr_address.map(|address|
		create_service_contract(address,
			listener::service_contract::DOC_KEY_SRETR_SERVICE_CONTRACT_REGISTRY_NAME.to_owned(),
			listener::ApiMask { document_key_shadow_retrieval_requests: true, ..Default::default() }))
		.map(|l| contracts.push(l));

	let contract: Option<Arc<listener::service_contract::ServiceContract>> = match contracts.len() {
		0 => None,
		1 => Some(contracts.pop().expect("contract.len() is 1; qed")),
		_ => Some(Arc::new(listener::service_contract_aggregate::OnChainServiceContractAggregate::new(contracts))),
	};

	let contract_listener = match contract {
		Some(contract) => Some({
			let listener = listener::service_contract_listener::ServiceContractListener::new(
				listener::service_contract_listener::ServiceContractListenerParams {
					contract: contract,
					self_key_pair: self_key_pair.clone(),
					key_server_set: key_server_set,
					acl_storage: acl_storage,
					cluster: cluster,
					key_storage: key_storage,
				}
			)?;
			client.add_notify(listener.clone());
			listener
		}),
		None => None,
	};

	Ok(Box::new(listener::Listener::new(key_server, http_listener, contract_listener)))
}
