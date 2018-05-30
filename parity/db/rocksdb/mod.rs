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

extern crate kvdb_rocksdb;
extern crate migration_rocksdb;

use std::fs;
use std::sync::Arc;
use std::path::Path;
use parking_lot::RwLock;
use blooms_db;
use ethcore::{BlockChainDBHandler, BlockChainDB};
use ethcore::error::Error;
use ethcore::db::NUM_COLUMNS;
use ethcore::client::{ClientConfig, DatabaseCompactionProfile};
use kvdb::KeyValueDB;
use self::kvdb_rocksdb::{Database, DatabaseConfig};

use cache::CacheConfig;

mod migration;
mod helpers;

pub use self::migration::migrate;

struct AppDB {
	key_value: Arc<KeyValueDB>,
	blooms: RwLock<blooms_db::Database>,
	trace_blooms: RwLock<blooms_db::Database>,
}

impl BlockChainDB for AppDB {
	fn key_value(&self) -> &Arc<KeyValueDB> {
		&self.key_value
	}

	fn blooms(&self) -> &RwLock<blooms_db::Database> {
		&self.blooms
	}

	fn trace_blooms(&self) -> &RwLock<blooms_db::Database> {
		&self.trace_blooms
	}
}

/// Open a secret store DB using the given secret store data path. The DB path is one level beneath the data path.
#[cfg(feature = "secretstore")]
pub fn open_secretstore_db(data_path: &str) -> Result<Arc<KeyValueDB>, String> {
	use std::path::PathBuf;

	let mut db_path = PathBuf::from(data_path);
	db_path.push("db");
	let db_path = db_path.to_str().ok_or_else(|| "Invalid secretstore path".to_string())?;
	Ok(Arc::new(Database::open_default(&db_path).map_err(|e| format!("Error opening database: {:?}", e))?))
}

/// Open a new client DB.
pub fn open_client_db(client_path: &Path, client_config: &ClientConfig) -> Result<Arc<BlockChainDB>, String> {
	let client_db_config = helpers::client_db_config(client_path, client_config);
	let blooms_path = client_path.join("blooms");
	let trace_blooms_path = client_path.join("trace_blooms");
	fs::create_dir(&blooms_path).map_err(|e| e.to_string())?;
	fs::create_dir(&trace_blooms_path).map_err(|e| e.to_string())?;

	let client_db = Arc::new(Database::open(
		&client_db_config,
		&client_path.to_str().expect("DB path could not be converted to string.")
	).map_err(|e| format!("Client service database error: {:?}", e))?);

	let db = AppDB {
		key_value: client_db,
		blooms: RwLock::new(blooms_db::Database::open(blooms_path).map_err(|e| e.to_string())?),
		trace_blooms: RwLock::new(blooms_db::Database::open(trace_blooms_path).map_err(|e| e.to_string())?),
	};

	Ok(Arc::new(db))
}

/// Create a restoration db handler using the config generated by `client_path` and `client_config`.
pub fn restoration_db_handler(client_path: &Path, client_config: &ClientConfig) -> Box<BlockChainDBHandler> {
	let client_db_config = helpers::client_db_config(client_path, client_config);

	struct RestorationDBHandler {
		config: DatabaseConfig,
	}

	impl BlockChainDBHandler for RestorationDBHandler {
		fn open(&self, db_path: &Path) -> Result<Arc<BlockChainDB>, Error> {
			let blooms_path = db_path.join("blooms");
			let trace_blooms_path = db_path.join("trace_blooms");
			fs::create_dir(&blooms_path)?;
			fs::create_dir(&trace_blooms_path)?;

			let db = AppDB {
				key_value: Arc::new(Database::open(&self.config, &db_path.to_string_lossy())?),
				blooms: RwLock::new(blooms_db::Database::open(blooms_path)?),
				trace_blooms: RwLock::new(blooms_db::Database::open(trace_blooms_path)?),
			};

			Ok(Arc::new(db))
		}
	}

	Box::new(RestorationDBHandler {
		config: client_db_config,
	})
}

/// Open a new main DB.
pub fn open_db(client_path: &str, cache_config: &CacheConfig, compaction: &DatabaseCompactionProfile, wal: bool) -> Result<Arc<BlockChainDB>, String> {
	let path = Path::new(client_path);

	let db_config = DatabaseConfig {
		memory_budget: Some(cache_config.blockchain() as usize * 1024 * 1024),
		compaction: helpers::compaction_profile(&compaction, path),
		wal,
		.. DatabaseConfig::with_columns(NUM_COLUMNS)
	};

	let key_value = Arc::new(Database::open(
		&db_config,
		client_path
	).map_err(|e| format!("Failed to open database: {}", e))?);

	let blooms_path = path.join("blooms");
	let trace_blooms_path = path.join("trace_blooms");
	fs::create_dir(&blooms_path).map_err(|e| e.to_string())?;
	fs::create_dir(&trace_blooms_path).map_err(|e| e.to_string())?;

	let db = AppDB {
		key_value,
		blooms: RwLock::new(blooms_db::Database::open(blooms_path).map_err(|e| e.to_string())?),
		trace_blooms: RwLock::new(blooms_db::Database::open(trace_blooms_path).map_err(|e| e.to_string())?),
	};

	Ok(Arc::new(db))
}
