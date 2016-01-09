#![feature(cell_extras)]

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

extern crate env_logger;
#[cfg(feature = "jit" )]
extern crate evmjit;

extern crate ethcore_util as util;

pub mod common;
pub mod env_info;
pub mod engine;
pub mod state;
pub mod account;
pub mod header;
pub mod transaction;
pub mod receipt;
pub mod null_engine;
pub mod evm_schedule;
pub mod builtin;
pub mod spec;
pub mod views;
pub mod blockchain;
pub mod extras;
pub mod evm;
pub mod block;

pub mod ethereum;
