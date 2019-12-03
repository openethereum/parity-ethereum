// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::time::Duration;

use ethcore::client::DatabaseCompactionProfile;
use spec::SpecParams;
use light::client::fetch::Unavailable as UnavailableDataFetcher;
use light::Cache as LightDataCache;
use types::engines::OptimizeFor;
use params::{SpecType, Pruning};
use helpers::execute_upgrades;
use dir::Directories;
use cache::CacheConfig;
use user_defaults::UserDefaults;
use db;

// Number of minutes before a given gas price corpus should expire.
// Light client only.
const GAS_CORPUS_EXPIRATION_MINUTES: u64 = 60 * 6;

#[derive(Debug, PartialEq)]
pub struct ExportHsyncCmd {
	pub cache_config: CacheConfig,
	pub dirs: Directories,
	pub spec: SpecType,
	pub pruning: Pruning,
	pub compaction: DatabaseCompactionProfile,
}

pub fn execute(cmd: ExportHsyncCmd) -> Result<String, String> {
	use light::client as light_client;
	use parking_lot::Mutex;

	// load spec
	let spec = cmd.spec.spec(SpecParams::new(cmd.dirs.cache.as_ref(), OptimizeFor::Memory))?;

	// load genesis hash
	let genesis_hash = spec.genesis_header().hash();

	// database paths
	let db_dirs = cmd.dirs.database(genesis_hash, cmd.spec.legacy_fork_name(), spec.data_dir.clone());

	// user defaults path
	let user_defaults_path = db_dirs.user_defaults_path();

	// load user defaults
	let user_defaults = UserDefaults::load(&user_defaults_path)?;

	// select pruning algorithm
	let algorithm = cmd.pruning.to_algorithm(&user_defaults);

	// execute upgrades
	execute_upgrades(&cmd.dirs.base, &db_dirs, algorithm, &cmd.compaction)?;

	// create dirs used by parity
	cmd.dirs.create_dirs(false, false)?;

	// TODO: configurable cache size.
	let cache = LightDataCache::new(Default::default(), Duration::from_secs(60 * GAS_CORPUS_EXPIRATION_MINUTES));
	let cache = Arc::new(Mutex::new(cache));

	// start client and create transaction queue.
	let mut config = light_client::Config {
		queue: Default::default(),
		chain_column: ::ethcore_db::COL_LIGHT_CHAIN,
		verify_full: true,
		check_seal: true,
		no_hardcoded_sync: true,
	};

	config.queue.max_mem_use = cmd.cache_config.queue() as usize * 1024 * 1024;

	// initialize database.
	let db = db::open_db_light(
		&db_dirs.client_path(algorithm).to_str().expect("DB path could not be converted to string."),
		&cmd.cache_config,
		&cmd.compaction,
	).map_err(|e| format!("Failed to open database {:?}", e))?;

	let service = light_client::Service::start(config, &spec, UnavailableDataFetcher, db, cache)
		.map_err(|e| format!("Error starting light client: {}", e))?;

	let hs = service.client().read_hardcoded_sync()
		.map_err(|e| format!("Error reading hardcoded sync: {}", e))?;
	if let Some(hs) = hs {
		Ok(hs.to_string())
	} else {
		Err("Error: cannot generate hardcoded sync because the database is empty.".into())
	}
}
