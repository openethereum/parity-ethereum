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

use std::sync::Weak;
use util::misc::{VersionInfo, ReleaseTrack, platform};
use util::{Address, H160, H256, FixedHash};
use client::operations::Operations;
use client::{Client, UpdatePolicy, BlockId};

pub struct ReleaseInfo {
	fork_supported: usize,
	latest_known_fork: usize,

	latest: VersionInfo,
	latest_fork: usize,
	latest_binary: Option<H256>,
}

pub struct Updater {
	client: Weak<Client>,
	operations: Operations,

	pub this: VersionInfo,
	pub release_info: Option<ReleaseInfo>,
	
}

impl Updater {
	pub fn new(client: Weak<Client>, operations: Address, _update_policy: UpdatePolicy) -> Self {
		let mut u = Updater {
			client: client.clone(),
			operations: Operations::new(operations, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))),
			this: VersionInfo::this(),
			release_info: None,
		};
		u.release_info = u.get_release_info().ok();
		if u.this.track == ReleaseTrack::Unknown {
			u.this.track = ReleaseTrack::Nightly;
		} 
		u
	}

	fn get_release_info(&mut self) -> Result<ReleaseInfo, String> {
		//601e0fb0fd7e9e1cec18f8872e8713117cab4e84
		if self.this.track == ReleaseTrack::Unknown {
			return Err(format!("Current executable ({}) is unreleased.", H160::from(self.this.hash)));
		}

		let client_id = "parity";
		let latest_known_fork = self.operations.latest_fork()?;
		let our_fork = self.operations.release(client_id, &self.this.hash.into())?.0;
		let latest_release = self.operations.latest_in_track(client_id, self.this.track.into())?;
		let (fork, track, semver, _critical) = self.operations.release(client_id, &latest_release)?;
		let maybe_latest_binary = self.operations.checksum(client_id, &latest_release, &platform())?;
		Ok(ReleaseInfo {
			fork_supported: our_fork as usize,
			latest_known_fork: latest_known_fork as usize,
			latest: VersionInfo::from_raw(semver, track, latest_release.into()),
			latest_fork: fork as usize,
			latest_binary: if maybe_latest_binary.is_zero() { None } else { Some(maybe_latest_binary) },
		})
	}

	pub fn tick(&mut self) {
		self.release_info = self.get_release_info().ok();
		let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));
		info!(target: "updater", "Current release is {}", self.this);
		if let Some(ref relinfo) = self.release_info {
			info!(target: "updater", "Latest release in our track is {} ({} binary is {})",
				relinfo.latest,
				platform(),
				if let Some(ref b) = relinfo.latest_binary {
					format!("{}", b)
				 } else {
					 "unreleased".into()
				 }
			);
			info!(target: "updater", "Fork: this/current/latest/latest-known: #{}/#{}/#{}/#{}", relinfo.fork_supported, current_number, relinfo.latest_fork, relinfo.latest_known_fork);
		}
	}
}
