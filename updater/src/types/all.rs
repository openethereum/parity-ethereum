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

use bigint::hash::H256;
pub use ipc_common_types::{VersionInfo, ReleaseTrack};

/// Information regarding a particular release of Parity
#[derive(Debug, Clone, PartialEq)]
#[binary]
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

/// Information on our operations environment.
#[derive(Debug, Clone, PartialEq)]
#[binary]
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

/// Information on the current version's consensus capabililty.
#[derive(Debug, Clone, Copy, PartialEq)]
#[binary]
pub enum CapState {
	/// Unknown.
	Unknown,
	/// Capable of consensus indefinitely.
	Capable,
	/// Capable of consensus up until a definite block.
	CapableUntil(u64),
	/// Incapable of consensus since a particular block.
	IncapableSince(u64),
}

impl Default for CapState {
	fn default() -> Self { CapState::Unknown }
}
