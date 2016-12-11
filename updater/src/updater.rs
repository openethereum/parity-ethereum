// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use std::sync::{Arc, Weak};
use std::{fs, env};
use std::io::Write;
use std::path::{PathBuf};
use util::misc::{VersionInfo, ReleaseTrack/*, platform*/};
use util::{Address, H160, H256, FixedHash, Mutex, Bytes};
use ethcore::client::{BlockId, BlockChainClient, ChainNotify};
use hash_fetch::{self as fetch, HashFetch};
use operations::Operations;
use service::{Service, ReleaseInfo, OperationsInfo, CapState};

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
	/// Disable client if we know we're incapable of syncing.
	pub require_consensus: bool,
	/// Which of those downloaded should be automatically installed.
	pub filter: UpdateFilter,
}

impl Default for UpdatePolicy {
	fn default() -> Self {
		UpdatePolicy {
			enable_downloading: false,
			require_consensus: true,
			filter: UpdateFilter::None,
		}
	}
}

#[derive(Debug, Default)]
struct UpdaterState {
	latest: Option<OperationsInfo>,

	fetching: Option<ReleaseInfo>,
	ready: Option<ReleaseInfo>,
	installed: Option<ReleaseInfo>,

	capability: CapState,
}

/// Service for checking for updates and determining whether we can achieve consensus.
pub struct Updater {
	// Useful environmental stuff.
	update_policy: UpdatePolicy,
	weak_self: Mutex<Weak<Updater>>,
	client: Weak<BlockChainClient>,
	fetcher: Mutex<Option<fetch::Client>>,
	operations: Mutex<Option<Operations>>,
	exit_handler: Mutex<Option<Box<Fn() + 'static + Send>>>,

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

impl Updater {
	pub fn new(client: Weak<BlockChainClient>, update_policy: UpdatePolicy) -> Arc<Self> {
		let mut u = Updater {
			update_policy: update_policy,
			weak_self: Mutex::new(Default::default()),
			client: client.clone(),
			fetcher: Mutex::new(None),
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
		*r.fetcher.lock() = Some(fetch::Client::new(r.clone()));
		*r.weak_self.lock() = Arc::downgrade(&r);
		r.poll();
		r
	}

	/// Set a closure to call when we want to restart the client
	pub fn set_exit_handler<F>(&self, f: F) where F: Fn() + 'static + Send {
		*self.exit_handler.lock() = Some(Box::new(f)); 
	}

	fn collect_release_info(operations: &Operations, release_id: &H256) -> Result<ReleaseInfo, String> {
		let (fork, track, semver, is_critical) = operations.release(CLIENT_ID, release_id)?;
		let latest_binary = operations.checksum(CLIENT_ID, release_id, &platform())?;
		Ok(ReleaseInfo {
			version: VersionInfo::from_raw(semver, track, release_id.clone().into()),
			is_critical: is_critical,
			fork: fork as u64,
			binary: if latest_binary.is_zero() { None } else { Some(latest_binary) },
		})
	}

	fn collect_latest(&self) -> Result<OperationsInfo, String> {
		if let Some(ref operations) = *self.operations.lock() {
			let this_fork = operations.release(CLIENT_ID, &self.this.hash.into()).ok()
				.and_then(|(fork, track, _, _)| if track > 0 {Some(fork as u64)} else {None});

			if self.this.track == ReleaseTrack::Unknown {
				return Err(format!("Current executable ({}) is unreleased.", H160::from(self.this.hash)));
			}

			let latest_in_track = operations.latest_in_track(CLIENT_ID, self.this.track.into())?;
			let in_track = Self::collect_release_info(operations, &latest_in_track)?;
			let mut in_minor = Some(in_track.clone());
			const PROOF: &'static str = "in_minor initialised and assigned with Some; loop breaks if None assigned; qed";
			while in_minor.as_ref().expect(PROOF).version.track != self.this.track {
				let track = match in_minor.as_ref().expect(PROOF).version.track {
					ReleaseTrack::Beta => ReleaseTrack::Stable,
					ReleaseTrack::Nightly => ReleaseTrack::Beta,
					_ => { in_minor = None; break; }
				};
				in_minor = Some(Self::collect_release_info(operations, &operations.latest_in_track(CLIENT_ID, track.into())?)?);
			}

			Ok(OperationsInfo {
				fork: operations.latest_fork()? as u64,
				this_fork: this_fork,
				track: in_track,
				minor: in_minor,
			})
		} else {
			Err("Operations not available".into())
		}
	}

	fn update_file_name(v: &VersionInfo) -> String {
		format!("parity-{}.{}.{}-{:?}", v.version.major, v.version.minor, v.version.patch, v.hash)
	}

	fn updates_path(name: &str) -> PathBuf {
		let mut dest = PathBuf::from(env::home_dir().unwrap().to_str().expect("env filesystem paths really should be valid; qed"));
		dest.push(".parity-updates");
		dest.push(name);
		dest
	}

	fn fetch_done(&self, result: Result<PathBuf, fetch::Error>) {
		(|| -> Result<(), String> {
			let auto = {
				let mut s = self.state.lock();
				let fetched = s.fetching.take().unwrap();
				let b = result.map_err(|e| format!("Unable to fetch update ({}): {:?}", fetched.version, e))?;
				info!("Fetched latest version ({}) OK to {}", fetched.version, b.display());
				let dest = Self::updates_path(&Self::update_file_name(&fetched.version));
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
				// will lock self.state, so ensure it's outside of previous block.
				self.execute_upgrade();
			}
			Ok(())
		})().unwrap_or_else(|e| warn!("{}", e));
	}

	fn poll(&self) {
		info!(target: "updater", "Current release is {}", self.this);

		if self.operations.lock().is_none() {
			if let Some(ops_addr) = self.client.upgrade().and_then(|c| c.registry_address("operations".into())) {
				trace!(target: "client", "Found operations at {}", ops_addr);
				let client = self.client.clone();
				*self.operations.lock() = Some(Operations::new(ops_addr, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))));
			} else {
				// No Operations contract - bail.
				return;
			}
		}

		let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));

		let mut capability = CapState::Unknown; 
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
					if s.fetching.is_none() {
						info!("Attempting to get parity binary {}", b);
						s.fetching = Some(latest.track.clone());
						let weak_self = self.weak_self.lock().clone();
						let f = move |r: Result<PathBuf, fetch::Error>| if let Some(this) = weak_self.upgrade() { this.fetch_done(r) };
						self.fetcher.lock().as_ref().expect("Created on `new`; qed").fetch(b, Box::new(f)).ok();
					}
				}
			}
			info!(target: "updater", "Fork: this/current/latest/latest-known: {}/#{}/#{}/#{}", match latest.this_fork { Some(f) => format!("#{}", f), None => "unknown".into(), }, current_number, latest.track.fork, latest.fork);

			if let Some(this_fork) = latest.this_fork {
				if this_fork < latest.fork {
					// We're behind the latest fork. Now is the time to be upgrading; perhaps we're too late... 
					if let Some(c) = self.client.upgrade() {
						let current_number = c.block_number(BlockId::Latest).unwrap_or(0);
						if current_number >= latest.fork - 1 {
							// We're at (or past) the last block we can import. Disable the client.
							if self.update_policy.require_consensus {
								c.disable();
							}
							capability = CapState::IncapableSince(latest.fork);
						} else {
							capability = CapState::CapableUntil(latest.fork);
						}
					}
				} else {
					capability = CapState::Capable;
				}
			}
		}

		let mut s = self.state.lock();
		s.latest = latest;
		s.capability = capability;
	}
}

impl ChainNotify for Updater {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, _enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, _duration: u64) {
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

impl Service for Updater {
	fn capability(&self) -> CapState {
		self.state.lock().capability
	}

	fn upgrade_ready(&self) -> Option<ReleaseInfo> {
		self.state.lock().ready.clone()
	}

	fn execute_upgrade(&self) -> bool {
		(|| -> Result<bool, String> {
			let mut s = self.state.lock();
			if let Some(r) = s.ready.take() {
				let p = Self::update_file_name(&r.version);
				let n = Self::updates_path("latest");
				// TODO: creating then writing is a bit fragile. would be nice to make it atomic.
				match fs::File::create(&n).and_then(|mut f| f.write_all(p.as_bytes())) {
					Ok(_) => {
						info!("Completed upgrade to {}", &r.version);
						s.installed = Some(r);
						if let Some(ref h) = *self.exit_handler.lock() {
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

	fn version_info(&self) -> VersionInfo { self.this.clone() }

	fn info(&self) -> Option<OperationsInfo> { self.state.lock().latest.clone() }
}