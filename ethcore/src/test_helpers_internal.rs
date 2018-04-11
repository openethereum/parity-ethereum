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
