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

use std::fs;
use std::io::Write;
use std::path::{PathBuf};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use ethcore::client::{BlockId, BlockChainClient, ChainNotify};
use ethsync::{SyncProvider};
use hash_fetch::{self as fetch, HashFetch};
use path::restrict_permissions_owner;
use service::{Service};
use target_info::Target;
use types::{ReleaseInfo, OperationsInfo, CapState, VersionInfo, ReleaseTrack};
use ethereum_types::H256;
use bytes::Bytes;
use parking_lot::Mutex;
use version;

use_contract!(operations_contract, "Operations", "res/operations.json");

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
	/// Which track we should be following.
	pub track: ReleaseTrack,
	/// Path for the updates to go.
	pub path: String,
}

impl Default for UpdatePolicy {
	fn default() -> Self {
		UpdatePolicy {
			enable_downloading: false,
			require_consensus: true,
			filter: UpdateFilter::None,
			track: ReleaseTrack::Unknown,
			path: Default::default(),
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

	disabled: bool,

	backoff: Option<(u32, Instant)>,
}

/// Service for checking for updates and determining whether we can achieve consensus.
pub struct Updater {
	// Useful environmental stuff.
	update_policy: UpdatePolicy,
	weak_self: Mutex<Weak<Updater>>,
	client: Weak<BlockChainClient>,
	sync: Weak<SyncProvider>,
	fetcher: fetch::Client,
	operations_contract: operations_contract::Operations,
	exit_handler: Mutex<Option<Box<Fn() + 'static + Send>>>,

	// Our version info (static)
	this: VersionInfo,

	// All the other info - this changes so leave it behind a Mutex.
	state: Mutex<UpdaterState>,
}

const CLIENT_ID: &'static str = "parity";

fn client_id_hash() -> H256 {
	CLIENT_ID.as_bytes().into()
}

fn platform() -> String {
	if cfg!(target_os = "macos") {
		"x86_64-apple-darwin".into()
	} else if cfg!(windows) {
		"x86_64-pc-windows-msvc".into()
	} else if cfg!(target_os = "linux") {
		format!("{}-unknown-linux-gnu", Target::arch())
	} else {
		version::platform()
	}
}

fn platform_id_hash() -> H256 {
	platform().as_bytes().into()
}

impl Updater {
	pub fn new(client: Weak<BlockChainClient>, sync: Weak<SyncProvider>, update_policy: UpdatePolicy, fetcher: fetch::Client) -> Arc<Self> {
		let r = Arc::new(Updater {
			update_policy: update_policy,
			weak_self: Mutex::new(Default::default()),
			client: client.clone(),
			sync: sync.clone(),
			fetcher,
			operations_contract: operations_contract::Operations::default(),
			exit_handler: Mutex::new(None),
			this: VersionInfo::this(),
			state: Mutex::new(Default::default()),
		});
		*r.weak_self.lock() = Arc::downgrade(&r);
		r.poll();
		r
	}

	/// Set a closure to call when we want to restart the client
	pub fn set_exit_handler<F>(&self, f: F) where F: Fn() + 'static + Send {
		*self.exit_handler.lock() = Some(Box::new(f));
	}

	fn collect_release_info<T: Fn(Vec<u8>) -> Result<Vec<u8>, String>>(&self, release_id: H256, do_call: &T) -> Result<ReleaseInfo, String> {
		let (fork, track, semver, is_critical) = self.operations_contract.functions()
			.release()
			.call(client_id_hash(), release_id, &do_call)
			.map_err(|e| format!("{:?}", e))?;

		let (fork, track, semver) = (fork.low_u64(), track.low_u32(), semver.low_u32());

		let latest_binary = self.operations_contract.functions()
			.checksum()
			.call(client_id_hash(), release_id, platform_id_hash(), &do_call)
			.map_err(|e| format!("{:?}", e))?;

		Ok(ReleaseInfo {
			version: VersionInfo::from_raw(semver, track as u8, release_id.into()),
			is_critical,
			fork,
			binary: if latest_binary.is_zero() { None } else { Some(latest_binary) },
		})
	}

	/// Returns release track of the parity node.
	/// `update_policy.track` is the track specified from the command line, whereas `this.track`
	/// is the track of the software which is currently run
	fn track(&self) -> ReleaseTrack {
		match self.update_policy.track {
			ReleaseTrack::Unknown => self.this.track,
			x => x,
		}
	}

	fn latest_in_track<T: Fn(Vec<u8>) -> Result<Vec<u8>, String>>(&self, track: ReleaseTrack, do_call: &T) -> Result<H256, String> {
		self.operations_contract.functions()
			.latest_in_track()
			.call(client_id_hash(), u8::from(track), do_call)
			.map_err(|e| format!("{:?}", e))
	}

	fn collect_latest(&self) -> Result<OperationsInfo, String> {
		if self.track() == ReleaseTrack::Unknown {
			return Err(format!("Current executable ({}) is unreleased.", self.this.hash));
		}

		let client = self.client.upgrade().ok_or_else(|| "Cannot obtain client")?;
		let address = client.registry_address("operations".into(), BlockId::Latest).ok_or_else(|| "Cannot get operations contract address")?;
		let do_call = |data| client.call_contract(BlockId::Latest, address, data).map_err(|e| format!("{:?}", e));

		trace!(target: "updater", "Looking up this_fork for our release: {}/{:?}", CLIENT_ID, self.this.hash);

		// get the fork number of this release
		let this_fork = self.operations_contract.functions()
			.release()
			.call(client_id_hash(), self.this.hash, &do_call)
			.ok()
			.and_then(|(fork, track, _, _)| {
				let this_track: ReleaseTrack = (track.low_u64() as u8).into();
				match this_track {
					ReleaseTrack::Unknown => None,
					_ => Some(fork.low_u64()),
				}
			});

		// get the hash of the latest release in our track
		let latest_in_track = self.latest_in_track(self.track(), &do_call)?;

		// get the release info for the latest version in track
		let in_track = self.collect_release_info(latest_in_track, &do_call)?;
		let mut in_minor = Some(in_track.clone());
		const PROOF: &'static str = "in_minor initialised and assigned with Some; loop breaks if None assigned; qed";

		// if the minor version has changed, let's check the minor version on a different track
		while in_minor.as_ref().expect(PROOF).version.version.minor != self.this.version.minor {
			let track = match in_minor.as_ref().expect(PROOF).version.track {
				ReleaseTrack::Beta => ReleaseTrack::Stable,
				ReleaseTrack::Nightly => ReleaseTrack::Beta,
				_ => { in_minor = None; break; }
			};

			let latest_in_track = self.latest_in_track(track, &do_call)?;
			in_minor = Some(self.collect_release_info(latest_in_track, &do_call)?);
		}

		let fork = self.operations_contract.functions()
			.latest_fork()
			.call(&do_call)
			.map_err(|e| format!("{:?}", e))?.low_u64();

		Ok(OperationsInfo {
			fork,
			this_fork,
			track: in_track,
			minor: in_minor,
		})
	}

	fn update_file_name(v: &VersionInfo) -> String {
		format!("parity-{}.{}.{}-{:?}", v.version.major, v.version.minor, v.version.patch, v.hash)
	}

	fn updates_path(&self, name: &str) -> PathBuf {
		let mut dest = PathBuf::from(self.update_policy.path.clone());
		dest.push(name);
		dest
	}

	fn fetch_done(&self, result: Result<PathBuf, fetch::Error>) {
		// old below
		(|| -> Result<(), (String, bool)> {
			let auto = {
				let mut s = self.state.lock();
				let fetched = s.fetching.take().unwrap();
				let dest = self.updates_path(&Self::update_file_name(&fetched.version));
				if !dest.exists() {
					let b = match result {
						Ok(b) => {
							s.backoff = None;
							b
						},
						Err(e) => {
							let mut n = s.backoff.map(|b| b.0 + 1).unwrap_or(1);
							s.backoff = Some((n, Instant::now() + Duration::from_secs(2usize.pow(n) as u64)));

							return Err((format!("Unable to fetch update ({}): {:?}", fetched.version, e), false));
						},
					};

					info!(target: "updater", "Fetched latest version ({}) OK to {}", fetched.version, b.display());
					fs::create_dir_all(dest.parent().expect("at least one thing pushed; qed")).map_err(|e| (format!("Unable to create updates path: {:?}", e), true))?;
					fs::copy(&b, &dest).map_err(|e| (format!("Unable to copy update: {:?}", e), true))?;
					restrict_permissions_owner(&dest, false, true).map_err(|e| (format!("Unable to update permissions: {}", e), true))?;
					info!(target: "updater", "Installed updated binary to {}", dest.display());
				}
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
		})().unwrap_or_else(|(e, fatal)| { self.state.lock().disabled = fatal; warn!("{}", e); });
	}

	fn poll(&self) {
		trace!(target: "updater", "Current release is {} ({:?})", self.this, self.this.hash);

		// We rely on a secure state. Bail if we're unsure about it.
		if self.client.upgrade().map_or(true, |s| !s.chain_info().security_level().is_full()) {
			return;
		}

		let current_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));

		let mut capability = CapState::Unknown;
		let latest = self.collect_latest().ok();
		if let Some(ref latest) = latest {
			trace!(target: "updater", "Latest release in our track is v{} it is {}critical ({} binary is {})",
				latest.track.version,
				if latest.track.is_critical {""} else {"non-"},
				&platform(),
				if let Some(ref b) = latest.track.binary {
					format!("{}", b)
				} else {
					"unreleased".into()
				}
			);
			let mut s = self.state.lock();
			let running_later = latest.track.version.version < self.version_info().version;
			let running_latest = latest.track.version.hash == self.version_info().hash;
			let already_have_latest = s.installed.as_ref().or(s.ready.as_ref()).map_or(false, |t| *t == latest.track);

			if !s.disabled && self.update_policy.enable_downloading && !running_later && !running_latest && !already_have_latest {
				if let Some(b) = latest.track.binary {
					if s.fetching.is_none() {
						if self.updates_path(&Self::update_file_name(&latest.track.version)).exists() {
							info!(target: "updater", "Already fetched binary.");
							s.fetching = Some(latest.track.clone());
							drop(s);
							self.fetch_done(Ok(PathBuf::new()));
						} else {
							if s.backoff.iter().all(|&(_, instant)| Instant::now() >= instant) {
								info!(target: "updater", "Attempting to get parity binary {}", b);
								s.fetching = Some(latest.track.clone());
								drop(s);
								let weak_self = self.weak_self.lock().clone();
								let f = move |r: Result<PathBuf, fetch::Error>| if let Some(this) = weak_self.upgrade() { this.fetch_done(r) };
								self.fetcher.fetch(b, Box::new(f));
							}
						}
					}
				}
			}
			trace!(target: "updater", "Fork: this/current/latest/latest-known: {}/#{}/#{}/#{}", match latest.this_fork { Some(f) => format!("#{}", f), None => "unknown".into(), }, current_number, latest.track.fork, latest.fork);

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

		if s.latest != latest {
			s.backoff = None;
		}

		s.latest = latest;
		s.capability = capability;
	}
}

impl ChainNotify for Updater {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, _enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		match (self.client.upgrade(), self.sync.upgrade()) {
			(Some(ref c), Some(ref s)) if !s.status().is_syncing(c.queue_info()) => self.poll(),
			_ => {},
		}
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
		let mut s = self.state.lock();
		let ready = match s.ready.take() {
			Some(ready) => ready,
			None => {
				warn!(target: "updater", "Execute upgrade called when no upgrade ready.");
				return false;
			}
		};

		let p = Self::update_file_name(&ready.version);
		let n = self.updates_path("latest");

		// TODO: creating then writing is a bit fragile. would be nice to make it atomic.
		if let Err(e) = fs::File::create(&n).and_then(|mut f| f.write_all(p.as_bytes())) {
			s.ready = Some(ready);
			warn!(target: "updater", "Unable to create soft-link for update {:?}", e);
			return false;
		}

		info!(target: "updater", "Completed upgrade to {}", &ready.version);
		s.installed = Some(ready);
		match *self.exit_handler.lock() {
			Some(ref h) => (*h)(),
			None => info!(target: "updater", "Update installed; ready for restart."),
		}

		true
	}

	fn version_info(&self) -> VersionInfo {
		self.this.clone()
	}

	fn info(&self) -> Option<OperationsInfo> {
		self.state.lock().latest.clone()
	}
}
