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
#[macro_use]
extern crate hyper; // for price_info.rs

#[cfg(feature = "rpc")]
extern crate ethcore_rpc;

#[cfg(feature = "webapp")]
extern crate ethcore_webapp;

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
mod configuration;

use ctrlc::CtrlC;
use util::*;
use util::panics::{MayPanic, ForwardPanic, PanicHandler};
use ethcore::service::ClientService;
use ethsync::EthSync;
use ethminer::{Miner, MinerService, ExternalMiner};
use daemonize::Daemonize;

use die::*;
use cli::print_version;
use rpc::RpcServer;
use webapp::WebappServer;
use io_handler::ClientIoHandler;
use configuration::Configuration;

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
	).unwrap_or_else(|e| die_with_error("Client", e));

	panic_handler.forward_from(&service);
	let client = service.client();

	// Miner
	let miner = Miner::new(conf.args.flag_force_sealing);
	miner.set_author(conf.author());
	miner.set_gas_floor_target(conf.gas_floor_target());
	miner.set_extra_data(conf.extra_data());
	miner.set_minimal_gas_price(conf.gas_price());
	miner.set_transactions_limit(conf.args.flag_tx_limit);

	let external_miner = Arc::new(ExternalMiner::default());
	let network_settings = Arc::new(conf.network_settings());

	// Sync
	let sync = EthSync::register(service.network(), sync_config, client.clone(), miner.clone());

	// Setup rpc
	let rpc_server = rpc::new(rpc::Configuration {
		enabled: network_settings.rpc_enabled,
		interface: network_settings.rpc_interface.clone(),
		port: network_settings.rpc_port,
		apis: conf.rpc_apis(),
		cors: conf.rpc_cors(),
	}, rpc::Dependencies {
		panic_handler: panic_handler.clone(),
		client: client.clone(),
		sync: sync.clone(),
		secret_store: account_service.clone(),
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: network_settings.clone(),
	});

	let webapp_server = webapp::new(webapp::Configuration {
		enabled: conf.args.flag_webapp,
		interface: conf.args.flag_webapp_interface.clone(),
		port: conf.args.flag_webapp_port,
		user: conf.args.flag_webapp_user.clone(),
		pass: conf.args.flag_webapp_pass.clone(),
	}, webapp::Dependencies {
		panic_handler: panic_handler.clone(),
		client: client.clone(),
		sync: sync.clone(),
		secret_store: account_service.clone(),
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: network_settings.clone(),
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

fn flush_stdout() {
	::std::io::stdout().flush().ok().expect("stdout is flushable; qed");
}

fn execute_account_cli(conf: Configuration) {
	use util::keys::store::SecretStore;
	use rpassword::read_password;
	let mut secret_store = SecretStore::new_in(Path::new(&conf.keys_path()));
	if conf.args.cmd_new {
		println!("Please note that password is NOT RECOVERABLE.");
		print!("Type password: ");
		flush_stdout();
		let password = read_password().unwrap();
		print!("Repeat password: ");
		flush_stdout();
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

/// Parity needs at least 1 test to generate coverage reports correctly.
#[test]
fn if_works() {
}
