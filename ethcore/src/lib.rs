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
//!   git clone https://github.com/paritytech/parity-ethereum
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
//!   git clone https://github.com/paritytech/parity-ethereum
//!   cd parity
//!   cargo build --release
//!   ```

extern crate account_db;
extern crate account_state;
extern crate ansi_term;
extern crate client_traits;
extern crate common_types as types;
extern crate crossbeam_utils;
extern crate engine;
extern crate ethabi;
extern crate ethcore_blockchain as blockchain;
extern crate ethcore_bloom_journal as bloom_journal;
extern crate ethcore_call_contract as call_contract;
extern crate ethcore_db as db;
extern crate ethcore_io as io;
extern crate ethcore_miner;
extern crate ethereum_types;
extern crate executive_state;
extern crate futures;
extern crate hash_db;
extern crate itertools;
extern crate journaldb;
extern crate keccak_hash as hash;
extern crate keccak_hasher;
extern crate kvdb;
extern crate machine;
extern crate memory_cache;
extern crate num_cpus;
extern crate parity_bytes as bytes;
extern crate parity_snappy as snappy;
extern crate parking_lot;
extern crate trie_db as trie;
extern crate patricia_trie_ethereum as ethtrie;
extern crate rand;
extern crate rayon;
extern crate rlp;
extern crate serde;
extern crate spec;
extern crate state_db;
extern crate trace;
extern crate trie_vm_factories;
extern crate triehash_ethereum as triehash;
extern crate unexpected;
extern crate using_queue;
extern crate verification;
extern crate vm;

#[cfg(test)]
extern crate rand_xorshift;
#[cfg(test)]
extern crate ethcore_accounts as accounts;
#[cfg(feature = "stratum")]
extern crate ethcore_stratum;
#[cfg(any(test, feature = "stratum"))]
extern crate ethash;

#[cfg(any(test, feature = "test-helpers"))]
extern crate ethkey;
#[cfg(any(test, feature = "test-helpers"))]
extern crate ethjson;
#[cfg(any(test, feature = "test-helpers"))]
extern crate kvdb_memorydb;
#[cfg(any(test, feature = "kvdb-rocksdb"))]
extern crate kvdb_rocksdb;
#[cfg(any(test, feature = "json-tests"))]
#[macro_use]
extern crate lazy_static;
#[cfg(any(test, feature = "test-helpers"))]
#[macro_use]
extern crate macros;
#[cfg(test)]
extern crate null_engine;
#[cfg(any(test, feature = "test-helpers"))]
extern crate pod;
#[cfg(any(test, feature = "blooms-db"))]
extern crate blooms_db;
#[cfg(any(test, feature = "env_logger"))]
extern crate env_logger;
#[cfg(any(test, feature = "test-helpers"))]
extern crate rustc_hex;
#[cfg(test)]
extern crate serde_json;
#[cfg(any(test, feature = "tempdir"))]
extern crate tempdir;

#[macro_use]
extern crate ethabi_contract;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rlp_derive;
#[macro_use]
extern crate trace_time;

#[cfg_attr(test, macro_use)]
extern crate evm;

#[cfg(all(test, feature = "price-info"))]
extern crate fetch;

#[cfg(all(test, feature = "price-info"))]
extern crate parity_runtime;

pub mod block;
pub mod client;
pub mod miner;
pub mod snapshot;

#[cfg(test)]
mod tests;
#[cfg(feature = "json-tests")]
pub mod json_tests;
#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers;

pub use evm::CreateContractAddress;
pub use trie::TrieSpec;
