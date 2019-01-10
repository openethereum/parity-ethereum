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

//! Test implementation of fetch client.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use semver::Version;
use updater::{Service as UpdateService, CapState, ReleaseInfo, VersionInfo, OperationsInfo, ReleaseTrack};

/// Test implementation of fetcher. Will always return the same file.
#[derive(Default)]
pub struct TestUpdater {
	updated: AtomicBool,
	current_block: AtomicUsize,
}

impl TestUpdater {
	/// Update the (faked) current block.
	pub fn set_current_block(&self, n: usize) {
		self.current_block.store(n, Ordering::Relaxed);
	}

	/// Update the (faked) current block.
	pub fn set_updated(&self, v: bool) {
		self.updated.store(v, Ordering::Relaxed);
	}
}

impl UpdateService for TestUpdater {
	fn capability(&self) -> CapState {
		if self.updated.load(Ordering::Relaxed) {
			CapState::Capable
		} else {
			if self.current_block.load(Ordering::Relaxed) < 15100 {
				CapState::CapableUntil(15100)
			} else {
				CapState::IncapableSince(15100)
			}
		}
	}

	fn upgrade_ready(&self) -> Option<ReleaseInfo> {
		if self.updated.load(Ordering::Relaxed) {
			None
		} else {
			self.info().map(|i| i.track)
		}
	}

	fn execute_upgrade(&self) -> bool {
		if self.updated.load(Ordering::Relaxed) {
			false
		} else {
			self.updated.store(true, Ordering::Relaxed);
			true
		}
	}

	fn version_info(&self) -> VersionInfo {
		VersionInfo {
			track: ReleaseTrack::Beta,
			version: Version{major: 1, minor: 5, patch: 0, build: vec![], pre: vec![]},
			hash: 150.into(),
		}
	}

	fn info(&self) -> Option<OperationsInfo> {
		Some(OperationsInfo {
			fork: 15100,
			this_fork: Some(15000),
			track: ReleaseInfo {
				version: VersionInfo {
					track: ReleaseTrack::Beta,
					version: Version{major: 1, minor: 5, patch: 1, build: vec![], pre: vec![]},
					hash: 151.into(),
				},
				is_critical: true,
				fork: 15100,
				binary: Some(1510.into()),
			},
			minor: None,
		})
	}
}
