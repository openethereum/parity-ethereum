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
#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]

// Clippy settings
// Most of the time much more readable
#![cfg_attr(feature="dev", allow(needless_range_loop))]
// Shorter than if-else
#![cfg_attr(feature="dev", allow(match_bool))]
// We use that to be more explicit about handled cases
#![cfg_attr(feature="dev", allow(match_same_arms))]
// Keeps consistency (all lines with `.clone()`).
#![cfg_attr(feature="dev", allow(clone_on_copy))]
// Some false positives when doing pattern matching.
#![cfg_attr(feature="dev", allow(needless_borrow))]
// TODO [todr] a lot of warnings to be fixed
#![cfg_attr(feature="dev", allow(assign_op_pattern))]


//! Ethcore-util library
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
//!   # install rocksdb
//!   add-apt-repository "deb http://ppa.launchpad.net/giskou/librocksdb/ubuntu trusty main"
//!   apt-get update
//!   apt-get install -y --force-yes librocksdb
//!
//!   # install multirust
//!   curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes
//!
//!   # install nightly and make it default
//!   multirust update nightly && multirust default nightly
//!
//!   # export rust LIBRARY_PATH
//!   export LIBRARY_PATH=/usr/local/lib
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
//!   # install rocksdb && multirust
//!   brew update
//!   brew install rocksdb
//!   brew install multirust
//!
//!   # install nightly and make it default
//!   multirust update nightly && multirust default nightly
//!
//!   # export rust LIBRARY_PATH
//!   export LIBRARY_PATH=/usr/local/lib
//!
//!   # download and build parity
//!   git clone https://github.com/paritytech/parity
//!   cd parity
//!   cargo build --release
//!   ```

extern crate rustc_hex;
extern crate rand;
extern crate rocksdb;
extern crate env_logger;
extern crate crypto as rcrypto;
extern crate secp256k1;
extern crate elastic_array;
extern crate time;
extern crate ethcore_devtools as devtools;
extern crate libc;
extern crate target_info;
extern crate ethcore_bigint as bigint;
extern crate parking_lot;
extern crate ansi_term;
extern crate tiny_keccak;
extern crate rlp;
extern crate regex;
extern crate lru_cache;
extern crate heapsize;
extern crate ethcore_logger;
extern crate hash as keccak;

#[macro_use]
extern crate log as rlog;

#[macro_use]
pub mod common;
pub mod error;
pub mod bytes;
pub mod misc;
pub mod vector;
//pub mod sha3;
pub mod hashdb;
pub mod memorydb;
pub mod migration;
pub mod overlaydb;
pub mod journaldb;
pub mod kvdb;
pub mod triehash;
pub mod trie;
pub mod nibbleslice;
pub mod nibblevec;
pub mod semantic_version;
pub mod snappy;
pub mod cache;
mod timer;

pub use misc::*;
pub use hashdb::*;
pub use memorydb::MemoryDB;
pub use overlaydb::*;
pub use journaldb::JournalDB;
pub use triehash::*;
pub use trie::{Trie, TrieMut, TrieDB, TrieDBMut, TrieFactory, TrieError, SecTrieDB, SecTrieDBMut};
pub use semantic_version::*;
pub use kvdb::*;
pub use timer::*;
pub use error::*;
pub use bytes::*;
pub use vector::*;
pub use bigint::prelude::*;
pub use bigint::hash;

pub use ansi_term::{Colour, Style};
pub use heapsize::HeapSizeOf;
pub use parking_lot::{Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// 160-bit integer representing account address
pub type Address = H160;
