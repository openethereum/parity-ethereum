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

use std::{str, fs, fmt};
use std::time::Duration;

use ethcore::client::Mode;
use ethcore::ethereum;
use ethcore::spec::{Spec, SpecParams};
use ethereum_types::{U256, Address};
use parity_runtime::Executor;
use hash_fetch::fetch::Client as FetchClient;
use journaldb::Algorithm;
use miner::gas_pricer::GasPricer;
use miner::gas_price_calibrator::{GasPriceCalibratorOptions, GasPriceCalibrator};
use parity_version::version_data;
use user_defaults::UserDefaults;

#[derive(Debug, PartialEq)]
pub enum SpecType {
	Foundation,
	Classic,
	Poanet,
	Tobalaba,
	Expanse,
	Musicoin,
	Ellaism,
	Easthub,
	Social,
	Callisto,
	Morden,
	Ropsten,
	Kovan,
	Sokol,
	Dev,
	Custom(String),
}

impl Default for SpecType {
	fn default() -> Self {
		SpecType::Foundation
	}
}

impl str::FromStr for SpecType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let spec = match s {
			"ethereum" | "frontier" | "homestead" | "byzantium" | "foundation" | "mainnet" => SpecType::Foundation,
			"classic" | "frontier-dogmatic" | "homestead-dogmatic" => SpecType::Classic,
			"poanet" | "poacore" => SpecType::Poanet,
			"tobalaba" => SpecType::Tobalaba,
			"expanse" => SpecType::Expanse,
			"musicoin" => SpecType::Musicoin,
			"ellaism" => SpecType::Ellaism,
			"easthub" => SpecType::Easthub,
			"social" => SpecType::Social,
			"callisto" => SpecType::Callisto,
			"morden" | "classic-testnet" => SpecType::Morden,
			"ropsten" => SpecType::Ropsten,
			"kovan" | "testnet" => SpecType::Kovan,
			"sokol" | "poasokol" => SpecType::Sokol,
			"dev" => SpecType::Dev,
			other => SpecType::Custom(other.into()),
		};
		Ok(spec)
	}
}

impl fmt::Display for SpecType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match *self {
			SpecType::Foundation => "foundation",
			SpecType::Classic => "classic",
			SpecType::Poanet => "poanet",
			SpecType::Tobalaba => "tobalaba",
			SpecType::Expanse => "expanse",
			SpecType::Musicoin => "musicoin",
			SpecType::Ellaism => "ellaism",
			SpecType::Easthub => "easthub",
			SpecType::Social => "social",
			SpecType::Callisto => "callisto",
			SpecType::Morden => "morden",
			SpecType::Ropsten => "ropsten",
			SpecType::Kovan => "kovan",
			SpecType::Sokol => "sokol",
			SpecType::Dev => "dev",
			SpecType::Custom(ref custom) => custom,
		})
	}
}

impl SpecType {
	pub fn spec<'a, T: Into<SpecParams<'a>>>(&self, params: T) -> Result<Spec, String> {
		let params = params.into();
		match *self {
			SpecType::Foundation => Ok(ethereum::new_foundation(params)),
			SpecType::Classic => Ok(ethereum::new_classic(params)),
			SpecType::Poanet => Ok(ethereum::new_poanet(params)),
			SpecType::Tobalaba => Ok(ethereum::new_tobalaba(params)),
			SpecType::Expanse => Ok(ethereum::new_expanse(params)),
			SpecType::Musicoin => Ok(ethereum::new_musicoin(params)),
			SpecType::Ellaism => Ok(ethereum::new_ellaism(params)),
			SpecType::Easthub => Ok(ethereum::new_easthub(params)),
			SpecType::Social => Ok(ethereum::new_social(params)),
			SpecType::Callisto => Ok(ethereum::new_callisto(params)),
			SpecType::Morden => Ok(ethereum::new_morden(params)),
			SpecType::Ropsten => Ok(ethereum::new_ropsten(params)),
			SpecType::Kovan => Ok(ethereum::new_kovan(params)),
			SpecType::Sokol => Ok(ethereum::new_sokol(params)),
			SpecType::Dev => Ok(Spec::new_instant()),
			SpecType::Custom(ref filename) => {
				let file = fs::File::open(filename).map_err(|e| format!("Could not load specification file at {}: {}", filename, e))?;
				Spec::load(params, file)
			}
		}
	}

	pub fn legacy_fork_name(&self) -> Option<String> {
		match *self {
			SpecType::Classic => Some("classic".to_owned()),
			SpecType::Expanse => Some("expanse".to_owned()),
			SpecType::Musicoin => Some("musicoin".to_owned()),
			_ => None,
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

impl str::FromStr for Pruning {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"auto" => Ok(Pruning::Auto),
			other => other.parse().map(Pruning::Specific),
		}
	}
}

impl Pruning {
	pub fn to_algorithm(&self, user_defaults: &UserDefaults) -> Algorithm {
		match *self {
			Pruning::Specific(algo) => algo,
			Pruning::Auto => user_defaults.pruning,
		}
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

impl str::FromStr for ResealPolicy {
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
	pub refresh_time: u64,
	pub testnet: bool,
	pub password_files: Vec<String>,
	pub unlocked_accounts: Vec<Address>,
	pub enable_hardware_wallets: bool,
	pub enable_fast_unlock: bool,
}

impl Default for AccountsConfig {
	fn default() -> Self {
		AccountsConfig {
			iterations: 10240,
			refresh_time: 5,
			testnet: false,
			password_files: Vec::new(),
			unlocked_accounts: Vec::new(),
			enable_hardware_wallets: true,
			enable_fast_unlock: false,
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
			usd_per_tx: 0.0001f32,
			recalibration_period: Duration::from_secs(3600),
		}
	}
}

impl GasPricerConfig {
	pub fn to_gas_pricer(&self, fetch: FetchClient, p: Executor) -> GasPricer {
		match *self {
			GasPricerConfig::Fixed(u) => GasPricer::Fixed(u),
			GasPricerConfig::Calibrated { usd_per_tx, recalibration_period, .. } => {
				GasPricer::new_calibrated(
					GasPriceCalibrator::new(
						GasPriceCalibratorOptions {
							usd_per_tx: usd_per_tx,
							recalibration_period: recalibration_period,
						},
						fetch,
						p,
					)
				)
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct MinerExtras {
	pub author: Address,
	pub engine_signer: Address,
	pub extra_data: Vec<u8>,
	pub gas_range_target: (U256, U256),
	pub work_notify: Vec<String>,
}

impl Default for MinerExtras {
	fn default() -> Self {
		MinerExtras {
			author: Default::default(),
			engine_signer: Default::default(),
			extra_data: version_data(),
			gas_range_target: (8_000_000.into(), 10_000_000.into()),
			work_notify: Default::default(),
		}
	}
}

/// 3-value enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Switch {
	/// True.
	On,
	/// False.
	Off,
	/// Auto.
	Auto,
}

impl Default for Switch {
	fn default() -> Self {
		Switch::Auto
	}
}

impl str::FromStr for Switch {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"on" => Ok(Switch::On),
			"off" => Ok(Switch::Off),
			"auto" => Ok(Switch::Auto),
			other => Err(format!("Invalid switch value: {}", other))
		}
	}
}

pub fn tracing_switch_to_bool(switch: Switch, user_defaults: &UserDefaults) -> Result<bool, String> {
	match (user_defaults.is_first_launch, switch, user_defaults.tracing) {
		(false, Switch::On, false) => Err("TraceDB resync required".into()),
		(_, Switch::On, _) => Ok(true),
		(_, Switch::Off, _) => Ok(false),
		(_, Switch::Auto, def) => Ok(def),
	}
}

pub fn fatdb_switch_to_bool(switch: Switch, user_defaults: &UserDefaults, _algorithm: Algorithm) -> Result<bool, String> {
	let result = match (user_defaults.is_first_launch, switch, user_defaults.fat_db) {
		(false, Switch::On, false) => Err("FatDB resync required".into()),
		(_, Switch::On, _) => Ok(true),
		(_, Switch::Off, _) => Ok(false),
		(_, Switch::Auto, def) => Ok(def),
	};
	result
}

pub fn mode_switch_to_bool(switch: Option<Mode>, user_defaults: &UserDefaults) -> Result<Mode, String> {
	Ok(switch.unwrap_or(user_defaults.mode().clone()))
}

#[cfg(test)]
mod tests {
	use journaldb::Algorithm;
	use user_defaults::UserDefaults;
	use super::{SpecType, Pruning, ResealPolicy, Switch, tracing_switch_to_bool};

	#[test]
	fn test_spec_type_parsing() {
		assert_eq!(SpecType::Foundation, "foundation".parse().unwrap());
		assert_eq!(SpecType::Foundation, "frontier".parse().unwrap());
		assert_eq!(SpecType::Foundation, "homestead".parse().unwrap());
		assert_eq!(SpecType::Foundation, "byzantium".parse().unwrap());
		assert_eq!(SpecType::Foundation, "mainnet".parse().unwrap());
		assert_eq!(SpecType::Foundation, "ethereum".parse().unwrap());
		assert_eq!(SpecType::Classic, "classic".parse().unwrap());
		assert_eq!(SpecType::Classic, "frontier-dogmatic".parse().unwrap());
		assert_eq!(SpecType::Classic, "homestead-dogmatic".parse().unwrap());
		assert_eq!(SpecType::Poanet, "poanet".parse().unwrap());
		assert_eq!(SpecType::Poanet, "poacore".parse().unwrap());
		assert_eq!(SpecType::Tobalaba, "tobalaba".parse().unwrap());
		assert_eq!(SpecType::Expanse, "expanse".parse().unwrap());
		assert_eq!(SpecType::Musicoin, "musicoin".parse().unwrap());
		assert_eq!(SpecType::Ellaism, "ellaism".parse().unwrap());
		assert_eq!(SpecType::Easthub, "easthub".parse().unwrap());
		assert_eq!(SpecType::Social, "social".parse().unwrap());
		assert_eq!(SpecType::Callisto, "callisto".parse().unwrap());
		assert_eq!(SpecType::Morden, "morden".parse().unwrap());
		assert_eq!(SpecType::Morden, "classic-testnet".parse().unwrap());
		assert_eq!(SpecType::Ropsten, "ropsten".parse().unwrap());
		assert_eq!(SpecType::Kovan, "kovan".parse().unwrap());
		assert_eq!(SpecType::Kovan, "testnet".parse().unwrap());
		assert_eq!(SpecType::Sokol, "sokol".parse().unwrap());
		assert_eq!(SpecType::Sokol, "poasokol".parse().unwrap());
	}

	#[test]
	fn test_spec_type_default() {
		assert_eq!(SpecType::Foundation, SpecType::default());
	}

	#[test]
	fn test_spec_type_display() {
		assert_eq!(format!("{}", SpecType::Foundation), "foundation");
		assert_eq!(format!("{}", SpecType::Classic), "classic");
		assert_eq!(format!("{}", SpecType::Poanet), "poanet");
		assert_eq!(format!("{}", SpecType::Tobalaba), "tobalaba");
		assert_eq!(format!("{}", SpecType::Expanse), "expanse");
		assert_eq!(format!("{}", SpecType::Musicoin), "musicoin");
		assert_eq!(format!("{}", SpecType::Ellaism), "ellaism");
		assert_eq!(format!("{}", SpecType::Easthub), "easthub");
		assert_eq!(format!("{}", SpecType::Social), "social");
		assert_eq!(format!("{}", SpecType::Callisto), "callisto");
		assert_eq!(format!("{}", SpecType::Morden), "morden");
		assert_eq!(format!("{}", SpecType::Ropsten), "ropsten");
		assert_eq!(format!("{}", SpecType::Kovan), "kovan");
		assert_eq!(format!("{}", SpecType::Sokol), "sokol");
		assert_eq!(format!("{}", SpecType::Dev), "dev");
		assert_eq!(format!("{}", SpecType::Custom("foo/bar".into())), "foo/bar");
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

	#[test]
	fn test_switch_parsing() {
		assert_eq!(Switch::On, "on".parse().unwrap());
		assert_eq!(Switch::Off, "off".parse().unwrap());
		assert_eq!(Switch::Auto, "auto".parse().unwrap());
	}

	#[test]
	fn test_switch_default() {
		assert_eq!(Switch::default(), Switch::Auto);
	}

	fn user_defaults_with_tracing(first_launch: bool, tracing: bool) -> UserDefaults {
		let mut ud = UserDefaults::default();
		ud.is_first_launch = first_launch;
		ud.tracing = tracing;
		ud
	}

	#[test]
	fn test_switch_to_bool() {
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(true, true)).unwrap());
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(true, false)).unwrap());
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(false, true)).unwrap());
		assert!(!tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(false, false)).unwrap());

		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(true, true)).unwrap());
		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(true, false)).unwrap());
		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(false, true)).unwrap());
		assert!(tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(false, false)).is_err());
	}
}
