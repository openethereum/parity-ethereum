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

//! Network and general IO module.
//!
//! Example usage for craeting a network service and adding an IO handler:
//!
//! ```rust
//! extern crate ethcore_network as net;
//! use net::*;
//! use std::sync::Arc;
//!
//! struct MyHandler;
//!
//! impl NetworkProtocolHandler for MyHandler {
//!		fn initialize(&self, io: &NetworkContext, _host_info: &HostInfo) {
//!			io.register_timer(0, 1000);
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
//! 	service.register_protocol(Arc::new(MyHandler), *b"myp", 1, &[1u8]);
//!
//! 	// Wait for quit condition
//! 	// ...
//! 	// Drop the service
//! }
//! ```

//TODO: use Poll from mio
#![allow(deprecated)]

extern crate ethcore_io as io;
extern crate ethcore_util as util;
extern crate ethcore_bigint as bigint;
extern crate parking_lot;
extern crate mio;
extern crate tiny_keccak;
extern crate crypto as rcrypto;
extern crate rand;
extern crate time;
extern crate ansi_term; //TODO: remove this
extern crate rustc_hex;
extern crate rustc_serialize;
extern crate igd;
extern crate libc;
extern crate slab;
extern crate ethkey;
extern crate ethcrypto as crypto;
extern crate rlp;
extern crate bytes;
extern crate path;
extern crate ethcore_logger;
extern crate ipnetwork;
extern crate hash;

#[macro_use]
extern crate log;

#[cfg(test)]
extern crate ethcore_devtools as devtools;

mod host;
mod connection;
mod handshake;
mod session;
mod discovery;
mod service;
mod error;
mod node_table;
mod stats;
mod ip_utils;
mod connection_filter;

#[cfg(test)]
mod tests;

pub use host::{HostInfo, PeerId, PacketId, ProtocolId, NetworkContext, NetworkIoMessage, NetworkConfiguration};
pub use service::NetworkService;
pub use error::NetworkError;
pub use stats::NetworkStats;
pub use session::SessionInfo;
pub use connection_filter::{ConnectionFilter, ConnectionDirection};

pub use io::TimerToken;
pub use node_table::{is_valid_node_url, NodeId};
use ipnetwork::{IpNetwork, IpNetworkError};
use std::str::FromStr;

const PROTOCOL_VERSION: u32 = 4;

/// Network IO protocol handler. This needs to be implemented for each new subprotocol.
/// All the handler function are called from within IO event loop.
/// `Message` is the type for message data.
pub trait NetworkProtocolHandler: Sync + Send {
	/// Initialize the handler
	fn initialize(&self, _io: &NetworkContext, _host_info: &HostInfo) {}
	/// Called when new network packet received.
	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]);
	/// Called when new peer is connected. Only called when peer supports the same protocol.
	fn connected(&self, io: &NetworkContext, peer: &PeerId);
	/// Called when a previously connected peer disconnects.
	fn disconnected(&self, io: &NetworkContext, peer: &PeerId);
	/// Timer function called after a timeout created with `NetworkContext::timeout`.
	fn timeout(&self, _io: &NetworkContext, _timer: TimerToken) {}
}

/// Non-reserved peer modes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NonReservedPeerMode {
	/// Accept them. This is the default.
	Accept,
	/// Deny them.
	Deny,
}

impl NonReservedPeerMode {
	/// Attempt to parse the peer mode from a string.
	pub fn parse(s: &str) -> Option<Self> {
		match s {
			"accept" => Some(NonReservedPeerMode::Accept),
			"deny" => Some(NonReservedPeerMode::Deny),
			_ => None,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct IpFilter {
    pub predefined: AllowIP,
    pub custom_allow: Vec<IpNetwork>,
    pub custom_block: Vec<IpNetwork>,
}

impl Default for IpFilter {
    fn default() -> Self {
        IpFilter {
            predefined: AllowIP::All,
            custom_allow: vec![],
            custom_block: vec![],
        }
    }
}

impl IpFilter {
    /// Attempt to parse the peer mode from a string.
    pub fn parse(s: &str) -> Result<IpFilter, IpNetworkError> {
        let mut filter = IpFilter::default();
        for f in s.split_whitespace() {
            match f {
                "all" => filter.predefined = AllowIP::All,
                "private" => filter.predefined = AllowIP::Private,
                "public" => filter.predefined = AllowIP::Public,
                "none" => filter.predefined = AllowIP::None,
                custom => {
                    if custom.starts_with("-") {
                        filter.custom_block.push(IpNetwork::from_str(&custom.to_owned().split_off(1))?)
                    } else {
                        filter.custom_allow.push(IpNetwork::from_str(custom)?)
                    }
                }
            }
        }
        Ok(filter)
    }
}

/// IP fiter
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllowIP {
	/// Connect to any address
	All,
	/// Connect to private network only
	Private,
	/// Connect to public network only
	Public,
    /// Block all addresses
    None,
}

