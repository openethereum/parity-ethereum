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
use std::fs;
use mio::*;
use mio::tcp::*;
use target_info::Target;
use hash::*;
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
use network::error::DisconnectReason;
use network::discovery::{Discovery, TableUpdates, NodeEntry};
use network::ip_utils::{map_external_address, select_public_address};

type Slab<T> = ::slab::Slab<T, usize>;

const _DEFAULT_PORT: u16 = 30304;
const MAX_SESSIONS: usize = 1024;
const MAX_HANDSHAKES: usize = 256;
const MAX_HANDSHAKES_PER_ROUND: usize = 64;
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

impl NetworkConfiguration {
	/// Create a new instance of default settings.
	pub fn new() -> NetworkConfiguration {
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
}

// Tokens
const TCP_ACCEPT: usize = LAST_HANDSHAKE + 1;
const IDLE: usize = LAST_HANDSHAKE + 2;
const DISCOVERY: usize = LAST_HANDSHAKE + 3;
const DISCOVERY_REFRESH: usize = LAST_HANDSHAKE + 4;
const DISCOVERY_ROUND: usize = LAST_HANDSHAKE + 5;
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
	/// Disconnect a peer
	Disconnect(PeerId),
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
	session: Option<StreamToken>,
}

impl<'s, Message> NetworkContext<'s, Message> where Message: Send + Sync + Clone + 'static, {
	/// Create a new network IO access point. Takes references to all the data that can be updated within the IO handler.
	fn new(io: &'s IoContext<NetworkIoMessage<Message>>, 
		protocol: ProtocolId, 
		session: Option<StreamToken>, sessions: Arc<RwLock<Slab<SharedSession>>>) -> NetworkContext<'s, Message> {
		NetworkContext {
			io: io,
			protocol: protocol,
			session: session,
			sessions: sessions,
		}
	}

	/// Send a packet over the network to another peer.
	pub fn send(&self, peer: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		let session = { self.sessions.read().unwrap().get(peer).cloned() };
		if let Some(session) = session {
			session.lock().unwrap().deref_mut().send_packet(self.protocol, packet_id as u8, &data).unwrap_or_else(|e| {
						warn!(target: "net", "Send error: {:?}", e);
					}); //TODO: don't copy vector data
			try!(self.io.update_registration(peer));
		} else  {
			trace!(target: "net", "Send: Peer no longer exist")
		}
		Ok(())
	}

	/// Respond to a current network message. Panics if no there is no packet in the context.
	pub fn respond(&self, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		match self.session {
			Some(session) => self.send(session, packet_id, data),
			None => {
				panic!("Respond: Session does not exist")
			}
		}
	}

	/// Disable current protocol capability for given peer. If no capabilities left peer gets disconnected.
	pub fn disable_peer(&self, peer: PeerId) {
		//TODO: remove capability, disconnect if no capabilities left
		self.disconnect_peer(peer);
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
		let session = { self.sessions.read().unwrap().get(peer).cloned() };
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
	/// TCP connection port.
	pub listen_port: u16,
	/// Registered capabilities (handlers)
	pub capabilities: Vec<CapabilityInfo>
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
	discovery: Option<Mutex<Discovery>>,
	nodes: RwLock<NodeTable>,
	handlers: RwLock<HashMap<ProtocolId, Arc<NetworkProtocolHandler<Message>>>>,
	timers: RwLock<HashMap<TimerToken, ProtocolTimer>>,
	timer_counter: RwLock<usize>,
	stats: Arc<NetworkStats>,
	public_endpoint: NodeEndpoint,
	pinned_nodes: Vec<NodeId>,
}

impl<Message> Host<Message> where Message: Send + Sync + Clone {
	/// Create a new instance
	pub fn new(config: NetworkConfiguration) -> Host<Message> {
		let listen_address = match config.listen_address {
			None => SocketAddr::from_str("0.0.0.0:30304").unwrap(),
			Some(addr) => addr,
		};

		let udp_port = config.udp_port.unwrap_or(listen_address.port());
		let public_endpoint = match config.public_address {
			None => {
				let public_address = select_public_address(listen_address.port());
				let local_endpoint = NodeEndpoint { address: public_address, udp_port: udp_port };
				if config.nat_enabled {
					match map_external_address(&local_endpoint) {
						Some(endpoint) => {
							info!("NAT Mappped to external address {}", endpoint.address);
							endpoint
						},
						None => local_endpoint
					}
				} else {
					local_endpoint
				}
			}
			Some(addr) => NodeEndpoint { address: addr, udp_port: udp_port }
		};

		// Setup the server socket
		let tcp_listener = TcpListener::bind(&listen_address).unwrap();
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
		let discovery = if config.discovery_enabled && !config.pin {
			Some(Discovery::new(&keys, listen_address.clone(), public_endpoint.clone(), DISCOVERY)) 
		} else { None };
		let path = config.config_path.clone();
		let mut host = Host::<Message> {
			info: RwLock::new(HostInfo {
				keys: keys,
				config: config,
				nonce: H256::random(),
				protocol_version: PROTOCOL_VERSION,
				client_version: format!("Parity/{}/{}-{}-{}", env!("CARGO_PKG_VERSION"), Target::arch(), Target::env(), Target::os()),
				listen_port: 0,
				capabilities: Vec::new(),
			}),
			discovery: discovery.map(Mutex::new),
			tcp_listener: Mutex::new(tcp_listener),
			handshakes: Arc::new(RwLock::new(Slab::new_starting_at(FIRST_HANDSHAKE, MAX_HANDSHAKES))),
			sessions: Arc::new(RwLock::new(Slab::new_starting_at(FIRST_SESSION, MAX_SESSIONS))),
			nodes: RwLock::new(NodeTable::new(path)),
			handlers: RwLock::new(HashMap::new()),
			timers: RwLock::new(HashMap::new()),
			timer_counter: RwLock::new(USER_TIMER),
			stats: Arc::new(NetworkStats::default()),
			public_endpoint: public_endpoint,
			pinned_nodes: Vec::new(),
		};
		let port = listen_address.port();
		host.info.write().unwrap().deref_mut().listen_port = port;

		let boot_nodes = host.info.read().unwrap().config.boot_nodes.clone();
		for n in boot_nodes {
			host.add_node(&n);
		}
		if let Some(ref mut discovery) = host.discovery {
			discovery.lock().unwrap().init_node_list(host.nodes.read().unwrap().unordered_entries());
		}
		host
	}

	pub fn stats(&self) -> Arc<NetworkStats> {
		self.stats.clone()
	}

	pub fn add_node(&mut self, id: &str) {
		match Node::from_str(id) {
			Err(e) => { warn!("Could not add node: {:?}", e); },
			Ok(n) => {
				let entry = NodeEntry { endpoint: n.endpoint.clone(), id: n.id.clone() };
				self.pinned_nodes.push(n.id.clone());
				self.nodes.write().unwrap().add_node(n);
				if let Some(ref mut discovery) = self.discovery {
					discovery.lock().unwrap().add_node(entry);
				}
			}
		}
	}

	pub fn client_version(&self) -> String {
		self.info.read().unwrap().client_version.clone()
	}

	pub fn client_url(&self) -> String {
		format!("{}", Node::new(self.info.read().unwrap().id().clone(), self.public_endpoint.clone()))
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
		debug!(target: "net", "Connecting peers: {} sessions, {} pending", self.session_count(), self.handshake_count());
	}

	#[allow(single_match)]
	fn connect_peer(&self, id: &NodeId, io: &IoContext<NetworkIoMessage<Message>>) {
		if self.have_session(id)
		{
			trace!("Aborted connect. Node already connected.");
			return;
		}
		if self.connecting_to(id) {
			trace!("Aborted connect. Node already connecting.");
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
					debug!("Connection to expired node aborted");
					return;
				}
			};
			match TcpStream::connect(&address) {
				Ok(socket) => socket,
				Err(e) => {
					warn!("Can't connect to node: {:?}", e);
					return;
				}
			}
		};
		self.create_connection(socket, Some(id), io);
	}

	#[allow(block_in_if_condition_stmt)]
	fn create_connection(&self, socket: TcpStream, id: Option<&NodeId>, io: &IoContext<NetworkIoMessage<Message>>) {
		let nonce = self.info.write().unwrap().next_nonce();
		let mut handshakes = self.handshakes.write().unwrap();
		if handshakes.insert_with(|token| {
			let mut handshake = Handshake::new(token, id, socket, &nonce, self.stats.clone()).expect("Can't create handshake");
			handshake.start(io, &self.info.read().unwrap(), id.is_some()).and_then(|_| io.register_stream(token)).unwrap_or_else (|e| {
				debug!(target: "net", "Handshake create error: {:?}", e);
			});
			Arc::new(Mutex::new(handshake))
		}).is_none() {
			debug!("Max handshakes reached");
		}
	}

	fn accept(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		trace!(target: "net", "accept");
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
		let mut create_session = false;
		let mut kill = false;
		let handshake = { self.handshakes.read().unwrap().get(token).cloned() };
		if let Some(handshake) = handshake {
			let mut h = handshake.lock().unwrap();
			if let Err(e) = h.writable(io, &self.info.read().unwrap()) {
				debug!(target: "net", "Handshake write error: {}:{:?}", token, e);
				kill = true;
			}
			if h.done() {
				create_session = true;
			}
		} 
		if kill {
			self.kill_connection(token, io, true); //TODO: mark connection as dead an check in kill_connection
			return;
		} else if create_session {
			self.start_session(token, io);
			io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
		}
	}

	fn session_writable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut kill = false;
		let session = { self.sessions.read().unwrap().get(token).cloned() };
		if let Some(session) = session {
			let mut s = session.lock().unwrap();
			if let Err(e) = s.writable(io, &self.info.read().unwrap()) {
				debug!(target: "net", "Session write error: {}:{:?}", token, e);
				kill = true;
			}
			io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
		} 
		if kill {
			self.kill_connection(token, io, true); //TODO: mark connection as dead an check in kill_connection
		}
	}

	fn connection_closed(&self, token: TimerToken, io: &IoContext<NetworkIoMessage<Message>>) {
		self.kill_connection(token, io, true);
	}

	fn handshake_readable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut create_session = false;
		let mut kill = false;
		let handshake = { self.handshakes.read().unwrap().get(token).cloned() };
		if let Some(handshake) = handshake {
			let mut h = handshake.lock().unwrap();
			if let Err(e) = h.readable(io, &self.info.read().unwrap()) {
				debug!(target: "net", "Handshake read error: {}:{:?}", token, e);
				kill = true;
			}
			if h.done() {
				create_session = true;
			}
		}
		if kill {
			self.kill_connection(token, io, true); //TODO: mark connection as dead an check in kill_connection
			return;
		} else if create_session {
			self.start_session(token, io);
			io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
		}
		io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Token registration error: {:?}", e));
	}

	fn session_readable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut ready_data: Vec<ProtocolId> = Vec::new();
		let mut packet_data: Option<(ProtocolId, PacketId, Vec<u8>)> = None;
		let mut kill = false;
		let session = { self.sessions.read().unwrap().get(token).cloned() };
		if let Some(session) = session {
			let mut s = session.lock().unwrap();
			match s.readable(io, &self.info.read().unwrap()) {
				Err(e) => {
					debug!(target: "net", "Session read error: {}:{:?}", token, e);
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
						None => { warn!(target: "net", "No handler found for protocol: {:?}", protocol) },
						Some(_) => packet_data = Some((protocol, packet_id, data)),
					}
				},
				Ok(SessionData::None) => {},
			}
		} 
		if kill {
			self.kill_connection(token, io, true); //TODO: mark connection as dead an check in kill_connection
		}
		for p in ready_data {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.connected(&NetworkContext::new(io, p, Some(token), self.sessions.clone()), &token);
		}
		if let Some((p, packet_id, data)) = packet_data {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.read(&NetworkContext::new(io, p, Some(token), self.sessions.clone()), &token, packet_id, &data[1..]);
		}
		io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Token registration error: {:?}", e));
	}

	fn start_session(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut handshakes = self.handshakes.write().unwrap();
		if handshakes.get(token).is_none() {
			return;
		}
		
		// turn a handshake into a session
		let mut sessions = self.sessions.write().unwrap();
		let mut h = handshakes.remove(token).unwrap();
		// wait for other threads to stop using it
		{
			while Arc::get_mut(&mut h).is_none() {
				h.lock().ok();
			}
		}
		let h = Arc::try_unwrap(h).ok().unwrap().into_inner().unwrap();
		let originated = h.originated;
		let mut session = match Session::new(h, &self.info.read().unwrap()) {
			Ok(s) => s,
			Err(e) => {
				debug!("Session creation error: {:?}", e);
				return;
			}
		};
		let result = sessions.insert_with(move |session_token| {
			session.set_token(session_token);
			io.update_registration(session_token).expect("Error updating session registration");
			self.stats.inc_sessions();
			if !originated {
				// Add it no node table
				if let Ok(address) = session.remote_addr() {
					let entry = NodeEntry { id: session.id().clone(), endpoint: NodeEndpoint { address: address, udp_port: address.port() } };
					self.nodes.write().unwrap().add_node(Node::new(entry.id.clone(), entry.endpoint.clone()));
					if let Some(ref discovery) = self.discovery {
						discovery.lock().unwrap().add_node(entry);
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
		self.kill_connection(token, io, true)
	}

	fn kill_connection(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>, remote: bool) {
		let mut to_disconnect: Vec<ProtocolId> = Vec::new();
		let mut failure_id = None;
		match token {
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => {
				let mut handshakes = self.handshakes.write().unwrap();
				if let Some(handshake) = handshakes.get(token).cloned() {
					failure_id = Some(handshake.lock().unwrap().id().clone());
					handshakes.remove(token);
				}
			},
			FIRST_SESSION ... LAST_SESSION => {
				let mut sessions = self.sessions.write().unwrap();
				if let Some(session) = sessions.get(token).cloned() {
					let s = session.lock().unwrap();
					if s.is_ready() {
						for (p, _) in self.handlers.read().unwrap().iter() {
							if s.have_capability(p)  {
								to_disconnect.push(p);
							}
						}
					}
					failure_id = Some(s.id().clone());
					sessions.remove(token);
				}
			},
			_ => {},
		}
		io.deregister_stream(token).expect("Error deregistering stream");
		if let Some(id) = failure_id {
			if remote {
				self.nodes.write().unwrap().note_failure(&id);
			}
		}
		for p in to_disconnect {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.disconnected(&NetworkContext::new(io, p, Some(token), self.sessions.clone()), &token);
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
			self.kill_connection(i, io, false);
		}
		self.nodes.write().unwrap().update(node_changes);
	}
}

impl<Message> IoHandler<NetworkIoMessage<Message>> for Host<Message> where Message: Send + Sync + Clone + 'static {
	/// Initialize networking
	fn initialize(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		io.register_stream(TCP_ACCEPT).expect("Error registering TCP listener");
		io.register_timer(IDLE, MAINTENANCE_TIMEOUT).expect("Error registering Network idle timer");
		if self.discovery.is_some() {
			io.register_stream(DISCOVERY).expect("Error registering UDP listener");
			io.register_timer(DISCOVERY_REFRESH, 7200).expect("Error registering discovery timer");
			io.register_timer(DISCOVERY_ROUND, 300).expect("Error registering discovery timer");
		}
	}

	fn stream_hup(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		trace!(target: "net", "Hup: {}", stream);
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.connection_closed(stream, io),
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => self.connection_closed(stream, io),
			_ => warn!(target: "net", "Unexpected hup"),
		};
	}

	fn stream_readable(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		match stream {
			FIRST_SESSION ... LAST_SESSION => self.session_readable(stream, io),
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => self.handshake_readable(stream, io),
			DISCOVERY => {
				if let Some(node_changes) = self.discovery.as_ref().unwrap().lock().unwrap().readable() {
					self.update_nodes(io, node_changes);
				}
				io.update_registration(DISCOVERY).expect("Error updating disicovery registration");
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
				self.discovery.as_ref().unwrap().lock().unwrap().writable();
				io.update_registration(DISCOVERY).expect("Error updating disicovery registration");
			}
			_ => panic!("Received unknown writable token"),
		}
	}

	fn timeout(&self, io: &IoContext<NetworkIoMessage<Message>>, token: TimerToken) {
		match token {
			IDLE => self.maintain_network(io),
			FIRST_SESSION ... LAST_SESSION => self.connection_timeout(token, io),
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => self.connection_timeout(token, io),
			DISCOVERY_REFRESH => {
				self.discovery.as_ref().unwrap().lock().unwrap().refresh();
				io.update_registration(DISCOVERY).expect("Error updating disicovery registration");
			},
			DISCOVERY_ROUND => {
				if let Some(node_changes) = self.discovery.as_ref().unwrap().lock().unwrap().round() {
					self.update_nodes(io, node_changes);
				}
				io.update_registration(DISCOVERY).expect("Error updating disicovery registration");
			},
			_ => match self.timers.read().unwrap().get(&token).cloned() {
				Some(timer) => match self.handlers.read().unwrap().get(timer.protocol).cloned() {
						None => { warn!(target: "net", "No handler found for protocol: {:?}", timer.protocol) },
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
				warn!("Unexpected session stream registration");
			}
			FIRST_HANDSHAKE ... LAST_HANDSHAKE => {
				let connection = { self.handshakes.read().unwrap().get(stream).cloned() };
				if let Some(connection) = connection {
					connection.lock().unwrap().register_socket(reg, event_loop).expect("Error registering socket");
				}
			}
			DISCOVERY => self.discovery.as_ref().unwrap().lock().unwrap().register_socket(event_loop).expect("Error registering discovery socket"),
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
			TCP_ACCEPT => event_loop.deregister(self.tcp_listener.lock().unwrap().deref()).unwrap(),
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
			DISCOVERY => self.discovery.as_ref().unwrap().lock().unwrap().update_registration(event_loop).expect("Error reregistering discovery socket"),
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
