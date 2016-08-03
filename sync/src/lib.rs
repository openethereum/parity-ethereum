// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

#![warn(missing_docs)]
#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]
// Keeps consistency (all lines with `.clone()`) and helpful when changing ref to non-ref.
#![cfg_attr(feature="dev", allow(clone_on_copy))]
// In most cases it expresses function flow better
#![cfg_attr(feature="dev", allow(if_not_else))]

//! Blockchain sync module
//! Implements ethereum protocol version 63 as specified here:
//! https://github.com/ethereum/wiki/wiki/Ethereum-Wire-Protocol
//!
//! Usage example:
//!
//! ```rust
//! extern crate ethcore_util as util;
//! extern crate ethcore;
//! extern crate ethsync;
//! use std::env;
//! use std::sync::Arc;
//! use util::io::IoChannel;
//! use ethcore::client::{Client, ClientConfig};
//! use ethsync::{EthSync, SyncConfig, ManageNetwork, NetworkConfiguration};
//! use ethcore::ethereum;
//! use ethcore::miner::{GasPricer, Miner};
//!
//! fn main() {
//! 	let dir = env::temp_dir();
//! 	let miner = Miner::new(
//! 		Default::default(),
//! 		GasPricer::new_fixed(20_000_000_000u64.into()),
//! 		ethereum::new_frontier(),
//! 		None
//! 	);
//! 	let client = Client::new(
//!			ClientConfig::default(),
//!			ethereum::new_frontier(),
//!			&dir,
//!			miner,
//!			IoChannel::disconnected()
//!		).unwrap();
//! 	let sync = EthSync::new(SyncConfig::default(), client, NetworkConfiguration::from(util::NetworkConfiguration::new())).unwrap();
//! 	sync.start_network();
//! }
//! ```

#[macro_use]
extern crate log;
#[macro_use]
extern crate ethcore_util as util;
extern crate ethcore_network as network;
extern crate ethcore_io as io;
extern crate ethcore;
extern crate env_logger;
extern crate time;
extern crate rand;
#[macro_use]
extern crate heapsize;
#[macro_use]
extern crate ethcore_ipc as ipc;
extern crate semver;
extern crate parking_lot;

mod chain;
mod blocks;
mod sync_io;

#[cfg(test)]
mod tests;

mod api {
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/api.rs"));
}

pub use api::{EthSync, SyncProvider, SyncClient, NetworkManagerClient, ManageNetwork, SyncConfig,
	ServiceConfiguration, NetworkConfiguration};
pub use chain::{SyncStatus, SyncState};
pub use network::{is_valid_node_url, NonReservedPeerMode, NetworkError};

