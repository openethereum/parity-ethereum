// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
use std::fs;
use std::time::Duration;
use util::{contents, H256, Address, U256, version_data};
use util::journaldb::Algorithm;
use ethcore::spec::Spec;
use ethcore::ethereum;
use ethcore::miner::{GasPricer, GasPriceCalibratorOptions};
use dir::Directories;

#[derive(Debug, PartialEq)]
pub enum SpecType {
	Mainnet,
	Testnet,
	Ropsten,
	Olympic,
	Classic,
	Custom(String),
}

impl Default for SpecType {
	fn default() -> Self {
		SpecType::Mainnet
	}
}

impl FromStr for SpecType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let spec = match s {
			"frontier" | "homestead" | "mainnet" => SpecType::Mainnet,
			"frontier-dogmatic" | "homestead-dogmatic" | "classic" => SpecType::Classic,
			"morden" | "testnet" => SpecType::Testnet,
			"ropsten" => SpecType::Ropsten,
			"olympic" => SpecType::Olympic,
			other => SpecType::Custom(other.into()),
		};
		Ok(spec)
	}
}

impl SpecType {
	pub fn spec(&self) -> Result<Spec, String> {
		match *self {
			SpecType::Mainnet => Ok(ethereum::new_frontier()),
			SpecType::Testnet => Ok(ethereum::new_morden()),
			SpecType::Ropsten => Ok(ethereum::new_ropsten()),
			SpecType::Olympic => Ok(ethereum::new_olympic()),
			SpecType::Classic => Ok(ethereum::new_classic()),
			SpecType::Custom(ref file) => Ok(Spec::load(&try!(contents(file).map_err(|_| "Could not load specification file."))))
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Pruning {
	Specific(Algorithm),
	Auto,
}

impl Default for Pruning {
	fn default() -> Self {
		Pruning::Auto
	}
}

impl FromStr for Pruning {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"auto" => Ok(Pruning::Auto),
			other => other.parse().map(Pruning::Specific),
		}
	}
}

impl Pruning {
	pub fn to_algorithm(&self, dirs: &Directories, genesis_hash: H256, fork_name: Option<&String>) -> Algorithm {
		match *self {
			Pruning::Specific(algo) => algo,
			Pruning::Auto => Self::find_best_db(dirs, genesis_hash, fork_name),
		}
	}

	fn find_best_db(dirs: &Directories, genesis_hash: H256, fork_name: Option<&String>) -> Algorithm {
		let mut algo_types = Algorithm::all_types();
		// if all dbs have the same modification time, the last element is the default one
		algo_types.push(Algorithm::default());

		algo_types.into_iter().max_by_key(|i| {
			let mut client_path = dirs.client_path(genesis_hash, fork_name, *i);
			client_path.push("CURRENT");
			fs::metadata(&client_path).and_then(|m| m.modified()).ok()
		}).unwrap()
	}
}

#[derive(Debug, PartialEq)]
pub struct ResealPolicy {
	pub own: bool,
	pub external: bool,
}

impl Default for ResealPolicy {
	fn default() -> Self {
		ResealPolicy {
			own: true,
			external: true,
		}
	}
}

impl FromStr for ResealPolicy {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (own, external) = match s {
			"none" => (false, false),
			"own" => (true, false),
			"ext" => (false, true),
			"all" => (true, true),
			x => return Err(format!("Invalid reseal value: {}", x)),
		};

		let reseal = ResealPolicy {
			own: own,
			external: external,
		};

		Ok(reseal)
	}
}

#[derive(Debug, PartialEq)]
pub struct AccountsConfig {
	pub iterations: u32,
	pub import_keys: bool,
	pub testnet: bool,
	pub password_files: Vec<String>,
	pub unlocked_accounts: Vec<Address>,
}

impl Default for AccountsConfig {
	fn default() -> Self {
		AccountsConfig {
			iterations: 10240,
			import_keys: true,
			testnet: false,
			password_files: Vec::new(),
			unlocked_accounts: Vec::new(),
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum GasPricerConfig {
	Fixed(U256),
	Calibrated {
		usd_per_tx: f32,
		recalibration_period: Duration,
	}
}

impl Default for GasPricerConfig {
	fn default() -> Self {
		GasPricerConfig::Calibrated {
			usd_per_tx: 0.005,
			recalibration_period: Duration::from_secs(3600),
		}
	}
}

impl Into<GasPricer> for GasPricerConfig {
	fn into(self) -> GasPricer {
		match self {
			GasPricerConfig::Fixed(u) => GasPricer::Fixed(u),
			GasPricerConfig::Calibrated { usd_per_tx, recalibration_period } => {
				GasPricer::new_calibrated(GasPriceCalibratorOptions {
					usd_per_tx: usd_per_tx,
					recalibration_period: recalibration_period,
				})
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct MinerExtras {
	pub author: Address,
	pub extra_data: Vec<u8>,
	pub gas_floor_target: U256,
	pub gas_ceil_target: U256,
	pub transactions_limit: usize,
}

impl Default for MinerExtras {
	fn default() -> Self {
		MinerExtras {
			author: Default::default(),
			extra_data: version_data(),
			gas_floor_target: U256::from(4_700_000),
			gas_ceil_target: U256::from(6_283_184),
			transactions_limit: 2048,
		}
	}
}

#[cfg(test)]
mod tests {
	use util::journaldb::Algorithm;
	use super::{SpecType, Pruning, ResealPolicy};

	#[test]
	fn test_spec_type_parsing() {
		assert_eq!(SpecType::Mainnet, "frontier".parse().unwrap());
		assert_eq!(SpecType::Mainnet, "homestead".parse().unwrap());
		assert_eq!(SpecType::Mainnet, "mainnet".parse().unwrap());
		assert_eq!(SpecType::Testnet, "testnet".parse().unwrap());
		assert_eq!(SpecType::Testnet, "morden".parse().unwrap());
		assert_eq!(SpecType::Ropsten, "ropsten".parse().unwrap());
		assert_eq!(SpecType::Olympic, "olympic".parse().unwrap());
	}

	#[test]
	fn test_spec_type_default() {
		assert_eq!(SpecType::Mainnet, SpecType::default());
	}

	#[test]
	fn test_pruning_parsing() {
		assert_eq!(Pruning::Auto, "auto".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::Archive), "archive".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::EarlyMerge), "light".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::OverlayRecent), "fast".parse().unwrap());
		assert_eq!(Pruning::Specific(Algorithm::RefCounted), "basic".parse().unwrap());
	}

	#[test]
	fn test_pruning_default() {
		assert_eq!(Pruning::Auto, Pruning::default());
	}

	#[test]
	fn test_reseal_policy_parsing() {
		let none = ResealPolicy { own: false, external: false };
		let own = ResealPolicy { own: true, external: false };
		let ext = ResealPolicy { own: false, external: true };
		let all = ResealPolicy { own: true, external: true };
		assert_eq!(none, "none".parse().unwrap());
		assert_eq!(own, "own".parse().unwrap());
		assert_eq!(ext, "ext".parse().unwrap());
		assert_eq!(all, "all".parse().unwrap());
	}

	#[test]
	fn test_reseal_policy_default() {
		let all = ResealPolicy { own: true, external: true };
		assert_eq!(all, ResealPolicy::default());
	}
}
