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

//! Creates and registers client and network services.

use util::*;
use io::*;
use spec::Spec;
use error::*;
use client::{Client, ClientConfig, ChainNotify};
use miner::Miner;

use snapshot::ManifestData;
use snapshot::service::{Service as SnapshotService, ServiceParams as SnapServiceParams};
use std::sync::atomic::AtomicBool;

#[cfg(feature="ipc")]
use nanoipc;

/// Message type for external and internal events
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClientIoMessage {
	/// Best Block Hash in chain has been changed
	NewChainHead,
	/// A block is ready
	BlockVerified,
	/// New transaction RLPs are ready to be imported
	NewTransactions(Vec<Bytes>, usize),
	/// Begin snapshot restoration
	BeginRestoration(ManifestData),
	/// Feed a state chunk to the snapshot service
	FeedStateChunk(H256, Bytes),
	/// Feed a block chunk to the snapshot service
	FeedBlockChunk(H256, Bytes),
	/// Take a snapshot for the block with given number.
	TakeSnapshot(u64),
	/// New consensus message received.
	NewMessage(Bytes)
}

/// Client service setup. Creates and registers client and network services with the IO subsystem.
pub struct ClientService {
	io_service: Arc<IoService<ClientIoMessage>>,
	client: Arc<Client>,
	snapshot: Arc<SnapshotService>,
	database: Arc<Database>,
	_stop_guard: ::devtools::StopGuard,
}

impl ClientService {
	/// Start the `ClientService`.
	pub fn start(
		config: ClientConfig,
		spec: &Spec,
		client_path: &Path,
		snapshot_path: &Path,
		ipc_path: &Path,
		miner: Arc<Miner>,
		) -> Result<ClientService, Error>
	{
		let io_service = IoService::<ClientIoMessage>::start()?;

		info!("Configured for {} using {} engine", Colour::White.bold().paint(spec.name.clone()), Colour::Yellow.bold().paint(spec.engine.name()));

		let mut db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

		// give all rocksdb cache to state column; everything else has its
		// own caches.
		if let Some(size) = config.db_cache_size {
			db_config.set_cache(::db::COL_STATE, size);
		}

		db_config.compaction = config.db_compaction.compaction_profile(client_path);
		db_config.wal = config.db_wal;

		let db = Arc::new(Database::open(
			&db_config,
			&client_path.to_str().expect("DB path could not be converted to string.")
		).map_err(::client::Error::Database)?);


		let pruning = config.pruning;
		let client = Client::new(config, &spec, db.clone(), miner, io_service.channel())?;

		let snapshot_params = SnapServiceParams {
			engine: spec.engine.clone(),
			genesis_block: spec.genesis_block(),
			db_config: db_config.clone(),
			pruning: pruning,
			channel: io_service.channel(),
			snapshot_root: snapshot_path.into(),
			db_restore: client.clone(),
		};
		let snapshot = Arc::new(SnapshotService::new(snapshot_params)?);

		let client_io = Arc::new(ClientIoHandler {
			client: client.clone(),
			snapshot: snapshot.clone(),
		});
		io_service.register_handler(client_io)?;

		spec.engine.register_client(Arc::downgrade(&client) as _);

		let stop_guard = ::devtools::StopGuard::new();
		run_ipc(ipc_path, client.clone(), snapshot.clone(), stop_guard.share());

		Ok(ClientService {
			io_service: Arc::new(io_service),
			client: client,
			snapshot: snapshot,
			database: db,
			_stop_guard: stop_guard,
		})
	}

	/// Get general IO interface
	pub fn register_io_handler(&self, handler: Arc<IoHandler<ClientIoMessage> + Send>) -> Result<(), IoError> {
		self.io_service.register_handler(handler)
	}

	/// Get client interface
	pub fn client(&self) -> Arc<Client> {
		self.client.clone()
	}

	/// Get snapshot interface.
	pub fn snapshot_service(&self) -> Arc<SnapshotService> {
		self.snapshot.clone()
	}

	/// Get network service component
	pub fn io(&self) -> Arc<IoService<ClientIoMessage>> {
		self.io_service.clone()
	}

	/// Set the actor to be notified on certain chain events
	pub fn add_notify(&self, notify: Arc<ChainNotify>) {
		self.client.add_notify(notify);
	}

	/// Get a handle to the database.
	pub fn db(&self) -> Arc<KeyValueDB> { self.database.clone() }
}

/// IO interface for the Client handler
struct ClientIoHandler {
	client: Arc<Client>,
	snapshot: Arc<SnapshotService>,
}

const CLIENT_TICK_TIMER: TimerToken = 0;
const SNAPSHOT_TICK_TIMER: TimerToken = 1;

const CLIENT_TICK_MS: u64 = 5000;
const SNAPSHOT_TICK_MS: u64 = 10000;

impl IoHandler<ClientIoMessage> for ClientIoHandler {
	fn initialize(&self, io: &IoContext<ClientIoMessage>) {
		io.register_timer(CLIENT_TICK_TIMER, CLIENT_TICK_MS).expect("Error registering client timer");
		io.register_timer(SNAPSHOT_TICK_TIMER, SNAPSHOT_TICK_MS).expect("Error registering snapshot timer");
	}

	fn timeout(&self, _io: &IoContext<ClientIoMessage>, timer: TimerToken) {
		match timer {
			CLIENT_TICK_TIMER => self.client.tick(),
			SNAPSHOT_TICK_TIMER => self.snapshot.tick(),
			_ => warn!("IO service triggered unregistered timer '{}'", timer),
		}
	}

	#[cfg_attr(feature="dev", allow(single_match))]
	fn message(&self, _io: &IoContext<ClientIoMessage>, net_message: &ClientIoMessage) {
		use std::thread;

		match *net_message {
			ClientIoMessage::BlockVerified => { self.client.import_verified_blocks(); }
			ClientIoMessage::NewTransactions(ref transactions, peer_id) => {
				self.client.import_queued_transactions(transactions, peer_id);
			}
			ClientIoMessage::BeginRestoration(ref manifest) => {
				if let Err(e) = self.snapshot.init_restore(manifest.clone(), true) {
					warn!("Failed to initialize snapshot restoration: {}", e);
				}
			}
			ClientIoMessage::FeedStateChunk(ref hash, ref chunk) => self.snapshot.feed_state_chunk(*hash, chunk),
			ClientIoMessage::FeedBlockChunk(ref hash, ref chunk) => self.snapshot.feed_block_chunk(*hash, chunk),
			ClientIoMessage::TakeSnapshot(num) => {
				let client = self.client.clone();
				let snapshot = self.snapshot.clone();

				let res = thread::Builder::new().name("Periodic Snapshot".into()).spawn(move || {
					if let Err(e) = snapshot.take_snapshot(&*client, num) {
						warn!("Failed to take snapshot at block #{}: {}", num, e);
					}
				});

				if let Err(e) = res {
					debug!(target: "snapshot", "Failed to initialize periodic snapshot thread: {:?}", e);
				}
			},
			ClientIoMessage::NewMessage(ref message) => if let Err(e) = self.client.engine().handle_message(message) {
				trace!(target: "poa", "Invalid message received: {}", e);
			},
			_ => {} // ignore other messages
		}
	}
}

#[cfg(feature="ipc")]
fn run_ipc(base_path: &Path, client: Arc<Client>, snapshot_service: Arc<SnapshotService>, stop: Arc<AtomicBool>) {
	let mut path = base_path.to_owned();
	path.push("parity-chain.ipc");
	let socket_addr = format!("ipc://{}", path.to_string_lossy());
	let s = stop.clone();
	::std::thread::spawn(move || {
		let mut worker = nanoipc::Worker::new(&(client as Arc<BlockChainClient>));
		worker.add_reqrep(&socket_addr).expect("Ipc expected to initialize with no issues");

		while !s.load(::std::sync::atomic::Ordering::Relaxed) {
			worker.poll();
		}
	});

	let mut path = base_path.to_owned();
	path.push("parity-snapshot.ipc");
	let socket_addr = format!("ipc://{}", path.to_string_lossy());
	::std::thread::spawn(move || {
		let mut worker = nanoipc::Worker::new(&(snapshot_service as Arc<::snapshot::SnapshotService>));
		worker.add_reqrep(&socket_addr).expect("Ipc expected to initialize with no issues");

		while !stop.load(::std::sync::atomic::Ordering::Relaxed) {
			worker.poll();
		}
	});
}

#[cfg(not(feature="ipc"))]
fn run_ipc(_base_path: &Path, _client: Arc<Client>, _snapshot_service: Arc<SnapshotService>, _stop: Arc<AtomicBool>) {
}

#[cfg(test)]
mod tests {
	use super::*;
	use tests::helpers::*;
	use devtools::*;
	use client::ClientConfig;
	use std::sync::Arc;
	use miner::Miner;

	#[test]
	fn it_can_be_started() {
		let temp_path = RandomTempPath::new();
		let path = temp_path.as_path().to_owned();
		let client_path = {
			let mut path = path.to_owned();
			path.push("client");
			path
		};

		let snapshot_path = {
			let mut path = path.to_owned();
			path.push("snapshot");
			path
		};

		let spec = get_test_spec();
		let service = ClientService::start(
			ClientConfig::default(),
			&spec,
			&client_path,
			&snapshot_path,
			&path,
			Arc::new(Miner::with_spec(&spec)),
		);
		assert!(service.is_ok());
		drop(service.unwrap());
		::std::thread::park_timeout(::std::time::Duration::from_millis(100));
	}
}
