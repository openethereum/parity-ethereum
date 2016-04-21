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

//! Ethcore client application.

#![warn(missing_docs)]
#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]
#![cfg_attr(feature="dev", allow(useless_format))]

extern crate docopt;
extern crate num_cpus;
extern crate rustc_serialize;
extern crate ethcore_util as util;
extern crate ethcore;
extern crate ethsync;
extern crate ethminer;
#[macro_use]
extern crate log as rlog;
extern crate env_logger;
extern crate ctrlc;
extern crate fdlimit;
extern crate daemonize;
extern crate time;
extern crate number_prefix;
extern crate rpassword;
extern crate semver;
extern crate ethcore_ipc as ipc;
extern crate ethcore_ipc_nano as nanoipc;
extern crate serde;
extern crate bincode;
// for price_info.rs
#[macro_use] extern crate hyper;

#[cfg(feature = "rpc")]
extern crate ethcore_rpc;

#[cfg(feature = "webapp")]
extern crate ethcore_webapp;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::net::{SocketAddr, IpAddr};
use std::env;
use std::path::PathBuf;
use ctrlc::CtrlC;
use util::*;
use util::panics::{MayPanic, ForwardPanic, PanicHandler};
use util::keys::store::AccountService;
use ethcore::ethereum;
use ethcore::client::{append_path, get_db_path, ClientConfig};
use ethcore::spec::Spec;
use ethcore::service::ClientService;
use ethsync::{EthSync, SyncConfig};
use ethminer::{Miner, MinerService};
use docopt::Docopt;
use daemonize::Daemonize;

#[macro_use]
mod die;
mod price_info;
mod upgrade;
mod hypervisor;
mod setup_log;
mod rpc;
mod webapp;
mod informant;
mod io_handler;
mod cli;

use die::*;
use cli::{USAGE, print_version, Args};
use rpc::RpcServer;
use webapp::WebappServer;
use io_handler::ClientIoHandler;

struct Configuration {
	args: Args
}

fn main() {
	let conf = Configuration::parse();
	execute(conf);
}

fn execute(conf: Configuration) {
	if conf.args.flag_version {
		print_version();
		return;
	}

	execute_upgrades(&conf);

	if conf.args.cmd_daemon {
		Daemonize::new()
			.pid_file(conf.args.arg_pid_file.clone())
			.chown_pid_file(true)
			.start()
			.unwrap_or_else(|e| die!("Couldn't daemonize; {}", e));
	}

	if conf.args.cmd_account {
		execute_account_cli(conf);
		return;
	}

	execute_client(conf);
}

fn execute_upgrades(conf: &Configuration) {
	match ::upgrade::upgrade(Some(&conf.path())) {
		Ok(upgrades_applied) if upgrades_applied > 0 => {
			println!("Executed {} upgrade scripts - ok", upgrades_applied);
		},
		Err(e) => {
			die!("Error upgrading parity data: {:?}", e);
		},
		_ => {},
	}
}

fn execute_client(conf: Configuration) {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Setup logging
	let logger = setup_log::setup_log(&conf.args.flag_logging);
	// Raise fdlimit
	unsafe { ::fdlimit::raise_fd_limit(); }

	let spec = conf.spec();
	let net_settings = conf.net_settings(&spec);
	let sync_config = conf.sync_config(&spec);
	let client_config = conf.client_config(&spec);

	// Secret Store
	let account_service = Arc::new(conf.account_service());

	// Build client
	let mut service = ClientService::start(
		client_config, spec, net_settings, &Path::new(&conf.path())
	).unwrap_or_else(|e| die_with_error(e));

	panic_handler.forward_from(&service);
	let client = service.client();

	// Miner
	let miner = Miner::new(conf.args.flag_force_sealing);
	miner.set_author(conf.author());
	miner.set_gas_floor_target(conf.gas_floor_target());
	miner.set_extra_data(conf.extra_data());
	miner.set_minimal_gas_price(conf.gas_price());
	miner.set_transactions_limit(conf.args.flag_tx_limit);

	// Sync
	let sync = EthSync::register(service.network(), sync_config, client.clone(), miner.clone());

	// Setup rpc
	let rpc_server = rpc::new(rpc::Configuration {
		enabled: conf.args.flag_jsonrpc || conf.args.flag_rpc,
		interface: conf.args.flag_rpcaddr.clone().unwrap_or(conf.args.flag_jsonrpc_interface.clone()),
		port: conf.args.flag_rpcport.unwrap_or(conf.args.flag_jsonrpc_port),
		apis: conf.args.flag_rpcapi.clone().unwrap_or(conf.args.flag_jsonrpc_apis.clone()),
		cors: conf.args.flag_jsonrpc_cors.clone().or(conf.args.flag_rpccorsdomain.clone()),
	}, rpc::Dependencies {
		client: client.clone(),
		sync: sync.clone(),
		secret_store: account_service.clone(),
		miner: miner.clone(),
		logger: logger.clone()
	});

	let webapp_server = webapp::new(webapp::Configuration {
		enabled: conf.args.flag_webapp,
		interface: conf.args.flag_webapp_interface.clone(),
		port: conf.args.flag_webapp_port,
		user: conf.args.flag_webapp_user.clone(),
		pass: conf.args.flag_webapp_pass.clone(),
	}, webapp::Dependencies {
		client: client.clone(),
		sync: sync.clone(),
		secret_store: account_service.clone(),
		miner: miner.clone(),
		logger: logger.clone()
	});

	// Register IO handler
	let io_handler  = Arc::new(ClientIoHandler {
		client: service.client(),
		info: Default::default(),
		sync: sync.clone(),
		accounts: account_service.clone(),
	});
	service.io().register_handler(io_handler).expect("Error registering IO handler");

	// Handle exit
	wait_for_exit(panic_handler, rpc_server, webapp_server);
}

fn execute_account_cli(conf: Configuration) {
	use util::keys::store::SecretStore;
	use rpassword::read_password;
	let mut secret_store = SecretStore::new_in(Path::new(&conf.keys_path()));
	if conf.args.cmd_new {
		println!("Please note that password is NOT RECOVERABLE.");
		print!("Type password: ");
		let password = read_password().unwrap();
		print!("Repeat password: ");
		let password_repeat = read_password().unwrap();
		if password != password_repeat {
			println!("Passwords do not match!");
			return;
		}
		println!("New account address:");
		let new_address = secret_store.new_account(&password).unwrap();
		println!("{:?}", new_address);
		return;
	}
	if conf.args.cmd_list {
		println!("Known addresses:");
		for &(addr, _) in &secret_store.accounts().unwrap() {
			println!("{:?}", addr);
		}
	}
}

fn wait_for_exit(panic_handler: Arc<PanicHandler>, _rpc_server: Option<RpcServer>, _webapp_server: Option<WebappServer>) {
	let exit = Arc::new(Condvar::new());

	// Handle possible exits
	let e = exit.clone();
	CtrlC::set_handler(move || { e.notify_all(); });

	// Handle panics
	let e = exit.clone();
	panic_handler.on_panic(move |_reason| { e.notify_all(); });

	// Wait for signal
	let mutex = Mutex::new(());
	let _ = exit.wait(mutex.lock().unwrap()).unwrap();
	info!("Finishing work, please wait...");
}

impl Configuration {
	fn parse() -> Self {
		Configuration {
			args: Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit()),
		}
	}

	fn path(&self) -> String {
		let d = self.args.flag_datadir.as_ref().unwrap_or(&self.args.flag_db_path);
		d.replace("$HOME", env::home_dir().unwrap().to_str().unwrap())
	}

	fn author(&self) -> Address {
		let d = self.args.flag_etherbase.as_ref().unwrap_or(&self.args.flag_author);
		Address::from_str(clean_0x(d)).unwrap_or_else(|_| {
			die!("{}: Invalid address for --author. Must be 40 hex characters, with or without the 0x at the beginning.", d)
		})
	}

	fn gas_floor_target(&self) -> U256 {
		let d = &self.args.flag_gas_floor_target;
		U256::from_dec_str(d).unwrap_or_else(|_| {
			die!("{}: Invalid target gas floor given. Must be a decimal unsigned 256-bit number.", d)
		})
	}

	fn gas_price(&self) -> U256 {
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
					"etherscan" => price_info::PriceInfo::get().map_or_else(|| {
						die!("Unable to retrieve USD value of ETH from etherscan. Rerun with a different value for --usd-per-eth.")
					}, |x| x.ethusd),
					x => FromStr::from_str(x).unwrap_or_else(|_| die!("{}: Invalid ether price given in USD. Must be a decimal number.", x))
				};
				let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
				let gas_per_tx: f32 = 21000.0;
				let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
				info!("Using a conversion rate of Ξ1 = US${} ({} wei/gas)", usd_per_eth, wei_per_gas);
				U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap()
			}
		}
	}

	fn extra_data(&self) -> Bytes {
		match self.args.flag_extradata.as_ref().or(self.args.flag_extra_data.as_ref()) {
			Some(ref x) if x.len() <= 32 => x.as_bytes().to_owned(),
			None => version_data(),
			Some(ref x) => { die!("{}: Extra data must be at most 32 characters.", x); }
		}
	}

	fn keys_path(&self) -> String {
		self.args.flag_keys_path.replace("$HOME", env::home_dir().unwrap().to_str().unwrap())
	}

	fn spec(&self) -> Spec {
		if self.args.flag_testnet {
			return ethereum::new_morden();
		}
		match self.args.flag_chain.as_ref() {
			"frontier" | "homestead" | "mainnet" => ethereum::new_frontier(),
			"morden" | "testnet" => ethereum::new_morden(),
			"olympic" => ethereum::new_olympic(),
			f => Spec::load(contents(f).unwrap_or_else(|_| {
				die!("{}: Couldn't read chain specification file. Sure it exists?", f)
			}).as_ref()),
		}
	}

	fn normalize_enode(e: &str) -> Option<String> {
		if is_valid_node_url(e) {
			Some(e.to_owned())
		} else {
			None
		}
	}

	fn init_nodes(&self, spec: &Spec) -> Vec<String> {
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

	fn net_addresses(&self) -> (Option<SocketAddr>, Option<SocketAddr>) {
		let listen_address = Some(SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), self.args.flag_port));
		let public_address = if self.args.flag_nat.starts_with("extip:") {
			let host = &self.args.flag_nat[6..];
			let host = IpAddr::from_str(host).unwrap_or_else(|_| die!("Invalid host given with `--nat extip:{}`", host));
			Some(SocketAddr::new(host, self.args.flag_port))
		} else {
			listen_address
		};
		(listen_address, public_address)
	}

	fn net_settings(&self, spec: &Spec) -> NetworkConfiguration {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.flag_nat == "any" || self.args.flag_nat == "upnp";
		ret.boot_nodes = self.init_nodes(spec);
		let (listen, public) = self.net_addresses();
		ret.listen_address = listen;
		ret.public_address = public;
		ret.use_secret = self.args.flag_node_key.as_ref().map(|s| Secret::from_str(&s).unwrap_or_else(|_| s.sha3()));
		ret.discovery_enabled = !self.args.flag_no_discovery && !self.args.flag_nodiscover;
		ret.ideal_peers = self.args.flag_maxpeers.unwrap_or(self.args.flag_peers) as u32;
		let mut net_path = PathBuf::from(&self.path());
		net_path.push("network");
		ret.config_path = Some(net_path.to_str().unwrap().to_owned());
		ret
	}

	fn find_best_db(&self, spec: &Spec) -> Option<journaldb::Algorithm> {
		let mut ret = None;
		let mut latest_era = None;
		let jdb_types = [journaldb::Algorithm::Archive, journaldb::Algorithm::EarlyMerge, journaldb::Algorithm::OverlayRecent, journaldb::Algorithm::RefCounted];
		for i in jdb_types.into_iter() {
			let db = journaldb::new(&append_path(&get_db_path(&Path::new(&self.path()), *i, spec.genesis_header().hash()), "state"), *i);
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

	fn client_config(&self, spec: &Spec) -> ClientConfig {
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
		client_config.pruning = match self.args.flag_pruning.as_str() {
			"archive" => journaldb::Algorithm::Archive,
			"light" => journaldb::Algorithm::EarlyMerge,
			"fast" => journaldb::Algorithm::OverlayRecent,
			"basic" => journaldb::Algorithm::RefCounted,
			"auto" => self.find_best_db(spec).unwrap_or(journaldb::Algorithm::OverlayRecent),
			_ => { die!("Invalid pruning method given."); }
		};
		trace!(target: "parity", "Using pruning strategy of {}", client_config.pruning);
		client_config.name = self.args.flag_identity.clone();
		client_config.queue.max_mem_use = self.args.flag_queue_max_size;
		client_config
	}

	fn sync_config(&self, spec: &Spec) -> SyncConfig {
		let mut sync_config = SyncConfig::default();
		sync_config.network_id = self.args.flag_network_id.as_ref().or(self.args.flag_networkid.as_ref()).map_or(spec.network_id(), |id| {
			U256::from_str(id).unwrap_or_else(|_| die!("{}: Invalid index given with --network-id/--networkid", id))
		});
		sync_config
	}

	fn account_service(&self) -> AccountService {
		// Secret Store
		let passwords = self.args.flag_password.iter().flat_map(|filename| {
			BufReader::new(&File::open(filename).unwrap_or_else(|_| die!("{} Unable to read password file. Ensure it exists and permissions are correct.", filename)))
				.lines()
				.map(|l| l.unwrap())
				.collect::<Vec<_>>()
				.into_iter()
		}).collect::<Vec<_>>();
		let account_service = AccountService::new_in(Path::new(&self.keys_path()));
		if let Some(ref unlocks) = self.args.flag_unlock {
			for d in unlocks.split(',') {
				let a = Address::from_str(clean_0x(&d)).unwrap_or_else(|_| {
					die!("{}: Invalid address for --unlock. Must be 40 hex characters, without the 0x at the beginning.", d)
				});
				if passwords.iter().find(|p| account_service.unlock_account_no_expire(&a, p).is_ok()).is_none() {
					die!("No password given to unlock account {}. Pass the password using `--password`.", a);
				}
			}
		}
		account_service
	}
}

/// Parity needs at least 1 test to generate coverage reports correctly.
#[test]
fn if_works() {
}
