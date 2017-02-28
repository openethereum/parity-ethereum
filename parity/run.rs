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

use std::sync::Arc;
use std::net::{TcpListener};
use ctrlc::CtrlC;
use fdlimit::raise_fd_limit;
use ethcore_rpc::{NetworkSettings, informant, is_major_importing};
use ethsync::NetworkConfiguration;
use util::{Colour, version, RotatingLogger, Mutex, Condvar};
use io::{MayPanic, ForwardPanic, PanicHandler};
use ethcore_logger::{Config as LogConfig};
use ethcore::miner::{StratumOptions, Stratum};
use ethcore::client::{Mode, DatabaseCompactionProfile, VMType, BlockChainClient};
use ethcore::service::ClientService;
use ethcore::account_provider::{AccountProvider, AccountProviderSettings};
use ethcore::miner::{Miner, MinerService, ExternalMiner, MinerOptions};
use ethcore::snapshot;
use ethcore::verification::queue::VerifierSettings;
use ethsync::SyncConfig;
use informant::Informant;
use updater::{UpdatePolicy, Updater};
use parity_reactor::EventLoop;
use hash_fetch::fetch::{Fetch, Client as FetchClient};

use rpc::{HttpConfiguration, IpcConfiguration};
use params::{
	SpecType, Pruning, AccountsConfig, GasPricerConfig, MinerExtras, Switch,
	tracing_switch_to_bool, fatdb_switch_to_bool, mode_switch_to_bool
};
use helpers::{to_client_config, execute_upgrades, passwords_from_files};
use upgrade::upgrade_key_location;
use dir::Directories;
use cache::CacheConfig;
use user_defaults::UserDefaults;
use dapps;
use ipfs;
use signer;
use secretstore;
use modules;
use rpc_apis;
use rpc;
use url;

// how often to take periodic snapshots.
const SNAPSHOT_PERIOD: u64 = 10000;

// how many blocks to wait before starting a periodic snapshot.
const SNAPSHOT_HISTORY: u64 = 100;

// Pops along with error messages when a password is missing or invalid.
const VERIFY_PASSWORD_HINT: &'static str = "Make sure valid password is present in files passed using `--password` or in the configuration file.";

#[derive(Debug, PartialEq)]
pub struct RunCmd {
	pub cache_config: CacheConfig,
	pub dirs: Directories,
	pub spec: SpecType,
	pub pruning: Pruning,
	pub pruning_history: u64,
	pub pruning_memory: usize,
	/// Some if execution should be daemonized. Contains pid_file path.
	pub daemon: Option<String>,
	pub logger_config: LogConfig,
	pub miner_options: MinerOptions,
	pub http_conf: HttpConfiguration,
	pub ipc_conf: IpcConfiguration,
	pub net_conf: NetworkConfiguration,
	pub network_id: Option<u64>,
	pub warp_sync: bool,
	pub acc_conf: AccountsConfig,
	pub gas_pricer: GasPricerConfig,
	pub miner_extras: MinerExtras,
	pub update_policy: UpdatePolicy,
	pub mode: Option<Mode>,
	pub tracing: Switch,
	pub fat_db: Switch,
	pub compaction: DatabaseCompactionProfile,
	pub wal: bool,
	pub vm_type: VMType,
	pub geth_compatibility: bool,
	pub ui_address: Option<(String, u16)>,
	pub net_settings: NetworkSettings,
	pub dapps_conf: dapps::Configuration,
	pub ipfs_conf: ipfs::Configuration,
	pub signer_conf: signer::Configuration,
	pub secretstore_conf: secretstore::Configuration,
	pub dapp: Option<String>,
	pub ui: bool,
	pub name: String,
	pub custom_bootnodes: bool,
	pub stratum: Option<StratumOptions>,
	pub no_periodic_snapshot: bool,
	pub check_seal: bool,
	pub download_old_blocks: bool,
	pub verifier_settings: VerifierSettings,
}

pub fn open_ui(dapps_conf: &dapps::Configuration, signer_conf: &signer::Configuration) -> Result<(), String> {
	if !dapps_conf.enabled {
		return Err("Cannot use UI command with Dapps turned off.".into())
	}

	if !signer_conf.enabled {
		return Err("Cannot use UI command with UI turned off.".into())
	}

	let token = signer::generate_token_and_url(signer_conf)?;
	// Open a browser
	url::open(&token.url);
	// Print a message
	println!("{}", token.message);
	Ok(())
}

pub fn open_dapp(dapps_conf: &dapps::Configuration, dapp: &str) -> Result<(), String> {
	if !dapps_conf.enabled {
		return Err("Cannot use DAPP command with Dapps turned off.".into())
	}

	let url = format!("http://{}:{}/{}/", dapps_conf.interface, dapps_conf.port, dapp);
	url::open(&url);
	Ok(())
}

// node info fetcher for the local store.
struct FullNodeInfo {
	miner: Arc<Miner>, // TODO: only TXQ needed, just use that after decoupling.
}

impl ::local_store::NodeInfo for FullNodeInfo {
	fn pending_transactions(&self) -> Vec<::ethcore::transaction::PendingTransaction> {
		let local_txs = self.miner.local_transactions();
		self.miner.pending_transactions()
			.into_iter()
			.chain(self.miner.future_transactions())
			.filter(|tx| local_txs.contains_key(&tx.hash()))
			.collect()
	}
}

pub fn execute(cmd: RunCmd, can_restart: bool, logger: Arc<RotatingLogger>) -> Result<bool, String> {
	if cmd.ui && cmd.dapps_conf.enabled {
		// Check if Parity is already running
		let addr = format!("{}:{}", cmd.dapps_conf.interface, cmd.dapps_conf.port);
		if !TcpListener::bind(&addr as &str).is_ok() {
			return open_ui(&cmd.dapps_conf, &cmd.signer_conf).map(|_| false);
		}
	}

	// set up panic handler
	let panic_handler = PanicHandler::new_in_arc();

	// increase max number of open files
	raise_fd_limit();

	// load spec
	let spec = cmd.spec.spec()?;

	// load genesis hash
	let genesis_hash = spec.genesis_header().hash();

	// database paths
	let db_dirs = cmd.dirs.database(genesis_hash, cmd.spec.legacy_fork_name(), spec.data_dir.clone());

	// user defaults path
	let user_defaults_path = db_dirs.user_defaults_path();

	// load user defaults
	let mut user_defaults = UserDefaults::load(&user_defaults_path)?;

	// select pruning algorithm
	let algorithm = cmd.pruning.to_algorithm(&user_defaults);

	// check if tracing is on
	let tracing = tracing_switch_to_bool(cmd.tracing, &user_defaults)?;

	// check if fatdb is on
	let fat_db = fatdb_switch_to_bool(cmd.fat_db, &user_defaults, algorithm)?;

	// get the mode
	let mode = mode_switch_to_bool(cmd.mode, &user_defaults)?;
	trace!(target: "mode", "mode is {:?}", mode);
	let network_enabled = match mode { Mode::Dark(_) | Mode::Off => false, _ => true, };

	// get the update policy
	let update_policy = cmd.update_policy;

	// prepare client and snapshot paths.
	let client_path = db_dirs.client_path(algorithm);
	let snapshot_path = db_dirs.snapshot_path();

	// execute upgrades
	execute_upgrades(&cmd.dirs.base, &db_dirs, algorithm, cmd.compaction.compaction_profile(db_dirs.db_root_path().as_path()))?;

	// create dirs used by parity
	cmd.dirs.create_dirs(cmd.dapps_conf.enabled, cmd.signer_conf.enabled, cmd.secretstore_conf.enabled)?;

	// run in daemon mode
	if let Some(pid_file) = cmd.daemon {
		daemonize(pid_file)?;
	}

	// display info about used pruning algorithm
	info!("Starting {}", Colour::White.bold().paint(version()));
	info!("State DB configuration: {}{}{}",
		Colour::White.bold().paint(algorithm.as_str()),
		match fat_db {
			true => Colour::White.bold().paint(" +Fat").to_string(),
			false => "".to_owned(),
		},
		match tracing {
			true => Colour::White.bold().paint(" +Trace").to_string(),
			false => "".to_owned(),
		}
	);
	info!("Operating mode: {}", Colour::White.bold().paint(format!("{}", mode)));

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
	if spec.subprotocol_name().len() != 3 {
		warn!("Your chain specification's subprotocol length is not 3. Ignoring.");
	} else {
		sync_config.subprotocol_name.clone_from_slice(spec.subprotocol_name().as_bytes());
	}
	sync_config.fork_block = spec.fork_block();
	sync_config.warp_sync = cmd.warp_sync;
	sync_config.download_old_blocks = cmd.download_old_blocks;

	let passwords = passwords_from_files(&cmd.acc_conf.password_files)?;

	// prepare account provider
	let account_provider = Arc::new(prepare_account_provider(&cmd.spec, &cmd.dirs, &spec.data_dir, cmd.acc_conf, &passwords)?);

	// create miner
	let initial_min_gas_price = cmd.gas_pricer.initial_min();
	let miner = Miner::new(cmd.miner_options, cmd.gas_pricer.into(), &spec, Some(account_provider.clone()));
	miner.set_author(cmd.miner_extras.author);
	miner.set_gas_floor_target(cmd.miner_extras.gas_floor_target);
	miner.set_gas_ceil_target(cmd.miner_extras.gas_ceil_target);
	miner.set_extra_data(cmd.miner_extras.extra_data);
	miner.set_transactions_limit(cmd.miner_extras.transactions_limit);
	miner.set_minimal_gas_price(initial_min_gas_price);
	miner.recalibrate_minimal_gas_price();
	let engine_signer = cmd.miner_extras.engine_signer;

	if engine_signer != Default::default() {
		// Check if engine signer exists
		if !account_provider.has_account(engine_signer).unwrap_or(false) {
			return Err(format!("Consensus signer account not found for the current chain. {}", build_create_account_hint(&cmd.spec, &cmd.dirs.keys)));
		}

		// Check if any passwords have been read from the password file(s)
		if passwords.is_empty() {
			return Err(format!("No password found for the consensus signer {}. {}", engine_signer, VERIFY_PASSWORD_HINT));
		}

		// Attempt to sign in the engine signer.
		if !passwords.into_iter().any(|p| miner.set_engine_signer(engine_signer, p).is_ok()) {
			return Err(format!("No valid password for the consensus signer {}. {}", engine_signer, VERIFY_PASSWORD_HINT));
		}
	}

	// create client config
	let mut client_config = to_client_config(
		&cmd.cache_config,
		mode.clone(),
		tracing,
		fat_db,
		cmd.compaction,
		cmd.wal,
		cmd.vm_type,
		cmd.name,
		algorithm,
		cmd.pruning_history,
		cmd.pruning_memory,
		cmd.check_seal,
	);

	client_config.queue.verifier_settings = cmd.verifier_settings;

	// set up bootnodes
	let mut net_conf = cmd.net_conf;
	if !cmd.custom_bootnodes {
		net_conf.boot_nodes = spec.nodes.clone();
	}

	// set network path.
	net_conf.net_config_path = Some(db_dirs.network_path().to_string_lossy().into_owned());

	// create supervisor
	let mut hypervisor = modules::hypervisor(&cmd.dirs.ipc_path());

	// create client service.
	let service = ClientService::start(
		client_config,
		&spec,
		&client_path,
		&snapshot_path,
		&cmd.dirs.ipc_path(),
		miner.clone(),
	).map_err(|e| format!("Client service error: {:?}", e))?;

	// drop the spec to free up genesis state.
	drop(spec);

	// forward panics from service
	panic_handler.forward_from(&service);

	// take handle to client
	let client = service.client();
	let snapshot_service = service.snapshot_service();

	// initialize the local node information store.
	let store = {
		let db = service.db();
		let node_info = FullNodeInfo {
			miner: miner.clone(),
		};

		let store = ::local_store::create(db, ::ethcore::db::COL_NODE_INFO, node_info);

		// re-queue pending transactions.
		match store.pending_transactions() {
			Ok(pending) => {
				for pending_tx in pending {
					if let Err(e) = miner.import_own_transaction(&*client, pending_tx) {
						warn!("Error importing saved transaction: {}", e)
					}
				}
			}
			Err(e) => warn!("Error loading cached pending transactions from disk: {}", e),
		}

		Arc::new(store)
	};

	// register it as an IO service to update periodically.
	service.register_io_handler(store).map_err(|_| "Unable to register local store handler".to_owned())?;

	// create external miner
	let external_miner = Arc::new(ExternalMiner::default());

	// start stratum
	if let Some(ref stratum_config) = cmd.stratum {
		Stratum::register(stratum_config, miner.clone(), Arc::downgrade(&client))
			.map_err(|e| format!("Stratum start error: {:?}", e))?;
	}

	// create sync object
	let (sync_provider, manage_network, chain_notify) = modules::sync(
		&mut hypervisor,
		sync_config,
		net_conf.into(),
		client.clone(),
		snapshot_service.clone(),
		client.clone(),
		&cmd.logger_config,
	).map_err(|e| format!("Sync error: {}", e))?;

	service.add_notify(chain_notify.clone());

	// start network
	if network_enabled {
		chain_notify.start();
	}

	// spin up event loop
	let event_loop = EventLoop::spawn();

	// fetch service
	let fetch = FetchClient::new().map_err(|e| format!("Error starting fetch client: {:?}", e))?;

	// the updater service
	let updater = Updater::new(
		Arc::downgrade(&(service.client() as Arc<BlockChainClient>)),
		Arc::downgrade(&sync_provider),
		update_policy,
		fetch.clone(),
		event_loop.remote(),
	);
	service.add_notify(updater.clone());

	// set up dependencies for rpc servers
	let rpc_stats = Arc::new(informant::RpcStats::default());
	let signer_path = cmd.signer_conf.signer_path.clone();
	let deps_for_rpc_apis = Arc::new(rpc_apis::Dependencies {
		signer_service: Arc::new(rpc_apis::SignerService::new(move || {
			signer::generate_new_token(signer_path.clone()).map_err(|e| format!("{:?}", e))
		}, cmd.ui_address)),
		snapshot: snapshot_service.clone(),
		client: client.clone(),
		sync: sync_provider.clone(),
		net: manage_network.clone(),
		secret_store: account_provider.clone(),
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: Arc::new(cmd.net_settings.clone()),
		net_service: manage_network.clone(),
		updater: updater.clone(),
		geth_compatibility: cmd.geth_compatibility,
		dapps_interface: match cmd.dapps_conf.enabled {
			true => Some(cmd.dapps_conf.interface.clone()),
			false => None,
		},
		dapps_port: match cmd.dapps_conf.enabled {
			true => Some(cmd.dapps_conf.port),
			false => None,
		},
		fetch: fetch.clone(),
	});

	let dependencies = rpc::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
		remote: event_loop.raw_remote(),
		stats: rpc_stats.clone(),
	};

	// start rpc servers
	let http_server = rpc::new_http(cmd.http_conf, &dependencies)?;
	let ipc_server = rpc::new_ipc(cmd.ipc_conf, &dependencies)?;

	// the dapps server
	let dapps_deps = dapps::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
		client: client.clone(),
		sync: sync_provider.clone(),
		remote: event_loop.raw_remote(),
		fetch: fetch.clone(),
		signer: deps_for_rpc_apis.signer_service.clone(),
		stats: rpc_stats.clone(),
	};
	let dapps_server = dapps::new(cmd.dapps_conf.clone(), dapps_deps)?;

	// the signer server
	let signer_deps = signer::Dependencies {
		panic_handler: panic_handler.clone(),
		apis: deps_for_rpc_apis.clone(),
		remote: event_loop.raw_remote(),
		rpc_stats: rpc_stats.clone(),
	};
	let signer_server = signer::start(cmd.signer_conf.clone(), signer_deps)?;

	// secret store key server
	let secretstore_deps = secretstore::Dependencies { };
	let secretstore_key_server = secretstore::start(cmd.secretstore_conf.clone(), secretstore_deps);

	// the ipfs server
	let ipfs_server = ipfs::start_server(cmd.ipfs_conf.clone(), client.clone())?;

	// the informant
	let informant = Arc::new(Informant::new(
		service.client(),
		Some(sync_provider.clone()),
		Some(manage_network.clone()),
		Some(snapshot_service.clone()),
		Some(rpc_stats.clone()),
		cmd.logger_config.color,
	));
	service.add_notify(informant.clone());
	service.register_io_handler(informant.clone()).map_err(|_| "Unable to register informant handler".to_owned())?;

	// save user defaults
	user_defaults.pruning = algorithm;
	user_defaults.tracing = tracing;
	user_defaults.fat_db = fat_db;
	user_defaults.mode = mode;
	user_defaults.save(&user_defaults_path)?;

	// tell client how to save the default mode if it gets changed.
	client.on_mode_change(move |mode: &Mode| {
		user_defaults.mode = mode.clone();
		let _ = user_defaults.save(&user_defaults_path);	// discard failures - there's nothing we can do
	});

	// the watcher must be kept alive.
	let _watcher = match cmd.no_periodic_snapshot {
		true => None,
		false => {
			let sync = sync_provider.clone();
			let watcher = Arc::new(snapshot::Watcher::new(
				service.client(),
				move || is_major_importing(Some(sync.status().state), client.queue_info()),
				service.io().channel(),
				SNAPSHOT_PERIOD,
				SNAPSHOT_HISTORY,
			));

			service.add_notify(watcher.clone());
			Some(watcher)
		},
	};

	// start ui
	if cmd.ui {
		open_ui(&cmd.dapps_conf, &cmd.signer_conf)?;
	}

	if let Some(dapp) = cmd.dapp {
		open_dapp(&cmd.dapps_conf, &dapp)?;
	}

	// Handle exit
	let restart = wait_for_exit(panic_handler, Some(updater), can_restart);

	// drop this stuff as soon as exit detected.
	drop((http_server, ipc_server, dapps_server, signer_server, secretstore_key_server, ipfs_server, event_loop));

	info!("Finishing work, please wait...");

	// to make sure timer does not spawn requests while shutdown is in progress
	informant.shutdown();
	// just Arc is dropping here, to allow other reference release in its default time
	drop(informant);

	// hypervisor should be shutdown first while everything still works and can be
	// terminated gracefully
	drop(hypervisor);

	Ok(restart)
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

fn prepare_account_provider(spec: &SpecType, dirs: &Directories, data_dir: &str, cfg: AccountsConfig, passwords: &[String]) -> Result<AccountProvider, String> {
	use ethcore::ethstore::EthStore;
	use ethcore::ethstore::dir::RootDiskDirectory;

	let path = dirs.keys_path(data_dir);
	upgrade_key_location(&dirs.legacy_keys_path(cfg.testnet), &path);
	let dir = Box::new(RootDiskDirectory::create(&path).map_err(|e| format!("Could not open keys directory: {}", e))?);
	let account_settings = AccountProviderSettings {
		enable_hardware_wallets: cfg.enable_hardware_wallets,
		hardware_wallet_classic_key: spec == &SpecType::Classic,
	};
	let account_provider = AccountProvider::new(
		Box::new(EthStore::open_with_iterations(dir, cfg.iterations).map_err(|e| format!("Could not open keys directory: {}", e))?),
		account_settings);

	for a in cfg.unlocked_accounts {
		// Check if the account exists
		if !account_provider.has_account(a).unwrap_or(false) {
			return Err(format!("Account {} not found for the current chain. {}", a, build_create_account_hint(spec, &dirs.keys)));
		}

		// Check if any passwords have been read from the password file(s)
		if passwords.is_empty() {
			return Err(format!("No password found to unlock account {}. {}", a, VERIFY_PASSWORD_HINT));
		}

		if !passwords.iter().any(|p| account_provider.unlock_account_permanently(a, (*p).clone()).is_ok()) {
			return Err(format!("No valid password to unlock account {}. {}", a, VERIFY_PASSWORD_HINT));
		}
	}

	Ok(account_provider)
}

// Construct an error `String` with an adaptive hint on how to create an account.
fn build_create_account_hint(spec: &SpecType, keys: &str) -> String {
	format!("You can create an account via RPC, UI or `parity account new --chain {} --keys-path {}`.", spec, keys)
}

fn wait_for_exit(
	panic_handler: Arc<PanicHandler>,
	updater: Option<Arc<Updater>>,
	can_restart: bool
) -> bool {
	let exit = Arc::new((Mutex::new(false), Condvar::new()));

	// Handle possible exits
	let e = exit.clone();
	CtrlC::set_handler(move || { e.1.notify_all(); });

	// Handle panics
	let e = exit.clone();
	panic_handler.on_panic(move |_reason| { e.1.notify_all(); });

	if let Some(updater) = updater {
		// Handle updater wanting to restart us
		if can_restart {
			let e = exit.clone();
			updater.set_exit_handler(move || { *e.0.lock() = true; e.1.notify_all(); });
		} else {
			updater.set_exit_handler(|| info!("Update installed; ready for restart."));
		}
	}

	// Wait for signal
	let mut l = exit.0.lock();
	let _ = exit.1.wait(&mut l);
	*l
}
