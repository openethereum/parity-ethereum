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

//! Internal helpers for client tests

use std::path::Path;
use std::sync::Arc;
use kvdb::{KeyValueDB, KeyValueDBHandler};
use kvdb_rocksdb::{Database, DatabaseConfig};

/// Creates new instance of KeyValueDBHandler
pub fn restoration_db_handler(config: DatabaseConfig) -> Box<KeyValueDBHandler> {
	use kvdb::Error;

	struct RestorationDBHandler {
		config: DatabaseConfig,
	}

	impl KeyValueDBHandler for RestorationDBHandler {
		fn open(&self, db_path: &Path) -> Result<Arc<KeyValueDB>, Error> {
			Ok(Arc::new(Database::open(&self.config, &db_path.to_string_lossy())?))
		}
	}

	Box::new(RestorationDBHandler { config })
}
