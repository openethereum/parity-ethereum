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

// Recursion limit required because of
// error_chain foreign_links.
#![recursion_limit="128"]

extern crate ansi_term;
extern crate bn;
extern crate byteorder;
extern crate common_types as types;
extern crate crossbeam;
extern crate ethabi;
extern crate ethash;
extern crate ethcore_blockchain as blockchain;
extern crate ethcore_bloom_journal as bloom_journal;
extern crate ethcore_db as db;
extern crate ethcore_io as io;
extern crate ethcore_miner;
extern crate ethereum_types;
extern crate ethjson;
extern crate ethkey;
extern crate ethstore;
extern crate hashdb;
extern crate heapsize;
extern crate itertools;
extern crate journaldb;
extern crate keccak_hash as hash;
extern crate keccak_hasher;
extern crate kvdb;
extern crate kvdb_memorydb;
extern crate len_caching_lock;
extern crate lru_cache;
extern crate memory_cache;
extern crate memorydb;
extern crate num;
extern crate num_cpus;
extern crate parity_bytes as bytes;
extern crate parity_crypto;
extern crate parity_machine;
extern crate parity_snappy as snappy;
extern crate parking_lot;
extern crate patricia_trie as trie;
extern crate patricia_trie_ethereum as ethtrie;
extern crate rand;
extern crate rayon;
extern crate rlp;
extern crate rustc_hex;
extern crate serde;
extern crate stats;
extern crate triehash_ethereum as triehash;
extern crate unexpected;
extern crate using_queue;
extern crate vm;
extern crate wasm;

#[cfg(feature = "stratum")]
extern crate ethcore_stratum;
#[cfg(any(test, feature = "tempdir"))]
extern crate tempdir;
#[cfg(any(test, feature = "kvdb-rocksdb"))]
extern crate kvdb_rocksdb;
#[cfg(any(test, feature = "blooms-db"))]
extern crate blooms_db;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
extern crate hardware_wallet;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
extern crate fake_hardware_wallet as hardware_wallet;

#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate macros;
#[macro_use]
extern crate rlp_derive;
#[macro_use]
extern crate trace_time;
#[macro_use]
extern crate serde_derive;

#[cfg_attr(test, macro_use)]
extern crate evm;

#[cfg(any(test, feature = "env_logger"))]
extern crate env_logger;
#[cfg(test)]
extern crate rlp_compress;

pub mod account_provider;
pub mod block;
pub mod builtin;
pub mod client;
pub mod engines;
pub mod error;
pub mod ethereum;
pub mod executed;
pub mod executive;
pub mod machine;
pub mod miner;
pub mod pod_state;
pub mod pod_account;
pub mod snapshot;
pub mod spec;
pub mod state;
pub mod state_db;
pub mod trace;
pub mod transaction_ext;
pub mod verification;

mod account_db;
mod externalities;
mod factory;
mod tx_filter;

#[cfg(test)]
mod tests;
#[cfg(feature = "json-tests")]
pub mod json_tests;
#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers;

pub use executive::contract_address;
pub use evm::CreateContractAddress;
pub use trie::TrieSpec;
