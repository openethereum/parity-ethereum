// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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
//! extern crate network as net;
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

pub use ethcore_io::TimerToken;
pub use host::NetworkContext;
pub use node_table::{MAX_NODES_IN_TABLE, NodeId, validate_node_url};
pub use service::NetworkService;

mod host;
mod connection;
mod handshake;
mod session;
mod discovery;
mod service;
mod node_table;
mod ip_utils;

const PROTOCOL_VERSION: u32 = 5;
