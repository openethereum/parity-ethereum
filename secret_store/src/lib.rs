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

#[macro_use]
extern crate log;
extern crate hyper;
extern crate parking_lot;
extern crate url;

extern crate ethcore_devtools as devtools;
extern crate ethcore_util as util;
extern crate ethcore_ipc as ipc;
extern crate ethcrypto;
extern crate ethkey;

mod types;

mod traits {
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/traits.rs"));
}

mod acl_storage;
mod http_listener;
mod key_server;
mod key_storage;

pub use types::all::{DocumentAddress, DocumentKey, DocumentEncryptedKey, RequestSignature, Public,
	Error, ServiceConfiguration};
pub use traits::{KeyServer};

/// Start new key server instance
pub fn start(config: ServiceConfiguration) -> Result<Box<KeyServer>, Error> {
	let acl_storage = acl_storage::DummyAclStorage::default();
	let key_storage = key_storage::PersistentKeyStorage::new(&config)?;
	let key_server = key_server::KeyServerImpl::new(acl_storage, key_storage);
	let listener = http_listener::KeyServerHttpListener::start(config, key_server)?;
	Ok(Box::new(listener))
}
