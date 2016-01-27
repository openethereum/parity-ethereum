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
//! - beta
//! - nightly
//!
//! ### Supported platforms:
//! - OSX
//! - Linux
//!
//! ### Dependencies:
//! - RocksDB 3.13
//!
//! ### Dependencies Installation:
//!
//! - OSX:
//!
//!   ```bash
//!   brew install rocksdb
//!   ```
//!
//! - From source:
//!
//!   ```bash
//!   wget https://github.com/facebook/rocksdb/archive/rocksdb-3.13.tar.gz
//!   tar xvf rocksdb-3.13.tar.gz && cd rocksdb-rocksdb-3.13 && make shared_lib
//!   sudo make install
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
extern crate log;
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

/// TODO [Gav Wood] Please document me
pub mod standard;
#[macro_use]
/// TODO [Gav Wood] Please document me
pub mod from_json;
#[macro_use]
/// TODO [Gav Wood] Please document me
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
/// TODO [Gav Wood] Please document me
pub mod heapsizeof;
pub mod squeeze;
/// TODO [Gav Wood] Please document me
pub mod semantic_version;
/// TODO [Gav Wood] Please document me
pub mod io;
/// TODO [Gav Wood] Please document me
pub mod network;

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
pub use heapsizeof::*;
pub use squeeze::*;
pub use semantic_version::*;
pub use network::*;
pub use io::*;
