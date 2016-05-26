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
#[macro_use]
extern crate hyper; // for price_info.rs
extern crate json_ipc_server as jsonipc;

#[cfg(feature = "rpc")]
extern crate ethcore_rpc;

#[cfg(feature = "dapps")]
extern crate ethcore_dapps;

#[macro_use]
mod die;
mod price_info;
mod upgrade;
mod hypervisor;
mod setup_log;
mod rpc;
mod dapps;
mod informant;
mod io_handler;
mod cli;
mod configuration;

use ctrlc::CtrlC;
use util::*;
use std::time::Duration;
use std::fs::File;
use std::thread::sleep;
use std::io::{BufReader, BufRead};
use util::panics::{MayPanic, ForwardPanic, PanicHandler};
use ethcore::client::{BlockID, BlockChainClient};
use ethcore::error::{Error, ImportError};
use ethcore::service::ClientService;
use ethsync::EthSync;
use ethminer::{Miner, MinerService, ExternalMiner};
use daemonize::Daemonize;
use informant::Informant;

use die::*;
use cli::print_version;
use rpc::RpcServer;
use dapps::WebappServer;
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

	if conf.args.cmd_export {
		execute_export(conf);
		return;
	}

	if conf.args.cmd_import {
		execute_import(conf);
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
		client_config, spec, net_settings, Path::new(&conf.path())
	).unwrap_or_else(|e| die_with_error("Client", e));

	panic_handler.forward_from(&service);
	let client = service.client();

	// Miner
	let miner = Miner::with_accounts(conf.args.flag_force_sealing, conf.spec(), account_service.clone());
	miner.set_author(conf.author());
	miner.set_gas_floor_target(conf.gas_floor_target());
	miner.set_extra_data(conf.extra_data());
	miner.set_minimal_gas_price(conf.gas_price());
	miner.set_transactions_limit(conf.args.flag_tx_limit);

	let external_miner = Arc::new(ExternalMiner::default());
	let network_settings = Arc::new(conf.network_settings());

	// Sync
	let sync = EthSync::register(service.network(), sync_config, client.clone(), miner.clone());

	let dependencies = Arc::new(rpc::Dependencies {
		panic_handler: panic_handler.clone(),
		client: client.clone(),
		sync: sync.clone(),
		secret_store: account_service.clone(),
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: network_settings.clone(),
	});

	// Setup http rpc
	let rpc_server = rpc::new_http(rpc::HttpConfiguration {
		enabled: network_settings.rpc_enabled,
		interface: network_settings.rpc_interface.clone(),
		port: network_settings.rpc_port,
		apis: conf.rpc_apis(),
		cors: conf.rpc_cors(),
	}, &dependencies);

	// setup ipc rpc
	let _ipc_server = rpc::new_ipc(conf.ipc_settings(), &dependencies);

	if conf.args.flag_webapp { println!("WARNING: Flag -w/--webapp is deprecated. Dapps server is now on by default. Ignoring."); }
	let dapps_server = dapps::new(dapps::Configuration {
		enabled: !conf.args.flag_dapps_off,
		interface: conf.args.flag_dapps_interface.clone(),
		port: conf.args.flag_dapps_port,
		user: conf.args.flag_dapps_user.clone(),
		pass: conf.args.flag_dapps_pass.clone(),
	}, dapps::Dependencies {
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
		info: Informant::new(!conf.args.flag_no_color),
		sync: sync.clone(),
		accounts: account_service.clone(),
	});
	service.io().register_handler(io_handler).expect("Error registering IO handler");

	// Handle exit
	wait_for_exit(panic_handler, rpc_server, dapps_server);
}

fn flush_stdout() {
	::std::io::stdout().flush().expect("stdout is flushable; qed");
}

enum DataFormat {
	Hex,
	Binary,
}

fn execute_export(conf: Configuration) {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Raise fdlimit
	unsafe { ::fdlimit::raise_fd_limit(); }

	let spec = conf.spec();
	let net_settings = NetworkConfiguration {
		config_path: None,
		listen_address: None,
		public_address: None,
		udp_port: None,
		nat_enabled: false,
		discovery_enabled: false,
		pin: true,
		boot_nodes: Vec::new(),
		use_secret: None,
		ideal_peers: 0,
	};
	let client_config = conf.client_config(&spec);

	// Build client
	let service = ClientService::start(
		client_config, spec, net_settings, Path::new(&conf.path())
	).unwrap_or_else(|e| die_with_error("Client", e));

	panic_handler.forward_from(&service);
	let client = service.client();

	// we have a client!
	let parse_block_id = |s: &str, arg: &str| -> u64 {
		if s == "latest" {
			client.chain_info().best_block_number
		} else if let Ok(n) = s.parse::<u64>() {
			n
		} else if let Ok(h) = H256::from_str(s) {
			client.block_number(BlockID::Hash(h)).unwrap_or_else(|| {
				die!("Unknown block hash passed to {} parameter: {:?}", arg, s);
			})
		} else {
			die!("Invalid {} parameter given: {:?}", arg, s);
		}
	};
	let from = parse_block_id(&conf.args.flag_from, "--from");
	let to = parse_block_id(&conf.args.flag_to, "--to");
	let format = match conf.args.flag_format {
		Some(x) => match x.deref() {
			"binary" | "bin" => DataFormat::Binary,
			"hex" => DataFormat::Hex,
			x => die!("Invalid --format parameter given: {:?}", x),
		},
		None if conf.args.arg_file.is_none() => DataFormat::Hex,
		None => DataFormat::Binary,
	};

	let mut out: Box<Write> = if let Some(f) = conf.args.arg_file {
		Box::new(File::create(&f).unwrap_or_else(|_| die!("Cannot write to file given: {}", f)))
	} else {
		Box::new(::std::io::stdout())
	};

	for i in from..(to + 1) {
		let b = client.deref().block(BlockID::Number(i)).unwrap();
		match format {
			DataFormat::Binary => { out.write(&b).expect("Couldn't write to stream."); }
			DataFormat::Hex => { out.write_fmt(format_args!("{}", b.pretty())).expect("Couldn't write to stream."); }
		}
	}
}

fn execute_import(conf: Configuration) {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Raise fdlimit
	unsafe { ::fdlimit::raise_fd_limit(); }

	let spec = conf.spec();
	let net_settings = NetworkConfiguration {
		config_path: None,
		listen_address: None,
		public_address: None,
		udp_port: None,
		nat_enabled: false,
		discovery_enabled: false,
		pin: true,
		boot_nodes: Vec::new(),
		use_secret: None,
		ideal_peers: 0,
	};
	let client_config = conf.client_config(&spec);

	// Build client
	let service = ClientService::start(
		client_config, spec, net_settings, Path::new(&conf.path())
	).unwrap_or_else(|e| die_with_error("Client", e));

	panic_handler.forward_from(&service);
	let client = service.client();

	let mut instream: Box<Read> = if let Some(f) = conf.args.arg_file {
		let f = File::open(&f).unwrap_or_else(|_| die!("Cannot open the file given: {}", f));
		Box::new(f)
	} else {
		Box::new(::std::io::stdin())
	};

	let mut first_bytes: Bytes = vec![0; 3];
	let mut first_read = 0;

	let format = match conf.args.flag_format {
		Some(x) => match x.deref() {
			"binary" | "bin" => DataFormat::Binary,
			"hex" => DataFormat::Hex,
			x => die!("Invalid --format parameter given: {:?}", x),
		},
		None => {
			// autodetect...
			first_read = instream.read(&mut(first_bytes[..])).unwrap_or_else(|_| die!("Error reading from the file/stream."));
			match first_bytes[0] {
				0xf9 => {
					println!("Autodetected binary data format.");
					DataFormat::Binary
				}
				_ => {
					println!("Autodetected hex data format.");
					DataFormat::Hex
				}
			}
		}
	};

	let informant = Informant::new(!conf.args.flag_no_color);

	let do_import = |bytes| {
		while client.queue_info().is_full() { sleep(Duration::from_secs(1)); }
		match client.import_block(bytes) {
			Ok(_) => {}
			Err(Error::Import(ImportError::AlreadyInChain)) => { trace!("Skipping block already in chain."); }
			Err(e) => die!("Cannot import block: {:?}", e)
		}
		informant.tick(client.deref(), None);
	};

	match format {
		DataFormat::Binary => {
			loop {
				let mut bytes: Bytes = if first_read > 0 {first_bytes.clone()} else {vec![0; 3]};
				let n = if first_read > 0 {first_read} else {instream.read(&mut(bytes[..])).unwrap_or_else(|_| die!("Error reading from the file/stream."))};
				if n == 0 { break; }
				first_read = 0;
				let s = PayloadInfo::from(&(bytes[..])).unwrap_or_else(|e| die!("Invalid RLP in the file/stream: {:?}", e)).total();
				bytes.resize(s, 0);
				instream.read_exact(&mut(bytes[3..])).unwrap_or_else(|_| die!("Error reading from the file/stream."));
				do_import(bytes);
			}
		}
		DataFormat::Hex => {
			for line in BufReader::new(instream).lines() {
				let s = line.unwrap_or_else(|_| die!("Error reading from the file/stream."));
				let s = if first_read > 0 {str::from_utf8(&first_bytes).unwrap().to_owned() + &(s[..])} else {s};
				first_read = 0;
				let bytes = FromHex::from_hex(&(s[..])).unwrap_or_else(|_| die!("Invalid hex in file/stream."));
				do_import(bytes);
			}
		}
	}
	client.flush_queue();
}

fn execute_account_cli(conf: Configuration) {
	use util::keys::store::SecretStore;
	use rpassword::read_password;
	let mut secret_store = SecretStore::with_security(Path::new(&conf.keys_path()), conf.keys_iterations());
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

fn wait_for_exit(panic_handler: Arc<PanicHandler>, _rpc_server: Option<RpcServer>, _dapps_server: Option<WebappServer>) {
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
