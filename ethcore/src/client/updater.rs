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
	pub latest_known_fork: usize,

	pub latest: VersionInfo,
	pub latest_fork: usize,
	pub latest_binary: Option<H256>,
}

pub struct Updater {
	client: Weak<Client>,
	operations: Operations,
	update_policy: UpdatePolicy,

	pub this: VersionInfo,
	pub this_fork: Option<usize>,
	pub release_info: Option<ReleaseInfo>,
}

impl Updater {
	pub fn new(client: Weak<Client>, operations: Address, update_policy: UpdatePolicy) -> Self {
		let mut u = Updater {
			client: client.clone(),
			operations: Operations::new(operations, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))),
			update_policy: update_policy,
			this: VersionInfo::this(),
			this_fork: None,
			release_info: None,
		};

		let (fork, track, _, _) = self.operations.release(client_id, &v.hash.into())?;
		u.this_fork = if track > 0 { Some(fork) } else { None };

		u.release_info = u.get_release_info().ok();

		// TODO!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! REMOVE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
		if u.this.track == ReleaseTrack::Unknown {
			u.this.track = ReleaseTrack::Nightly;
		}

		u
	}

	/// Is the currently running client capable of supporting the current chain?
	/// `Some` answer or `None` if information on the running client is not available.  
	pub fn is_capable(&self) -> Option<bool> {
		self.release_info.and_then(|relinfo| {
			relinfo.fork_supported.map(|fork_supported| {
				let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));
				fork_supported >= relinfo.latest_fork || current_number < relinfo.latest_fork  
			})
		})
	}

	/// The release which is ready to be upgraded to, if any. If this returns `Some`, then
	/// `execute_upgrade` may be called.
	pub fn upgrade_ready(&self) -> Option<VersionInfo> {
		unimplemented!()
	}

	/// Actually upgrades the client. Assumes that the binary has been downloaded.
	/// @returns `true` on success. 
	pub fn execute_upgrade(&mut self) -> bool {
		unimplemented!()
	}

	/// Our version info. 
	pub fn version_info() -> &VersionInfo { &self.this }

	/// Information gathered concerning the release. 
	pub fn release_info() -> &Option<ReleaseInfo> { &self.release_info }

	fn get_release_info(&mut self) -> Result<ReleaseInfo, String> {
		if self.this.track == ReleaseTrack::Unknown {
			return Err(format!("Current executable ({}) is unreleased.", H160::from(self.this.hash)));
		}

		let client_id = "parity";


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
