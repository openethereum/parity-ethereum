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
use std::collections::{HashMap, BTreeMap};
use std::io;
use bytes::Bytes;
use devp2p::{NetworkService, ConnectionFilter};
use network::{NetworkProtocolHandler, NetworkContext, HostInfo, PeerId, ProtocolId,
	NetworkConfiguration as BasicNetworkConfiguration, NonReservedPeerMode, Error, ErrorKind};
use ethereum_types::{H256, H512, U256};
use io::{TimerToken};
use ethcore::ethstore::ethkey::Secret;
use ethcore::client::{BlockChainClient, ChainNotify, ChainMessageType};
use ethcore::snapshot::SnapshotService;
use ethcore::header::BlockNumber;
use sync_io::NetSyncIo;
use chain::{ChainSync, SyncStatus as EthSyncStatus};
use std::net::{SocketAddr, AddrParseError};
use std::str::FromStr;
use parking_lot::RwLock;
use chain::{ETH_PACKET_COUNT, SNAPSHOT_SYNC_PACKET_COUNT, ETH_PROTOCOL_VERSION_63, ETH_PROTOCOL_VERSION_62,
	PAR_PROTOCOL_VERSION_1, PAR_PROTOCOL_VERSION_2, PAR_PROTOCOL_VERSION_3};
use light::client::AsLightClient;
use light::Provider;
use light::net::{self as light_net, LightProtocol, Params as LightParams, Capabilities, Handler as LightHandler, EventContext};
use network::IpFilter;
use private_tx::PrivateTxHandler;

/// Parity sync protocol
pub const WARP_SYNC_PROTOCOL_ID: ProtocolId = *b"par";
/// Ethereum sync protocol
pub const ETH_PROTOCOL: ProtocolId = *b"eth";
/// Ethereum light protocol
pub const LIGHT_PROTOCOL: ProtocolId = *b"pip";

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

/// Current sync status
pub trait SyncProvider: Send + Sync {
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
	pub client_version: String,
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

/// Configuration to attach alternate protocol handlers.
/// Only works when IPC is disabled.
pub struct AttachedProtocol {
	/// The protocol handler in question.
	pub handler: Arc<NetworkProtocolHandler + Send + Sync>,
	/// 3-character ID for the protocol.
	pub protocol_id: ProtocolId,
	/// Packet count.
	pub packet_count: u8,
	/// Supported versions.
	pub versions: &'static [u8],
}

impl AttachedProtocol {
	fn register(&self, network: &NetworkService) {
		let res = network.register_protocol(
			self.handler.clone(),
			self.protocol_id,
			self.packet_count,
			self.versions
		);

		if let Err(e) = res {
			warn!(target: "sync", "Error attaching protocol {:?}: {:?}", self.protocol_id, e);
		}
	}
}

/// EthSync initialization parameters.
pub struct Params {
	/// Configuration.
	pub config: SyncConfig,
	/// Blockchain client.
	pub chain: Arc<BlockChainClient>,
	/// Snapshot service.
	pub snapshot_service: Arc<SnapshotService>,
	/// Private tx service.
	pub private_tx_handler: Arc<PrivateTxHandler>,
	/// Light data provider.
	pub provider: Arc<::light::Provider>,
	/// Network layer configuration.
	pub network_config: NetworkConfiguration,
	/// Other protocols to attach.
	pub attached_protos: Vec<AttachedProtocol>,
}

/// Ethereum network protocol handler
pub struct EthSync {
	/// Network service
	network: NetworkService,
	/// Main (eth/par) protocol handler
	eth_handler: Arc<SyncProtocolHandler>,
	/// Light (pip) protocol handler
	light_proto: Option<Arc<LightProtocol>>,
	/// Other protocols to attach.
	attached_protos: Vec<AttachedProtocol>,
	/// The main subprotocol name
	subprotocol_name: [u8; 3],
	/// Light subprotocol name.
	light_subprotocol_name: [u8; 3],
}

impl EthSync {
	/// Creates and register protocol with the network service
	pub fn new(params: Params, connection_filter: Option<Arc<ConnectionFilter>>) -> Result<Arc<EthSync>, Error> {
		const MAX_LIGHTSERV_LOAD: f64 = 0.5;

		let pruning_info = params.chain.pruning_info();
		let light_proto = match params.config.serve_light {
			false => None,
			true => Some({
				let sample_store = params.network_config.net_config_path
					.clone()
					.map(::std::path::PathBuf::from)
					.map(|mut p| { p.push("request_timings"); light_net::FileStore(p) })
					.map(|store| Box::new(store) as Box<_>);

				let mut light_params = LightParams {
					network_id: params.config.network_id,
					config: Default::default(),
					capabilities: Capabilities {
						serve_headers: true,
						serve_chain_since: Some(pruning_info.earliest_chain),
						serve_state_since: Some(pruning_info.earliest_state),
						tx_relay: true,
					},
					sample_store: sample_store,
				};

				let max_peers = ::std::cmp::min(params.network_config.max_peers, 1);
				light_params.config.load_share = MAX_LIGHTSERV_LOAD / max_peers as f64;

				let mut light_proto = LightProtocol::new(params.provider, light_params);
				light_proto.add_handler(Arc::new(TxRelay(params.chain.clone())));

				Arc::new(light_proto)
			})
		};

		let chain_sync = ChainSync::new(params.config, &*params.chain, params.private_tx_handler.clone());
		let service = NetworkService::new(params.network_config.clone().into_basic()?, connection_filter)?;

		let sync = Arc::new(EthSync {
			network: service,
			eth_handler: Arc::new(SyncProtocolHandler {
				sync: RwLock::new(chain_sync),
				chain: params.chain,
				snapshot_service: params.snapshot_service,
				overlay: RwLock::new(HashMap::new()),
			}),
			light_proto: light_proto,
			subprotocol_name: params.config.subprotocol_name,
			light_subprotocol_name: params.config.light_subprotocol_name,
			attached_protos: params.attached_protos,
		});

		Ok(sync)
	}
}

impl SyncProvider for EthSync {
	/// Get sync status
	fn status(&self) -> EthSyncStatus {
		self.eth_handler.sync.write().status()
	}

	/// Get sync peers
	fn peers(&self) -> Vec<PeerInfo> {
		self.network.with_context_eval(self.subprotocol_name, |ctx| {
			let peer_ids = self.network.connected_peers();
			let eth_sync = self.eth_handler.sync.read();
			let light_proto = self.light_proto.as_ref();

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
					eth_info: eth_sync.peer_info(&peer_id),
					pip_info: light_proto.as_ref().and_then(|lp| lp.peer_status(&peer_id)).map(Into::into),
				})
			}).collect()
		}).unwrap_or_else(Vec::new)
	}

	fn enode(&self) -> Option<String> {
		self.network.external_url()
	}

	fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats> {
		let sync = self.eth_handler.sync.read();
		sync.transactions_stats()
			.iter()
			.map(|(hash, stats)| (*hash, stats.into()))
			.collect()
	}
}

struct SyncProtocolHandler {
	/// Shared blockchain client.
	chain: Arc<BlockChainClient>,
	/// Shared snapshot service.
	snapshot_service: Arc<SnapshotService>,
	/// Sync strategy
	sync: RwLock<ChainSync>,
	/// Chain overlay used to cache data such as fork block.
	overlay: RwLock<HashMap<BlockNumber, Bytes>>,
}

impl NetworkProtocolHandler for SyncProtocolHandler {
	fn initialize(&self, io: &NetworkContext, _host_info: &HostInfo) {
		if io.subprotocol_name() != WARP_SYNC_PROTOCOL_ID {
			io.register_timer(0, 1000).expect("Error registering sync timer");
		}
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		ChainSync::dispatch_packet(&self.sync, &mut NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay), *peer, packet_id, data);
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		// If warp protocol is supported only allow warp handshake
		let warp_protocol = io.protocol_version(WARP_SYNC_PROTOCOL_ID, *peer).unwrap_or(0) != 0;
		let warp_context = io.subprotocol_name() == WARP_SYNC_PROTOCOL_ID;
		if warp_protocol == warp_context {
			self.sync.write().on_peer_connected(&mut NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay), *peer);
		}
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
		if io.subprotocol_name() != WARP_SYNC_PROTOCOL_ID {
			self.sync.write().on_peer_aborting(&mut NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay), *peer);
		}
	}

	fn timeout(&self, io: &NetworkContext, _timer: TimerToken) {
		let mut io = NetSyncIo::new(io, &*self.chain, &*self.snapshot_service, &self.overlay);
		self.sync.write().maintain_peers(&mut io);
		self.sync.write().maintain_sync(&mut io);
		self.sync.write().propagate_new_transactions(&mut io);
	}
}

impl ChainNotify for EthSync {
	fn new_blocks(&self,
		imported: Vec<H256>,
		invalid: Vec<H256>,
		enacted: Vec<H256>,
		retracted: Vec<H256>,
		sealed: Vec<H256>,
		proposed: Vec<Bytes>,
		_duration: u64)
	{
		use light::net::Announcement;

		self.network.with_context(self.subprotocol_name, |context| {
			let mut sync_io = NetSyncIo::new(context, &*self.eth_handler.chain, &*self.eth_handler.snapshot_service,
				&self.eth_handler.overlay);
			self.eth_handler.sync.write().chain_new_blocks(
				&mut sync_io,
				&imported,
				&invalid,
				&enacted,
				&retracted,
				&sealed,
				&proposed);
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
		match self.network.start().map_err(Into::into) {
			Err(ErrorKind::Io(ref e)) if  e.kind() == io::ErrorKind::AddrInUse => warn!("Network port {:?} is already in use, make sure that another instance of an Ethereum client is not running or change the port using the --port option.", self.network.config().listen_address.expect("Listen address is not set.")),
			Err(err) => warn!("Error starting network: {}", err),
			_ => {},
		}
		self.network.register_protocol(self.eth_handler.clone(), self.subprotocol_name, ETH_PACKET_COUNT, &[ETH_PROTOCOL_VERSION_62, ETH_PROTOCOL_VERSION_63])
			.unwrap_or_else(|e| warn!("Error registering ethereum protocol: {:?}", e));
		// register the warp sync subprotocol
		self.network.register_protocol(self.eth_handler.clone(), WARP_SYNC_PROTOCOL_ID, SNAPSHOT_SYNC_PACKET_COUNT, &[PAR_PROTOCOL_VERSION_1, PAR_PROTOCOL_VERSION_2, PAR_PROTOCOL_VERSION_3])
			.unwrap_or_else(|e| warn!("Error registering snapshot sync protocol: {:?}", e));

		// register the light protocol.
		if let Some(light_proto) = self.light_proto.as_ref().map(|x| x.clone()) {
			self.network.register_protocol(light_proto, self.light_subprotocol_name, ::light::net::PACKET_COUNT, ::light::net::PROTOCOL_VERSIONS)
				.unwrap_or_else(|e| warn!("Error registering light client protocol: {:?}", e));
		}

		// register any attached protocols.
		for proto in &self.attached_protos { proto.register(&self.network) }
	}

	fn stop(&self) {
		self.eth_handler.snapshot_service.abort_restore();
		self.network.stop().unwrap_or_else(|e| warn!("Error stopping network: {:?}", e));
	}

	fn broadcast(&self, message_type: ChainMessageType) {
		self.network.with_context(WARP_SYNC_PROTOCOL_ID, |context| {
			let mut sync_io = NetSyncIo::new(context, &*self.eth_handler.chain, &*self.eth_handler.snapshot_service, &self.eth_handler.overlay);
			match message_type {
				ChainMessageType::Consensus(message) => self.eth_handler.sync.write().propagate_consensus_packet(&mut sync_io, message),
				ChainMessageType::PrivateTransaction(message) => self.eth_handler.sync.write().propagate_private_transaction(&mut sync_io, message),
				ChainMessageType::SignedPrivateTransaction(message) => self.eth_handler.sync.write().propagate_signed_private_transaction(&mut sync_io, message),
			}
		});
	}

	fn transactions_received(&self, hashes: Vec<H256>, peer_id: PeerId) {
		let mut sync = self.eth_handler.sync.write();
		sync.transactions_received(hashes, peer_id);
	}
}

/// PIP event handler.
/// Simply queues transactions from light client peers.
struct TxRelay(Arc<BlockChainClient>);

impl LightHandler for TxRelay {
	fn on_transactions(&self, ctx: &EventContext, relay: &[::transaction::UnverifiedTransaction]) {
		trace!(target: "pip", "Relaying {} transactions from peer {}", relay.len(), ctx.peer());
		self.0.queue_transactions(relay.iter().map(|tx| ::rlp::encode(tx).into_vec()).collect(), ctx.peer())
	}
}

/// Trait for managing network
pub trait ManageNetwork : Send + Sync {
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
	/// Query the current configuration of the network
	fn network_config(&self) -> NetworkConfiguration;
	/// Get network context for protocol.
	fn with_proto_context(&self, proto: ProtocolId, f: &mut FnMut(&NetworkContext));
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
			let mut sync_io = NetSyncIo::new(context, &*self.eth_handler.chain, &*self.eth_handler.snapshot_service, &self.eth_handler.overlay);
			self.eth_handler.sync.write().abort(&mut sync_io);
		});

		if let Some(light_proto) = self.light_proto.as_ref() {
			light_proto.abort();
		}

		self.stop();
	}

	fn network_config(&self) -> NetworkConfiguration {
		NetworkConfiguration::from(self.network.config().clone())
	}

	fn with_proto_context(&self, proto: ProtocolId, f: &mut FnMut(&NetworkContext)) {
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
			listen_address: match self.listen_address { None => None, Some(addr) => Some(SocketAddr::from_str(&addr)?) },
			public_address:  match self.public_address { None => None, Some(addr) => Some(SocketAddr::from_str(&addr)?) },
			udp_port: self.udp_port,
			nat_enabled: self.nat_enabled,
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
	pub handlers: Vec<Arc<LightHandler>>,
	/// Other subprotocols to run.
	pub attached_protos: Vec<AttachedProtocol>,
}

/// Service for light synchronization.
pub struct LightSync {
	proto: Arc<LightProtocol>,
	sync: Arc<::light_sync::SyncInfo + Sync + Send>,
	attached_protos: Vec<AttachedProtocol>,
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
			attached_protos: params.attached_protos,
			network: service,
			subprotocol_name: params.subprotocol_name,
			network_id: params.network_id,
		})
	}

	/// Execute a closure with a protocol context.
	pub fn with_context<F, T>(&self, f: F) -> Option<T>
		where F: FnOnce(&::light::net::BasicContext) -> T
	{
		self.network.with_context_eval(
			self.subprotocol_name,
			move |ctx| self.proto.with_context(&ctx, f),
		)
	}
}

impl ::std::ops::Deref for LightSync {
	type Target = ::light_sync::SyncInfo;

	fn deref(&self) -> &Self::Target { &*self.sync }
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
		match self.network.start().map_err(Into::into) {
			Err(ErrorKind::Io(ref e)) if e.kind() == io::ErrorKind::AddrInUse => {
				warn!("Network port {:?} is already in use, make sure that another instance of an Ethereum client is not running or change the port using the --port option.", self.network.config().listen_address.expect("Listen address is not set."))
			}
			Err(err) => warn!("Error starting network: {}", err),
			_ => {},
		}

		let light_proto = self.proto.clone();

		self.network.register_protocol(light_proto, self.subprotocol_name, ::light::net::PACKET_COUNT, ::light::net::PROTOCOL_VERSIONS)
			.unwrap_or_else(|e| warn!("Error registering light client protocol: {:?}", e));

		for proto in &self.attached_protos { proto.register(&self.network) }
	}

	fn stop_network(&self) {
		self.proto.abort();
		if let Err(e) = self.network.stop() {
			warn!("Error stopping network: {}", e);
		}
	}

	fn network_config(&self) -> NetworkConfiguration {
		NetworkConfiguration::from(self.network.config().clone())
	}

	fn with_proto_context(&self, proto: ProtocolId, f: &mut FnMut(&NetworkContext)) {
		self.network.with_context_eval(proto, f);
	}
}

impl LightSyncProvider for LightSync {
	fn peer_numbers(&self) -> PeerNumbers {
		let (connected, active) = self.proto.peer_count();
		let config = self.network_config();
		PeerNumbers {
			connected: connected,
			active: active,
			max: config.max_peers as usize,
			min: config.min_peers as usize,
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
					pip_info: self.proto.peer_status(&peer_id).map(Into::into),
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
