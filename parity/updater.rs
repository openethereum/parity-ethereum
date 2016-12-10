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
use std::{io, os, fs, env};
use std::path::{Path, PathBuf};
use util::misc::{VersionInfo, ReleaseTrack/*, platform*/};
use util::{Address, H160, H256, FixedHash, Mutex};
use super::operations::Operations;
use ethcore::client::{Client, BlockId};
use fetch::HashFetch;
use fetch;

/// Filter for releases.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum UpdateFilter {
	/// All releases following the same track.
	All,
	/// As with `All`, but only those which are known to be critical. 
	Critical,
	/// None.
	None,
}

/// The policy for auto-updating.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UpdatePolicy {
	/// Download potential updates.
	pub enable_downloading: bool,
	/// Which of those downloaded should be automatically installed.
	pub filter: UpdateFilter,
}

impl Default for UpdatePolicy {
	fn default() -> Self {
		UpdatePolicy {
			enable_downloading: false,
			filter: UpdateFilter::None,
		}
	}
}

/// Information regarding a particular release of Parity
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Default)]
struct UpdaterState {
	latest: Option<OperationsInfo>,

	fetching: Option<ReleaseInfo>,
	ready: Option<ReleaseInfo>,
	installed: Option<ReleaseInfo>,
}

/// Service for checking for updates and determining whether we can achieve consensus.
pub struct Updater {
	// Useful environmental stuff.
	update_policy: UpdatePolicy,
	weak_self: Weak<Updater>,
	client: Weak<Client>,
	fetcher: Option<fetch::Client>,
	operations: Mutex<Option<Operations>>,
	exit_handler: Mutex<Option<Fn()>>,

	// Our version info (static)
	this: VersionInfo,

	// All the other info - this changes so leave it behind a Mutex.
	state: Mutex<UpdaterState>,
}

const CLIENT_ID: &'static str = "parity";

// TODO!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! REMOVE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
fn platform() -> String {
	"test".to_owned()
}

#[cfg(windows)]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
	os::windows::fs::symlink_file(src, dst)
}

#[cfg(not(windows))]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
	os::unix::fs::symlink(src, dst)
}

impl Updater {
	pub fn new(client: Weak<BlockChainClient>, update_policy: UpdatePolicy) -> Arc<Self> {
		let mut u = Updater {
			update_policy: update_policy,
			weak_self: Default::default(),
			client: client.clone(),
			fetcher: None,
			operations: Mutex::new(None),
			exit_handler: Mutex::new(None),
			this: VersionInfo::this(),
			state: Mutex::new(Default::default()),
		};

		// TODO!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! REMOVE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
		if u.this.track == ReleaseTrack::Unknown {
			u.this.track = ReleaseTrack::Nightly;
		}

		let r = Arc::new(u);
		r.as_mut().weak_self = Arc::downgrade(&r);
		r.as_mut().fetcher = Some(fetch::Client::new(r));
		r
	}

	/// Is the currently running client capable of supporting the current chain?
	/// `Some` answer or `None` if information on the running client is not available.
	pub fn is_capable(&self) -> Option<bool> {
		self.state.lock().latest.as_ref().and_then(|latest| {
			latest.this_fork.map(|this_fork| {
				let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));
				this_fork >= latest.fork || current_number < latest.fork
			})
		})
	}

	/// The release which is ready to be upgraded to, if any. If this returns `Some`, then
	/// `execute_upgrade` may be called.
	pub fn upgrade_ready(&self) -> Option<ReleaseInfo> {
		self.state.lock().ready.clone()
	}

	/// Actually upgrades the client. Assumes that the binary has been downloaded.
	/// @returns `true` on success.
	pub fn execute_upgrade(&mut self) -> bool {
		(|| -> Result<bool, String> {
			let s = state.lock();
			if let Some(r) = s.ready.take() {
				let p = Self::update_file_path(&r.version);
				let n = Self::updates_latest();
				let _ = fs::remove_file(&n);
				match symlink(p, n) {
					Ok(_) => {
						info!("Completed upgrade to {}", &r.version);
						s.installed = Some(r);
						if let Some(ref h) = self.exit_handler().lock() {
							(*h)();
						}
						Ok(true)
					}
					Err(e) => {
						s.ready = Some(r);
						Err(format!("Unable to create soft-link for update {:?}", e))
					}
				}
			} else {
				warn!("Execute upgrade called when no upgrade ready.");
				Ok(false)
			}
		})().unwrap_or_else(|e| { warn!("{}", e); false })
	}

	/// Returns true iff the current version is capable of forming consensus.
	pub fn is_consensus_capable(&self) -> bool {
/*		if let Some(ref latest) = self.latest {
			

*/		unimplemented!()
	}

	/// Our version info.
	pub fn version_info(&self) -> &VersionInfo { &self.this }

	/// Information gathered concerning the release.
	pub fn info(&self) -> Option<OperationsInfo> { self.state.lock().latest.clone() }

	/// Set a closure to call when we want to restart the client
	pub fn set_exit_handler(&self, f: Fn()) {
		*self.exit_handler.lock() = f; 
	}

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
		let this_fork = u.operations.release(CLIENT_ID, &u.this.hash.into()).ok()
			.and_then(|(fork, track, _, _)| if track > 0 {Some(fork as u64)} else {None});

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
			this_fork: this_fork,
			track: in_track,
			minor: in_minor,
		})
	}

	fn update_file_path(v: &VersionInfo) -> PathBuf {
		let mut dest = PathBuf::from(env::home_dir().unwrap().to_str().expect("env filesystem paths really should be valid; qed"));
		dest.push(".parity-updates");
		dest.push(format!("parity-{}.{}.{}-{:?}", v.version.major, v.version.minor, v.version.patch, v.hash));
		dest
	}

	fn updates_latest() -> PathBuf {
		let mut dest = PathBuf::from(env::home_dir().unwrap().to_str().expect("env filesystem paths really should be valid; qed"));
		dest.push(".parity-updates");
		dest.push("parity");
		dest
	}

	fn fetch_done(&mut self, result: Result<PathBuf, fetch::Error>) {
		(|| -> Result<(), String> {
			let auto = {
				let mut s = state.lock();
				let fetched = s.fetching.take().unwrap();
				let b = result.map_err(|e| format!("Unable to fetch update ({}): {:?}", fetched.version, e))?;
				info!("Fetched latest version ({}) OK to {}", fetched.version, b.display());
				let dest = Self::update_file_path(&fetched.version);
				fs::create_dir_all(dest.parent().expect("at least one thing pushed; qed")).map_err(|e| format!("Unable to create updates path: {:?}", e))?;
				fs::copy(&b, &dest).map_err(|e| format!("Unable to copy update: {:?}", e))?;
				info!("Copied file to {}", dest.display());
				let auto = match self.update_policy.filter {
					UpdateFilter::All => true,
					UpdateFilter::Critical if fetched.is_critical /* TODO: or is on a bad fork */ => true,
					_ => false,
				};
				s.ready = Some(fetched);
				auto
			};
			if auto {
				self.execute_upgrade();
			}
			Ok(())
		})().unwrap_or_else(|e| warn!("{}", e));
	}

	fn poll(&mut self) {
		info!(target: "updater", "Current release is {}", self.this);

		if *self.operations.lock().is_none() {
			if let Some(ops_addr) = client.upgrade().registry_address("operations") {
				trace!(target: "client", "Found operations at {}", ops_addr);
				let client = self.client.clone();
				*self.operations.lock() = Some(Operations::new(ops_addr, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))));
			} else {
				// No Operations contract - bail.
				return;
			}
		}

		u.latest = u.collect_latest().ok();

		let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));

		let latest = self.collect_latest().ok();
		if let Some(ref latest) = latest {
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
			let mut s = self.state.lock();
			let running_latest = latest.track.version.hash == self.version_info().hash;
			let already_have_latest = s.installed.as_ref().or(s.ready.as_ref()).map_or(false, |t| *t == latest.track);
			if self.update_policy.enable_downloading && !running_latest && !already_have_latest {
				if let Some(b) = latest.track.binary {
					if fetching.is_none() {
						info!("Attempting to get parity binary {}", b);
						s.fetching = Some(latest.track.clone());
						let weak_self = self.weak_self.clone();
						let f = move |r: Result<PathBuf, fetch::Error>| if let Some(this) = weak_self.upgrade().as_mut() { this.fetch_done(r) }};
						fetcher.fetch(b, Box::new(f)).ok();
					}
				}
			}
			info!(target: "updater", "Fork: this/current/latest/latest-known: {}/#{}/#{}/#{}", match s.latest.this_fork { Some(f) => format!("#{}", f), None => "unknown".into(), }, current_number, s.latest.track.fork, s.latest.fork);
		}
		(*self.state.lock()).latest = latest;
	}
}

impl ChainNotify for Updater {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, _enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, duration: u64) {
		// TODO: something like this
//		if !self.client.upgrade().map_or(true, |c| c.is_major_syncing()) {
			self.poll();
//		}
	}
}

impl fetch::urlhint::ContractClient for Updater {
	fn registrar(&self) -> Result<Address, String> {
		self.client.upgrade().ok_or_else(|| "Client not available".to_owned())?
			.registrar_address()
			.ok_or_else(|| "Registrar not available".into())
	}

	fn call(&self, address: Address, data: Bytes) -> Result<Bytes, String> {
		self.client.upgrade().ok_or_else(|| "Client not available".to_owned())?
			.call_contract(address, data)
	}
}
