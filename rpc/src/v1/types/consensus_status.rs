// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use ethereum_types::{H160, H256};
use semver;
use updater::{self, CapState};

/// Capability info
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConsensusCapability {
	/// Unknown.
	Unknown,
	/// Capable of consensus indefinitely.
	Capable,
	/// Capable of consensus up until a definite block.
	CapableUntil(u64),
	/// Incapable of consensus since a particular block.
	IncapableSince(u64),
}

impl Into<ConsensusCapability> for CapState {
	fn into(self) -> ConsensusCapability {
		match self {
			CapState::Unknown => ConsensusCapability::Unknown,
			CapState::Capable => ConsensusCapability::Capable,
			CapState::CapableUntil(n) => ConsensusCapability::CapableUntil(n),
			CapState::IncapableSince(n) => ConsensusCapability::IncapableSince(n),
		}
	}
}

/// A release's track.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReleaseTrack {
	/// Stable track.
	Stable,
	/// Nightly track.
	Nightly,
	/// Testing track.
	Testing,
	/// No known track.
	#[serde(rename = "null")]
	Unknown,
}

impl Into<ReleaseTrack> for updater::ReleaseTrack {
	fn into(self) -> ReleaseTrack {
		match self {
			updater::ReleaseTrack::Stable => ReleaseTrack::Stable,
			updater::ReleaseTrack::Nightly => ReleaseTrack::Nightly,
			updater::ReleaseTrack::Unknown => ReleaseTrack::Unknown,
		}
	}
}

/// Semantic version.
#[derive(Debug, PartialEq, Serialize)]
pub struct Version {
	/// Major part.
	major: u64,
	/// Minor part.
	minor: u64,
	/// Patch part.
	patch: u64,
}

impl Into<Version> for semver::Version {
	fn into(self) -> Version {
		Version {
			major: self.major,
			minor: self.minor,
			patch: self.patch,
		}
	}
}

/// Version information of a particular release.
#[derive(Debug, PartialEq, Serialize)]
pub struct VersionInfo {
	/// The track on which it was released.
	pub track: ReleaseTrack,
	/// The version.
	pub version: Version,
	/// The (SHA1?) 160-bit hash of this build's code base.
	pub hash: H160,
}

impl Into<VersionInfo> for updater::VersionInfo {
	fn into(self) -> VersionInfo {
		VersionInfo {
			track: self.track.into(),
			version: self.version.into(),
			hash: self.hash,
		}
	}
}

/// Information regarding a particular release of Parity
#[derive(Debug, PartialEq, Serialize)]
pub struct ReleaseInfo {
	/// Information on the version.
	pub version: VersionInfo,
	/// Does this release contain critical security updates?
	pub is_critical: bool,
	/// The latest fork that this release can handle.
	pub fork: u64,
	/// Our platform's binary, if known.
	pub binary: Option<H256>,
}

impl Into<ReleaseInfo> for updater::ReleaseInfo {
	fn into(self) -> ReleaseInfo {
		ReleaseInfo {
			version: self.version.into(),
			is_critical: self.is_critical,
			fork: self.fork,
			binary: self.binary.map(Into::into),
		}
	}
}

/// Information on our operations environment.
#[derive(Debug, PartialEq, Serialize)]
pub struct OperationsInfo {
	/// Our blockchain's latest fork.
	pub fork: u64,
	/// Last fork our client supports, if known.
	pub this_fork: Option<u64>,
	/// Information on our track's latest release.
	pub track: ReleaseInfo,
	/// Information on our minor version's latest release.
	pub minor: Option<ReleaseInfo>,
}

impl Into<OperationsInfo> for updater::OperationsInfo {
	fn into(self) -> OperationsInfo {
		OperationsInfo {
			fork: self.fork,
			this_fork: self.this_fork,
			track: self.track.into(),
			minor: self.minor.map(Into::into),
		}
	}
}
