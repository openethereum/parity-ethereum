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

#![warn(missing_docs)]

//! Miner module
//! Keeps track of transactions and mined block.

extern crate common_types as types;
extern crate ethabi;
extern crate ethcore_transaction as transaction;
extern crate ethereum_types;
extern crate futures;
extern crate heapsize;
extern crate keccak_hash as hash;
extern crate linked_hash_map;
extern crate parking_lot;
extern crate table;
extern crate transient_hashmap;

#[cfg(test)]
extern crate ethkey;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate rustc_hex;

pub mod banning_queue;
pub mod external;
pub mod local_transactions;
pub mod transaction_queue;
pub mod work_notify;
