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

use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::ops::*;
use std::cmp::{min, max};
use std::path::{Path, PathBuf};
use std::io::{Read, Write, self};
use std::fs;
use std::time::Duration;
use ethkey::{KeyPair, Secret, Random, Generator};
use hash::keccak;
use mio::*;
use mio::deprecated::{EventLoop};
use mio::tcp::*;
use ethereum_types::H256;
use rlp::{RlpStream, Encodable};

use session::{Session, SessionData};
use io::*;
use PROTOCOL_VERSION;
use node_table::*;
use network::{NetworkConfiguration, NetworkIoMessage, ProtocolId, PeerId, PacketId};
use network::{NonReservedPeerMode, NetworkContext as NetworkContextTrait};
use network::{SessionInfo, Error, ErrorKind, DisconnectReason, NetworkProtocolHandler};
use discovery::{Discovery, TableUpdates, NodeEntry};
use ip_utils::{map_external_address, select_public_address};
use path::restrict_permissions_owner;
use parking_lot::{Mutex, RwLock};
use network::{ConnectionFilter, ConnectionDirection};

type Slab<T> = ::slab::Slab<T, usize>;

const MAX_SESSIONS: usize = 1024 + MAX_HANDSHAKES;
const MAX_HANDSHAKES: usize = 1024;

const DEFAULT_PORT: u16 = 30303;

// StreamToken/TimerToken
const TCP_ACCEPT: StreamToken = SYS_TIMER + 1;
const IDLE: TimerToken = SYS_TIMER + 2;
const DISCOVERY: StreamToken = SYS_TIMER + 3;
const DISCOVERY_REFRESH: TimerToken = SYS_TIMER + 4;
const DISCOVERY_ROUND: TimerToken = SYS_TIMER + 5;
const NODE_TABLE: TimerToken = SYS_TIMER + 6;
const FIRST_SESSION: StreamToken = 0;
const LAST_SESSION: StreamToken = FIRST_SESSION + MAX_SESSIONS - 1;
const USER_TIMER: TimerToken = LAST_SESSION + 256;
const SYS_TIMER: TimerToken = LAST_SESSION + 1;

// Timeouts
// for IDLE TimerToken
const MAINTENANCE_TIMEOUT: Duration = Duration::from_secs(1);
// for DISCOVERY_REFRESH TimerToken
const DISCOVERY_REFRESH_TIMEOUT: Duration = Duration::from_secs(60);
// for DISCOVERY_ROUND TimerToken
const DISCOVERY_ROUND_TIMEOUT: Duration = Duration::from_millis(300);
// for NODE_TABLE TimerToken
const NODE_TABLE_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, PartialEq, Eq)]
/// Protocol info
pub struct CapabilityInfo {
	/// Protocol ID
	pub protocol: ProtocolId,
	/// Protocol version
	pub version: u8,
	/// Total number of packet IDs this protocol support.
	pub packet_count: u8,
}

impl Encodable for CapabilityInfo {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&&self.protocol[..]);
		s.append(&self.version);
	}
}

/// IO access point. This is passed to all IO handlers and provides an interface to the IO subsystem.
pub struct NetworkContext<'s> {
	io: &'s IoContext<NetworkIoMessage>,
	protocol: ProtocolId,
	sessions: Arc<RwLock<Slab<SharedSession>>>,
	session: Option<SharedSession>,
	session_id: Option<StreamToken>,
	_reserved_peers: &'s HashSet<NodeId>,
}

impl<'s> NetworkContext<'s> {
	/// Create a new network IO access point. Takes references to all the data that can be updated within the IO handler.
	fn new(
		io: &'s IoContext<NetworkIoMessage>,
		protocol: ProtocolId,
		session: Option<SharedSession>,
		sessions: Arc<RwLock<Slab<SharedSession>>>,
		reserved_peers: &'s HashSet<NodeId>,
	) -> NetworkContext<'s> {
		let id = session.as_ref().map(|s| s.lock().token());
		NetworkContext {
			io,
			protocol,
			session_id: id,
			session,
			sessions,
			_reserved_peers: reserved_peers,
		}
	}

	fn resolve_session(&self, peer: PeerId) -> Option<SharedSession> {
		match self.session_id {
			Some(id) if id == peer => self.session.clone(),
			_ => self.sessions.read().get(peer).cloned(),
		}
	}
}

impl<'s> NetworkContextTrait for NetworkContext<'s> {
	fn send(&self, peer: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error> {
		self.send_protocol(self.protocol, peer, packet_id, data)
	}

	fn send_protocol(&self, protocol: ProtocolId, peer: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error> {
		let session = self.resolve_session(peer);
		if let Some(session) = session {
			session.lock().send_packet(self.io, Some(protocol), packet_id as u8, &data)?;
		} else  {
			trace!(target: "network", "Send: Peer no longer exist")
		}
		Ok(())
	}

	fn respond(&self, packet_id: PacketId, data: Vec<u8>) -> Result<(), Error> {
		assert!(self.session.is_some(), "Respond called without network context");
		self.session_id.map_or_else(|| Err(ErrorKind::Expired.into()), |id| self.send(id, packet_id, data))
	}

	fn disable_peer(&self, peer: PeerId) {
		self.io.message(NetworkIoMessage::DisablePeer(peer))
			.unwrap_or_else(|e| warn!("Error sending network IO message: {:?}", e));
	}

	fn disconnect_peer(&self, peer: PeerId) {
		self.io.message(NetworkIoMessage::Disconnect(peer))
			.unwrap_or_else(|e| warn!("Error sending network IO message: {:?}", e));
	}

	fn is_expired(&self) -> bool {
		self.session.as_ref().map_or(false, |s| s.lock().expired())
	}

	fn register_timer(&self, token: TimerToken, delay: Duration) -> Result<(), Error> {
		self.io.message(NetworkIoMessage::AddTimer {
			token,
			delay,
			protocol: self.protocol,
		}).unwrap_or_else(|e| warn!("Error sending network IO message: {:?}", e));
		Ok(())
	}

	fn peer_client_version(&self, peer: PeerId) -> String {
		self.resolve_session(peer).map_or("unknown".to_owned(), |s| s.lock().info.client_version.clone())
	}

	fn session_info(&self, peer: PeerId) -> Option<SessionInfo> {
		self.resolve_session(peer).map(|s| s.lock().info.clone())
	}

	fn protocol_version(&self, protocol: ProtocolId, peer: PeerId) -> Option<u8> {
		let session = self.resolve_session(peer);
		session.and_then(|s| s.lock().capability_version(protocol))
	}

	fn subprotocol_name(&self) -> ProtocolId { self.protocol }
}

/// Shared host information
pub struct HostInfo {
	/// Our private and public keys.
	keys: KeyPair,
	/// Current network configuration
	config: NetworkConfiguration,
	/// Connection nonce.
	nonce: H256,
	/// RLPx protocol version
	pub protocol_version: u32,
	/// Registered capabilities (handlers)
	pub capabilities: Vec<CapabilityInfo>,
	/// Local address + discovery port
	pub local_endpoint: NodeEndpoint,
	/// Public address + discovery port
	pub public_endpoint: Option<NodeEndpoint>,
}

impl HostInfo {
	fn next_nonce(&mut self) -> H256 {
		self.nonce = keccak(&self.nonce);
		self.nonce
	}

	pub(crate) fn client_version(&self) -> &str {
		&self.config.client_version
	}

	pub(crate) fn secret(&self) -> &Secret {
		self.keys.secret()
	}

	pub(crate) fn id(&self) -> &NodeId {
		self.keys.public()
	}
}

type SharedSession = Arc<Mutex<Session>>;

#[derive(Copy, Clone)]
struct ProtocolTimer {
	pub protocol: ProtocolId,
	pub token: TimerToken, // Handler level token
}

/// Root IO handler. Manages protocol handlers, IO timers and network connections.
pub struct Host {
	pub info: RwLock<HostInfo>,
	tcp_listener: Mutex<TcpListener>,
	sessions: Arc<RwLock<Slab<SharedSession>>>,
	discovery: Mutex<Option<Discovery>>,
	nodes: RwLock<NodeTable>,
	handlers: RwLock<HashMap<ProtocolId, Arc<NetworkProtocolHandler + Sync>>>,
	timers: RwLock<HashMap<TimerToken, ProtocolTimer>>,
	timer_counter: RwLock<usize>,
	reserved_nodes: RwLock<HashSet<NodeId>>,
	stopping: AtomicBool,
	filter: Option<Arc<ConnectionFilter>>,
}

impl Host {
	/// Create a new instance
	pub fn new(mut config: NetworkConfiguration, filter: Option<Arc<ConnectionFilter>>) -> Result<Host, Error> {
		let mut listen_address = match config.listen_address {
			None => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), DEFAULT_PORT)),
			Some(addr) => addr,
		};

		let keys = if let Some(ref secret) = config.use_secret {
			KeyPair::from_secret(secret.clone())?
		} else {
			config.config_path.clone().and_then(|ref p| load_key(Path::new(&p)))
				.map_or_else(|| {
				let key = Random.generate().expect("Error generating random key pair");
				if let Some(path) = config.config_path.clone() {
					save_key(Path::new(&path), key.secret());
				}
				key
			},
			|s| KeyPair::from_secret(s).expect("Error creating node secret key"))
		};
		let path = config.net_config_path.clone();
		// Setup the server socket
		let tcp_listener = TcpListener::bind(&listen_address)?;
		listen_address = SocketAddr::new(listen_address.ip(), tcp_listener.local_addr()?.port());
		debug!(target: "network", "Listening at {:?}", listen_address);
		let udp_port = config.udp_port.unwrap_or_else(|| listen_address.port());
		let local_endpoint = NodeEndpoint { address: listen_address, udp_port: udp_port };

		let boot_nodes = config.boot_nodes.clone();
		let reserved_nodes = config.reserved_nodes.clone();
		config.max_handshakes = min(config.max_handshakes, MAX_HANDSHAKES as u32);

		let mut host = Host {
			info: RwLock::new(HostInfo {
				keys: keys,
				config: config,
				nonce: H256::random(),
				protocol_version: PROTOCOL_VERSION,
				capabilities: Vec::new(),
				public_endpoint: None,
				local_endpoint: local_endpoint,
			}),
			discovery: Mutex::new(None),
			tcp_listener: Mutex::new(tcp_listener),
			sessions: Arc::new(RwLock::new(Slab::new_starting_at(FIRST_SESSION, MAX_SESSIONS))),
			nodes: RwLock::new(NodeTable::new(path)),
			handlers: RwLock::new(HashMap::new()),
			timers: RwLock::new(HashMap::new()),
			timer_counter: RwLock::new(USER_TIMER),
			reserved_nodes: RwLock::new(HashSet::new()),
			stopping: AtomicBool::new(false),
			filter: filter,
		};

		for n in boot_nodes {
			host.add_node(&n);
		}

		for n in reserved_nodes {
			if let Err(e) = host.add_reserved_node(&n) {
				debug!(target: "network", "Error parsing node id: {}: {:?}", n, e);
			}
		}
		Ok(host)
	}

	pub fn add_node(&mut self, id: &str) {
		match Node::from_str(id) {
			Err(e) => { debug!(target: "network", "Could not add node {}: {:?}", id, e); },
			Ok(n) => {
				let entry = NodeEntry { endpoint: n.endpoint.clone(), id: n.id };

				self.nodes.write().add_node(n);
				if let Some(ref mut discovery) = *self.discovery.lock() {
					discovery.add_node(entry);
				}
			}
		}
	}

	pub fn add_reserved_node(&self, id: &str) -> Result<(), Error> {
		let n = Node::from_str(id)?;

		let entry = NodeEntry { endpoint: n.endpoint.clone(), id: n.id };
		self.reserved_nodes.write().insert(n.id);
		self.nodes.write().add_node(Node::new(entry.id, entry.endpoint.clone()));

		if let Some(ref mut discovery) = *self.discovery.lock() {
			discovery.add_node(entry);
		}

		Ok(())
	}

	pub fn set_non_reserved_mode(&self, mode: &NonReservedPeerMode, io: &IoContext<NetworkIoMessage>) {
		let mut info = self.info.write();

		if &info.config.non_reserved_mode != mode {
			info.config.non_reserved_mode = mode.clone();
			drop(info);
			if let NonReservedPeerMode::Deny = mode {
				// disconnect all non-reserved peers here.
				let reserved: HashSet<NodeId> = self.reserved_nodes.read().clone();
				let mut to_kill = Vec::new();
				for e in self.sessions.read().iter() {
					let mut s = e.lock();
					{
						let id = s.id();
						if id.map_or(false, |id| reserved.contains(id)) {
							continue;
						}
					}

					s.disconnect(io, DisconnectReason::ClientQuit);
					to_kill.push(s.token());
				}
				for p in to_kill {
					trace!(target: "network", "Disconnecting on reserved-only mode: {}", p);
					self.kill_connection(p, io, false);
				}
			}
		}
	}

	pub fn remove_reserved_node(&self, id: &str) -> Result<(), Error> {
		let n = Node::from_str(id)?;
		self.reserved_nodes.write().remove(&n.id);

		Ok(())
	}

	pub fn external_url(&self) -> Option<String> {
		let info = self.info.read();
		info.public_endpoint.as_ref().map(|e| format!("{}", Node::new(*info.id(), e.clone())))
	}

	pub fn local_url(&self) -> String {
		let info = self.info.read();
		format!("{}", Node::new(*info.id(), info.local_endpoint.clone()))
	}

	pub fn stop(&self, io: &IoContext<NetworkIoMessage>) {
		self.stopping.store(true, AtomicOrdering::Release);
		let mut to_kill = Vec::new();
		for e in self.sessions.read().iter() {
			let mut s = e.lock();
			s.disconnect(io, DisconnectReason::ClientQuit);
			to_kill.push(s.token());
		}
		for p in to_kill {
			trace!(target: "network", "Disconnecting on shutdown: {}", p);
			self.kill_connection(p, io, true);
		}
		io.unregister_handler();
	}

	/// Get all connected peers.
	pub fn connected_peers(&self) -> Vec<PeerId> {
		let sessions = self.sessions.read();
		let sessions = &*sessions;

		let mut peers = Vec::with_capacity(sessions.count());
		for i in (0..MAX_SESSIONS).map(|x| x + FIRST_SESSION) {
			if sessions.get(i).is_some() {
				peers.push(i);
			}
		}
		peers
	}

	fn init_public_interface(&self, io: &IoContext<NetworkIoMessage>) -> Result<(), Error> {
		if self.info.read().public_endpoint.is_some() {
			return Ok(());
		}
		let local_endpoint = self.info.read().local_endpoint.clone();
		let public_address = self.info.read().config.public_address.clone();
		let allow_ips = self.info.read().config.ip_filter.clone();
		let public_endpoint = match public_address {
			None => {
				let public_address = select_public_address(local_endpoint.address.port());
				let public_endpoint = NodeEndpoint { address: public_address, udp_port: local_endpoint.udp_port };
				if self.info.read().config.nat_enabled {
					match map_external_address(&local_endpoint) {
						Some(endpoint) => {
							info!("NAT mapped to external address {}", endpoint.address);
							endpoint
						},
						None => public_endpoint
					}
				} else {
					public_endpoint
				}
			}
			Some(addr) => NodeEndpoint { address: addr, udp_port: local_endpoint.udp_port }
		};

		self.info.write().public_endpoint = Some(public_endpoint.clone());

		if let Some(url) = self.external_url() {
			io.message(NetworkIoMessage::NetworkStarted(url)).unwrap_or_else(|e| warn!("Error sending IO notification: {:?}", e));
		}

		// Initialize discovery.
		let discovery = {
			let info = self.info.read();
			if info.config.discovery_enabled && info.config.non_reserved_mode == NonReservedPeerMode::Accept {
				let mut udp_addr = local_endpoint.address.clone();
				udp_addr.set_port(local_endpoint.udp_port);
				Some(Discovery::new(&info.keys, udp_addr, public_endpoint, DISCOVERY, allow_ips))
			} else { None }
		};

		if let Some(mut discovery) = discovery {
			discovery.init_node_list(self.nodes.read().entries());
			discovery.add_node_list(self.nodes.read().entries());
			*self.discovery.lock() = Some(discovery);
			io.register_stream(DISCOVERY)?;
			io.register_timer(DISCOVERY_REFRESH, DISCOVERY_REFRESH_TIMEOUT)?;
			io.register_timer(DISCOVERY_ROUND, DISCOVERY_ROUND_TIMEOUT)?;
		}
		io.register_timer(NODE_TABLE, NODE_TABLE_TIMEOUT)?;
		io.register_stream(TCP_ACCEPT)?;
		Ok(())
	}

	fn maintain_network(&self, io: &IoContext<NetworkIoMessage>) {
		self.keep_alive(io);
		self.connect_peers(io);
	}

	fn have_session(&self, id: &NodeId) -> bool {
		self.sessions.read().iter().any(|e| e.lock().info.id == Some(id.clone()))
	}

	// returns (handshakes, egress, ingress)
	fn session_count(&self) -> (usize, usize, usize) {
		let mut handshakes = 0;
		let mut egress = 0;
		let mut ingress = 0;
		for s in self.sessions.read().iter() {
			match s.try_lock() {
				Some(ref s) if s.is_ready() && s.info.originated => egress += 1,
				Some(ref s) if s.is_ready() && !s.info.originated => ingress += 1,
				_ => handshakes +=1,
			}
		}
		(handshakes, egress, ingress)
	}

	fn connecting_to(&self, id: &NodeId) -> bool {
		self.sessions.read().iter().any(|e| e.lock().id() == Some(id))
	}

	fn keep_alive(&self, io: &IoContext<NetworkIoMessage>) {
		let mut to_kill = Vec::new();
		for e in self.sessions.read().iter() {
			let mut s = e.lock();
			if !s.keep_alive(io) {
				s.disconnect(io, DisconnectReason::PingTimeout);
				to_kill.push(s.token());
			}
		}
		for p in to_kill {
			trace!(target: "network", "Ping timeout: {}", p);
			self.kill_connection(p, io, true);
		}
	}

	fn connect_peers(&self, io: &IoContext<NetworkIoMessage>) {
		let (min_peers, mut pin, max_handshakes, allow_ips, self_id) = {
			let info = self.info.read();
			if info.capabilities.is_empty() {
				return;
			}
			let config = &info.config;

			(config.min_peers, config.non_reserved_mode == NonReservedPeerMode::Deny, config.max_handshakes as usize, config.ip_filter.clone(), info.id().clone())
		};

		let (handshake_count, egress_count, ingress_count) = self.session_count();
		let reserved_nodes = self.reserved_nodes.read();
		if egress_count + ingress_count >= min_peers as usize + reserved_nodes.len() {
			// check if all pinned nodes are connected.
			if reserved_nodes.iter().all(|n| self.have_session(n) && self.connecting_to(n)) {
				return;
			}

			// if not, only attempt connect to reserved peers
			pin = true;
		}

		// allow 16 slots for incoming connections
		if handshake_count >= max_handshakes {
			return;
		}

		// iterate over all nodes, reserved ones coming first.
		// if we are pinned to only reserved nodes, ignore all others.
		let nodes = reserved_nodes.iter().cloned().chain(if !pin {
			self.nodes.read().nodes(&allow_ips)
		} else {
			Vec::new()
		});

		let max_handshakes_per_round = max_handshakes / 2;
		let mut started: usize = 0;
		for id in nodes.filter(|id|
				!self.have_session(id) &&
				!self.connecting_to(id) &&
				*id != self_id &&
				self.filter.as_ref().map_or(true, |f| f.connection_allowed(&self_id, &id, ConnectionDirection::Outbound))
			).take(min(max_handshakes_per_round, max_handshakes - handshake_count)) {
			self.connect_peer(&id, io);
			started += 1;
		}
		debug!(target: "network", "Connecting peers: {} sessions, {} pending + {} started", egress_count + ingress_count, handshake_count, started);
	}

	fn connect_peer(&self, id: &NodeId, io: &IoContext<NetworkIoMessage>) {
		if self.have_session(id) {
			trace!(target: "network", "Aborted connect. Node already connected.");
			return;
		}
		if self.connecting_to(id) {
			trace!(target: "network", "Aborted connect. Node already connecting.");
			return;
		}

		let socket = {
			let address = {
				let mut nodes = self.nodes.write();
				if let Some(node) = nodes.get_mut(id) {
					node.endpoint.address
				} else {
					debug!(target: "network", "Connection to expired node aborted");
					return;
				}
			};
			match TcpStream::connect(&address) {
				Ok(socket) => {
					trace!(target: "network", "{}: Connecting to {:?}", id, address);
					socket
				},
				Err(e) => {
					debug!(target: "network", "{}: Can't connect to address {:?}: {:?}", id, address, e);
					self.nodes.write().note_failure(&id);
					return;
				}
			}
		};

		if let Err(e) = self.create_connection(socket, Some(id), io) {
			debug!(target: "network", "Can't create connection: {:?}", e);
		}
	}

	fn create_connection(&self, socket: TcpStream, id: Option<&NodeId>, io: &IoContext<NetworkIoMessage>) -> Result<(), Error> {
		let nonce = self.info.write().next_nonce();
		let mut sessions = self.sessions.write();

		let token = sessions.insert_with_opt(|token| {
			trace!(target: "network", "{}: Initiating session {:?}", token, id);
			match Session::new(io, socket, token, id, &nonce, &self.info.read()) {
				Ok(s) => Some(Arc::new(Mutex::new(s))),
				Err(e) => {
					debug!(target: "network", "Session create error: {:?}", e);
					None
				}
			}
		});

		match token {
			Some(t) => io.register_stream(t).map(|_| ()).map_err(Into::into),
			None => {
				debug!(target: "network", "Max sessions reached");
				Ok(())
			}
		}
	}

	fn accept(&self, io: &IoContext<NetworkIoMessage>) {
		trace!(target: "network", "Accepting incoming connection");
		loop {
			let socket = match self.tcp_listener.lock().accept() {
				Ok((sock, _addr)) => sock,
				Err(e) => {
					if e.kind() != io::ErrorKind::WouldBlock {
						debug!(target: "network", "Error accepting connection: {:?}", e);
					}
					break
				},
			};
			if let Err(e) = self.create_connection(socket, None, io) {
				debug!(target: "network", "Can't accept connection: {:?}", e);
			}
		}
	}

	fn session_writable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage>) {
		let session = { self.sessions.read().get(token).cloned() };

		if let Some(session) = session {
			let mut s = session.lock();
			if let Err(e) = s.writable(io, &self.info.read()) {
				trace!(target: "network", "Session write error: {}: {:?}", token, e);
			}
			if s.done() {
				io.deregister_stream(token).unwrap_or_else(|e| debug!("Error deregistering stream: {:?}", e));
			}
		}
	}

	fn connection_closed(&self, token: TimerToken, io: &IoContext<NetworkIoMessage>) {
		trace!(target: "network", "Connection closed: {}", token);
		self.kill_connection(token, io, true);
	}

	fn session_readable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage>) {
		let mut ready_data: Vec<ProtocolId> = Vec::new();
		let mut packet_data: Vec<(ProtocolId, PacketId, Vec<u8>)> = Vec::new();
		let mut kill = false;
		let session = { self.sessions.read().get(token).cloned() };
		let mut ready_id = None;
		if let Some(session) = session.clone() {
			{
				loop {
					let session_result = session.lock().readable(io, &self.info.read());
					match session_result {
						Err(e) => {
							let s = session.lock();
							trace!(target: "network", "Session read error: {}:{:?} ({:?}) {:?}", token, s.id(), s.remote_addr(), e);
							match *e.kind() {
								ErrorKind::Disconnect(DisconnectReason::IncompatibleProtocol) | ErrorKind::Disconnect(DisconnectReason::UselessPeer) => {
									if let Some(id) = s.id() {
										if !self.reserved_nodes.read().contains(id) {
											let mut nodes = self.nodes.write();
											nodes.note_failure(&id);
											nodes.mark_as_useless(id);
										}
									}
								},
								_ => {},
							}
							kill = true;
							break;
						},
						Ok(SessionData::Ready) => {
							let (_, egress_count, ingress_count) = self.session_count();
							let mut s = session.lock();
							let (min_peers, mut max_peers, reserved_only, self_id) = {
								let info = self.info.read();
								let mut max_peers = info.config.max_peers;
								for cap in s.info.capabilities.iter() {
									if let Some(num) = info.config.reserved_protocols.get(&cap.protocol) {
										max_peers += *num;
										break;
									}
								}
								(info.config.min_peers as usize, max_peers as usize, info.config.non_reserved_mode == NonReservedPeerMode::Deny, info.id().clone())
							};

							max_peers = max(max_peers, min_peers);

							let id = s.id().expect("Ready session always has id").clone();

							// Check for the session limit.
							// Outgoing connections are allowed as long as their count is <= min_peers
							// Incoming connections are allowed to take all of the max_peers reserve, or at most half of the slots.
							let max_ingress = max(max_peers - min_peers, min_peers / 2);
							if reserved_only ||
								(s.info.originated && egress_count > min_peers) ||
								(!s.info.originated && ingress_count > max_ingress) {
								// only proceed if the connecting peer is reserved.
								if !self.reserved_nodes.read().contains(&id) {
									s.disconnect(io, DisconnectReason::TooManyPeers);
									kill = true;
									break;
								}
							}

							if !self.filter.as_ref().map_or(true, |f| f.connection_allowed(&self_id, &id, ConnectionDirection::Inbound)) {
								trace!(target: "network", "Inbound connection not allowed for {:?}", id);
								s.disconnect(io, DisconnectReason::UnexpectedIdentity);
								kill = true;
								break;
							}

							ready_id = Some(id);

							// Add it to the node table
							if !s.info.originated {
								if let Ok(address) = s.remote_addr() {
									// We can't know remote listening ports, so just assume defaults and hope for the best.
									let endpoint = NodeEndpoint { address: SocketAddr::new(address.ip(), DEFAULT_PORT), udp_port: DEFAULT_PORT };
									let entry = NodeEntry { id: id, endpoint: endpoint };
									let mut nodes = self.nodes.write();
									if !nodes.contains(&entry.id) {
										nodes.add_node(Node::new(entry.id, entry.endpoint.clone()));
										let mut discovery = self.discovery.lock();
										if let Some(ref mut discovery) = *discovery {
											discovery.add_node(entry);
										}
									}
								}
							}

							// Note connection success
							self.nodes.write().note_success(&id);

							for (p, _) in self.handlers.read().iter() {
								if s.have_capability(*p) {
									ready_data.push(*p);
								}
							}
						},
						Ok(SessionData::Packet {
							data,
							protocol,
							packet_id,
						}) => {
							match self.handlers.read().get(&protocol) {
								None => { warn!(target: "network", "No handler found for protocol: {:?}", protocol) },
								Some(_) => packet_data.push((protocol, packet_id, data)),
							}
						},
						Ok(SessionData::Continue) => (),
						Ok(SessionData::None) => break,
					}
				}
			}

			if kill {
				self.kill_connection(token, io, true);
			}

			let handlers = self.handlers.read();
			if !ready_data.is_empty() {
				let duplicate = self.sessions.read().iter().any(|e| {
					let session = e.lock();
					session.token() != token && session.info.id == ready_id
				});
				if duplicate {
					trace!(target: "network", "Rejected duplicate connection: {}", token);
					session.lock().disconnect(io, DisconnectReason::DuplicatePeer);
					self.kill_connection(token, io, false);
					return;
				}
				for p in ready_data {
					let reserved = self.reserved_nodes.read();
					if let Some(h) = handlers.get(&p).clone() {
						h.connected(&NetworkContext::new(io, p, Some(session.clone()), self.sessions.clone(), &reserved), &token);
						// accumulate pending packets.
						let mut session = session.lock();
						packet_data.extend(session.mark_connected(p));
					}
				}
			}

			for (p, packet_id, data) in packet_data {
				let reserved = self.reserved_nodes.read();
				if let Some(h) = handlers.get(&p).clone() {
					h.read(&NetworkContext::new(io, p, Some(session.clone()), self.sessions.clone(), &reserved), &token, packet_id, &data);
				}
			}
		}
	}

	fn connection_timeout(&self, token: StreamToken, io: &IoContext<NetworkIoMessage>) {
		trace!(target: "network", "Connection timeout: {}", token);
		self.kill_connection(token, io, true)
	}

	fn kill_connection(&self, token: StreamToken, io: &IoContext<NetworkIoMessage>, remote: bool) {
		let mut to_disconnect: Vec<ProtocolId> = Vec::new();
		let mut failure_id = None;
		let mut deregister = false;
		let mut expired_session = None;
		if let FIRST_SESSION ... LAST_SESSION = token {
			let sessions = self.sessions.read();
			if let Some(session) = sessions.get(token).cloned() {
				expired_session = Some(session.clone());
				let mut s = session.lock();
				if !s.expired() {
					if s.is_ready() {
						for (p, _) in self.handlers.read().iter() {
							if s.have_capability(*p)  {
								to_disconnect.push(*p);
							}
						}
					}
					s.set_expired();
					failure_id = s.id().cloned();
				}
				deregister = remote || s.done();
			}
		}
		if let Some(id) = failure_id {
			if remote {
				self.nodes.write().note_failure(&id);
			}
		}
		for p in to_disconnect {
			let reserved = self.reserved_nodes.read();
			if let Some(h) = self.handlers.read().get(&p).clone() {
				h.disconnected(&NetworkContext::new(io, p, expired_session.clone(), self.sessions.clone(), &reserved), &token);
			}
		}
		if deregister {
			io.deregister_stream(token).unwrap_or_else(|e| debug!("Error deregistering stream: {:?}", e));
		}
	}

	fn update_nodes(&self, _io: &IoContext<NetworkIoMessage>, node_changes: TableUpdates) {
		let mut to_remove: Vec<PeerId> = Vec::new();
		{
			let sessions = self.sessions.read();
			for c in sessions.iter() {
				let s = c.lock();
				if let Some(id) = s.id() {
					if node_changes.removed.contains(id) {
						to_remove.push(s.token());
					}
				}
			}
		}
		for i in to_remove {
			trace!(target: "network", "Removed from node table: {}", i);
		}
		self.nodes.write().update(node_changes, &*self.reserved_nodes.read());
	}

	pub fn with_context<F>(&self, protocol: ProtocolId, io: &IoContext<NetworkIoMessage>, action: F) where F: FnOnce(&NetworkContextTrait) {
		let reserved = { self.reserved_nodes.read() };

		let context = NetworkContext::new(io, protocol, None, self.sessions.clone(), &reserved);
		action(&context);
	}

	pub fn with_context_eval<F, T>(&self, protocol: ProtocolId, io: &IoContext<NetworkIoMessage>, action: F) -> T where F: FnOnce(&NetworkContextTrait) -> T {
		let reserved = { self.reserved_nodes.read() };

		let context = NetworkContext::new(io, protocol, None, self.sessions.clone(), &reserved);
		action(&context)
	}
}

impl IoHandler<NetworkIoMessage> for Host {
	/// Initialize networking
	fn initialize(&self, io: &IoContext<NetworkIoMessage>) {
		io.register_timer(IDLE, MAINTENANCE_TIMEOUT).expect("Error registering Network idle timer");
		io.message(NetworkIoMessage::InitPublicInterface).unwrap_or_else(|e| warn!("Error sending IO notification: {:?}", e));
		self.maintain_network(io)
	}

	fn stream_hup(&self, io: &IoContext<NetworkIoMessage>, stream: StreamToken) {
		trace!(target: "network", "Hup: {}", stream);
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.connection_closed(stream, io),
			_ => warn!(target: "network", "Unexpected hup"),
		};
	}

	fn stream_readable(&self, io: &IoContext<NetworkIoMessage>, stream: StreamToken) {
		if self.stopping.load(AtomicOrdering::Acquire) {
			return;
		}
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.session_readable(stream, io),
			DISCOVERY => {
				let node_changes = { self.discovery.lock().as_mut().map_or(None, |d| d.readable(io)) };
				if let Some(node_changes) = node_changes {
					self.update_nodes(io, node_changes);
				}
			},
			TCP_ACCEPT => self.accept(io),
			_ => panic!("Received unknown readable token"),
		}
	}

	fn stream_writable(&self, io: &IoContext<NetworkIoMessage>, stream: StreamToken) {
		if self.stopping.load(AtomicOrdering::Acquire) {
			return;
		}
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.session_writable(stream, io),
			DISCOVERY => {
				self.discovery.lock().as_mut().map(|d| d.writable(io));
			}
			_ => panic!("Received unknown writable token"),
		}
	}

	fn timeout(&self, io: &IoContext<NetworkIoMessage>, token: TimerToken) {
		if self.stopping.load(AtomicOrdering::Acquire) {
			return;
		}
		match token {
			IDLE => self.maintain_network(io),
			FIRST_SESSION ... LAST_SESSION => self.connection_timeout(token, io),
			DISCOVERY_REFRESH => {
				self.discovery.lock().as_mut().map(|d| d.refresh());
				io.update_registration(DISCOVERY).unwrap_or_else(|e| debug!("Error updating discovery registration: {:?}", e));
			},
			DISCOVERY_ROUND => {
				let node_changes = { self.discovery.lock().as_mut().map_or(None, |d| d.round()) };
				if let Some(node_changes) = node_changes {
					self.update_nodes(io, node_changes);
				}
				io.update_registration(DISCOVERY).unwrap_or_else(|e| debug!("Error updating discovery registration: {:?}", e));
			},
			NODE_TABLE => {
				trace!(target: "network", "Refreshing node table");
				self.nodes.write().clear_useless();
				self.nodes.write().save();
			},
			_ => match self.timers.read().get(&token).cloned() {
				Some(timer) => match self.handlers.read().get(&timer.protocol).cloned() {
					None => { warn!(target: "network", "No handler found for protocol: {:?}", timer.protocol) },
					Some(h) => {
						let reserved = self.reserved_nodes.read();
						h.timeout(&NetworkContext::new(io, timer.protocol, None, self.sessions.clone(), &reserved), timer.token);
					}
				},
				None => { warn!("Unknown timer token: {}", token); } // timer is not registerd through us
			}
		}
	}

	fn message(&self, io: &IoContext<NetworkIoMessage>, message: &NetworkIoMessage) {
		if self.stopping.load(AtomicOrdering::Acquire) {
			return;
		}
		match *message {
			NetworkIoMessage::AddHandler {
				ref handler,
				ref protocol,
				ref versions,
			} => {
				let h = handler.clone();
				let reserved = self.reserved_nodes.read();
				h.initialize(
					&NetworkContext::new(io, *protocol, None, self.sessions.clone(), &reserved),
				);
				self.handlers.write().insert(*protocol, h);
				let mut info = self.info.write();
				for &(version, packet_count) in versions {
					info.capabilities.push(CapabilityInfo {
						protocol: *protocol,
						version,
						packet_count,
					});
				}
			},
			NetworkIoMessage::AddTimer {
				ref protocol,
				ref delay,
				ref token,
			} => {
				let handler_token = {
					let mut timer_counter = self.timer_counter.write();
					let counter = &mut *timer_counter;
					let handler_token = *counter;
					*counter += 1;
					handler_token
				};
				self.timers.write().insert(handler_token, ProtocolTimer { protocol: *protocol, token: *token });
				io.register_timer(handler_token, *delay).unwrap_or_else(|e| debug!("Error registering timer {}: {:?}", token, e));
			},
			NetworkIoMessage::Disconnect(ref peer) => {
				let session = { self.sessions.read().get(*peer).cloned() };
				if let Some(session) = session {
					session.lock().disconnect(io, DisconnectReason::DisconnectRequested);
				}
				trace!(target: "network", "Disconnect requested {}", peer);
				self.kill_connection(*peer, io, false);
			},
			NetworkIoMessage::DisablePeer(ref peer) => {
				let session = { self.sessions.read().get(*peer).cloned() };
				if let Some(session) = session {
					session.lock().disconnect(io, DisconnectReason::DisconnectRequested);
					if let Some(id) = session.lock().id() {
						let mut nodes = self.nodes.write();
						nodes.note_failure(&id);
						nodes.mark_as_useless(id);
					}
				}
				trace!(target: "network", "Disabling peer {}", peer);
				self.kill_connection(*peer, io, false);
			},
			NetworkIoMessage::InitPublicInterface =>
				self.init_public_interface(io).unwrap_or_else(|e| warn!("Error initializing public interface: {:?}", e)),
			_ => {}	// ignore others.
		}
	}

	fn register_stream(&self, stream: StreamToken, reg: Token, event_loop: &mut EventLoop<IoManager<NetworkIoMessage>>) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => {
				let session = { self.sessions.read().get(stream).cloned() };
				if let Some(session) = session {
					session.lock().register_socket(reg, event_loop).expect("Error registering socket");
				}
			}
			DISCOVERY => self.discovery.lock().as_ref().and_then(|d| d.register_socket(event_loop).ok()).expect("Error registering discovery socket"),
			TCP_ACCEPT => event_loop.register(&*self.tcp_listener.lock(), Token(TCP_ACCEPT), Ready::all(), PollOpt::edge()).expect("Error registering stream"),
			_ => warn!("Unexpected stream registration")
		}
	}

	fn deregister_stream(&self, stream: StreamToken, event_loop: &mut EventLoop<IoManager<NetworkIoMessage>>) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => {
				let mut connections = self.sessions.write();
				if let Some(connection) = connections.get(stream).cloned() {
					let c = connection.lock();
					if c.expired() { // make sure it is the same connection that the event was generated for
						c.deregister_socket(event_loop).expect("Error deregistering socket");
						connections.remove(stream);
					}
				}
			}
			DISCOVERY => (),
			_ => warn!("Unexpected stream deregistration")
		}
	}

	fn update_stream(&self, stream: StreamToken, reg: Token, event_loop: &mut EventLoop<IoManager<NetworkIoMessage>>) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => {
				let connection = { self.sessions.read().get(stream).cloned() };
				if let Some(connection) = connection {
					connection.lock().update_socket(reg, event_loop).expect("Error updating socket");
				}
			}
			DISCOVERY => self.discovery.lock().as_ref().and_then(|d| d.update_registration(event_loop).ok()).expect("Error reregistering discovery socket"),
			TCP_ACCEPT => event_loop.reregister(&*self.tcp_listener.lock(), Token(TCP_ACCEPT), Ready::all(), PollOpt::edge()).expect("Error reregistering stream"),
			_ => warn!("Unexpected stream update")
		}
	}
}

fn save_key(path: &Path, key: &Secret) {
	let mut path_buf = PathBuf::from(path);
	if let Err(e) = fs::create_dir_all(path_buf.as_path()) {
		warn!("Error creating key directory: {:?}", e);
		return;
	};
	path_buf.push("key");
	let path = path_buf.as_path();
	let mut file = match fs::File::create(&path) {
		Ok(file) => file,
		Err(e) => {
			warn!("Error creating key file: {:?}", e);
			return;
		}
	};
	if let Err(e) = restrict_permissions_owner(path, true, false) {
		warn!(target: "network", "Failed to modify permissions of the file ({})", e);
	}
	if let Err(e) = file.write(&key.hex().into_bytes()[2..]) {
		warn!("Error writing key file: {:?}", e);
	}
}

fn load_key(path: &Path) -> Option<Secret> {
	let mut path_buf = PathBuf::from(path);
	path_buf.push("key");
	let mut file = match fs::File::open(path_buf.as_path()) {
		Ok(file) => file,
		Err(e) => {
			debug!("Error opening key file: {:?}", e);
			return None;
		}
	};
	let mut buf = String::new();
	match file.read_to_string(&mut buf) {
		Ok(_) => {},
		Err(e) => {
			warn!("Error reading key file: {:?}", e);
			return None;
		}
	}
	match Secret::from_str(&buf) {
		Ok(key) => Some(key),
		Err(e) => {
			warn!("Error parsing key file: {:?}", e);
			None
		}
	}
}

#[test]
fn key_save_load() {
	use tempdir::TempDir;

	let tempdir = TempDir::new("").unwrap();
	let key = H256::random().into();
	save_key(tempdir.path(), &key);
	let r = load_key(tempdir.path());
	assert_eq!(key, r.unwrap());
}


#[test]
fn host_client_url() {
	let mut config = NetworkConfiguration::new_local();
	let key = "6f7b0d801bc7b5ce7bbd930b84fd0369b3eb25d09be58d64ba811091046f3aa2".parse().unwrap();
	config.use_secret = Some(key);
	let host: Host = Host::new(config, None).unwrap();
	assert!(host.local_url().starts_with("enode://101b3ef5a4ea7a1c7928e24c4c75fd053c235d7b80c22ae5c03d145d0ac7396e2a4ffff9adee3133a7b05044a5cee08115fd65145e5165d646bde371010d803c@"));
}
