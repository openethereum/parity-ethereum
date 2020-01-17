// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

use std::sync::{Arc, mpsc, atomic};
use std::collections::{HashMap, BTreeMap};
use std::io;
use std::ops::RangeInclusive;
use std::time::Duration;
use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::sync_io::NetSyncIo;
use crate::light_sync::{self, SyncInfo};
use crate::private_tx::PrivateTxHandler;
use crate::chain::{
	sync_packet::SyncPacket::{PrivateTransactionPacket, SignedPrivateTransactionPacket},
	ChainSyncApi, SyncState, SyncStatus as EthSyncStatus, ETH_PROTOCOL_VERSION_62,
	ETH_PROTOCOL_VERSION_63, PAR_PROTOCOL_VERSION_1, PAR_PROTOCOL_VERSION_2,
	PAR_PROTOCOL_VERSION_3, PAR_PROTOCOL_VERSION_4,
};

use bytes::Bytes;
use client_traits::{BlockChainClient, ChainNotify};
use devp2p::NetworkService;
use ethcore_io::TimerToken;
use ethcore_private_tx::PrivateStateDB;
use ethereum_types::{H256, H512, U256};
use parity_crypto::publickey::Secret;
use futures::sync::mpsc as futures_mpsc;
use futures::Stream;
use light::client::AsLightClient;
use light::Provider;
use light::net::{
	self as light_net, LightProtocol, Params as LightParams,
	Capabilities, Handler as LightHandler, EventContext, SampleStore,
};
use log::{trace, warn};
use macros::hash_map;
use network::{
	client_version::ClientVersion,
	NetworkProtocolHandler, NetworkContext, PeerId, ProtocolId,
	NetworkConfiguration as BasicNetworkConfiguration, NonReservedPeerMode, Error,
	ConnectionFilter, IpFilter, NatType
};
use snapshot::SnapshotService;
use parking_lot::{RwLock, Mutex};
use parity_runtime::Executor;
use trace_time::trace_time;
use common_types::{
	BlockNumber,
	chain_notify::{NewBlocks, ChainMessageType},
	pruning_info::PruningInfo,
	transaction::UnverifiedTransaction,
};


/// Parity sync protocol
pub const WARP_SYNC_PROTOCOL_ID: ProtocolId = *b"par";
/// Ethereum sync protocol
pub const ETH_PROTOCOL: ProtocolId = *b"eth";
/// Ethereum light protocol
pub const LIGHT_PROTOCOL: ProtocolId = *b"pip";

/// Determine warp sync status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, MallocSizeOf)]
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
	/// Light subprotocol name.
	pub light_subprotocol_name: [u8; 3],
	/// Fork block to check
	pub fork_block: Option<(BlockNumber, H256)>,
	/// Enable snapshot sync
	pub warp_sync: WarpSync,
	/// Enable light client server.
	pub serve_light: bool,
}

impl Default for SyncConfig {
	fn default() -> SyncConfig {
		SyncConfig {
			max_download_ahead_blocks: 20000,
			download_old_blocks: true,
			network_id: 1,
			subprotocol_name: ETH_PROTOCOL,
			light_subprotocol_name: LIGHT_PROTOCOL,
			fork_block: None,
			warp_sync: WarpSync::Disabled,
			serve_light: false,
		}
	}
}

/// receiving end of a futures::mpsc channel
pub type Notification<T> = futures_mpsc::UnboundedReceiver<T>;

/// Current sync status
pub trait SyncProvider: Send + Sync {
	/// Get sync status
	fn status(&self) -> EthSyncStatus;

	/// Get peers information
	fn peers(&self) -> Vec<PeerInfo>;

	/// Get the enode if available.
	fn enode(&self) -> Option<String>;

	/// gets sync status notifications
	fn sync_notification(&self) -> Notification<SyncState>;

	/// Returns propagation count for pending transactions.
	fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats>;

	/// are we in the middle of a major sync?
	fn is_major_syncing(&self) -> bool;
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
	/// Light protocol info.
	pub pip_info: Option<PipProtocolInfo>,
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

/// PIP protocol info.
#[derive(Debug)]
pub struct PipProtocolInfo {
	/// Protocol version
	pub version: u32,
	/// SHA3 of peer best block hash
	pub head: H256,
	/// Peer total difficulty if known
	pub difficulty: U256,
}

impl From<light_net::Status> for PipProtocolInfo {
	fn from(status: light_net::Status) -> Self {
		PipProtocolInfo {
			version: status.protocol_version,
			head: status.head_hash,
			difficulty: status.head_td,
		}
	}
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
			PriorityTask::PropagateTransactions(_, ref is_ready) => is_ready.store(true, atomic::Ordering::SeqCst),
			_ => {},
		}
	}
}

/// EthSync initialization parameters.
pub struct Params {
	/// Configuration.
	pub config: SyncConfig,
	/// Runtime executor
	pub executor: Executor,
	/// Blockchain client.
	pub chain: Arc<dyn BlockChainClient>,
	/// Snapshot service.
	pub snapshot_service: Arc<dyn SnapshotService>,
	/// Private tx service.
	pub private_tx_handler: Option<Arc<dyn PrivateTxHandler>>,
	/// Private state wrapper
	pub private_state: Option<Arc<PrivateStateDB>>,
	/// Light data provider.
	pub provider: Arc<dyn (::light::Provider)>,
	/// Network layer configuration.
	pub network_config: NetworkConfiguration,
}

/// Ethereum network protocol handler
pub struct EthSync {
	/// Network service
	network: NetworkService,
	/// Main (eth/par) protocol handler
	eth_handler: Arc<SyncProtocolHandler>,
	/// Light (pip) protocol handler
	light_proto: Option<Arc<LightProtocol>>,
	/// The main subprotocol name
	subprotocol_name: [u8; 3],
	/// Light subprotocol name.
	light_subprotocol_name: [u8; 3],
	/// Priority tasks notification channel
	priority_tasks: Mutex<mpsc::Sender<PriorityTask>>,
	/// Track the sync state: are we importing or verifying blocks?
	is_major_syncing: Arc<AtomicBool>
}

fn light_params(
	network_id: u64,
	median_peers: f64,
	pruning_info: PruningInfo,
	sample_store: Option<Box<dyn SampleStore>>,
) -> LightParams {
	let mut light_params = LightParams {
		network_id: network_id,
		config: Default::default(),
		capabilities: Capabilities {
			serve_headers: true,
			serve_chain_since: Some(pruning_info.earliest_chain),
			serve_state_since: Some(pruning_info.earliest_state),
			tx_relay: true,
		},
		sample_store: sample_store,
	};

	light_params.config.median_peers = median_peers;
	light_params
}

impl EthSync {
	/// Creates and register protocol with the network service
	pub fn new(params: Params, connection_filter: Option<Arc<dyn ConnectionFilter>>) -> Result<Arc<EthSync>, Error> {
		let pruning_info = params.chain.pruning_info();
		let light_proto = match params.config.serve_light {
			false => None,
			true => Some({
				let sample_store = params.network_config.net_config_path
					.clone()
					.map(::std::path::PathBuf::from)
					.map(|mut p| { p.push("request_timings"); light_net::FileStore(p) })
					.map(|store| Box::new(store) as Box<_>);

				let median_peers = (params.network_config.min_peers + params.network_config.max_peers) as f64 / 2.0;
				let light_params = light_params(
					params.config.network_id,
					median_peers,
					pruning_info,
					sample_store,
				);

				let mut light_proto = LightProtocol::new(params.provider, light_params);
				light_proto.add_handler(Arc::new(TxRelay(params.chain.clone())));

				Arc::new(light_proto)
			})
		};

		let (priority_tasks_tx, priority_tasks_rx) = mpsc::channel();
		let sync = ChainSyncApi::new(
			params.config,
			&*params.chain,
			params.private_tx_handler.as_ref().cloned(),
			priority_tasks_rx,
		);

		let is_major_syncing = Arc::new(AtomicBool::new(false));

		{
			// spawn task that constantly updates EthSync.is_major_sync
			let notifications = sync.write().sync_notifications();
			let moved_client = Arc::downgrade(&params.chain);
			let moved_is_major_syncing = is_major_syncing.clone();

			params.executor.spawn(notifications.for_each(move |sync_status| {
				if let Some(queue_info) = moved_client.upgrade().map(|client| client.queue_info()) {
					let is_syncing_state = match sync_status {
						SyncState::Idle | SyncState::NewBlocks => false,
						_ => true
					};
					let is_verifying = queue_info.unverified_queue_size + queue_info.verified_queue_size > 3;
					moved_is_major_syncing.store(is_verifying || is_syncing_state, Ordering::SeqCst);
					return Ok(())
				}

				// client has been dropped
				return Err(())
			}));
		}
		let service = NetworkService::new(params.network_config.clone().into_basic()?, connection_filter)?;

		let sync = Arc::new(EthSync {
			network: service,
			eth_handler: Arc::new(SyncProtocolHandler {
				sync,
				chain: params.chain,
				snapshot_service: params.snapshot_service,
				overlay: RwLock::new(HashMap::new()),
				private_state: params.private_state,
			}),
			light_proto: light_proto,
			subprotocol_name: params.config.subprotocol_name,
			light_subprotocol_name: params.config.light_subprotocol_name,
			priority_tasks: Mutex::new(priority_tasks_tx),
			is_major_syncing
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
		self.network.with_context_eval(self.subprotocol_name, |ctx| {
			let peer_ids = self.network.connected_peers();
			let light_proto = self.light_proto.as_ref();

			let peer_info = self.eth_handler.sync.peer_info(&peer_ids);
			peer_ids.into_iter().zip(peer_info).filter_map(|(peer_id, peer_info)| {
				let session_info = match ctx.session_info(peer_id) {
					None => return None,
					Some(info) => info,
				};

				Some(PeerInfo {
					id: session_info.id.map(|id| format!("{:x}", id)),
					client_version: session_info.client_version,
					capabilities: session_info.peer_capabilities.into_iter().map(|c| c.to_string()).collect(),
					remote_address: session_info.remote_address,
					local_address: session_info.local_address,
					eth_info: peer_info,
					pip_info: light_proto.as_ref().and_then(|lp| lp.peer_status(peer_id)).map(Into::into),
				})
			}).collect()
		}).unwrap_or_else(Vec::new)
	}

	fn enode(&self) -> Option<String> {
		self.network.external_url()
	}

	fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats> {
		self.eth_handler.sync.transactions_stats()
	}

	fn sync_notification(&self) -> Notification<SyncState> {
		self.eth_handler.sync.write().sync_notifications()
	}

	fn is_major_syncing(&self) -> bool {
		self.is_major_syncing.load(Ordering::SeqCst)
	}
}

const PEERS_TIMER: TimerToken = 0;
const MAINTAIN_SYNC_TIMER: TimerToken = 1;
const CONTINUE_SYNC_TIMER: TimerToken = 2;
const TX_TIMER: TimerToken = 3;
const PRIORITY_TIMER: TimerToken = 4;

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
	/// Private state db
	private_state: Option<Arc<PrivateStateDB>>,
}

impl NetworkProtocolHandler for SyncProtocolHandler {
	fn initialize(&self, io: &dyn NetworkContext) {
		if io.subprotocol_name() != WARP_SYNC_PROTOCOL_ID {
			io.register_timer(PEERS_TIMER, Duration::from_millis(700)).expect("Error registering peers timer");
			io.register_timer(MAINTAIN_SYNC_TIMER, Duration::from_millis(1100)).expect("Error registering sync timer");
			io.register_timer(CONTINUE_SYNC_TIMER, Duration::from_millis(2500)).expect("Error registering sync timer");
			io.register_timer(TX_TIMER, Duration::from_millis(1300)).expect("Error registering transactions timer");

			io.register_timer(PRIORITY_TIMER, PRIORITY_TIMER_INTERVAL).expect("Error registering peers timer");
		}
	}

	fn read(&self, io: &dyn NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		self.sync.dispatch_packet(&mut NetSyncIo::new(io,
			&*self.chain,
			&*self.snapshot_service,
			&self.overlay,
			self.private_state.clone()),
			*peer, packet_id, data);
	}

	fn connected(&self, io: &dyn NetworkContext, peer: &PeerId) {
		trace_time!("sync::connected");
		// If warp protocol is supported only allow warp handshake
		let warp_protocol = io.protocol_version(WARP_SYNC_PROTOCOL_ID, *peer).unwrap_or(0) != 0;
		let warp_context = io.subprotocol_name() == WARP_SYNC_PROTOCOL_ID;
		if warp_protocol == warp_context {
			self.sync.write().on_peer_connected(&mut NetSyncIo::new(io,
			&*self.chain,
			&*self.snapshot_service,
			&self.overlay,
			self.private_state.clone()),
			*peer);
		}
	}

	fn disconnected(&self, io: &dyn NetworkContext, peer: &PeerId) {
		trace_time!("sync::disconnected");
		if io.subprotocol_name() != WARP_SYNC_PROTOCOL_ID {
			self.sync.write().on_peer_aborting(&mut NetSyncIo::new(io,
				&*self.chain,
				&*self.snapshot_service,
				&self.overlay,
				self.private_state.clone()),
				*peer);
		}
	}

	fn timeout(&self, io: &dyn NetworkContext, timer: TimerToken) {
		trace_time!("sync::timeout");
		let mut io = NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay, self.private_state.clone());
		match timer {
			PEERS_TIMER => self.sync.write().maintain_peers(&mut io),
			MAINTAIN_SYNC_TIMER => self.sync.write().maintain_sync(&mut io),
			CONTINUE_SYNC_TIMER => self.sync.write().continue_sync(&mut io),
			TX_TIMER => self.sync.write().propagate_new_transactions(&mut io),
			PRIORITY_TIMER => self.sync.process_priority_queue(&mut io),
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

	fn new_blocks(&self, new_blocks: NewBlocks)
	{
		if new_blocks.has_more_blocks_to_import { return }
		use light::net::Announcement;

		self.network.with_context(self.subprotocol_name, |context| {
			let mut sync_io = NetSyncIo::new(context,
				&*self.eth_handler.chain,
				&*self.eth_handler.snapshot_service,
				&self.eth_handler.overlay,
				self.eth_handler.private_state.clone());
			self.eth_handler.sync.write().chain_new_blocks(
				&mut sync_io,
				&new_blocks.imported,
				&new_blocks.invalid,
				new_blocks.route.enacted(),
				new_blocks.route.retracted(),
				&new_blocks.sealed,
				&new_blocks.proposed);
		});

		self.network.with_context(self.light_subprotocol_name, |context| {
			let light_proto = match self.light_proto.as_ref() {
				Some(lp) => lp,
				None => return,
			};

			let chain_info = self.eth_handler.chain.chain_info();
			light_proto.make_announcement(&context, Announcement {
				head_hash: chain_info.best_block_hash,
				head_num: chain_info.best_block_number,
				head_td: chain_info.total_difficulty,
				reorg_depth: 0, // recalculated on a per-peer basis.
				serve_headers: false, // these fields consist of _changes_ in capability.
				serve_state_since: None,
				serve_chain_since: None,
				tx_relay: false,
			})
		})
	}

	fn start(&self) {
		match self.network.start() {
			Err((err, listen_address)) => {
				match err.into() {
					Error::Io(ref e) if e.kind() == io::ErrorKind::AddrInUse => {
						warn!("Network port {:?} is already in use, make sure that another instance of an Ethereum client is not running or change the port using the --port option.", listen_address.expect("Listen address is not set."))
					},
					err => warn!("Error starting network: {}", err),
				}
			},
			_ => {},
		}

		self.network.register_protocol(self.eth_handler.clone(), self.subprotocol_name, &[ETH_PROTOCOL_VERSION_62, ETH_PROTOCOL_VERSION_63])
			.unwrap_or_else(|e| warn!("Error registering ethereum protocol: {:?}", e));
		// register the warp sync subprotocol
		self.network.register_protocol(self.eth_handler.clone(), WARP_SYNC_PROTOCOL_ID, &[PAR_PROTOCOL_VERSION_1, PAR_PROTOCOL_VERSION_2, PAR_PROTOCOL_VERSION_3, PAR_PROTOCOL_VERSION_4])
			.unwrap_or_else(|e| warn!("Error registering snapshot sync protocol: {:?}", e));

		// register the light protocol.
		if let Some(light_proto) = self.light_proto.as_ref().map(|x| x.clone()) {
			self.network.register_protocol(light_proto, self.light_subprotocol_name, ::light::net::PROTOCOL_VERSIONS)
				.unwrap_or_else(|e| warn!("Error registering light client protocol: {:?}", e));
		}
	}

	fn stop(&self) {
		self.eth_handler.snapshot_service.abort_restore();
		self.network.stop();
	}

	fn broadcast(&self, message_type: ChainMessageType) {
		self.network.with_context(WARP_SYNC_PROTOCOL_ID, |context| {
			let mut sync_io = NetSyncIo::new(context,
				&*self.eth_handler.chain,
				&*self.eth_handler.snapshot_service,
				&self.eth_handler.overlay,
				self.eth_handler.private_state.clone());
			match message_type {
				ChainMessageType::Consensus(message) => self.eth_handler.sync.write().propagate_consensus_packet(&mut sync_io, message),
				ChainMessageType::PrivateTransaction(transaction_hash, message) =>
					self.eth_handler.sync.write().propagate_private_transaction(&mut sync_io, transaction_hash, PrivateTransactionPacket, message),
				ChainMessageType::SignedPrivateTransaction(transaction_hash, message) =>
					self.eth_handler.sync.write().propagate_private_transaction(&mut sync_io, transaction_hash, SignedPrivateTransactionPacket, message),
				ChainMessageType::PrivateStateRequest(hash) =>
					self.eth_handler.sync.write().request_private_state(&mut sync_io, &hash),
			}
		});
	}

	fn transactions_received(&self, txs: &[UnverifiedTransaction], peer_id: PeerId) {
		let mut sync = self.eth_handler.sync.write();
		sync.transactions_received(txs, peer_id);
	}
}

/// PIP event handler.
/// Simply queues transactions from light client peers.
struct TxRelay(Arc<dyn BlockChainClient>);

impl LightHandler for TxRelay {
	fn on_transactions(&self, ctx: &dyn EventContext, relay: &[UnverifiedTransaction]) {
		trace!(target: "pip", "Relaying {} transactions from peer {}", relay.len(), ctx.peer());
		self.0.queue_transactions(relay.iter().map(|tx| rlp::encode(tx)).collect(), ctx.peer())
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
		self.network.set_non_reserved_mode(NonReservedPeerMode::Accept);
	}

	fn deny_unreserved_peers(&self) {
		self.network.set_non_reserved_mode(NonReservedPeerMode::Deny);
	}

	fn remove_reserved_peer(&self, peer: String) -> Result<(), String> {
		self.network.remove_reserved_peer(&peer).map_err(|e| format!("{:?}", e))
	}

	fn add_reserved_peer(&self, peer: String) -> Result<(), String> {
		self.network.add_reserved_peer(&peer).map_err(|e| format!("{:?}", e))
	}

	fn start_network(&self) {
		self.start();
	}

	fn stop_network(&self) {
		self.network.with_context(self.subprotocol_name, |context| {
			let mut sync_io = NetSyncIo::new(context,
				&*self.eth_handler.chain,
				&*self.eth_handler.snapshot_service,
				&self.eth_handler.overlay,
				self.eth_handler.private_state.clone());
			self.eth_handler.sync.write().abort(&mut sync_io);
		});

		if let Some(light_proto) = self.light_proto.as_ref() {
			light_proto.abort();
		}

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
	/// Nat type
	pub nat_type: NatType,
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
			listen_address: match self.listen_address { None => None, Some(addr) => Some(SocketAddr::from_str(&addr)?) },
			public_address: match self.public_address { None => None, Some(addr) => Some(SocketAddr::from_str(&addr)?) },
			udp_port: self.udp_port,
			nat_enabled: self.nat_enabled,
			nat_type: self.nat_type,
			discovery_enabled: self.discovery_enabled,
			boot_nodes: self.boot_nodes,
			use_secret: self.use_secret,
			max_peers: self.max_peers,
			min_peers: self.min_peers,
			max_handshakes: self.max_pending_peers,
			reserved_protocols: hash_map![WARP_SYNC_PROTOCOL_ID => self.snapshot_peers],
			reserved_nodes: self.reserved_nodes,
			ip_filter: self.ip_filter,
			non_reserved_mode: if self.allow_non_reserved { NonReservedPeerMode::Accept } else { NonReservedPeerMode::Deny },
			client_version: self.client_version,
		})
	}
}

impl From<BasicNetworkConfiguration> for NetworkConfiguration {
	fn from(other: BasicNetworkConfiguration) -> Self {
		NetworkConfiguration {
			config_path: other.config_path,
			net_config_path: other.net_config_path,
			listen_address: other.listen_address.and_then(|addr| Some(format!("{}", addr))),
			public_address: other.public_address.and_then(|addr| Some(format!("{}", addr))),
			udp_port: other.udp_port,
			nat_enabled: other.nat_enabled,
			nat_type: other.nat_type,
			discovery_enabled: other.discovery_enabled,
			boot_nodes: other.boot_nodes,
			use_secret: other.use_secret,
			max_peers: other.max_peers,
			min_peers: other.min_peers,
			max_pending_peers: other.max_handshakes,
			snapshot_peers: *other.reserved_protocols.get(&WARP_SYNC_PROTOCOL_ID).unwrap_or(&0),
			reserved_nodes: other.reserved_nodes,
			ip_filter: other.ip_filter,
			allow_non_reserved: match other.non_reserved_mode { NonReservedPeerMode::Accept => true, _ => false } ,
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

/// Light synchronization.
pub trait LightSyncProvider {
	/// Get peer numbers.
	fn peer_numbers(&self) -> PeerNumbers;

	/// Get peers information
	fn peers(&self) -> Vec<PeerInfo>;

	/// Get network id.
	fn network_id(&self) -> u64;

	/// Get the enode if available.
	fn enode(&self) -> Option<String>;

	/// Returns propagation count for pending transactions.
	fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats>;
}

/// Wrapper around `light_sync::SyncInfo` to expose those methods without the concrete type `LightSync`
pub trait LightSyncInfo: Send + Sync {
	/// Get the highest block advertised on the network.
	fn highest_block(&self) -> Option<u64>;

	/// Get the block number at the time of sync start.
	fn start_block(&self) -> u64;

	/// Whether major sync is underway.
	fn is_major_importing(&self) -> bool;
}

/// Execute a closure with a protocol context.
pub trait LightNetworkDispatcher {
	/// Execute a closure with a protocol context.
	fn with_context<F, T>(&self, f: F) -> Option<T> where F: FnOnce(&dyn (::light::net::BasicContext)) -> T;
}

/// Configuration for the light sync.
pub struct LightSyncParams<L> {
	/// Network configuration.
	pub network_config: BasicNetworkConfiguration,
	/// Light client to sync to.
	pub client: Arc<L>,
	/// Network ID.
	pub network_id: u64,
	/// Subprotocol name.
	pub subprotocol_name: [u8; 3],
	/// Other handlers to attach.
	pub handlers: Vec<Arc<dyn LightHandler>>,
}

/// Service for light synchronization.
pub struct LightSync {
	proto: Arc<LightProtocol>,
	sync: Arc<dyn SyncInfo + Sync + Send>,
	network: NetworkService,
	subprotocol_name: [u8; 3],
	network_id: u64,
}

impl LightSync {
	/// Create a new light sync service.
	pub fn new<L>(params: LightSyncParams<L>) -> Result<Self, Error>
		where L: AsLightClient + Provider + Sync + Send + 'static
	{
		use light_sync::LightSync as SyncHandler;

		// initialize light protocol handler and attach sync module.
		let (sync, light_proto) = {
			let light_params = LightParams {
				network_id: params.network_id,
				config: Default::default(),
				capabilities: Capabilities {
					serve_headers: false,
					serve_chain_since: None,
					serve_state_since: None,
					tx_relay: false,
				},
				sample_store: None,
			};

			let mut light_proto = LightProtocol::new(params.client.clone(), light_params);
			let sync_handler = Arc::new(SyncHandler::new(params.client.clone())?);
			light_proto.add_handler(sync_handler.clone());

			for handler in params.handlers {
				light_proto.add_handler(handler);
			}

			(sync_handler, Arc::new(light_proto))
		};

		let service = NetworkService::new(params.network_config, None)?;

		Ok(LightSync {
			proto: light_proto,
			sync: sync,
			network: service,
			subprotocol_name: params.subprotocol_name,
			network_id: params.network_id,
		})
	}

}

impl std::ops::Deref for LightSync {
	type Target = dyn (light_sync::SyncInfo);

	fn deref(&self) -> &Self::Target { &*self.sync }
}


impl LightNetworkDispatcher for LightSync {
	fn with_context<F, T>(&self, f: F) -> Option<T> where F: FnOnce(&dyn (light::net::BasicContext)) -> T {
		self.network.with_context_eval(
			self.subprotocol_name,
			move |ctx| self.proto.with_context(&ctx, f),
		)
	}
}

impl ManageNetwork for LightSync {
	fn accept_unreserved_peers(&self) {
		self.network.set_non_reserved_mode(NonReservedPeerMode::Accept);
	}

	fn deny_unreserved_peers(&self) {
		self.network.set_non_reserved_mode(NonReservedPeerMode::Deny);
	}

	fn remove_reserved_peer(&self, peer: String) -> Result<(), String> {
		self.network.remove_reserved_peer(&peer).map_err(|e| format!("{:?}", e))
	}

	fn add_reserved_peer(&self, peer: String) -> Result<(), String> {
		self.network.add_reserved_peer(&peer).map_err(|e| format!("{:?}", e))
	}

	fn start_network(&self) {
		match self.network.start() {
			Err((err, listen_address)) => {
				match err.into() {
					Error::Io(ref e) if e.kind() == io::ErrorKind::AddrInUse => {
						warn!("Network port {:?} is already in use, make sure that another instance of an Ethereum client is not running or change the port using the --port option.", listen_address.expect("Listen address is not set."))
					},
					err => warn!("Error starting network: {}", err),
				}
			},
			_ => {},
		}

		let light_proto = self.proto.clone();

		self.network.register_protocol(light_proto, self.subprotocol_name, ::light::net::PROTOCOL_VERSIONS)
			.unwrap_or_else(|e| warn!("Error registering light client protocol: {:?}", e));
	}

	fn stop_network(&self) {
		self.proto.abort();
		self.network.stop();
	}

	fn num_peers_range(&self) -> RangeInclusive<u32> {
		self.network.num_peers_range()
	}

	fn with_proto_context(&self, proto: ProtocolId, f: &mut dyn FnMut(&dyn NetworkContext)) {
		self.network.with_context_eval(proto, f);
	}
}

impl LightSyncProvider for LightSync {
	fn peer_numbers(&self) -> PeerNumbers {
		let (connected, active) = self.proto.peer_count();
		let peers_range = self.num_peers_range();
		debug_assert!(peers_range.end() >= peers_range.start());
		PeerNumbers {
			connected: connected,
			active: active,
			max: *peers_range.end() as usize,
			min: *peers_range.start() as usize,
		}
	}

	fn peers(&self) -> Vec<PeerInfo> {
		self.network.with_context_eval(self.subprotocol_name, |ctx| {
			let peer_ids = self.network.connected_peers();

			peer_ids.into_iter().filter_map(|peer_id| {
				let session_info = match ctx.session_info(peer_id) {
					None => return None,
					Some(info) => info,
				};

				Some(PeerInfo {
					id: session_info.id.map(|id| format!("{:x}", id)),
					client_version: session_info.client_version,
					capabilities: session_info.peer_capabilities.into_iter().map(|c| c.to_string()).collect(),
					remote_address: session_info.remote_address,
					local_address: session_info.local_address,
					eth_info: None,
					pip_info: self.proto.peer_status(peer_id).map(Into::into),
				})
			}).collect()
		}).unwrap_or_else(Vec::new)
	}

	fn enode(&self) -> Option<String> {
		self.network.external_url()
	}

	fn network_id(&self) -> u64 {
		self.network_id
	}

	fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats> {
		Default::default() // TODO
	}
}

impl LightSyncInfo for LightSync {
	fn highest_block(&self) -> Option<u64> {
		(*self.sync).highest_block()
	}

	fn start_block(&self) -> u64 {
		(*self.sync).start_block()
	}

	fn is_major_importing(&self) -> bool {
		(*self.sync).is_major_importing()
	}
}
