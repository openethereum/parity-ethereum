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
#![cfg_attr(feature="dev", allow(match_bool))]

extern crate docopt;
extern crate num_cpus;
extern crate rustc_serialize;
extern crate ethcore_util as util;
extern crate ethcore;
extern crate ethsync;
#[macro_use]
extern crate log as rlog;
extern crate env_logger;
extern crate ctrlc;
extern crate fdlimit;
extern crate time;
extern crate number_prefix;
extern crate rpassword;
extern crate semver;
extern crate ethcore_ipc as ipc;
extern crate ethcore_ipc_nano as nanoipc;
#[macro_use]
extern crate hyper; // for price_info.rs
extern crate json_ipc_server as jsonipc;

extern crate ethcore_ipc_hypervisor as hypervisor;
extern crate ethcore_rpc;

extern crate ethcore_signer;
extern crate ansi_term;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate isatty;

#[cfg(feature = "dapps")]
extern crate ethcore_dapps;

mod commands;
mod cache;
mod upgrade;
mod setup_log;
mod rpc;
mod dapps;
mod informant;
mod io_handler;
mod cli;
mod configuration;
mod migration;
mod signer;
mod rpc_apis;
mod url;
mod helpers;
mod params;
mod deprecated;
mod dir;
mod modules;

use std::sync::{Arc, Mutex, Condvar};
use std::path::Path;
use std::env;
use ctrlc::CtrlC;
use util::{Colour, version, H256, NetworkConfiguration, U256};
use util::journaldb::Algorithm;
use util::panics::{MayPanic, ForwardPanic, PanicHandler};
use ethcore::client::{Mode, ClientConfig, ChainNotify};
use ethcore::service::ClientService;
use ethcore::spec::Spec;
use ethsync::EthSync;
use ethcore::miner::{Miner, MinerService, ExternalMiner, MinerOptions};
use ethsync::SyncConfig;
use migration::migrate;
use informant::Informant;

use rpc::{RpcServer, HttpConfiguration, IpcConfiguration};
use signer::SignerServer;
use dapps::WebappServer;
use io_handler::ClientIoHandler;
use configuration::{Configuration, IOPasswordReader};
use helpers::{to_mode, to_address, to_u256};
use params::{SpecType, Pruning, AccountsConfig, GasPricerConfig};
use dir::Directories;
use setup_log::{LoggerConfig, setup_log};
use fdlimit::raise_fd_limit;
use std::process;

fn main() {
	let conf = Configuration::parse(env::args()).unwrap_or_else(|e| e.exit());
	match new_execute(conf) {
		Ok(result) => {
			print!("{}", result);
		},
		Err(err) => {
			print!("{}", err);
			process::exit(1);
		}
	}
}

fn new_execute(conf: Configuration) -> Result<String, String> {
	let cmd = try!(conf.into_command(&IOPasswordReader));
	commands::execute(cmd)
}

#[derive(Debug, PartialEq)]
pub struct RunCmd {
	directories: Directories,
	spec: SpecType,
	pruning: Pruning,
	/// Some if execution should be daemonized. Contains pid_file path.
	daemon: Option<String>,
	logger_config: LoggerConfig,
	miner_options: MinerOptions,
	http_conf: HttpConfiguration,
	ipc_conf: IpcConfiguration,
	net_conf: NetworkConfiguration,
	network_id: Option<U256>,
	acc_conf: AccountsConfig,
	gas_pricer: GasPricerConfig,
	extra_data: Vec<u8>,
}

fn execute(cmd: RunCmd) -> Result<(), String> {
	// increase max number of open files
	raise_fd_limit();

	// set up logger
	let _logger = setup_log(&cmd.logger_config);

	// set up panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// create directories used by parity
	try!(cmd.directories.create_dirs());

	// load spec
	let spec = try!(cmd.spec.spec());

	// store genesis hash
	let genesis_hash = spec.genesis_header().hash();

	// select pruning algorithm
	let algorithm = cmd.pruning.to_algorithm(&cmd.directories, genesis_hash);

	// execute upgrades
	try!(execute_upgrades(&cmd.directories, genesis_hash, algorithm));

	// run in daemon mode
	if let Some(pid_file) = cmd.daemon {
		try!(daemonize(pid_file));
	}

	// display warning about using experimental journaldb alorithm
	if !algorithm.is_stable() {
		warn!("Your chosen strategy is {}! You can re-run with --pruning to change.", Colour::Red.bold().paint("unstable"));
	}

	// create sync config
	let mut sync_config = SyncConfig::default();
	sync_config.network_id = match cmd.network_id {
		Some(id) => id,
		None => spec.network_id(),
	};

	Ok(())
}

#[cfg(not(windows))]
fn daemonize(pid_file: String) -> Result<(), String> {
	extern crate daemonize;

	daemonize::Daemonize::new()
			.pid_file(pid_file)
			.chown_pid_file(true)
			.start()
			.map(|_| ())
			.map_err(|e| format!("Couldn't daemonize; {}", e))
}

#[cfg(windows)]
fn daemonize(_conf: &Configuration) -> ! {
}

fn execute_upgrades(dirs: &Directories, genesis_hash: H256, pruning: Algorithm) -> Result<(), String> {
	match upgrade::upgrade(Some(&dirs.db)) {
		Ok(upgrades_applied) if upgrades_applied > 0 => {
			debug!("Executed {} upgrade scripts - ok", upgrades_applied);
		},
		Err(e) => {
			return Err(format!("Error upgrading parity data: {:?}", e));
		},
		_ => {},
	}

	let client_path = dirs.client_path(genesis_hash, pruning);
	migrate(&client_path, pruning).map_err(|e| format!("{}", e))
}

fn execute_client(conf: Configuration, spec: Spec, client_config: ClientConfig) -> Result<(), String> {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Setup logging
	//let logger = setup_log::setup_log(&conf.args.flag_logging, conf.have_color());
	let logger = try!(setup_log::setup_log({
		unimplemented!()
	}));
	// Raise fdlimit
	unsafe { ::fdlimit::raise_fd_limit(); }

	info!("Starting {}", Colour::White.bold().paint(version()));
	info!("Using state DB journalling strategy {}", Colour::White.bold().paint(client_config.pruning.as_str()));

	// Display warning about using experimental journaldb types
	if !client_config.pruning.is_stable() {
		warn!("Your chosen strategy is {}! You can re-run with --pruning to change.", Colour::Red.bold().paint("unstable"));
	}

	// Display warning about using unlock with signer
	if conf.signer_enabled() && conf.args.flag_unlock.is_some() {
		warn!("Using Trusted Signer and --unlock is not recommended!");
		warn!("NOTE that Signer will not ask you to confirm transactions from unlocked account.");
	}

	let net_settings = { unimplemented!() }; //try!(conf.net_settings());
	let sync_config = { unimplemented!() };

	// Secret Store
	//let account_service = Arc::new(conf.account_service());
	let account_service = { unimplemented!() };

	// Miner
	let miner_options = try!(conf.miner_options());
	let miner = { unimplemented!() }; // Miner::new(miner_options, conf.gas_pricer().expect("TODO!"), conf.spec(), Some(account_service.clone()));
	//miner.set_author(try!(conf.author()));
	//miner.set_author(try!(to_address(conf.args.flag_etherbase.clone().or(conf.args.flag_author.clone()))));
	//miner.set_gas_floor_target(try!(to_u256(&conf.args.flag_gas_floor_target)));
	//miner.set_gas_ceil_target(try!(to_u256(&conf.args.flag_gas_cap)));
	//miner.set_extra_data(try!(conf.extra_data()));
	//miner.set_transactions_limit(conf.args.flag_tx_queue_size);

	//let directories = conf.directories();

	// Build client
	//let mut service = try!(ClientService::start(
		//client_config,
		//spec,
		//Path::new(&directories.db),
		//miner.clone(),
	//).map_err(|e| format!("Client service error: {:?}", e)));

	//panic_handler.forward_from(&service);
	//let client = service.client();

	//let external_miner = Arc::new(ExternalMiner::default());
	////let network_settings = Arc::new(conf.network_settings());

	//// Sync
	//let sync = try!(EthSync::new(sync_config, client.clone(), net_settings.into()).map_err(|_| "Error registering eth protocol handler"));
	//service.set_notify(&(sync.clone() as Arc<ChainNotify>));

	//// if network is active by default
	//let enable_network = match try!(to_mode(&conf.args.flag_mode, conf.args.flag_mode_timeout, conf.args.flag_mode_alarm)) {
		//Mode::Dark(..) => false,
		//_ => !conf.args.flag_no_network,
	//};

	//if enable_network {
		//sync.start();
	//}

	//let deps_for_rpc_apis = Arc::new(rpc_apis::Dependencies {
		//signer_port: conf.signer_port(),
		//signer_queue: Arc::new(rpc_apis::ConfirmationsQueue::default()),
		//client: client.clone(),
		//sync: sync.clone(),
		//secret_store: account_service.clone(),
		//miner: miner.clone(),
		//external_miner: external_miner.clone(),
		//logger: logger.clone(),
		//settings: network_settings.clone(),
		//allow_pending_receipt_query: !conf.args.flag_geth,
		//net_service: sync.clone(),
	//});

	//let dependencies = rpc::Dependencies {
		//panic_handler: panic_handler.clone(),
		//apis: deps_for_rpc_apis.clone(),
	//};

	//// Setup http rpc
	//let rpc_conf = rpc::HttpConfiguration {
		//enabled: network_settings.rpc_enabled,
		//interface: conf.rpc_interface(),
		//port: network_settings.rpc_port,
		//apis: try!(conf.rpc_apis().parse()),
		//cors: conf.rpc_cors(),
	//};

	//let rpc_server = try!(rpc::new_http(rpc_conf, &dependencies));

	//// setup ipc rpc
	//let ipc_settings = { unimplemented!() };// try!(conf.ipc_settings());
	////debug!("IPC: {}", ipc_settings);
	//let _ipc_server = rpc::new_ipc(ipc_settings, &dependencies);

	//// Set up dapps
	//let dapps_conf = dapps::Configuration {
		//enabled: conf.dapps_enabled(),
		//interface: conf.dapps_interface(),
		//port: conf.args.flag_dapps_port,
		//user: conf.args.flag_dapps_user.clone(),
		//pass: conf.args.flag_dapps_pass.clone(),
		//dapps_path: directories.dapps,
	//};

	//let dapps_deps = dapps::Dependencies {
		//panic_handler: panic_handler.clone(),
		//apis: deps_for_rpc_apis.clone(),
	//};

	//let dapps_server = try!(dapps::new(dapps_conf, dapps_deps));

	//// Set up a signer
	//let signer_conf = signer::Configuration {
		//enabled: conf.signer_enabled(),
		//port: conf.args.flag_signer_port,
		//signer_path: directories.signer,
	//};

	//let signer_deps = signer::Dependencies {
		//panic_handler: panic_handler.clone(),
		//apis: deps_for_rpc_apis.clone(),
	//};

	//let signer_server = try!(signer::start(signer_conf, signer_deps));

	//// Register IO handler
	//let io_handler = Arc::new(ClientIoHandler {
		//client: service.client(),
		//info: Informant::new(conf.have_color()),
		//sync: sync.clone(),
		//accounts: account_service.clone(),
	//});
	//service.register_io_handler(io_handler).expect("Error registering IO handler");

	//if conf.args.cmd_ui {
		//if !conf.dapps_enabled() {
			//return Err("Cannot use UI command with Dapps turned off.".into())
		//}
		//url::open(&format!("http://{}:{}/", conf.dapps_interface(), conf.args.flag_dapps_port));
	//}

	// Handle exit
	//wait_for_exit(panic_handler, rpc_server, dapps_server, signer_server);
	Ok(())
}

fn wait_for_exit(
	panic_handler: Arc<PanicHandler>,
	_rpc_server: Option<RpcServer>,
	_dapps_server: Option<WebappServer>,
	_signer_server: Option<SignerServer>
	) {
	let exit = Arc::new(Condvar::new());

	// Handle possible exits
	let e = exit.clone();
	CtrlC::set_handler(move || { e.notify_all(); });

	// Handle panics
	let e = exit.clone();
	panic_handler.on_panic(move |_reason| { e.notify_all(); });

	// Wait for signal
	let mutex = Mutex::new(());
	let _ = exit.wait(mutex.lock().unwrap());
	info!("Finishing work, please wait...");
}
