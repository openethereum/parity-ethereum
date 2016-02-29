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
#![cfg_attr(x64asm, feature(asm))]
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
extern crate rustc_version;
extern crate target_info;
extern crate vergen;

pub mod standard;
#[macro_use]
pub mod from_json;
#[macro_use]
pub mod common;
pub mod error;
pub mod hash;
pub mod uint;
pub mod bytes;
pub mod rlp;
pub mod misc;
mod json_aid;
pub mod vector;
pub mod sha3;
pub mod hashdb;
pub mod memorydb;
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

pub use common::*;
pub use misc::*;
pub use json_aid::*;
pub use rlp::*;
pub use hashdb::*;
pub use memorydb::*;
pub use overlaydb::*;
pub use journaldb::*;
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

