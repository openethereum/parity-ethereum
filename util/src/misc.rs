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
use semver::{Identifier, Version};
use rlp::{Stream, RlpStream};
use target_info::Target;

include!(concat!(env!("OUT_DIR"), "/version.rs"));
include!(concat!(env!("OUT_DIR"), "/rustc_version.rs"));

/// Boolean type for clean/dirty status.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Filth {
	/// Data has not been changed.
	Clean,
	/// Data has been changed.
	Dirty,
}

/// A release's track.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ReleaseTrack {
	/// Stable track.
	Stable,
	/// Beta track.
	Beta,
	/// Nightly track.
	Nightly,
	/// No known track.
	Unknown,
}

impl fmt::Display for ReleaseTrack {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", match *self {
			ReleaseTrack::Stable => "stable", 
			ReleaseTrack::Beta => "beta", 
			ReleaseTrack::Nightly => "nightly", 
			ReleaseTrack::Unknown => "unknown", 
		})
	}
}

impl<'a> From<&'a str> for ReleaseTrack {
	fn from(s: &'a str) -> Self {
		match s {
			"stable" => ReleaseTrack::Stable, 
			"beta" => ReleaseTrack::Beta, 
			"nightly" => ReleaseTrack::Nightly, 
			_ => ReleaseTrack::Unknown, 
		}		
	}
}

impl From<u8> for ReleaseTrack {
	fn from(i: u8) -> Self {
		match i {
			1 => ReleaseTrack::Stable, 
			2 => ReleaseTrack::Beta, 
			3 => ReleaseTrack::Nightly, 
			_ => ReleaseTrack::Unknown, 
		}		
	}
}

impl Into<u8> for ReleaseTrack {
	fn into(self) -> u8 {
		match self {
			ReleaseTrack::Stable => 1, 
			ReleaseTrack::Beta => 2, 
			ReleaseTrack::Nightly => 3, 
			ReleaseTrack::Unknown => 0, 
		}		
	}
}

/// Version information of a particular release. 
#[derive(Debug, PartialEq)]
pub struct VersionInfo {
	/// The track on which it was released.
	pub track: ReleaseTrack,
	/// The version.
	pub version: Version,
	/// The (SHA1?) 160-bit hash of this build's code base.
	pub hash: H160,
}

impl fmt::Display for VersionInfo {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}-{}", self.version, self.hash)
	}
}

impl VersionInfo {
	/// Get information for this (currently running) binary.
	pub fn this() -> Self {
		VersionInfo {
			track: env!["CARGO_PKG_VERSION_PRE"].into(),
			version: Version::parse(env!["CARGO_PKG_VERSION"]).expect("Environment variables are known to be valid; qed"),
			hash: sha().into(),
		}
	}

	/// Compose the information from the provided raw fields.
	pub fn from_raw(semver: u32, track: u8, hash: H160) -> Self {
		let t = track.into();
		VersionInfo {
			version: Version {
				major: (semver >> 16) as u64,
				minor: ((semver >> 8) & 0xff) as u64,
				patch: (semver & 0xff) as u64,
				build: vec![],
				pre: vec![Identifier::AlphaNumeric(format!("{}", t))]
			},
			track: t,
			hash: hash,
		}
	}
}

/// Get the platform identifier.
pub fn platform() -> String {
	let env = Target::env();
	let env_dash = if env.is_empty() { "" } else { "-" };
	format!("{}-{}{}{}", Target::arch(), Target::os(), env_dash, env)
}

/// Get the standard version string for this software.
pub fn version() -> String {
	let sha3 = short_sha();
	let sha3_dash = if sha3.is_empty() { "" } else { "-" };
	let commit_date = commit_date().replace("-", "");
	let date_dash = if commit_date.is_empty() { "" } else { "-" };
	format!("Parity/v{}-unstable{}{}{}{}/{}/rustc{}", env!("CARGO_PKG_VERSION"), sha3_dash, sha3, date_dash, commit_date, platform(), rustc_version())
}

/// Get the standard version data for this software.
pub fn version_data() -> Bytes {
	let mut s = RlpStream::new_list(4);
	let v =
		(u32::from_str(env!("CARGO_PKG_VERSION_MAJOR")).expect("Environment variables are known to be valid; qed") << 16) +
		(u32::from_str(env!("CARGO_PKG_VERSION_MINOR")).expect("Environment variables are known to be valid; qed") << 8) +
		u32::from_str(env!("CARGO_PKG_VERSION_PATCH")).expect("Environment variables are known to be valid; qed");
	s.append(&v);
	s.append(&"Parity");
	s.append(&rustc_version());
	s.append(&&Target::os()[0..2]);
	s.out()
}
