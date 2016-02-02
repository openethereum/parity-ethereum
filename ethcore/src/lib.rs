#![warn(missing_docs)]
#![feature(cell_extras)]
#![feature(augmented_assignments)]
#![feature(plugin)]
#![plugin(clippy)]
#![allow(needless_range_loop, match_bool)]

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

#[macro_use] extern crate log;
#[macro_use] extern crate ethcore_util as util;
#[macro_use] extern crate lazy_static;
extern crate rustc_serialize;
extern crate flate2;
extern crate rocksdb;
extern crate heapsize;
extern crate crypto;
extern crate time;
extern crate env_logger;
extern crate num_cpus;
extern crate crossbeam;

#[cfg(feature = "jit" )] extern crate evmjit;

pub mod block;
pub mod blockchain;
pub mod block_queue;
pub mod client;
pub mod error;
pub mod ethereum;
pub mod header;
pub mod service;
pub mod spec;
pub mod views;

mod common;
mod basic_types;
#[macro_use] mod evm;
mod log_entry;
mod env_info;
mod pod_account;
mod pod_state;
mod account_diff;
mod state_diff;
mod engine;
mod state;
mod account;
mod action_params;
mod transaction;
mod receipt;
mod null_engine;
mod builtin;
mod extras;
mod substate;
mod executive;
mod externalities;
mod verification;

#[cfg(test)] mod tests;
