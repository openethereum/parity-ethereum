// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::any::Any;
use std::sync::{Arc, Weak, atomic};
use std::time::{Duration, Instant};
use std::thread;

use ansi_term::Colour;
use bytes::Bytes;
use call_contract::CallContract;
use client_traits::{BlockInfo, BlockChainClient};
use ethcore::client::{Client, DatabaseCompactionProfile, VMType};
use ethcore::miner::{self, stratum, Miner, MinerService, MinerOptions};
use snapshot::{self, SnapshotConfiguration};
use spec::SpecParams;
use verification::queue::VerifierSettings;
use ethcore_logger::{Config as LogConfig, RotatingLogger};
use ethcore_service::ClientService;
use ethereum_types::Address;
use futures::{IntoFuture, Stream};
use hash_fetch::{self, fetch};
use informant::{Informant, LightNodeInformantData, FullNodeInformantData};
use journaldb::Algorithm;
use light::Cache as LightDataCache;
use miner::external::ExternalMiner;
use miner::work_notify::WorkPoster;
use node_filter::NodeFilter;
use parity_runtime::Runtime;
use sync::{self, SyncConfig, PrivateTxHandler};
use types::{
	client_types::Mode,
	engines::OptimizeFor,
	ids::BlockId,
	snapshot::Snapshotting,
};
use parity_rpc::{
	Origin, Metadata, NetworkSettings, informant, PubSubSession, FutureResult, FutureResponse, FutureOutput
};
use updater::{UpdatePolicy, Updater};
use parity_version::version;
use ethcore_private_tx::{ProviderConfig, EncryptorConfig, SecretStoreEncryptor};
use params::{
	SpecType, Pruning, AccountsConfig, GasPricerConfig, MinerExtras, Switch,
	tracing_switch_to_bool, fatdb_switch_to_bool, mode_switch_to_bool
};
use account_utils;
use helpers::{to_client_config, execute_upgrades, passwords_from_files};
use dir::{Directories, DatabaseDirectories};
use cache::CacheConfig;
use user_defaults::UserDefaults;
use ipfs;
use jsonrpc_core;
use modules;
use registrar::{RegistrarClient, Asynchronous};
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

// Full client number of DNS threads
const FETCH_FULL_NUM_DNS_THREADS: usize = 4;

// Light client number of DNS threads
const FETCH_LIGHT_NUM_DNS_THREADS: usize = 1;

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
	pub vm_type: VMType,
	pub geth_compatibility: bool,
	pub experimental_rpcs: bool,
	pub net_settings: NetworkSettings,
	pub ipfs_conf: ipfs::Configuration,
	pub secretstore_conf: secretstore::Configuration,
	pub private_provider_conf: ProviderConfig,
	pub private_encryptor_conf: EncryptorConfig,
	pub private_tx_enabled: bool,
	pub name: String,
	pub custom_bootnodes: bool,
	pub stratum: Option<stratum::Options>,
	pub snapshot_conf: SnapshotConfiguration,
	pub check_seal: bool,
	pub allow_missing_blocks: bool,
	pub download_old_blocks: bool,
	pub verifier_settings: VerifierSettings,
	pub serve_light: bool,
	pub light: bool,
	pub no_persistent_txqueue: bool,
	pub no_hardcoded_sync: bool,
	pub max_round_blocks_to_import: usize,
	pub on_demand_response_time_window: Option<u64>,
	pub on_demand_request_backoff_start: Option<u64>,
	pub on_demand_request_backoff_max: Option<u64>,
	pub on_demand_request_backoff_rounds_max: Option<usize>,
	pub on_demand_request_consecutive_failures: Option<usize>,
}

// node info fetcher for the local store.
struct FullNodeInfo {
	miner: Option<Arc<Miner>>, // TODO: only TXQ needed, just use that after decoupling.
}

impl ::local_store::NodeInfo for FullNodeInfo {
	fn pending_transactions(&self) -> Vec<::types::transaction::PendingTransaction> {
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
fn execute_light_impl<Cr>(cmd: RunCmd, logger: Arc<RotatingLogger>, on_client_rq: Cr) -> Result<RunningClient, String>
	where Cr: Fn(String) + 'static + Send
{
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
	cmd.dirs.create_dirs(cmd.acc_conf.unlocked_accounts.len() == 0, cmd.secretstore_conf.enabled)?;

	//print out running parity environment
	print_running_environment(&spec.data_dir, &cmd.dirs, &db_dirs);

	info!("Running in experimental {} mode.", Colour::Blue.bold().paint("Light Client"));

	// TODO: configurable cache size.
	let cache = LightDataCache::new(Default::default(), Duration::from_secs(60 * GAS_CORPUS_EXPIRATION_MINUTES));
	let cache = Arc::new(Mutex::new(cache));

	// start client and create transaction queue.
	let mut config = light_client::Config {
		queue: Default::default(),
		chain_column: ::ethcore_db::COL_LIGHT_CHAIN,
		verify_full: true,
		check_seal: cmd.check_seal,
		no_hardcoded_sync: cmd.no_hardcoded_sync,
	};

	config.queue.max_mem_use = cmd.cache_config.queue() as usize * 1024 * 1024;
	config.queue.verifier_settings = cmd.verifier_settings;

	// start on_demand service.

	let response_time_window = cmd.on_demand_response_time_window.map_or(
		::light::on_demand::DEFAULT_RESPONSE_TIME_TO_LIVE,
		|s| Duration::from_secs(s)
	);

	let request_backoff_start = cmd.on_demand_request_backoff_start.map_or(
		::light::on_demand::DEFAULT_REQUEST_MIN_BACKOFF_DURATION,
		|s| Duration::from_secs(s)
	);

	let request_backoff_max = cmd.on_demand_request_backoff_max.map_or(
		::light::on_demand::DEFAULT_REQUEST_MAX_BACKOFF_DURATION,
		|s| Duration::from_secs(s)
	);

	let on_demand = Arc::new({
		::light::on_demand::OnDemand::new(
			cache.clone(),
			response_time_window,
			request_backoff_start,
			request_backoff_max,
			cmd.on_demand_request_backoff_rounds_max.unwrap_or(::light::on_demand::DEFAULT_MAX_REQUEST_BACKOFF_ROUNDS),
			cmd.on_demand_request_consecutive_failures.unwrap_or(::light::on_demand::DEFAULT_NUM_CONSECUTIVE_FAILED_REQUESTS)
		)
	});

	let sync_handle = Arc::new(RwLock::new(Weak::new()));
	let fetch = ::light_helpers::EpochFetch {
		on_demand: on_demand.clone(),
		sync: sync_handle.clone(),
	};

	// initialize database.
	let db = db::open_db(&db_dirs.client_path(algorithm).to_str().expect("DB path could not be converted to string."),
						 &cmd.cache_config,
						 &cmd.compaction).map_err(|e| format!("Failed to open database {:?}", e))?;

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

	// set network path.
	net_conf.net_config_path = Some(db_dirs.network_path().to_string_lossy().into_owned());
	let sync_params = LightSyncParams {
		network_config: net_conf.into_basic().map_err(|e| format!("Failed to produce network config: {}", e))?,
		client: Arc::new(provider),
		network_id: cmd.network_id.unwrap_or(spec.network_id()),
		subprotocol_name: sync::LIGHT_PROTOCOL,
		handlers: vec![on_demand.clone()],
	};
	let light_sync = LightSync::new(sync_params).map_err(|e| format!("Error starting network: {}", e))?;
	let light_sync = Arc::new(light_sync);
	*sync_handle.write() = Arc::downgrade(&light_sync);

	// spin up event loop
	let runtime = Runtime::with_default_thread_count();

	// start the network.
	light_sync.start_network();

	// fetch service
	let fetch = fetch::Client::new(FETCH_LIGHT_NUM_DNS_THREADS).map_err(|e| format!("Error starting fetch client: {:?}", e))?;
	let passwords = passwords_from_files(&cmd.acc_conf.password_files)?;

	// prepare account provider
	let account_provider = Arc::new(account_utils::prepare_account_provider(&cmd.spec, &cmd.dirs, &spec.data_dir, cmd.acc_conf, &passwords)?);
	let rpc_stats = Arc::new(informant::RpcStats::default());

	// the dapps server
	let signer_service = Arc::new(signer::new_service(&cmd.ws_conf, &cmd.logger_config));

	// start RPCs
	let deps_for_rpc_apis = Arc::new(rpc_apis::LightDependencies {
		signer_service,
		client: client.clone(),
		sync: light_sync.clone(),
		net: light_sync.clone(),
		accounts: account_provider,
		logger,
		settings: Arc::new(cmd.net_settings),
		on_demand,
		cache: cache.clone(),
		transaction_queue: txq,
		ws_address: cmd.ws_conf.address(),
		fetch,
		geth_compatibility: cmd.geth_compatibility,
		experimental_rpcs: cmd.experimental_rpcs,
		executor: runtime.executor(),
		private_tx_service: None, //TODO: add this to client.
		gas_price_percentile: cmd.gas_price_percentile,
		poll_lifetime: cmd.poll_lifetime
	});

	let dependencies = rpc::Dependencies {
		apis: deps_for_rpc_apis.clone(),
		executor: runtime.executor(),
		stats: rpc_stats.clone(),
	};

	// start rpc servers
	let rpc_direct = rpc::setup_apis(rpc_apis::ApiSet::All, &dependencies);
	let ws_server = rpc::new_ws(cmd.ws_conf, &dependencies)?;
	let http_server = rpc::new_http("HTTP JSON-RPC", "jsonrpc", cmd.http_conf.clone(), &dependencies)?;
	let ipc_server = rpc::new_ipc(cmd.ipc_conf, &dependencies)?;

	// the informant
	let informant = Arc::new(Informant::new(
		LightNodeInformantData {
			client: client.clone(),
			sync: light_sync.clone(),
			cache,
		},
		None,
		Some(rpc_stats),
		cmd.logger_config.color,
	));
	service.add_notify(informant.clone());
	service.register_handler(informant.clone()).map_err(|_| "Unable to register informant handler".to_owned())?;

	client.set_exit_handler(on_client_rq);

	Ok(RunningClient {
		inner: RunningClientInner::Light {
			rpc: rpc_direct,
			informant,
			client,
			keep_alive: Box::new((service, ws_server, http_server, ipc_server, runtime)),
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
	cmd.dirs.create_dirs(cmd.acc_conf.unlocked_accounts.len() == 0, cmd.secretstore_conf.enabled)?;

	//print out running parity environment
	print_running_environment(&spec.data_dir, &cmd.dirs, &db_dirs);

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
	let snapshot_supported =
		if let Snapshotting::Unsupported = spec.engine.snapshot_mode() {
			false
		} else {
			true
		};

	let mut warp_sync = snapshot_supported && cmd.warp_sync;
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
	let account_provider = Arc::new(account_utils::prepare_account_provider(&cmd.spec, &cmd.dirs, &spec.data_dir, cmd.acc_conf, &passwords)?);

	// spin up event loop
	let runtime = Runtime::with_default_thread_count();

	// fetch service
	let fetch = fetch::Client::new(FETCH_FULL_NUM_DNS_THREADS).map_err(|e| format!("Error starting fetch client: {:?}", e))?;

	let txpool_size = cmd.miner_options.pool_limits.max_count;
	// create miner
	let miner = Arc::new(Miner::new(
		cmd.miner_options,
		cmd.gas_pricer_conf.to_gas_pricer(fetch.clone(), runtime.executor()),
		&spec,
		(
			cmd.miner_extras.local_accounts,
			account_utils::miner_local_accounts(account_provider.clone()),
		)
	));
	miner.set_author(miner::Author::External(cmd.miner_extras.author));
	miner.set_gas_range_target(cmd.miner_extras.gas_range_target);
	miner.set_extra_data(cmd.miner_extras.extra_data);

	if !cmd.miner_extras.work_notify.is_empty() {
		miner.add_work_listener(Box::new(
			WorkPoster::new(&cmd.miner_extras.work_notify, fetch.clone(), runtime.executor())
		));
	}

	let engine_signer = cmd.miner_extras.engine_signer;
	if engine_signer != Default::default() {
		if let Some(author) = account_utils::miner_author(&cmd.spec, &cmd.dirs, &account_provider, engine_signer, &passwords)? {
			miner.set_author(author);
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
		cmd.vm_type,
		cmd.name,
		algorithm,
		cmd.pruning_history,
		cmd.pruning_memory,
		cmd.check_seal,
		cmd.max_round_blocks_to_import,
	);

	client_config.queue.verifier_settings = cmd.verifier_settings;
	client_config.transaction_verification_queue_size = ::std::cmp::max(2048, txpool_size / 4);
	client_config.snapshot = cmd.snapshot_conf.clone();

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

	let private_tx_signer = account_utils::private_tx_signer(account_provider.clone(), &passwords)?;

	// create client service.
	let service = ClientService::start(
		client_config,
		&spec,
		client_db,
		&snapshot_path,
		restoration_db_handler,
		&cmd.dirs.ipc_path(),
		miner.clone(),
		private_tx_signer.clone(),
		Box::new(SecretStoreEncryptor::new(cmd.private_encryptor_conf.clone(), fetch.clone(), private_tx_signer).map_err(|e| e.to_string())?),
		cmd.private_provider_conf,
		cmd.private_encryptor_conf,
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
	let connection_filter = connection_filter_address.map(|a| Arc::new(NodeFilter::new(Arc::downgrade(&client) as Weak<dyn BlockChainClient>, a)));
	let snapshot_service = service.snapshot_service();
	if let Some(filter) = connection_filter.clone() {
		service.add_notify(filter.clone());
	}
	// initialize the local node information store.
	let store = {
		let db = service.db();
		let node_info = FullNodeInfo {
			miner: match cmd.no_persistent_txqueue {
				true => None,
				false => Some(miner.clone()),
			}
		};

		let store = ::local_store::create(db.key_value().clone(), ::ethcore_db::COL_NODE_INFO, node_info);

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

	let (private_tx_sync, private_state) = match cmd.private_tx_enabled {
		true => (Some(private_tx_service.clone() as Arc<dyn PrivateTxHandler>), Some(private_tx_provider.private_state_db())),
		false => (None, None),
	};

	// create sync object
	let (sync_provider, manage_network, chain_notify, priority_tasks) = modules::sync(
		sync_config,
		runtime.executor(),
		net_conf.clone().into(),
		client.clone(),
		snapshot_service.clone(),
		private_tx_sync,
		private_state,
		client.clone(),
		&cmd.logger_config,
		connection_filter.clone().map(|f| f as Arc<dyn sync::ConnectionFilter + 'static>),
	).map_err(|e| format!("Sync error: {}", e))?;

	service.add_notify(chain_notify.clone());

	// Propagate transactions as soon as they are imported.
	let tx = ::parking_lot::Mutex::new(priority_tasks);
	let is_ready = Arc::new(atomic::AtomicBool::new(true));
	let executor = runtime.executor();
	let pool_receiver = miner.pending_transactions_receiver();
	executor.spawn(
		pool_receiver.for_each(move |_hashes| {
			// we want to have only one PendingTransactions task in the queue.
			if is_ready.compare_and_swap(true, false, atomic::Ordering::SeqCst) {
				let task = ::sync::PriorityTask::PropagateTransactions(Instant::now(), is_ready.clone());
				// we ignore error cause it means that we are closing
				let _ = tx.lock().send(task);
			}
			Ok(())
		})
	);

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

	let contract_client = {
		struct FullRegistrar { client: Arc<Client> }
		impl RegistrarClient for FullRegistrar {
			type Call = Asynchronous;
			fn registrar_address(&self) -> Result<Address, String> {
				self.client.registrar_address()
					.ok_or_else(|| "Registrar not defined.".into())
			}
			fn call_contract(&self, address: Address, data: Bytes) -> Self::Call {
				Box::new(self.client.call_contract(BlockId::Latest, address, data).into_future())
			}
		}

		Arc::new(FullRegistrar { client: client.clone() })
	};

	// the updater service
	let updater_fetch = fetch.clone();
	let updater = Updater::new(
		&Arc::downgrade(&(service.client() as Arc<dyn BlockChainClient>)),
		&Arc::downgrade(&sync_provider),
		update_policy,
		hash_fetch::Client::with_fetch(contract_client.clone(), updater_fetch, runtime.executor())
	);
	service.add_notify(updater.clone());

	// set up dependencies for rpc servers
	let rpc_stats = Arc::new(informant::RpcStats::default());
	let secret_store = account_provider.clone();
	let signer_service = Arc::new(signer::new_service(&cmd.ws_conf, &cmd.logger_config));

	let deps_for_rpc_apis = Arc::new(rpc_apis::FullDependencies {
		signer_service: signer_service,
		snapshot: snapshot_service.clone(),
		client: client.clone(),
		sync: sync_provider.clone(),
		net: manage_network.clone(),
		accounts: secret_store,
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: Arc::new(cmd.net_settings.clone()),
		net_service: manage_network.clone(),
		updater: updater.clone(),
		geth_compatibility: cmd.geth_compatibility,
		experimental_rpcs: cmd.experimental_rpcs,
		ws_address: cmd.ws_conf.address(),
		fetch: fetch.clone(),
		executor: runtime.executor(),
		private_tx_service: Some(private_tx_service.clone()),
		gas_price_percentile: cmd.gas_price_percentile,
		poll_lifetime: cmd.poll_lifetime,
		allow_missing_blocks: cmd.allow_missing_blocks,
		no_ancient_blocks: !cmd.download_old_blocks,
	});

	let dependencies = rpc::Dependencies {
		apis: deps_for_rpc_apis.clone(),
		executor: runtime.executor(),
		stats: rpc_stats.clone(),
	};

	// start rpc servers
	let rpc_direct = rpc::setup_apis(rpc_apis::ApiSet::All, &dependencies);
	let ws_server = rpc::new_ws(cmd.ws_conf.clone(), &dependencies)?;
	let ipc_server = rpc::new_ipc(cmd.ipc_conf, &dependencies)?;
	let http_server = rpc::new_http("HTTP JSON-RPC", "jsonrpc", cmd.http_conf.clone(), &dependencies)?;

	// secret store key server
	let secretstore_deps = secretstore::Dependencies {
		client: client.clone(),
		sync: sync_provider.clone(),
		miner: miner.clone(),
		account_provider,
		accounts_passwords: &passwords,
	};
	let secretstore_key_server = secretstore::start(cmd.secretstore_conf.clone(), secretstore_deps, runtime.executor())?;

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
	user_defaults.set_mode(mode);
	user_defaults.save(&user_defaults_path)?;

	// tell client how to save the default mode if it gets changed.
	client.on_user_defaults_change(move |mode: Option<Mode>| {
		if let Some(mode) = mode {
			user_defaults.set_mode(mode);
		}
		let _ = user_defaults.save(&user_defaults_path);	// discard failures - there's nothing we can do
	});

	// the watcher must be kept alive.
	let watcher = match cmd.snapshot_conf.no_periodic {
		true => None,
		false => {
			let sync = sync_provider.clone();
			let watcher = Arc::new(snapshot::Watcher::new(
				service.client(),
				move || sync.is_major_syncing(),
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
			keep_alive: Box::new((watcher, updater, ws_server, http_server, ipc_server, secretstore_key_server, ipfs_server, runtime)),
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
		keep_alive: Box<dyn Any>,
	},
	Full {
		rpc: jsonrpc_core::MetaIoHandler<Metadata, informant::Middleware<informant::ClientNotifier>>,
		informant: Arc<Informant<FullNodeInformantData>>,
		client: Arc<Client>,
		client_service: Arc<ClientService>,
		keep_alive: Box<dyn Any>,
	},
}

impl RunningClient {
	/// Performs an asynchronous RPC query.
	// FIXME: [tomaka] This API should be better, with for example a Future
	pub fn rpc_query(&self, request: &str, session: Option<Arc<PubSubSession>>)
		-> FutureResult<FutureResponse, FutureOutput>
	{
		let metadata = Metadata {
			origin: Origin::CApi,
			session,
		};

		match self.inner {
			RunningClientInner::Light { ref rpc, .. } => rpc.handle_request(request, metadata),
			RunningClientInner::Full { ref rpc, .. } => rpc.handle_request(request, metadata),
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
				// Shutdown and drop the ClientService
				client_service.shutdown();
				trace!(target: "shutdown", "ClientService shut down");
				drop(client_service);
				trace!(target: "shutdown", "ClientService dropped");
				// drop this stuff as soon as exit detected.
				drop(rpc);
				trace!(target: "shutdown", "RPC dropped");
				drop(keep_alive);
				trace!(target: "shutdown", "KeepAlive dropped");
				// to make sure timer does not spawn requests while shutdown is in progress
				informant.shutdown();
				trace!(target: "shutdown", "Informant shut down");
				// just Arc is dropping here, to allow other reference release in its default time
				drop(informant);
				trace!(target: "shutdown", "Informant dropped");
				drop(client);
				trace!(target: "shutdown", "Client dropped");
				// This may help when debugging ref cycles. Requires nightly-only  `#![feature(weak_counts)]`
				// trace!(target: "shutdown", "Waiting for refs to Client to shutdown, strong_count={:?}, weak_count={:?}", weak_client.strong_count(), weak_client.weak_count());
				trace!(target: "shutdown", "Waiting for refs to Client to shutdown");
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
		execute_light_impl(cmd, logger, on_client_rq)
	} else {
		execute_impl(cmd, logger, on_client_rq, on_updater_rq)
	}
}

fn print_running_environment(data_dir: &str, dirs: &Directories, db_dirs: &DatabaseDirectories) {
	info!("Starting {}", Colour::White.bold().paint(version()));
	info!("Keys path {}", Colour::White.bold().paint(dirs.keys_path(data_dir).to_string_lossy().into_owned()));
	info!("DB path {}", Colour::White.bold().paint(db_dirs.db_root_path().to_string_lossy().into_owned()));
}

fn wait_for_drop<T>(w: Weak<T>) {
	const SLEEP_DURATION: Duration = Duration::from_secs(1);
	const WARN_TIMEOUT: Duration = Duration::from_secs(60);
	const MAX_TIMEOUT: Duration = Duration::from_secs(300);

	let instant = Instant::now();
	let mut warned = false;

	while instant.elapsed() < MAX_TIMEOUT {
		if w.upgrade().is_none() {
			return;
		}

		if !warned && instant.elapsed() > WARN_TIMEOUT {
			warned = true;
			warn!("Shutdown is taking longer than expected.");
		}

		thread::sleep(SLEEP_DURATION);

		// When debugging shutdown issues on a nightly build it can help to enable this with the
		// `#![feature(weak_counts)]` added to lib.rs (TODO: enable when
		// https://github.com/rust-lang/rust/issues/57977 is stable)
		// trace!(target: "shutdown", "Waiting for client to drop, strong_count={:?}, weak_count={:?}", w.strong_count(), w.weak_count());
		trace!(target: "shutdown", "Waiting for client to drop");
	}

	warn!("Shutdown timeout reached, exiting uncleanly.");
}

