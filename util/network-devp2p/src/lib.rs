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

//! Network and general IO module.
//!
//! Example usage for creating a network service and adding an IO handler:
//!
//! ```rust
//! extern crate ethcore_network as net;
//! extern crate ethcore_network_devp2p as devp2p;
//! use net::*;
//! use devp2p::NetworkService;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! struct MyHandler;
//!
//! impl NetworkProtocolHandler for MyHandler {
//!		fn initialize(&self, io: &NetworkContext) {
//!			io.register_timer(0, Duration::from_secs(1));
//!		}
//!
//!		fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
//!			println!("Received {} ({} bytes) from {}", packet_id, data.len(), peer);
//!		}
//!
//!		fn connected(&self, io: &NetworkContext, peer: &PeerId) {
//!			println!("Connected {}", peer);
//!		}
//!
//!		fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
//!			println!("Disconnected {}", peer);
//!		}
//! }
//!
//! fn main () {
//! 	let mut service = NetworkService::new(NetworkConfiguration::new_local(), None).expect("Error creating network service");
//! 	service.start().expect("Error starting service");
//! 	service.register_protocol(Arc::new(MyHandler), *b"myp", &[(1u8, 1u8)]);
//!
//! 	// Wait for quit condition
//! 	// ...
//! 	// Drop the service
//! }
//! ```

//TODO: use Poll from mio
#![allow(deprecated)]

extern crate ethcore_io as io;
extern crate parity_bytes;
extern crate parity_crypto as crypto;
extern crate ethereum_types;
extern crate parking_lot;
extern crate mio;
extern crate tiny_keccak;
extern crate crypto as rcrypto;
extern crate rand;
extern crate ansi_term; //TODO: remove this
extern crate rustc_hex;
extern crate igd;
extern crate libc;
extern crate slab;
extern crate ethkey;
extern crate rlp;
extern crate bytes;
extern crate parity_path;
extern crate ethcore_network as network;
extern crate ipnetwork;
extern crate keccak_hash as hash;
extern crate serde;
extern crate serde_json;
extern crate parity_snappy as snappy;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
extern crate tempdir;
#[cfg(test)] #[macro_use]
extern crate assert_matches;

mod host;
mod connection;
mod handshake;
mod session;
mod discovery;
mod service;
mod node_table;
mod ip_utils;

pub use service::NetworkService;
pub use host::NetworkContext;

pub use io::TimerToken;
pub use node_table::{validate_node_url, NodeId};

const PROTOCOL_VERSION: u32 = 5;
