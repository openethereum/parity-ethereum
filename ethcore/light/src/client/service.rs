// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Minimal IO service for light client.
//! Just handles block import messages and passes them to the client.

use std::fmt;
use std::sync::Arc;
use std::path::Path;

use ethcore::client::ClientIoMessage;
use ethcore::{db, BlockChainDB, BlockChainDBHandler};
use ethcore::error::Error as CoreError;
use ethcore::spec::Spec;
use io::{IoContext, IoError, IoHandler, IoService};
use ethcore::snapshot::{
	SnapshotService as _SnapshotService,
	service::{
		ServiceParams as SnapServiceParams,
		Service as SnapService,
	},
};
use client::header_chain::HardcodedSync;
use client::snapshot::LightClientRestorationParams;

use cache::Cache;
use parking_lot::Mutex;

use super::{ChainDataFetcher, LightChainNotify, Client, Config as ClientConfig};

/// Errors on service initialization.
#[derive(Debug)]
pub enum Error {
	/// Core error.
	Core(CoreError),
	/// I/O service error.
	Io(IoError),
}

impl From<CoreError> for Error {
	#[inline]
	fn from(err: CoreError) -> Error {
		Error::Core(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Core(ref msg) => write!(f, "Core error: {}", msg),
			Error::Io(ref err) => write!(f, "I/O service error: {}", err),
		}
	}
}

type SnapshotService = SnapService<LightClientRestorationParams>;

/// Light client service.
pub struct Service<T> {
	client: Arc<Client<T>>,
	io_service: IoService<ClientIoMessage>,
	snapshot: Arc<SnapshotService>,
}

impl<T: ChainDataFetcher> Service<T> {
	/// Start the service: initialize I/O workers and client itself.
	pub fn start(
		config: ClientConfig,
		spec: Spec,
		fetcher: T,
		db: Arc<BlockChainDB>,
		cache: Arc<Mutex<Cache>>,
		restoration_db_handler: Box<BlockChainDBHandler>,
		snapshot_path: &Path,
	) -> Result<Self, Error> {
		let io_service = IoService::<ClientIoMessage>::start().map_err(Error::Io)?;
		let hs = if config.no_hardcoded_sync { HardcodedSync::Deny } else { HardcodedSync::Allow };
		let client = Arc::new(Client::new(config,
			db.key_value().clone(),
			db::COL_LIGHT_CHAIN,
			&spec,
			fetcher,
			io_service.channel(),
			cache.clone(),
		)?);
		spec.engine.register_client(Arc::downgrade(&client) as _);
		let snapshot_params = SnapServiceParams {
			engine: spec.engine.clone(),
			chain_params: LightClientRestorationParams {
				col: db::COL_LIGHT_CHAIN,
				spec: spec,
				cache: cache,
				allow_hs: hs,
			},
			restoration_db_handler,
			channel: io_service.channel(),
			snapshot_root: snapshot_path.into(),
			db_restore: client.clone(),
		};
		let snapshot = Arc::new(SnapshotService::new(snapshot_params)?);
		let client_io = ClientIoHandler::new(client.clone(), snapshot.clone());
		io_service.register_handler(Arc::new(client_io)).map_err(Error::Io)?;

		Ok(Service {
			client: client,
			io_service: io_service,
			snapshot: snapshot,
		})
	}

	/// Set the actor to be notified on certain chain events
	pub fn add_notify(&self, notify: Arc<LightChainNotify>) {
		self.client.add_listener(Arc::downgrade(&notify));
	}

	/// Register an I/O handler on the service.
	pub fn register_handler(&self, handler: Arc<IoHandler<ClientIoMessage> + Send>) -> Result<(), IoError> {
		self.io_service.register_handler(handler)
	}

	/// Get a handle to the client.
	pub fn client(&self) -> &Arc<Client<T>> {
		&self.client
	}

	/// Get snapshot interface.
	pub fn snapshot_service(&self) -> Arc<SnapshotService> {
		self.snapshot.clone()
	}

	/// Shutdown the Service.
	pub fn shutdown(&self) {
		self.snapshot.shutdown();
	}
}

/// IO interface for the Client handler
struct ClientIoHandler<T> {
	client: Arc<Client<T>>,
	snapshot: Arc<SnapshotService>,
}

impl<T> ClientIoHandler<T> {
	pub fn new(client: Arc<Client<T>>, snapshot: Arc<SnapshotService>) -> Self {
		ClientIoHandler {
			client: client,
			snapshot: snapshot,
		}
	}
}

impl<T: ChainDataFetcher> IoHandler<ClientIoMessage> for ClientIoHandler<T> {
	fn message(&self, _io: &IoContext<ClientIoMessage>, message: &ClientIoMessage) {
		match *message {
			ClientIoMessage::BlockVerified => {
				self.client.import_verified();
			},
			ClientIoMessage::BeginRestoration(ref manifest) => {
				if let Err(e) = self.snapshot.init_restore(manifest.clone(), true) {
					warn!("Failed to initialize snapshot restoration: {}", e);
				}
			},
			ClientIoMessage::FeedBlockChunk(ref hash, ref chunk) => self.snapshot.feed_block_chunk(*hash, chunk),
			_ => {} // ignore other messages
		}
	}
}

#[cfg(test)]
mod tests {
	use cache::Cache;
	use client::fetch;
	use ethcore::client::ClientConfig;
	use ethcore::db::NUM_COLUMNS;
	use ethcore::spec::Spec;
	use ethcore::test_helpers;
	use kvdb_rocksdb::{DatabaseConfig, CompactionProfile};
	use parking_lot::Mutex;
	use std::sync::Arc;
	use std::time::Duration;
	use super::Service;
	use tempdir::TempDir;

	#[test]
	fn it_works() {
		let tempdir = TempDir::new("").unwrap();
		let client_path = tempdir.path().join("client");
		let snapshot_path = tempdir.path().join("snapshot");

		let client_config = ClientConfig::default();
		let mut client_db_config = DatabaseConfig::with_columns(NUM_COLUMNS);

		client_db_config.memory_budget = client_config.db_cache_size;
		client_db_config.compaction = CompactionProfile::auto(&client_path);

		let restoration_db_handler = test_helpers::restoration_db_handler(client_db_config);
		let db = test_helpers::new_db();
		let spec = Spec::new_test();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::from_secs(6 * 3600))));

		Service::start(
			Default::default(), spec, fetch::unavailable(), db, cache,
			restoration_db_handler, &snapshot_path,
		).unwrap();
	}
}
