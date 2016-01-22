use std::net::{SocketAddr};
use std::collections::{HashMap};
use std::hash::{Hasher};
use std::str::{FromStr};
use std::sync::*;
use std::ops::*;
use mio::*;
use mio::tcp::*;
use mio::udp::*;
use hash::*;
use crypto::*;
use sha3::Hashable;
use rlp::*;
use network::handshake::Handshake;
use network::session::{Session, SessionData};
use error::*;
use io::*;
use network::NetworkProtocolHandler;
use network::node::*;

type Slab<T> = ::slab::Slab<T, usize>;

const _DEFAULT_PORT: u16 = 30304;

const MAX_CONNECTIONS: usize = 1024;
const IDEAL_PEERS: u32 = 10;

const MAINTENANCE_TIMEOUT: u64 = 1000;

#[derive(Debug)]
struct NetworkConfiguration {
	listen_address: SocketAddr,
	public_address: SocketAddr,
	nat_enabled: bool,
	discovery_enabled: bool,
	pin: bool,
}

impl NetworkConfiguration {
	fn new() -> NetworkConfiguration {
		NetworkConfiguration {
			listen_address: SocketAddr::from_str("0.0.0.0:30304").unwrap(),
			public_address: SocketAddr::from_str("0.0.0.0:30304").unwrap(),
			nat_enabled: true,
			discovery_enabled: true,
			pin: false,
		}
	}
}

// Tokens
//const TOKEN_BEGIN: usize = USER_TOKEN_START; // TODO: ICE in rustc 1.7.0-nightly (49c382779 2016-01-12)
const TOKEN_BEGIN: usize = 32;
const TCP_ACCEPT: usize = TOKEN_BEGIN + 1;
const IDLE: usize = TOKEN_BEGIN + 2;
const NODETABLE_RECEIVE: usize = TOKEN_BEGIN + 3;
const NODETABLE_MAINTAIN: usize = TOKEN_BEGIN + 4;
const NODETABLE_DISCOVERY: usize = TOKEN_BEGIN + 5;
const FIRST_CONNECTION: usize = TOKEN_BEGIN + 16;
const LAST_CONNECTION: usize = FIRST_CONNECTION + MAX_CONNECTIONS - 1;

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
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder {
		encoder.emit_list(|e| {
			self.protocol.encode(e);
			(self.version as u32).encode(e);
		});
	}
}

/// IO access point. This is passed to all IO handlers and provides an interface to the IO subsystem.
pub struct NetworkContext<'s, Message> where Message: Send + Sync + Clone + 'static, 's {
	io: &'s IoContext<NetworkIoMessage<Message>>,
	protocol: ProtocolId,
	connections: Arc<RwLock<Slab<SharedConnectionEntry>>>,
	session: Option<StreamToken>,
}

impl<'s, Message> NetworkContext<'s, Message> where Message: Send + Sync + Clone + 'static, {
	/// Create a new network IO access point. Takes references to all the data that can be updated within the IO handler.
	fn new(io: &'s IoContext<NetworkIoMessage<Message>>, 
		protocol: ProtocolId, 
		session: Option<StreamToken>, connections: Arc<RwLock<Slab<SharedConnectionEntry>>>) -> NetworkContext<'s, Message> {
		NetworkContext {
			io: io,
			protocol: protocol,
			session: session,
			connections: connections,
		}
	}

	/// Send a packet over the network to another peer.
	pub fn send(&self, peer: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		if let Some(connection) = self.connections.read().unwrap().get(peer).cloned() {
			match *connection.lock().unwrap().deref_mut() {
				ConnectionEntry::Session(ref mut s) => {
					s.send_packet(self.protocol, packet_id as u8, &data).unwrap_or_else(|e| {
						warn!(target: "net", "Send error: {:?}", e);
					}); //TODO: don't copy vector data
				},
				_ => warn!(target: "net", "Send: Peer is not connected yet")
			}
		} else  {
			warn!(target: "net", "Send: Peer does not exist")
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
	pub fn disable_peer(&self, _peer: PeerId) {
		//TODO: remove capability, disconnect if no capabilities left
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
		if let Some(connection) = self.connections.read().unwrap().get(peer).cloned() {
			if let ConnectionEntry::Session(ref s) = *connection.lock().unwrap().deref() {
				return s.info.client_version.clone()
			}
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

enum ConnectionEntry {
	Handshake(Handshake),
	Session(Session)
}

type SharedConnectionEntry = Arc<Mutex<ConnectionEntry>>;

#[derive(Copy, Clone)]
struct ProtocolTimer {
	pub protocol: ProtocolId,
	pub token: TimerToken, // Handler level token
}

/// Root IO handler. Manages protocol handlers, IO timers and network connections.
pub struct Host<Message> where Message: Send + Sync + Clone {
	pub info: RwLock<HostInfo>,
	udp_socket: Mutex<UdpSocket>,
	tcp_listener: Mutex<TcpListener>,
	connections: Arc<RwLock<Slab<SharedConnectionEntry>>>,
	nodes: RwLock<HashMap<NodeId, Node>>,
	handlers: RwLock<HashMap<ProtocolId, Arc<NetworkProtocolHandler<Message>>>>,
	timers: RwLock<HashMap<TimerToken, ProtocolTimer>>,
	timer_counter: RwLock<usize>,
}

impl<Message> Host<Message> where Message: Send + Sync + Clone {
	pub fn new() -> Host<Message> {
		let config = NetworkConfiguration::new();
		let addr = config.listen_address;
		// Setup the server socket
		let tcp_listener = TcpListener::bind(&addr).unwrap();
		let udp_socket = UdpSocket::bound(&addr).unwrap();
		let mut host = Host::<Message> {
			info: RwLock::new(HostInfo {
				keys: KeyPair::create().unwrap(),
				config: config,
				nonce: H256::random(),
				protocol_version: 4,
				client_version: "parity".to_owned(),
				listen_port: 0,
				capabilities: Vec::new(),
			}),
			udp_socket: Mutex::new(udp_socket),
			tcp_listener: Mutex::new(tcp_listener),
			connections: Arc::new(RwLock::new(Slab::new_starting_at(FIRST_CONNECTION, MAX_CONNECTIONS))),
			nodes: RwLock::new(HashMap::new()),
			handlers: RwLock::new(HashMap::new()),
			timers: RwLock::new(HashMap::new()),
			timer_counter: RwLock::new(LAST_CONNECTION + 1),
		};
		let port = host.info.read().unwrap().config.listen_address.port();
		host.info.write().unwrap().deref_mut().listen_port = port;

		/*
		match ::ifaces::Interface::get_all().unwrap().into_iter().filter(|x| x.kind == ::ifaces::Kind::Packet && x.addr.is_some()).next() {
		Some(iface) => config.public_address = iface.addr.unwrap(),
		None => warn!("No public network interface"),
		*/

		// self.add_node("enode://a9a921de2ff09a9a4d38b623c67b2d6b477a8e654ae95d874750cbbcb31b33296496a7b4421934e2629269e180823e52c15c2b19fc59592ec51ffe4f2de76ed7@127.0.0.1:30303");
		// GO bootnodes
		host.add_node("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@52.16.188.185:30303"); // IE
		host.add_node("enode://de471bccee3d042261d52e9bff31458daecc406142b401d4cd848f677479f73104b9fdeb090af9583d3391b7f10cb2ba9e26865dd5fca4fcdc0fb1e3b723c786@54.94.239.50:30303");  // BR
		host.add_node("enode://1118980bf48b0a3640bdba04e0fe78b1add18e1cd99bf22d53daac1fd9972ad650df52176e7c7d89d1114cfef2bc23a2959aa54998a46afcf7d91809f0855082@52.74.57.123:30303");  // SG
		// ETH/DEV cpp-ethereum (poc-9.ethdev.com)
		host.add_node("enode://979b7fa28feeb35a4741660a16076f1943202cb72b6af70d327f053e248bab9ba81760f39d0701ef1d8f89cc1fbd2cacba0710a12cd5314d5e0c9021aa3637f9@5.1.83.226:30303");
		host
	}

	pub fn add_node(&mut self, id: &str) {
		match Node::from_str(id) {
			Err(e) => { warn!("Could not add node: {:?}", e); },
			Ok(n) => {
				self.nodes.write().unwrap().insert(n.id.clone(), n);
			}
		}
	}

	pub fn client_version(&self) -> String {
		self.info.read().unwrap().client_version.clone()
	}

	pub fn client_id(&self) -> NodeId {
		self.info.read().unwrap().id().clone()
	}

	fn maintain_network(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		self.connect_peers(io);
	}

	fn have_session(&self, id: &NodeId) -> bool {
		self.connections.read().unwrap().iter().any(|e| match *e.lock().unwrap().deref() { ConnectionEntry::Session(ref s) => s.info.id.eq(&id), _ => false  })
	}

	fn connecting_to(&self, id: &NodeId) -> bool {
		self.connections.read().unwrap().iter().any(|e| match *e.lock().unwrap().deref() { ConnectionEntry::Handshake(ref h) => h.id.eq(&id), _ => false  })
	}

	fn connect_peers(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		struct NodeInfo {
			id: NodeId,
			peer_type: PeerType
		}

		let mut to_connect: Vec<NodeInfo> = Vec::new();

		let mut req_conn = 0;
		//TODO: use nodes from discovery here
		//for n in self.node_buckets.iter().flat_map(|n| &n.nodes).map(|id| NodeInfo { id: id.clone(), peer_type: self.nodes.get(id).unwrap().peer_type}) {
		let pin = self.info.read().unwrap().deref().config.pin;
		for n in self.nodes.read().unwrap().values().map(|n| NodeInfo { id: n.id.clone(), peer_type: n.peer_type }) {
			let connected = self.have_session(&n.id) || self.connecting_to(&n.id);
			let required = n.peer_type == PeerType::Required;
			if connected && required {
				req_conn += 1;
			}
			else if !connected && (!pin || required) {
				to_connect.push(n);
			}
		}

		for n in &to_connect {
			if n.peer_type == PeerType::Required {
				if req_conn < IDEAL_PEERS {
					self.connect_peer(&n.id, io);
				}
				req_conn += 1;
			}
		}

		if !pin {
			let pending_count = 0; //TODO:
			let peer_count = 0;
			let mut open_slots = IDEAL_PEERS - peer_count  - pending_count + req_conn;
			if open_slots > 0 {
				for n in &to_connect {
					if n.peer_type == PeerType::Optional && open_slots > 0 {
						open_slots -= 1;
						self.connect_peer(&n.id, io);
					}
				}
			}
		}
	}

	#[allow(single_match)]
	#[allow(block_in_if_condition_stmt)]
	fn connect_peer(&self, id: &NodeId, io: &IoContext<NetworkIoMessage<Message>>) {
		if self.have_session(id)
		{
			warn!("Aborted connect. Node already connected.");
			return;
		}
		if self.connecting_to(id) {
			warn!("Aborted connect. Node already connecting.");
			return;
		}

		let socket = {
			let address = {
				let mut nodes = self.nodes.write().unwrap();
				let node = nodes.get_mut(id).unwrap();
				node.last_attempted = Some(::time::now());
				node.endpoint.address
			};
			match TcpStream::connect(&address) {
				Ok(socket) => socket,
				Err(_) => {
					warn!("Cannot connect to node");
					return;
				}
			}
		};

		let nonce = self.info.write().unwrap().next_nonce();
		if self.connections.write().unwrap().insert_with(|token| {
			let mut handshake = Handshake::new(token, id, socket, &nonce).expect("Can't create handshake");
			handshake.start(io, &self.info.read().unwrap(), true).and_then(|_| io.register_stream(token)).unwrap_or_else (|e| {
				debug!(target: "net", "Handshake create error: {:?}", e);
			});
			Arc::new(Mutex::new(ConnectionEntry::Handshake(handshake)))
		}).is_none() {
			warn!("Max connections reached");
		}
	}

	fn accept(&self, _io: &IoContext<NetworkIoMessage<Message>>) {
		trace!(target: "net", "accept");
	}

	#[allow(single_match)]
	fn connection_writable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut create_session = false;
		let mut kill = false;
		if let Some(connection) = self.connections.read().unwrap().get(token).cloned() {
			match *connection.lock().unwrap().deref_mut() {
				ConnectionEntry::Handshake(ref mut h) => {
					match h.writable(io, &self.info.read().unwrap()) {
						Err(e) => {
							debug!(target: "net", "Handshake write error: {:?}", e);
							kill = true;
						},
						Ok(_) => ()
					}
					if h.done() {
						create_session = true;
					}
				},
				ConnectionEntry::Session(ref mut s) => {
					match s.writable(io, &self.info.read().unwrap()) {
						Err(e) => {
							debug!(target: "net", "Session write error: {:?}", e);
							kill = true;
						},
						Ok(_) => ()
					}
					io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
				}
			}
		} 
		if kill {
			self.kill_connection(token, io); //TODO: mark connection as dead an check in kill_connection
			return;
		} else if create_session {
			self.start_session(token, io);
			io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
		}
	}

	fn connection_closed(&self, token: TimerToken, io: &IoContext<NetworkIoMessage<Message>>) {
		self.kill_connection(token, io);
	}

	fn connection_readable(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut ready_data: Vec<ProtocolId> = Vec::new();
		let mut packet_data: Option<(ProtocolId, PacketId, Vec<u8>)> = None;
		let mut create_session = false;
		let mut kill = false;
		if let Some(connection) = self.connections.read().unwrap().get(token).cloned() {
			match *connection.lock().unwrap().deref_mut() {
				ConnectionEntry::Handshake(ref mut h) => {
					if let Err(e) = h.readable(io, &self.info.read().unwrap()) {
						debug!(target: "net", "Handshake read error: {:?}", e);
						kill = true;
					}
					if h.done() {
						create_session = true;
					}
				},
				ConnectionEntry::Session(ref mut s) => {
					match s.readable(io, &self.info.read().unwrap()) {
						Err(e) => {
							debug!(target: "net", "Handshake read error: {:?}", e);
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
			}
		} 
		if kill {
			self.kill_connection(token, io); //TODO: mark connection as dead an check in kill_connection
			return;
		} else if create_session {
			self.start_session(token, io);
			io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
		}
		for p in ready_data {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.connected(&NetworkContext::new(io, p, Some(token), self.connections.clone()), &token);
		}
		if let Some((p, packet_id, data)) = packet_data {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.read(&NetworkContext::new(io, p, Some(token), self.connections.clone()), &token, packet_id, &data[1..]);
		}
		io.update_registration(token).unwrap_or_else(|e| debug!(target: "net", "Token registration error: {:?}", e));
	}

	fn start_session(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		self.connections.write().unwrap().replace_with(token, |c| {
			match Arc::try_unwrap(c).ok().unwrap().into_inner().unwrap() {
				ConnectionEntry::Handshake(h) => {
					let session = Session::new(h, io, &self.info.read().unwrap()).expect("Session creation error");
					io.update_registration(token).expect("Error updating session registration");
					Some(Arc::new(Mutex::new(ConnectionEntry::Session(session))))
				},
				_ => { None } // handshake expired
			}
		}).ok();
	}

	fn connection_timeout(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		self.kill_connection(token, io)
	}

	fn kill_connection(&self, token: StreamToken, io: &IoContext<NetworkIoMessage<Message>>) {
		let mut to_disconnect: Vec<ProtocolId> = Vec::new();
		{
			let mut connections = self.connections.write().unwrap();
			if let Some(connection) = connections.get(token).cloned() {
				match *connection.lock().unwrap().deref_mut() {
					ConnectionEntry::Handshake(_) => {
						connections.remove(token);
					},
					ConnectionEntry::Session(ref mut s) if s.is_ready() => {
						for (p, _) in self.handlers.read().unwrap().iter() {
							if s.have_capability(p)  {
								to_disconnect.push(p);
							}
						}
						connections.remove(token);
					},
					_ => {},
				}
			}
		}
		for p in to_disconnect {
			let h = self.handlers.read().unwrap().get(p).unwrap().clone();
			h.disconnected(&NetworkContext::new(io, p, Some(token), self.connections.clone()), &token);
		}
	}
}

impl<Message> IoHandler<NetworkIoMessage<Message>> for Host<Message> where Message: Send + Sync + Clone + 'static {
	/// Initialize networking
	fn initialize(&self, io: &IoContext<NetworkIoMessage<Message>>) {
		io.register_stream(TCP_ACCEPT).expect("Error registering TCP listener");
		io.register_stream(NODETABLE_RECEIVE).expect("Error registering UDP listener");
		io.register_timer(IDLE, MAINTENANCE_TIMEOUT).expect("Error registering Network idle timer");
		//io.register_timer(NODETABLE_MAINTAIN, 7200);
	}

	fn stream_hup(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		trace!(target: "net", "Hup: {}", stream);
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_closed(stream, io),
			_ => warn!(target: "net", "Unexpected hup"),
		};
	}

	fn stream_readable(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_readable(stream, io),
			NODETABLE_RECEIVE => {},
			TCP_ACCEPT => self.accept(io), 
			_ => panic!("Received unknown readable token"),
		}
	}

	fn stream_writable(&self, io: &IoContext<NetworkIoMessage<Message>>, stream: StreamToken) {
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_writable(stream, io),
			NODETABLE_RECEIVE => {},
			_ => panic!("Received unknown writable token"),
		}
	}

	fn timeout(&self, io: &IoContext<NetworkIoMessage<Message>>, token: TimerToken) {
		match token {
			IDLE => self.maintain_network(io),
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_timeout(token, io),
			NODETABLE_DISCOVERY => {},
			NODETABLE_MAINTAIN => {},
			_ => match self.timers.read().unwrap().get(&token).cloned() {
				Some(timer) => match self.handlers.read().unwrap().get(timer.protocol).cloned() {
						None => { warn!(target: "net", "No handler found for protocol: {:?}", timer.protocol) },
						Some(h) => { h.timeout(&NetworkContext::new(io, timer.protocol, None, self.connections.clone()), timer.token); }
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
				h.initialize(&NetworkContext::new(io, protocol, None, self.connections.clone()));
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
			NetworkIoMessage::User(ref message) => {
				for (p, h) in self.handlers.read().unwrap().iter() {
					h.message(&NetworkContext::new(io, p, None, self.connections.clone()), &message);
				}
			}
		}
	}

	fn register_stream(&self, stream: StreamToken, reg: Token, event_loop: &mut EventLoop<IoManager<NetworkIoMessage<Message>>>) {
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => {
				if let Some(connection) = self.connections.read().unwrap().get(stream).cloned() {
					match *connection.lock().unwrap().deref() {
						ConnectionEntry::Handshake(ref h) => h.register_socket(reg, event_loop).expect("Error registering socket"),
						ConnectionEntry::Session(_) => warn!("Unexpected session stream registration")
					}
				} else {} // expired
			}
			NODETABLE_RECEIVE => event_loop.register(self.udp_socket.lock().unwrap().deref(), Token(NODETABLE_RECEIVE), EventSet::all(), PollOpt::edge()).expect("Error registering stream"),
			TCP_ACCEPT => event_loop.register(self.tcp_listener.lock().unwrap().deref(), Token(TCP_ACCEPT), EventSet::all(), PollOpt::edge()).expect("Error registering stream"),
			_ => warn!("Unexpected stream registration")
		}
	}

	fn update_stream(&self, stream: StreamToken, reg: Token, event_loop: &mut EventLoop<IoManager<NetworkIoMessage<Message>>>) {
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => {
				if let Some(connection) = self.connections.read().unwrap().get(stream).cloned() {
					match *connection.lock().unwrap().deref() {
						ConnectionEntry::Handshake(ref h) => h.update_socket(reg, event_loop).expect("Error updating socket"),
						ConnectionEntry::Session(ref s) => s.update_socket(reg, event_loop).expect("Error updating socket"),
					}
				} else {} // expired
			}
			NODETABLE_RECEIVE => event_loop.reregister(self.udp_socket.lock().unwrap().deref(), Token(NODETABLE_RECEIVE), EventSet::all(), PollOpt::edge()).expect("Error reregistering stream"),
			TCP_ACCEPT => event_loop.reregister(self.tcp_listener.lock().unwrap().deref(), Token(TCP_ACCEPT), EventSet::all(), PollOpt::edge()).expect("Error reregistering stream"),
			_ => warn!("Unexpected stream update")
		}
	}
}
