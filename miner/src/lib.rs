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

extern crate ansi_term;
extern crate ethcore_transaction as transaction;
extern crate ethereum_types;
extern crate futures;
extern crate futures_cpupool;
extern crate heapsize;
extern crate keccak_hash as hash;
extern crate linked_hash_map;
extern crate parking_lot;
extern crate price_info;
extern crate rayon;
extern crate rlp;
extern crate trace_time;
extern crate transaction_pool as txpool;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate rustc_hex;
#[cfg(test)]
extern crate ethkey;
#[cfg(test)]
extern crate env_logger;

pub mod external;
pub mod gas_pricer;
pub mod pool;
pub mod work_notify;
