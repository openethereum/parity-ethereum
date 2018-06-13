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

use std::str::FromStr;
use std::fmt::{Display, Formatter, Error as FmtError};

use verification::{VerifierType, QueueConfig};
use journaldb;

pub use std::time::Duration;
pub use blockchain::Config as BlockChainConfig;
pub use trace::Config as TraceConfig;
pub use evm::VMType;

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

/// Operating mode for the client.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Mode {
	/// Always on.
	Active,
	/// Goes offline after client is inactive for some (given) time, but
	/// comes back online after a while of inactivity.
	Passive(Duration, Duration),
	/// Goes offline after client is inactive for some (given) time and
	/// stays inactive.
	Dark(Duration),
	/// Always off.
	Off,
}

impl Default for Mode {
	fn default() -> Self {
		Mode::Active
	}
}

impl Display for Mode {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			Mode::Active => write!(f, "active"),
			Mode::Passive(..) => write!(f, "passive"),
			Mode::Dark(..) => write!(f, "dark"),
			Mode::Off => write!(f, "offline"),
		}
	}
}

/// Client configuration. Includes configs for all sub-systems.
#[derive(Debug, PartialEq, Default, Clone)]
pub struct ClientConfig {
	/// Block queue configuration.
	pub queue: QueueConfig,
	/// Blockchain configuration.
	pub blockchain: BlockChainConfig,
	/// Trace configuration.
	pub tracing: TraceConfig,
	/// VM type.
	pub vm_type: VMType,
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
	/// Should db have WAL enabled?
	pub db_wal: bool,
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
	/// Check seal valididity on block import
	pub check_seal: bool,
}

#[cfg(test)]
mod test {
	use super::{DatabaseCompactionProfile, Mode};

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

	#[test]
	fn test_mode_default() {
		assert_eq!(Mode::default(), Mode::Active);
	}
}
