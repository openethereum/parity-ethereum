use std::path::Path;
use ethcore::db::NUM_COLUMNS;
use ethcore::client::{ClientConfig, DatabaseCompactionProfile};
use super::kvdb_rocksdb::{CompactionProfile, DatabaseConfig};

pub fn compaction_profile(profile: &DatabaseCompactionProfile, db_path: &Path) -> CompactionProfile {
	match profile {
		&DatabaseCompactionProfile::Auto => CompactionProfile::auto(db_path),
		&DatabaseCompactionProfile::SSD => CompactionProfile::ssd(),
		&DatabaseCompactionProfile::HDD => CompactionProfile::hdd(),
	}
}

pub fn client_db_config(client_path: &Path, client_config: &ClientConfig) -> DatabaseConfig {
	let mut client_db_config = DatabaseConfig::with_columns(NUM_COLUMNS);

	client_db_config.memory_budget = client_config.db_cache_size;
	client_db_config.compaction = compaction_profile(&client_config.db_compaction, &client_path);
	client_db_config.wal = client_config.db_wal;

	client_db_config
}
