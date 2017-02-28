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
//!   # install multirust
//!   curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes
//!
//!   # export rust LIBRARY_PATH
//!   export LIBRARY_PATH=/usr/local/lib
//!
//!   # download and build parity
//!   git clone https://github.com/ethcore/parity
//!   cd parity
//!   multirust override beta
//!   cargo build --release
//!   ```
//!
//! - OSX:
//!
//!   ```bash
//!   # install rocksdb && multirust
//!   brew update
//!   brew install multirust
//!
//!   # export rust LIBRARY_PATH
//!   export LIBRARY_PATH=/usr/local/lib
//!
//!   # download and build parity
//!   git clone https://github.com/ethcore/parity
//!   cd parity
//!   multirust override beta
//!   cargo build --release
//!   ```


extern crate ethcore_io as io;
extern crate rustc_serialize;
extern crate crypto;
extern crate time;
extern crate env_logger;
extern crate num_cpus;
extern crate crossbeam;
extern crate ethjson;
extern crate bloomchain;
extern crate hyper;
extern crate ethash;
extern crate ethkey;
extern crate semver;
extern crate ethcore_ipc_nano as nanoipc;
extern crate ethcore_devtools as devtools;
extern crate rand;
extern crate bit_set;
extern crate rlp;
extern crate ethcore_bloom_journal as bloom_journal;
extern crate byteorder;
extern crate transient_hashmap;
extern crate linked_hash_map;
extern crate lru_cache;
extern crate ethcore_stratum;
extern crate ethabi;
extern crate hardware_wallet;
extern crate stats;

#[macro_use]
extern crate log;
#[macro_use]
extern crate ethcore_util as util;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate ethcore_ipc as ipc;

#[cfg(feature = "jit" )]
extern crate evmjit;

pub extern crate ethstore;

pub mod account_provider;
pub mod engines;
pub mod block;
pub mod client;
pub mod error;
pub mod ethereum;
pub mod header;
pub mod service;
pub mod trace;
pub mod spec;
pub mod views;
pub mod pod_state;
pub mod migrations;
pub mod miner;
pub mod snapshot;
pub mod action_params;
pub mod db;
pub mod verification;
pub mod state;
#[macro_use] pub mod evm;

mod cache_manager;
mod blooms;
mod basic_types;
mod env_info;
mod pod_account;
mod state_db;
mod account_db;
mod builtin;
mod executive;
mod externalities;
mod blockchain;
mod types;
mod factory;

#[cfg(test)]
mod tests;
#[cfg(test)]
#[cfg(feature="json-tests")]
mod json_tests;

pub use types::*;
pub use executive::contract_address;
