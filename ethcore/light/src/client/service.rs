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

//! Minimal IO service for light client.
//! Just handles block import messages and passes them to the client.

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use ethcore::db;
use ethcore::service::ClientIoMessage;
use ethcore::spec::Spec;
use io::{IoContext, IoError, IoHandler, IoService};
use util::kvdb::{Database, DatabaseConfig};

use cache::Cache;
use parking_lot::Mutex;

use super::{ChainDataFetcher, Client, Config as ClientConfig};

/// Errors on service initialization.
#[derive(Debug)]
pub enum Error {
	/// Database error.
	Database(String),
	/// I/O service error.
	Io(IoError),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Database(ref msg) => write!(f, "Database error: {}", msg),
			Error::Io(ref err) => write!(f, "I/O service error: {}", err),
		}
	}
}

/// Light client service.
pub struct Service<T> {
	client: Arc<Client<T>>,
	io_service: IoService<ClientIoMessage>,
}

impl<T: ChainDataFetcher> Service<T> {
	/// Start the service: initialize I/O workers and client itself.
	pub fn start(config: ClientConfig, spec: &Spec, fetcher: T, path: &Path, cache: Arc<Mutex<Cache>>) -> Result<Self, Error> {

		// initialize database.
		let mut db_config = DatabaseConfig::with_columns(db::NUM_COLUMNS);

		// give all rocksdb cache to the header chain column.
		if let Some(size) = config.db_cache_size {
			db_config.set_cache(db::COL_LIGHT_CHAIN, size);
		}

		db_config.compaction = config.db_compaction;
		db_config.wal = config.db_wal;

		let db = Arc::new(Database::open(
			&db_config,
			&path.to_str().expect("DB path could not be converted to string.")
		).map_err(Error::Database)?);

		let io_service = IoService::<ClientIoMessage>::start().map_err(Error::Io)?;
		let client = Arc::new(Client::new(config,
			db,
			db::COL_LIGHT_CHAIN,
			spec,
			fetcher,
			io_service.channel(),
			cache,
		).map_err(Error::Database)?);

		io_service.register_handler(Arc::new(ImportBlocks(client.clone()))).map_err(Error::Io)?;
		spec.engine.register_client(Arc::downgrade(&client) as _);

		Ok(Service {
			client: client,
			io_service: io_service,
		})
	}

	/// Register an I/O handler on the service.
	pub fn register_handler(&self, handler: Arc<IoHandler<ClientIoMessage> + Send>) -> Result<(), IoError> {
		self.io_service.register_handler(handler)
	}

	/// Get a handle to the client.
	pub fn client(&self) -> &Arc<Client<T>> {
		&self.client
	}
}

struct ImportBlocks<T>(Arc<Client<T>>);

impl<T: ChainDataFetcher> IoHandler<ClientIoMessage> for ImportBlocks<T> {
	fn message(&self, _io: &IoContext<ClientIoMessage>, message: &ClientIoMessage) {
		if let ClientIoMessage::BlockVerified = *message {
			self.0.import_verified();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Service;
	use devtools::RandomTempPath;
	use ethcore::spec::Spec;

	use std::sync::Arc;
	use cache::Cache;
	use client::fetch;
	use time::Duration;
	use parking_lot::Mutex;

	#[test]
	fn it_works() {
		let spec = Spec::new_test();
		let temp_path = RandomTempPath::new();
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));

		Service::start(Default::default(), &spec, fetch::unavailable(), temp_path.as_path(), cache).unwrap();
	}
}
