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

#[cfg(feature = "dapps")]
extern crate ethcore_dapps;

mod commands;
mod cache;
#[macro_use]
mod die;
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
mod params;

use std::sync::{Arc, Mutex, Condvar};
use std::path::Path;
use std::env;
use ctrlc::CtrlC;
use util::{UtilError, Colour, Applyable, version, Lockable};
use util::panics::{MayPanic, ForwardPanic, PanicHandler};
use ethcore::client::{Mode, ClientConfig, get_db_path};
use ethcore::service::ClientService;
use ethcore::spec::Spec;
use ethsync::EthSync;
use ethcore::miner::{Miner, MinerService, ExternalMiner};
use migration::migrate;
use informant::Informant;

use die::*;
use rpc::RpcServer;
use signer::SignerServer;
use dapps::WebappServer;
use io_handler::ClientIoHandler;
use configuration::{Policy, Configuration, IOPasswordReader};
use params::to_mode;
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

fn execute(conf: Configuration) -> Result<(), String> {
	let spec = conf.spec();
	let client_config = conf.client_config(&spec);

	try!(execute_upgrades(&conf, &spec, &client_config));

	if conf.args.cmd_daemon {
		try!(daemonize(&conf));
	}

	execute_client(conf, spec, client_config)
}

#[cfg(not(windows))]
fn daemonize(conf: &Configuration) -> Result<(), String> {
	extern crate daemonize;

	daemonize::Daemonize::new()
			.pid_file(conf.args.arg_pid_file.clone())
			.chown_pid_file(true)
			.start()
			.map(|_| ())
			.map_err(|e| format!("Couldn't daemonize; {}", e))
}

#[cfg(windows)]
fn daemonize(_conf: &Configuration) -> ! {
}

fn execute_upgrades(conf: &Configuration, spec: &Spec, client_config: &ClientConfig) -> Result<(), String> {
	match upgrade::upgrade(Some(&conf.path())) {
		Ok(upgrades_applied) if upgrades_applied > 0 => {
			println!("Executed {} upgrade scripts - ok", upgrades_applied);
		},
		Err(e) => {
			return Err(format!("Error upgrading parity data: {:?}", e));
		},
		_ => {},
	}

	let db_path = get_db_path(Path::new(&conf.path()), client_config.pruning, spec.genesis_header().hash());
	migrate(&db_path, client_config.pruning).map_err(|e| format!("{}", e))
}

fn execute_client(conf: Configuration, spec: Spec, client_config: ClientConfig) -> Result<(), String> {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Setup logging
	let logger = setup_log::setup_log(&conf.args.flag_logging, conf.have_color());
	// Raise fdlimit
	unsafe { ::fdlimit::raise_fd_limit(); }

	info!("Starting {}", format!("{}", version()).apply(Colour::White.bold()));
	info!("Using state DB journalling strategy {}", client_config.pruning.as_str().apply(Colour::White.bold()));

	// Display warning about using experimental journaldb types
	if !client_config.pruning.is_stable() {
		warn!("Your chosen strategy is {}! You can re-run with --pruning to change.", "unstable".apply(Colour::Red.bold()));
	}

	// Display warning about using unlock with signer
	if conf.signer_enabled() && conf.args.flag_unlock.is_some() {
		warn!("Using Trusted Signer and --unlock is not recommended!");
		warn!("NOTE that Signer will not ask you to confirm transactions from unlocked account.");
	}

	// Check fork settings.
	if conf.policy() != Policy::None {
		warn!("Value given for --policy, yet no proposed forks exist. Ignoring.");
	}

	let net_settings = conf.net_settings(&spec);
	let sync_config = conf.sync_config(&spec);

	// Secret Store
	let account_service = Arc::new(conf.account_service());

	// Miner
	let miner = Miner::new(conf.miner_options(), conf.gas_pricer().expect("TODO!"), conf.spec(), Some(account_service.clone()));
	miner.set_author(conf.author().unwrap_or_default());
	miner.set_gas_floor_target(conf.gas_floor_target());
	miner.set_gas_ceil_target(conf.gas_ceil_target());
	miner.set_extra_data(conf.extra_data());
	miner.set_transactions_limit(conf.args.flag_tx_queue_size);

	// Build client
	let mut service = ClientService::start(
		client_config,
		spec,
		net_settings,
		Path::new(&conf.path()),
		miner.clone(),
		//match conf.mode() { Mode::Dark(..) => false, _ => !conf.args.flag_no_network }
		//match conf.mode().unwrap() { Mode::Dark(..) => false, _ => !conf.args.flag_no_network }
		match to_mode(&conf.args.flag_mode, conf.args.flag_mode_timeout, conf.args.flag_mode_alarm).unwrap() {
			Mode::Dark(..) => false,
			_ => !conf.args.flag_no_network,
		}
	).unwrap_or_else(|e| die_with_error("Client", e));

	panic_handler.forward_from(&service);
	let client = service.client();

	let external_miner = Arc::new(ExternalMiner::default());
	let network_settings = Arc::new(conf.network_settings());

	// Sync
	let sync = EthSync::new(sync_config, client.clone());
	EthSync::register(&*service.network(), sync.clone()).unwrap_or_else(|e| die_with_error("Error registering eth protocol handler", UtilError::from(e).into()));

	let deps_for_rpc_apis = Arc::new(rpc_apis::Dependencies {
		signer_port: conf.signer_port(),
		signer_queue: Arc::new(rpc_apis::ConfirmationsQueue::default()),
		client: client.clone(),
		sync: sync.clone(),
		secret_store: account_service.clone(),
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: network_settings.clone(),
		allow_pending_receipt_query: !conf.args.flag_geth,
		net_service: service.network(),
	});

	let dependencies = rpc::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
	};

	// Setup http rpc
	let rpc_server = rpc::new_http(rpc::HttpConfiguration {
		enabled: network_settings.rpc_enabled,
		interface: conf.rpc_interface(),
		port: network_settings.rpc_port,
		apis: conf.rpc_apis(),
		cors: conf.rpc_cors(),
	}, &dependencies);

	// setup ipc rpc
	let _ipc_server = rpc::new_ipc(conf.ipc_settings(), &dependencies);
	debug!("IPC: {}", conf.ipc_settings());

	if conf.args.flag_webapp { println!("WARNING: Flag -w/--webapp is deprecated. Dapps server is now on by default. Ignoring."); }
	let dapps_server = dapps::new(dapps::Configuration {
		enabled: conf.dapps_enabled(),
		interface: conf.dapps_interface(),
		port: conf.args.flag_dapps_port,
		user: conf.args.flag_dapps_user.clone(),
		pass: conf.args.flag_dapps_pass.clone(),
		dapps_path: conf.directories().dapps,
	}, dapps::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
	});

	// Set up a signer
	let signer_conf = signer::Configuration {
		enabled: conf.signer_enabled(),
		port: conf.args.flag_signer_port,
		signer_path: conf.directories().signer,
	};

	let signer_deps = signer::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
	};

	let signer_server = try!(signer::start(signer_conf, signer_deps));

	// Register IO handler
	let io_handler = Arc::new(ClientIoHandler {
		client: service.client(),
		info: Informant::new(conf.have_color()),
		sync: sync.clone(),
		accounts: account_service.clone(),
		network: Arc::downgrade(&service.network()),
	});
	service.register_io_handler(io_handler).expect("Error registering IO handler");

	if conf.args.cmd_ui {
		if !conf.dapps_enabled() {
			die_with_message("Cannot use UI command with Dapps turned off.");
		}
		url::open(&format!("http://{}:{}/", conf.dapps_interface(), conf.args.flag_dapps_port));
	}

	// Handle exit
	wait_for_exit(panic_handler, rpc_server, dapps_server, signer_server);
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
	let _ = exit.wait(mutex.locked()).unwrap();
	info!("Finishing work, please wait...");
}
