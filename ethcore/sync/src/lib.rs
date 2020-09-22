// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

#![warn(missing_docs)]

//! Blockchain sync module
//! Implements ethereum protocol version 63 as specified here:
//! https://github.com/ethereum/wiki/wiki/Ethereum-Wire-Protocol
//!

extern crate common_types as types;
extern crate derive_more;
extern crate ethcore;
extern crate ethcore_io as io;
extern crate ethcore_network as network;
extern crate ethcore_network_devp2p as devp2p;
extern crate ethereum_forkid;
extern crate ethereum_types;
extern crate ethkey;
extern crate ethstore;
extern crate fastmap;
extern crate keccak_hash as hash;
extern crate parity_bytes as bytes;
extern crate parking_lot;
extern crate primitive_types07;
extern crate rand;
extern crate rlp;
extern crate rlp04;
extern crate stats;
extern crate triehash_ethereum;

#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
extern crate kvdb_memorydb;
#[cfg(test)]
extern crate rustc_hex;

#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate macros;
#[macro_use]
extern crate log;
#[macro_use]
extern crate heapsize;
#[macro_use]
extern crate trace_time;

mod block_sync;
mod blocks;
mod chain;
mod snapshot;
mod sync_io;
mod transactions_stats;

#[cfg(test)]
mod tests;

mod api;

pub use api::*;
pub use chain::{SyncState, SyncStatus};
pub use devp2p::validate_node_url;
pub use network::{ConnectionDirection, ConnectionFilter, Error, ErrorKind, NonReservedPeerMode};
