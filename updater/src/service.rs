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

use types::{CapState, ReleaseInfo, OperationsInfo, VersionInfo};

pub trait Service: Send + Sync {
	/// Is the currently running client capable of supporting the current chain?
	/// We default to true if there's no clear information.
	fn capability(&self) -> CapState;

	/// The release which is ready to be upgraded to, if any. If this returns `Some`, then
	/// `execute_upgrade` may be called.
	fn upgrade_ready(&self) -> Option<ReleaseInfo>;

	/// Actually upgrades the client. Assumes that the binary has been downloaded.
	/// @returns `true` on success.
	fn execute_upgrade(&self) -> bool;

	/// Our version info.
	fn version_info(&self) -> VersionInfo;

	/// Information gathered concerning the release.
	fn info(&self) -> Option<OperationsInfo>;
}
