use std::mem;
use std::net::{SocketAddr};
use std::collections::{HashMap};
use std::hash::{Hasher};
use std::str::{FromStr};
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
pub enum NetworkIoMessage<Message> where Message: Send {
	/// Register a new protocol handler.
	AddHandler {
		handler: Option<Box<NetworkProtocolHandler<Message>+Send>>,
		protocol: ProtocolId,
		versions: Vec<u8>,
	},
	/// Send data over the network.
	Send {
		peer: PeerId,
		packet_id: PacketId,
		protocol: ProtocolId,
		data: Vec<u8>,
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
pub struct NetworkContext<'s, 'io, Message> where Message: Send + 'static, 'io: 's {
	io: &'s mut IoContext<'io, NetworkIoMessage<Message>>,
	protocol: ProtocolId,
	connections: &'s mut Slab<ConnectionEntry>,
	timers: &'s mut HashMap<TimerToken, ProtocolId>,
	session: Option<StreamToken>,
}

impl<'s, 'io, Message> NetworkContext<'s, 'io, Message> where Message: Send + 'static, {
	/// Create a new network IO access point. Takes references to all the data that can be updated within the IO handler.
	fn new(io: &'s mut IoContext<'io, NetworkIoMessage<Message>>, 
		protocol: ProtocolId, 
		session: Option<StreamToken>, connections: &'s mut Slab<ConnectionEntry>, 
		timers: &'s mut HashMap<TimerToken, ProtocolId>) -> NetworkContext<'s, 'io, Message> {
		NetworkContext {
			io: io,
			protocol: protocol,
			session: session,
			connections: connections,
			timers: timers,
		}
	}

	/// Send a packet over the network to another peer.
	pub fn send(&mut self, peer: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		match self.connections.get_mut(peer) {
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				s.send_packet(self.protocol, packet_id as u8, &data).unwrap_or_else(|e| {
					warn!(target: "net", "Send error: {:?}", e);
				}); //TODO: don't copy vector data
			},
			_ => {
				warn!(target: "net", "Send: Peer does not exist");
			}
		}
		Ok(())
	}

	/// Respond to a current network message. Panics if no there is no packet in the context.
	pub fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		match self.session {
			Some(session) => self.send(session, packet_id, data),
			None => {
				panic!("Respond: Session does not exist")
			}
		}
	}

	/// Disable current protocol capability for given peer. If no capabilities left peer gets disconnected.
	pub fn disable_peer(&mut self, _peer: PeerId) {
		//TODO: remove capability, disconnect if no capabilities left
	}

	/// Register a new IO timer. Returns a new timer token. 'NetworkProtocolHandler::timeout' will be called with the token.
	pub fn register_timer(&mut self, ms: u64) -> Result<TimerToken, UtilError>{
		match self.io.register_timer(ms) {
			Ok(token) => {
				self.timers.insert(token, self.protocol);
				Ok(token)
			},
			e => e,
		}
	}

	/// Returns peer identification string
	pub fn peer_info(&self, peer: PeerId) -> String {
		match self.connections.get(peer) {
			Some(&ConnectionEntry::Session(ref s)) => {
				s.info.client_version.clone()
			},
			_ => {
				"unknown".to_owned()
			}
		}
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

/// Root IO handler. Manages protocol handlers, IO timers and network connections.
pub struct Host<Message> where Message: Send {
	pub info: HostInfo,
	udp_socket: UdpSocket,
	listener: TcpListener,
	connections: Slab<ConnectionEntry>,
	timers: HashMap<TimerToken, ProtocolId>,
	nodes: HashMap<NodeId, Node>,
	handlers: HashMap<ProtocolId, Box<NetworkProtocolHandler<Message>>>,
}

impl<Message> Host<Message> where Message: Send {
	pub fn new() -> Host<Message> {
		let config = NetworkConfiguration::new();
		let addr = config.listen_address;
		// Setup the server socket
		let listener = TcpListener::bind(&addr).unwrap();
		let udp_socket = UdpSocket::bound(&addr).unwrap();
		Host::<Message> {
			info: HostInfo {
				keys: KeyPair::create().unwrap(),
				config: config,
				nonce: H256::random(),
				protocol_version: 4,
				client_version: "parity".to_owned(),
				listen_port: 0,
				capabilities: Vec::new(),
			},
			udp_socket: udp_socket,
			listener: listener,
			connections: Slab::new_starting_at(FIRST_CONNECTION, MAX_CONNECTIONS),
			timers: HashMap::new(),
			nodes: HashMap::new(),
			handlers: HashMap::new(),
		}
	}

	fn add_node(&mut self, id: &str) {
		match Node::from_str(id) {
			Err(e) => { warn!("Could not add node: {:?}", e); },
			Ok(n) => {
				self.nodes.insert(n.id.clone(), n);
			}
		}
	}

	fn maintain_network(&mut self, io: &mut IoContext<NetworkIoMessage<Message>>) {
		self.connect_peers(io);
		io.event_loop.timeout_ms(Token(IDLE), MAINTENANCE_TIMEOUT).unwrap();
	}

	fn have_session(&self, id: &NodeId) -> bool {
		self.connections.iter().any(|e| match *e { ConnectionEntry::Session(ref s) => s.info.id.eq(&id), _ => false  })
	}

	fn connecting_to(&self, id: &NodeId) -> bool {
		self.connections.iter().any(|e| match *e { ConnectionEntry::Handshake(ref h) => h.id.eq(&id), _ => false  })
	}

	fn connect_peers(&mut self, io: &mut IoContext<NetworkIoMessage<Message>>) {
		struct NodeInfo {
			id: NodeId,
			peer_type: PeerType
		}

		let mut to_connect: Vec<NodeInfo> = Vec::new();

		let mut req_conn = 0;
		//TODO: use nodes from discovery here
		//for n in self.node_buckets.iter().flat_map(|n| &n.nodes).map(|id| NodeInfo { id: id.clone(), peer_type: self.nodes.get(id).unwrap().peer_type}) {
		for n in self.nodes.values().map(|n| NodeInfo { id: n.id.clone(), peer_type: n.peer_type }) {
			let connected = self.have_session(&n.id) || self.connecting_to(&n.id);
			let required = n.peer_type == PeerType::Required;
			if connected && required {
				req_conn += 1;
			}
			else if !connected && (!self.info.config.pin || required) {
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

		if !self.info.config.pin
		{
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
	fn connect_peer(&mut self, id: &NodeId, io: &mut IoContext<NetworkIoMessage<Message>>) {
		if self.have_session(id)
		{
			warn!("Aborted connect. Node already connected.");
			return;
		}
		if self.connecting_to(id)
		{
			warn!("Aborted connect. Node already connecting.");
			return;
		}

		let socket = {
			let node = self.nodes.get_mut(id).unwrap();
			node.last_attempted = Some(::time::now());

			match TcpStream::connect(&node.endpoint.address) {
				Ok(socket) => socket,
				Err(_) => {
					warn!("Cannot connect to node");
					return;
				}
			}
		};

		let nonce = self.info.next_nonce();
		match self.connections.insert_with(|token| ConnectionEntry::Handshake(Handshake::new(Token(token), id, socket, &nonce).expect("Can't create handshake"))) {
			Some(token) => {
				match self.connections.get_mut(token) {
					Some(&mut ConnectionEntry::Handshake(ref mut h)) => {
						h.start(&self.info, true)
							.and_then(|_| h.register(io.event_loop))
							.unwrap_or_else (|e| {
								debug!(target: "net", "Handshake create error: {:?}", e);
							});
					},
					_ => {}
				}
			},
			None => { warn!("Max connections reached") }
		}
	}


	fn accept(&mut self, _io: &mut IoContext<NetworkIoMessage<Message>>) {
		trace!(target: "net", "accept");
	}

	#[allow(single_match)]
	fn connection_writable<'s>(&'s mut self, token: StreamToken, io: &mut IoContext<'s, NetworkIoMessage<Message>>) {
		let mut kill = false;
		let mut create_session = false;
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Handshake(ref mut h)) => {
				h.writable(io.event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Handshake write error: {:?}", e);
					kill = true;
				});
				create_session = h.done();
			},
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				s.writable(io.event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Session write error: {:?}", e);
					kill = true;
				});
			}
			_ => {
				warn!(target: "net", "Received event for unknown connection");
			}
		}
		if kill {
			self.kill_connection(token, io);
			return;
		} else if create_session {
			self.start_session(token, io);
		}
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				s.reregister(io.event_loop).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
			},
			_ => (),
		}
	}

	fn connection_closed<'s>(&'s mut self, token: TimerToken, io: &mut IoContext<'s, NetworkIoMessage<Message>>) {
		self.kill_connection(token, io);
	}

	fn connection_readable<'s>(&'s mut self, token: StreamToken, io: &mut IoContext<'s, NetworkIoMessage<Message>>) {
		let mut kill = false;
		let mut create_session = false;
		let mut ready_data: Vec<ProtocolId> = Vec::new();
		let mut packet_data: Option<(ProtocolId, PacketId, Vec<u8>)> = None;
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Handshake(ref mut h)) => {
				h.readable(io.event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Handshake read error: {:?}", e);
					kill = true;
				});
				create_session = h.done();
			},
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				let sd = { s.readable(io.event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Session read error: {:?}", e);
					kill = true;
					SessionData::None
				}) };
				match sd {
					SessionData::Ready => {
						for (p, _) in &mut self.handlers {
							if s.have_capability(p)  {
								ready_data.push(p);
							}
						}
					},
					SessionData::Packet {
						data,
						protocol,
						packet_id,
					} => {
						match self.handlers.get_mut(protocol) {
							None => { warn!(target: "net", "No handler found for protocol: {:?}", protocol) },
							Some(_) => packet_data = Some((protocol, packet_id, data)),
						}
					},
					SessionData::None => {},
				}
			}
			_ => {
				warn!(target: "net", "Received event for unknown connection");
			}
		}
		if kill {
			self.kill_connection(token, io);
			return;
		}
		if create_session {
			self.start_session(token, io);
		}
		for p in ready_data {
			let mut h = self.handlers.get_mut(p).unwrap();
			h.connected(&mut NetworkContext::new(io, p, Some(token), &mut self.connections, &mut self.timers), &token);
		}
		if let Some((p, packet_id, data)) = packet_data {
			let mut h = self.handlers.get_mut(p).unwrap();
			h.read(&mut NetworkContext::new(io, p, Some(token), &mut self.connections, &mut self.timers), &token, packet_id, &data[1..]);
		}

		if let Some(&mut ConnectionEntry::Session(ref mut s)) = self.connections.get_mut(token) {
			s.reregister(io.event_loop).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
		}
	}

	fn start_session(&mut self, token: StreamToken, io: &mut IoContext<NetworkIoMessage<Message>>) {
		let info = &self.info;
		// TODO: use slab::replace_with (currently broken)
		/*
		match self.connections.remove(token) {
			Some(ConnectionEntry::Handshake(h)) => {
				match Session::new(h, io.event_loop, info) {
					Ok(session) => {
						assert!(token == self.connections.insert(ConnectionEntry::Session(session)).ok().unwrap());
					},
					Err(e) => {
						debug!(target: "net", "Session construction error: {:?}", e);
					}
				}
			},
			_ => panic!("Error updating slab with session")
		}*/
		self.connections.replace_with(token, |c| {
			match c {
				ConnectionEntry::Handshake(h) => Session::new(h, io.event_loop, info)
					.map(|s| Some(ConnectionEntry::Session(s)))
					.unwrap_or_else(|e| {
						debug!(target: "net", "Session construction error: {:?}", e);
						None
					}),
					_ => { panic!("No handshake to create a session from"); }
			}
		}).expect("Error updating slab with session");
	}

	fn connection_timeout<'s>(&'s mut self, token: StreamToken, io: &mut IoContext<'s, NetworkIoMessage<Message>>) {
		self.kill_connection(token, io)
	}

	fn kill_connection<'s>(&'s mut self, token: StreamToken, io: &mut IoContext<'s, NetworkIoMessage<Message>>) {
		let mut to_disconnect: Vec<ProtocolId> = Vec::new();
		let mut remove = true;
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Handshake(_)) => (), // just abandon handshake
			Some(&mut ConnectionEntry::Session(ref mut s)) if s.is_ready() => {
				for (p, _) in &mut self.handlers {
					if s.have_capability(p)  {
						to_disconnect.push(p);
					}
				}
			},
			_ => {
				remove = false;
			},
		}
		for p in to_disconnect {
			let mut h = self.handlers.get_mut(p).unwrap();
			h.disconnected(&mut NetworkContext::new(io, p, Some(token), &mut self.connections, &mut self.timers), &token);
		}
		if remove {
			self.connections.remove(token);
		}
	}
}

impl<Message> IoHandler<NetworkIoMessage<Message>> for Host<Message> where Message: Send + 'static {
	/// Initialize networking
	fn initialize(&mut self, io: &mut IoContext<NetworkIoMessage<Message>>) {
		/*
		match ::ifaces::Interface::get_all().unwrap().into_iter().filter(|x| x.kind == ::ifaces::Kind::Packet && x.addr.is_some()).next() {
		Some(iface) => config.public_address = iface.addr.unwrap(),
		None => warn!("No public network interface"),
		*/

		// Start listening for incoming connections
		io.event_loop.register(&self.listener, Token(TCP_ACCEPT), EventSet::readable(), PollOpt::edge()).unwrap();
		io.event_loop.timeout_ms(Token(IDLE), MAINTENANCE_TIMEOUT).unwrap();
		// open the udp socket
		io.event_loop.register(&self.udp_socket, Token(NODETABLE_RECEIVE), EventSet::readable(), PollOpt::edge()).unwrap();
		io.event_loop.timeout_ms(Token(NODETABLE_MAINTAIN), 7200).unwrap();
		let port = self.info.config.listen_address.port();
		self.info.listen_port = port;

		self.add_node("enode://a9a921de2ff09a9a4d38b623c67b2d6b477a8e654ae95d874750cbbcb31b33296496a7b4421934e2629269e180823e52c15c2b19fc59592ec51ffe4f2de76ed7@127.0.0.1:30303");
/*		// GO bootnodes
		self.add_node("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@52.16.188.185:30303"); // IE
		self.add_node("enode://de471bccee3d042261d52e9bff31458daecc406142b401d4cd848f677479f73104b9fdeb090af9583d3391b7f10cb2ba9e26865dd5fca4fcdc0fb1e3b723c786@54.94.239.50:30303");  // BR
		self.add_node("enode://1118980bf48b0a3640bdba04e0fe78b1add18e1cd99bf22d53daac1fd9972ad650df52176e7c7d89d1114cfef2bc23a2959aa54998a46afcf7d91809f0855082@52.74.57.123:30303");  // SG
		// ETH/DEV cpp-ethereum (poc-9.ethdev.com)
		self.add_node("enode://979b7fa28feeb35a4741660a16076f1943202cb72b6af70d327f053e248bab9ba81760f39d0701ef1d8f89cc1fbd2cacba0710a12cd5314d5e0c9021aa3637f9@5.1.83.226:30303");*/
	}

	fn stream_hup<'s>(&'s mut self, io: &mut IoContext<'s, NetworkIoMessage<Message>>, stream: StreamToken) {
		trace!(target: "net", "Hup: {}", stream);
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_closed(stream, io),
			_ => warn!(target: "net", "Unexpected hup"),
		};
	}

	fn stream_readable<'s>(&'s mut self, io: &mut IoContext<'s, NetworkIoMessage<Message>>, stream: StreamToken) {
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_readable(stream, io),
			NODETABLE_RECEIVE => {},
			TCP_ACCEPT => self.accept(io), 
			_ => panic!("Received unknown readable token"),
		}
	}

	fn stream_writable<'s>(&'s mut self, io: &mut IoContext<'s, NetworkIoMessage<Message>>, stream: StreamToken) {
		match stream {
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_writable(stream, io),
			_ => panic!("Received unknown writable token"),
		}
	}

	fn timeout<'s>(&'s mut self, io: &mut IoContext<'s, NetworkIoMessage<Message>>, token: TimerToken) {
		match token {
			IDLE => self.maintain_network(io),
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_timeout(token, io),
			NODETABLE_DISCOVERY => {},
			NODETABLE_MAINTAIN => {},
			_ => {
				if let Some(protocol) = self.timers.get_mut(&token).map(|p| *p) {
					match self.handlers.get_mut(protocol) {
							None => { warn!(target: "net", "No handler found for protocol: {:?}", protocol) },
							Some(h) => { h.timeout(&mut NetworkContext::new(io, protocol, Some(token), &mut self.connections, &mut self.timers), token); }
					};
				} // else time not registerd through us
			}
		}
	}

	fn message<'s>(&'s mut self, io: &mut IoContext<'s, NetworkIoMessage<Message>>, message: &'s mut NetworkIoMessage<Message>) {
		match *message {
			NetworkIoMessage::AddHandler {
				ref mut handler,
				ref protocol,
				ref versions
			} => {
				let mut h = mem::replace(handler, None).unwrap();
				h.initialize(&mut NetworkContext::new(io, protocol, None, &mut self.connections, &mut self.timers));
				self.handlers.insert(protocol, h);
				for v in versions {
					self.info.capabilities.push(CapabilityInfo { protocol: protocol, version: *v, packet_count:0 });
				}
			},
			NetworkIoMessage::Send {
				ref peer,
				ref packet_id,
				ref protocol,
				ref data,
			} => {
				match self.connections.get_mut(*peer as usize) {
					Some(&mut ConnectionEntry::Session(ref mut s)) => {
						s.send_packet(protocol, *packet_id as u8, &data).unwrap_or_else(|e| {
							warn!(target: "net", "Send error: {:?}", e);
						}); //TODO: don't copy vector data
					},
					_ => {
						warn!(target: "net", "Send: Peer does not exist");
					}
				}
			},
			NetworkIoMessage::User(ref message) => {
				for (p, h) in &mut self.handlers {
					h.message(&mut NetworkContext::new(io, p, None, &mut self.connections, &mut self.timers), &message);
				}
			}
		}
	}
}
