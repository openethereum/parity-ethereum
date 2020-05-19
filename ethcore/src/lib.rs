// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

#![warn(missing_docs, unused_extern_crates)]

//! Ethcore library

pub extern crate account_state;
pub extern crate ansi_term;
pub extern crate client_traits;
pub extern crate common_types as types;
pub extern crate engine;
pub extern crate ethcore_blockchain as blockchain;
pub extern crate ethcore_call_contract as call_contract;
pub extern crate ethcore_db as db;
pub extern crate ethcore_io as io;
pub extern crate ethcore_miner;
pub extern crate ethereum_types;
pub extern crate executive_state;
pub extern crate futures;
pub extern crate hash_db;
pub extern crate itertools;
pub extern crate journaldb;
pub extern crate keccak_hash as hash;
pub extern crate kvdb;
pub extern crate machine;
pub extern crate memory_cache;
pub extern crate parity_bytes as bytes;
pub extern crate parking_lot;
pub extern crate trie_db as trie;
pub extern crate patricia_trie_ethereum as ethtrie;
pub extern crate rand;
pub extern crate rayon;
pub extern crate registrar;
pub extern crate rlp;
pub extern crate rustc_hex;
pub extern crate serde;
pub extern crate snapshot;
pub extern crate spec;
pub extern crate state_db;
pub extern crate trace;
pub extern crate trie_vm_factories;
pub extern crate triehash_ethereum as triehash;
pub extern crate unexpected;
pub extern crate using_queue;
pub extern crate verification;
pub extern crate vm;

#[cfg(test)]
extern crate account_db;
#[cfg(test)]
extern crate ethcore_accounts as accounts;

#[cfg(feature = "stratum")]
extern crate ethcore_stratum;

#[cfg(feature = "stratum")]
extern crate ethash;

#[cfg(any(test, feature = "test-helpers"))]
extern crate parity_crypto;
#[cfg(any(test, feature = "test-helpers"))]
extern crate ethjson;
#[cfg(any(test, feature = "test-helpers"))]
extern crate kvdb_memorydb;
#[cfg(any(test, feature = "kvdb-rocksdb"))]
extern crate kvdb_rocksdb;
#[cfg(feature = "json-tests")]
#[macro_use]
extern crate lazy_static;
#[cfg(any(test, feature = "test-helpers"))]
pub extern crate pod;
#[cfg(any(test, feature = "blooms-db"))]
extern crate blooms_db;
#[cfg(feature = "env_logger")]
extern crate env_logger;
#[cfg(test)]
extern crate serde_json;
#[cfg(any(test, feature = "tempdir"))]
extern crate tempfile;

#[macro_use]
pub extern crate log;
#[macro_use]
pub extern crate trace_time;

#[cfg_attr(test, macro_use)]
pub extern crate evm;

#[cfg(all(test, feature = "price-info"))]
pub extern crate fetch;

#[cfg(all(test, feature = "price-info"))]
extern crate parity_runtime;

pub mod block;
pub mod client;
pub mod miner;

#[cfg(test)]
mod tests;
#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers;
