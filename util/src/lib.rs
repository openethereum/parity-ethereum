#![warn(missing_docs)]
#![feature(op_assign_traits)]
#![feature(augmented_assignments)]
#![feature(associated_consts)]
#![feature(plugin)]
#![plugin(clippy)]
#![allow(needless_range_loop, match_bool)]
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
/// TODO [Gav Wood] Please document me
pub mod misc;
/// TODO [Gav Wood] Please document me
pub mod json_aid;
pub mod vector;
pub mod sha3;
pub mod hashdb;
pub mod memorydb;
pub mod overlaydb;
pub mod journaldb;
/// TODO [Gav Wood] Please document me
pub mod math;
pub mod chainfilter;
/// TODO [Gav Wood] Please document me
pub mod crypto;
pub mod triehash;
/// TODO [Gav Wood] Please document me
pub mod trie;
pub mod nibbleslice;
mod heapsizeof;
pub mod squeeze;
/// TODO [Gav Wood] Please document me
pub mod semantic_version;
/// TODO [Gav Wood] Please document me
pub mod io;
/// TODO [Gav Wood] Please document me
pub mod network;
pub mod log;

pub use common::*;
pub use misc::*;
pub use json_aid::*;
pub use rlp::*;
pub use hashdb::*;
pub use memorydb::*;
pub use overlaydb::*;
pub use journaldb::*;
pub use math::*;
pub use chainfilter::*;
pub use crypto::*;
pub use triehash::*;
pub use trie::*;
pub use nibbleslice::*;
pub use squeeze::*;
pub use semantic_version::*;
pub use network::*;
pub use io::*;
pub use log::*;
