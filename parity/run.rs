// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{
    any::Any,
    sync::{atomic, Arc, Weak},
    thread,
    time::{Duration, Instant},
};

use account_utils;
use ansi_term::Colour;
use cache::CacheConfig;
use db;
use dir::{DatabaseDirectories, Directories};
use ethcore::{
    client::{BlockChainClient, BlockInfo, Client, DatabaseCompactionProfile, Mode, VMType},
    miner::{self, stratum, Miner, MinerOptions, MinerService},
    snapshot::{self, SnapshotConfiguration},
    verification::queue::VerifierSettings,
};
use ethcore_logger::{Config as LogConfig, RotatingLogger};
use ethcore_service::ClientService;
use ethereum_types::H256;
use helpers::{execute_upgrades, passwords_from_files, to_client_config};
use informant::{FullNodeInformantData, Informant};
use journaldb::Algorithm;
use jsonrpc_core;
use metrics::{start_prometheus_metrics, MetricsConfiguration};
use miner::{external::ExternalMiner, work_notify::WorkPoster};
use modules;
use node_filter::NodeFilter;
use params::{
    fatdb_switch_to_bool, mode_switch_to_bool, tracing_switch_to_bool, AccountsConfig,
    GasPricerConfig, MinerExtras, Pruning, SpecType, Switch,
};
use parity_rpc::{
    informant, is_major_importing, FutureOutput, FutureResponse, FutureResult, Metadata,
    NetworkSettings, Origin, PubSubSession,
};
use parity_runtime::Runtime;
use parity_version::version;
use rpc;
use rpc_apis;
use secretstore;
use signer;
use sync::{self, SyncConfig};
use user_defaults::UserDefaults;

// how often to take periodic snapshots.
const SNAPSHOT_PERIOD: u64 = 5000;

// how many blocks to wait before starting a periodic snapshot.
const SNAPSHOT_HISTORY: u64 = 100;

// Full client number of DNS threads
const FETCH_FULL_NUM_DNS_THREADS: usize = 4;

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
    pub mode: Option<Mode>,
    pub tracing: Switch,
    pub fat_db: Switch,
    pub compaction: DatabaseCompactionProfile,
    pub vm_type: VMType,
    pub experimental_rpcs: bool,
    pub net_settings: NetworkSettings,
    pub secretstore_conf: secretstore::Configuration,
    pub name: String,
    pub custom_bootnodes: bool,
    pub stratum: Option<stratum::Options>,
    pub snapshot_conf: SnapshotConfiguration,
    pub check_seal: bool,
    pub allow_missing_blocks: bool,
    pub download_old_blocks: bool,
    pub verifier_settings: VerifierSettings,
    pub no_persistent_txqueue: bool,
    pub max_round_blocks_to_import: usize,
    pub metrics_conf: MetricsConfiguration,
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

        miner
            .local_transactions()
            .values()
            .filter_map(|status| match *status {
                ::miner::pool::local_transactions::Status::Pending(ref tx) => {
                    Some(tx.pending().clone())
                }
                _ => None,
            })
            .collect()
    }
}

/// Executes the given run command.
///
/// On error, returns what to print on stderr.
pub fn execute(cmd: RunCmd, logger: Arc<RotatingLogger>) -> Result<RunningClient, String> {
    // load spec
    let spec = cmd.spec.spec(&cmd.dirs.cache)?;

    // load genesis hash
    let genesis_hash = spec.genesis_header().hash();

    // database paths
    let db_dirs = cmd.dirs.database(
        genesis_hash,
        cmd.spec.legacy_fork_name(),
        spec.data_dir.clone(),
    );

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
    let network_enabled = match mode {
        Mode::Dark(_) | Mode::Off => false,
        _ => true,
    };

    // prepare client and snapshot paths.
    let client_path = db_dirs.client_path(algorithm);
    let snapshot_path = db_dirs.snapshot_path();

    // execute upgrades
    execute_upgrades(&cmd.dirs.base, &db_dirs, algorithm, &cmd.compaction)?;

    // create dirs used by parity
    cmd.dirs.create_dirs(
        cmd.acc_conf.unlocked_accounts.len() == 0,
        cmd.secretstore_conf.enabled,
    )?;

    //print out running parity environment
    print_running_environment(&spec.data_dir, &cmd.dirs, &db_dirs);

    // display info about used pruning algorithm
    info!(
        "State DB configuration: {}{}{}",
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
    info!(
        "Operating mode: {}",
        Colour::White.bold().paint(format!("{}", mode))
    );

    // display warning about using experimental journaldb algorithm
    if !algorithm.is_stable() {
        warn!(
            "Your chosen strategy is {}! You can re-run with --pruning to change.",
            Colour::Red.bold().paint("unstable")
        );
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
        sync_config
            .subprotocol_name
            .clone_from_slice(spec.subprotocol_name().as_bytes());
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

    let passwords = passwords_from_files(&cmd.acc_conf.password_files)?;

    // prepare account provider
    let account_provider = Arc::new(account_utils::prepare_account_provider(
        &cmd.spec,
        &cmd.dirs,
        &spec.data_dir,
        cmd.acc_conf,
        &passwords,
    )?);

    // spin up event loop
    let runtime = Runtime::with_default_thread_count();

    // fetch service
    let fetch = fetch::Client::new(FETCH_FULL_NUM_DNS_THREADS)
        .map_err(|e| format!("Error starting fetch client: {:?}", e))?;

    let txpool_size = cmd.miner_options.pool_limits.max_count;
    // create miner
    let miner = Arc::new(Miner::new(
        cmd.miner_options,
        cmd.gas_pricer_conf
            .to_gas_pricer(fetch.clone(), runtime.executor()),
        &spec,
        (
            cmd.miner_extras.local_accounts,
            account_utils::miner_local_accounts(account_provider.clone()),
        ),
    ));
    miner.set_author(miner::Author::External(cmd.miner_extras.author));
    miner.set_gas_range_target(cmd.miner_extras.gas_range_target);
    miner.set_extra_data(cmd.miner_extras.extra_data);

    if !cmd.miner_extras.work_notify.is_empty() {
        miner.add_work_listener(Box::new(WorkPoster::new(
            &cmd.miner_extras.work_notify,
            fetch.clone(),
            runtime.executor(),
        )));
    }

    let engine_signer = cmd.miner_extras.engine_signer;
    if engine_signer != Default::default() {
        if let Some(author) = account_utils::miner_author(
            &cmd.spec,
            &cmd.dirs,
            &account_provider,
            engine_signer,
            &passwords,
        )? {
            miner.set_author(author);
        }
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
    client_config.queue.verifier_settings.bad_hashes = verification_bad_blocks(&cmd.spec);
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
    let client_db = restoration_db_handler
        .open(&client_path)
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
    )
    .map_err(|e| format!("Client service error: {:?}", e))?;

    let connection_filter_address = spec.params().node_permission_contract;
    // drop the spec to free up genesis state.
    let forks = spec.hard_forks.clone();
    drop(spec);

    // take handle to client
    let client = service.client();
    // Update miners block gas limit
    miner.update_transaction_queue_limits(*client.best_block_header().gas_limit());

    let connection_filter = connection_filter_address.map(|a| {
        Arc::new(NodeFilter::new(
            Arc::downgrade(&client) as Weak<dyn BlockChainClient>,
            a,
        ))
    });
    let snapshot_service = service.snapshot_service();

    // initialize the local node information store.
    let store = {
        let db = service.db();
        let node_info = FullNodeInfo {
            miner: match cmd.no_persistent_txqueue {
                true => None,
                false => Some(miner.clone()),
            },
        };

        let store = ::local_store::create(
            db.key_value().clone(),
            ::ethcore_db::COL_NODE_INFO,
            node_info,
        );

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
    service
        .register_io_handler(store)
        .map_err(|_| "Unable to register local store handler".to_owned())?;

    // create external miner
    let external_miner = Arc::new(ExternalMiner::default());

    // start stratum
    if let Some(ref stratum_config) = cmd.stratum {
        stratum::Stratum::register(stratum_config, miner.clone(), Arc::downgrade(&client))
            .map_err(|e| format!("Stratum start error: {:?}", e))?;
    }

    // create sync object
    let (sync_provider, manage_network, chain_notify, priority_tasks) = modules::sync(
        sync_config,
        net_conf.clone().into(),
        client.clone(),
        forks,
        snapshot_service.clone(),
        &cmd.logger_config,
        connection_filter
            .clone()
            .map(|f| f as Arc<dyn crate::sync::ConnectionFilter + 'static>),
    )
    .map_err(|e| format!("Sync error: {}", e))?;

    service.add_notify(chain_notify.clone());

    // Propagate transactions as soon as they are imported.
    let tx = ::parking_lot::Mutex::new(priority_tasks);
    let is_ready = Arc::new(atomic::AtomicBool::new(true));
    miner.add_transactions_listener(Box::new(move |_hashes| {
        // we want to have only one PendingTransactions task in the queue.
        if is_ready.compare_and_swap(true, false, atomic::Ordering::SeqCst) {
            let task =
                ::sync::PriorityTask::PropagateTransactions(Instant::now(), is_ready.clone());
            // we ignore error cause it means that we are closing
            let _ = tx.lock().send(task);
        }
    }));

    // start network
    if network_enabled {
        chain_notify.start();
    }

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
        experimental_rpcs: cmd.experimental_rpcs,
        ws_address: cmd.ws_conf.address(),
        fetch: fetch.clone(),
        executor: runtime.executor(),
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

    // start the prometheus metrics server
    start_prometheus_metrics(&cmd.metrics_conf, &dependencies)?;

    let http_server = rpc::new_http(
        "HTTP JSON-RPC",
        "jsonrpc",
        cmd.http_conf.clone(),
        &dependencies,
    )?;

    // secret store key server
    let secretstore_deps = secretstore::Dependencies {
        client: client.clone(),
        sync: sync_provider.clone(),
        miner: miner.clone(),
        account_provider,
        accounts_passwords: &passwords,
    };
    let secretstore_key_server = secretstore::start(
        cmd.secretstore_conf.clone(),
        secretstore_deps,
        runtime.executor(),
    )?;

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
    service
        .register_io_handler(informant.clone())
        .map_err(|_| "Unable to register informant handler".to_owned())?;

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
        let _ = user_defaults.save(&user_defaults_path); // discard failures - there's nothing we can do
    });

    // the watcher must be kept alive.
    let watcher = match cmd.snapshot_conf.enable {
        false => None,
        true => {
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
        }
    };

    Ok(RunningClient {
        inner: RunningClientInner::Full {
            rpc: rpc_direct,
            informant,
            client,
            client_service: Arc::new(service),
            keep_alive: Box::new((
                watcher,
                ws_server,
                http_server,
                ipc_server,
                secretstore_key_server,
                runtime,
            )),
        },
    })
}

/// Set bad blocks in VerificationQeueu. By omiting header we can omit particular fork of chain.
fn verification_bad_blocks(spec: &SpecType) -> Vec<H256> {
    match *spec {
        SpecType::Ropsten => {
            vec!["1eac3d16c642411f13c287e29144c6f58fda859407c8f24c38deb168e1040714".into()]
        }
        _ => vec![],
    }
}

/// Parity client currently executing in background threads.
///
/// Should be destroyed by calling `shutdown()`, otherwise execution will continue in the
/// background.
pub struct RunningClient {
    inner: RunningClientInner,
}

enum RunningClientInner {
    Full {
        rpc:
            jsonrpc_core::MetaIoHandler<Metadata, informant::Middleware<informant::ClientNotifier>>,
        informant: Arc<Informant<FullNodeInformantData>>,
        client: Arc<Client>,
        client_service: Arc<ClientService>,
        keep_alive: Box<dyn Any>,
    },
}

impl RunningClient {
    /// Performs an asynchronous RPC query.
    // FIXME: [tomaka] This API should be better, with for example a Future
    pub fn rpc_query(
        &self,
        request: &str,
        session: Option<Arc<PubSubSession>>,
    ) -> FutureResult<FutureResponse, FutureOutput> {
        let metadata = Metadata {
            origin: Origin::CApi,
            session,
        };

        match self.inner {
            RunningClientInner::Full { ref rpc, .. } => rpc.handle_request(request, metadata),
        }
    }

    /// Shuts down the client.
    pub fn shutdown(self) {
        match self.inner {
            RunningClientInner::Full {
                rpc,
                informant,
                client,
                client_service,
                keep_alive,
            } => {
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

fn print_running_environment(data_dir: &str, dirs: &Directories, db_dirs: &DatabaseDirectories) {
    info!("Starting {}", Colour::White.bold().paint(version()));
    info!(
        "Keys path {}",
        Colour::White
            .bold()
            .paint(dirs.keys_path(data_dir).to_string_lossy().into_owned())
    );
    info!(
        "DB path {}",
        Colour::White
            .bold()
            .paint(db_dirs.db_root_path().to_string_lossy().into_owned())
    );
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
