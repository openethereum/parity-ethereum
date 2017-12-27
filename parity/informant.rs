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

extern crate ansi_term;
use self::ansi_term::Colour::{White, Yellow, Green, Cyan, Blue};
use self::ansi_term::{Colour, Style};

use std::sync::{Arc};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering as AtomicOrdering};
use std::time::{Instant, Duration};

use ethcore::client::{BlockId, BlockChainClient, BlockChainInfo, BlockQueueInfo, ChainNotify, ClientReport, Client};
use ethcore::header::BlockNumber;
use ethcore::service::ClientIoMessage;
use ethcore::snapshot::{RestorationStatus, SnapshotService as SS};
use ethcore::snapshot::service::Service as SnapshotService;
use ethsync::{LightSyncProvider, LightSync, SyncProvider, ManageNetwork};
use io::{TimerToken, IoContext, IoHandler};
use isatty::{stdout_isatty};
use light::Cache as LightDataCache;
use light::client::LightChainClient;
use number_prefix::{binary_prefix, Standalone, Prefixed};
use parity_rpc::{is_major_importing};
use parity_rpc::informant::RpcStats;
use bigint::hash::H256;
use bytes::Bytes;
use parking_lot::{RwLock, Mutex};

/// Format byte counts to standard denominations.
pub fn format_bytes(b: usize) -> String {
	match binary_prefix(b as f64) {
		Standalone(bytes)   => format!("{} bytes", bytes),
		Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
	}
}

/// Something that can be converted to milliseconds.
pub trait MillisecondDuration {
	/// Get the value in milliseconds.
	fn as_milliseconds(&self) -> u64;
}

impl MillisecondDuration for Duration {
	fn as_milliseconds(&self) -> u64 {
		self.as_secs() * 1000 + self.subsec_nanos() as u64 / 1_000_000
	}
}

#[derive(Default)]
struct CacheSizes {
	sizes: ::std::collections::BTreeMap<&'static str, usize>,
}

impl CacheSizes {
	fn insert(&mut self, key: &'static str, bytes: usize) {
		self.sizes.insert(key, bytes);
	}

	fn display<F>(&self, style: Style, paint: F) -> String
		where F: Fn(Style, String) -> String
	{
		use std::fmt::Write;

		let mut buf = String::new();
		for (name, &size) in &self.sizes {

			write!(buf, " {:>8} {}", paint(style, format_bytes(size)), name)
				.expect("writing to string won't fail unless OOM; qed")
		}

		buf
	}
}

pub struct SyncInfo {
	last_imported_block_number: BlockNumber,
	last_imported_old_block_number: Option<BlockNumber>,
	num_peers: usize,
	max_peers: u32,
	snapshot_sync: bool,
}

pub struct Report {
	importing: bool,
	chain_info: BlockChainInfo,
	client_report: ClientReport,
	queue_info: BlockQueueInfo,
	cache_sizes: CacheSizes,
	sync_info: Option<SyncInfo>,
}

/// Something which can provide data to the informant.
pub trait InformantData: Send + Sync {
	/// Whether it executes transactions
	fn executes_transactions(&self) -> bool;

	/// Whether it is currently importing (also included in `Report`)
	fn is_major_importing(&self) -> bool;

	/// Generate a report of blockchain status, memory usage, and sync info.
	fn report(&self) -> Report;
}

/// Informant data for a full node.
pub struct FullNodeInformantData {
	pub client: Arc<Client>,
	pub sync: Option<Arc<SyncProvider>>,
	pub net: Option<Arc<ManageNetwork>>,
}

impl InformantData for FullNodeInformantData {
	fn executes_transactions(&self) -> bool { true }

	fn is_major_importing(&self) -> bool {
		let state = self.sync.as_ref().map(|sync| sync.status().state);
		is_major_importing(state, self.client.queue_info())
	}

	fn report(&self) -> Report {
		let (client_report, queue_info, blockchain_cache_info) =
			(self.client.report(), self.client.queue_info(), self.client.blockchain_cache_info());

		let chain_info = self.client.chain_info();

		let mut cache_sizes = CacheSizes::default();
		cache_sizes.insert("db", client_report.state_db_mem);
		cache_sizes.insert("queue", queue_info.mem_used);
		cache_sizes.insert("chain", blockchain_cache_info.total());

		let (importing, sync_info) = match (self.sync.as_ref(), self.net.as_ref()) {
			(Some(sync), Some(net)) => {
				let status = sync.status();
				let net_config = net.network_config();

				cache_sizes.insert("sync", status.mem_used);

				let importing = is_major_importing(Some(status.state), queue_info.clone());
				(importing, Some(SyncInfo {
					last_imported_block_number: status.last_imported_block_number.unwrap_or(chain_info.best_block_number),
					last_imported_old_block_number: status.last_imported_old_block_number,
					num_peers: status.num_peers,
					max_peers: status.current_max_peers(net_config.min_peers, net_config.max_peers),
					snapshot_sync: status.is_snapshot_syncing(),
				}))
			}
			_ => (is_major_importing(self.sync.as_ref().map(|s| s.status().state), queue_info.clone()), None),
		};

		Report {
			importing,
			chain_info,
			client_report,
			queue_info,
			cache_sizes,
			sync_info,
		}
	}
}

/// Informant data for a light node -- note that the network is required.
pub struct LightNodeInformantData {
	pub client: Arc<LightChainClient>,
	pub sync: Arc<LightSync>,
	pub cache: Arc<Mutex<LightDataCache>>,
}

impl InformantData for LightNodeInformantData {
	fn executes_transactions(&self) -> bool { false }

	fn is_major_importing(&self) -> bool {
		self.sync.is_major_importing()
	}

	fn report(&self) -> Report {
		let (client_report, queue_info, chain_info) =
			(self.client.report(), self.client.queue_info(), self.client.chain_info());

		let mut cache_sizes = CacheSizes::default();
		cache_sizes.insert("queue", queue_info.mem_used);
		cache_sizes.insert("cache", self.cache.lock().mem_used());

		let peer_numbers = self.sync.peer_numbers();
		let sync_info = Some(SyncInfo {
			last_imported_block_number: chain_info.best_block_number,
			last_imported_old_block_number: None,
			num_peers: peer_numbers.connected,
			max_peers: peer_numbers.max as u32,
			snapshot_sync: false,
		});

		Report {
			importing: self.sync.is_major_importing(),
			chain_info,
			client_report,
			queue_info,
			cache_sizes,
			sync_info,
		}
	}
}

pub struct Informant<T> {
	last_tick: RwLock<Instant>,
	with_color: bool,
	target: T,
	snapshot: Option<Arc<SnapshotService>>,
	rpc_stats: Option<Arc<RpcStats>>,
	last_import: Mutex<Instant>,
	skipped: AtomicUsize,
	skipped_txs: AtomicUsize,
	in_shutdown: AtomicBool,
	last_report: Mutex<ClientReport>,
}

impl<T: InformantData> Informant<T> {
	/// Make a new instance potentially `with_color` output.
	pub fn new(
		target: T,
		snapshot: Option<Arc<SnapshotService>>,
		rpc_stats: Option<Arc<RpcStats>>,
		with_color: bool,
	) -> Self {
		Informant {
			last_tick: RwLock::new(Instant::now()),
			with_color: with_color,
			target: target,
			snapshot: snapshot,
			rpc_stats: rpc_stats,
			last_import: Mutex::new(Instant::now()),
			skipped: AtomicUsize::new(0),
			skipped_txs: AtomicUsize::new(0),
			in_shutdown: AtomicBool::new(false),
			last_report: Mutex::new(Default::default()),
		}
	}

	/// Signal that we're shutting down; no more output necessary.
	pub fn shutdown(&self) {
		self.in_shutdown.store(true, ::std::sync::atomic::Ordering::SeqCst);
	}

	pub fn tick(&self) {
		let elapsed = self.last_tick.read().elapsed();
		if elapsed < Duration::from_secs(5) {
			return;
		}

		let (client_report, full_report) = {
			let mut last_report = self.last_report.lock();
			let full_report = self.target.report();
			let diffed = full_report.client_report.clone() - &*last_report;
			*last_report = full_report.client_report.clone();
			(diffed, full_report)
		};

		let Report {
			importing,
			chain_info,
			queue_info,
			cache_sizes,
			sync_info,
			..
		} = full_report;

		let rpc_stats = self.rpc_stats.as_ref();

		let (snapshot_sync, snapshot_current, snapshot_total) = self.snapshot.as_ref().map_or((false, 0, 0), |s|
			match s.status() {
				RestorationStatus::Ongoing { state_chunks, block_chunks, state_chunks_done, block_chunks_done } =>
					(true, state_chunks_done + block_chunks_done, state_chunks + block_chunks),
				_ => (false, 0, 0),
			}
		);
		let snapshot_sync = snapshot_sync && sync_info.as_ref().map_or(false, |s| s.snapshot_sync);
		if !importing && !snapshot_sync && elapsed < Duration::from_secs(30) {
			return;
		}

		*self.last_tick.write() = Instant::now();

		let paint = |c: Style, t: String| match self.with_color && stdout_isatty() {
			true => format!("{}", c.paint(t)),
			false => t,
		};

		info!(target: "import", "{}  {}  {}  {}",
			match importing {
				true => match snapshot_sync {
					false => format!("Syncing {} {}  {}  {}+{} Qed",
						paint(White.bold(), format!("{:>8}", format!("#{}", chain_info.best_block_number))),
						paint(White.bold(), format!("{}", chain_info.best_block_hash)),
						if self.target.executes_transactions() {
							format!("{} blk/s {} tx/s {} Mgas/s",
								paint(Yellow.bold(), format!("{:4}", (client_report.blocks_imported * 1000) as u64 / elapsed.as_milliseconds())),
								paint(Yellow.bold(), format!("{:4}", (client_report.transactions_applied * 1000) as u64 / elapsed.as_milliseconds())),
								paint(Yellow.bold(), format!("{:3}", (client_report.gas_processed / From::from(elapsed.as_milliseconds() * 1000)).low_u64()))
							)
						} else {
							format!("{} hdr/s",
								paint(Yellow.bold(), format!("{:4}", (client_report.blocks_imported * 1000) as u64 / elapsed.as_milliseconds()))
							)
						},
						paint(Green.bold(), format!("{:5}", queue_info.unverified_queue_size)),
						paint(Green.bold(), format!("{:5}", queue_info.verified_queue_size))
					),
					true => format!("Syncing snapshot {}/{}", snapshot_current, snapshot_total),
				},
				false => String::new(),
			},
			match sync_info.as_ref() {
				Some(ref sync_info) => format!("{}{}/{} peers",
					match importing {
						true => format!("{}   ", paint(Green.bold(), format!("{:>8}", format!("#{}", sync_info.last_imported_block_number)))),
						false => match sync_info.last_imported_old_block_number {
							Some(number) => format!("{}   ", paint(Yellow.bold(), format!("{:>8}", format!("#{}", number)))),
							None => String::new(),
						}
					},
					paint(Cyan.bold(), format!("{:2}", sync_info.num_peers)),
					paint(Cyan.bold(), format!("{:2}", sync_info.max_peers)),
				),
				_ => String::new(),
			},
			cache_sizes.display(Blue.bold(), &paint),
			match rpc_stats {
				Some(ref rpc_stats) => format!(
					"RPC: {} conn, {} req/s, {} µs",
					paint(Blue.bold(), format!("{:2}", rpc_stats.sessions())),
					paint(Blue.bold(), format!("{:2}", rpc_stats.requests_rate())),
					paint(Blue.bold(), format!("{:3}", rpc_stats.approximated_roundtrip())),
				),
				_ => String::new(),
			},
		);
	}
}

impl ChainNotify for Informant<FullNodeInformantData> {
	fn new_blocks(&self, imported: Vec<H256>, _invalid: Vec<H256>, _enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, duration: u64) {
		let mut last_import = self.last_import.lock();
		let client = &self.target.client;

		let importing = self.target.is_major_importing();
		let ripe = Instant::now() > *last_import + Duration::from_secs(1) && !importing;
		let txs_imported = imported.iter()
			.take(imported.len().saturating_sub(if ripe { 1 } else { 0 }))
			.filter_map(|h| client.block(BlockId::Hash(*h)))
			.map(|b| b.transactions_count())
			.sum();

		if ripe {
			if let Some(block) = imported.last().and_then(|h| client.block(BlockId::Hash(*h))) {
				let header_view = block.header_view();
				let size = block.rlp().as_raw().len();
				let (skipped, skipped_txs) = (self.skipped.load(AtomicOrdering::Relaxed) + imported.len() - 1, self.skipped_txs.load(AtomicOrdering::Relaxed) + txs_imported);
				info!(target: "import", "Imported {} {} ({} txs, {} Mgas, {} ms, {} KiB){}",
					Colour::White.bold().paint(format!("#{}", header_view.number())),
					Colour::White.bold().paint(format!("{}", header_view.hash())),
					Colour::Yellow.bold().paint(format!("{}", block.transactions_count())),
					Colour::Yellow.bold().paint(format!("{:.2}", header_view.gas_used().low_u64() as f32 / 1000000f32)),
					Colour::Purple.bold().paint(format!("{:.2}", duration as f32 / 1000000f32)),
					Colour::Blue.bold().paint(format!("{:.2}", size as f32 / 1024f32)),
					if skipped > 0 {
						format!(" + another {} block(s) containing {} tx(s)",
							Colour::Red.bold().paint(format!("{}", skipped)),
							Colour::Red.bold().paint(format!("{}", skipped_txs))
						)
					} else {
						String::new()
					}
				);
				self.skipped.store(0, AtomicOrdering::Relaxed);
				self.skipped_txs.store(0, AtomicOrdering::Relaxed);
				*last_import = Instant::now();
			}
		} else {
			self.skipped.fetch_add(imported.len(), AtomicOrdering::Relaxed);
			self.skipped_txs.fetch_add(txs_imported, AtomicOrdering::Relaxed);
		}
	}
}

const INFO_TIMER: TimerToken = 0;

impl<T: InformantData> IoHandler<ClientIoMessage> for Informant<T> {
	fn initialize(&self, io: &IoContext<ClientIoMessage>) {
		io.register_timer(INFO_TIMER, 5000).expect("Error registering timer");
	}

	fn timeout(&self, _io: &IoContext<ClientIoMessage>, timer: TimerToken) {
		if timer == INFO_TIMER && !self.in_shutdown.load(AtomicOrdering::SeqCst) {
			self.tick();
		}
	}
}
