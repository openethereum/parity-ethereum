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

extern crate rustc_serialize;
extern crate mio;
extern crate rand;
extern crate rocksdb;
extern crate tiny_keccak;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate env_logger;

extern crate time;
extern crate crypto as rcrypto;
extern crate secp256k1;
extern crate arrayvec;
extern crate elastic_array;

pub mod error;
pub mod hash;
pub mod uint;
pub mod bytes;
pub mod rlp;
pub mod vector;
pub mod sha3;
pub mod hashdb;
pub mod memorydb;
pub mod overlaydb;
pub mod math;
pub mod chainfilter;
pub mod crypto;
pub mod triehash;
pub mod trie;
pub mod nibbleslice;

//pub mod network;
