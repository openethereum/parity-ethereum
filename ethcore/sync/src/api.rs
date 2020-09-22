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

use bytes::Bytes;
use devp2p::NetworkService;
use network::{
    client_version::ClientVersion, ConnectionFilter, Error, ErrorKind,
    NetworkConfiguration as BasicNetworkConfiguration, NetworkContext, NetworkProtocolHandler,
    NonReservedPeerMode, PeerId, ProtocolId,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    io,
    ops::RangeInclusive,
    sync::{atomic, mpsc, Arc},
    time::Duration,
};

use chain::{
    fork_filter::ForkFilterApi, ChainSyncApi, SyncState, SyncStatus as EthSyncStatus,
    ETH_PROTOCOL_VERSION_63, ETH_PROTOCOL_VERSION_64, PAR_PROTOCOL_VERSION_1,
    PAR_PROTOCOL_VERSION_2,
};
use ethcore::{
    client::{BlockChainClient, ChainMessageType, ChainNotify, NewBlocks},
    snapshot::SnapshotService,
};
use ethereum_types::{H256, H512, U256};
use ethkey::Secret;
use io::TimerToken;
use network::IpFilter;
use parking_lot::{Mutex, RwLock};
use stats::{prometheus, prometheus_counter, prometheus_gauge, PrometheusMetrics};

use std::{
    net::{AddrParseError, SocketAddr},
    str::FromStr,
};
use sync_io::NetSyncIo;
use types::{
    creation_status::CreationStatus, restoration_status::RestorationStatus,
    transaction::UnverifiedTransaction, BlockNumber,
};

/// OpenEthereum sync protocol
pub const PAR_PROTOCOL: ProtocolId = *b"par";
/// Ethereum sync protocol
pub const ETH_PROTOCOL: ProtocolId = *b"eth";

/// Determine warp sync status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpSync {
    /// Warp sync is enabled.
    Enabled,
    /// Warp sync is disabled.
    Disabled,
    /// Only warp sync is allowed (no regular sync) and only after given block number.
    OnlyAndAfter(BlockNumber),
}

impl WarpSync {
    /// Returns true if warp sync is enabled.
    pub fn is_enabled(&self) -> bool {
        match *self {
            WarpSync::Enabled => true,
            WarpSync::OnlyAndAfter(_) => true,
            WarpSync::Disabled => false,
        }
    }

    /// Returns `true` if we are in warp-only mode.
    ///
    /// i.e. we will never fall back to regular sync
    /// until given block number is reached by
    /// successfuly finding and restoring from a snapshot.
    pub fn is_warp_only(&self) -> bool {
        if let WarpSync::OnlyAndAfter(_) = *self {
            true
        } else {
            false
        }
    }
}

/// Sync configuration
#[derive(Debug, Clone, Copy)]
pub struct SyncConfig {
    /// Max blocks to download ahead
    pub max_download_ahead_blocks: usize,
    /// Enable ancient block download.
    pub download_old_blocks: bool,
    /// Network ID
    pub network_id: u64,
    /// Main "eth" subprotocol name.
    pub subprotocol_name: [u8; 3],
    /// Fork block to check
    pub fork_block: Option<(BlockNumber, H256)>,
    /// Enable snapshot sync
    pub warp_sync: WarpSync,
}

impl Default for SyncConfig {
    fn default() -> SyncConfig {
        SyncConfig {
            max_download_ahead_blocks: 20000,
            download_old_blocks: true,
            network_id: 1,
            subprotocol_name: ETH_PROTOCOL,
            fork_block: None,
            warp_sync: WarpSync::Disabled,
        }
    }
}

/// Current sync status
pub trait SyncProvider: Send + Sync + PrometheusMetrics {
    /// Get sync status
    fn status(&self) -> EthSyncStatus;

    /// Get peers information
    fn peers(&self) -> Vec<PeerInfo>;

    /// Get the enode if available.
    fn enode(&self) -> Option<String>;

    /// Returns propagation count for pending transactions.
    fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats>;
}

/// Transaction stats
#[derive(Debug)]
pub struct TransactionStats {
    /// Block number where this TX was first seen.
    pub first_seen: u64,
    /// Peers it was propagated to.
    pub propagated_to: BTreeMap<H512, usize>,
}

/// Peer connection information
#[derive(Debug)]
pub struct PeerInfo {
    /// Public node id
    pub id: Option<String>,
    /// Node client ID
    pub client_version: ClientVersion,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Remote endpoint address
    pub remote_address: String,
    /// Local endpoint address
    pub local_address: String,
    /// Eth protocol info.
    pub eth_info: Option<EthProtocolInfo>,
}

/// Ethereum protocol info.
#[derive(Debug)]
pub struct EthProtocolInfo {
    /// Protocol version
    pub version: u32,
    /// SHA3 of peer best block hash
    pub head: H256,
    /// Peer total difficulty if known
    pub difficulty: Option<U256>,
}

/// A prioritized tasks run in a specialised timer.
/// Every task should be completed within a hard deadline,
/// if it's not it's either cancelled or split into multiple tasks.
/// NOTE These tasks might not complete at all, so anything
/// that happens here should work even if the task is cancelled.
#[derive(Debug)]
pub enum PriorityTask {
    /// Propagate given block
    PropagateBlock {
        /// When the task was initiated
        started: ::std::time::Instant,
        /// Raw block RLP to propagate
        block: Bytes,
        /// Block hash
        hash: H256,
        /// Blocks difficulty
        difficulty: U256,
    },
    /// Propagate a list of transactions
    PropagateTransactions(::std::time::Instant, Arc<atomic::AtomicBool>),
}
impl PriorityTask {
    /// Mark the task as being processed, right after it's retrieved from the queue.
    pub fn starting(&self) {
        match *self {
            PriorityTask::PropagateTransactions(_, ref is_ready) => {
                is_ready.store(true, atomic::Ordering::SeqCst)
            }
            _ => {}
        }
    }
}

/// EthSync initialization parameters.
pub struct Params {
    /// Configuration.
    pub config: SyncConfig,
    /// Blockchain client.
    pub chain: Arc<dyn BlockChainClient>,
    /// Forks.
    pub forks: BTreeSet<BlockNumber>,
    /// Snapshot service.
    pub snapshot_service: Arc<dyn SnapshotService>,
    /// Network layer configuration.
    pub network_config: NetworkConfiguration,
}

/// Ethereum network protocol handler
pub struct EthSync {
    /// Network service
    network: NetworkService,
    /// Main (eth/par) protocol handler
    eth_handler: Arc<SyncProtocolHandler>,
    /// The main subprotocol name
    subprotocol_name: [u8; 3],
    /// Priority tasks notification channel
    priority_tasks: Mutex<mpsc::Sender<PriorityTask>>,
}

impl EthSync {
    /// Creates and register protocol with the network service
    pub fn new(
        params: Params,
        connection_filter: Option<Arc<dyn ConnectionFilter>>,
    ) -> Result<Arc<EthSync>, Error> {
        let (priority_tasks_tx, priority_tasks_rx) = mpsc::channel();
        let fork_filter = ForkFilterApi::new(&*params.chain, params.forks);

        let sync = ChainSyncApi::new(
            params.config,
            &*params.chain,
            fork_filter,
            priority_tasks_rx,
        );
        let service = NetworkService::new(
            params.network_config.clone().into_basic()?,
            connection_filter,
        )?;

        let sync = Arc::new(EthSync {
            network: service,
            eth_handler: Arc::new(SyncProtocolHandler {
                sync,
                chain: params.chain,
                snapshot_service: params.snapshot_service,
                overlay: RwLock::new(HashMap::new()),
            }),
            subprotocol_name: params.config.subprotocol_name,
            priority_tasks: Mutex::new(priority_tasks_tx),
        });

        Ok(sync)
    }

    /// Priority tasks producer
    pub fn priority_tasks(&self) -> mpsc::Sender<PriorityTask> {
        self.priority_tasks.lock().clone()
    }
}

impl SyncProvider for EthSync {
    /// Get sync status
    fn status(&self) -> EthSyncStatus {
        self.eth_handler.sync.status()
    }

    /// Get sync peers
    fn peers(&self) -> Vec<PeerInfo> {
        self.network
            .with_context_eval(self.subprotocol_name, |ctx| {
                let peer_ids = self.network.connected_peers();

                let peer_info = self.eth_handler.sync.peer_info(&peer_ids);
                peer_ids
                    .into_iter()
                    .zip(peer_info)
                    .filter_map(|(peer_id, peer_info)| {
                        let session_info = match ctx.session_info(peer_id) {
                            None => return None,
                            Some(info) => info,
                        };

                        Some(PeerInfo {
                            id: session_info.id.map(|id| format!("{:x}", id)),
                            client_version: session_info.client_version,
                            capabilities: session_info
                                .peer_capabilities
                                .into_iter()
                                .map(|c| c.to_string())
                                .collect(),
                            remote_address: session_info.remote_address,
                            local_address: session_info.local_address,
                            eth_info: peer_info,
                        })
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    fn enode(&self) -> Option<String> {
        self.network.external_url()
    }

    fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats> {
        self.eth_handler.sync.transactions_stats()
    }
}

impl PrometheusMetrics for EthSync {
    fn prometheus_metrics(&self, r: &mut prometheus::Registry) {
        let scalar = |b| if b { 1i64 } else { 0i64 };
        let sync_status = self.status();

        prometheus_gauge(r,
			"sync_status",
			"WaitingPeers(0), SnapshotManifest(1), SnapshotData(2), SnapshotWaiting(3), Blocks(4), Idle(5), Waiting(6), NewBlocks(7)", 
			match self.eth_handler.sync.status().state {
			SyncState::WaitingPeers => 0,
			SyncState::SnapshotManifest => 1,
			SyncState::SnapshotData => 2,
			SyncState::SnapshotWaiting => 3,
			SyncState::Blocks => 4,
			SyncState::Idle => 5,
			SyncState::Waiting => 6,
			SyncState::NewBlocks => 7,
        });

        for (key, value) in sync_status.item_sizes.iter() {
            prometheus_gauge(
                r,
                &key,
                format!("Total item number of {}", key).as_str(),
                *value as i64,
            );
        }

        prometheus_gauge(
            r,
            "net_peers",
            "Total number of connected peers",
            sync_status.num_peers as i64,
        );
        prometheus_gauge(
            r,
            "net_active_peers",
            "Total number of active peers",
            sync_status.num_active_peers as i64,
        );
        prometheus_counter(
            r,
            "sync_blocks_recieved",
            "Number of blocks downloaded so far",
            sync_status.blocks_received as i64,
        );
        prometheus_counter(
            r,
            "sync_blocks_total",
            "Total number of blocks for the sync process",
            sync_status.blocks_total as i64,
        );
        prometheus_gauge(
            r,
            "sync_blocks_highest",
            "Highest block number in the download queue",
            sync_status.highest_block_number.unwrap_or(0) as i64,
        );

        prometheus_gauge(
            r,
            "snapshot_download_active",
            "1 if downloading snapshots",
            scalar(sync_status.is_snapshot_syncing()),
        );
        prometheus_gauge(
            r,
            "snapshot_download_chunks",
            "Snapshot chunks",
            sync_status.num_snapshot_chunks as i64,
        );
        prometheus_gauge(
            r,
            "snapshot_download_chunks_done",
            "Snapshot chunks downloaded",
            sync_status.snapshot_chunks_done as i64,
        );

        let restoration = self.eth_handler.snapshot_service.restoration_status();
        let creation = self.eth_handler.snapshot_service.creation_status();

        prometheus_gauge(
            r,
            "snapshot_create_block",
            "First block of the current snapshot creation",
            if let CreationStatus::Ongoing { block_number } = creation {
                block_number as i64
            } else {
                0
            },
        );
        prometheus_gauge(
            r,
            "snapshot_restore_block",
            "First block of the current snapshot restoration",
            if let RestorationStatus::Ongoing { block_number, .. } = restoration {
                block_number as i64
            } else {
                0
            },
        );
    }
}

const PEERS_TIMER: TimerToken = 0;
const MAINTAIN_SYNC_TIMER: TimerToken = 1;
const CONTINUE_SYNC_TIMER: TimerToken = 2;
const TX_TIMER: TimerToken = 3;
const PRIORITY_TIMER: TimerToken = 4;
const DELAYED_PROCESSING_TIMER: TimerToken = 5;

pub(crate) const PRIORITY_TIMER_INTERVAL: Duration = Duration::from_millis(250);

struct SyncProtocolHandler {
    /// Shared blockchain client.
    chain: Arc<dyn BlockChainClient>,
    /// Shared snapshot service.
    snapshot_service: Arc<dyn SnapshotService>,
    /// Sync strategy
    sync: ChainSyncApi,
    /// Chain overlay used to cache data such as fork block.
    overlay: RwLock<HashMap<BlockNumber, Bytes>>,
}

impl NetworkProtocolHandler for SyncProtocolHandler {
    fn initialize(&self, io: &dyn NetworkContext) {
        if io.subprotocol_name() != PAR_PROTOCOL {
            io.register_timer(PEERS_TIMER, Duration::from_millis(700))
                .expect("Error registering peers timer");
            io.register_timer(MAINTAIN_SYNC_TIMER, Duration::from_millis(1100))
                .expect("Error registering sync timer");
            io.register_timer(CONTINUE_SYNC_TIMER, Duration::from_millis(2500))
                .expect("Error registering sync timer");
            io.register_timer(TX_TIMER, Duration::from_millis(1300))
                .expect("Error registering transactions timer");
            io.register_timer(DELAYED_PROCESSING_TIMER, Duration::from_millis(2100))
                .expect("Error registering delayed processing timer");

            io.register_timer(PRIORITY_TIMER, PRIORITY_TIMER_INTERVAL)
                .expect("Error registering peers timer");
        }
    }

    fn read(&self, io: &dyn NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
        self.sync.dispatch_packet(
            &mut NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay),
            *peer,
            packet_id,
            data,
        );
    }

    fn connected(&self, io: &dyn NetworkContext, peer: &PeerId) {
        trace_time!("sync::connected");
        // If warp protocol is supported only allow warp handshake
        let warp_protocol = io.protocol_version(PAR_PROTOCOL, *peer).unwrap_or(0) != 0;
        let warp_context = io.subprotocol_name() == PAR_PROTOCOL;
        if warp_protocol == warp_context {
            self.sync.write().on_peer_connected(
                &mut NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay),
                *peer,
            );
        }
    }

    fn disconnected(&self, io: &dyn NetworkContext, peer: &PeerId) {
        trace_time!("sync::disconnected");
        if io.subprotocol_name() != PAR_PROTOCOL {
            self.sync.write().on_peer_aborting(
                &mut NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay),
                *peer,
            );
        }
    }

    fn timeout(&self, io: &dyn NetworkContext, timer: TimerToken) {
        trace_time!("sync::timeout");
        let mut io = NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay);
        match timer {
            PEERS_TIMER => self.sync.write().maintain_peers(&mut io),
            MAINTAIN_SYNC_TIMER => self.sync.write().maintain_sync(&mut io),
            CONTINUE_SYNC_TIMER => self.sync.write().continue_sync(&mut io),
            TX_TIMER => self.sync.write().propagate_new_transactions(&mut io),
            PRIORITY_TIMER => self.sync.process_priority_queue(&mut io),
            DELAYED_PROCESSING_TIMER => self.sync.process_delayed_requests(&mut io),
            _ => warn!("Unknown timer {} triggered.", timer),
        }
    }
}

impl ChainNotify for EthSync {
    fn block_pre_import(&self, bytes: &Bytes, hash: &H256, difficulty: &U256) {
        let task = PriorityTask::PropagateBlock {
            started: ::std::time::Instant::now(),
            block: bytes.clone(),
            hash: *hash,
            difficulty: *difficulty,
        };
        if let Err(e) = self.priority_tasks.lock().send(task) {
            warn!(target: "sync", "Unexpected error during priority block propagation: {:?}", e);
        }
    }

    fn new_blocks(&self, new_blocks: NewBlocks) {
        if new_blocks.has_more_blocks_to_import {
            return;
        }
        self.network.with_context(self.subprotocol_name, |context| {
            let mut sync_io = NetSyncIo::new(
                context,
                &*self.eth_handler.chain,
                &*self.eth_handler.snapshot_service,
                &self.eth_handler.overlay,
            );
            self.eth_handler.sync.write().chain_new_blocks(
                &mut sync_io,
                &new_blocks.imported,
                &new_blocks.invalid,
                new_blocks.route.enacted(),
                new_blocks.route.retracted(),
                &new_blocks.sealed,
                &new_blocks.proposed,
            );
        });
    }

    fn start(&self) {
        match self.network.start() {
			Err((err, listen_address)) => {
				match err.into() {
					ErrorKind::Io(ref e) if e.kind() == io::ErrorKind::AddrInUse => {
						warn!("Network port {:?} is already in use, make sure that another instance of an Ethereum client is not running or change the port using the --port option.", listen_address.expect("Listen address is not set."))
					},
					err => warn!("Error starting network: {}", err),
				}
			},
			_ => {},
		}

        self.network
            .register_protocol(
                self.eth_handler.clone(),
                self.subprotocol_name,
                &[ETH_PROTOCOL_VERSION_63, ETH_PROTOCOL_VERSION_64],
            )
            .unwrap_or_else(|e| warn!("Error registering ethereum protocol: {:?}", e));
        // register the warp sync subprotocol
        self.network
            .register_protocol(
                self.eth_handler.clone(),
                PAR_PROTOCOL,
                &[PAR_PROTOCOL_VERSION_1, PAR_PROTOCOL_VERSION_2],
            )
            .unwrap_or_else(|e| warn!("Error registering snapshot sync protocol: {:?}", e));
    }

    fn stop(&self) {
        self.eth_handler.snapshot_service.abort_restore();
        self.network.stop();
    }

    fn broadcast(&self, message_type: ChainMessageType) {
        self.network.with_context(PAR_PROTOCOL, |context| {
            let mut sync_io = NetSyncIo::new(
                context,
                &*self.eth_handler.chain,
                &*self.eth_handler.snapshot_service,
                &self.eth_handler.overlay,
            );
            match message_type {
                ChainMessageType::Consensus(message) => self
                    .eth_handler
                    .sync
                    .write()
                    .propagate_consensus_packet(&mut sync_io, message),
            }
        });
    }

    fn transactions_received(&self, txs: &[UnverifiedTransaction], peer_id: PeerId) {
        let mut sync = self.eth_handler.sync.write();
        sync.transactions_received(txs, peer_id);
    }
}

/// Trait for managing network
pub trait ManageNetwork: Send + Sync {
    /// Set to allow unreserved peers to connect
    fn accept_unreserved_peers(&self);
    /// Set to deny unreserved peers to connect
    fn deny_unreserved_peers(&self);
    /// Remove reservation for the peer
    fn remove_reserved_peer(&self, peer: String) -> Result<(), String>;
    /// Add reserved peer
    fn add_reserved_peer(&self, peer: String) -> Result<(), String>;
    /// Start network
    fn start_network(&self);
    /// Stop network
    fn stop_network(&self);
    /// Returns the minimum and maximum peers.
    fn num_peers_range(&self) -> RangeInclusive<u32>;
    /// Get network context for protocol.
    fn with_proto_context(&self, proto: ProtocolId, f: &mut dyn FnMut(&dyn NetworkContext));
}

impl ManageNetwork for EthSync {
    fn accept_unreserved_peers(&self) {
        self.network
            .set_non_reserved_mode(NonReservedPeerMode::Accept);
    }

    fn deny_unreserved_peers(&self) {
        self.network
            .set_non_reserved_mode(NonReservedPeerMode::Deny);
    }

    fn remove_reserved_peer(&self, peer: String) -> Result<(), String> {
        self.network
            .remove_reserved_peer(&peer)
            .map_err(|e| format!("{:?}", e))
    }

    fn add_reserved_peer(&self, peer: String) -> Result<(), String> {
        self.network
            .add_reserved_peer(&peer)
            .map_err(|e| format!("{:?}", e))
    }

    fn start_network(&self) {
        self.start();
    }

    fn stop_network(&self) {
        self.network.with_context(self.subprotocol_name, |context| {
            let mut sync_io = NetSyncIo::new(
                context,
                &*self.eth_handler.chain,
                &*self.eth_handler.snapshot_service,
                &self.eth_handler.overlay,
            );
            self.eth_handler.sync.write().abort(&mut sync_io);
        });

        self.stop();
    }

    fn num_peers_range(&self) -> RangeInclusive<u32> {
        self.network.num_peers_range()
    }

    fn with_proto_context(&self, proto: ProtocolId, f: &mut dyn FnMut(&dyn NetworkContext)) {
        self.network.with_context_eval(proto, f);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Network service configuration
pub struct NetworkConfiguration {
    /// Directory path to store general network configuration. None means nothing will be saved
    pub config_path: Option<String>,
    /// Directory path to store network-specific configuration. None means nothing will be saved
    pub net_config_path: Option<String>,
    /// IP address to listen for incoming connections. Listen to all connections by default
    pub listen_address: Option<String>,
    /// IP address to advertise. Detected automatically if none.
    pub public_address: Option<String>,
    /// Port for UDP connections, same as TCP by default
    pub udp_port: Option<u16>,
    /// Enable NAT configuration
    pub nat_enabled: bool,
    /// Enable discovery
    pub discovery_enabled: bool,
    /// List of initial node addresses
    pub boot_nodes: Vec<String>,
    /// Use provided node key instead of default
    pub use_secret: Option<Secret>,
    /// Max number of connected peers to maintain
    pub max_peers: u32,
    /// Min number of connected peers to maintain
    pub min_peers: u32,
    /// Max pending peers.
    pub max_pending_peers: u32,
    /// Reserved snapshot sync peers.
    pub snapshot_peers: u32,
    /// List of reserved node addresses.
    pub reserved_nodes: Vec<String>,
    /// The non-reserved peer mode.
    pub allow_non_reserved: bool,
    /// IP Filtering
    pub ip_filter: IpFilter,
    /// Client version string
    pub client_version: String,
}

impl NetworkConfiguration {
    /// Create a new default config.
    pub fn new() -> Self {
        From::from(BasicNetworkConfiguration::new())
    }

    /// Create a new local config.
    pub fn new_local() -> Self {
        From::from(BasicNetworkConfiguration::new_local())
    }

    /// Attempt to convert this config into a BasicNetworkConfiguration.
    pub fn into_basic(self) -> Result<BasicNetworkConfiguration, AddrParseError> {
        Ok(BasicNetworkConfiguration {
            config_path: self.config_path,
            net_config_path: self.net_config_path,
            listen_address: match self.listen_address {
                None => None,
                Some(addr) => Some(SocketAddr::from_str(&addr)?),
            },
            public_address: match self.public_address {
                None => None,
                Some(addr) => Some(SocketAddr::from_str(&addr)?),
            },
            udp_port: self.udp_port,
            nat_enabled: self.nat_enabled,
            discovery_enabled: self.discovery_enabled,
            boot_nodes: self.boot_nodes,
            use_secret: self.use_secret,
            max_peers: self.max_peers,
            min_peers: self.min_peers,
            max_handshakes: self.max_pending_peers,
            reserved_protocols: hash_map![PAR_PROTOCOL => self.snapshot_peers],
            reserved_nodes: self.reserved_nodes,
            ip_filter: self.ip_filter,
            non_reserved_mode: if self.allow_non_reserved {
                NonReservedPeerMode::Accept
            } else {
                NonReservedPeerMode::Deny
            },
            client_version: self.client_version,
        })
    }
}

impl From<BasicNetworkConfiguration> for NetworkConfiguration {
    fn from(other: BasicNetworkConfiguration) -> Self {
        NetworkConfiguration {
            config_path: other.config_path,
            net_config_path: other.net_config_path,
            listen_address: other
                .listen_address
                .and_then(|addr| Some(format!("{}", addr))),
            public_address: other
                .public_address
                .and_then(|addr| Some(format!("{}", addr))),
            udp_port: other.udp_port,
            nat_enabled: other.nat_enabled,
            discovery_enabled: other.discovery_enabled,
            boot_nodes: other.boot_nodes,
            use_secret: other.use_secret,
            max_peers: other.max_peers,
            min_peers: other.min_peers,
            max_pending_peers: other.max_handshakes,
            snapshot_peers: *other.reserved_protocols.get(&PAR_PROTOCOL).unwrap_or(&0),
            reserved_nodes: other.reserved_nodes,
            ip_filter: other.ip_filter,
            allow_non_reserved: match other.non_reserved_mode {
                NonReservedPeerMode::Accept => true,
                _ => false,
            },
            client_version: other.client_version,
        }
    }
}

/// Configuration for IPC service.
#[derive(Debug, Clone)]
pub struct ServiceConfiguration {
    /// Sync config.
    pub sync: SyncConfig,
    /// Network configuration.
    pub net: NetworkConfiguration,
    /// IPC path.
    pub io_path: String,
}

/// Numbers of peers (max, min, active).
#[derive(Debug, Clone)]
pub struct PeerNumbers {
    /// Number of connected peers.
    pub connected: usize,
    /// Number of active peers.
    pub active: usize,
    /// Max peers.
    pub max: usize,
    /// Min peers.
    pub min: usize,
}
