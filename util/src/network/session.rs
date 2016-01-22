use mio::*;
use hash::*;
use rlp::*;
use network::connection::{EncryptedConnection, Packet};
use network::handshake::Handshake;
use error::*;
use io::{IoContext};
use network::error::{NetworkError, DisconnectReason};
use network::host::*;
use network::node::NodeId;

/// Peer session over encrypted connection.
/// When created waits for Hello packet exchange and signals ready state.
/// Sends and receives protocol packets and handles basic packes such as ping/pong and disconnect.
pub struct Session {
	/// Shared session information
	pub info: SessionInfo,
	/// Underlying connection
	connection: EncryptedConnection,
	/// Session ready flag. Set after successfull Hello packet exchange
	had_hello: bool,
}

/// Structure used to report various session events.
pub enum SessionData {
	None,
	/// Session is ready to send/receive packets.
	Ready,
	/// A packet has been received
	Packet {
		/// Packet data
		data: Vec<u8>,
		/// Packet protocol ID
		protocol: &'static str,
		/// Zero based packet ID 
		packet_id: u8,
	},
}

/// Shared session information
pub struct SessionInfo {
	/// Peer public key
	pub id: NodeId,
	/// Peer client ID
	pub client_version: String,
	/// Peer RLPx protocol version
	pub protocol_version: u32,
	/// Peer protocol capabilities
	capabilities: Vec<SessionCapabilityInfo>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PeerCapabilityInfo {
	pub protocol: String,
	pub version: u8,
}

impl Decodable for PeerCapabilityInfo {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let c = try!(decoder.as_list());
		let v: u32 = try!(Decodable::decode(&c[1]));
		Ok(PeerCapabilityInfo {
			protocol: try!(Decodable::decode(&c[0])),
			version: v as u8,
		})
	}
}

#[derive(Debug, PartialEq, Eq)]
struct SessionCapabilityInfo {
	pub protocol: &'static str,
	pub version: u8,
	pub packet_count: u8,
	pub id_offset: u8,
}

const PACKET_HELLO: u8 = 0x80;
const PACKET_DISCONNECT: u8 = 0x01;
const PACKET_PING: u8 = 0x02;
const PACKET_PONG: u8 = 0x03;
const PACKET_GET_PEERS: u8 = 0x04;
const PACKET_PEERS: u8 = 0x05;
const PACKET_USER: u8 = 0x10;
const PACKET_LAST: u8 = 0x7f;

impl Session {
	/// Create a new session out of comepleted handshake. Consumes handshake object.
	pub fn new<Message>(h: Handshake, _io: &IoContext<Message>, host: &HostInfo) -> Result<Session, UtilError> where Message: Send + Sync + Clone {
		let id = h.id.clone();
		let connection = try!(EncryptedConnection::new(h));
		let mut session = Session {
			connection: connection,
			had_hello: false,
			info: SessionInfo {
				id: id,
				client_version: String::new(),
				protocol_version: 0,
				capabilities: Vec::new(),
			},
		};
		try!(session.write_hello(host));
		try!(session.write_ping());
		Ok(session)
	}

	/// Check if session is ready to send/receive data
	pub fn is_ready(&self) -> bool {
		self.had_hello
	}

	/// Readable IO handler. Returns packet data if available.
	pub fn readable<Message>(&mut self, io: &IoContext<Message>, host: &HostInfo) -> Result<SessionData, UtilError>  where Message: Send + Sync + Clone {
		match try!(self.connection.readable(io)) {
			Some(data) => Ok(try!(self.read_packet(data, host))),
			None => Ok(SessionData::None)
		}
	}

	/// Writable IO handler. Sends pending packets.
	pub fn writable<Message>(&mut self, io: &IoContext<Message>, _host: &HostInfo) -> Result<(), UtilError> where Message: Send + Sync + Clone {
		self.connection.writable(io)
	}

	/// Checks if peer supports given capability
	pub fn have_capability(&self, protocol: &str) -> bool {
		self.info.capabilities.iter().any(|c| c.protocol == protocol)
	}

	/// Update registration with the event loop. Should be called at the end of the IO handler.
	pub fn update_socket<Host:Handler>(&self, reg:Token, event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		self.connection.update_socket(reg, event_loop)
	}

	/// Send a protocol packet to peer.
	pub fn send_packet(&mut self, protocol: &str, packet_id: u8, data: &[u8]) -> Result<(), UtilError> {
		let mut i = 0usize;
		while protocol != self.info.capabilities[i].protocol {
			i += 1;
			if i == self.info.capabilities.len() {
				debug!(target: "net", "Unkown protocol: {:?}", protocol);
				return Ok(())
			}
		}
		let pid = self.info.capabilities[i].id_offset + packet_id;
		let mut rlp = RlpStream::new();
		rlp.append(&(pid as u32));
		rlp.append_raw(data, 1);
		self.connection.send_packet(&rlp.out())
	}

	fn read_packet(&mut self, packet: Packet, host: &HostInfo) -> Result<SessionData, UtilError> {
		if packet.data.len() < 2 {
			return Err(From::from(NetworkError::BadProtocol));
		}
		let packet_id = packet.data[0];
		if packet_id != PACKET_HELLO && packet_id != PACKET_DISCONNECT && !self.had_hello {
			return Err(From::from(NetworkError::BadProtocol));
		}
		match packet_id {
			PACKET_HELLO => {
				let rlp = UntrustedRlp::new(&packet.data[1..]); //TODO: validate rlp expected size
				try!(self.read_hello(&rlp, host));
				Ok(SessionData::Ready)
			},
			PACKET_DISCONNECT => Err(From::from(NetworkError::Disconnect(DisconnectReason::DisconnectRequested))),
			PACKET_PING => {
				try!(self.write_pong());
				Ok(SessionData::None)
			},
			PACKET_GET_PEERS => Ok(SessionData::None), //TODO;
			PACKET_PEERS => Ok(SessionData::None),
			PACKET_USER ... PACKET_LAST => {
				let mut i = 0usize;
				while packet_id < self.info.capabilities[i].id_offset {
					i += 1;
					if i == self.info.capabilities.len() {
						debug!(target: "net", "Unkown packet: {:?}", packet_id);
						return Ok(SessionData::None)
					}
				}

				// map to protocol
				let protocol = self.info.capabilities[i].protocol;
				let pid = packet_id - self.info.capabilities[i].id_offset;
				Ok(SessionData::Packet { data: packet.data, protocol: protocol, packet_id: pid } )
			},
			_ => {
				debug!(target: "net", "Unkown packet: {:?}", packet_id);
				Ok(SessionData::None)
			}
		}
	}

	fn write_hello(&mut self, host: &HostInfo) -> Result<(), UtilError> {
		let mut rlp = RlpStream::new();
		rlp.append_raw(&[PACKET_HELLO as u8], 0);
		rlp.append_list(5)
			.append(&host.protocol_version)
			.append(&host.client_version)
			.append(&host.capabilities)
			.append(&host.listen_port)
			.append(host.id());
		self.connection.send_packet(&rlp.out())
	}

	fn read_hello(&mut self, rlp: &UntrustedRlp, host: &HostInfo) -> Result<(), UtilError> {
		let protocol = try!(rlp.val_at::<u32>(0));
		let client_version = try!(rlp.val_at::<String>(1));
		let peer_caps = try!(rlp.val_at::<Vec<PeerCapabilityInfo>>(2));
		let id = try!(rlp.val_at::<NodeId>(4));

		// Intersect with host capabilities
		// Leave only highset mutually supported capability version
		let mut caps: Vec<SessionCapabilityInfo> = Vec::new();
		for hc in &host.capabilities {
			if peer_caps.iter().any(|c| c.protocol == hc.protocol && c.version == hc.version) {
				caps.push(SessionCapabilityInfo {
					protocol: hc.protocol,
					version: hc.version,
					id_offset: 0,
					packet_count: hc.packet_count,
				});
			}
		}

		caps.retain(|c| host.capabilities.iter().any(|hc| hc.protocol == c.protocol && hc.version == c.version));
		let mut i = 0;
		while i < caps.len() {
			if caps.iter().any(|c| c.protocol == caps[i].protocol && c.version > caps[i].version) {
				caps.remove(i);
			}
			else {
				i += 1;
			}
		}

		i = 0;
		let mut offset: u8 = PACKET_USER;
		while i < caps.len() {
			caps[i].id_offset = offset;
			offset += caps[i].packet_count;
			i += 1;
		}
		trace!(target: "net", "Hello: {} v{} {} {:?}", client_version, protocol, id, caps);
		self.info.client_version = client_version;
		self.info.capabilities = caps;
		if protocol != host.protocol_version {
			return Err(From::from(self.disconnect(DisconnectReason::UselessPeer)));
		}
		self.had_hello = true;
		Ok(())
	}

	fn write_ping(&mut self) -> Result<(), UtilError>  {
		self.send(try!(Session::prepare(PACKET_PING)))
	}

	fn write_pong(&mut self) -> Result<(), UtilError>  {
		self.send(try!(Session::prepare(PACKET_PONG)))
	}

	fn disconnect(&mut self, reason: DisconnectReason) -> NetworkError {
		let mut rlp = RlpStream::new();
		rlp.append(&(PACKET_DISCONNECT as u32));
		rlp.append_list(1);
		rlp.append(&(reason.clone() as u32));
		self.connection.send_packet(&rlp.out()).ok();
		NetworkError::Disconnect(reason)
	}

	fn prepare(packet_id: u8) -> Result<RlpStream, UtilError> {
		let mut rlp = RlpStream::new();
		rlp.append(&(packet_id as u32));
		rlp.append_list(0);
		Ok(rlp)
	}

	fn send(&mut self, rlp: RlpStream) -> Result<(), UtilError> {
		self.connection.send_packet(&rlp.out())
	}
}

