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
#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate tiny_keccak;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_service;
extern crate tokio_proto;
extern crate url;

extern crate ethabi;
extern crate ethcore;
extern crate ethcore_devtools as devtools;
extern crate ethcore_bytes as bytes;
extern crate ethcore_util as util;
extern crate ethcore_bigint as bigint;
extern crate ethcore_ipc as ipc;
extern crate ethcore_logger as logger;
extern crate ethcrypto;
extern crate ethkey;
extern crate native_contracts;
extern crate hash;

mod key_server_cluster;
mod types;

mod traits {
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/traits.rs"));
}

mod acl_storage;
mod http_listener;
mod key_server;
mod key_storage;
mod serialization;
mod key_server_set;
mod node_key_pair;

use std::sync::Arc;
use ethcore::client::Client;

pub use types::all::{ServerKeyId, EncryptedDocumentKey, RequestSignature, Public,
	Error, NodeAddress, ServiceConfiguration, ClusterConfiguration};
pub use traits::{NodeKeyPair, KeyServer};
pub use self::node_key_pair::{PlainNodeKeyPair, KeyStoreNodeKeyPair};

/// Start new key server instance
pub fn start(client: Arc<Client>, self_key_pair: Arc<NodeKeyPair>, config: ServiceConfiguration) -> Result<Box<KeyServer>, Error> {
	use std::sync::Arc;

	let acl_storage: Arc<acl_storage::AclStorage> = if config.acl_check_enabled {
			acl_storage::OnChainAclStorage::new(&client)
		} else {
			Arc::new(acl_storage::DummyAclStorage::default())
		};
	let key_server_set = key_server_set::OnChainKeyServerSet::new(&client, config.cluster_config.nodes.clone())?;
	let key_storage = Arc::new(key_storage::PersistentKeyStorage::new(&config)?);
	let key_server = key_server::KeyServerImpl::new(&config.cluster_config, key_server_set, self_key_pair, acl_storage, key_storage)?;
	let listener = http_listener::KeyServerHttpListener::start(config.listener_address, key_server)?;
	Ok(Box::new(listener))
}
