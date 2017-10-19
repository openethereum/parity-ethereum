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
extern crate rocksdb;
extern crate env_logger;
extern crate secp256k1;
extern crate elastic_array;
extern crate libc;
extern crate target_info;
extern crate ethcore_bigint as bigint;
extern crate ethcore_bytes as bytes;
extern crate parking_lot;
extern crate tiny_keccak;
extern crate rlp;
extern crate heapsize;
extern crate ethcore_logger;
extern crate hash as keccak;
extern crate hashdb;
extern crate memorydb;
extern crate patricia_trie as trie;
extern crate kvdb;
extern crate util_error as error;

#[cfg(test)]
extern crate kvdb_memorydb;


pub mod misc;

pub use misc::*;
pub use hashdb::*;
pub use memorydb::MemoryDB;

/// 160-bit integer representing account address
pub type Address = bigint::hash::H160;
