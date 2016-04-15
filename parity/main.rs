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
extern crate ethcore_rpc as rpc;
#[cfg(feature = "webapp")]
extern crate ethcore_webapp as webapp;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::net::{SocketAddr, IpAddr};
use std::env;
use std::process::exit;
use std::path::PathBuf;
use env_logger::LogBuilder;
use ctrlc::CtrlC;
use util::*;
use util::panics::{MayPanic, ForwardPanic, PanicHandler};
use util::keys::store::*;
use ethcore::spec::*;
use ethcore::client::*;
use ethcore::service::{ClientService, NetSyncMessage};
use ethcore::ethereum;
use ethsync::{EthSync, SyncConfig, SyncProvider};
use ethminer::{Miner, MinerService};
use docopt::Docopt;
use daemonize::Daemonize;
use number_prefix::{binary_prefix, Standalone, Prefixed};
#[cfg(feature = "rpc")]
use rpc::Server as RpcServer;
#[cfg(feature = "webapp")]
use webapp::Listening as WebappServer;

mod price_info;
mod upgrade;
mod hypervisor;

fn die_with_message(msg: &str) -> ! {
	println!("ERROR: {}", msg);
	exit(1);
}

#[macro_export]
macro_rules! die {
	($($arg:tt)*) => (die_with_message(&format!("{}", format_args!($($arg)*))));
}

const USAGE: &'static str = r#"
Parity. Ethereum Client.
  By Wood/Paronyan/Kotewicz/Drwięga/Volf.
  Copyright 2015, 2016 Ethcore (UK) Limited

Usage:
  parity daemon <pid-file> [options]
  parity account (new | list) [options]
  parity [options]

Protocol Options:
  --chain CHAIN            Specify the blockchain type. CHAIN may be either a
                           JSON chain specification file or olympic, frontier,
                           homestead, mainnet, morden, or testnet
                           [default: homestead].
  -d --db-path PATH        Specify the database & configuration directory path
                           [default: $HOME/.parity].
  --keys-path PATH         Specify the path for JSON key files to be found
                           [default: $HOME/.parity/keys].
  --identity NAME          Specify your node's name.

Account Options:
  --unlock ACCOUNTS        Unlock ACCOUNTS for the duration of the execution.
                           ACCOUNTS is a comma-delimited list of addresses.
  --password FILE          Provide a file containing a password for unlocking
                           an account.

Networking Options:
  --port PORT              Override the port on which the node should listen
                           [default: 30303].
  --peers NUM              Try to maintain that many peers [default: 25].
  --nat METHOD             Specify method to use for determining public
                           address. Must be one of: any, none, upnp,
                           extip:<IP> [default: any].
  --network-id INDEX       Override the network identifier from the chain we
                           are on.
  --bootnodes NODES        Override the bootnodes from our chain. NODES should
                           be comma-delimited enodes.
  --no-discovery           Disable new peer discovery.
  --node-key KEY           Specify node secret key, either as 64-character hex
                           string or input to SHA3 operation.

API and Console Options:
  -j --jsonrpc             Enable the JSON-RPC API server.
  --jsonrpc-port PORT      Specify the port portion of the JSONRPC API server
                           [default: 8545].
  --jsonrpc-interface IP   Specify the hostname portion of the JSONRPC API
                           server, IP should be an interface's IP address, or
                           all (all interfaces) or local [default: local].
  --jsonrpc-cors URL       Specify CORS header for JSON-RPC API responses.
  --jsonrpc-apis APIS      Specify the APIs available through the JSONRPC
                           interface. APIS is a comma-delimited list of API
                           name. Possible name are web3, eth and net.
                           [default: web3,eth,net,personal,ethcore].
  -w --webapp              Enable the web applications server (e.g.
                           status page).
  --webapp-port PORT       Specify the port portion of the WebApps server
                           [default: 8080].
  --webapp-interface IP    Specify the hostname portion of the WebApps
                           server, IP should be an interface's IP address, or
                           all (all interfaces) or local [default: local].
  --webapp-user USERNAME   Specify username for WebApps server. It will be
                           used in HTTP Basic Authentication Scheme.
                           If --webapp-pass is not specified you will be
                           asked for password on startup.
  --webapp-pass PASSWORD   Specify password for WebApps server. Use only in
                           conjunction with --webapp-user.

Sealing/Mining Options:
  --force-sealing          Force the node to author new blocks as if it were
                           always sealing/mining.
  --usd-per-tx USD         Amount of USD to be paid for a basic transaction
                           [default: 0.005]. The minimum gas price is set
                           accordingly.
  --usd-per-eth SOURCE     USD value of a single ETH. SOURCE may be either an
                           amount in USD or a web service [default: etherscan].
  --gas-floor-target GAS   Amount of gas per block to target when sealing a new
                           block [default: 4712388].
  --author ADDRESS         Specify the block author (aka "coinbase") address
                           for sending block rewards from sealed blocks
                           [default: 0037a6b811ffeb6e072da21179d11b1406371c63].
  --extra-data STRING      Specify a custom extra-data for authored blocks, no
                           more than 32 characters.

Footprint Options:
  --pruning METHOD         Configure pruning of the state/storage trie. METHOD
                           may be one of auto, archive, basic, fast, light:
                           archive - keep all state trie data. No pruning.
                           basic - reference count in disk DB. Slow but light.
                           fast - maintain journal overlay. Fast but 50MB used.
                           light - early merges with partial tracking. Fast
                           and light. Experimental!
                           auto - use the method most recently synced or
                           default to archive if none synced [default: auto].
  --cache-pref-size BYTES  Specify the prefered size of the blockchain cache in
                           bytes [default: 16384].
  --cache-max-size BYTES   Specify the maximum size of the blockchain cache in
                           bytes [default: 262144].
  --queue-max-size BYTES   Specify the maximum size of memory to use for block
                           queue [default: 52428800].
  --cache MEGABYTES        Set total amount of discretionary memory to use for
                           the entire system, overrides other cache and queue
                           options.

Geth-compatibility Options:
  --datadir PATH           Equivalent to --db-path PATH.
  --testnet                Equivalent to --chain testnet.
  --networkid INDEX        Equivalent to --network-id INDEX.
  --maxpeers COUNT         Equivalent to --peers COUNT.
  --nodekey KEY            Equivalent to --node-key KEY.
  --nodiscover             Equivalent to --no-discovery.
  --rpc                    Equivalent to --jsonrpc.
  --rpcaddr IP             Equivalent to --jsonrpc-interface IP.
  --rpcport PORT           Equivalent to --jsonrpc-port PORT.
  --rpcapi APIS            Equivalent to --jsonrpc-apis APIS.
  --rpccorsdomain URL      Equivalent to --jsonrpc-cors URL.
  --gasprice WEI           Minimum amount of Wei per GAS to be paid for a
                           transaction to be accepted for mining. Overrides
                           --basic-tx-usd.
  --etherbase ADDRESS      Equivalent to --author ADDRESS.
  --extradata STRING       Equivalent to --extra-data STRING.

Miscellaneous Options:
  -l --logging LOGGING     Specify the logging level. Must conform to the same
                           format as RUST_LOG.
  -v --version             Show information about version.
  -h --help                Show this screen.
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
	cmd_daemon: bool,
	cmd_account: bool,
	cmd_new: bool,
	cmd_list: bool,
	arg_pid_file: String,
	flag_chain: String,
	flag_db_path: String,
	flag_identity: String,
	flag_unlock: Option<String>,
	flag_password: Vec<String>,
	flag_cache: Option<usize>,
	flag_keys_path: String,
	flag_bootnodes: Option<String>,
	flag_network_id: Option<String>,
	flag_pruning: String,
	flag_port: u16,
	flag_peers: usize,
	flag_no_discovery: bool,
	flag_nat: String,
	flag_node_key: Option<String>,
	flag_cache_pref_size: usize,
	flag_cache_max_size: usize,
	flag_queue_max_size: usize,
	flag_jsonrpc: bool,
	flag_jsonrpc_interface: String,
	flag_jsonrpc_port: u16,
	flag_jsonrpc_cors: Option<String>,
	flag_jsonrpc_apis: String,
	flag_webapp: bool,
	flag_webapp_port: u16,
	flag_webapp_interface: String,
	flag_webapp_user: Option<String>,
	flag_webapp_pass: Option<String>,
	flag_force_sealing: bool,
	flag_author: String,
	flag_usd_per_tx: String,
	flag_usd_per_eth: String,
	flag_gas_floor_target: String,
	flag_extra_data: Option<String>,
	flag_logging: Option<String>,
	flag_version: bool,
	// geth-compatibility...
	flag_nodekey: Option<String>,
	flag_nodiscover: bool,
	flag_maxpeers: Option<usize>,
	flag_datadir: Option<String>,
	flag_extradata: Option<String>,
	flag_etherbase: Option<String>,
	flag_gasprice: Option<String>,
	flag_rpc: bool,
	flag_rpcaddr: Option<String>,
	flag_rpcport: Option<u16>,
	flag_rpccorsdomain: Option<String>,
	flag_rpcapi: Option<String>,
	flag_testnet: bool,
	flag_networkid: Option<String>,
}

fn setup_log(init: &Option<String>) {
	use rlog::*;

	let mut builder = LogBuilder::new();
	builder.filter(None, LogLevelFilter::Info);

	if env::var("RUST_LOG").is_ok() {
		builder.parse(&env::var("RUST_LOG").unwrap());
	}

	if let Some(ref s) = *init {
		builder.parse(s);
	}

	let format = |record: &LogRecord| {
		let timestamp = time::strftime("%Y-%m-%d %H:%M:%S %Z", &time::now()).unwrap();
		if max_log_level() <= LogLevelFilter::Info {
			format!("{}{}", timestamp, record.args())
		} else {
			format!("{}{}:{}: {}", timestamp, record.level(), record.target(), record.args())
		}
    };
	builder.format(format);
	builder.init().unwrap();
}

#[cfg(feature = "rpc")]
fn setup_rpc_server(
	client: Arc<Client>,
	sync: Arc<EthSync>,
	secret_store: Arc<AccountService>,
	miner: Arc<Miner>,
	url: &SocketAddr,
	cors_domain: Option<String>,
	apis: Vec<&str>,
) -> RpcServer {
	use rpc::v1::*;

	let server = rpc::RpcServer::new();
	for api in apis.into_iter() {
		match api {
			"web3" => server.add_delegate(Web3Client::new().to_delegate()),
			"net" => server.add_delegate(NetClient::new(&sync).to_delegate()),
			"eth" => {
				server.add_delegate(EthClient::new(&client, &sync, &secret_store, &miner).to_delegate());
				server.add_delegate(EthFilterClient::new(&client, &miner).to_delegate());
			},
			"personal" => server.add_delegate(PersonalClient::new(&secret_store).to_delegate()),
			"ethcore" => server.add_delegate(EthcoreClient::new(&miner).to_delegate()),
			_ => {
				die!("{}: Invalid API name to be enabled.", api);
			},
		}
	}
	let start_result = server.start_http(url, cors_domain);
	match start_result {
		Err(rpc::RpcServerError::IoError(err)) => die_with_io_error(err),
		Err(e) => die!("{:?}", e),
		Ok(server) => server,
	}
}

#[cfg(feature = "webapp")]
fn setup_webapp_server(
	client: Arc<Client>,
	sync: Arc<EthSync>,
	secret_store: Arc<AccountService>,
	miner: Arc<Miner>,
	url: &str,
	auth: Option<(String, String)>,
) -> WebappServer {
	use rpc::v1::*;

	let server = webapp::WebappServer::new();
	server.add_delegate(Web3Client::new().to_delegate());
	server.add_delegate(NetClient::new(&sync).to_delegate());
	server.add_delegate(EthClient::new(&client, &sync, &secret_store, &miner).to_delegate());
	server.add_delegate(EthFilterClient::new(&client, &miner).to_delegate());
	server.add_delegate(PersonalClient::new(&secret_store).to_delegate());
	server.add_delegate(EthcoreClient::new(&miner).to_delegate());
	let start_result = match auth {
		None => {
			server.start_unsecure_http(url, ::num_cpus::get())
		},
		Some((username, password)) => {
			server.start_basic_auth_http(url, ::num_cpus::get(), &username, &password)
		},
	};
	match start_result {
		Err(webapp::WebappServerError::IoError(err)) => die_with_io_error(err),
		Err(e) => die!("{:?}", e),
		Ok(handle) => handle,
	}

}

#[cfg(not(feature = "rpc"))]
struct RpcServer;

#[cfg(not(feature = "rpc"))]
fn setup_rpc_server(
	_client: Arc<Client>,
	_sync: Arc<EthSync>,
	_secret_store: Arc<AccountService>,
	_miner: Arc<Miner>,
	_url: &str,
	_cors_domain: Option<String>,
	_apis: Vec<&str>,
) -> ! {
	die!("Your Parity version has been compiled without JSON-RPC support.")
}

#[cfg(not(feature = "webapp"))]
struct WebappServer;

#[cfg(not(feature = "webapp"))]
fn setup_webapp_server(
	_client: Arc<Client>,
	_sync: Arc<EthSync>,
	_secret_store: Arc<AccountService>,
	_miner: Arc<Miner>,
	_url: &str,
	_auth: Option<(String, String)>,
) -> ! {
	die!("Your Parity version has been compiled without WebApps support.")
}

fn print_version() {
	println!("\
Parity
  version {}
Copyright 2015, 2016 Ethcore (UK) Limited
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

By Wood/Paronyan/Kotewicz/Drwięga/Volf.\
", version());
}

struct Configuration {
	args: Args
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

	fn execute(&self) {
		if self.args.flag_version {
			print_version();
			return;
		}
		if self.args.cmd_daemon {
			Daemonize::new()
				.pid_file(self.args.arg_pid_file.clone())
				.chown_pid_file(true)
				.start()
				.unwrap_or_else(|e| die!("Couldn't daemonize; {}", e));
		}
		if self.args.cmd_account {
			self.execute_account_cli();
			return;
		}
		self.execute_client();
	}

	fn execute_account_cli(&self) {
		use util::keys::store::SecretStore;
		use rpassword::read_password;
		let mut secret_store = SecretStore::new_in(Path::new(&self.keys_path()));
		if self.args.cmd_new {
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
		if self.args.cmd_list {
			println!("Known addresses:");
			for &(addr, _) in &secret_store.accounts().unwrap() {
				println!("{:?}", addr);
			}
		}
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

	fn execute_client(&self) {
		// Setup panic handler
		let panic_handler = PanicHandler::new_in_arc();

		// Setup logging
		setup_log(&self.args.flag_logging);
		// Raise fdlimit
		unsafe { ::fdlimit::raise_fd_limit(); }

		let spec = self.spec();
		let net_settings = self.net_settings(&spec);
		let sync_config = self.sync_config(&spec);
		let client_config = self.client_config(&spec);

		// Secret Store
		let account_service = Arc::new(self.account_service());

		// Build client
		let mut service = ClientService::start(
			client_config, spec, net_settings, &Path::new(&self.path())
		).unwrap_or_else(|e| die_with_error(e));

		panic_handler.forward_from(&service);
		let client = service.client();

		// Miner
		let miner = Miner::new(self.args.flag_force_sealing);
		miner.set_author(self.author());
		miner.set_gas_floor_target(self.gas_floor_target());
		miner.set_extra_data(self.extra_data());
		miner.set_minimal_gas_price(self.gas_price());

		// Sync
		let sync = EthSync::register(service.network(), sync_config, client.clone(), miner.clone());

		// Setup rpc
		let rpc_server = if self.args.flag_jsonrpc || self.args.flag_rpc {
			let apis = self.args.flag_rpcapi.as_ref().unwrap_or(&self.args.flag_jsonrpc_apis);
			let url = format!("{}:{}",
				match self.args.flag_rpcaddr.as_ref().unwrap_or(&self.args.flag_jsonrpc_interface).as_str() {
					"all" => "0.0.0.0",
					"local" => "127.0.0.1",
					x => x,
				},
				self.args.flag_rpcport.unwrap_or(self.args.flag_jsonrpc_port)
			);
			let addr = SocketAddr::from_str(&url).unwrap_or_else(|_| die!("{}: Invalid JSONRPC listen host/port given.", url));
			let cors_domain = self.args.flag_jsonrpc_cors.clone().or(self.args.flag_rpccorsdomain.clone());

			Some(setup_rpc_server(
				service.client(),
				sync.clone(),
				account_service.clone(),
				miner.clone(),
				&addr,
				cors_domain,
				apis.split(',').collect()
			))
		} else {
			None
		};

		let webapp_server = if self.args.flag_webapp {
			let url = format!("{}:{}",
				match self.args.flag_webapp_interface.as_str() {
					"all" => "0.0.0.0",
					"local" => "127.0.0.1",
					x => x,
				},
				self.args.flag_webapp_port
			);
			let auth = self.args.flag_webapp_user.as_ref().map(|username| {
				let password = self.args.flag_webapp_pass.as_ref().map_or_else(|| {
					use rpassword::read_password;
					println!("Type password for WebApps server (user: {}): ", username);
					let pass = read_password().unwrap();
					println!("OK, got it. Starting server...");
					pass
				}, |pass| pass.to_owned());
				(username.to_owned(), password)
			});

			Some(setup_webapp_server(
				service.client(),
				sync.clone(),
				account_service.clone(),
				miner.clone(),
				&url,
				auth,
			))
		} else {
			None
		};

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

fn die_with_error(e: ethcore::error::Error) -> ! {
	use ethcore::error::Error;

	match e {
		Error::Util(UtilError::StdIo(e)) => die_with_io_error(e),
		_ => die!("{:?}", e),
	}
}
fn die_with_io_error(e: std::io::Error) -> ! {
	match e.kind() {
		std::io::ErrorKind::PermissionDenied => {
			die!("No permissions to bind to specified port.")
		},
		std::io::ErrorKind::AddrInUse => {
			die!("Specified address is already in use. Please make sure that nothing is listening on the same port or try using a different one.")
		},
		std::io::ErrorKind::AddrNotAvailable => {
			die!("Could not use specified interface or given address is invalid.")
		},
		_ => die!("{:?}", e),
	}
}

fn main() {
	match ::upgrade::upgrade() {
		Ok(upgrades_applied) => {
			if upgrades_applied > 0 {
				println!("Executed {} upgrade scripts - ok", upgrades_applied);
			}
		},
		Err(e) => {
			die!("Error upgrading parity data: {:?}", e);
		}
	}

	Configuration::parse().execute();
}

struct Informant {
	chain_info: RwLock<Option<BlockChainInfo>>,
	cache_info: RwLock<Option<BlockChainCacheSize>>,
	report: RwLock<Option<ClientReport>>,
}

impl Default for Informant {
	fn default() -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
		}
	}
}

impl Informant {
	fn format_bytes(b: usize) -> String {
		match binary_prefix(b as f64) {
			Standalone(bytes)   => format!("{} bytes", bytes),
			Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
		}
	}

	pub fn tick(&self, client: &Client, sync: &EthSync) {
		// 5 seconds betwen calls. TODO: calculate this properly.
		let dur = 5usize;

		let chain_info = client.chain_info();
		let queue_info = client.queue_info();
		let cache_info = client.blockchain_cache_info();
		let sync_info = sync.status();

		let mut write_report = self.report.write().unwrap();
		let report = client.report();

		if let (_, _, &Some(ref last_report)) = (
			self.chain_info.read().unwrap().deref(),
			self.cache_info.read().unwrap().deref(),
			write_report.deref()
		) {
			println!("[ #{} {} ]---[ {} blk/s | {} tx/s | {} gas/s  //··· {}/{} peers, #{}, {}+{} queued ···// mem: {} db, {} chain, {} queue, {} sync ]",
				chain_info.best_block_number,
				chain_info.best_block_hash,
				(report.blocks_imported - last_report.blocks_imported) / dur,
				(report.transactions_applied - last_report.transactions_applied) / dur,
				(report.gas_processed - last_report.gas_processed) / From::from(dur),

				sync_info.num_active_peers,
				sync_info.num_peers,
				sync_info.last_imported_block_number.unwrap_or(chain_info.best_block_number),
				queue_info.unverified_queue_size,
				queue_info.verified_queue_size,

				Informant::format_bytes(report.state_db_mem),
				Informant::format_bytes(cache_info.total()),
				Informant::format_bytes(queue_info.mem_used),
				Informant::format_bytes(sync_info.mem_used),
			);
		}

		*self.chain_info.write().unwrap().deref_mut() = Some(chain_info);
		*self.cache_info.write().unwrap().deref_mut() = Some(cache_info);
		*write_report.deref_mut() = Some(report);
	}
}

const INFO_TIMER: TimerToken = 0;

const ACCOUNT_TICK_TIMER: TimerToken = 10;
const ACCOUNT_TICK_MS: u64 = 60000;

struct ClientIoHandler {
	client: Arc<Client>,
	sync: Arc<EthSync>,
	accounts: Arc<AccountService>,
	info: Informant,
}

impl IoHandler<NetSyncMessage> for ClientIoHandler {
	fn initialize(&self, io: &IoContext<NetSyncMessage>) {
		io.register_timer(INFO_TIMER, 5000).expect("Error registering timer");
		io.register_timer(ACCOUNT_TICK_TIMER, ACCOUNT_TICK_MS).expect("Error registering account timer");

	}

	fn timeout(&self, _io: &IoContext<NetSyncMessage>, timer: TimerToken) {
		match timer {
			INFO_TIMER => { self.info.tick(&self.client, &self.sync); }
			ACCOUNT_TICK_TIMER => { self.accounts.tick(); },
			_ => {}
		}
	}
}

/// Parity needs at least 1 test to generate coverage reports correctly.
#[test]
fn if_works() {
}
