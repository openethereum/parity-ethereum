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

use std::net::{SocketAddr};
use std::collections::{HashMap};
use std::hash::{Hasher};
use std::str::{FromStr};
use std::sync::*;
use std::ops::*;
use std::cmp::min;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::default::Default;
use std::fs;
use mio::*;
use mio::tcp::*;
use hash::*;
use misc::version;
use crypto::*;
use sha3::Hashable;
use rlp::*;
use network::handshake::Handshake;
use network::session::{Session, SessionData};
use error::*;
use io::*;
use network::{NetworkProtocolHandler, PROTOCOL_VERSION};
use network::node_table::*;
use network::stats::NetworkStats;
use network::error::{NetworkError, DisconnectReason};
use network::discovery::{Discovery, TableUpdates, NodeEntry};
use network::ip_utils::{map_external_address, select_public_address};

type Slab<T> = ::slab::Slab<T, usize>;

const _DEFAULT_PORT: u16 = 30304;
const MAX_SESSIONS: usize = 1024;
const MAX_HANDSHAKES: usize = 80;
const MAX_HANDSHAKES_PER_ROUND: usize = 32;
const MAINTENANCE_TIMEOUT: u64 = 1000;

#[derive(Debug)]
/// Network service configuration
pub struct NetworkConfiguration {
	/// Directory path to store network configuration. None means nothing will be saved
	pub config_path: Option<String>,
	/// IP address to listen for incoming connections. Listen to all connections by default
	pub listen_address: Option<SocketAddr>,
	/// IP address to advertise. Detected automatically if none.
	pub public_address: Option<SocketAddr>,
	/// Port for UDP connections, same as TCP by default
	pub udp_port: Option<u16>,
	/// Enable NAT configuration
	pub nat_enabled: bool,
	/// Enable discovery
	pub discovery_enabled: bool,
	/// Pin to boot nodes only
	pub pin: bool,
	/// List of initial node addresses
	pub boot_nodes: Vec<String>,
	/// Use provided node key instead of default
	pub use_secret: Option<Secret>,
	/// Number of connected peers to maintain
	pub ideal_peers: u32,
}

impl Default for NetworkConfiguration {
	fn default() -> Self {
		NetworkConfiguration::new()
	}
}

impl NetworkConfiguration {
	/// Create a new instance of default settings.
	pub fn new() -> Self {
		NetworkConfiguration {
			config_path: None,
			listen_address: None,
			public_address: None,
			udp_port: None,
			nat_enabled: true,
			discovery_enabled: true,
			pin: false,
			boot_nodes: Vec::new(),
			use_secret: None,
			ideal_peers: 25,
		}
	}

	/// Create new default configuration with sepcified listen port.
	pub fn new_with_port(port: u16) -> NetworkConfiguration {
		let mut config = NetworkConfiguration::new();
		config.listen_address = Some(SocketAddr::from_str(&format!("0.0.0.0:{}", port)).unwrap());
		config
	}

	/// Create new default configuration for localhost-only connection with random port (usefull for testing)
	pub fn new_local() -> NetworkConfiguration {
		let mut config = NetworkConfiguration::new();
		config.listen_address = Some(SocketAddr::from_str("127.0.0.1:0").unwrap());
		config.nat_enabled = false;
		config
	}
}

// Tokens
const TCP_ACCEPT: usize = LAST_HANDSHAKE + 1;
const IDLE: usize = LAST_HANDSHAKE + 2;
const DISCOVERY: usize = LAST_HANDSHAKE + 3;
const DISCOVERY_REFRESH: usize = LAST_HANDSHAKE + 4;
const DISCOVERY_ROUND: usize = LAST_HANDSHAKE + 5;
const INIT_PUBLIC: usize = LAST_HANDSHAKE + 6;
const NODE_TABLE: usize = LAST_HANDSHAKE + 7;
const FIRST_SESSION: usize = 0;
const LAST_SESSION: usize = FIRST_SESSION + MAX_SESSIONS - 1;
const FIRST_HANDSHAKE: usize = LAST_SESSION + 1;
const LAST_HANDSHAKE: usize = FIRST_HANDSHAKE + MAX_HANDSHAKES - 1;
const USER_TIMER: usize = LAST_HANDSHAKE + 256;

/// Protocol handler level packet id
pub type PacketId = u8;
/// Protocol / handler id
pub type ProtocolId = &'static str;

/// Messages used to communitate with the event loop from other threads.
#[derive(Clone)]
pub enum NetworkIoMessage<Message> where Message: Send + Sync + Clone {
	/// Register a new protocol handler.
	AddHandler {
		/// Handler shared instance.
		handler: Arc<NetworkProtocolHandler<Message> + Sync>,
		/// Protocol Id.
		protocol: ProtocolId,
		/// Supported protocol versions.
		versions: Vec<u8>,
	},
	/// Register a new protocol timer
	AddTimer {
		/// Protocol Id.
		protocol: ProtocolId,
		/// Timer token.
		token: TimerToken,
		/// Timer delay in milliseconds.
		delay: u64,
	},
	/// Disconnect a peer.
	Disconnect(PeerId),
	/// Disconnect and temporary disable peer.
	DisablePeer(PeerId),
	/// User message
	User(Message),
}

/// Local (temporary) peer session ID.
pub type PeerId = usize;

#[derive(Debug, PartialEq, Eq)]
/// Protocol info
pub struct CapabilityInfo {
	pub protocol: ProtocolId,
	pub version: u8,
	/// Total number of packet IDs this protocol support.
	pub packet_count: u8,
}

impl Encodable for CapabilityInfo {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.protocol);
		s.append(&self.version);
	}
}

/// IO access point. This is passed to all IO handlers and provides an interface to the IO subsystem.
pub struct NetworkContext<'s, Message> where Message: Send + Sync + Clone + 'static, 's {
	io: &'s IoContext<NetworkIoMessage<Message>>,
	protocol: ProtocolId,
	sessions: Arc<RwLock<Slab<SharedSession>>>,
	session: Option<SharedSession>,
	session_id: Option<StreamToken>,
}

impl<'s, Message> NetworkContext<'s, Message> where Message: Send + Sync + Clone + 'static, {
	/// Create a new network IO access point. Takes references to all the data that can be updated within the IO handler.
	fn new(io: &'s IoContext<NetworkIoMessage<Message>>,
		protocol: ProtocolId,
		session: Option<SharedSession>, sessions: Arc<RwLock<Slab<SharedSession>>>) -> NetworkContext<'s, Message> {
		let id = session.as_ref().map(|s| s.lock().unwrap().token());
		NetworkContext {
			io: io,
			protocol: protocol,
			session_id: id,
			session: session,
			sessions: sessions,
		}
	}

	fn resolve_session(&self, peer: PeerId) -> Option<SharedSession> {
		match self.session_id {
			Some(id) if id == peer => self.session.clone(),
			_ => self.sessions.read().unwrap().get(peer).cloned(),
		}
	}

	/// Send a packet over the network to another peer.
	pub fn send(&self, peer: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		let session = self.resolve_session(peer);
		if let Some(session) = session {
			try!(session.lock().unwrap().deref_mut().send_packet(self.protocol, packet_id as u8, &data));
			try!(self.io.update_registration(peer));
		} else  {
			trace!(target: "network", "Send: Peer no longer exist")
		}
		Ok(())
	}

	/// Respond to a current network message. Panics if no there is no packet in the context. If the session is expired returns nothing.
	pub fn respond(&self, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		assert!(self.session.is_some(), "Respond called without network context");
		self.send(self.session_id.unwrap(), packet_id, data)
	}

	/// Send an IO message
	pub fn message(&self, msg: Message) {
		self.io.message(NetworkIoMessage::User(msg));
	}

	/// Disable current protocol capability for given peer. If no capabilities left peer gets disconnected.
	pub fn disable_peer(&self, peer: PeerId) {
		//TODO: remove capability, disconnect if no capabilities left
		self.io.message(NetworkIoMessage::DisablePeer(peer));
	}

	/// Disconnect peer. Reconnect can be attempted later.
	pub fn disconnect_peer(&self, peer: PeerId) {
		self.io.message(NetworkIoMessage::Disconnect(peer));
	}

	/// Register a new IO timer. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer(&self, token: TimerToken, ms: u64) -> Result<(), UtilError> {
		self.io.message(NetworkIoMessage::AddTimer {
			token: token,
			delay: ms,
			protocol: self.protocol,
		});
		Ok(())
	}

	/// Returns peer identification string
	pub fn peer_info(&self, peer: PeerId) -> String {
		let session = self.resolve_session(peer);
		if let Some(session) = session {
			return session.lock().unwrap().info.client_version.clone()
		}
		"unknown".to_owned()
	}
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
	/// Client identifier
	pub client_version: String,
	/// Registered capabilities (handlers)
	pub capabilities: Vec<CapabilityInfo>,
	/// Local address + discovery port
	pub local_endpoint: NodeEndpoint,
	/// Public address + discovery port
	pub public_endpoint: Option<NodeEndpoint>,
}

impl HostInfo {
	/// Returns public key
	pub fn id(&self) -> &NodeId {
		self.keys.public()
	}

	/// Returns secret key
	pub fn secret(&self) -> &Secret {
		self.keys.secret()
	}

	/// Increments and returns connection nonce.
	pub fn next_nonce(&mut self) -> H256 {
		self.nonce = self.nonce.sha3();
		self.nonce.clone()
	}
}

type SharedSession = Arc<Mutex<Session>>;
type SharedHandshake = Arc<Mutex<Handshake>>;

#[derive(Copy, Clone)]
struct ProtocolTimer {
	pub protocol: ProtocolId,
	pub token: TimerToken, // Handler level token
}

/// Root IO handler. Manages protocol handlers, IO timers and network connections.
pub struct Host<Message> where Message: Send + Sync + Clone {
	pub info: RwLock<HostInfo>,
	tcp_listener: Mutex<TcpListener>,
	handshakes: Arc<RwLock<Slab<SharedHandshake>>>,
	sessions: Arc<RwLock<Slab<SharedSession>>>,
	discovery: Mutex<Option<Discovery>>,
	nodes: RwLock<NodeTable>,
	handlers: RwLock<HashMap<ProtocolId, Arc<NetworkProtocolHandler<Message>>>>,
	timers: RwLock<HashMap<TimerToken, ProtocolTimer>>,
	timer_counter: RwLock<usize>,
	stats: Arc<NetworkStats>,
	pinned_nodes: Vec<NodeId>,
}

impl<Message> Host<Message> where Message: Send + Sync + Clone {
	/// Create a new instance
	pub fn new(config: NetworkConfiguration) -> Result<Host<Message>, UtilError> {
		let mut listen_address = match config.listen_address {
			None => SocketAddr::from_str("0.0.0.0:30304").unwrap(),
			Some(addr) => addr,
		};

		let keys = if let Some(ref secret) = config.use_secret {
			KeyPair::from_secret(secret.clone()).unwrap()
		} else {
			config.config_path.clone().and_then(|ref p| load_key(&Path::new(&p)))
				.map_or_else(|| {
				let key = KeyPair::create().unwrap();
				if let Some(path) = config.config_path.clone() {
					save_key(&Path::new(&path), &key.secret());
				}
				key
			},
			|s| KeyPair::from_secret(s).expect("Error creating node secret key"))
		};
		let path = config.config_path.clone();
		// Setup the server socket
		let tcp_listener = try!(TcpListener::bind(&listen_address));
		listen_address = SocketAddr::new(listen_address.ip(), try!(tcp_listener.local_addr()).port());
		let udp_port = config.udp_port.unwrap_or(listen_address.port());
		let local_endpoint = NodeEndpoint { address: listen_address, udp_port: udp_port };

		let mut host = Host::<Message> {
			info: RwLock::new(HostInfo {
				keys: keys,
				config: config,
				nonce: H256::random(),
				protocol_version: PROTOCOL_VERSION,
				client_version: version(),
				capabilities: Vec::new(),
				public_endpoint: None,
				local_endpoint: local_endpoint,
			}),
			discovery: Mutex::new(None),
			tcp_listener: Mutex::new(tcp_listener),
			handshakes: Arc::new(RwLock::new(Slab::new_starting_at(FIRST_HANDSHAKE, MAX_HANDSHAKES))),
			sessions: Arc::new(RwLock::new(Slab::new_starting_at(FIRST_SESSION, MAX_SESSIONS))),
			nodes: RwLock::new(NodeTable::new(path)),
			handlers: RwLock::new(HashMap::new()),
			timers: RwLock::new(HashMap::new()),
			timer_counter: RwLock::new(USER_TIMER),
			stats: Arc::new(NetworkStats::default()),
			pinned_nodes: Vec::new(),
		};

		let boot_nodes = host.info.read().unwrap().config.boot_nodes.clone();
		for n in boot_nodes {
			host.add_node(&n);
		}
		Ok(host)
	}

	pub fn stats(&self) -> Arc<NetworkStats> {
		self.stats.clone()
	}

	pub fn add_node(&mut self, id: &str) {
		match Node::from_str(id) {
			Err(e) => { debug!(target: "network", "Could not add node {}: {:?}", id, e); },
			Ok(n) => {
				let entry = NodeEntry { endpoint: n.endpoint.clone(), id: n.id.clone() };
				self.pinned_nodes.push(n.id.clone());
				self.nodes.write().unwrap().add_node(n);
				if let Some(ref mut discovery) = *self.discovery.lock().unwrap().deref_mut() {
					discovery.add_node(entry);
				}
			}
		}
	}

	pub fn client_version(&self) -> String {
		self.info.read().unwrap().client_version.clone()
	}

	pub fn external_url(&self) -> Option<String> {
		self.info.read().unwrap().public_endpoint.as_ref().map(|e| format!("{}", Node::new(self.info.read().unwrap().id().clone(), e.clone())))
	}

	pub fn local_url(&self) -> String {
		let r = format!("{}", Node::new(self.info.read().unwrap().id().clone(), self.info.read().unwrap().local_endpoint.clone()));
		println!("{}", r);
		r
	}

	fn init_public_interface(&self, io: &IoContext<NetworkIoMessage<Message>>) -> Result<(), UtilError> {
		io.clear_timer(INIT_PUBLIC).unwrap();
		if self.info.read().unwrap().public_endpoint.is_some() {
			return Ok(());
		}
		let local_endpoint = self.info.read().unwrap().local_endpoint.clone();
		let public_address = self.info.read().unwrap().config.public_address.clone();
		let public_endpoint = match public_address {
			None => {
				let public_address = select_public_address(local_endpoint.address.port());
				let public_endpoint = NodeEndpoint { address: public_address, udp_port: local_endpoint.udp_port };
				if self.info.read().unwrap().config.nat_enabled {
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

		self.info.write().unwrap().public_endpoint = Some(public_endpoint.clone());
		info!("Public node URL: {}", self.external_url().unwrap());

		// Initialize discovery.
		let discovery = {
			let info = self.info.read().unwrap();
			if info.config.discovery_enabled && !info.config.pin {
				Some(Discovery::new(&info.keys, public_endpoint.address.clone(), public_endpoint, DISCOVERY))
			} else { None }
		};

		if let Some(mut discovery) = discovery {
			discovery.init_node_list(self.nodes.read().unwrap().unordered_entries());
			for n in self.nodes.read().unwrap().unordered_entries() {
				discovery.add_node(n.clone());
			}
			*self.discovery.lock().unwrap().deref_mut() = Some(discovery);
			io.register_stream(DISCOVERY).expect("Error registering UDP listener");
			io.register_timer(DISCOVERY_REFRESH, 7200).expect("Error registering discovery timer");
			io.register_timer(DISCOVERY_ROUND, 300).expect("Error registering discovery timer");
			io.register_timer(NODE_TABLE, 300_000).expect("Error registering node table timer");
		}
		try!(io.register_stream(TCP_ACCEPT));
		Ok(())
	}

	fn maintain_network(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		self.keep_alive(io);
		self.connect_peers(io);
	}

	fn have_session(&self, id: &NodeId) -> bool {
		self.sessions.read().unwrap().iter().any(|e| e.lock().unwrap().info.id.eq(&id))
	}

	fn session_count(&self) -> usize {
		self.sessions.read().unwrap().count()
	}

	fn connecting_to(&self, id: &NodeId) -> bool {
		self.handshakes.read().unwrap().iter().any(|e| e.lock().unwrap().id.eq(&id))
	}

	fn handshake_count(&self) -> usize {
		self.handshakes.read().unwrap().count()
	}

	fn keep_alive(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut to_kill = Vec::new();
		for e in self.sessions.write().unwrap().iter_mut() {
			let mut s = e.lock().unwrap();
			if !s.keep_alive(io) {
				s.disconnect(DisconnectReason::PingTimeout);
				to_kill.push(s.token());
			}
		}
		for p in to_kill {
			trace!(target: "network", "Ping timeout: {}", p);
			self.kill_connection(p, io, true);
		}
	}

	fn connect_peers(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		let ideal_peers = { self.info.read().unwrap().deref().config.ideal_peers };
		let pin = { self.info.read().unwrap().deref().config.pin };
		let session_count = self.session_count();
		if session_count >= ideal_peers as usize {
			return;
		}

		let handshake_count = self.handshake_count();
		// allow 16 slots for incoming connections
		let handshake_limit = MAX_HANDSHAKES - 16;
		if handshake_count >= handshake_limit {
			return;
		}

		let nodes = if pin { self.pinned_nodes.clone() } else { self.nodes.read().unwrap().nodes() };
		for id in nodes.iter().filter(|ref id| !self.have_session(id) && !self.connecting_to(id))
			.take(min(MAX_HANDSHAKES_PER_ROUND, handshake_limit - handshake_count)) {
			self.connect_peer(&id, io);
		}
		debug!(target: "network", "Connecting peers: {} sessions, {} pending", self.session_count(), self.handshake_count());
	}

	#[cfg_attr(feature="dev", allow(single_match))]
	fn connect_peer(&self, id: &NodeId, io: &IoContext<NetworkIoMessage<Message>>) {
		if self.have_session(id)
		{
			trace!(target: "network", "Aborted connect. Node already connected.");
			return;
		}
		if self.connecting_to(id) {
			trace!(target: "network", "Aborted connect. Node already connecting.");
			return;
		}

		let socket = {
			let address = {
				let mut nodes = self.nodes.write().unwrap();
				if let Some(node) = nodes.get_mut(id) {
					node.last_attempted = Some(::time::now());
					node.endpoint.address
				}
				else {
					debug!(target: "network", "Connection to expired node aborted");
					return;
				}
			};
			match TcpStream::connect(&address) {
				Ok(socket) => socket,
				Err(e) => {
					debug!(target: "network", "Can't connect to address {:?}: {:?}", address, e);
					return;
				}
			}
		};
		self.create_connection(socket, Some(id), io);
	}

	#[cfg_attr(feature="dev", allow(block_in_if_condition_stmt))]
	fn create_connection(&self, socket: TcpStream, id: Option<&NodeId>, io: &IoContext<NetworkIoMessage<Message>>) {
		let nonce = self.info.write().unwrap().next_nonce();
		let mut handshakes = self.handshakes.write().unwrap();
		if handshakes.insert_with(|token| {
			let mut handshake = Handshake::new(token, id, socket, &nonce, self.stats.clone()).expect("Can't create handshake");
			handshake.start(io, &self.info.read().unwrap(), id.is_some()).and_then(|_| io.register_stream(token)).unwrap_or_else (|e| {
				debug!(target: "network", "Handshake create error: {:?}", e);
			});
			Arc::new(Mutex::new(handshake))
		}).is_none() {
			debug!(target: "network", "Max handshakes reached");
		}
	}

	fn accept(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		trace!(target: "network", "Accepting incoming connection");
		loop {
			let socket = match self.tcp_listener.lock().unwrap().accept() {
				Ok(None) => break,
				Ok(Some((sock, _addr))) => sock,
				Err(e) => {
					warn!("Error accepting connection: {:?}", e);
					break
				},
			};
			self.create_connection(socket, None, io);
		}
		io.update_registration(TCP_ACCEPT).expect("Error registering TCP listener");
	}

	fn handshake_writable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let handshake = { self.handshakes.read().unwrap().get(token).cloned() };
		if let Some(handshake) = handshake {
			let mut h = handshake.lock().unwrap();
			if let Err(e) = h.writable(io, &self.info.read().unwrap()) {
				trace!(target: "network", "Handshake write error: {}: {:?}", token, e);
			}
		}
	}

	fn session_writable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let session = { self.sessions.read().unwrap().get(token).cloned() };
		if let Some(session) = session {
			let mut s = session.lock().unwrap();
			if let Err(e) = s.writable(io, &self.info.read().unwrap()) {
				trace!(target: "network", "Session write error: {}: {:?}", token, e);
			}
			if s.done() {
				io.deregister_stream(token).expect("Error deregistering stream");
			} else {
				io.update_registration(token).unwrap_or_else(|e| debug!(target: "network", "Session registration error: {:?}", e));
			}
		}
	}

	fn connection_closed(&self, token: TimerToken, io: &IoContext<NetworkIoMessage<Message>>) {
		trace!(target: "network", "Connection closed: {}", token);
		self.kill_connection(token, io, true);
	}

	fn handshake_readable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut create_session = false;
		let mut kill = false;
		let handshake = { self.handshakes.read().unwrap().get(token).cloned() };
		if let Some(handshake) = handshake {
			let mut h = handshake.lock().unwrap();
			if let Err(e) = h.readable(io, &self.info.read().unwrap()) {
				debug!(target: "network", "Handshake read error: {}: {:?}", token, e);
				kill = true;
			}
			if h.done() {
				create_session = true;
			}
		}
		if kill {
			self.kill_connection(token, io, true);
			return;
		} else if create_session {
			self.start_session(token, io);
			return;
		}
		io.update_registration(token).unwrap_or_else(|e| debug!(target: "network", "Token registration error: {:?}", e));
	}

	fn session_readable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut ready_data: Vec<ProtocolId> = Vec::new();
		let mut packet_data: Option<(ProtocolId, PacketId, Vec<u8>)> = None;
		let mut kill = false;
		let session = { self.sessions.read().unwrap().get(token).cloned() };
		if let Some(session) = session.clone() {
			let mut s = session.lock().unwrap();
			match s.readable(io, &self.info.read().unwrap()) {
				Err(e) => {
					trace!(target: "network", "Session read error: {}:{} ({:?}) {:?}", token, s.id(), s.remote_addr(), e);
					match e {
						UtilError::Network(NetworkError::Disconnect(DisconnectReason::UselessPeer)) |
						UtilError::Network(NetworkError::Disconnect(DisconnectReason::IncompatibleProtocol)) => {
							self.nodes.write().unwrap().mark_as_useless(s.id());
						}
						_ => (),
					}
					kill = true;
				},
				Ok(SessionData::Ready) => {
					for (p, _) in self.handlers.read().unwrap().iter() {
						if s.have_capability(p)  {
							ready_data.push(p);
						}
					}
				},
				Ok(SessionData::Packet {
					data,
					protocol,
					packet_id,
				}) => {
					match self.handlers.read().unwrap().get(protocol) {
						None => { warn!(target: "network", "No handler found for protocol: {:?}", protocol) },
						Some(_) => packet_data = Some((protocol, packet_id, data)),
					}
				},
				Ok(SessionData::None) => {},
			}
		}
		if kill {
			self.kill_connection(token, io, true);
		}
		for p in ready_data {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.connected(&NetworkContext::new(io, p, session.clone(), self.sessions.clone()), &token);
		}
		if let Some((p, packet_id, data)) = packet_data {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.read(&NetworkContext::new(io, p, session.clone(), self.sessions.clone()), &token, packet_id, &data[1..]);
		}
		io.update_registration(token).unwrap_or_else(|e| debug!(target: "network", "Token registration error: {:?}", e));
	}

	fn start_session(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut handshakes = self.handshakes.write().unwrap();
		if handshakes.get(token).is_none() {
			return;
		}

		// turn a handshake into a session
		let mut sessions = self.sessions.write().unwrap();
		let mut h = handshakes.get_mut(token).unwrap().lock().unwrap();
		if h.expired {
			return;
		}
		io.deregister_stream(token).expect("Error deleting handshake registration");
		h.set_expired();
		let originated = h.originated;
		let mut session = match Session::new(&mut h, &self.info.read().unwrap()) {
			Ok(s) => s,
			Err(e) => {
				debug!(target: "network", "Session creation error: {:?}", e);
				return;
			}
		};
		if !originated {
			let session_count = sessions.count();
			let ideal_peers = { self.info.read().unwrap().deref().config.ideal_peers };
			if session_count >= ideal_peers as usize {
				session.disconnect(DisconnectReason::TooManyPeers);
				return;
			}
		}
		let result = sessions.insert_with(move |session_token| {
			session.set_token(session_token);
			io.register_stream(session_token).expect("Error creating session registration");
			self.stats.inc_sessions();
			trace!(target: "network", "Creating session {} -> {}:{} ({:?})", token, session_token, session.id(), session.remote_addr());
			if !originated {
				// Add it no node table
				if let Ok(address) = session.remote_addr() {
					let entry = NodeEntry { id: session.id().clone(), endpoint: NodeEndpoint { address: address, udp_port: address.port() } };
					self.nodes.write().unwrap().add_node(Node::new(entry.id.clone(), entry.endpoint.clone()));
					let mut discovery = self.discovery.lock().unwrap();
					if let Some(ref mut discovery) = *discovery.deref_mut() {
						discovery.add_node(entry);
					}
				}
			}
			Arc::new(Mutex::new(session))
		});
		if result.is_none() {
			warn!("Max sessions reached");
		}
	}

	fn connection_timeout(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		trace!(target: "network", "Connection timeout: {}", token);
		self.kill_connection(token, io, true)
	}

	fn kill_connection(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>, remote: bool) {
		let mut to_disconnect: Vec<ProtocolId> = Vec::new();
		let mut failure_id = None;
		let mut deregister = false;
		let mut expired_session = None;
		match token {
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => {
				let handshakes = self.handshakes.write().unwrap();
				if let Some(handshake) = handshakes.get(token).cloned() {
					let mut handshake = handshake.lock().unwrap();
					if !handshake.expired() {
						handshake.set_expired();
						failure_id = Some(handshake.id().clone());
						deregister = true;
					}
				}
			},
			FIRST_SESSION ... LAST_SESSION => {
				let sessions = self.sessions.write().unwrap();
				if let Some(session) = sessions.get(token).cloned() {
					expired_session = Some(session.clone());
					let mut s = session.lock().unwrap();
					if !s.expired() {
						if s.is_ready() {
							for (p, _) in self.handlers.read().unwrap().iter() {
								if s.have_capability(p)  {
									to_disconnect.push(p);
								}
							}
						}
						s.set_expired();
						failure_id = Some(s.id().clone());
					}
					deregister = remote || s.done();
				}
			},
			_ => {},
		}
		if let Some(id) = failure_id {
			if remote {
				self.nodes.write().unwrap().note_failure(&id);
			}
		}
		for p in to_disconnect {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.disconnected(&NetworkContext::new(io, p, expired_session.clone(), self.sessions.clone()), &token);
		}
		if deregister {
			io.deregister_stream(token).expect("Error deregistering stream");
		} else if expired_session.is_some() {
			io.update_registration(token).unwrap_or_else(|e| debug!(target: "network", "Connection registration error: {:?}", e));
		}
	}

	fn update_nodes(&self, io: &IoContext<NetworkIoMessage<Message>>, node_changes: TableUpdates) {
		let mut to_remove: Vec<PeerId> = Vec::new();
		{
			{
				let handshakes = self.handshakes.write().unwrap();
				for c in handshakes.iter() {
					let h = c.lock().unwrap();
					if node_changes.removed.contains(&h.id()) {
						to_remove.push(h.token());
					}
				}
			}
			{
				let sessions = self.sessions.write().unwrap();
				for c in sessions.iter() {
					let s = c.lock().unwrap();
					if node_changes.removed.contains(&s.id()) {
						to_remove.push(s.token());
					}
				}
			}
		}
		for i in to_remove {
			trace!(target: "network", "Removed from node table: {}", i);
			self.kill_connection(i, io, false);
		}
		self.nodes.write().unwrap().update(node_changes);
	}
}

impl<Message> IoHandler<NetworkIoMessage<Message>> for Host<Message> where Message: Send + Sync + Clone + 'static {
	/// Initialize networking
	fn initialize(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		io.register_timer(IDLE, MAINTENANCE_TIMEOUT).expect("Error registering Network idle timer");
		io.register_timer(INIT_PUBLIC, 0).expect("Error registering initialization timer");
		self.maintain_network(io)
	}

	fn stream_hup(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		trace!(target: "network", "Hup: {}", stream);
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.connection_closed(stream, io),
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => self.connection_closed(stream, io),
			_ => warn!(target: "network", "Unexpected hup"),
		};
	}

	fn stream_readable(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.session_readable(stream, io),
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => self.handshake_readable(stream, io),
			DISCOVERY => {
				let node_changes = { self.discovery.lock().unwrap().as_mut().unwrap().readable() };
				if let Some(node_changes) = node_changes {
					self.update_nodes(io, node_changes);
				}
				io.update_registration(DISCOVERY).expect("Error updating discovery registration");
			},
			TCP_ACCEPT => self.accept(io),
			_ => panic!("Received unknown readable token"),
		}
	}

	fn stream_writable(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.session_writable(stream, io),
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => self.handshake_writable(stream, io),
			DISCOVERY => {
				self.discovery.lock().unwrap().as_mut().unwrap().writable();
				io.update_registration(DISCOVERY).expect("Error updating discovery registration");
			}
			_ => panic!("Received unknown writable token"),
		}
	}

	fn timeout(&self, io: &IoContext<NetworkIoMessage<Message>>, token: TimerToken) {
		match token {
			IDLE => self.maintain_network(io),
			INIT_PUBLIC => self.init_public_interface(io).unwrap_or_else(|e| 
				warn!("Error initializing public interface: {:?}", e)),
			FIRST_SESSION ... LAST_SESSION => self.connection_timeout(token, io),
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => self.connection_timeout(token, io),
			DISCOVERY_REFRESH => {
				self.discovery.lock().unwrap().as_mut().unwrap().refresh();
				io.update_registration(DISCOVERY).expect("Error updating discovery registration");
			},
			DISCOVERY_ROUND => {
				let node_changes = { self.discovery.lock().unwrap().as_mut().unwrap().round() };
				if let Some(node_changes) = node_changes {
					self.update_nodes(io, node_changes);
				}
				io.update_registration(DISCOVERY).expect("Error updating discovery registration");
			},
			NODE_TABLE => {
				self.nodes.write().unwrap().clear_useless();
			},
			_ => match self.timers.read().unwrap().get(&token).cloned() {
				Some(timer) => match self.handlers.read().unwrap().get(timer.protocol).cloned() {
						None => { warn!(target: "network", "No handler found for protocol: {:?}", timer.protocol) },
						Some(h) => { h.timeout(&NetworkContext::new(io, timer.protocol, None, self.sessions.clone()), timer.token); }
				},
				None => { warn!("Unknown timer token: {}", token); } // timer is not registerd through us
			}
		}
	}

	fn message(&self, io: &IoContext<NetworkIoMessage<Message>>, message: &NetworkIoMessage<Message>) {
		match *message {
			NetworkIoMessage::AddHandler {
				ref handler,
				ref protocol,
				ref versions
			} => {
				let h = handler.clone();
				h.initialize(&NetworkContext::new(io, protocol, None, self.sessions.clone()));
				self.handlers.write().unwrap().insert(protocol, h);
				let mut info = self.info.write().unwrap();
				for v in versions {
					info.capabilities.push(CapabilityInfo { protocol: protocol, version: *v, packet_count:0 });
				}
			},
			NetworkIoMessage::AddTimer {
				ref protocol,
				ref delay,
				ref token,
			} => {
				let handler_token = {
					let mut timer_counter = self.timer_counter.write().unwrap();
					let counter = timer_counter.deref_mut();
					let handler_token = *counter;
					*counter += 1;
					handler_token
				};
				self.timers.write().unwrap().insert(handler_token, ProtocolTimer { protocol: protocol, token: *token });
				io.register_timer(handler_token, *delay).expect("Error registering timer");
			},
			NetworkIoMessage::Disconnect(ref peer) => {
				let session = { self.sessions.read().unwrap().get(*peer).cloned() };
				if let Some(session) = session {
					session.lock().unwrap().disconnect(DisconnectReason::DisconnectRequested);
				}
				trace!(target: "network", "Disconnect requested {}", peer);
				self.kill_connection(*peer, io, false);
			},
			NetworkIoMessage::DisablePeer(ref peer) => {
				let session = { self.sessions.read().unwrap().get(*peer).cloned() };
				if let Some(session) = session {
					session.lock().unwrap().disconnect(DisconnectReason::DisconnectRequested);
					self.nodes.write().unwrap().mark_as_useless(session.lock().unwrap().id());
				}
				trace!(target: "network", "Disabling peer {}", peer);
				self.kill_connection(*peer, io, false);
			},
			NetworkIoMessage::User(ref message) => {
				for (p, h) in self.handlers.read().unwrap().iter() {
					h.message(&NetworkContext::new(io, p, None, self.sessions.clone()), &message);
				}
			}
		}
	}

	fn register_stream(&self, stream: StreamToken, reg: Token, event_loop: &mut EventLoop<IoManager<NetworkIoMessage<Message>>>) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => {
				let session = { self.sessions.read().unwrap().get(stream).cloned() };
				if let Some(session) = session {
					session.lock().unwrap().register_socket(reg, event_loop).expect("Error registering socket");
				}
			}
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => {
				let connection = { self.handshakes.read().unwrap().get(stream).cloned() };
				if let Some(connection) = connection {
					connection.lock().unwrap().register_socket(reg, event_loop).expect("Error registering socket");
				}
			}
			DISCOVERY => self.discovery.lock().unwrap().as_ref().unwrap().register_socket(event_loop).expect("Error registering discovery socket"),
			TCP_ACCEPT => event_loop.register(self.tcp_listener.lock().unwrap().deref(), Token(TCP_ACCEPT), EventSet::all(), PollOpt::edge()).expect("Error registering stream"),
			_ => warn!("Unexpected stream registration")
		}
	}

	fn deregister_stream(&self, stream: StreamToken, event_loop: &mut EventLoop<IoManager<NetworkIoMessage<Message>>>) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => {
				let mut connections = self.sessions.write().unwrap();
				if let Some(connection) = connections.get(stream).cloned() {
					connection.lock().unwrap().deregister_socket(event_loop).expect("Error deregistering socket");
					connections.remove(stream);
				}
			}
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => {
				let mut connections = self.handshakes.write().unwrap();
				if let Some(connection) = connections.get(stream).cloned() {
					connection.lock().unwrap().deregister_socket(event_loop).expect("Error deregistering socket");
					connections.remove(stream);
				}
			}
			DISCOVERY => (),
			_ => warn!("Unexpected stream deregistration")
		}
	}

	fn update_stream(&self, stream: StreamToken, reg: Token, event_loop: &mut EventLoop<IoManager<NetworkIoMessage<Message>>>) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => {
				let connection = { self.sessions.read().unwrap().get(stream).cloned() };
				if let Some(connection) = connection {
					connection.lock().unwrap().update_socket(reg, event_loop).expect("Error updating socket");
				}
			}
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => {
				let connection = { self.handshakes.read().unwrap().get(stream).cloned() };
				if let Some(connection) = connection {
					connection.lock().unwrap().update_socket(reg, event_loop).expect("Error updating socket");
				}
			}
			DISCOVERY => self.discovery.lock().unwrap().as_ref().unwrap().update_registration(event_loop).expect("Error reregistering discovery socket"),
			TCP_ACCEPT => event_loop.reregister(self.tcp_listener.lock().unwrap().deref(), Token(TCP_ACCEPT), EventSet::all(), PollOpt::edge()).expect("Error reregistering stream"),
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
	let mut file = match fs::File::create(path_buf.as_path()) {
		Ok(file) => file,
		Err(e) => {
			warn!("Error creating key file: {:?}", e);
			return;
		}
	};
	if let Err(e) = file.write(&key.hex().into_bytes()) {
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
	use ::devtools::RandomTempPath;
	let temp_path = RandomTempPath::create_dir();
	let key = H256::random();
	save_key(temp_path.as_path(), &key);
	let r = load_key(temp_path.as_path());
	assert_eq!(key, r.unwrap());
}


#[test]
fn host_client_url() {
	let mut config = NetworkConfiguration::new();
	let key = h256_from_hex("6f7b0d801bc7b5ce7bbd930b84fd0369b3eb25d09be58d64ba811091046f3aa2");
	config.use_secret = Some(key);
	let host: Host<u32> = Host::new(config).unwrap();
	assert!(host.local_url().starts_with("enode://101b3ef5a4ea7a1c7928e24c4c75fd053c235d7b80c22ae5c03d145d0ac7396e2a4ffff9adee3133a7b05044a5cee08115fd65145e5165d646bde371010d803c@"));
}
