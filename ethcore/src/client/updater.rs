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

use std::sync::{Weak};
use std::{fs, env};
use std::path::PathBuf;
use util::misc::{VersionInfo, ReleaseTrack/*, platform*/};
use util::{Address, H160, H256, FixedHash, Mutex};
use client::operations::Operations;
use client::{Client, UpdatePolicy, UpdateFilter, BlockId};
use fetch::HashFetch;
use fetch;

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseInfo {
	pub version: VersionInfo,
	pub is_critical: bool,
	pub fork: u64,
	pub binary: Option<H256>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationsInfo {
	pub fork: u64,

	pub track: ReleaseInfo,
	pub minor: Option<ReleaseInfo>,
}

pub struct Updater {
	client: Weak<Client>,
	fetch: Weak<HashFetch>,
	operations: Operations,
	update_policy: UpdatePolicy,
	fetching: Mutex<Option<ReleaseInfo>>,

	// These don't change
	pub this: VersionInfo,
	pub this_fork: Option<u64>,

	// This does change
	pub latest: Option<OperationsInfo>,
	pub ready: Option<ReleaseInfo>,

}

const CLIENT_ID: &'static str = "parity";

fn platform() -> String {
	"test".to_owned()
}

impl Updater {
	pub fn new(client: Weak<Client>, fetch: Weak<fetch::Client>, operations: Address, update_policy: UpdatePolicy) -> Self {
		let mut u = Updater {
			client: client.clone(),
			fetch: fetch.clone(),
			operations: Operations::new(operations, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))),
			update_policy: update_policy,
			fetching: Mutex::new(None),
			this: VersionInfo::this(),
			this_fork: None,
			latest: None,
			ready: None,
		};

		u.this_fork = u.operations.release(CLIENT_ID, &u.this.hash.into()).ok()
			.and_then(|(fork, track, _, _)| if track > 0 {Some(fork as u64)} else {None});

		// TODO!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! REMOVE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
		if u.this.track == ReleaseTrack::Unknown {
			u.this.track = ReleaseTrack::Nightly;
		}

		u.latest = u.collect_latest().ok();

		u
	}

	/// Is the currently running client capable of supporting the current chain?
	/// `Some` answer or `None` if information on the running client is not available.
	pub fn is_capable(&self) -> Option<bool> {
		self.latest.as_ref().and_then(|latest| {
			self.this_fork.map(|this_fork| {
				let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));
				this_fork >= latest.fork || current_number < latest.fork
			})
		})
	}

	/// The release which is ready to be upgraded to, if any. If this returns `Some`, then
	/// `execute_upgrade` may be called.
	pub fn upgrade_ready(&self) -> Option<ReleaseInfo> {
		self.ready.clone()
	}

	/// Actually upgrades the client. Assumes that the binary has been downloaded.
	/// @returns `true` on success.
	pub fn execute_upgrade(&mut self) -> bool {
		// TODO: link ~/.parity-updates/parity to self.ready 
		// TODO: restart parity.
		unimplemented!()
	}

	/// Returns true iff the current version is capable of forming consensus.
	pub fn consensus_capable(&self) -> bool {
/*		if let Some(ref latest) = self.latest {
			

*/		unimplemented!()
	}

	/// Our version info.
	pub fn version_info(&self) -> &VersionInfo { &self.this }

	/// Information gathered concerning the release.
	pub fn info(&self) -> &Option<OperationsInfo> { &self.latest }

	fn collect_release_info(&self, release_id: &H256) -> Result<ReleaseInfo, String> {
		let (fork, track, semver, is_critical) = self.operations.release(CLIENT_ID, release_id)?;
		let latest_binary = self.operations.checksum(CLIENT_ID, release_id, &platform())?;
		Ok(ReleaseInfo {
			version: VersionInfo::from_raw(semver, track, release_id.clone().into()),
			is_critical: is_critical,
			fork: fork as u64,
			binary: if latest_binary.is_zero() { None } else { Some(latest_binary) },
		})
	}

	fn collect_latest(&self) -> Result<OperationsInfo, String> {
		if self.this.track == ReleaseTrack::Unknown {
			return Err(format!("Current executable ({}) is unreleased.", H160::from(self.this.hash)));
		}

		let latest_in_track = self.operations.latest_in_track(CLIENT_ID, self.this.track.into())?;
		let in_track = self.collect_release_info(&latest_in_track)?;
		let mut in_minor = Some(in_track.clone());
		const PROOF: &'static str = "in_minor initialised and assigned with Some; loop breaks if None assigned; qed";
		while in_minor.as_ref().expect(PROOF).version.track != self.this.track {
			let track = match in_minor.as_ref().expect(PROOF).version.track {
				ReleaseTrack::Beta => ReleaseTrack::Stable,
				ReleaseTrack::Nightly => ReleaseTrack::Beta,
				_ => { in_minor = None; break; }
			};
			in_minor = Some(self.collect_release_info(&self.operations.latest_in_track(CLIENT_ID, track.into())?)?);
		}

		Ok(OperationsInfo {
			fork: self.operations.latest_fork()? as u64,
			track: in_track,
			minor: in_minor,
		})
	}

	fn fetch_done(&mut self, _r: Result<PathBuf, fetch::Error>) {
		let fetched = self.fetching.lock().take().unwrap();
		match _r {
			Ok(b) => {
				info!("Fetched latest version ({}) OK to {}", fetched.version, b.display());
				let mut dest = PathBuf::from(env::home_dir().unwrap().to_str().expect("env filesystem paths really should be valid; qed"));
				dest.push(".parity-updates");
				match fs::create_dir_all(&dest) {
					Ok(_) => {
						dest.push(format!("parity-{}-{:?}", fetched.version, fetched.version.hash));
						match fs::copy(&b, &dest) {
							Ok(_) => {
								info!("Copied file to {}", dest.display());
								let auto = match self.update_policy.filter {
									UpdateFilter::All => true,
									UpdateFilter::Critical if fetched.is_critical /* TODO: or is on a bad fork */ => true,
									_ => false,
								};
								self.ready = Some(fetched);
								if auto {
									self.execute_upgrade();
								}
							},
							Err(e) => warn!("Unable to copy update: {:?}", e),
						}
					},
					Err(e) => warn!("Unable to create updates path: {:?}", e),
				}
			},
			Err(e) => warn!("Unable to fetch update ({}): {:?}", fetched.version, e),
		}
	}

	pub fn tick(&mut self) {
		info!(target: "updater", "Current release is {}", self.this);

		self.latest = self.collect_latest().ok();
		let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));

		if let Some(ref latest) = self.latest {
			info!(target: "updater", "Latest release in our track is v{} it is {}critical ({} binary is {})",
				latest.track.version,
				if latest.track.is_critical {""} else {"non-"},
				platform(),
				if let Some(ref b) = latest.track.binary {
					format!("{}", b)
				} else {
					"unreleased".into()
				}
			);
			if self.update_policy.enable_downloading && latest.track.version.hash != self.version_info().hash && self.ready.as_ref().map_or(true, |t| *t != latest.track) {
				if let Some(b) = latest.track.binary {
					let mut fetching = self.fetching.lock();
					if fetching.is_none() {
						info!("Attempting to get parity binary {}", b);
						let c = self.client.clone();
						let f = move |r: Result<PathBuf, fetch::Error>| if let Some(c) = c.upgrade() { c.updater().as_mut().expect("updater exists; updater only owned by client; qed").fetch_done(r); };
						if let Some(fetch) = self.fetch.clone().upgrade() {
							fetch.fetch(b, Box::new(f)).ok();
							*fetching = Some(latest.track.clone());
						}
					}
				}
			}
			info!(target: "updater", "Fork: this/current/latest/latest-known: {}/#{}/#{}/#{}", match self.this_fork { Some(f) => format!("#{}", f), None => "unknown".into(), }, current_number, latest.track.fork, latest.fork);
		}
	}
}
