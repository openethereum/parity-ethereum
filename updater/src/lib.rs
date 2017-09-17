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

//! Updater for Parity executables

#[macro_use] extern crate log;
extern crate ethcore_util as util;
extern crate ethcore_bigint as bigint;
extern crate ethcore_bytes as bytes;
extern crate ipc_common_types;
extern crate parking_lot;
extern crate parity_hash_fetch as hash_fetch;
extern crate ethcore;
extern crate ethabi;
extern crate ethsync;
extern crate ethcore_ipc as ipc;
extern crate futures;
extern crate target_info;
extern crate parity_reactor;
extern crate path;

mod updater;
mod operations;
mod types;

mod service {
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/service.rs"));
}

pub use service::{Service};
pub use types::all::{ReleaseInfo, OperationsInfo, CapState, VersionInfo, ReleaseTrack};
pub use updater::{Updater, UpdateFilter, UpdatePolicy};
