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

use std::env;
use std::fs::File;
use std::time::Duration;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, IpAddr};
use std::path::PathBuf;
use cli::{USAGE, Args};
use docopt::Docopt;

use die::*;
use util::*;
use util::log::Colour::*;
use ethcore::account_provider::AccountProvider;
use util::network_settings::NetworkSettings;
use ethcore::client::{append_path, get_db_path, ClientConfig, DatabaseCompactionProfile, Switch, VMType};
use ethcore::miner::{MinerOptions, PendingSet};
use ethcore::ethereum;
use ethcore::spec::Spec;
use ethsync::SyncConfig;
use price_info::PriceInfo;
use rpc::IpcConfiguration;

pub struct Configuration {
	pub args: Args
}

pub struct Directories {
	pub keys: String,
	pub db: String,
	pub dapps: String,
	pub signer: String,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Policy {
	DaoSoft,
	Normal,
	Dogmatic,
}

impl Configuration {
	pub fn parse() -> Self {
		Configuration {
			args: Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit()),
		}
	}

	fn net_port(&self) -> u16 {
		self.args.flag_port
	}

	fn chain(&self) -> String {
		if self.args.flag_testnet {
			"morden".to_owned()
		} else {
			self.args.flag_chain.clone()
		}
	}

	fn max_peers(&self) -> u32 {
		self.args.flag_maxpeers.unwrap_or(self.args.flag_peers) as u32
	}

	fn decode_u256(d: &str, argument: &str) -> U256 {
		U256::from_dec_str(d).unwrap_or_else(|_|
			U256::from_str(clean_0x(d)).unwrap_or_else(|_|
				die!("{}: Invalid numeric value for {}. Must be either a decimal or a hex number.", d, argument)
			)
		)
	}

	fn work_notify(&self) -> Vec<String> {
		self.args.flag_notify_work.as_ref().map_or_else(Vec::new, |s| s.split(',').map(|s| s.to_owned()).collect())
	}

	pub fn miner_options(&self) -> MinerOptions {
		let (own, ext) = match self.args.flag_reseal_on_txs.as_str() {
			"none" => (false, false),
			"own" => (true, false),
			"ext" => (false, true),
			"all" => (true, true),
			x => die!("{}: Invalid value for --reseal option. Use --help for more information.", x)
		};
		MinerOptions {
			new_work_notify: self.work_notify(),
			force_sealing: self.args.flag_force_sealing,
			reseal_on_external_tx: ext,
			reseal_on_own_tx: own,
			tx_gas_limit: self.args.flag_tx_gas_limit.as_ref().map_or(!U256::zero(), |d| Self::decode_u256(d, "--tx-gas-limit")),
			tx_queue_size: self.args.flag_tx_queue_size,
			pending_set: match self.args.flag_relay_set.as_str() {
				"cheap" => PendingSet::AlwaysQueue,
				"strict" => PendingSet::AlwaysSealing,
				"lenient" => PendingSet::SealingOrElseQueue,
				x => die!("{}: Invalid value for --relay-set option. Use --help for more information.", x)
			},
			reseal_min_period: Duration::from_millis(self.args.flag_reseal_min_period),
			work_queue_size: self.args.flag_work_queue_size,
			enable_resubmission: !self.args.flag_remove_solved,
		}
	}

	pub fn author(&self) -> Option<Address> {
		self.args.flag_etherbase.as_ref()
			.or(self.args.flag_author.as_ref())
			.map(|d| Address::from_str(clean_0x(d)).unwrap_or_else(|_| {
				die!("{}: Invalid address for --author. Must be 40 hex characters, with or without the 0x at the beginning.", d)
			}))
	}

	pub fn policy(&self) -> Policy {
		match self.args.flag_fork.as_str() {
			"dao-soft" => Policy::DaoSoft,
			"normal" => Policy::Normal,
			"dogmatic" => Policy::Dogmatic,
			x => die!("{}: Invalid value given for --policy option. Use --help for more info.", x)
		}
	}

	pub fn gas_floor_target(&self) -> U256 {
		if self.policy() == Policy::DaoSoft {
			3_141_592.into()
		} else {
			let d = &self.args.flag_gas_floor_target;
			U256::from_dec_str(d).unwrap_or_else(|_| {
				die!("{}: Invalid target gas floor given. Must be a decimal unsigned 256-bit number.", d)
			})
		}
	}

	pub fn gas_ceil_target(&self) -> U256 {
		if self.policy() == Policy::DaoSoft {
			3_141_592.into()
		} else {
			let d = &self.args.flag_gas_cap;
			U256::from_dec_str(d).unwrap_or_else(|_| {
				die!("{}: Invalid target gas ceiling given. Must be a decimal unsigned 256-bit number.", d)
			})
		}
	}

	pub fn gas_price(&self) -> U256 {
		match self.args.flag_gasprice.as_ref() {
			Some(d) => {
				U256::from_dec_str(d).unwrap_or_else(|_| {
					die!("{}: Invalid gas price given. Must be a decimal unsigned 256-bit number.", d)
				})
			}
			_ => {
				let usd_per_tx: f32 = FromStr::from_str(&self.args.flag_usd_per_tx).unwrap_or_else(|_| {
					die!("{}: Invalid basic transaction price given in USD. Must be a decimal number.", self.args.flag_usd_per_tx)
				});
				let usd_per_eth = match self.args.flag_usd_per_eth.as_str() {
					"auto" => PriceInfo::get().map_or_else(|| {
						let last_known_good = 9.69696;
						// TODO: use #1083 to read last known good value.
						last_known_good
					}, |x| x.ethusd),
					"etherscan" => PriceInfo::get().map_or_else(|| {
						die!("Unable to retrieve USD value of ETH from etherscan. Rerun with a different value for --usd-per-eth.")
					}, |x| x.ethusd),
					x => FromStr::from_str(x).unwrap_or_else(|_| die!("{}: Invalid ether price given in USD. Must be a decimal number.", x))
				};
				// TODO: use #1083 to write last known good value as use_per_eth.

				let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
				let gas_per_tx: f32 = 21000.0;
				let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
				info!("Using a conversion rate of Ξ1 = {} ({} wei/gas)", paint(White.bold(), format!("US${}", usd_per_eth)), paint(Yellow.bold(), format!("{}", wei_per_gas)));
				U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap()
			}
		}
	}

	pub fn extra_data(&self) -> Bytes {
		match self.args.flag_extradata.as_ref().or(self.args.flag_extra_data.as_ref()) {
			Some(ref x) if x.len() <= 32 => x.as_bytes().to_owned(),
			None => version_data(),
			Some(ref x) => { die!("{}: Extra data must be at most 32 characters.", x); }
		}
	}

	pub fn spec(&self) -> Spec {
		match self.chain().as_str() {
			"frontier" | "homestead" | "mainnet" => ethereum::new_frontier(self.policy() != Policy::Dogmatic),
			"morden" | "testnet" => ethereum::new_morden(),
			"olympic" => ethereum::new_olympic(),
			f => Spec::load(contents(f).unwrap_or_else(|_| {
				die!("{}: Couldn't read chain specification file. Sure it exists?", f)
			}).as_ref()),
		}
	}

	pub fn normalize_enode(e: &str) -> Option<String> {
		if is_valid_node_url(e) {
			Some(e.to_owned())
		} else {
			None
		}
	}

	pub fn init_nodes(&self, spec: &Spec) -> Vec<String> {
		match self.args.flag_bootnodes {
			Some(ref x) if !x.is_empty() => x.split(',').map(|s| {
				Self::normalize_enode(s).unwrap_or_else(|| {
					die!("{}: Invalid node address format given for a boot node.", s)
				})
			}).collect(),
			Some(_) => Vec::new(),
			None => spec.nodes().clone(),
		}
	}

	pub fn init_reserved_nodes(&self) -> Vec<String> {
		use std::fs::File;

		if let Some(ref path) = self.args.flag_reserved_peers {
			let mut buffer = String::new();
			let mut node_file = File::open(path).unwrap_or_else(|e| {
				die!("Error opening reserved nodes file: {}", e);
			});
			node_file.read_to_string(&mut buffer).expect("Error reading reserved node file");
			buffer.lines().map(|s| {
				Self::normalize_enode(s).unwrap_or_else(|| {
					die!("{}: Invalid node address format given for a reserved node.", s);
				})
			}).collect()
		} else {
			Vec::new()
		}
	}

	pub fn net_addresses(&self) -> (Option<SocketAddr>, Option<SocketAddr>) {
		let port = self.net_port();
		let listen_address = Some(SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port));
		let public_address = if self.args.flag_nat.starts_with("extip:") {
			let host = &self.args.flag_nat[6..];
			let host = IpAddr::from_str(host).unwrap_or_else(|_| die!("Invalid host given with `--nat extip:{}`", host));
			Some(SocketAddr::new(host, port))
		} else {
			listen_address
		};
		(listen_address, public_address)
	}

	pub fn net_settings(&self, spec: &Spec) -> NetworkConfiguration {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.flag_nat == "any" || self.args.flag_nat == "upnp";
		ret.boot_nodes = self.init_nodes(spec);
		let (listen, public) = self.net_addresses();
		ret.listen_address = listen;
		ret.public_address = public;
		ret.use_secret = self.args.flag_node_key.as_ref().map(|s| Secret::from_str(s).unwrap_or_else(|_| s.sha3()));
		ret.discovery_enabled = !self.args.flag_no_discovery && !self.args.flag_nodiscover;
		ret.ideal_peers = self.max_peers();
		let mut net_path = PathBuf::from(&self.path());
		net_path.push("network");
		ret.config_path = Some(net_path.to_str().unwrap().to_owned());
		ret.reserved_nodes = self.init_reserved_nodes();

		if self.args.flag_reserved_only {
			ret.non_reserved_mode = ::util::network::NonReservedPeerMode::Deny;
		}
		ret
	}

	pub fn find_best_db(&self, spec: &Spec) -> Option<journaldb::Algorithm> {
		let mut ret = None;
		let mut latest_era = None;
		let jdb_types = [journaldb::Algorithm::Archive, journaldb::Algorithm::EarlyMerge, journaldb::Algorithm::OverlayRecent, journaldb::Algorithm::RefCounted];
		for i in jdb_types.into_iter() {
			let db = journaldb::new(&append_path(&get_db_path(Path::new(&self.path()), *i, spec.genesis_header().hash()), "state"), *i, kvdb::DatabaseConfig::default());
			trace!(target: "parity", "Looking for best DB: {} at {:?}", i, db.latest_era());
			match (latest_era, db.latest_era()) {
				(Some(best), Some(this)) if best >= this => {}
				(_, None) => {}
				(_, Some(this)) => {
					latest_era = Some(this);
					ret = Some(*i);
				}
			}
		}
		ret
	}

	pub fn client_config(&self, spec: &Spec) -> ClientConfig {
		let mut client_config = ClientConfig::default();

		match self.args.flag_cache {
			Some(mb) => {
				client_config.blockchain.max_cache_size = mb * 1024 * 1024;
				client_config.blockchain.pref_cache_size = client_config.blockchain.max_cache_size * 3 / 4;
			}
			None => {
				client_config.blockchain.pref_cache_size = self.args.flag_cache_pref_size;
				client_config.blockchain.max_cache_size = self.args.flag_cache_max_size;
			}
		}
		// forced blockchain (blocks + extras) db cache size if provided
		client_config.blockchain.db_cache_size = self.args.flag_db_cache_size.and_then(|cs| Some(cs / 2));

		client_config.tracing.enabled = match self.args.flag_tracing.as_str() {
			"auto" => Switch::Auto,
			"on" => Switch::On,
			"off" => Switch::Off,
			_ => { die!("Invalid tracing method given!") }
		};
		// forced trace db cache size if provided
		client_config.tracing.db_cache_size = self.args.flag_db_cache_size.and_then(|cs| Some(cs / 4));

		client_config.pruning = match self.args.flag_pruning.as_str() {
			"archive" => journaldb::Algorithm::Archive,
			"light" => journaldb::Algorithm::EarlyMerge,
			"fast" => journaldb::Algorithm::OverlayRecent,
			"basic" => journaldb::Algorithm::RefCounted,
			"auto" => self.find_best_db(spec).unwrap_or(journaldb::Algorithm::OverlayRecent),
			_ => { die!("Invalid pruning method given."); }
		};

		if self.args.flag_fat_db {
			if let journaldb::Algorithm::Archive = client_config.pruning {
				client_config.trie_spec = TrieSpec::Fat;
			} else {
				die!("Fatdb is not supported. Please rerun with --pruning=archive")
			}
		}

		// forced state db cache size if provided
		client_config.db_cache_size = self.args.flag_db_cache_size.and_then(|cs| Some(cs / 4));

		// compaction profile
		client_config.db_compaction = match self.args.flag_db_compaction.as_str() {
			"ssd" => DatabaseCompactionProfile::Default,
			"hdd" => DatabaseCompactionProfile::HDD,
			_ => { die!("Invalid compaction profile given (--db-compaction argument), expected hdd/ssd (default)."); }
		};

		if self.args.flag_jitvm {
			client_config.vm_type = VMType::jit().unwrap_or_else(|| die!("Parity built without jit vm."))
		}

		trace!(target: "parity", "Using pruning strategy of {}", client_config.pruning);
		client_config.name = self.args.flag_identity.clone();
		client_config.queue.max_mem_use = self.args.flag_queue_max_size;
		client_config
	}

	pub fn sync_config(&self, spec: &Spec) -> SyncConfig {
		let mut sync_config = SyncConfig::default();
		sync_config.network_id = self.args.flag_network_id.as_ref().or(self.args.flag_networkid.as_ref()).map_or(spec.network_id(), |id| {
			U256::from_str(id).unwrap_or_else(|_| die!("{}: Invalid index given with --network-id/--networkid", id))
		});
		sync_config
	}

	pub fn account_service(&self) -> AccountProvider {
		use ethcore::ethstore::{import_accounts, EthStore};
		use ethcore::ethstore::dir::{GethDirectory, DirectoryType, DiskDirectory};

		// Secret Store
		let passwords = self.args.flag_password.iter().flat_map(|filename| {
			BufReader::new(&File::open(filename).unwrap_or_else(|_| die!("{} Unable to read password file. Ensure it exists and permissions are correct.", filename)))
				.lines()
				.map(|l| l.unwrap())
				.collect::<Vec<_>>()
				.into_iter()
		}).collect::<Vec<_>>();

		if !self.args.flag_no_import_keys {
			let dir_type = if self.args.flag_testnet {
				DirectoryType::Testnet
			} else {
				DirectoryType::Main
			};

			let from = GethDirectory::open(dir_type);
			let to = DiskDirectory::create(self.keys_path()).unwrap();
			// ignore error, cause geth may not exist
			let _ = import_accounts(&from, &to);
		}

		let dir = Box::new(DiskDirectory::create(self.keys_path()).unwrap());
		let iterations = self.keys_iterations();
		let account_service = AccountProvider::new(Box::new(EthStore::open_with_iterations(dir, iterations).unwrap()));

		if let Some(ref unlocks) = self.args.flag_unlock {
			for d in unlocks.split(',') {
				let a = Address::from_str(clean_0x(d)).unwrap_or_else(|_| {
					die!("{}: Invalid address for --unlock. Must be 40 hex characters, without the 0x at the beginning.", d)
				});
				if passwords.iter().find(|p| account_service.unlock_account_permanently(a, (*p).clone()).is_ok()).is_none() {
					die!("No password given to unlock account {}. Pass the password using `--password`.", a);
				}
			}
		}
		account_service
	}

	pub fn rpc_apis(&self) -> String {
		self.args.flag_rpcapi.clone().unwrap_or(self.args.flag_jsonrpc_apis.clone())
	}

	pub fn rpc_cors(&self) -> Vec<String> {
		let cors = self.args.flag_jsonrpc_cors.clone().or(self.args.flag_rpccorsdomain.clone());
		cors.map_or_else(Vec::new, |c| c.split(',').map(|s| s.to_owned()).collect())
	}

	fn geth_ipc_path(&self) -> String {
		if cfg!(windows) {
			r"\\.\pipe\geth.ipc".to_owned()
		}
		else {
			if self.args.flag_testnet { path::ethereum::with_testnet("geth.ipc") }
			else { path::ethereum::with_default("geth.ipc") }
				.to_str().unwrap().to_owned()
		}
	}

	pub fn keys_iterations(&self) -> u32 {
		self.args.flag_keys_iterations
	}

	pub fn ipc_settings(&self) -> IpcConfiguration {
		IpcConfiguration {
			enabled: !(self.args.flag_ipcdisable || self.args.flag_ipc_off || self.args.flag_no_ipc),
			socket_addr: self.ipc_path(),
			apis: self.args.flag_ipcapi.clone().unwrap_or(self.args.flag_ipc_apis.clone()),
		}
	}

	pub fn network_settings(&self) -> NetworkSettings {
		if self.args.flag_jsonrpc { println!("WARNING: Flag -j/--json-rpc is deprecated. JSON-RPC is now on by default. Ignoring."); }
		NetworkSettings {
			name: self.args.flag_identity.clone(),
			chain: self.chain(),
			max_peers: self.max_peers(),
			network_port: self.net_port(),
			rpc_enabled: !self.args.flag_jsonrpc_off && !self.args.flag_no_jsonrpc,
			rpc_interface: self.args.flag_rpcaddr.clone().unwrap_or(self.args.flag_jsonrpc_interface.clone()),
			rpc_port: self.args.flag_rpcport.unwrap_or(self.args.flag_jsonrpc_port),
		}
	}

	pub fn directories(&self) -> Directories {
		let db_path = Configuration::replace_home(
			self.args.flag_datadir.as_ref().unwrap_or(&self.args.flag_db_path));
		::std::fs::create_dir_all(&db_path).unwrap_or_else(|e| die_with_io_error("main", e));

		let keys_path = Configuration::replace_home(
			if self.args.flag_testnet {
				"$HOME/.parity/testnet_keys"
			} else {
				&self.args.flag_keys_path
			}
		);
		::std::fs::create_dir_all(&keys_path).unwrap_or_else(|e| die_with_io_error("main", e));
		let dapps_path = Configuration::replace_home(&self.args.flag_dapps_path);
		::std::fs::create_dir_all(&dapps_path).unwrap_or_else(|e| die_with_io_error("main", e));
		let signer_path = Configuration::replace_home(&self.args.flag_signer_path);
		::std::fs::create_dir_all(&signer_path).unwrap_or_else(|e| die_with_io_error("main", e));


		Directories {
			keys: keys_path,
			db: db_path,
			dapps: dapps_path,
			signer: signer_path,
		}
	}

	pub fn keys_path(&self) -> String {
		self.directories().keys
	}

	pub fn path(&self) -> String {
		self.directories().db
	}

	fn replace_home(arg: &str) -> String {
		arg.replace("$HOME", env::home_dir().unwrap().to_str().unwrap())
	}

	fn ipc_path(&self) -> String {
		if self.args.flag_geth {
			self.geth_ipc_path()
		} else if cfg!(windows) {
			r"\\.\pipe\parity.jsonrpc".to_owned()
		} else {
			Configuration::replace_home(&self.args.flag_ipcpath.clone().unwrap_or(self.args.flag_ipc_path.clone()))
		}
	}

	pub fn have_color(&self) -> bool {
		!self.args.flag_no_color && !cfg!(windows)
	}

	pub fn signer_port(&self) -> Option<u16> {
		if !self.signer_enabled() {
			None
		} else {
			Some(self.args.flag_signer_port)
		}
	}

	pub fn rpc_interface(&self) -> String {
		match self.network_settings().rpc_interface.as_str() {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	pub fn dapps_interface(&self) -> String {
		match self.args.flag_dapps_interface.as_str() {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	pub fn dapps_enabled(&self) -> bool {
		!self.args.flag_dapps_off && !self.args.flag_no_dapps
	}

	pub fn signer_enabled(&self) -> bool {
		(self.args.cmd_ui && !self.args.flag_no_signer) ||
		(!self.args.cmd_ui && self.args.flag_signer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use cli::USAGE;
	use docopt::Docopt;
	use util::network_settings::NetworkSettings;

	fn parse(args: &[&str]) -> Configuration {
		Configuration {
			args: Docopt::new(USAGE).unwrap().argv(args).decode().unwrap(),
		}
	}

	#[test]
	fn should_parse_network_settings() {
		// given

		// when
		let conf = parse(&["parity", "--testnet", "--identity", "testname"]);

		// then
		assert_eq!(conf.network_settings(), NetworkSettings {
			name: "testname".to_owned(),
			chain: "morden".to_owned(),
			max_peers: 25,
			network_port: 30303,
			rpc_enabled: true,
			rpc_interface: "local".to_owned(),
			rpc_port: 8545,
		});
	}

	#[test]
	fn should_parse_rpc_settings_with_geth_compatiblity() {
		// given
		fn assert(conf: Configuration) {
			let net = conf.network_settings();
			assert_eq!(net.rpc_enabled, true);
			assert_eq!(net.rpc_interface, "all".to_owned());
			assert_eq!(net.rpc_port, 8000);
			assert_eq!(conf.rpc_cors(), vec!["*".to_owned()]);
			assert_eq!(conf.rpc_apis(), "web3,eth".to_owned());
		}

		// when
		let conf1 = parse(&["parity", "-j",
						 "--jsonrpc-port", "8000",
						 "--jsonrpc-interface", "all",
						 "--jsonrpc-cors", "*",
						 "--jsonrpc-apis", "web3,eth"
						 ]);
		let conf2 = parse(&["parity", "--rpc",
						  "--rpcport", "8000",
						  "--rpcaddr", "all",
						  "--rpccorsdomain", "*",
						  "--rpcapi", "web3,eth"
						  ]);

		// then
		assert(conf1);
		assert(conf2);
	}
}

