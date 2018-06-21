// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Whisper P2P messaging system as a DevP2P subprotocol, with RPC and Rust
//! interface.

extern crate byteorder;
extern crate ethcore_crypto as crypto;
extern crate ethcore_network as network;
extern crate ethereum_types;
extern crate ethkey;
extern crate hex;
extern crate mem;
extern crate ordered_float;
extern crate parking_lot;
extern crate rand;
extern crate rlp;
extern crate serde;
extern crate slab;
extern crate smallvec;
extern crate tiny_keccak;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate log;

#[macro_use]
extern crate jsonrpc_macros;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate serde_json;

pub use self::message::Message;
pub use self::net::{Network, MessageHandler};

pub mod message;
pub mod net;
pub mod rpc;
