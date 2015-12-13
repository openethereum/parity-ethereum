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
extern crate env_logger;
#[cfg(feature = "jit" )]
extern crate evmjit;
extern crate ethcore_util as util;

//use util::error::*;
pub use util::hash::*;
pub use util::uint::*;
pub use util::bytes::*;

pub mod state;
pub mod blockheader;
pub mod transaction;
pub mod networkparams;
pub mod denominations;

#[test]
fn it_works() {
}
