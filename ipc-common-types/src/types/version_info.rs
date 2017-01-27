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

//! Types used in the public API

use std::fmt;
use std::str::FromStr;
use semver::{Version};
use util::{H160, FixedHash};
use util::misc::raw_package_info;
use release_track::ReleaseTrack;

/// Version information of a particular release.
#[derive(Debug, Clone, PartialEq)]
#[binary]
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
		write!(f, "{}.{}.{}-{}-{}", self.version.major, self.version.minor, self.version.patch, self.track, self.hash)
	}
}

impl VersionInfo {
	/// Get information for this (currently running) binary.
	pub fn this() -> Self {
		let raw = raw_package_info();
		VersionInfo {
			track: raw.0.into(),
			version: { let mut v = Version::parse(raw.1).expect("Environment variables are known to be valid; qed"); v.build = vec![]; v.pre = vec![]; v },
			hash: H160::from_str(raw.2).unwrap_or_else(|_| H160::zero()),
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
				pre: vec![],
			},
			track: t,
			hash: hash,
		}
	}
}
