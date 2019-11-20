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

use std::str::FromStr;

use blockchain::Config as BlockChainConfig;
use journaldb;
use snapshot::SnapshotConfiguration;
use trace::Config as TraceConfig;
use types::client_types::Mode;
use verification::{VerifierType, QueueConfig};

/// Client state db compaction profile
#[derive(Debug, PartialEq, Clone)]
pub enum DatabaseCompactionProfile {
	/// Try to determine compaction profile automatically
	Auto,
	/// SSD compaction profile
	SSD,
	/// HDD or other slow storage io compaction profile
	HDD,
}

impl Default for DatabaseCompactionProfile {
	fn default() -> Self {
		DatabaseCompactionProfile::Auto
	}
}

impl FromStr for DatabaseCompactionProfile {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"auto" => Ok(DatabaseCompactionProfile::Auto),
			"ssd" => Ok(DatabaseCompactionProfile::SSD),
			"hdd" => Ok(DatabaseCompactionProfile::HDD),
			_ => Err("Invalid compaction profile given. Expected default/hdd/ssd.".into()),
		}
	}
}

/// Client configuration. Includes configs for all sub-systems.
#[derive(Debug, PartialEq, Clone)]
pub struct ClientConfig {
	/// Block queue configuration.
	pub queue: QueueConfig,
	/// Blockchain configuration.
	pub blockchain: BlockChainConfig,
	/// Trace configuration.
	pub tracing: TraceConfig,
	/// Fat DB enabled?
	pub fat_db: bool,
	/// The JournalDB ("pruning") algorithm to use.
	pub pruning: journaldb::Algorithm,
	/// The name of the client instance.
	pub name: String,
	/// RocksDB column cache-size if not default
	pub db_cache_size: Option<usize>,
	/// State db compaction profile
	pub db_compaction: DatabaseCompactionProfile,
	/// Operating mode
	pub mode: Mode,
	/// The chain spec name
	pub spec_name: String,
	/// Type of block verifier used by client.
	pub verifier_type: VerifierType,
	/// State db cache-size.
	pub state_cache_size: usize,
	/// EVM jump-tables cache size.
	pub jump_table_size: usize,
	/// Minimum state pruning history size.
	pub history: u64,
	/// Ideal memory usage for state pruning history.
	pub history_mem: usize,
	/// Check seal validity on block import
	pub check_seal: bool,
	/// Maximal number of transactions queued for verification in a separate thread.
	pub transaction_verification_queue_size: usize,
	/// Maximal number of blocks to import at each round.
	pub max_round_blocks_to_import: usize,
	/// Snapshot configuration
	pub snapshot: SnapshotConfiguration,
}

impl Default for ClientConfig {
	fn default() -> Self {
		let mb = 1024 * 1024;
		ClientConfig {
			queue: Default::default(),
			blockchain: Default::default(),
			tracing: Default::default(),
			fat_db: false,
			pruning: journaldb::Algorithm::OverlayRecent,
			name: "default".into(),
			db_cache_size: None,
			db_compaction: Default::default(),
			mode: Mode::Active,
			spec_name: "".into(),
			verifier_type: VerifierType::Canon,
			state_cache_size: 1 * mb,
			jump_table_size: 1 * mb,
			history: 64,
			history_mem: 32 * mb,
			check_seal: true,
			transaction_verification_queue_size: 8192,
			max_round_blocks_to_import: 12,
			snapshot: Default::default(),
		}
	}
}
#[cfg(test)]
mod test {
	use super::DatabaseCompactionProfile;

	#[test]
	fn test_default_compaction_profile() {
		assert_eq!(DatabaseCompactionProfile::default(), DatabaseCompactionProfile::Auto);
	}

	#[test]
	fn test_parsing_compaction_profile() {
		assert_eq!(DatabaseCompactionProfile::Auto, "auto".parse().unwrap());
		assert_eq!(DatabaseCompactionProfile::SSD, "ssd".parse().unwrap());
		assert_eq!(DatabaseCompactionProfile::HDD, "hdd".parse().unwrap());
	}
}
