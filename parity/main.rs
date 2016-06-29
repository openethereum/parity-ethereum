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
#[macro_use]
extern crate log as rlog;
extern crate env_logger;
extern crate ctrlc;
extern crate fdlimit;
#[cfg(not(windows))]
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

#[cfg(feature = "ethcore-signer")]
extern crate ethcore_signer;

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
mod migration;
mod signer;
mod rpc_apis;
mod url;

use std::io::{Write, Read, BufReader, BufRead};
use std::ops::Deref;
use std::sync::{Arc, Mutex, Condvar};
use std::path::Path;
use std::fs::File;
use std::str::{FromStr, from_utf8};
use std::thread::sleep;
use std::time::Duration;
use rustc_serialize::hex::FromHex;
use ctrlc::CtrlC;
use util::{H256, ToPretty, NetworkConfiguration, PayloadInfo, Bytes, UtilError, paint, Colour, version};
use util::panics::{MayPanic, ForwardPanic, PanicHandler};
use ethcore::client::{BlockID, BlockChainClient, ClientConfig, get_db_path};
use ethcore::error::{Error, ImportError};
use ethcore::service::ClientService;
use ethcore::spec::Spec;
use ethsync::EthSync;
use ethcore::miner::{Miner, MinerService, ExternalMiner};
use migration::migrate;
use informant::Informant;

use die::*;
use cli::print_version;
use rpc::RpcServer;
use signer::{SignerServer, new_token};
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

	if conf.args.cmd_signer {
		execute_signer(conf);
		return;
	}

	let spec = conf.spec();
	let client_config = conf.client_config(&spec);

	execute_upgrades(&conf, &spec, &client_config);

	if conf.args.cmd_daemon {
		daemonize(&conf);
	}

	if conf.args.cmd_account {
		execute_account_cli(conf);
		return;
	}

	if conf.args.cmd_wallet {
		execute_wallet_cli(conf);
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

	execute_client(conf, spec, client_config);
}

#[cfg(not(windows))]
fn daemonize(conf: &Configuration) {
	use daemonize::Daemonize;
	Daemonize::new()
			.pid_file(conf.args.arg_pid_file.clone())
			.chown_pid_file(true)
			.start()
			.unwrap_or_else(|e| die!("Couldn't daemonize; {}", e));
}

#[cfg(windows)]
fn daemonize(_conf: &Configuration) {
}

fn execute_upgrades(conf: &Configuration, spec: &Spec, client_config: &ClientConfig) {
	match ::upgrade::upgrade(Some(&conf.path())) {
		Ok(upgrades_applied) if upgrades_applied > 0 => {
			println!("Executed {} upgrade scripts - ok", upgrades_applied);
		},
		Err(e) => {
			die!("Error upgrading parity data: {:?}", e);
		},
		_ => {},
	}

	let db_path = get_db_path(Path::new(&conf.path()), client_config.pruning, spec.genesis_header().hash());
	let result = migrate(&db_path);
	if let Err(err) = result {
		die_with_message(&format!("{}", err));
	}
}

fn execute_client(conf: Configuration, spec: Spec, client_config: ClientConfig) {
	// Setup panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// Setup logging
	let logger = setup_log::setup_log(&conf.args.flag_logging, !conf.args.flag_no_color);
	// Raise fdlimit
	unsafe { ::fdlimit::raise_fd_limit(); }

	info!("Starting {}", paint(Colour::White.bold(), format!("{}", version())));

	let net_settings = conf.net_settings(&spec);
	let sync_config = conf.sync_config(&spec);

	// Create and display a new token for UIs.
	if conf.signer_enabled() && !conf.args.flag_no_token {
		new_token(conf.directories().signer).unwrap_or_else(|e| {
			die!("Error generating token: {:?}", e)
		});
	}

	// Display warning about using unlock with signer
	if conf.signer_enabled() && conf.args.flag_unlock.is_some() {
		warn!("Using Trusted Signer and --unlock is not recommended!");
		warn!("NOTE that Signer will not ask you to confirm transactions from unlocked account.");
	}

	// Secret Store
	let account_service = Arc::new(conf.account_service());

	// Miner
	let miner = Miner::new(conf.miner_options(), conf.spec(), Some(account_service.clone()));
	miner.set_author(conf.author().unwrap_or_default());
	miner.set_gas_floor_target(conf.gas_floor_target());
	miner.set_gas_ceil_target(conf.gas_ceil_target());
	miner.set_extra_data(conf.extra_data());
	miner.set_minimal_gas_price(conf.gas_price());
	miner.set_transactions_limit(conf.args.flag_tx_queue_size);

	// Build client
	let mut service = ClientService::start(
		client_config, spec, net_settings, Path::new(&conf.path()), miner.clone(), !conf.args.flag_no_network
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
	let signer_server = signer::start(signer::Configuration {
		enabled: conf.signer_enabled(),
		port: conf.args.flag_signer_port,
		signer_path: conf.directories().signer,
	}, signer::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
	});

	// Register IO handler
	let io_handler  = Arc::new(ClientIoHandler {
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

	// Setup logging
	let _logger = setup_log::setup_log(&conf.args.flag_logging, conf.args.flag_no_color);
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
		boot_nodes: Vec::new(),
		use_secret: None,
		ideal_peers: 0,
		reserved_nodes: Vec::new(),
		non_reserved_mode: ::util::network::NonReservedPeerMode::Accept,
	};
	let client_config = conf.client_config(&spec);

	// Build client
	let service = ClientService::start(
		client_config, spec, net_settings, Path::new(&conf.path()), Arc::new(Miner::with_spec(conf.spec())), false
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

	// Setup logging
	let _logger = setup_log::setup_log(&conf.args.flag_logging, conf.args.flag_no_color);
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
		boot_nodes: Vec::new(),
		use_secret: None,
		ideal_peers: 0,
		reserved_nodes: Vec::new(),
		non_reserved_mode: ::util::network::NonReservedPeerMode::Accept,
	};
	let client_config = conf.client_config(&spec);

	// Build client
	let service = ClientService::start(
		client_config, spec, net_settings, Path::new(&conf.path()), Arc::new(Miner::with_spec(conf.spec())), false
	).unwrap_or_else(|e| die_with_error("Client", e));

	panic_handler.forward_from(&service);
	let client = service.client();

	let mut instream: Box<Read> = if let Some(ref f) = conf.args.arg_file {
		let f = File::open(f).unwrap_or_else(|_| die!("Cannot open the file given: {}", f));
		Box::new(f)
	} else {
		Box::new(::std::io::stdin())
	};

	const READAHEAD_BYTES: usize = 8;

	let mut first_bytes: Bytes = vec![0; READAHEAD_BYTES];
	let mut first_read = 0;

	let format = match conf.args.flag_format {
		Some(ref x) => match x.deref() {
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

	let informant = Informant::new(conf.have_color());

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
				let mut bytes: Bytes = if first_read > 0 {first_bytes.clone()} else {vec![0; READAHEAD_BYTES]};
				let n = if first_read > 0 {first_read} else {instream.read(&mut(bytes[..])).unwrap_or_else(|_| die!("Error reading from the file/stream."))};
				if n == 0 { break; }
				first_read = 0;
				let s = PayloadInfo::from(&(bytes[..])).unwrap_or_else(|e| die!("Invalid RLP in the file/stream: {:?}", e)).total();
				bytes.resize(s, 0);
				instream.read_exact(&mut(bytes[READAHEAD_BYTES..])).unwrap_or_else(|_| die!("Error reading from the file/stream."));
				do_import(bytes);
			}
		}
		DataFormat::Hex => {
			for line in BufReader::new(instream).lines() {
				let s = line.unwrap_or_else(|_| die!("Error reading from the file/stream."));
				let s = if first_read > 0 {from_utf8(&first_bytes).unwrap().to_owned() + &(s[..])} else {s};
				first_read = 0;
				let bytes = FromHex::from_hex(&(s[..])).unwrap_or_else(|_| die!("Invalid hex in file/stream."));
				do_import(bytes);
			}
		}
	}
	client.flush_queue();
}

fn execute_signer(conf: Configuration) {
	if !conf.args.cmd_new_token {
		die!("Unknown command.");
	}

	let path = conf.directories().signer;
	new_token(path).unwrap_or_else(|e| {
		die!("Error generating token: {:?}", e)
	});
}

fn execute_account_cli(conf: Configuration) {
	use ethcore::ethstore::{EthStore, import_accounts};
	use ethcore::ethstore::dir::DiskDirectory;
	use ethcore::account_provider::AccountProvider;
	use rpassword::read_password;

	let dir = Box::new(DiskDirectory::create(conf.keys_path()).unwrap());
	let iterations = conf.keys_iterations();
	let secret_store = AccountProvider::new(Box::new(EthStore::open_with_iterations(dir, iterations).unwrap()));

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
		for addr in &secret_store.accounts() {
			println!("{:?}", addr);
		}
		return;
	}

	if conf.args.cmd_import {
		let to = DiskDirectory::create(conf.keys_path()).unwrap();
		let mut imported = 0;
		for path in &conf.args.arg_path {
			let from = DiskDirectory::at(path);
			imported += import_accounts(&from, &to).unwrap_or_else(|e| die!("Could not import accounts {}", e)).len();
		}
		println!("Imported {} keys", imported);
	}
}

fn execute_wallet_cli(conf: Configuration) {
	use ethcore::ethstore::{PresaleWallet, EthStore};
	use ethcore::ethstore::dir::DiskDirectory;
	use ethcore::account_provider::AccountProvider;

	let wallet_path = conf.args.arg_path.first().unwrap();
	let filename = conf.args.flag_password.first().unwrap();
	let mut file = File::open(filename).unwrap_or_else(|_| die!("{} Unable to read password file.", filename));
	let mut file_content = String::new();
	file.read_to_string(&mut file_content).unwrap_or_else(|_| die!("{} Unable to read password file.", filename));

	let dir = Box::new(DiskDirectory::create(conf.keys_path()).unwrap());
	let iterations = conf.keys_iterations();
	let store = AccountProvider::new(Box::new(EthStore::open_with_iterations(dir, iterations).unwrap()));

	// remove eof
	let pass = &file_content[..file_content.len() - 1];
	let wallet = PresaleWallet::open(wallet_path).unwrap_or_else(|_| die!("Unable to open presale wallet."));
	let kp = wallet.decrypt(pass).unwrap_or_else(|_| die!("Invalid password"));
	let address = store.insert_account(kp.secret().clone(), pass).unwrap();

	println!("Imported account: {}", address);
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
	let _ = exit.wait(mutex.lock().unwrap()).unwrap();
	info!("Finishing work, please wait...");
}

/// Parity needs at least 1 test to generate coverage reports correctly.
#[test]
fn if_works() {
}
