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

use std::cmp;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use parking_lot::{Mutex, MutexGuard};
use rand::{self, Rng};
use target_info::Target;

use bytes::Bytes;
use ethcore::BlockNumber;
use ethcore::filter::Filter;
use ethcore::client::{BlockId, BlockChainClient, ChainNotify};
use ethereum_types::H256;
use ethsync::{SyncProvider};
use hash_fetch::{self as fetch, HashFetch};
use path::restrict_permissions_owner;
use service::Service;
use types::{ReleaseInfo, OperationsInfo, CapState, VersionInfo, ReleaseTrack};
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
	pub path: PathBuf,
	/// Maximum download size.
	pub max_size: usize,
	/// Random update delay range in blocks.
	pub max_delay: u64,
	/// Number of blocks between each check for updates.
	pub frequency: u64,
}

impl Default for UpdatePolicy {
	fn default() -> Self {
		UpdatePolicy {
			enable_downloading: false,
			require_consensus: true,
			filter: UpdateFilter::None,
			track: ReleaseTrack::Unknown,
			path: Default::default(),
			max_size: 128 * 1024 * 1024,
			max_delay: 100,
			frequency: 20,
		}
	}
}

/// The current updater status
#[derive(Clone, Debug, PartialEq)]
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
		retries: u32,
	},
	/// Updater failed fetching a new release and it is now backing off until the next retry.
	FetchBackoff {
		release: ReleaseInfo,
		binary: H256,
		backoff: (u32, Instant),
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
pub struct Updater<O = OperationsContractClient, F = fetch::Client, T = StdTimeProvider, R = ThreadRngGenRange> {
	// Useful environmental stuff.
	update_policy: UpdatePolicy,
	weak_self: Mutex<Weak<Updater<O, F, T, R>>>,
	client: Weak<BlockChainClient>,
	sync: Option<Weak<SyncProvider>>,
	fetcher: F,
	operations_client: O,
	exit_handler: Mutex<Option<Box<Fn() + 'static + Send>>>,

	time_provider: T,
	rng: R,

	// Our version info (static)
	this: VersionInfo,

	// All the other info - this changes so leave it behind a Mutex.
	state: Mutex<UpdaterState>,
}

const CLIENT_ID: &'static str = "parity";

lazy_static! {
	static ref CLIENT_ID_HASH: H256 = CLIENT_ID.as_bytes().into();
}

lazy_static! {
	static ref PLATFORM: String = {
		if cfg!(target_os = "macos") {
			"x86_64-apple-darwin".into()
		} else if cfg!(windows) {
			"x86_64-pc-windows-msvc".into()
		} else if cfg!(target_os = "linux") {
			format!("{}-unknown-linux-gnu", Target::arch())
		} else {
			version::platform()
		}
	};
}

lazy_static! {
	static ref PLATFORM_ID_HASH: H256 = PLATFORM.as_bytes().into();
}

/// Client trait for getting latest release information from operations contract.
/// Useful for mocking in tests.
pub trait OperationsClient: Send + Sync + 'static {
	/// Get the latest release operations info for the given track.
	fn latest(&self, this: &VersionInfo, track: ReleaseTrack) -> Result<OperationsInfo, String>;

	/// Fetches the block number when the given release was added, checking the interval [from; latest_block].
	fn release_block_number(&self, from: BlockNumber, release: &ReleaseInfo) -> Option<BlockNumber>;
}

/// OperationsClient that delegates calls to the operations contract.
pub struct OperationsContractClient {
	operations_contract: operations_contract::Operations,
	client: Weak<BlockChainClient>,
}

impl OperationsContractClient {
	fn new(
		operations_contract: operations_contract::Operations,
		client: Weak<BlockChainClient>,
	) -> OperationsContractClient {
		OperationsContractClient { operations_contract, client }
	}

	/// Get the hash of the latest release for the given track
	fn latest_hash<F>(&self, track: ReleaseTrack, do_call: &F) -> Result<H256, String>
	where F: Fn(Vec<u8>) -> Result<Vec<u8>, String> {
		self.operations_contract.functions()
			.latest_in_track()
			.call(*CLIENT_ID_HASH, u8::from(track), do_call)
			.map_err(|e| format!("{:?}", e))
	}

	/// Get release info for the given release
	fn release_info<F>(&self, release_id: H256, do_call: &F) -> Result<ReleaseInfo, String>
	where F: Fn(Vec<u8>) -> Result<Vec<u8>, String> {
		let (fork, track, semver, is_critical) = self.operations_contract.functions()
			.release()
			.call(*CLIENT_ID_HASH, release_id, &do_call)
			.map_err(|e| format!("{:?}", e))?;

		let (fork, track, semver) = (fork.low_u64(), track.low_u32(), semver.low_u32());

		let latest_binary = self.operations_contract.functions()
			.checksum()
			.call(*CLIENT_ID_HASH, release_id, *PLATFORM_ID_HASH, &do_call)
			.map_err(|e| format!("{:?}", e))?;

		Ok(ReleaseInfo {
			version: VersionInfo::from_raw(semver, track as u8, release_id.into()),
			is_critical,
			fork,
			binary: if latest_binary.is_zero() { None } else { Some(latest_binary) },
		})
	}
}

impl OperationsClient for OperationsContractClient {
	fn latest(&self, this: &VersionInfo, track: ReleaseTrack) -> Result<OperationsInfo, String> {
		if track == ReleaseTrack::Unknown {
			return Err(format!("Current executable ({}) is unreleased.", this.hash));
		}

		let client = self.client.upgrade().ok_or_else(|| "Cannot obtain client")?;
		let address = client.registry_address("operations".into(), BlockId::Latest).ok_or_else(|| "Cannot get operations contract address")?;
		let do_call = |data| client.call_contract(BlockId::Latest, address, data).map_err(|e| format!("{:?}", e));

		trace!(target: "updater", "Looking up this_fork for our release: {}/{:?}", CLIENT_ID, this.hash);

		// get the fork number of this release
		let this_fork = self.operations_contract.functions()
			.release()
			.call(*CLIENT_ID_HASH, this.hash, &do_call)
			.ok()
			.and_then(|(fork, track, _, _)| {
				let this_track: ReleaseTrack = (track.low_u64() as u8).into();
				match this_track {
					ReleaseTrack::Unknown => None,
					_ => Some(fork.low_u64()),
				}
			});

		// get the hash of the latest release in our track
		let latest_in_track = self.latest_hash(track, &do_call)?;

		// get the release info for the latest version in track
		let in_track = self.release_info(latest_in_track, &do_call)?;
		let mut in_minor = Some(in_track.clone());
		const PROOF: &'static str = "in_minor initialised and assigned with Some; loop breaks if None assigned; qed";

		// if the minor version has changed, let's check the minor version on a different track
		while in_minor.as_ref().expect(PROOF).version.version.minor != this.version.minor {
			let track = match in_minor.as_ref().expect(PROOF).version.track {
				ReleaseTrack::Beta => ReleaseTrack::Stable,
				ReleaseTrack::Nightly => ReleaseTrack::Beta,
				_ => { in_minor = None; break; }
			};

			let latest_in_track = self.latest_hash(track, &do_call)?;
			in_minor = Some(self.release_info(latest_in_track, &do_call)?);
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

	fn release_block_number(&self, from: BlockNumber, release: &ReleaseInfo) -> Option<BlockNumber> {
		let client = self.client.upgrade()?;
		let address = client.registry_address("operations".into(), BlockId::Latest)?;

		let event = self.operations_contract.events().release_added();

		let topics = event.create_filter(Some(*CLIENT_ID_HASH), Some(release.fork.into()), Some(release.is_critical));
		let topics = vec![topics.topic0, topics.topic1, topics.topic2, topics.topic3];
		let topics = topics.into_iter().map(Into::into).map(Some).collect();

		let filter = Filter {
			from_block: BlockId::Number(from),
			to_block: BlockId::Latest,
			address: Some(vec![address]),
			topics: topics,
			limit: None,
		};

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
}

/// Trait to provide current time. Useful for mocking in tests.
pub trait TimeProvider: Send + Sync + 'static {
	/// Returns an instant corresponding to "now".
	fn now(&self) -> Instant;
}

/// TimeProvider implementation that delegates calls to std::time.
pub struct StdTimeProvider;

impl TimeProvider for StdTimeProvider {
	fn now(&self) -> Instant {
		Instant::now()
	}
}

/// Trait to generate a random number within a given range.
/// Useful for mocking in tests.
pub trait GenRange: Send + Sync + 'static {
	/// Generate a random value in the range [low, high), i.e. inclusive of low and exclusive of high.
	fn gen_range(&self, low: u64, high: u64) -> u64;
}

/// GenRange implementation that uses a rand::thread_rng for randomness.
pub struct ThreadRngGenRange;

impl GenRange for ThreadRngGenRange {
	fn gen_range(&self, low: u64, high: u64) -> u64 {
		rand::thread_rng().gen_range(low, high)
	}
}

impl Updater {
	pub fn new(
		client: Weak<BlockChainClient>,
		sync: Weak<SyncProvider>,
		update_policy: UpdatePolicy,
		fetcher: fetch::Client,
	) -> Arc<Updater> {
		let r = Arc::new(Updater {
			update_policy: update_policy,
			weak_self: Mutex::new(Default::default()),
			client: client.clone(),
			sync: Some(sync.clone()),
			fetcher,
			operations_client: OperationsContractClient::new(
				operations_contract::Operations::default(),
				client.clone()),
			exit_handler: Mutex::new(None),
			this: VersionInfo::this(),
			time_provider: StdTimeProvider,
			rng: ThreadRngGenRange,
			state: Mutex::new(Default::default()),
		});
		*r.weak_self.lock() = Arc::downgrade(&r);
		r.poll();
		r
	}

	fn update_file_name(v: &VersionInfo) -> String {
		format!("parity-{}.{}.{}-{:x}", v.version.major, v.version.minor, v.version.patch, v.hash)
	}
}

impl<O: OperationsClient, F: HashFetch, T: TimeProvider, R: GenRange> Updater<O, F, T, R> {
	/// Set a closure to call when we want to restart the client
	pub fn set_exit_handler<G>(&self, g: G) where G: Fn() + 'static + Send {
		*self.exit_handler.lock() = Some(Box::new(g));
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

	fn updates_path(&self, name: &str) -> PathBuf {
		self.update_policy.path.join(name)
	}

	fn on_fetch(&self, latest: &OperationsInfo, res: Result<PathBuf, fetch::Error>) {
		let mut state = self.state.lock();

		// Bail out if the latest release has changed in the meantime
		if state.latest.as_ref() != Some(&latest) {
			return;
		}

		// The updated status should be set to fetching
		if let UpdaterStatus::Fetching { ref release, binary, retries } = state.status.clone() {
			match res {
				// We've successfully fetched the binary
				Ok(path) => {
					let setup = |path: &Path| -> Result<(), String> {
						let dest = self.updates_path(&Updater::update_file_name(&release.version));
						if !dest.exists() {
							info!(target: "updater", "Fetched latest version ({}) OK to {}", release.version, path.display());
							fs::create_dir_all(dest.parent().expect("at least one thing pushed; qed")).map_err(|e| format!("Unable to create updates path: {:?}", e))?;
							fs::copy(path, &dest).map_err(|e| format!("Unable to copy update: {:?}", e))?;
							restrict_permissions_owner(&dest, false, true).map_err(|e| format!("Unable to update permissions: {}", e))?;
							info!(target: "updater", "Copied updated binary to {}", dest.display());
						}

						Ok(())
					};

					// There was a fatal error setting up the update, disable the updater
					if let Err(err) = setup(&path) {
						state.status = UpdaterStatus::Disabled;
						warn!("{}", err);
					} else {
						state.status = UpdaterStatus::Ready { release: release.clone() };
						self.updater_step(state);
					}
				},
				// There was an error fetching the update, apply a backoff delay before retrying
				Err(err) => {
					let delay = 2usize.pow(retries) as u64;
					// cap maximum backoff to 1 day
					let delay = cmp::min(delay, 24 * 60 * 60);
					let backoff = (retries, self.time_provider.now() + Duration::from_secs(delay));

					state.status = UpdaterStatus::FetchBackoff { release: release.clone(), backoff, binary };

					warn!("Unable to fetch update ({}): {:?}, retrying in {} seconds.", release.version, err, delay);
				},
			}
		}
	}

	fn execute_upgrade(&self, mut state: MutexGuard<UpdaterState>) -> bool {
		if let UpdaterStatus::Ready { ref release } = state.status.clone() {
			let file = Updater::update_file_name(&release.version);
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
				None => info!(target: "updater", "Update installed, ready for restart."),
			}

			return true;
		};

		warn!(target: "updater", "Execute upgrade called when no upgrade ready.");
		false
	}

	fn updater_step(&self, mut state: MutexGuard<UpdaterState>) {
		let current_block_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));

		if let Some(latest) = state.latest.clone() {
			let fetch = |latest, binary| {
				info!(target: "updater", "Attempting to get parity binary {}", binary);
				let weak_self = self.weak_self.lock().clone();
				let f = move |res: Result<PathBuf, fetch::Error>| {
					if let Some(this) = weak_self.upgrade() {
						this.on_fetch(&latest, res)
					}
				};

				self.fetcher.fetch(
					binary,
					fetch::Abort::default().with_max_size(self.update_policy.max_size),
					Box::new(f));
			};

			match state.status.clone() {
				// updater is disabled
				UpdaterStatus::Disabled => {},
				// the update has already been installed
				UpdaterStatus::Installed { ref release, .. } if *release == latest.track => {},
				// we're currently fetching this update
				UpdaterStatus::Fetching { ref release, .. } if *release == latest.track => {},
				// the fetch has failed and we're backing off the next retry
				UpdaterStatus::FetchBackoff { ref release, backoff, .. } if *release == latest.track && self.time_provider.now() < backoff.1 => {},
				// we're delaying the update until the given block number
				UpdaterStatus::Waiting { ref release, block_number, .. } if *release == latest.track && current_block_number < block_number => {},
				// we're at (or past) the block that triggers the update, let's fetch the binary
				UpdaterStatus::Waiting { ref release, block_number, binary } if *release == latest.track && current_block_number >= block_number => {
					info!(target: "updater", "Update for binary {} triggered", binary);

					state.status = UpdaterStatus::Fetching { release: release.clone(), binary, retries: 1 };
					fetch(latest, binary);
				},
				// we're ready to retry the fetch after we applied a backoff for the previous failure
				UpdaterStatus::FetchBackoff { ref release, backoff, binary } if *release == latest.track && self.time_provider.now() >= backoff.1 => {
					state.status = UpdaterStatus::Fetching { release: release.clone(), binary, retries: backoff.0 + 1 };
					fetch(latest, binary);
				},
				// the update is ready to be installed
				UpdaterStatus::Ready { ref release } if *release == latest.track => {
					let auto = match self.update_policy.filter {
						UpdateFilter::All => true,
						UpdateFilter::Critical if release.is_critical /* TODO: or is on a bad fork */ => true,
						_ => false,
					};

					if auto {
						self.execute_upgrade(state);
					}
				},
				// this is the default case that does the initial triggering to update. we can reach this case by being
				// `Idle` but also if the latest release is updated, regardless of the state we're in (except if the
				// updater is in the `Disabled` state). if we push a bad update (e.g. wrong hashes or download url)
				// clients might eventually be on a really long backoff state for that release, but as soon a new
				// release is pushed we'll fall through to the default case.
				_ => {
					if let Some(binary) = latest.track.binary {
						let running_later = latest.track.version.version < self.version_info().version;
						let running_latest = latest.track.version.hash == self.version_info().hash;

						// Bail out if we're already running the latest version or a later one
						if running_later || running_latest {
							return;
						}

						let path = self.updates_path(&Updater::update_file_name(&latest.track.version));
						if path.exists() {
							info!(target: "updater", "Already fetched binary.");
							state.status = UpdaterStatus::Ready { release: latest.track.clone() };
							self.updater_step(state);

						} else if self.update_policy.enable_downloading {
							let update_block_number = {
								let max_delay = if latest.fork >= current_block_number {
									cmp::min(latest.fork - current_block_number, self.update_policy.max_delay)
								} else {
									self.update_policy.max_delay
								};

								let from = current_block_number.saturating_sub(max_delay);
								match self.operations_client.release_block_number(from, &latest.track) {
									Some(block_number) => {
										let delay = self.rng.gen_range(0, max_delay);
										block_number.saturating_add(delay)
									},
									None => current_block_number,
								}
							};

							state.status = UpdaterStatus::Waiting { release: latest.track.clone(), binary, block_number: update_block_number };

							if update_block_number > current_block_number {
								info!(target: "updater", "Update for binary {} will be triggered at block {}", binary, update_block_number);
							} else {
								self.updater_step(state);
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
		if self.client.upgrade().map_or(true, |c| !c.chain_info().security_level().is_full()) {
			return;
		}

		// Only check for updates every n blocks
		let current_block_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));
		if current_block_number % cmp::max(self.update_policy.frequency, 1) != 0 {
			return;
		}

		let mut state = self.state.lock();

		// Get the latest available release
		let latest = self.operations_client.latest(&self.this, self.track()).ok();

		if let Some(latest) = latest {
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

			// There's a new release available
			if state.latest.as_ref() != Some(&latest) {
				trace!(target: "updater", "Latest release in our track is v{} it is {}critical ({} binary is {})",
					   latest.track.version,
					   if latest.track.is_critical {""} else {"non-"},
					   *PLATFORM,
					   latest.track.binary.map(|b| format!("{}", b)).unwrap_or("unreleased".into()));

				trace!(target: "updater", "Fork: this/current/latest/latest-known: {}/#{}/#{}/#{}",
					   latest.this_fork.map(|f| format!("#{}", f)).unwrap_or("unknown".into()),
					   current_block_number,
					   latest.track.fork,
					   latest.fork);

				// Update latest release
				state.latest = Some(latest);
			}
		}

		self.updater_step(state);
	}
}

impl ChainNotify for Updater {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, _enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		match (self.client.upgrade(), self.sync.as_ref().and_then(Weak::upgrade)) {
			(Some(ref c), Some(ref s)) if !s.status().is_syncing(c.queue_info()) => self.poll(),
			_ => {},
		}
	}
}

impl<O: OperationsClient, F: HashFetch, T: TimeProvider, R: GenRange> Service for Updater<O, F, T, R> {
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
		let state = self.state.lock();
		self.execute_upgrade(state)
	}

	fn version_info(&self) -> VersionInfo {
		self.this.clone()
	}

	fn info(&self) -> Option<OperationsInfo> {
		self.state.lock().latest.clone()
	}
}

#[cfg(test)]
pub mod tests {
	use std::fs::File;
	use std::io::Read;
	use std::sync::Arc;
	use semver::Version;
	use tempdir::TempDir;
	use ethcore::client::{TestBlockChainClient, EachBlockWith};
	use self::fetch::Error;
	use super::*;

	#[derive(Clone)]
	struct FakeOperationsClient {
		result: Arc<Mutex<(Option<OperationsInfo>, Option<BlockNumber>)>>,
	}

	impl FakeOperationsClient {
		fn new() -> FakeOperationsClient {
			FakeOperationsClient { result: Arc::new(Mutex::new((None, None))) }
		}

		fn set_result(&self, operations_info: Option<OperationsInfo>, release_block_number: Option<BlockNumber>) {
			let mut result = self.result.lock();
			result.0 = operations_info;
			result.1 = release_block_number;
		}
	}

	impl OperationsClient for FakeOperationsClient {
		fn latest(&self, _this: &VersionInfo, _track: ReleaseTrack) -> Result<OperationsInfo, String> {
			self.result.lock().0.clone().ok_or("unavailable".into())
		}

		fn release_block_number(&self, _from: BlockNumber, _release: &ReleaseInfo) -> Option<BlockNumber> {
			self.result.lock().1.clone()
		}
	}

	#[derive(Clone)]
	struct FakeFetch {
		on_done: Arc<Mutex<Option<Box<Fn(Result<PathBuf, Error>) + Send>>>>,
	}

	impl FakeFetch {
		fn new() -> FakeFetch {
			FakeFetch { on_done: Arc::new(Mutex::new(None)) }
		}

		fn trigger(&self, result: Option<PathBuf>) {
			if let Some(ref on_done) = *self.on_done.lock() {
				on_done(result.ok_or(Error::NoResolution))
			}
		}
	}

	impl HashFetch for FakeFetch {
		fn fetch(&self, _hash: H256, _abort: fetch::Abort, on_done: Box<Fn(Result<PathBuf, Error>) + Send>) {
			*self.on_done.lock() = Some(on_done);
		}
	}

	#[derive(Clone)]
	struct FakeTimeProvider {
		result: Arc<Mutex<Instant>>,
	}

	impl FakeTimeProvider {
		fn new() -> FakeTimeProvider {
			FakeTimeProvider { result: Arc::new(Mutex::new(Instant::now())) }
		}

		fn set_result(&self, result: Instant) {
			*self.result.lock() = result;
		}
	}

	impl TimeProvider for FakeTimeProvider {
		fn now(&self) -> Instant {
			*self.result.lock()
		}
	}

	#[derive(Clone)]
	struct FakeGenRange {
		result: Arc<Mutex<u64>>,
	}

	impl FakeGenRange {
		fn new() -> FakeGenRange {
			FakeGenRange { result: Arc::new(Mutex::new(0)) }
		}

		fn set_result(&self, result: u64) {
			*self.result.lock() = result;
		}
	}

	impl GenRange for FakeGenRange {
		fn gen_range(&self, _low: u64, _high: u64) -> u64 {
			*self.result.lock()
		}
	}

	type TestUpdater = Updater<FakeOperationsClient, FakeFetch, FakeTimeProvider, FakeGenRange>;

	fn setup(update_policy: UpdatePolicy) -> (
		Arc<TestBlockChainClient>,
		Arc<TestUpdater>,
		FakeOperationsClient,
		FakeFetch,
		FakeTimeProvider,
		FakeGenRange) {

		let client = Arc::new(TestBlockChainClient::new());
		let weak_client = Arc::downgrade(&client);

		let operations_client = FakeOperationsClient::new();
		let fetcher = FakeFetch::new();
		let time_provider = FakeTimeProvider::new();
		let rng = FakeGenRange::new();

		let this = VersionInfo {
			track: ReleaseTrack::Beta,
			version: Version::parse("1.0.0").unwrap(),
			hash: 0.into(),
		};

		let updater = Arc::new(Updater {
			update_policy: update_policy,
			weak_self: Mutex::new(Default::default()),
			client: weak_client,
			sync: None,
			fetcher: fetcher.clone(),
			operations_client: operations_client.clone(),
			exit_handler: Mutex::new(None),
			this: this,
			time_provider: time_provider.clone(),
			rng: rng.clone(),
			state: Mutex::new(Default::default()),
		});

		*updater.weak_self.lock() = Arc::downgrade(&updater);

		(client, updater, operations_client, fetcher, time_provider, rng)
	}

	fn update_policy() -> (UpdatePolicy, TempDir) {
		let tempdir = TempDir::new("").unwrap();

		let update_policy = UpdatePolicy {
			path: tempdir.path().into(),
			enable_downloading: true,
			max_delay: 10,
			frequency: 1,
			..Default::default()
		};

		(update_policy, tempdir)
	}

	fn new_upgrade(version: &str) -> (VersionInfo, ReleaseInfo, OperationsInfo) {
		let latest_version = VersionInfo {
			track: ReleaseTrack::Beta,
			version: Version::parse(version).unwrap(),
			hash: 1.into(),
		};

		let latest_release = ReleaseInfo {
			version: latest_version.clone(),
			is_critical: false,
			fork: 0,
			binary: Some(0.into()),
		};

		let latest = OperationsInfo {
			fork: 0,
			this_fork: Some(0),
			track: latest_release.clone(),
			minor: None,
		};

		(latest_version, latest_release, latest)
	}

	#[test]
	fn should_stay_idle_when_no_release() {
		let (update_policy, _) = update_policy();
		let (_client, updater, _, _, ..) = setup(update_policy);

		assert_eq!(updater.state.lock().status, UpdaterStatus::Idle);
		updater.poll();
		assert_eq!(updater.state.lock().status, UpdaterStatus::Idle);
	}

	#[test]
	fn should_update_on_new_release() {
		let (update_policy, tempdir) = update_policy();
		let (_client, updater, operations_client, fetcher, ..) = setup(update_policy);
		let (latest_version, latest_release, latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		// we start in idle state and with no information regarding the latest release
		assert_eq!(updater.state.lock().latest, None);
		assert_eq!(updater.state.lock().status, UpdaterStatus::Idle);

		updater.poll();

		// after the first poll the latest release should be set to the one we're mocking and the updater should be
		// fetching it
		assert_eq!(updater.state.lock().latest, Some(latest));
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Fetching { ref release, retries, .. } if *release == latest_release && retries == 1);

		// mock fetcher with update binary and trigger the fetch
		let update_file = tempdir.path().join("parity");
		File::create(update_file.clone()).unwrap();
		fetcher.trigger(Some(update_file));

		// after the fetch finishes the upgrade should be ready to install
		assert_eq!(updater.state.lock().status, UpdaterStatus::Ready { release: latest_release.clone() });
		assert_eq!(updater.upgrade_ready(), Some(latest_release.clone()));

		// the current update_policy doesn't allow updating automatically, but we can trigger the update manually
		<TestUpdater as Service>::execute_upgrade(&*updater);

		assert_eq!(updater.state.lock().status, UpdaterStatus::Installed { release: latest_release });

		// the final binary should exist in the updates folder and the 'latest' file should be updated to point to it
		let updated_binary = tempdir.path().join(Updater::update_file_name(&latest_version));
		let latest_file = tempdir.path().join("latest");

		assert!(updated_binary.exists());
		assert!(latest_file.exists());

		let mut latest_file_content = String::new();
		File::open(latest_file).unwrap().read_to_string(&mut latest_file_content).unwrap();

		assert_eq!(latest_file_content, updated_binary.file_name().and_then(|n| n.to_str()).unwrap());
	}

	#[test]
	fn should_randomly_delay_new_updates() {
		let (update_policy, _) = update_policy();
		let (client, updater, operations_client, _, _, rng) = setup(update_policy);

		let (_, latest_release, latest) = new_upgrade("1.0.1");
		operations_client.set_result(Some(latest.clone()), Some(0));

		rng.set_result(5);

		updater.poll();

		// the update should be delayed for 5 blocks
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Waiting { ref release, block_number, .. } if *release == latest_release && block_number == 5);

		client.add_blocks(1, EachBlockWith::Nothing);
		updater.poll();

		// we should still be in the waiting state after we push one block
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Waiting { ref release, block_number, .. } if *release == latest_release && block_number == 5);

		client.add_blocks(5, EachBlockWith::Nothing);
		updater.poll();

		// after we're past the delay the status should switch to fetching
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Fetching { ref release, .. } if *release == latest_release);
	}

	#[test]
	fn should_not_delay_old_updates() {
		let (update_policy, _) = update_policy();
		let (client, updater, operations_client, ..) = setup(update_policy);
		client.add_blocks(100, EachBlockWith::Nothing);

		let (_, latest_release, latest) = new_upgrade("1.0.1");
		operations_client.set_result(Some(latest.clone()), Some(0));

		updater.poll();

		// the update should not be delayed since it's older than the maximum delay
		// the update was at block 0 (100 blocks ago), and the maximum delay is 10 blocks
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Fetching { ref release, .. } if *release == latest_release);
	}

	#[test]
	fn should_check_for_updates_with_configured_frequency() {
		let (mut update_policy, _) = update_policy();
		update_policy.frequency = 2;

		let (client, updater, operations_client, _, _, rng) = setup(update_policy);
		let (_, latest_release, latest) = new_upgrade("1.0.1");
		operations_client.set_result(Some(latest.clone()), Some(0));
		rng.set_result(5);

		client.add_blocks(1, EachBlockWith::Nothing);
		updater.poll();

		// the updater should stay idle since we only check for updates every other block (odd blocks in this case)
		assert_eq!(updater.state.lock().status, UpdaterStatus::Idle);

		client.add_blocks(1, EachBlockWith::Nothing);
		updater.poll();

		// after adding a block we check for a new update and trigger the random delay (of 5 blocks)
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Waiting { ref release, block_number, .. } if *release == latest_release && block_number == 5);
	}

	#[test]
	fn should_backoff_retry_when_update_fails() {
		let (update_policy, tempdir) = update_policy();
		let (_client, updater, operations_client, fetcher, time_provider, ..) = setup(update_policy);
		let (_, latest_release, latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		let mut now = Instant::now();
		time_provider.set_result(now);

		updater.poll();
		fetcher.trigger(None);

		// we triggered the fetcher with an error result so the updater should backoff any retry
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::FetchBackoff { ref release, ref backoff, .. } if *release == latest_release && backoff.0 == 1);

		now += Duration::from_secs(1);
		time_provider.set_result(now);
		updater.poll();

		// if we don't wait for the elapsed time the updater status should stay the same
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::FetchBackoff { ref release, ref backoff, .. } if *release == latest_release && backoff.0 == 1);

		now += Duration::from_secs(1);
		time_provider.set_result(now);
		updater.poll();
		fetcher.trigger(None);

		// the backoff time has elapsed so we retried again (and failed)
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::FetchBackoff { ref release, ref backoff, .. } if *release == latest_release && backoff.0 == 2);

		now += Duration::from_secs(4);
		time_provider.set_result(now);
		updater.poll();

		let update_file = tempdir.path().join("parity");
		File::create(update_file.clone()).unwrap();
		fetcher.trigger(Some(update_file));

		// after setting up the mocked fetch and waiting for the backoff period the update should succeed
		assert_eq!(updater.state.lock().status, UpdaterStatus::Ready { release: latest_release });
	}

	#[test]
	fn should_quit_backoff_on_new_release() {
		let (update_policy, tempdir) = update_policy();
		let (_client, updater, operations_client, fetcher, ..) = setup(update_policy);
		let (_, latest_release, latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		updater.poll();
		fetcher.trigger(None);

		// we triggered the fetcher with an error result so the updater should backoff any retry
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::FetchBackoff { ref release, ref backoff, .. } if *release == latest_release && backoff.0 == 1);

		// mock new working release and trigger the fetch afterwards
		let (_, latest_release, latest) = new_upgrade("1.0.2");
		operations_client.set_result(Some(latest.clone()), None);
		let update_file = tempdir.path().join("parity");
		File::create(update_file.clone()).unwrap();

		updater.poll();
		fetcher.trigger(Some(update_file));

		// a new release should short-circuit the backoff
		assert_eq!(updater.state.lock().status, UpdaterStatus::Ready { release: latest_release });
	}

	#[test]
	fn should_detect_already_downloaded_releases() {
		let (update_policy, tempdir) = update_policy();
		let (_client, updater, operations_client, ..) = setup(update_policy);
		let (latest_version, latest_release, latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		// mock final update file
		let update_file = tempdir.path().join(Updater::update_file_name(&latest_version));
		File::create(update_file.clone()).unwrap();

		updater.poll();

		// after checking for a new update we immediately declare it as ready since it already exists on disk
		// there was no need to trigger the fetch
		assert_eq!(updater.state.lock().status, UpdaterStatus::Ready { release: latest_release });
	}

	#[test]
	fn should_stay_disabled_after_fatal_error() {
		let (update_policy, tempdir) = update_policy();
		let (client, updater, operations_client, fetcher, ..) = setup(update_policy);
		let (_, _, latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		updater.poll();
		// trigger the fetch but don't create the file on-disk. this should lead to a fatal error that disables the updater
		let update_file = tempdir.path().join("parity");
		fetcher.trigger(Some(update_file));

		assert_eq!(updater.state.lock().status, UpdaterStatus::Disabled);

		client.add_blocks(100, EachBlockWith::Nothing);
		updater.poll();

		// the updater should stay disabled after new blocks are pushed
		assert_eq!(updater.state.lock().status, UpdaterStatus::Disabled);

		let (_, _, latest) = new_upgrade("1.0.2");
		operations_client.set_result(Some(latest.clone()), None);

		updater.poll();

		// the updater should stay disabled after a new release is pushed
		assert_eq!(updater.state.lock().status, UpdaterStatus::Disabled);
	}

	#[test]
	fn should_ignore_current_fetch_on_new_release() {
		let (update_policy, _) = update_policy();
		let (_client, updater, operations_client, fetcher, ..) = setup(update_policy);
		let (_, latest_release, latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		updater.poll();

		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Fetching { ref release, .. } if *release == latest_release);

		let (_, latest_release, latest) = new_upgrade("1.0.2");
		operations_client.set_result(Some(latest.clone()), None);
		fetcher.trigger(None);
		updater.poll();

		// even though we triggered the previous fetch with an error, the current state was updated to fetch the new
		// release, and the previous fetch is ignored
		assert_matches!(
			updater.state.lock().status,
			UpdaterStatus::Fetching { ref release, .. } if *release == latest_release);
	}

	#[test]
	fn should_auto_install_updates_if_update_policy_allows() {
		let (mut update_policy, tempdir) = update_policy();
		update_policy.filter = UpdateFilter::All;
		let (_client, updater, operations_client, fetcher, ..) = setup(update_policy);
		let (latest_version, latest_release, latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		// we start in idle state and with no information regarding the latest release
		assert_eq!(updater.state.lock().latest, None);
		assert_eq!(updater.state.lock().status, UpdaterStatus::Idle);

		updater.poll();

		// mock fetcher with update binary and trigger the fetch
		let update_file = tempdir.path().join("parity");
		File::create(update_file.clone()).unwrap();
		fetcher.trigger(Some(update_file));

		// the update is auto installed since the update policy allows it
		assert_eq!(updater.state.lock().status, UpdaterStatus::Installed { release: latest_release });

		// the final binary should exist in the updates folder and the 'latest' file should be updated to point to it
		let updated_binary = tempdir.path().join(Updater::update_file_name(&latest_version));
		let latest_file = tempdir.path().join("latest");

		assert!(updated_binary.exists());
		assert!(latest_file.exists());

		let mut latest_file_content = String::new();
		File::open(latest_file).unwrap().read_to_string(&mut latest_file_content).unwrap();

		assert_eq!(latest_file_content, updated_binary.file_name().and_then(|n| n.to_str()).unwrap());
	}

	#[test]
	fn should_update_capability() {
		let (update_policy, _tempdir) = update_policy();
		let (client, updater, operations_client, _, ..) = setup(update_policy);
		let (_, _, mut latest) = new_upgrade("1.0.1");

		// mock operations contract with a new version
		operations_client.set_result(Some(latest.clone()), None);

		// we start with no information regarding our node's capabilities
		assert_eq!(updater.state.lock().capability, CapState::Unknown);

		updater.poll();

		// our node supports the current fork
		assert_eq!(updater.state.lock().capability, CapState::Capable);

		// lets announce a new fork which our node doesn't support
		latest.fork = 2;
		operations_client.set_result(Some(latest.clone()), None);
		updater.poll();

		// our node is only capable of operating until block #2 when the fork triggers
		assert_eq!(updater.state.lock().capability, CapState::CapableUntil(2));

		client.add_blocks(3, EachBlockWith::Nothing);
		updater.poll();

		// after we move past the fork the capability should be updated to incapable
		assert_eq!(updater.state.lock().capability, CapState::IncapableSince(2));

		// and since our update policy requires consensus, the client should be disabled
		assert!(client.is_disabled());
	}
}
