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

extern crate byteorder;
extern crate ethabi;
extern crate ethcore;
extern crate ethcore_bytes as bytes;
extern crate ethcore_logger as logger;
extern crate ethcrypto;
extern crate ethereum_types;
extern crate ethkey;
extern crate ethsync;
extern crate futures_cpupool;
extern crate hyper;
extern crate keccak_hash as hash;
extern crate kvdb;
extern crate kvdb_rocksdb;
extern crate parking_lot;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate tiny_keccak;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
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

mod key_server_cluster;
mod types;

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
use ethcore::client::Client;
use ethsync::SyncProvider;

pub use types::all::{ServerKeyId, EncryptedDocumentKey, RequestSignature, Public,
	Error, NodeAddress, ContractAddress, ServiceConfiguration, ClusterConfiguration};
pub use traits::{NodeKeyPair, KeyServer};
pub use self::node_key_pair::{PlainNodeKeyPair, KeyStoreNodeKeyPair};

/// Start new key server instance
pub fn start(client: Arc<Client>, sync: Arc<SyncProvider>, self_key_pair: Arc<NodeKeyPair>, config: ServiceConfiguration) -> Result<Box<KeyServer>, Error> {
	let trusted_client = trusted_client::TrustedClient::new(client.clone(), sync);
	let acl_storage: Arc<acl_storage::AclStorage> = if config.acl_check_enabled {
			acl_storage::OnChainAclStorage::new(trusted_client.clone())?
		} else {
			Arc::new(acl_storage::DummyAclStorage::default())
		};

	let key_server_set = key_server_set::OnChainKeyServerSet::new(trusted_client.clone(), self_key_pair.clone(),
		config.cluster_config.auto_migrate_enabled, config.cluster_config.nodes.clone())?;
	let key_storage = Arc::new(key_storage::PersistentKeyStorage::new(&config)?);
	let key_server = Arc::new(key_server::KeyServerImpl::new(&config.cluster_config, key_server_set.clone(), self_key_pair.clone(), acl_storage, key_storage.clone())?);
	let cluster = key_server.cluster();

	// prepare listeners
	let http_listener = match config.listener_address {
		Some(listener_address) => Some(listener::http_listener::KeyServerHttpListener::start(listener_address, key_server.clone())?),
		None => None,
	};
	let contract_listener = config.service_contract_address.map(|service_contract_address| {
		let service_contract = Arc::new(listener::service_contract::OnChainServiceContract::new(trusted_client, service_contract_address, self_key_pair.clone()));
		let contract_listener = listener::service_contract_listener::ServiceContractListener::new(listener::service_contract_listener::ServiceContractListenerParams {
			contract: service_contract,
			key_server: key_server.clone(),
			self_key_pair: self_key_pair,
			key_server_set: key_server_set,
			cluster: cluster,
			key_storage: key_storage,
		});
		client.add_notify(contract_listener.clone());
		contract_listener
	});
	Ok(Box::new(listener::Listener::new(key_server, http_listener, contract_listener)))
}
