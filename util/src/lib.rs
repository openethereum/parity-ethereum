// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
// TODO [todr] not really sure
#![cfg_attr(feature="dev", allow(needless_range_loop))]
// Shorter than if-else
#![cfg_attr(feature="dev", allow(match_bool))]
// We use that to be more explicit about handled cases
#![cfg_attr(feature="dev", allow(match_same_arms))]
// Keeps consistency (all lines with `.clone()`) and helpful when changing ref to non-ref.
#![cfg_attr(feature="dev", allow(clone_on_copy))]
// In most cases it expresses function flow better
#![cfg_attr(feature="dev", allow(if_not_else))]
// TODO [todr] a lot of warnings to be fixed
#![cfg_attr(feature="dev", allow(needless_borrow))]
#![cfg_attr(feature="dev", allow(assign_op_pattern))]
#![cfg_attr(feature="dev", allow(unnecessary_operation))]


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
//!   git clone https://github.com/ethcore/parity
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
//!   git clone https://github.com/ethcore/parity
//!   cd parity
//!   cargo build --release
//!   ```

extern crate slab;
extern crate rustc_serialize;
extern crate mio;
extern crate rand;
extern crate rocksdb;
extern crate tiny_keccak;
#[macro_use]
extern crate heapsize;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate itertools;
extern crate env_logger;
extern crate time;
extern crate crypto as rcrypto;
extern crate secp256k1;
extern crate arrayvec;
extern crate elastic_array;
extern crate crossbeam;
extern crate serde;
#[macro_use]
extern crate log as rlog;
extern crate igd;
extern crate ethcore_devtools as devtools;
extern crate libc;
extern crate target_info;
extern crate bigint;
extern crate chrono;

pub mod standard;
#[macro_use]
pub mod from_json;
#[macro_use]
pub mod common;
pub mod numbers;
pub mod error;
pub mod hash;
pub mod bytes;
pub mod rlp;
pub mod misc;
pub mod using_queue;
mod json_aid;
pub mod vector;
pub mod sha3;
pub mod hashdb;
pub mod memorydb;
pub mod migration;
pub mod overlaydb;
pub mod journaldb;
pub mod kvdb;
mod math;
pub mod crypto;
pub mod triehash;
pub mod trie;
pub mod nibbleslice;
mod heapsizeof;
pub mod squeeze;
pub mod semantic_version;
pub mod io;
pub mod network;
pub mod log;
pub mod panics;
pub mod keys;
pub mod table;
pub mod network_settings;
pub mod path;

pub use common::*;
pub use misc::*;
pub use using_queue::*;
pub use json_aid::*;
pub use rlp::*;
pub use hashdb::*;
pub use memorydb::*;
pub use overlaydb::*;
pub use journaldb::JournalDB;
pub use math::*;
pub use crypto::*;
pub use triehash::*;
pub use trie::*;
pub use nibbleslice::*;
pub use squeeze::*;
pub use semantic_version::*;
pub use network::*;
pub use io::*;
pub use log::*;
pub use kvdb::*;

#[cfg(test)]
mod tests {
	use super::numbers::*;
	use std::str::FromStr;

	#[test]
	fn u256_multi_muls() {

		let (result, _) = U256([0, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 0]), result);

		let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([1, 0, 0, 0]));
		assert_eq!(U256([1, 0, 0, 0]), result);

		let (result, _) = U256([5, 0, 0, 0]).overflowing_mul(U256([5, 0, 0, 0]));
		assert_eq!(U256([25, 0, 0, 0]), result);

		let (result, _) = U256([0, 5, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 0, 25, 0]), result);

		let (result, _) = U256([0, 0, 0, 1]).overflowing_mul(U256([1, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 1]), result);

		let (result, _) = U256([0, 0, 0, 5]).overflowing_mul(U256([2, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 10]), result);

		let (result, _) = U256([0, 0, 1, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 0, 0, 5]), result);

		let (result, _) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
		assert_eq!(U256([0, 0, 0, 0]), result);

		let (result, _) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 10, 0, 0]), result);

		let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert_eq!(U256([0, 0, 0, ::std::u64::MAX]), result);

		let x1 = U256::from_str("0000000000000000000000000000000000000000000000000000012365124623").unwrap();
		let x2sqr_right = U256::from_str("000000000000000000000000000000000000000000014baeef72e0378e2328c9").unwrap();
		let x1sqr = x1 * x1;
		assert_eq!(H256::from(x2sqr_right), H256::from(x1sqr));
		let x1cube = x1sqr * x1;
		let x1cube_right = U256::from_str("0000000000000000000000000000000001798acde139361466f712813717897b").unwrap();
		assert_eq!(H256::from(x1cube_right), H256::from(x1cube));
		let x1quad = x1cube * x1;
		let x1quad_right = U256::from_str("000000000000000000000001adbdd6bd6ff027485484b97f8a6a4c7129756dd1").unwrap();
		assert_eq!(H256::from(x1quad_right), H256::from(x1quad));
		let x1penta = x1quad * x1;
		let x1penta_right = U256::from_str("00000000000001e92875ac24be246e1c57e0507e8c46cc8d233b77f6f4c72993").unwrap();
		assert_eq!(H256::from(x1penta_right), H256::from(x1penta));
		let x1septima = x1penta * x1;
		let x1septima_right = U256::from_str("00022cca1da3f6e5722b7d3cc5bbfb486465ebc5a708dd293042f932d7eee119").unwrap();
		assert_eq!(H256::from(x1septima_right), H256::from(x1septima));
	}
}
