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
#![cfg_attr(feature="benches", feature(test))]
#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]

// Clippy settings
// Most of the time much more readable
#![cfg_attr(feature="dev", allow(needless_range_loop))]
// Shorter than if-else
#![cfg_attr(feature="dev", allow(match_bool))]
// Keeps consistency (all lines with `.clone()`).
#![cfg_attr(feature="dev", allow(clone_on_copy))]
// Complains on Box<E> when implementing From<Box<E>>
#![cfg_attr(feature="dev", allow(boxed_local))]
// Complains about nested modules with same name as parent
#![cfg_attr(feature="dev", allow(module_inception))]
// TODO [todr] a lot of warnings to be fixed
#![cfg_attr(feature="dev", allow(assign_op_pattern))]


//! Ethcore library
//!
//! ### Rust version:
//! - nightly
//!
//! ### Supported platforms:
//! - OSX
//! - Linux
//!
//! ### Building:
//!
//! - Ubuntu 14.04 and later:
//!
//!   ```bash
//!
//!   # install rustup
//!   curl https://sh.rustup.rs -sSf | sh
//!
//!   # download and build parity
//!   git clone https://github.com/paritytech/parity
//!   cd parity
//!   cargo build --release
//!   ```
//!
//! - OSX:
//!
//!   ```bash
//!   # install rocksdb && rustup
//!   brew update
//!   curl https://sh.rustup.rs -sSf | sh
//!
//!   # download and build parity
//!   git clone https://github.com/paritytech/parity
//!   cd parity
//!   cargo build --release
//!   ```

extern crate bloomchain;
extern crate bn;
extern crate byteorder;
extern crate crossbeam;
extern crate common_types as types;
extern crate crypto;
extern crate ethash;
extern crate ethcore_bloom_journal as bloom_journal;
extern crate ethcore_devtools as devtools;
extern crate ethcore_io as io;
extern crate ethcore_ipc_nano as nanoipc;
extern crate ethcore_bigint as bigint;
extern crate ethcore_bytes as bytes;
extern crate ethcore_logger;
extern crate ethcore_stratum;
extern crate ethjson;
extern crate ethkey;
extern crate futures;
extern crate hardware_wallet;
extern crate hashdb;
extern crate hyper;
extern crate itertools;
extern crate linked_hash_map;
extern crate lru_cache;
extern crate native_contracts;
extern crate num_cpus;
extern crate num;
extern crate parity_machine;
extern crate parking_lot;
extern crate price_info;
extern crate rand;
extern crate rayon;
extern crate rlp;
extern crate hash;
extern crate heapsize;
extern crate memorydb;
extern crate patricia_trie as trie;
extern crate triehash;
extern crate ansi_term;
extern crate semantic_version;
extern crate unexpected;
extern crate kvdb;
extern crate util_error;
extern crate snappy;
extern crate migration;

#[macro_use]
extern crate rlp_derive;
extern crate rustc_hex;
extern crate stats;
extern crate time;
extern crate transient_hashmap;
extern crate using_queue;
extern crate table;
extern crate bloomable;
extern crate vm;
extern crate wasm;
extern crate ethcore_util as util;

#[macro_use]
extern crate macros;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate ethcore_ipc as ipc;
#[cfg_attr(test, macro_use)]
extern crate evm;

#[cfg(feature = "jit" )]
extern crate evmjit;

pub extern crate ethstore;

pub mod account_provider;
pub mod block;
pub mod client;
pub mod db;
pub mod encoded;
pub mod engines;
pub mod error;
pub mod ethereum;
pub mod executed;
pub mod header;
pub mod machine;
pub mod migrations;
pub mod miner;
pub mod pod_state;
pub mod service;
pub mod snapshot;
pub mod spec;
pub mod state;
pub mod timer;
pub mod trace;
pub mod transaction;
pub mod verification;
pub mod views;

mod cache_manager;
mod blooms;
mod basic_types;
mod pod_account;
mod state_db;
mod account_db;
mod builtin;
mod executive;
mod externalities;
mod blockchain;
mod factory;
mod tx_filter;

#[cfg(test)]
mod tests;
#[cfg(test)]
#[cfg(feature="json-tests")]
mod json_tests;

pub use types::*;
pub use executive::contract_address;
pub use evm::CreateContractAddress;
