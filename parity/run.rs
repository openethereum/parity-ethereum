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

use std::sync::{Arc, Mutex, Condvar};
use std::path::Path;
use std::io::ErrorKind;
use ctrlc::CtrlC;
use fdlimit::raise_fd_limit;
use ethcore_logger::{Config as LogConfig, setup_log};
use ethcore_rpc::NetworkSettings;
use ethsync::NetworkConfiguration;
use util::{Colour, version, U256};
use io::{MayPanic, ForwardPanic, PanicHandler};
use ethcore::client::{Mode, Switch, DatabaseCompactionProfile, VMType, ChainNotify};
use ethcore::service::ClientService;
use ethcore::account_provider::AccountProvider;
use ethcore::miner::{Miner, MinerService, ExternalMiner, MinerOptions};
use ethsync::SyncConfig;
use informant::Informant;

use rpc::{HttpServer, IpcServer, HttpConfiguration, IpcConfiguration};
use signer::SignerServer;
use dapps::WebappServer;
use io_handler::ClientIoHandler;
use params::{SpecType, Pruning, AccountsConfig, GasPricerConfig, MinerExtras};
use helpers::{to_client_config, execute_upgrades, passwords_from_files};
use dir::Directories;
use cache::CacheConfig;
use dapps;
use signer;
use modules;
use rpc_apis;
use rpc;
use url;

#[derive(Debug, PartialEq)]
pub struct RunCmd {
	pub cache_config: CacheConfig,
	pub dirs: Directories,
	pub spec: SpecType,
	pub pruning: Pruning,
	/// Some if execution should be daemonized. Contains pid_file path.
	pub daemon: Option<String>,
	pub logger_config: LogConfig,
	pub miner_options: MinerOptions,
	pub http_conf: HttpConfiguration,
	pub ipc_conf: IpcConfiguration,
	pub net_conf: NetworkConfiguration,
	pub network_id: Option<U256>,
	pub acc_conf: AccountsConfig,
	pub gas_pricer: GasPricerConfig,
	pub miner_extras: MinerExtras,
	pub mode: Mode,
	pub tracing: Switch,
	pub compaction: DatabaseCompactionProfile,
	pub wal: bool,
	pub vm_type: VMType,
	pub enable_network: bool,
	pub geth_compatibility: bool,
	pub signer_port: Option<u16>,
	pub net_settings: NetworkSettings,
	pub dapps_conf: dapps::Configuration,
	pub signer_conf: signer::Configuration,
	pub ui: bool,
	pub name: String,
	pub custom_bootnodes: bool,
}

pub fn execute(cmd: RunCmd) -> Result<(), String> {
	// create supervisor
	let mut hypervisor = modules::hypervisor();

	// increase max number of open files
	raise_fd_limit();

	// set up logger
	let logger = try!(setup_log(&cmd.logger_config));

	// set up panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// create dirs used by parity
	try!(cmd.dirs.create_dirs());

	// load spec
	let spec = try!(cmd.spec.spec());
	let fork_name = spec.fork_name.clone();

	// load genesis hash
	let genesis_hash = spec.genesis_header().hash();

	// select pruning algorithm
	let algorithm = cmd.pruning.to_algorithm(&cmd.dirs, genesis_hash, fork_name.as_ref());

	// prepare client_path
	let client_path = cmd.dirs.client_path(genesis_hash, fork_name.as_ref(), algorithm);

	// execute upgrades
	try!(execute_upgrades(&cmd.dirs, genesis_hash, fork_name.as_ref(), algorithm, cmd.compaction.compaction_profile()));

	// run in daemon mode
	if let Some(pid_file) = cmd.daemon {
		try!(daemonize(pid_file));
	}

	// display info about used pruning algorithm
	info!("Starting {}", Colour::White.bold().paint(version()));
	info!("Using state DB journalling strategy {}", Colour::White.bold().paint(algorithm.as_str()));

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
	sync_config.fork_block = spec.fork_block();

	// prepare account provider
	let account_provider = Arc::new(try!(prepare_account_provider(&cmd.dirs, cmd.acc_conf)));

	// create miner
	let miner = Miner::new(cmd.miner_options, cmd.gas_pricer.into(), &spec, Some(account_provider.clone()));
	miner.set_author(cmd.miner_extras.author);
	miner.set_gas_floor_target(cmd.miner_extras.gas_floor_target);
	miner.set_gas_ceil_target(cmd.miner_extras.gas_ceil_target);
	miner.set_extra_data(cmd.miner_extras.extra_data);
	miner.set_transactions_limit(cmd.miner_extras.transactions_limit);

	// create client config
	let client_config = to_client_config(
		&cmd.cache_config,
		&cmd.dirs,
		genesis_hash,
		cmd.mode,
		cmd.tracing,
		cmd.pruning,
		cmd.compaction,
		cmd.wal,
		cmd.vm_type,
		cmd.name,
		fork_name.as_ref(),
	);

	// set up bootnodes
	let mut net_conf = cmd.net_conf;
	if !cmd.custom_bootnodes {
		net_conf.boot_nodes = spec.nodes.clone();
	}

	// create client service.
	let service = try!(ClientService::start(
		client_config,
		&spec,
		Path::new(&client_path),
		miner.clone(),
	).map_err(|e| format!("Client service error: {:?}", e)));

	// forward panics from service
	panic_handler.forward_from(&service);

	// take handle to client
	let client = service.client();

	// create external miner
	let external_miner = Arc::new(ExternalMiner::default());

	// create sync object
	let (sync_provider, manage_network, chain_notify) = try!(modules::sync(
		&mut hypervisor, sync_config, net_conf.into(), client.clone(), &cmd.logger_config,
	).map_err(|e| format!("Sync error: {}", e)));

	service.add_notify(chain_notify.clone());

	// start network
	if cmd.enable_network {
		chain_notify.start();
	}

	// set up dependencies for rpc servers
	let deps_for_rpc_apis = Arc::new(rpc_apis::Dependencies {
		signer_port: cmd.signer_port,
		signer_queue: Arc::new(rpc_apis::ConfirmationsQueue::default()),
		client: client.clone(),
		sync: sync_provider.clone(),
		net: manage_network.clone(),
		secret_store: account_provider.clone(),
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: Arc::new(cmd.net_settings.clone()),
		net_service: manage_network.clone(),
		geth_compatibility: cmd.geth_compatibility,
	});

	let dependencies = rpc::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
	};

	// start rpc servers
	let http_server = try!(rpc::new_http(cmd.http_conf, &dependencies));
	let ipc_server = try!(rpc::new_ipc(cmd.ipc_conf, &dependencies));

	let dapps_deps = dapps::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
	};

	// start dapps server
	let dapps_server = try!(dapps::new(cmd.dapps_conf.clone(), dapps_deps));

	let signer_deps = signer::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
	};

	// start signer server
	let signer_server = try!(signer::start(cmd.signer_conf, signer_deps));

	let informant = Arc::new(Informant::new(service.client(), Some(sync_provider.clone()), Some(manage_network.clone()), cmd.logger_config.color));
	let info_notify: Arc<ChainNotify> = informant.clone();
	service.add_notify(info_notify);
	let io_handler = Arc::new(ClientIoHandler {
		client: service.client(),
		info: informant,
		sync: sync_provider.clone(),
		net: manage_network.clone(),
		accounts: account_provider.clone(),
	});
	service.register_io_handler(io_handler).expect("Error registering IO handler");

	// start ui
	if cmd.ui {
		if !cmd.dapps_conf.enabled {
			return Err("Cannot use UI command with Dapps turned off.".into())
		}
		url::open(&format!("http://{}:{}/", cmd.dapps_conf.interface, cmd.dapps_conf.port));
	}

	// Handle exit
	wait_for_exit(panic_handler, http_server, ipc_server, dapps_server, signer_server);

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
fn daemonize(_pid_file: String) -> Result<(), String> {
	Err("daemon is no supported on windows".into())
}

fn prepare_account_provider(dirs: &Directories, cfg: AccountsConfig) -> Result<AccountProvider, String> {
	use ethcore::ethstore::{import_accounts, EthStore};
	use ethcore::ethstore::dir::{GethDirectory, DirectoryType, DiskDirectory};
	use ethcore::ethstore::Error;

	let passwords = try!(passwords_from_files(cfg.password_files));

	if cfg.import_keys {
		let t = if cfg.testnet {
			DirectoryType::Testnet
		} else {
			DirectoryType::Main
		};

		let from = GethDirectory::open(t);
		let to = try!(DiskDirectory::create(dirs.keys.clone()).map_err(|e| format!("Could not open keys directory: {}", e)));
		match import_accounts(&from, &to) {
			Ok(_) => {}
			Err(Error::Io(ref io_err)) if io_err.kind() == ErrorKind::NotFound => {}
			Err(err) => warn!("Import geth accounts failed. {}", err)
		}
	}

	let dir = Box::new(try!(DiskDirectory::create(dirs.keys.clone()).map_err(|e| format!("Could not open keys directory: {}", e))));
	let account_service = AccountProvider::new(Box::new(
		try!(EthStore::open_with_iterations(dir, cfg.iterations).map_err(|e| format!("Could not open keys directory: {}", e)))
	));

	for a in cfg.unlocked_accounts {
		if passwords.iter().find(|p| account_service.unlock_account_permanently(a, (*p).clone()).is_ok()).is_none() {
			return Err(format!("No password found to unlock account {}. Make sure valid password is present in files passed using `--password`.", a));
		}
	}

	Ok(account_service)
}

fn wait_for_exit(
	panic_handler: Arc<PanicHandler>,
	_http_server: Option<HttpServer>,
	_ipc_server: Option<IpcServer>,
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
