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

use std::{io, env};
use std::io::{Write, BufReader, BufRead};
use std::time::Duration;
use std::fs::File;
use bigint::prelude::U256;
use bigint::hash::clean_0x;
use util::Address;
use kvdb_rocksdb::CompactionProfile;
use journaldb::Algorithm;
use ethcore::client::{Mode, BlockId, VMType, DatabaseCompactionProfile, ClientConfig, VerifierType};
use ethcore::miner::{PendingSet, GasLimit, PrioritizationStrategy};
use cache::CacheConfig;
use dir::DatabaseDirectories;
use upgrade::{upgrade, upgrade_data_paths};
use migration::migrate;
use ethsync::{validate_node_url, self};
use path;

pub fn to_duration(s: &str) -> Result<Duration, String> {
	to_seconds(s).map(Duration::from_secs)
}

fn to_seconds(s: &str) -> Result<u64, String> {
	let bad = |_| {
		format!("{}: Invalid duration given. See parity --help for more information.", s)
	};

	match s {
		"twice-daily" => Ok(12 * 60 * 60),
		"half-hourly" => Ok(30 * 60),
		"1second" | "1 second" | "second" => Ok(1),
		"1minute" | "1 minute" | "minute" => Ok(60),
		"hourly" | "1hour" | "1 hour" | "hour" => Ok(60 * 60),
		"daily" | "1day" | "1 day" | "day" => Ok(24 * 60 * 60),
		x if x.ends_with("seconds") => x[0..x.len() - 7].parse().map_err(bad),
		x if x.ends_with("minutes") => x[0..x.len() - 7].parse::<u64>().map_err(bad).map(|x| x * 60),
		x if x.ends_with("hours") => x[0..x.len() - 5].parse::<u64>().map_err(bad).map(|x| x * 60 * 60),
		x if x.ends_with("days") => x[0..x.len() - 4].parse::<u64>().map_err(bad).map(|x| x * 24 * 60 * 60),
		x => x.parse().map_err(bad),
	}
}

pub fn to_mode(s: &str, timeout: u64, alarm: u64) -> Result<Mode, String> {
	match s {
		"active" => Ok(Mode::Active),
		"passive" => Ok(Mode::Passive(Duration::from_secs(timeout), Duration::from_secs(alarm))),
		"dark" => Ok(Mode::Dark(Duration::from_secs(timeout))),
		"offline" => Ok(Mode::Off),
		_ => Err(format!("{}: Invalid value for --mode. Must be one of active, passive, dark or offline.", s)),
	}
}

pub fn to_block_id(s: &str) -> Result<BlockId, String> {
	if s == "latest" {
		Ok(BlockId::Latest)
	} else if let Ok(num) = s.parse() {
		Ok(BlockId::Number(num))
	} else if let Ok(hash) = s.parse() {
		Ok(BlockId::Hash(hash))
	} else {
		Err("Invalid block.".into())
	}
}

pub fn to_u256(s: &str) -> Result<U256, String> {
	if let Ok(decimal) = U256::from_dec_str(s) {
		Ok(decimal)
	} else if let Ok(hex) = clean_0x(s).parse() {
		Ok(hex)
	} else {
		Err(format!("Invalid numeric value: {}", s))
	}
}

pub fn to_pending_set(s: &str) -> Result<PendingSet, String> {
	match s {
		"cheap" => Ok(PendingSet::AlwaysQueue),
		"strict" => Ok(PendingSet::AlwaysSealing),
		"lenient" => Ok(PendingSet::SealingOrElseQueue),
		other => Err(format!("Invalid pending set value: {:?}", other)),
	}
}

pub fn to_gas_limit(s: &str) -> Result<GasLimit, String> {
	match s {
		"auto" => Ok(GasLimit::Auto),
		"off" => Ok(GasLimit::None),
		other => Ok(GasLimit::Fixed(to_u256(other)?)),
	}
}

pub fn to_queue_strategy(s: &str) -> Result<PrioritizationStrategy, String> {
	match s {
		"gas" => Ok(PrioritizationStrategy::GasAndGasPrice),
		"gas_price" => Ok(PrioritizationStrategy::GasPriceOnly),
		"gas_factor" => Ok(PrioritizationStrategy::GasFactorAndGasPrice),
		other => Err(format!("Invalid queue strategy: {}", other)),
	}
}

pub fn to_address(s: Option<String>) -> Result<Address, String> {
	match s {
		Some(ref a) => clean_0x(a).parse().map_err(|_| format!("Invalid address: {:?}", a)),
		None => Ok(Address::default())
	}
}

pub fn to_addresses(s: &Option<String>) -> Result<Vec<Address>, String> {
	match *s {
		Some(ref adds) if !adds.is_empty() => adds.split(',')
			.map(|a| clean_0x(a).parse().map_err(|_| format!("Invalid address: {:?}", a)))
			.collect(),
		_ => Ok(Vec::new()),
	}
}

/// Tries to parse string as a price.
pub fn to_price(s: &str) -> Result<f32, String> {
	s.parse::<f32>().map_err(|_| format!("Invalid transaciton price 's' given. Must be a decimal number."))
}

/// Replaces `$HOME` str with home directory path.
pub fn replace_home(base: &str, arg: &str) -> String {
	// the $HOME directory on mac os should be `~/Library` or `~/Library/Application Support`
	let r = arg.replace("$HOME", env::home_dir().unwrap().to_str().unwrap());
	let r = r.replace("$BASE", base);
	r.replace("/", &::std::path::MAIN_SEPARATOR.to_string())
}

pub fn replace_home_and_local(base: &str, local: &str, arg: &str) -> String {
	let r = replace_home(base, arg);
	r.replace("$LOCAL", local)
}

/// Flush output buffer.
pub fn flush_stdout() {
	io::stdout().flush().expect("stdout is flushable; qed");
}

/// Returns default geth ipc path.
pub fn geth_ipc_path(testnet: bool) -> String {
	// Windows path should not be hardcoded here.
	// Instead it should be a part of path::ethereum
	if cfg!(windows) {
		return r"\\.\pipe\geth.ipc".to_owned();
	}

	if testnet {
		path::ethereum::with_testnet("geth.ipc").to_str().unwrap().to_owned()
	} else {
		path::ethereum::with_default("geth.ipc").to_str().unwrap().to_owned()
	}
}

/// Formats and returns parity ipc path.
pub fn parity_ipc_path(base: &str, path: &str, shift: u16) -> String {
	let mut path = path.to_owned();
	if shift != 0 {
		path = path.replace("jsonrpc.ipc", &format!("jsonrpc-{}.ipc", shift));
	}
	replace_home(base, &path)
}

/// Validates and formats bootnodes option.
pub fn to_bootnodes(bootnodes: &Option<String>) -> Result<Vec<String>, String> {
	match *bootnodes {
		Some(ref x) if !x.is_empty() => x.split(',').map(|s| {
			match validate_node_url(s).map(Into::into) {
				None => Ok(s.to_owned()),
				Some(ethsync::ErrorKind::AddressResolve(_)) => Err(format!("Failed to resolve hostname of a boot node: {}", s)),
				Some(_) => Err(format!("Invalid node address format given for a boot node: {}", s)),
			}
		}).collect(),
		Some(_) => Ok(vec![]),
		None => Ok(vec![])
	}
}

#[cfg(test)]
pub fn default_network_config() -> ::ethsync::NetworkConfiguration {
	use ethsync::{NetworkConfiguration};
	use super::network::IpFilter;
	NetworkConfiguration {
		config_path: Some(replace_home(&::dir::default_data_path(), "$BASE/network")),
		net_config_path: None,
		listen_address: Some("0.0.0.0:30303".into()),
		public_address: None,
		udp_port: None,
		nat_enabled: true,
		discovery_enabled: true,
		boot_nodes: Vec::new(),
		use_secret: None,
		max_peers: 50,
		min_peers: 25,
		snapshot_peers: 0,
		max_pending_peers: 64,
		ip_filter: IpFilter::default(),
		reserved_nodes: Vec::new(),
		allow_non_reserved: true,
	}
}

#[cfg_attr(feature = "dev", allow(too_many_arguments))]
pub fn to_client_config(
		cache_config: &CacheConfig,
		spec_name: String,
		mode: Mode,
		tracing: bool,
		fat_db: bool,
		compaction: DatabaseCompactionProfile,
		wal: bool,
		vm_type: VMType,
		name: String,
		pruning: Algorithm,
		pruning_history: u64,
		pruning_memory: usize,
		check_seal: bool,
	) -> ClientConfig {
	let mut client_config = ClientConfig::default();

	let mb = 1024 * 1024;
	// in bytes
	client_config.blockchain.max_cache_size = cache_config.blockchain() as usize * mb;
	// in bytes
	client_config.blockchain.pref_cache_size = cache_config.blockchain() as usize * 3 / 4 * mb;
	// db blockchain cache size, in megabytes
	client_config.blockchain.db_cache_size = Some(cache_config.db_blockchain_cache_size() as usize);
	// db state cache size, in megabytes
	client_config.db_cache_size = Some(cache_config.db_state_cache_size() as usize);
	// db queue cache size, in bytes
	client_config.queue.max_mem_use = cache_config.queue() as usize * mb;
	// in bytes
	client_config.tracing.max_cache_size = cache_config.traces() as usize * mb;
	// in bytes
	client_config.tracing.pref_cache_size = cache_config.traces() as usize * 3 / 4 * mb;
	// in bytes
	client_config.state_cache_size = cache_config.state() as usize * mb;
	// in bytes
	client_config.jump_table_size = cache_config.jump_tables() as usize * mb;
	// in bytes
	client_config.history_mem = pruning_memory * mb;

	client_config.mode = mode;
	client_config.tracing.enabled = tracing;
	client_config.fat_db = fat_db;
	client_config.pruning = pruning;
	client_config.history = pruning_history;
	client_config.db_compaction = compaction;
	client_config.db_wal = wal;
	client_config.vm_type = vm_type;
	client_config.name = name;
	client_config.verifier_type = if check_seal { VerifierType::Canon } else { VerifierType::CanonNoSeal };
	client_config.spec_name = spec_name;
	client_config
}

pub fn execute_upgrades(
	base_path: &str,
	dirs: &DatabaseDirectories,
	pruning: Algorithm,
	compaction_profile: CompactionProfile
) -> Result<(), String> {

	upgrade_data_paths(base_path, dirs, pruning);

	match upgrade(Some(&dirs.path)) {
		Ok(upgrades_applied) if upgrades_applied > 0 => {
			debug!("Executed {} upgrade scripts - ok", upgrades_applied);
		},
		Err(e) => {
			return Err(format!("Error upgrading parity data: {:?}", e));
		},
		_ => {},
	}

	let client_path = dirs.db_path(pruning);
	migrate(&client_path, pruning, compaction_profile).map_err(|e| format!("{}", e))
}

/// Prompts user asking for password.
pub fn password_prompt() -> Result<String, String> {
	use rpassword::read_password;
	const STDIN_ERROR: &'static str = "Unable to ask for password on non-interactive terminal.";

	println!("Please note that password is NOT RECOVERABLE.");
	print!("Type password: ");
	flush_stdout();

	let password = read_password().map_err(|_| STDIN_ERROR.to_owned())?;

	print!("Repeat password: ");
	flush_stdout();

	let password_repeat = read_password().map_err(|_| STDIN_ERROR.to_owned())?;

	if password != password_repeat {
		return Err("Passwords do not match!".into());
	}

	Ok(password)
}

/// Read a password from password file.
pub fn password_from_file(path: String) -> Result<String, String> {
	let passwords = passwords_from_files(&[path])?;
	// use only first password from the file
	passwords.get(0).map(String::to_owned)
		.ok_or_else(|| "Password file seems to be empty.".to_owned())
}

/// Reads passwords from files. Treats each line as a separate password.
pub fn passwords_from_files(files: &[String]) -> Result<Vec<String>, String> {
	let passwords = files.iter().map(|filename| {
		let file = File::open(filename).map_err(|_| format!("{} Unable to read password file. Ensure it exists and permissions are correct.", filename))?;
		let reader = BufReader::new(&file);
		let lines = reader.lines()
			.filter_map(|l| l.ok())
			.map(|pwd| pwd.trim().to_owned())
			.collect::<Vec<String>>();
		Ok(lines)
	}).collect::<Result<Vec<Vec<String>>, String>>();
	Ok(passwords?.into_iter().flat_map(|x| x).collect())
}

#[cfg(test)]
mod tests {
	use std::time::Duration;
	use std::fs::File;
	use std::io::Write;
	use devtools::RandomTempPath;
	use bigint::prelude::U256;
	use ethcore::client::{Mode, BlockId};
	use ethcore::miner::PendingSet;
	use super::{to_duration, to_mode, to_block_id, to_u256, to_pending_set, to_address, to_addresses, to_price, geth_ipc_path, to_bootnodes, password_from_file};

	#[test]
	fn test_to_duration() {
		assert_eq!(to_duration("twice-daily").unwrap(), Duration::from_secs(12 * 60 * 60));
		assert_eq!(to_duration("half-hourly").unwrap(), Duration::from_secs(30 * 60));
		assert_eq!(to_duration("1second").unwrap(), Duration::from_secs(1));
		assert_eq!(to_duration("2seconds").unwrap(), Duration::from_secs(2));
		assert_eq!(to_duration("15seconds").unwrap(), Duration::from_secs(15));
		assert_eq!(to_duration("1minute").unwrap(), Duration::from_secs(1 * 60));
		assert_eq!(to_duration("2minutes").unwrap(), Duration::from_secs(2 * 60));
		assert_eq!(to_duration("15minutes").unwrap(), Duration::from_secs(15 * 60));
		assert_eq!(to_duration("hourly").unwrap(), Duration::from_secs(60 * 60));
		assert_eq!(to_duration("daily").unwrap(), Duration::from_secs(24 * 60 * 60));
		assert_eq!(to_duration("1hour").unwrap(), Duration::from_secs(1 * 60 * 60));
		assert_eq!(to_duration("2hours").unwrap(), Duration::from_secs(2 * 60 * 60));
		assert_eq!(to_duration("15hours").unwrap(), Duration::from_secs(15 * 60 * 60));
		assert_eq!(to_duration("1day").unwrap(), Duration::from_secs(1 * 24 * 60 * 60));
		assert_eq!(to_duration("2days").unwrap(), Duration::from_secs(2 * 24 *60 * 60));
		assert_eq!(to_duration("15days").unwrap(), Duration::from_secs(15 * 24 * 60 * 60));
	}

	#[test]
	fn test_to_mode() {
		assert_eq!(to_mode("active", 0, 0).unwrap(), Mode::Active);
		assert_eq!(to_mode("passive", 10, 20).unwrap(), Mode::Passive(Duration::from_secs(10), Duration::from_secs(20)));
		assert_eq!(to_mode("dark", 20, 30).unwrap(), Mode::Dark(Duration::from_secs(20)));
		assert!(to_mode("other", 20, 30).is_err());
	}

	#[test]
	fn test_to_block_id() {
		assert_eq!(to_block_id("latest").unwrap(), BlockId::Latest);
		assert_eq!(to_block_id("0").unwrap(), BlockId::Number(0));
		assert_eq!(to_block_id("2").unwrap(), BlockId::Number(2));
		assert_eq!(to_block_id("15").unwrap(), BlockId::Number(15));
		assert_eq!(
			to_block_id("9fc84d84f6a785dc1bd5abacfcf9cbdd3b6afb80c0f799bfb2fd42c44a0c224e").unwrap(),
			BlockId::Hash("9fc84d84f6a785dc1bd5abacfcf9cbdd3b6afb80c0f799bfb2fd42c44a0c224e".parse().unwrap())
		);
	}

	#[test]
	fn test_to_u256() {
		assert_eq!(to_u256("0").unwrap(), U256::from(0));
		assert_eq!(to_u256("11").unwrap(), U256::from(11));
		assert_eq!(to_u256("0x11").unwrap(), U256::from(17));
		assert!(to_u256("u").is_err())
	}

	#[test]
	fn test_pending_set() {
		assert_eq!(to_pending_set("cheap").unwrap(), PendingSet::AlwaysQueue);
		assert_eq!(to_pending_set("strict").unwrap(), PendingSet::AlwaysSealing);
		assert_eq!(to_pending_set("lenient").unwrap(), PendingSet::SealingOrElseQueue);
		assert!(to_pending_set("othe").is_err());
	}

	#[test]
	fn test_to_address() {
		assert_eq!(
			to_address(Some("0xD9A111feda3f362f55Ef1744347CDC8Dd9964a41".into())).unwrap(),
			"D9A111feda3f362f55Ef1744347CDC8Dd9964a41".parse().unwrap()
		);
		assert_eq!(
			to_address(Some("D9A111feda3f362f55Ef1744347CDC8Dd9964a41".into())).unwrap(),
			"D9A111feda3f362f55Ef1744347CDC8Dd9964a41".parse().unwrap()
		);
		assert_eq!(to_address(None).unwrap(), Default::default());
	}

	#[test]
	fn test_to_addresses() {
		let addresses = to_addresses(&Some("0xD9A111feda3f362f55Ef1744347CDC8Dd9964a41,D9A111feda3f362f55Ef1744347CDC8Dd9964a42".into())).unwrap();
		assert_eq!(
			addresses,
			vec![
				"D9A111feda3f362f55Ef1744347CDC8Dd9964a41".parse().unwrap(),
				"D9A111feda3f362f55Ef1744347CDC8Dd9964a42".parse().unwrap(),
			]
		);
	}

	#[test]
	fn test_password() {
		let path = RandomTempPath::new();
		let mut file = File::create(path.as_path()).unwrap();
		file.write_all(b"a bc ").unwrap();
		assert_eq!(password_from_file(path.as_str().into()).unwrap().as_bytes(), b"a bc");
	}

	#[test]
	fn test_password_multiline() {
		let path = RandomTempPath::new();
		let mut file = File::create(path.as_path()).unwrap();
		file.write_all(br#"    password with trailing whitespace
those passwords should be
ignored
but the first password is trimmed

"#).unwrap();
		assert_eq!(&password_from_file(path.as_str().into()).unwrap(), "password with trailing whitespace");
	}

	#[test]
	#[cfg_attr(feature = "dev", allow(float_cmp))]
	fn test_to_price() {
		assert_eq!(to_price("1").unwrap(), 1.0);
		assert_eq!(to_price("2.3").unwrap(), 2.3);
		assert_eq!(to_price("2.33").unwrap(), 2.33);
	}

	#[test]
	#[cfg(windows)]
	fn test_geth_ipc_path() {
		assert_eq!(geth_ipc_path(true), r"\\.\pipe\geth.ipc".to_owned());
		assert_eq!(geth_ipc_path(false), r"\\.\pipe\geth.ipc".to_owned());
	}

	#[test]
	#[cfg(not(windows))]
	fn test_geth_ipc_path() {
		use path;
		assert_eq!(geth_ipc_path(true), path::ethereum::with_testnet("geth.ipc").to_str().unwrap().to_owned());
		assert_eq!(geth_ipc_path(false), path::ethereum::with_default("geth.ipc").to_str().unwrap().to_owned());
	}

	#[test]
	fn test_to_bootnodes() {
		let one_bootnode = "enode://e731347db0521f3476e6bbbb83375dcd7133a1601425ebd15fd10f3835fd4c304fba6282087ca5a0deeafadf0aa0d4fd56c3323331901c1f38bd181c283e3e35@128.199.55.137:30303";
		let two_bootnodes = "enode://e731347db0521f3476e6bbbb83375dcd7133a1601425ebd15fd10f3835fd4c304fba6282087ca5a0deeafadf0aa0d4fd56c3323331901c1f38bd181c283e3e35@128.199.55.137:30303,enode://e731347db0521f3476e6bbbb83375dcd7133a1601425ebd15fd10f3835fd4c304fba6282087ca5a0deeafadf0aa0d4fd56c3323331901c1f38bd181c283e3e35@128.199.55.137:30303";

		assert_eq!(to_bootnodes(&Some("".into())), Ok(vec![]));
		assert_eq!(to_bootnodes(&None), Ok(vec![]));
		assert_eq!(to_bootnodes(&Some(one_bootnode.into())), Ok(vec![one_bootnode.into()]));
		assert_eq!(to_bootnodes(&Some(two_bootnodes.into())), Ok(vec![one_bootnode.into(), one_bootnode.into()]));
	}
}
