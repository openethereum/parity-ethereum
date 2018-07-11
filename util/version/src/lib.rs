// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Parity version specific information.

extern crate target_info;
extern crate parity_bytes as bytes;
extern crate rlp;

use target_info::Target;
use bytes::Bytes;
use rlp::RlpStream;

mod vergen {
	#![allow(unused)]
	include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

mod generated {
	include!(concat!(env!("OUT_DIR"), "/meta.rs"));
}

#[cfg(feature = "final")]
const THIS_TRACK: &'static str = generated::TRACK;
// ^^^ should be reset in Cargo.toml to "stable" or "beta" according to the release branch.

#[cfg(not(feature = "final"))]
const THIS_TRACK: &'static str = "unstable";
// ^^^ This gets used when we're not building a final release; should stay as "unstable".

/// Get the platform identifier.
pub fn platform() -> String {
	let env = Target::env();
	let env_dash = if env.is_empty() { "" } else { "-" };
	format!("{}-{}{}{}", Target::arch(), Target::os(), env_dash, env)
}

/// Get the standard version string for this software.
pub fn version() -> String {
	let sha3 = vergen::short_sha();
	let sha3_dash = if sha3.is_empty() { "" } else { "-" };
	let commit_date = vergen::commit_date().replace("-", "");
	let date_dash = if commit_date.is_empty() { "" } else { "-" };
	format!("Parity-Ethereum/v{}-{}{}{}{}{}/{}/rustc{}", env!("CARGO_PKG_VERSION"), THIS_TRACK, sha3_dash, sha3, date_dash, commit_date, platform(), generated::rustc_version())
}

/// Get the standard version data for this software.
pub fn version_data() -> Bytes {
	let mut s = RlpStream::new_list(4);
	let v =
		(env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().expect("Environment variables are known to be valid; qed") << 16) +
		(env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().expect("Environment variables are known to be valid; qed") << 8) +
		env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().expect("Environment variables are known to be valid; qed");
	s.append(&v);
	s.append(&"Parity-Ethereum");
	s.append(&generated::rustc_version());
	s.append(&&Target::os()[0..2]);
	s.out()
}

/// Provide raw information on the package.
pub fn raw_package_info() -> (&'static str, &'static str, &'static str) {
	(THIS_TRACK, env!["CARGO_PKG_VERSION"], vergen::sha())
}
