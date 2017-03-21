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
extern crate parking_lot;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
extern crate tokio_service;
extern crate tokio_proto;
extern crate url;

extern crate ethcore_devtools as devtools;
extern crate ethcore_util as util;
extern crate ethcore_ipc as ipc;
extern crate ethcrypto;
extern crate ethkey;

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

pub use types::all::{DocumentAddress, DocumentKey, DocumentEncryptedKey, RequestSignature, Public,
	Error, NodeAddress, ServiceConfiguration, ClusterConfiguration, EncryptionConfiguration};
pub use traits::{KeyServer};

/// Start new key server instance
pub fn start(config: ServiceConfiguration) -> Result<Box<KeyServer>, Error> {
	use std::sync::Arc;

	let acl_storage = Arc::new(acl_storage::DummyAclStorage::default());
	let key_storage = Arc::new(key_storage::PersistentKeyStorage::new(&config)?);
	let key_server = key_server::KeyServerImpl::new(&config.cluster_config, acl_storage, key_storage)?;
	let listener = http_listener::KeyServerHttpListener::start(config, key_server)?;
	Ok(Box::new(listener))
}
