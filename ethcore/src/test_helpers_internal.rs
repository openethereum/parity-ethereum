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

//! Internal helpers for client tests

use std::fs;
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use kvdb::{KeyValueDB};
use kvdb_rocksdb::{Database, DatabaseConfig};
use blockchain::{BlockChainDBHandler, BlockChainDB};
use blooms_db;
use error::Error;

/// Creates new instance of KeyValueDBHandler
pub fn restoration_db_handler(config: DatabaseConfig) -> Box<BlockChainDBHandler> {
	struct RestorationDBHandler {
		config: DatabaseConfig,
	}

	struct RestorationDB {
		blooms: RwLock<blooms_db::Database>,
		trace_blooms: RwLock<blooms_db::Database>,
		key_value: Arc<KeyValueDB>,
	}

	impl BlockChainDB for RestorationDB {
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

	impl BlockChainDBHandler for RestorationDBHandler {
		fn open(&self, db_path: &Path) -> Result<Arc<BlockChainDB>, Error> {
			let key_value = Arc::new(Database::open(&self.config, &db_path.to_string_lossy())?);
			let blooms_path = db_path.join("blooms");
			let trace_blooms_path = db_path.join("trace_blooms");
			fs::create_dir(&blooms_path)?;
			fs::create_dir(&trace_blooms_path)?;
			let blooms = RwLock::new(blooms_db::Database::open(blooms_path).unwrap());
			let trace_blooms = RwLock::new(blooms_db::Database::open(trace_blooms_path).unwrap());
			let db = RestorationDB {
				blooms,
				trace_blooms,
				key_value,
			};
			Ok(Arc::new(db))
		}
	}

	Box::new(RestorationDBHandler { config })
}
