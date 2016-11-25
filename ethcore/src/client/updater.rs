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

use std::sync::{Mutex, Weak, Arc};
use std::path::PathBuf;
use util::misc::{VersionInfo, ReleaseTrack, platform};
use std::str::FromStr;
use util::{Bytes, Address, H160, H256, FixedHash};
use client::operations::Operations;
use client::{Client, BlockChainClient, UpdatePolicy, BlockId};
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
	operations: Operations,
	update_policy: UpdatePolicy,
	fetch_handler: Mutex<Option<fetch::Client>>,

	// These don't change
	pub this: VersionInfo,
	pub this_fork: Option<u64>,

	// This does change
	pub latest: Option<OperationsInfo>,
}

const CLIENT_ID: &'static str = "parity";

struct FetchHandler {
	client: Weak<Client>,
}

impl fetch::urlhint::ContractClient for FetchHandler {
	fn registrar(&self) -> Result<Address, String> {
		self.client.upgrade().ok_or_else(|| "Client not available".to_owned())?
			.additional_params()
			.get("registrar")
			.and_then(|s| Address::from_str(s).ok())
			.ok_or_else(|| "Registrar not available".into())
	}

	fn call(&self, address: Address, data: Bytes) -> Result<Bytes, String> {
		self.client.upgrade().ok_or_else(|| "Client not available".to_owned())?
			.call_contract(address, data)
	}
}

fn start_fetch(client: Weak<Client>, hash: H256, on_done: Box<Fn(Result<PathBuf, fetch::Error>) + Send>) -> Result<fetch::Client, fetch::Error> {
	let f = fetch::Client::new(Arc::new(FetchHandler { client: client, }));
	let r = f.fetch(hash, on_done);
	r.map(|_| f)
}

impl Updater {
	pub fn new(client: Weak<Client>, operations: Address, update_policy: UpdatePolicy) -> Self {
		let mut u = Updater {
			client: client.clone(),
			operations: Operations::new(operations, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))),
			update_policy: update_policy,
			fetch_handler: Mutex::new(None),
			this: VersionInfo::this(),
			this_fork: None,
			latest: None,
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
	pub fn upgrade_ready(&self) -> Option<VersionInfo> {
		unimplemented!()
	}

	/// Actually upgrades the client. Assumes that the binary has been downloaded.
	/// @returns `true` on success. 
	pub fn execute_upgrade(&mut self) -> bool {
		unimplemented!()
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

	fn fetch_done(&self, _r: Result<PathBuf, fetch::Error>) {
		if let Ok(mut x) = self.fetch_handler.lock() {
			*x = None;
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
			if let Some(b) = latest.track.binary {
				if let Ok(mut fetch_handler) = self.fetch_handler.lock() {
					if fetch_handler.is_none() {
						let c = self.client.clone();
						let f = move |r: Result<PathBuf, fetch::Error>| if let Some(c) = c.upgrade() { c.updater().as_ref().expect("updater exists; updater only owned by client; qed").fetch_done(r); }; 
						*fetch_handler = start_fetch(self.client.clone(), b, Box::new(f)).ok();
					}
				}
			}
			info!(target: "updater", "Fork: this/current/latest/latest-known: {}/#{}/#{}/#{}", match self.this_fork { Some(f) => format!("#{}", f), None => "unknown".into(), }, current_number, latest.track.fork, latest.fork);
		}
	}
}
