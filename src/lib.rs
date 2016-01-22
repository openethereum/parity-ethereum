#![warn(missing_docs)]
#![feature(cell_extras)]
#![feature(augmented_assignments)]
//#![feature(plugin)]
//#![plugin(interpolate_idents)]
//! Ethcore's ethereum implementation
//!
//! ### Rust version
//! - beta
//! - nightly
//!
//! ### Supported platforms:
//! - OSX
//! - Linux/Ubuntu
//!
//! ### Dependencies:
//! - RocksDB 3.13
//! - LLVM 3.7 (optional, required for `jit`)
//! - evmjit (optional, required for `jit`)
//!
//! ### Dependencies Installation
//!
//! - OSX
//!
//!   - rocksdb
//!   ```bash
//!   brew install rocksdb
//!   ```
//!
//!   - llvm
//!
//!       - download llvm 3.7 from http://llvm.org/apt/
//!
//!       ```bash
//!       cd llvm-3.7.0.src
//!       mkdir build && cd $_
//!       cmake -G "Unix Makefiles" .. -DCMAKE_C_FLAGS_RELEASE= -DCMAKE_CXX_FLAGS_RELEASE= -DCMAKE_INSTALL_PREFIX=/usr/local/Cellar/llvm/3.7 -DCMAKE_BUILD_TYPE=Release
//!       make && make install
//!       ```
//!   - evmjit
//!
//!       - download from https://github.com/debris/evmjit
//!
//!       ```bash
//!       cd evmjit
//!       mkdir build && cd $_
//!       cmake -DLLVM_DIR=/usr/local/lib/llvm-3.7/share/llvm/cmake ..
//!       make && make install
//!       ```
//!
//! - Linux/Ubuntu
//!
//!   - rocksdb
//!
//!     ```bash
//!     wget https://github.com/facebook/rocksdb/archive/rocksdb-3.13.tar.gz
//!     tar xvf rocksdb-3.13.tar.gz && cd rocksdb-rocksdb-3.13 && make shared_lib
//!     sudo make install
//!     ```
//!
//!   - llvm
//!
//!       - install using packages from http://llvm.org/apt/
//!
//!   - evmjit
//!
//!       - download from https://github.com/debris/evmjit
//!
//!       ```bash
//!       cd evmjit
//!       mkdir build && cd $_
//!       cmake .. && make
//!       sudo make install
//!       sudo ldconfig
//!       ```

#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate flate2;
extern crate rocksdb;
extern crate heapsize;
extern crate crypto;
extern crate time;
extern crate env_logger;
extern crate num_cpus;
#[cfg(feature = "jit" )]
extern crate evmjit;
#[macro_use]
extern crate ethcore_util as util;

/// TODO [Gav Wood] Please document me
pub mod common;
/// TODO [Tomusdrw] Please document me
pub mod basic_types;
#[macro_use]
pub mod evm;
pub mod error;
/// TODO [Gav Wood] Please document me
pub mod log_entry;
/// TODO [Gav Wood] Please document me
pub mod env_info;
/// TODO [Gav Wood] Please document me
pub mod pod_account;
/// TODO [Gav Wood] Please document me
pub mod pod_state;
/// TODO [Gav Wood] Please document me
pub mod account_diff;
/// TODO [Gav Wood] Please document me
pub mod state_diff;
/// TODO [Gav Wood] Please document me
pub mod engine;
/// TODO [Gav Wood] Please document me
pub mod state;
/// TODO [Gav Wood] Please document me
pub mod account;
pub mod action_params;
/// TODO [debris] Please document me
pub mod header;
/// TODO [Gav Wood] Please document me
pub mod transaction;
/// TODO [Gav Wood] Please document me
pub mod receipt;
/// TODO [Gav Wood] Please document me
pub mod null_engine;
/// TODO [Gav Wood] Please document me
pub mod builtin;
/// TODO [debris] Please document me
pub mod spec;
pub mod views;
pub mod blockchain;
/// TODO [Gav Wood] Please document me
pub mod extras;
/// TODO [arkpar] Please document me
pub mod substate;
/// TODO [Gav Wood] Please document me
pub mod service;
pub mod executive;
pub mod externalities;

#[cfg(test)]
mod tests;

/// TODO [arkpar] Please document me
pub mod client;
/// TODO [arkpar] Please document me
pub mod sync;
/// TODO [arkpar] Please document me
pub mod block;
/// TODO [arkpar] Please document me
pub mod verification;
pub mod block_queue;
pub mod ethereum;
