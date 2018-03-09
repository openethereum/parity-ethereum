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
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use rand::{self, Rng};
use target_info::Target;

use bytes::Bytes;
use ethcore::BlockNumber;
use ethcore::filter::Filter;
use ethcore::client::{BlockId, BlockChainClient, ChainNotify};
use ethereum_types::H256;
use ethsync::{SyncProvider};
use hash::keccak;
use hash_fetch::{self as fetch, HashFetch};
use path::restrict_permissions_owner;
use service::Service;
use types::{ReleaseInfo, OperationsInfo, CapState, VersionInfo, ReleaseTrack};
use version;

use_contract!(operations_contract, "Operations", "res/operations.json");

const RELEASE_ADDED_EVENT_NAME: &'static [u8] = &*b"ReleaseAdded(bytes32,uint32,bytes32,uint8,uint24,bool)";
lazy_static! {
	static ref RELEASE_ADDED_EVENT_NAME_HASH: H256 = keccak(RELEASE_ADDED_EVENT_NAME);
}

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
	/// Random update delay range in blocks.
	pub max_delay: u64,
}

impl Default for UpdatePolicy {
	fn default() -> Self {
		UpdatePolicy {
			enable_downloading: false,
			require_consensus: true,
			filter: UpdateFilter::None,
			track: ReleaseTrack::Unknown,
			path: Default::default(),
			max_delay: 100,
		}
	}
}

/// The current updater status
#[derive(Clone, Debug)]
enum UpdaterStatus {
	/// Updater is currently disabled.
	Disabled,
	/// Updater is currently idle.
	Idle,
	/// Updater is waiting for block number to fetch a new release.
	Waiting {
		release: ReleaseInfo,
		binary: H256,
		block_number: BlockNumber,
	},
	/// Updater is fetching a new release.
	Fetching {
		release: ReleaseInfo,
		binary: H256,
		backoff: Option<(u32, Instant)>,
	},
	/// Updater is ready to update to a new release.
	Ready {
		release: ReleaseInfo,
	},
	/// Updater has installed a new release and can be manually restarted.
	Installed {
		release: ReleaseInfo,
	},
}

impl Default for UpdaterStatus {
	fn default() -> Self {
		UpdaterStatus::Idle
	}
}

#[derive(Debug, Default)]
struct UpdaterState {
	latest: Option<OperationsInfo>,
	capability: CapState,
	status: UpdaterStatus,
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

lazy_static! {
	static ref CLIENT_ID_HASH: H256 = CLIENT_ID.as_bytes().into();
}

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

	fn release_block_number(&self, from: BlockNumber, release: &ReleaseInfo) -> Option<BlockNumber> {
		let client = self.client.upgrade()?;
		let address = client.registry_address("operations".into(), BlockId::Latest)?;

		let filter = Filter {
			from_block: BlockId::Number(from),
			to_block: BlockId::Latest,
			address: Some(vec![address]),
			topics: vec![
				Some(vec![*RELEASE_ADDED_EVENT_NAME_HASH]),
				Some(vec![*CLIENT_ID_HASH]),
				Some(vec![release.fork.into()]),
				Some(vec![if release.is_critical { 1 } else { 0 }.into()]),
			],
			limit: None,
		};

		let event = self.operations_contract.events().release_added();

		client.logs(filter)
			.iter()
			.filter_map(|log| {
				let event = event.parse_log((log.topics.clone(), log.data.clone()).into()).ok()?;
				let version_info = VersionInfo::from_raw(event.semver.low_u32(), event.track.low_u32() as u8, event.release.into());
				if version_info == release.version {
					Some(log.block_number)
				} else {
					None
				}
			})
			.last()
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

	fn updater_step(&self) {
		let current_block_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));

		let mut state = self.state.lock();

		if let Some(latest) = state.latest.clone() {
			let fetch = |binary| {
				info!(target: "updater", "Attempting to get parity binary {}", binary);
				let weak_self = self.weak_self.lock().clone();
				let latest = latest.clone();
				let on_fetch = move |res: Result<PathBuf, fetch::Error>| {
					if let Some(this) = weak_self.upgrade() {
						let mut state = this.state.lock();

						// Check if the latest release and updater status hasn't changed
						if state.latest.as_ref() == Some(&latest) {
							if let UpdaterStatus::Fetching { ref release, backoff, binary } = state.status.clone() {
								match res {
									// We've successfully fetched the binary
									Ok(path) => {
										let setup = |path: &Path| -> Result<(), String> {
											let dest = this.updates_path(&Self::update_file_name(&release.version));
											if !dest.exists() {
												info!(target: "updater", "Fetched latest version ({}) OK to {}", release.version, path.display());
												fs::create_dir_all(dest.parent().expect("at least one thing pushed; qed")).map_err(|e| format!("Unable to create updates path: {:?}", e))?;
												fs::copy(path, &dest).map_err(|e| format!("Unable to copy update: {:?}", e))?;
												restrict_permissions_owner(&dest, false, true).map_err(|e| format!("Unable to update permissions: {}", e))?;
												info!(target: "updater", "Installed updated binary to {}", dest.display());
											}

											Ok(())
										};

										// There was a fatal error setting up the update, disable the updater
										if let Err(err) = setup(&path) {
											state.status = UpdaterStatus::Disabled;
											warn!("{}", err);
										} else {
											state.status = UpdaterStatus::Ready { release: release.clone() };
											this.updater_step();
										}
									},
									// There was an error fetching the update, apply a backoff delay before retrying
									Err(err) => {
										let n = backoff.map(|b| b.0 + 1).unwrap_or(1);
										let delay = 2usize.pow(n) as u64;
										let backoff = Some((n, Instant::now() + Duration::from_secs(delay)));

										state.status = UpdaterStatus::Fetching { release: release.clone(), backoff, binary };

										warn!("Unable to fetch update ({}): {:?}, retrying in {} seconds.", release.version, err, delay);
									},
								}
							}
						}
					}
				};

				self.fetcher.fetch(binary, Box::new(on_fetch));
			};

			match state.status.clone() {
				// updater is disabled
				UpdaterStatus::Disabled => {},
				// the update has already been installed
				UpdaterStatus::Installed { ref release, .. } if *release == latest.track => {},
				// we're currently fetching this update
				UpdaterStatus::Fetching { ref release, backoff: None, .. } if *release == latest.track => {},
				// we're delaying the update until the given block number
				UpdaterStatus::Waiting { ref release, block_number, .. } if *release == latest.track && current_block_number < block_number => {},
				// we're at (or past) the block that triggers the update, let's fetch the binary
				UpdaterStatus::Waiting { ref release, block_number, binary } if *release == latest.track && current_block_number >= block_number => {
					state.status = UpdaterStatus::Fetching { release: latest.track.clone(), binary, backoff: None };
					fetch(binary);
				},
				// we're ready to retry the fetch after we applied a backoff for the previous failure
				UpdaterStatus::Fetching { ref release, backoff: Some(backoff), binary } if *release == latest.track && Instant::now() > backoff.1 => {
					fetch(binary);
				}
				UpdaterStatus::Ready { ref release } if *release == latest.track => {
					let auto = match self.update_policy.filter {
						UpdateFilter::All => true,
						UpdateFilter::Critical if release.is_critical /* TODO: or is on a bad fork */ => true,
						_ => false,
					};

					if auto {
						// will lock self.state
						drop(state);
						self.execute_upgrade();
					}
				},
				_ => {
					if let Some(binary) = latest.track.binary {
						let running_later = latest.track.version.version < self.version_info().version;
						let running_latest = latest.track.version.hash == self.version_info().hash;

						// Check if we're already running the latest version or a newer version
						if !running_later && !running_latest {
							let path = self.updates_path(&Self::update_file_name(&latest.track.version));
							if path.exists() {
								info!(target: "updater", "Already fetched binary.");
								state.status = UpdaterStatus::Ready { release: latest.track.clone() };
								self.updater_step();

							} else if self.update_policy.enable_downloading {
								match self.release_block_number(current_block_number - self.update_policy.max_delay, &latest.track) {
									Some(block_number) => {
										let delay = rand::thread_rng().gen_range(0, self.update_policy.max_delay);
										let update_block_number = block_number + delay;

										info!(target: "updater", "Update for binary {} will be triggered at block {}", binary, update_block_number);

										state.status = UpdaterStatus::Waiting { release: latest.track.clone(), binary, block_number: update_block_number };
									},
									None => {
										state.status = UpdaterStatus::Waiting { release: latest.track.clone(), binary, block_number: current_block_number };
										self.updater_step();
									},
								}
							}
						}
					}
				},
			}
		}
	}

	fn poll(&self) {
		trace!(target: "updater", "Current release is {} ({:?})", self.this, self.this.hash);

		// We rely on a secure state. Bail if we're unsure about it.
		if self.client.upgrade().map_or(true, |s| !s.chain_info().security_level().is_full()) {
			return;
		}

		let mut state = self.state.lock();

		// Get the latest available release
		let latest = self.collect_latest().ok();

		if let Some(latest) = latest {
			// There's a new release available
			if state.latest.as_ref() != Some(&latest) {
				let current_block_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));

				trace!(target: "updater", "Latest release in our track is v{} it is {}critical ({} binary is {})",
					   latest.track.version,
					   if latest.track.is_critical {""} else {"non-"},
					   &platform(),
					   latest.track.binary.map(|b| format!("{}", b)).unwrap_or("unreleased".into()));

				trace!(target: "updater", "Fork: this/current/latest/latest-known: {}/#{}/#{}/#{}",
					   latest.this_fork.map(|f| format!("#{}", f)).unwrap_or("unknown".into()),
					   current_block_number,
					   latest.track.fork,
					   latest.fork);

				// Update current capability
				state.capability = match latest.this_fork {
					// We're behind the latest fork. Now is the time to be upgrading, perhaps we're too late...
					Some(this_fork) if this_fork < latest.fork => {
						if current_block_number >= latest.fork - 1 {
							// We're at (or past) the last block we can import. Disable the client.
							if self.update_policy.require_consensus {
								if let Some(c) = self.client.upgrade() {
									c.disable();
								}
							}

							CapState::IncapableSince(latest.fork)
						} else {
							CapState::CapableUntil(latest.fork)
						}
					},
					Some(_) => CapState::Capable,
					None => CapState::Unknown,
				};

				// Update latest release
				state.latest = Some(latest.clone());

			}
		}

		// will lock self.state
		drop(state);
		self.updater_step();
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
		match self.state.lock().status {
			UpdaterStatus::Ready { ref release, .. } => Some(release.clone()),
			_ => None,
		}
	}

	fn execute_upgrade(&self) -> bool {
		let mut state = self.state.lock();

		match state.status.clone() {
			UpdaterStatus::Ready { ref release } => {
				let file = Self::update_file_name(&release.version);
				let path = self.updates_path("latest");

				// TODO: creating then writing is a bit fragile. would be nice to make it atomic.
				if let Err(err) = fs::File::create(&path).and_then(|mut f| f.write_all(file.as_bytes())) {
					state.status = UpdaterStatus::Disabled;

					warn!(target: "updater", "Unable to create soft-link for update {:?}", err);
					return false;
				}

				info!(target: "updater", "Completed upgrade to {}", &release.version);
				state.status = UpdaterStatus::Installed { release: release.clone() };

				match *self.exit_handler.lock() {
					Some(ref h) => (*h)(),
					None => info!(target: "updater", "Update installed; ready for restart."),
				}

				true
			},
			_ => {
				warn!(target: "updater", "Execute upgrade called when no upgrade ready.");
				false
			},
		}
	}

	fn version_info(&self) -> VersionInfo {
		self.this.clone()
	}

	fn info(&self) -> Option<OperationsInfo> {
		self.state.lock().latest.clone()
	}
}
