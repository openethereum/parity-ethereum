// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::any::Any;
use std::fmt;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use std::thread;

use ansi_term::Colour;
use ethcore::account_provider::{AccountProvider, AccountProviderSettings};
use ethcore::client::{Client, Mode, DatabaseCompactionProfile, VMType, BlockChainClient, BlockInfo};
use ethcore::ethstore::ethkey;
use ethcore::miner::{stratum, Miner, MinerService, MinerOptions};
use ethcore::snapshot;
use ethcore::spec::{SpecParams, OptimizeFor};
use ethcore::verification::queue::VerifierSettings;
use ethcore_logger::{Config as LogConfig, RotatingLogger};
use ethcore_service::ClientService;
use sync::{self, SyncConfig};
use miner::work_notify::WorkPoster;
use futures_cpupool::CpuPool;
use hash_fetch::{self, fetch};
use informant::{Informant, LightNodeInformantData, FullNodeInformantData};
use journaldb::Algorithm;
use light::Cache as LightDataCache;
use miner::external::ExternalMiner;
use node_filter::NodeFilter;
use node_health;
use parity_reactor::EventLoop;
use parity_rpc::{Origin, Metadata, NetworkSettings, informant, is_major_importing};
use updater::{UpdatePolicy, Updater};
use parity_version::version;
use ethcore_private_tx::{ProviderConfig, EncryptorConfig, SecretStoreEncryptor};
use params::{
	SpecType, Pruning, AccountsConfig, GasPricerConfig, MinerExtras, Switch,
	tracing_switch_to_bool, fatdb_switch_to_bool, mode_switch_to_bool
};
use helpers::{to_client_config, execute_upgrades, passwords_from_files};
use upgrade::upgrade_key_location;
use dir::{Directories, DatabaseDirectories};
use cache::CacheConfig;
use user_defaults::UserDefaults;
use dapps;
use ipfs;
use jsonrpc_core;
use modules;
use rpc;
use rpc_apis;
use secretstore;
use signer;
use db;

// how often to take periodic snapshots.
const SNAPSHOT_PERIOD: u64 = 5000;

// how many blocks to wait before starting a periodic snapshot.
const SNAPSHOT_HISTORY: u64 = 100;

// Number of minutes before a given gas price corpus should expire.
// Light client only.
const GAS_CORPUS_EXPIRATION_MINUTES: u64 = 60 * 6;

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
	pub gas_price_percentile: usize,
	pub poll_lifetime: u32,
	pub ntp_servers: Vec<String>,
	pub ws_conf: rpc::WsConfiguration,
	pub http_conf: rpc::HttpConfiguration,
	pub ipc_conf: rpc::IpcConfiguration,
	pub net_conf: sync::NetworkConfiguration,
	pub network_id: Option<u64>,
	pub warp_sync: bool,
	pub warp_barrier: Option<u64>,
	pub acc_conf: AccountsConfig,
	pub gas_pricer_conf: GasPricerConfig,
	pub miner_extras: MinerExtras,
	pub update_policy: UpdatePolicy,
	pub mode: Option<Mode>,
	pub tracing: Switch,
	pub fat_db: Switch,
	pub compaction: DatabaseCompactionProfile,
	pub wal: bool,
	pub vm_type: VMType,
	pub geth_compatibility: bool,
	pub net_settings: NetworkSettings,
	pub dapps_conf: dapps::Configuration,
	pub ipfs_conf: ipfs::Configuration,
	pub secretstore_conf: secretstore::Configuration,
	pub private_provider_conf: ProviderConfig,
	pub private_encryptor_conf: EncryptorConfig,
	pub private_tx_enabled: bool,
	pub dapp: Option<String>,
	pub name: String,
	pub custom_bootnodes: bool,
	pub stratum: Option<stratum::Options>,
	pub no_periodic_snapshot: bool,
	pub check_seal: bool,
	pub download_old_blocks: bool,
	pub verifier_settings: VerifierSettings,
	pub serve_light: bool,
	pub light: bool,
	pub no_persistent_txqueue: bool,
	pub whisper: ::whisper::Config,
	pub no_hardcoded_sync: bool,
}

// node info fetcher for the local store.
struct FullNodeInfo {
	miner: Option<Arc<Miner>>, // TODO: only TXQ needed, just use that after decoupling.
}

impl ::local_store::NodeInfo for FullNodeInfo {
	fn pending_transactions(&self) -> Vec<::transaction::PendingTransaction> {
		let miner = match self.miner.as_ref() {
			Some(m) => m,
			None => return Vec::new(),
		};

		miner.local_transactions()
			.values()
			.filter_map(|status| match *status {
				::miner::pool::local_transactions::Status::Pending(ref tx) => Some(tx.pending().clone()),
				_ => None,
			})
			.collect()
	}
}

type LightClient = ::light::client::Client<::light_helpers::EpochFetch>;

// helper for light execution.
fn execute_light_impl(cmd: RunCmd, logger: Arc<RotatingLogger>) -> Result<RunningClient, String> {
	use light::client as light_client;
	use sync::{LightSyncParams, LightSync, ManageNetwork};
	use parking_lot::{Mutex, RwLock};

	// load spec
	let spec = cmd.spec.spec(SpecParams::new(cmd.dirs.cache.as_ref(), OptimizeFor::Memory))?;

	// load genesis hash
	let genesis_hash = spec.genesis_header().hash();

	// database paths
	let db_dirs = cmd.dirs.database(genesis_hash, cmd.spec.legacy_fork_name(), spec.data_dir.clone());

	// user defaults path
	let user_defaults_path = db_dirs.user_defaults_path();

	// load user defaults
	let user_defaults = UserDefaults::load(&user_defaults_path)?;

	// select pruning algorithm
	let algorithm = cmd.pruning.to_algorithm(&user_defaults);

	// execute upgrades
	execute_upgrades(&cmd.dirs.base, &db_dirs, algorithm, &cmd.compaction)?;

	// create dirs used by parity
	cmd.dirs.create_dirs(cmd.dapps_conf.enabled, cmd.acc_conf.unlocked_accounts.len() == 0, cmd.secretstore_conf.enabled)?;

	//print out running parity environment
	print_running_environment(&spec.name, &cmd.dirs, &db_dirs, &cmd.dapps_conf);

	info!("Running in experimental {} mode.", Colour::Blue.bold().paint("Light Client"));

	// TODO: configurable cache size.
	let cache = LightDataCache::new(Default::default(), Duration::from_secs(60 * GAS_CORPUS_EXPIRATION_MINUTES));
	let cache = Arc::new(Mutex::new(cache));

	// start client and create transaction queue.
	let mut config = light_client::Config {
		queue: Default::default(),
		chain_column: ::ethcore::db::COL_LIGHT_CHAIN,
		verify_full: true,
		check_seal: cmd.check_seal,
		no_hardcoded_sync: cmd.no_hardcoded_sync,
	};

	config.queue.max_mem_use = cmd.cache_config.queue() as usize * 1024 * 1024;
	config.queue.verifier_settings = cmd.verifier_settings;

	// start on_demand service.
	let on_demand = Arc::new(::light::on_demand::OnDemand::new(cache.clone()));

	let sync_handle = Arc::new(RwLock::new(Weak::new()));
	let fetch = ::light_helpers::EpochFetch {
		on_demand: on_demand.clone(),
		sync: sync_handle.clone(),
	};

	// initialize database.
	let db = db::open_db(&db_dirs.client_path(algorithm).to_str().expect("DB path could not be converted to string."),
						 &cmd.cache_config,
						 &cmd.compaction,
						 cmd.wal).map_err(|e| format!("Failed to open database {:?}", e))?;

	let service = light_client::Service::start(config, &spec, fetch, db, cache.clone())
		.map_err(|e| format!("Error starting light client: {}", e))?;
	let client = service.client().clone();
	let txq = Arc::new(RwLock::new(::light::transaction_queue::TransactionQueue::default()));
	let provider = ::light::provider::LightProvider::new(client.clone(), txq.clone());

	// start network.
	// set up bootnodes
	let mut net_conf = cmd.net_conf;
	if !cmd.custom_bootnodes {
		net_conf.boot_nodes = spec.nodes.clone();
	}

	let mut attached_protos = Vec::new();
	let whisper_factory = if cmd.whisper.enabled {
		let whisper_factory = ::whisper::setup(cmd.whisper.target_message_pool_size, &mut attached_protos)
			.map_err(|e| format!("Failed to initialize whisper: {}", e))?;
		whisper_factory
	} else {
		None
	};

	// set network path.
	net_conf.net_config_path = Some(db_dirs.network_path().to_string_lossy().into_owned());
	let sync_params = LightSyncParams {
		network_config: net_conf.into_basic().map_err(|e| format!("Failed to produce network config: {}", e))?,
		client: Arc::new(provider),
		network_id: cmd.network_id.unwrap_or(spec.network_id()),
		subprotocol_name: sync::LIGHT_PROTOCOL,
		handlers: vec![on_demand.clone()],
		attached_protos: attached_protos,
	};
	let light_sync = LightSync::new(sync_params).map_err(|e| format!("Error starting network: {}", e))?;
	let light_sync = Arc::new(light_sync);
	*sync_handle.write() = Arc::downgrade(&light_sync);

	// spin up event loop
	let event_loop = EventLoop::spawn();

	// queue cull service.
	let queue_cull = Arc::new(::light_helpers::QueueCull {
		client: client.clone(),
		sync: light_sync.clone(),
		on_demand: on_demand.clone(),
		txq: txq.clone(),
		remote: event_loop.remote(),
	});

	service.register_handler(queue_cull).map_err(|e| format!("Error attaching service: {:?}", e))?;

	// start the network.
	light_sync.start_network();

	let cpu_pool = CpuPool::new(4);

	// fetch service
	let fetch = fetch::Client::new().map_err(|e| format!("Error starting fetch client: {:?}", e))?;
	let passwords = passwords_from_files(&cmd.acc_conf.password_files)?;

	// prepare account provider
	let account_provider = Arc::new(prepare_account_provider(&cmd.spec, &cmd.dirs, &spec.data_dir, cmd.acc_conf, &passwords)?);
	let rpc_stats = Arc::new(informant::RpcStats::default());

	// the dapps server
	let signer_service = Arc::new(signer::new_service(&cmd.ws_conf, &cmd.logger_config));
	let (node_health, dapps_deps) = {
		let contract_client = ::dapps::LightRegistrar {
			client: client.clone(),
			sync: light_sync.clone(),
			on_demand: on_demand.clone(),
		};

		struct LightSyncStatus(Arc<LightSync>);
		impl fmt::Debug for LightSyncStatus {
			fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
				write!(fmt, "Light Sync Status")
			}
		}
		impl node_health::SyncStatus for LightSyncStatus {
			fn is_major_importing(&self) -> bool { self.0.is_major_importing() }
			fn peers(&self) -> (usize, usize) {
				let peers = sync::LightSyncProvider::peer_numbers(&*self.0);
				(peers.connected, peers.max)
			}
		}

		let sync_status = Arc::new(LightSyncStatus(light_sync.clone()));
		let node_health = node_health::NodeHealth::new(
			sync_status.clone(),
			node_health::TimeChecker::new(&cmd.ntp_servers, cpu_pool.clone()),
			event_loop.remote(),
		);

		(node_health.clone(), dapps::Dependencies {
			sync_status,
			node_health,
			contract_client: Arc::new(contract_client),
			fetch: fetch.clone(),
			pool: cpu_pool.clone(),
			signer: signer_service.clone(),
		})
	};

	let dapps_middleware = dapps::new(cmd.dapps_conf.clone(), dapps_deps.clone())?;

	// start RPCs
	let dapps_service = dapps::service(&dapps_middleware);
	let deps_for_rpc_apis = Arc::new(rpc_apis::LightDependencies {
		signer_service: signer_service,
		client: client.clone(),
		sync: light_sync.clone(),
		net: light_sync.clone(),
		health: node_health,
		secret_store: account_provider,
		logger: logger,
		settings: Arc::new(cmd.net_settings),
		on_demand: on_demand,
		cache: cache.clone(),
		transaction_queue: txq,
		dapps_service: dapps_service,
		dapps_address: cmd.dapps_conf.address(cmd.http_conf.address()),
		ws_address: cmd.ws_conf.address(),
		fetch: fetch,
		pool: cpu_pool.clone(),
		geth_compatibility: cmd.geth_compatibility,
		remote: event_loop.remote(),
		whisper_rpc: whisper_factory,
		private_tx_service: None, //TODO: add this to client.
		gas_price_percentile: cmd.gas_price_percentile,
		poll_lifetime: cmd.poll_lifetime
	});

	let dependencies = rpc::Dependencies {
		apis: deps_for_rpc_apis.clone(),
		remote: event_loop.raw_remote(),
		stats: rpc_stats.clone(),
		pool: if cmd.http_conf.processing_threads > 0 {
			Some(rpc::CpuPool::new(cmd.http_conf.processing_threads))
		} else {
			None
		},
	};

	// start rpc servers
	let rpc_direct = rpc::setup_apis(rpc_apis::ApiSet::All, &dependencies);
	let ws_server = rpc::new_ws(cmd.ws_conf, &dependencies)?;
	let http_server = rpc::new_http("HTTP JSON-RPC", "jsonrpc", cmd.http_conf.clone(), &dependencies, dapps_middleware)?;
	let ipc_server = rpc::new_ipc(cmd.ipc_conf, &dependencies)?;

	// the informant
	let informant = Arc::new(Informant::new(
		LightNodeInformantData {
			client: client.clone(),
			sync: light_sync.clone(),
			cache: cache,
		},
		None,
		Some(rpc_stats),
		cmd.logger_config.color,
	));
	service.add_notify(informant.clone());
	service.register_handler(informant.clone()).map_err(|_| "Unable to register informant handler".to_owned())?;

	Ok(RunningClient {
		inner: RunningClientInner::Light {
			rpc: rpc_direct,
			informant,
			client,
			keep_alive: Box::new((event_loop, service, ws_server, http_server, ipc_server)),
		}
	})
}

fn execute_impl<Cr, Rr>(cmd: RunCmd, logger: Arc<RotatingLogger>, on_client_rq: Cr,
						on_updater_rq: Rr) -> Result<RunningClient, String>
	where Cr: Fn(String) + 'static + Send,
		  Rr: Fn() + 'static + Send
{
	// load spec
	let spec = cmd.spec.spec(&cmd.dirs.cache)?;

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
	execute_upgrades(&cmd.dirs.base, &db_dirs, algorithm, &cmd.compaction)?;

	// create dirs used by parity
	cmd.dirs.create_dirs(cmd.dapps_conf.enabled, cmd.acc_conf.unlocked_accounts.len() == 0, cmd.secretstore_conf.enabled)?;

	// run in daemon mode
	if let Some(pid_file) = cmd.daemon {
		daemonize(pid_file)?;
	}

	//print out running parity environment
	print_running_environment(&spec.name, &cmd.dirs, &db_dirs, &cmd.dapps_conf);

	// display info about used pruning algorithm
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

	// display warning about using experimental journaldb algorithm
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
	let mut warp_sync = spec.engine.supports_warp() && cmd.warp_sync;
	if warp_sync {
		// Logging is not initialized yet, so we print directly to stderr
		if fat_db {
			warn!("Warning: Warp Sync is disabled because Fat DB is turned on.");
			warp_sync = false;
		} else if tracing {
			warn!("Warning: Warp Sync is disabled because tracing is turned on.");
			warp_sync = false;
		} else if algorithm != Algorithm::OverlayRecent {
			warn!("Warning: Warp Sync is disabled because of non-default pruning mode.");
			warp_sync = false;
		}
	}
	sync_config.warp_sync = match (warp_sync, cmd.warp_barrier) {
		(true, Some(block)) => sync::WarpSync::OnlyAndAfter(block),
		(true, _) => sync::WarpSync::Enabled,
		_ => sync::WarpSync::Disabled,
	};
	sync_config.download_old_blocks = cmd.download_old_blocks;
	sync_config.serve_light = cmd.serve_light;

	let passwords = passwords_from_files(&cmd.acc_conf.password_files)?;

	// prepare account provider
	let account_provider = Arc::new(prepare_account_provider(&cmd.spec, &cmd.dirs, &spec.data_dir, cmd.acc_conf, &passwords)?);

	let cpu_pool = CpuPool::new(4);

	// spin up event loop
	let event_loop = EventLoop::spawn();

	// fetch service
	let fetch = fetch::Client::new().map_err(|e| format!("Error starting fetch client: {:?}", e))?;

	// create miner
	let miner = Arc::new(Miner::new(
		cmd.miner_options,
		cmd.gas_pricer_conf.to_gas_pricer(fetch.clone(), cpu_pool.clone()),
		&spec,
		Some(account_provider.clone())
	));
	miner.set_author(cmd.miner_extras.author, None).expect("Fails only if password is Some; password is None; qed");
	miner.set_gas_range_target(cmd.miner_extras.gas_range_target);
	miner.set_extra_data(cmd.miner_extras.extra_data);
	if !cmd.miner_extras.work_notify.is_empty() {
		miner.add_work_listener(Box::new(
			WorkPoster::new(&cmd.miner_extras.work_notify, fetch.clone(), event_loop.remote())
		));
	}
	let engine_signer = cmd.miner_extras.engine_signer;
	if engine_signer != Default::default() {
		// Check if engine signer exists
		if !account_provider.has_account(engine_signer) {
			return Err(format!("Consensus signer account not found for the current chain. {}", build_create_account_hint(&cmd.spec, &cmd.dirs.keys)));
		}

		// Check if any passwords have been read from the password file(s)
		if passwords.is_empty() {
			return Err(format!("No password found for the consensus signer {}. {}", engine_signer, VERIFY_PASSWORD_HINT));
		}

		// Attempt to sign in the engine signer.
		if !passwords.iter().any(|p| miner.set_author(engine_signer, Some(p.to_owned())).is_ok()) {
			return Err(format!("No valid password for the consensus signer {}. {}", engine_signer, VERIFY_PASSWORD_HINT));
		}
	}

	// display warning if using --no-hardcoded-sync
	if cmd.no_hardcoded_sync {
		warn!("The --no-hardcoded-sync flag has no effect if you don't use --light");
	}

	// create client config
	let mut client_config = to_client_config(
		&cmd.cache_config,
		spec.name.to_lowercase(),
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

	let restoration_db_handler = db::restoration_db_handler(&client_path, &client_config);
	let client_db = restoration_db_handler.open(&client_path)
		.map_err(|e| format!("Failed to open database {:?}", e))?;

	// create client service.
	let service = ClientService::start(
		client_config,
		&spec,
		client_db,
		&snapshot_path,
		restoration_db_handler,
		&cmd.dirs.ipc_path(),
		miner.clone(),
		account_provider.clone(),
		Box::new(SecretStoreEncryptor::new(cmd.private_encryptor_conf, fetch.clone()).map_err(|e| e.to_string())?),
		cmd.private_provider_conf,
	).map_err(|e| format!("Client service error: {:?}", e))?;

	let connection_filter_address = spec.params().node_permission_contract;
	// drop the spec to free up genesis state.
	drop(spec);

	// take handle to client
	let client = service.client();
	// Update miners block gas limit
	miner.update_transaction_queue_limits(*client.best_block_header().gas_limit());

	// take handle to private transactions service
	let private_tx_service = service.private_tx_service();
	let private_tx_provider = private_tx_service.provider();
	let connection_filter = connection_filter_address.map(|a| Arc::new(NodeFilter::new(Arc::downgrade(&client) as Weak<BlockChainClient>, a)));
	let snapshot_service = service.snapshot_service();

	// initialize the local node information store.
	let store = {
		let db = service.db();
		let node_info = FullNodeInfo {
			miner: match cmd.no_persistent_txqueue {
				true => None,
				false => Some(miner.clone()),
			}
		};

		let store = ::local_store::create(db.key_value().clone(), ::ethcore::db::COL_NODE_INFO, node_info);

		if cmd.no_persistent_txqueue {
			info!("Running without a persistent transaction queue.");

			if let Err(e) = store.clear() {
				warn!("Error clearing persistent transaction queue: {}", e);
			}
		}

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
		stratum::Stratum::register(stratum_config, miner.clone(), Arc::downgrade(&client))
			.map_err(|e| format!("Stratum start error: {:?}", e))?;
	}

	let mut attached_protos = Vec::new();

	let whisper_factory = if cmd.whisper.enabled {
		let whisper_factory = ::whisper::setup(cmd.whisper.target_message_pool_size, &mut attached_protos)
			.map_err(|e| format!("Failed to initialize whisper: {}", e))?;

		whisper_factory
	} else {
		None
	};

	// create sync object
	let (sync_provider, manage_network, chain_notify) = modules::sync(
		sync_config,
		net_conf.clone().into(),
		client.clone(),
		snapshot_service.clone(),
		private_tx_service.clone(),
		client.clone(),
		&cmd.logger_config,
		attached_protos,
		connection_filter.clone().map(|f| f as Arc<::sync::ConnectionFilter + 'static>),
	).map_err(|e| format!("Sync error: {}", e))?;

	service.add_notify(chain_notify.clone());

	// provider not added to a notification center is effectively disabled
	// TODO [debris] refactor it later on
	if cmd.private_tx_enabled {
		service.add_notify(private_tx_provider.clone());
		// TODO [ToDr] PrivateTX should use separate notifications
		// re-using ChainNotify for this is a bit abusive.
		private_tx_provider.add_notify(chain_notify.clone());
	}

	// start network
	if network_enabled {
		chain_notify.start();
	}

	let contract_client = Arc::new(::dapps::FullRegistrar::new(client.clone()));

	// the updater service
	let updater_fetch = fetch.clone();
	let updater = Updater::new(
		Arc::downgrade(&(service.client() as Arc<BlockChainClient>)),
		Arc::downgrade(&sync_provider),
		update_policy,
		hash_fetch::Client::with_fetch(contract_client.clone(), cpu_pool.clone(), updater_fetch, event_loop.remote())
	);
	service.add_notify(updater.clone());

	// set up dependencies for rpc servers
	let rpc_stats = Arc::new(informant::RpcStats::default());
	let secret_store = account_provider.clone();
	let signer_service = Arc::new(signer::new_service(&cmd.ws_conf, &cmd.logger_config));

	// the dapps server
	let (node_health, dapps_deps) = {
		let (sync, client) = (sync_provider.clone(), client.clone());

		struct SyncStatus(Arc<sync::SyncProvider>, Arc<Client>, sync::NetworkConfiguration);
		impl fmt::Debug for SyncStatus {
			fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
				write!(fmt, "Dapps Sync Status")
			}
		}
		impl node_health::SyncStatus for SyncStatus {
			fn is_major_importing(&self) -> bool {
				is_major_importing(Some(self.0.status().state), self.1.queue_info())
			}
			fn peers(&self) -> (usize, usize) {
				let status = self.0.status();
				(status.num_peers, status.current_max_peers(self.2.min_peers, self.2.max_peers) as usize)
			}
		}

		let sync_status = Arc::new(SyncStatus(sync, client, net_conf));
		let node_health = node_health::NodeHealth::new(
			sync_status.clone(),
			node_health::TimeChecker::new(&cmd.ntp_servers, cpu_pool.clone()),
			event_loop.remote(),
		);
		(node_health.clone(), dapps::Dependencies {
			sync_status,
			node_health,
			contract_client,
			fetch: fetch.clone(),
			pool: cpu_pool.clone(),
			signer: signer_service.clone(),
		})
	};
	let dapps_middleware = dapps::new(cmd.dapps_conf.clone(), dapps_deps.clone())?;

	let dapps_service = dapps::service(&dapps_middleware);
	let deps_for_rpc_apis = Arc::new(rpc_apis::FullDependencies {
		signer_service: signer_service,
		snapshot: snapshot_service.clone(),
		client: client.clone(),
		sync: sync_provider.clone(),
		health: node_health,
		net: manage_network.clone(),
		secret_store: secret_store,
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: Arc::new(cmd.net_settings.clone()),
		net_service: manage_network.clone(),
		updater: updater.clone(),
		geth_compatibility: cmd.geth_compatibility,
		dapps_service: dapps_service,
		dapps_address: cmd.dapps_conf.address(cmd.http_conf.address()),
		ws_address: cmd.ws_conf.address(),
		fetch: fetch.clone(),
		pool: cpu_pool.clone(),
		remote: event_loop.remote(),
		whisper_rpc: whisper_factory,
		private_tx_service: Some(private_tx_service.clone()),
		gas_price_percentile: cmd.gas_price_percentile,
		poll_lifetime: cmd.poll_lifetime,
	});

	let dependencies = rpc::Dependencies {
		apis: deps_for_rpc_apis.clone(),
		remote: event_loop.raw_remote(),
		stats: rpc_stats.clone(),
		pool: if cmd.http_conf.processing_threads > 0 {
			Some(rpc::CpuPool::new(cmd.http_conf.processing_threads))
		} else {
			None
		},

	};

	// start rpc servers
	let rpc_direct = rpc::setup_apis(rpc_apis::ApiSet::All, &dependencies);
	let ws_server = rpc::new_ws(cmd.ws_conf.clone(), &dependencies)?;
	let ipc_server = rpc::new_ipc(cmd.ipc_conf, &dependencies)?;
	let http_server = rpc::new_http("HTTP JSON-RPC", "jsonrpc", cmd.http_conf.clone(), &dependencies, dapps_middleware)?;

	// secret store key server
	let secretstore_deps = secretstore::Dependencies {
		client: client.clone(),
		sync: sync_provider.clone(),
		miner: miner,
		account_provider: account_provider,
		accounts_passwords: &passwords,
	};
	let secretstore_key_server = secretstore::start(cmd.secretstore_conf.clone(), secretstore_deps)?;

	// the ipfs server
	let ipfs_server = ipfs::start_server(cmd.ipfs_conf.clone(), client.clone())?;

	// the informant
	let informant = Arc::new(Informant::new(
		FullNodeInformantData {
			client: service.client(),
			sync: Some(sync_provider.clone()),
			net: Some(manage_network.clone()),
		},
		Some(snapshot_service.clone()),
		Some(rpc_stats.clone()),
		cmd.logger_config.color,
	));
	service.add_notify(informant.clone());
	service.register_io_handler(informant.clone()).map_err(|_| "Unable to register informant handler".to_owned())?;

	// save user defaults
	user_defaults.is_first_launch = false;
	user_defaults.pruning = algorithm;
	user_defaults.tracing = tracing;
	user_defaults.fat_db = fat_db;
	user_defaults.mode = mode;
	user_defaults.save(&user_defaults_path)?;

	// tell client how to save the default mode if it gets changed.
	client.on_user_defaults_change(move |mode: Option<Mode>| {
		if let Some(mode) = mode {
			user_defaults.mode = mode;
		}
		let _ = user_defaults.save(&user_defaults_path);	// discard failures - there's nothing we can do
	});

	// the watcher must be kept alive.
	let watcher = match cmd.no_periodic_snapshot {
		true => None,
		false => {
			let sync = sync_provider.clone();
			let client = client.clone();
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

	client.set_exit_handler(on_client_rq);
	updater.set_exit_handler(on_updater_rq);

	Ok(RunningClient {
		inner: RunningClientInner::Full {
			rpc: rpc_direct,
			informant,
			client,
			client_service: Arc::new(service),
			keep_alive: Box::new((watcher, updater, ws_server, http_server, ipc_server, secretstore_key_server, ipfs_server, event_loop)),
		}
	})
}

/// Parity client currently executing in background threads.
///
/// Should be destroyed by calling `shutdown()`, otherwise execution will continue in the
/// background.
pub struct RunningClient {
	inner: RunningClientInner,
}

enum RunningClientInner {
	Light {
		rpc: jsonrpc_core::MetaIoHandler<Metadata, informant::Middleware<rpc_apis::LightClientNotifier>>,
		informant: Arc<Informant<LightNodeInformantData>>,
		client: Arc<LightClient>,
		keep_alive: Box<Any>,
	},
	Full {
		rpc: jsonrpc_core::MetaIoHandler<Metadata, informant::Middleware<informant::ClientNotifier>>,
		informant: Arc<Informant<FullNodeInformantData>>,
		client: Arc<Client>,
		client_service: Arc<ClientService>,
		keep_alive: Box<Any>,
	},
}

impl RunningClient {
	/// Performs a synchronous RPC query.
	/// Blocks execution until the result is ready.
	pub fn rpc_query_sync(&self, request: &str) -> Option<String> {
		let metadata = Metadata {
			origin: Origin::CApi,
			session: None,
		};

		match self.inner {
			RunningClientInner::Light { ref rpc, .. } => {
				rpc.handle_request_sync(request, metadata)
			},
			RunningClientInner::Full { ref rpc, .. } => {
				rpc.handle_request_sync(request, metadata)
			},
		}
	}

	/// Shuts down the client.
	pub fn shutdown(self) {
		match self.inner {
			RunningClientInner::Light { rpc, informant, client, keep_alive } => {
				// Create a weak reference to the client so that we can wait on shutdown
				// until it is dropped
				let weak_client = Arc::downgrade(&client);
				drop(rpc);
				drop(keep_alive);
				informant.shutdown();
				drop(informant);
				drop(client);
				wait_for_drop(weak_client);
			},
			RunningClientInner::Full { rpc, informant, client, client_service, keep_alive } => {
				info!("Finishing work, please wait...");
				// Create a weak reference to the client so that we can wait on shutdown
				// until it is dropped
				let weak_client = Arc::downgrade(&client);
				// Shutdown and drop the ServiceClient
				client_service.shutdown();
				drop(client_service);
				// drop this stuff as soon as exit detected.
				drop(rpc);
				drop(keep_alive);
				// to make sure timer does not spawn requests while shutdown is in progress
				informant.shutdown();
				// just Arc is dropping here, to allow other reference release in its default time
				drop(informant);
				drop(client);
				wait_for_drop(weak_client);
			}
		}
	}
}

/// Executes the given run command.
///
/// `on_client_rq` is the action to perform when the client receives an RPC request to be restarted
/// with a different chain.
///
/// `on_updater_rq` is the action to perform when the updater has a new binary to execute.
///
/// On error, returns what to print on stderr.
pub fn execute<Cr, Rr>(cmd: RunCmd, logger: Arc<RotatingLogger>,
						on_client_rq: Cr, on_updater_rq: Rr) -> Result<RunningClient, String>
	where Cr: Fn(String) + 'static + Send,
		  Rr: Fn() + 'static + Send
{
	if cmd.light {
		execute_light_impl(cmd, logger)
	} else {
		execute_impl(cmd, logger, on_client_rq, on_updater_rq)
	}
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

fn print_running_environment(spec_name: &String, dirs: &Directories, db_dirs: &DatabaseDirectories, dapps_conf: &dapps::Configuration) {
	info!("Starting {}", Colour::White.bold().paint(version()));
	info!("Keys path {}", Colour::White.bold().paint(dirs.keys_path(spec_name).to_string_lossy().into_owned()));
	info!("DB path {}", Colour::White.bold().paint(db_dirs.db_root_path().to_string_lossy().into_owned()));
	info!("Path to dapps {}", Colour::White.bold().paint(dapps_conf.dapps_path.to_string_lossy().into_owned()));
}

fn prepare_account_provider(spec: &SpecType, dirs: &Directories, data_dir: &str, cfg: AccountsConfig, passwords: &[String]) -> Result<AccountProvider, String> {
	use ethcore::ethstore::EthStore;
	use ethcore::ethstore::accounts_dir::RootDiskDirectory;

	let path = dirs.keys_path(data_dir);
	upgrade_key_location(&dirs.legacy_keys_path(cfg.testnet), &path);
	let dir = Box::new(RootDiskDirectory::create(&path).map_err(|e| format!("Could not open keys directory: {}", e))?);
	let account_settings = AccountProviderSettings {
		enable_hardware_wallets: cfg.enable_hardware_wallets,
		hardware_wallet_classic_key: spec == &SpecType::Classic,
		unlock_keep_secret: cfg.enable_fast_unlock,
		blacklisted_accounts: 	match *spec {
			SpecType::Morden | SpecType::Ropsten | SpecType::Kovan | SpecType::Dev => vec![],
			_ => vec![
				"00a329c0648769a73afac7f9381e08fb43dbea72".into()
			],
		},
	};

	let ethstore = EthStore::open_with_iterations(dir, cfg.iterations).map_err(|e| format!("Could not open keys directory: {}", e))?;
	if cfg.refresh_time > 0 {
		ethstore.set_refresh_time(::std::time::Duration::from_secs(cfg.refresh_time));
	}
	let account_provider = AccountProvider::new(
		Box::new(ethstore),
		account_settings,
	);

	for a in cfg.unlocked_accounts {
		// Check if the account exists
		if !account_provider.has_account(a) {
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

	// Add development account if running dev chain:
	if let SpecType::Dev = *spec {
		insert_dev_account(&account_provider);
	}

	Ok(account_provider)
}

fn insert_dev_account(account_provider: &AccountProvider) {
	let secret: ethkey::Secret = "4d5db4107d237df6a3d58ee5f70ae63d73d7658d4026f2eefd2f204c81682cb7".into();
	let dev_account = ethkey::KeyPair::from_secret(secret.clone()).expect("Valid secret produces valid key;qed");
	if !account_provider.has_account(dev_account.address()) {
		match account_provider.insert_account(secret, "") {
			Err(e) => warn!("Unable to add development account: {}", e),
			Ok(address) => {
				let _ = account_provider.set_account_name(address.clone(), "Development Account".into());
				let _ = account_provider.set_account_meta(address, ::serde_json::to_string(&(vec![
					("description", "Never use this account outside of development chain!"),
					("passwordHint","Password is empty string"),
				].into_iter().collect::<::std::collections::HashMap<_,_>>())).expect("Serialization of hashmap does not fail."));
			},
		}
	}
}

// Construct an error `String` with an adaptive hint on how to create an account.
fn build_create_account_hint(spec: &SpecType, keys: &str) -> String {
	format!("You can create an account via RPC, UI or `parity account new --chain {} --keys-path {}`.", spec, keys)
}

fn wait_for_drop<T>(w: Weak<T>) {
	let sleep_duration = Duration::from_secs(1);
	let warn_timeout = Duration::from_secs(60);
	let max_timeout = Duration::from_secs(300);

	let instant = Instant::now();
	let mut warned = false;

	while instant.elapsed() < max_timeout {
		if w.upgrade().is_none() {
			return;
		}

		if !warned && instant.elapsed() > warn_timeout {
			warned = true;
			warn!("Shutdown is taking longer than expected.");
		}

		thread::sleep(sleep_duration);
	}

	warn!("Shutdown timeout reached, exiting uncleanly.");
}
