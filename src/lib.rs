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
//!   ```
//!   brew install rocksdb
//!   ```
//!   
//!   - llvm
//!     
//!       - download llvm 3.7 from http://llvm.org/apt/
//!
//!       ```
//!       cd llvm-3.7.0.src
//!       mkdir build && cd $_
//!       cmake -G "Unix Makefiles" .. -DCMAKE_C_FLAGS_RELEASE= -DCMAKE_CXX_FLAGS_RELEASE= -DCMAKE_INSTALL_PREFIX=/usr/local/Cellar/llvm/3.7 -DCMAKE_BUILD_TYPE=Release 
//!       make && make install
//!       ```
//!   - evmjit
//!   
//!       - download from https://github.com/debris/evmjit
//!       
//!       ```
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
//!     ```
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
//!       ```
//!       cd evmjit
//!       mkdir build && cd $_
//!       cmake .. && make
//!       sudo make install
//!       sudo ldconfig
//!       ```

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ethcore_util;
#[cfg(feature = "jit" )]
extern crate evmjit;

//use ethcore_util::error::*;
use ethcore_util::hash::*;
use ethcore_util::uint::*;
use ethcore_util::bytes::*;

pub type LogBloom = H2048;

pub mod state;

pub static ZERO_ADDRESS: Address = Address([0x00; 20]);
pub static ZERO_H256: H256 = H256([0x00; 32]);
pub static ZERO_LOGBLOOM: LogBloom = H2048([0x00; 256]);

#[derive(Debug)]
pub struct Header {
	parent_hash: H256,
	timestamp: U256,
	number: U256,
	author: Address,

	transactions_root: H256,
	uncles_hash: H256,
	extra_data_hash: H256,

	state_root: H256,
	receipts_root: H256,
	log_bloom: LogBloom,
	gas_used: U256,
	gas_limit: U256,

	difficulty: U256,
	seal: Vec<Bytes>,
}

impl Header {
	pub fn new() -> Header {
		Header {
			parent_hash: ZERO_H256.clone(),
			timestamp: BAD_U256.clone(),
			number: ZERO_U256.clone(),
			author: ZERO_ADDRESS.clone(),

			transactions_root: ZERO_H256.clone(),
			uncles_hash: ZERO_H256.clone(),
			extra_data_hash: ZERO_H256.clone(),

			state_root: ZERO_H256.clone(),
			receipts_root: ZERO_H256.clone(),
			log_bloom: ZERO_LOGBLOOM.clone(),
			gas_used: ZERO_U256.clone(),
			gas_limit: ZERO_U256.clone(),

			difficulty: ZERO_U256.clone(),
			seal: vec![],
		}
	}
}

pub struct Transaction {
	pub to: Address,
	pub gas: U256,
	pub data: Bytes,
	pub code: Bytes,
}
