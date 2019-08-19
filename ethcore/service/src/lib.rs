// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

extern crate ansi_term;
extern crate common_types;
extern crate client_traits;
extern crate ethcore;
extern crate ethcore_blockchain as blockchain;
extern crate ethcore_io as io;
extern crate ethcore_private_tx;
extern crate ethcore_sync as sync;
extern crate ethereum_types;
extern crate kvdb;
extern crate spec;

#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;

#[cfg(test)]
extern crate ethcore_db;
#[cfg(test)]
extern crate tempdir;

mod service;

#[cfg(test)]
extern crate kvdb_rocksdb;

pub use service::{ClientService, PrivateTxService};
