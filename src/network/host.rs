#![allow(dead_code)] //TODO: remove this after everything is done
//TODO: remove all unwraps
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
use rlp::*;
use time::Tm;
use network::handshake::Handshake;
use network::session::Session;
use network::Error;

const DEFAULT_PORT: u16 = 30304;

const MAX_CONNECTIONS: usize = 1024;
const IDEAL_PEERS:u32 = 10;

pub type NodeId = H512;

#[derive(Debug)]
struct NetworkConfiguration {
    listen_address: SocketAddr,
    public_address: SocketAddr,
    no_nat: bool,
    no_discovery: bool,
    pin: bool,
}

impl NetworkConfiguration {
    fn new() -> NetworkConfiguration {
        NetworkConfiguration {
            listen_address: SocketAddr::from_str("0.0.0.0:30304").unwrap(),
            public_address: SocketAddr::from_str("0.0.0.0:30304").unwrap(),
            no_nat: false,
            no_discovery: false,
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
    fn new(address: SocketAddr) -> NodeEndpoint {
        NodeEndpoint {
            address: address,
			address_str: address.to_string(),
            udp_port: address.port()
        }
    }
    fn from_str(s: &str) -> Result<NodeEndpoint, Error> {
		println!("{:?}", s);
		let address = s.to_socket_addrs().map(|mut i| i.next());
		match address {
			Ok(Some(a)) => Ok(NodeEndpoint {
				address: a,
				address_str: s.to_string(),
				udp_port: a.port()
			}),
			Ok(_) => Err(Error::AddressResolve(None)),
			Err(e) => Err(Error::AddressResolve(Some(e)))
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
	confirmed: bool,
}

impl FromStr for Node {
	type Err = Error;
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
			confirmed: false
        })
	}
}

impl Node {
    fn new(id: NodeId, address: SocketAddr, t:PeerType) -> Node {
        Node {
            id: id,
            endpoint: NodeEndpoint::new(address),
            peer_type: t,
			last_attempted: None,
			confirmed: false
        }
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

pub enum HostMessage {
    Shutdown
}

#[derive(Debug, PartialEq, Eq)]
pub struct CapabilityInfo {
	pub protocol: String,
	pub version: u32,
}

impl Encodable for CapabilityInfo {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder {
		encoder.emit_list(|e| {
			self.protocol.encode(e);
			self.version.encode(e);
		});
	}
}

impl Decodable for CapabilityInfo {
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		Ok(CapabilityInfo {
			protocol: try!(String::decode_untrusted(&try!(rlp.at(0)))),
			version: try!(u32::decode_untrusted(&try!(rlp.at(1)))),
		})
	}
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
    sender: Sender<HostMessage>,
    udp_socket: UdpSocket,
    listener: TcpListener,
    connections: Slab<ConnectionEntry>,
	nodes: HashMap<NodeId, Node>,
	idle_timeout: Timeout,
}

impl Host {
    pub fn start() {
        let config = NetworkConfiguration::new();
		/*
		match ::ifaces::Interface::get_all().unwrap().into_iter().filter(|x| x.kind == ::ifaces::Kind::Packet && x.addr.is_some()).next() {
			Some(iface) => config.public_address = iface.addr.unwrap(),
			None => warn!("No public network interface"),
		}
		*/

        let addr = config.listen_address;
        // Setup the server socket
        let listener = TcpListener::bind(&addr).unwrap();
        // Create an event loop
        let mut event_loop = EventLoop::new().unwrap();
        let sender = event_loop.channel();
        // Start listening for incoming connections
        event_loop.register_opt(&listener, Token(TCP_ACCEPT), EventSet::readable(), PollOpt::edge()).unwrap();
        // Setup the client socket
        //let sock = TcpStream::connect(&addr).unwrap();
        // Register the socket
        //self.event_loop.register_opt(&sock, CLIENT, EventSet::readable(), PollOpt::edge()).unwrap();
        let idle_timeout = event_loop.timeout_ms(Token(IDLE), 1000).unwrap(); //TODO: check delay
        // open the udp socket
        let udp_socket = UdpSocket::bound(&addr).unwrap();
        event_loop.register_opt(&udp_socket, Token(NODETABLE_RECEIVE), EventSet::readable(), PollOpt::edge()).unwrap();
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
				capabilities: vec![ CapabilityInfo { protocol: "eth".to_string(), version: 63 }],
			},
            sender: sender,
            udp_socket: udp_socket,
            listener: listener,
			connections: Slab::new_starting_at(Token(FIRST_CONNECTION), MAX_CONNECTIONS),
			nodes: HashMap::new(),
			idle_timeout: idle_timeout
        };

		host.add_node("enode://c022e7a27affdd1632f2e67dffeb87f02bf506344bb142e08d12b28e7e5c6e5dbb8183a46a77bff3631b51c12e8cf15199f797feafdc8834aaf078ad1a2bcfa0@127.0.0.1:30303");
		host.add_node("enode://5374c1bff8df923d3706357eeb4983cd29a63be40a269aaa2296ee5f3b2119a8978c0ed68b8f6fc84aad0df18790417daadf91a4bfbb786a16c9b0a199fa254a@gav.ethdev.com:30300");
		host.add_node("enode://e58d5e26b3b630496ec640f2530f3e7fa8a8c7dfe79d9e9c4aac80e3730132b869c852d3125204ab35bb1b1951f6f2d40996c1034fd8c5a69b383ee337f02ddc@gav.ethdev.com:30303");
		host.add_node("enode://a979fb575495b8d6db44f750317d0f4622bf4c2aa3365d6af7c284339968eef29b69ad0dce72a4d8db5ebb4968de0e3bec910127f134779fbcb0cb6d3331163c@52.16.188.185:30303");
		host.add_node("enode://7f25d3eab333a6b98a8b5ed68d962bb22c876ffcd5561fca54e3c2ef27f754df6f7fd7c9b74cc919067abac154fb8e1f8385505954f161ae440abc355855e034@54.207.93.166:30303");
		host.add_node("enode://5374c1bff8df923d3706357eeb4983cd29a63be40a269aaa2296ee5f3b2119a8978c0ed68b8f6fc84aad0df18790417daadf91a4bfbb786a16c9b0a199fa254a@92.51.165.126:30303");

        event_loop.run(&mut host).unwrap();
    }

    fn stop(&mut self) {
    }

    fn have_network(&mut self) -> bool {
        true
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


			//blog(NetConnect) << "Attempting connection to node" << _p->id << "@" << ep << "from" << id();
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
		{
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
			};
		}
		if kill {
			self.kill_connection(token, event_loop);
		}
		if create_session {
			self.start_session(token, event_loop);
		}
	}
	fn connection_readable(&mut self, token: Token, event_loop: &mut EventLoop<Host>) {
		let mut kill = false;
		let mut create_session = false;
		{
			match self.connections.get_mut(token) {
				Some(&mut ConnectionEntry::Handshake(ref mut h)) => {
					h.readable(event_loop, &self.info).unwrap_or_else(|e| {
						debug!(target: "net", "Handshake read error: {:?}", e);
						kill = true;
					});
					create_session = h.done();
				},
				Some(&mut ConnectionEntry::Session(ref mut s)) => {
					s.readable(event_loop, &self.info).unwrap_or_else(|e| {
						debug!(target: "net", "Session read error: {:?}", e);
						kill = true;
					});
				}
				_ => {
					warn!(target: "net", "Received event for unknown connection");
				}
			};
		}
		if kill {
			self.kill_connection(token, event_loop);
		}
		if create_session {
			self.start_session(token, event_loop);
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
	fn kill_connection(&mut self, token: Token, _event_loop: &mut EventLoop<Host>) {
		self.connections.remove(token);
	}
}

impl Handler for Host {
    type Timeout = Token;
    type Message = HostMessage;

    fn ready(&mut self, event_loop: &mut EventLoop<Host>, token: Token, events: EventSet) {
        if events.is_readable() {
			match token.as_usize() {
				TCP_ACCEPT =>  self.accept(event_loop),
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
			_ => panic!("Received unknown timer token"),
		}
	}
}


#[cfg(test)]
mod tests {
    use network::host::Host;
	use env_logger;
    #[test]
	//#[ignore]
    fn net_connect() {
		env_logger::init().unwrap();
        let _ = Host::start();
    }
}



