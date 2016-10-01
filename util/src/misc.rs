// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Diff misc.

use common::*;
use rlp::{Stream, RlpStream};
use target_info::Target;

include!(concat!(env!("OUT_DIR"), "/version.rs"));
include!(concat!(env!("OUT_DIR"), "/rustc_version.rs"));

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
/// Boolean type for clean/dirty status.
pub enum Filth {
	/// Data has not been changed.
	Clean,
	/// Data has been changed.
	Dirty,
}

/// Get the standard version string for this software.
pub fn version() -> String {
	let sha3 = short_sha();
	let sha3_dash = if sha3.is_empty() { "" } else { "-" };
	let commit_date = commit_date().replace("-", "");
	let date_dash = if commit_date.is_empty() { "" } else { "-" };
	let env = Target::env();
	let env_dash = if env.is_empty() { "" } else { "-" };
	format!("Parity/v{}-unstable{}{}{}{}/{}-{}{}{}/rustc{}", env!("CARGO_PKG_VERSION"), sha3_dash, sha3, date_dash, commit_date, Target::arch(), Target::os(), env_dash, env, rustc_version())
}

/// Get the standard version data for this software.
pub fn version_data() -> Bytes {
	let mut s = RlpStream::new_list(4);
	let v =
		(u32::from_str(env!("CARGO_PKG_VERSION_MAJOR")).unwrap() << 16) +
		(u32::from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap() << 8) +
		u32::from_str(env!("CARGO_PKG_VERSION_PATCH")).unwrap();
	s.append(&v);
	s.append(&"Parity");
	s.append(&rustc_version());
	s.append(&&Target::os()[0..2]);
	s.out()
}
