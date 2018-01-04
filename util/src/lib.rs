// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
//!   git clone https://github.com/paritytech/parity
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
//!   git clone https://github.com/paritytech/parity
//!   cd parity
//!   cargo build --release
//!   ```

extern crate ethcore_bigint as bigint;
extern crate hashdb;
extern crate memorydb;

pub use hashdb::*;
pub use memorydb::MemoryDB;

/// 160-bit integer representing account address
pub type Address = bigint::hash::H160;
