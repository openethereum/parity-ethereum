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

#![warn(missing_docs, unused_extern_crates)]

//! Blockchain sync module
//! Implements ethereum protocol version 63 as specified here:
//! https://github.com/ethereum/wiki/wiki/Ethereum-Wire-Protocol
//!

// needed to make the procedural macro `MallocSizeOf` to work
#[macro_use] extern crate parity_util_mem as malloc_size_of;

mod api;
mod chain;
mod blocks;
mod block_sync;
mod sync_io;
mod private_tx;
mod snapshot_sync;
mod transactions_stats;

pub mod light_sync;

#[cfg(test)]
mod tests;

pub use api::*;
pub use chain::{SyncStatus, SyncState};
pub use devp2p::validate_node_url;
pub use network::{NonReservedPeerMode, Error, ConnectionFilter, ConnectionDirection};
pub use private_tx::{PrivateTxHandler, NoopPrivateTxHandler, SimplePrivateTxHandler};
