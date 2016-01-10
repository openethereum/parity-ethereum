use std::net::{SocketAddr, ToSocketAddrs};
use std::collections::{HashMap};
use std::hash::{Hash, Hasher};
use std::str::{FromStr};
use mio::*;
use mio::util::{Slab};
use mio::tcp::*;
use mio::udp::*;
use hash::*;
use crypto::*;
use sha3::Hashable;
use rlp::*;
use time::Tm;
use network::handshake::Handshake;
use network::session::{Session, SessionData};
use error::*;
use network::ProtocolHandler;

const _DEFAULT_PORT: u16 = 30304;

const MAX_CONNECTIONS: usize = 1024;
const MAX_USER_TIMERS: usize = 32;
const IDEAL_PEERS: u32 = 10;

pub type NodeId = H512;
pub type TimerToken = usize;

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

#[derive(Debug)]
pub struct NodeEndpoint {
	address: SocketAddr,
	address_str: String,
	udp_port: u16
}

impl NodeEndpoint {
	fn from_str(s: &str) -> Result<NodeEndpoint, UtilError> {
		let address = s.to_socket_addrs().map(|mut i| i.next());
		match address {
			Ok(Some(a)) => Ok(NodeEndpoint {
				address: a,
				address_str: s.to_string(),
				udp_port: a.port()
			}),
			Ok(_) => Err(UtilError::AddressResolve(None)),
			Err(e) => Err(UtilError::AddressResolve(Some(e)))
		}
	}
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum PeerType {
	Required,
	Optional
}

struct Node {
	id: NodeId,
	endpoint: NodeEndpoint,
	peer_type: PeerType,
	last_attempted: Option<Tm>,
}

impl FromStr for Node {
	type Err = UtilError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (id, endpoint) = if &s[0..8] == "enode://" && s.len() > 136 && &s[136..137] == "@" {
			(try!(NodeId::from_str(&s[8..136])), try!(NodeEndpoint::from_str(&s[137..])))
		}
		else {
			(NodeId::new(), try!(NodeEndpoint::from_str(s)))
		};

		Ok(Node {
			id: id,
			endpoint: endpoint,
			peer_type: PeerType::Optional,
			last_attempted: None,
		})
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}
impl Eq for Node { }

impl Hash for Node {
	fn hash<H>(&self, state: &mut H) where H: Hasher {
		self.id.hash(state)
	}
}

// Tokens
const TCP_ACCEPT: usize = 1;
const IDLE: usize = 3;
const NODETABLE_RECEIVE: usize = 4;
const NODETABLE_MAINTAIN: usize = 5;
const NODETABLE_DISCOVERY: usize = 6;
const FIRST_CONNECTION: usize = 7;
const LAST_CONNECTION: usize = FIRST_CONNECTION + MAX_CONNECTIONS - 1;
const USER_TIMER: usize = LAST_CONNECTION;
const LAST_USER_TIMER: usize = USER_TIMER + MAX_USER_TIMERS - 1;

pub type PacketId = u8;
pub type ProtocolId = &'static str;

pub enum HostMessage {
	Shutdown,
	AddHandler {
		handler: Box<ProtocolHandler+Send>,
		protocol: ProtocolId,
		versions: Vec<u8>,
	},
	Send {
		peer: PeerId,
		packet_id: PacketId,
		protocol: ProtocolId,
		data: Vec<u8>,
	},
	UserMessage(UserMessage),
}

pub type UserMessageId = u32;

pub struct UserMessage {
	pub protocol: ProtocolId,
	pub id: UserMessageId,
	pub data: Option<Vec<u8>>,
}

pub type PeerId = usize;

#[derive(Debug, PartialEq, Eq)]
pub struct CapabilityInfo {
	pub protocol: ProtocolId,
	pub version: u8,
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

/// IO access point
pub struct HostIo<'s> {
	protocol: ProtocolId,
	connections: &'s mut Slab<ConnectionEntry>,
	timers: &'s mut Slab<UserTimer>,
	session: Option<Token>,
	event_loop: &'s mut EventLoop<Host>,
}

impl<'s> HostIo<'s> {
	fn new(protocol: ProtocolId, session: Option<Token>, event_loop: &'s mut EventLoop<Host>, connections: &'s mut Slab<ConnectionEntry>, timers: &'s mut Slab<UserTimer>) -> HostIo<'s> {
		HostIo {
			protocol: protocol,
			session: session,
			event_loop: event_loop,
			connections: connections,
			timers: timers,
		}
	}

	/// Send a packet over the network to another peer.
	pub fn send(&mut self, peer: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		match self.connections.get_mut(Token(peer)) {
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
			Some(session) => self.send(session.as_usize(), packet_id, data),
			None => {
				panic!("Respond: Session does not exist")
			}
		}
	}

	/// Register a new IO timer. Returns a new timer toke. 'ProtocolHandler::timeout' will be called with the token.
	pub fn register_timer(&mut self, ms: u64) -> Result<TimerToken, UtilError>{
		match self.timers.insert(UserTimer {
			delay: ms,
			protocol: self.protocol,
		}) {
			Ok(token) => {
				self.event_loop.timeout_ms(token, ms).expect("Error registering user timer");
				Ok(token.as_usize())
			},
			_ => { panic!("Max timers reached") }
		}
	}

	/// Broadcast a message to other IO clients
	pub fn message(&mut self, id: UserMessageId, data: Option<Vec<u8>>) {
		match self.event_loop.channel().send(HostMessage::UserMessage(UserMessage {
			protocol: self.protocol,
			id: id,
			data: data
		})) {
			Ok(_) => {}
			Err(e) => { panic!("Error sending io message {:?}", e); }
		}
	}

	/// Disable current protocol capability for given peer. If no capabilities left peer gets disconnected.
	pub fn disable_peer(&mut self, _peer: PeerId) {
		//TODO: remove capability, disconnect if no capabilities left
	}

}

struct UserTimer {
	protocol: ProtocolId,
	delay: u64,
}

pub struct HostInfo {
	keys: KeyPair,
	config: NetworkConfiguration,
	nonce: H256,
	pub protocol_version: u32,
	pub client_version: String,
	pub listen_port: u16,
	pub capabilities: Vec<CapabilityInfo>
}

impl HostInfo {
	pub fn id(&self) -> &NodeId {
		self.keys.public()
	}

	pub fn secret(&self) -> &Secret {
		self.keys.secret()
	}
	pub fn next_nonce(&mut self) -> H256 {
		self.nonce = self.nonce.sha3();
		return self.nonce.clone();
	}
}

enum ConnectionEntry {
	Handshake(Handshake),
	Session(Session)
}

pub struct Host {
	info: HostInfo,
	_udp_socket: UdpSocket,
	_listener: TcpListener,
	connections: Slab<ConnectionEntry>,
	timers: Slab<UserTimer>,
	nodes: HashMap<NodeId, Node>,
	handlers: HashMap<ProtocolId, Box<ProtocolHandler>>,
	_idle_timeout: Timeout,
}

impl Host {
	pub fn start(event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		let config = NetworkConfiguration::new();
		/*
		match ::ifaces::Interface::get_all().unwrap().into_iter().filter(|x| x.kind == ::ifaces::Kind::Packet && x.addr.is_some()).next() {
		Some(iface) => config.public_address = iface.addr.unwrap(),
		None => warn!("No public network interface"),
		*/

		let addr = config.listen_address;
		// Setup the server socket
		let listener = TcpListener::bind(&addr).unwrap();
		// Start listening for incoming connections
		event_loop.register(&listener, Token(TCP_ACCEPT), EventSet::readable(), PollOpt::edge()).unwrap();
		let idle_timeout = event_loop.timeout_ms(Token(IDLE), 1000).unwrap(); //TODO: check delay
		// open the udp socket
		let udp_socket = UdpSocket::bound(&addr).unwrap();
		event_loop.register(&udp_socket, Token(NODETABLE_RECEIVE), EventSet::readable(), PollOpt::edge()).unwrap();
		event_loop.timeout_ms(Token(NODETABLE_MAINTAIN), 7200).unwrap();
		let port = config.listen_address.port();

		let mut host = Host {
			info: HostInfo {
				keys: KeyPair::create().unwrap(),
				config: config,
				nonce: H256::random(),
				protocol_version: 4,
				client_version: "parity".to_string(),
				listen_port: port,
				capabilities: Vec::new(),
			},
			_udp_socket: udp_socket,
			_listener: listener,
			connections: Slab::new_starting_at(Token(FIRST_CONNECTION), MAX_CONNECTIONS),
			timers: Slab::new_starting_at(Token(USER_TIMER), MAX_USER_TIMERS),
			nodes: HashMap::new(),
			handlers: HashMap::new(),
			_idle_timeout: idle_timeout,
		};

		host.add_node("enode://c022e7a27affdd1632f2e67dffeb87f02bf506344bb142e08d12b28e7e5c6e5dbb8183a46a77bff3631b51c12e8cf15199f797feafdc8834aaf078ad1a2bcfa0@127.0.0.1:30303");
		host.add_node("enode://5374c1bff8df923d3706357eeb4983cd29a63be40a269aaa2296ee5f3b2119a8978c0ed68b8f6fc84aad0df18790417daadf91a4bfbb786a16c9b0a199fa254a@gav.ethdev.com:30300");
		host.add_node("enode://e58d5e26b3b630496ec640f2530f3e7fa8a8c7dfe79d9e9c4aac80e3730132b869c852d3125204ab35bb1b1951f6f2d40996c1034fd8c5a69b383ee337f02ddc@gav.ethdev.com:30303");
		host.add_node("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@52.16.188.185:30303");
		host.add_node("enode://7f25d3eab333a6b98a8b5ed68d962bb22c876ffcd5561fca54e3c2ef27f754df6f7fd7c9b74cc919067abac154fb8e1f8385505954f161ae440abc355855e034@54.207.93.166:30303");
		host.add_node("enode://5374c1bff8df923d3706357eeb4983cd29a63be40a269aaa2296ee5f3b2119a8978c0ed68b8f6fc84aad0df18790417daadf91a4bfbb786a16c9b0a199fa254a@92.51.165.126:30303");

		try!(event_loop.run(&mut host));
		Ok(())
	}

	fn add_node(&mut self, id: &str) {
		match Node::from_str(id) {
			Err(e) => { warn!("Could not add node: {:?}", e); },
			Ok(n) => {
				self.nodes.insert(n.id.clone(), n);
			}
		}
	}

	fn maintain_network(&mut self, event_loop: &mut EventLoop<Host>) {
		self.connect_peers(event_loop);
	}

	fn have_session(&self, id: &NodeId) -> bool {
		self.connections.iter().any(|e| match e { &ConnectionEntry::Session(ref s) => s.info.id.eq(&id), _ => false  })
	}

	fn connecting_to(&self, id: &NodeId) -> bool {
		self.connections.iter().any(|e| match e { &ConnectionEntry::Handshake(ref h) => h.id.eq(&id), _ => false  })
	}

	fn connect_peers(&mut self, event_loop: &mut EventLoop<Host>) {

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

		for n in to_connect.iter() {
			if n.peer_type == PeerType::Required {
				if req_conn < IDEAL_PEERS {
					self.connect_peer(&n.id, event_loop);
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
				for n in to_connect.iter() {
					if n.peer_type == PeerType::Optional && open_slots > 0 {
						open_slots -= 1;
						self.connect_peer(&n.id, event_loop);
					}
				}
			}
		}
	}

	fn connect_peer(&mut self, id: &NodeId, event_loop: &mut EventLoop<Host>) {
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
		match self.connections.insert_with(|token| ConnectionEntry::Handshake(Handshake::new(token, id, socket, &nonce).expect("Can't create handshake"))) {
			Some(token) => {
				match self.connections.get_mut(token) {
					Some(&mut ConnectionEntry::Handshake(ref mut h)) => {
						h.start(&self.info, true)
							.and_then(|_| h.register(event_loop))
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


	fn accept(&mut self, _event_loop: &mut EventLoop<Host>) {
		warn!(target: "net", "accept");
	}

	fn connection_writable(&mut self, token: Token, event_loop: &mut EventLoop<Host>) {
		let mut kill = false;
		let mut create_session = false;
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Handshake(ref mut h)) => {
				h.writable(event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Handshake write error: {:?}", e);
					kill = true;
				});
				create_session = h.done();
			},
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				s.writable(event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Session write error: {:?}", e);
					kill = true;
				});
			}
			_ => {
				warn!(target: "net", "Received event for unknown connection");
			}
		}
		if kill {
			self.kill_connection(token, event_loop);
		}
		if create_session {
			self.start_session(token, event_loop);
		}
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				s.reregister(event_loop).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
			},
			_ => (),
		}
	}

	fn connection_closed(&mut self, token: Token, event_loop: &mut EventLoop<Host>) {
		self.kill_connection(token, event_loop);
	}

	fn connection_readable(&mut self, token: Token, event_loop: &mut EventLoop<Host>) {
		let mut kill = false;
		let mut create_session = false;
		let mut ready_data: Vec<ProtocolId> = Vec::new();
		let mut packet_data: Option<(ProtocolId, PacketId, Vec<u8>)> = None;
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Handshake(ref mut h)) => {
				h.readable(event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Handshake read error: {:?}", e);
					kill = true;
				});
				create_session = h.done();
			},
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				let sd = { s.readable(event_loop, &self.info).unwrap_or_else(|e| {
					debug!(target: "net", "Session read error: {:?}", e);
					kill = true;
					SessionData::None
				}) };
				match sd {
					SessionData::Ready => {
						for (p, _) in self.handlers.iter_mut() {
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
			self.kill_connection(token, event_loop);
		}
		if create_session {
			self.start_session(token, event_loop);
		}
		for p in ready_data {
			let mut h = self.handlers.get_mut(p).unwrap();
			h.connected(&mut HostIo::new(p, Some(token), event_loop, &mut self.connections, &mut self.timers), &token.as_usize());
		}
		if let Some((p, packet_id, data)) = packet_data {
			let mut h = self.handlers.get_mut(p).unwrap();
			h.read(&mut HostIo::new(p, Some(token), event_loop, &mut self.connections, &mut self.timers), &token.as_usize(), packet_id, &data[1..]);
		}

		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Session(ref mut s)) => {
				s.reregister(event_loop).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
			},
			_ => (),
		}
	}

	fn start_session(&mut self, token: Token, event_loop: &mut EventLoop<Host>) {
		let info = &self.info;
		self.connections.replace_with(token, |c| {
			match c {
				ConnectionEntry::Handshake(h) => Session::new(h, event_loop, info)
					.map(|s| Some(ConnectionEntry::Session(s)))
					.unwrap_or_else(|e| {
						debug!(target: "net", "Session construction error: {:?}", e);
						None
					}),
					_ => { panic!("No handshake to create a session from"); }
			}
		}).expect("Error updating slab with session");
	}

	fn connection_timeout(&mut self, token: Token, event_loop: &mut EventLoop<Host>) {
		self.kill_connection(token, event_loop)
	}

	fn kill_connection(&mut self, token: Token, event_loop: &mut EventLoop<Host>) {
		let mut to_disconnect: Vec<ProtocolId> = Vec::new();
		match self.connections.get_mut(token) {
			Some(&mut ConnectionEntry::Handshake(_)) => (), // just abandon handshake
			Some(&mut ConnectionEntry::Session(ref mut s)) if s.is_ready() => {
				for (p, _) in self.handlers.iter_mut() {
					if s.have_capability(p)  {
						to_disconnect.push(p);
					}
				}
			},
			_ => (),
		}
		for p in to_disconnect {
			let mut h = self.handlers.get_mut(p).unwrap();
			h.disconnected(&mut HostIo::new(p, Some(token), event_loop, &mut self.connections, &mut self.timers), &token.as_usize());
		}
		self.connections.remove(token);
	}
}

impl Handler for Host {
	type Timeout = Token;
	type Message = HostMessage;

	fn ready(&mut self, event_loop: &mut EventLoop<Host>, token: Token, events: EventSet) {
		if events.is_hup() {
			trace!(target: "net", "hup");
			match token.as_usize() {
				FIRST_CONNECTION ... LAST_CONNECTION => self.connection_closed(token, event_loop),
				_ => warn!(target: "net", "Unexpected hup"),
			};
		}
		else if events.is_readable() {
			match token.as_usize() {
				TCP_ACCEPT => self.accept(event_loop),
				IDLE => self.maintain_network(event_loop),
				FIRST_CONNECTION ... LAST_CONNECTION => self.connection_readable(token, event_loop),
				NODETABLE_RECEIVE => {},
				_ => panic!("Received unknown readable token"),
			}
		}
		else if events.is_writable() {
			match token.as_usize() {
				FIRST_CONNECTION ... LAST_CONNECTION => self.connection_writable(token, event_loop),
				_ => panic!("Received unknown writable token"),
			}
		}
	}

	fn timeout(&mut self, event_loop: &mut EventLoop<Host>, token: Token) {
		match token.as_usize() {
			IDLE => self.maintain_network(event_loop),
			FIRST_CONNECTION ... LAST_CONNECTION => self.connection_timeout(token, event_loop),
			NODETABLE_DISCOVERY => {},
			NODETABLE_MAINTAIN => {},
			USER_TIMER ... LAST_USER_TIMER => {
				let (protocol, delay) = {
					let timer = self.timers.get_mut(token).expect("Unknown user timer token");
					(timer.protocol, timer.delay)
				};
				match self.handlers.get_mut(protocol) {
					None => { warn!(target: "net", "No handler found for protocol: {:?}", protocol) },
					Some(h) => {
						h.timeout(&mut HostIo::new(protocol, None, event_loop, &mut self.connections, &mut self.timers), token.as_usize());
						event_loop.timeout_ms(token, delay).expect("Error re-registering user timer");
					}
				}
			}
			_ => panic!("Unknown timer token"),
		}
	}

	fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
		match msg {
			HostMessage::Shutdown => event_loop.shutdown(),
			HostMessage::AddHandler {
				handler,
				protocol,
				versions
			} => {
				self.handlers.insert(protocol, handler);
				for v in versions {
					self.info.capabilities.push(CapabilityInfo { protocol: protocol, version: v, packet_count:0 });
				}
			},
			HostMessage::Send {
				peer,
				packet_id,
				protocol,
				data,
			} => {
				match self.connections.get_mut(Token(peer as usize)) {
					Some(&mut ConnectionEntry::Session(ref mut s)) => {
						s.send_packet(protocol, packet_id as u8, &data).unwrap_or_else(|e| {
							warn!(target: "net", "Send error: {:?}", e);
						}); //TODO: don't copy vector data
					},
					_ => {
						warn!(target: "net", "Send: Peer does not exist");
					}
				}
			},
			HostMessage::UserMessage(message) => {
				for (p, h) in self.handlers.iter_mut() {
					if p != &message.protocol {
						h.message(&mut HostIo::new(message.protocol, None, event_loop, &mut self.connections, &mut self.timers), &message);
					}
				}
			}
		}
	}
}
