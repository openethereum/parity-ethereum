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

use std::net::SocketAddr;
use std::io;
use mio::*;
use hash::*;
use rlp::*;
use network::connection::{EncryptedConnection, Packet};
use network::handshake::Handshake;
use error::*;
use io::{IoContext, StreamToken};
use network::error::{NetworkError, DisconnectReason};
use network::host::*;
use network::node_table::NodeId;
use time;

const PING_TIMEOUT_SEC: u64 = 30;
const PING_INTERVAL_SEC: u64 = 30;

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
	/// Session is no longer active flag.
	expired: bool,
	ping_time_ns: u64,
	pong_time_ns: Option<u64>,
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
	/// Peer ping delay in milliseconds
	pub ping_ms: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PeerCapabilityInfo {
	pub protocol: String,
	pub version: u8,
}

impl Decodable for PeerCapabilityInfo {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let c = decoder.as_rlp();
		Ok(PeerCapabilityInfo {
			protocol: try!(c.val_at(0)),
			version: try!(c.val_at(1))
		})
	}
}

#[derive(Debug)]
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
	/// Create a new session out of comepleted handshake. This clones the handshake connection object 
	/// and leaves the handhsake in limbo to be deregistered from the event loop.
	pub fn new(h: &mut Handshake, host: &HostInfo) -> Result<Session, UtilError> {
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
				ping_ms: None,
			},
			ping_time_ns: 0,
			pong_time_ns: None,
			expired: false,
		};
		try!(session.write_hello(host));
		try!(session.send_ping());
		Ok(session)
	}

	/// Get id of the remote peer
	pub fn id(&self) -> &NodeId {
		&self.info.id
	}

	/// Check if session is ready to send/receive data
	pub fn is_ready(&self) -> bool {
		self.had_hello
	}

	/// Mark this session as inactive to be deleted lated.
	pub fn set_expired(&mut self) {
		self.expired = true;
	}

	/// Check if this session is expired.
	pub fn expired(&self) -> bool {
		self.expired
	}

	/// Check if this session is over and there is nothing to be sent.
	pub fn done(&self) -> bool {
		self.expired() && !self.connection.is_sending()
	}
	/// Replace socket token 
	pub fn set_token(&mut self, token: StreamToken) {
		self.connection.set_token(token);
	}

	/// Get remote peer address
	pub fn remote_addr(&self) -> io::Result<SocketAddr> {
		self.connection.remote_addr()
	}

	/// Readable IO handler. Returns packet data if available.
	pub fn readable<Message>(&mut self, io: &IoContext<Message>, host: &HostInfo) -> Result<SessionData, UtilError>  where Message: Send + Sync + Clone {
		if self.expired() {
			return Ok(SessionData::None) 
		}
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

	/// Register the session socket with the event loop
	pub fn register_socket<Host:Handler<Timeout = Token>>(&self, reg: Token, event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		if self.expired() {
			return Ok(());
		}
		try!(self.connection.register_socket(reg, event_loop));
		Ok(())
	}

	/// Update registration with the event loop. Should be called at the end of the IO handler.
	pub fn update_socket<Host:Handler>(&self, reg:Token, event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		self.connection.update_socket(reg, event_loop)
	}

	/// Delete registration
	pub fn deregister_socket<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		self.connection.deregister_socket(event_loop)
	}

	/// Send a protocol packet to peer.
	pub fn send_packet(&mut self, protocol: &str, packet_id: u8, data: &[u8]) -> Result<(), UtilError> {
		if self.info.capabilities.is_empty() || !self.had_hello {
			debug!(target: "network", "Sending to unconfirmed session {}, protocol: {}, packet: {}", self.token(), protocol, packet_id);
			return Err(From::from(NetworkError::BadProtocol));
		}
		if self.expired() {
			return Err(From::from(NetworkError::Expired));
		}
		let mut i = 0usize;
		while protocol != self.info.capabilities[i].protocol {
			i += 1;
			if i == self.info.capabilities.len() {
				debug!(target: "net", "Unknown protocol: {:?}", protocol);
				return Ok(())
			}
		}
		let pid = self.info.capabilities[i].id_offset + packet_id;
		let mut rlp = RlpStream::new();
		rlp.append(&(pid as u32));
		rlp.append_raw(data, 1);
		self.connection.send_packet(&rlp.out())
	}

	/// Keep this session alive. Returns false if ping timeout happened
	pub fn keep_alive<Message>(&mut self, io: &IoContext<Message>) -> bool where Message: Send + Sync + Clone {
		let timed_out = if let Some(pong) = self.pong_time_ns {
			pong - self.ping_time_ns > PING_TIMEOUT_SEC * 1000_000_000
		} else {
			time::precise_time_ns() - self.ping_time_ns > PING_TIMEOUT_SEC * 1000_000_000
		};

		if !timed_out && time::precise_time_ns() - self.ping_time_ns > PING_INTERVAL_SEC * 1000_000_000 {
			if let Err(e) = self.send_ping() {
				debug!("Error sending ping message: {:?}", e);
			}
			io.update_registration(self.token()).unwrap_or_else(|e| debug!(target: "net", "Session registration error: {:?}", e));
		}
		!timed_out
	}

	pub fn token(&self) -> StreamToken {
		self.connection.token()
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
			PACKET_DISCONNECT => {
				let rlp = UntrustedRlp::new(&packet.data[1..]);
				let reason: u8 = try!(rlp.val_at(0));
				Err(From::from(NetworkError::Disconnect(DisconnectReason::from_u8(reason))))
			}
			PACKET_PING => {
				try!(self.send_pong());
				Ok(SessionData::None)
			},
			PACKET_PONG => {
				self.pong_time_ns = Some(time::precise_time_ns());
				self.info.ping_ms = Some((self.pong_time_ns.unwrap() - self.ping_time_ns) / 1000_000);
				Ok(SessionData::None)
			},
			PACKET_GET_PEERS => Ok(SessionData::None), //TODO;
			PACKET_PEERS => Ok(SessionData::None),
			PACKET_USER ... PACKET_LAST => {
				let mut i = 0usize;
				while packet_id < self.info.capabilities[i].id_offset {
					i += 1;
					if i == self.info.capabilities.len() {
						debug!(target: "net", "Unknown packet: {:?}", packet_id);
						return Ok(SessionData::None)
					}
				}

				// map to protocol
				let protocol = self.info.capabilities[i].protocol;
				let pid = packet_id - self.info.capabilities[i].id_offset;
				Ok(SessionData::Packet { data: packet.data, protocol: protocol, packet_id: pid } )
			},
			_ => {
				debug!(target: "net", "Unknown packet: {:?}", packet_id);
				Ok(SessionData::None)
			}
		}
	}

	fn write_hello(&mut self, host: &HostInfo) -> Result<(), UtilError> {
		let mut rlp = RlpStream::new();
		rlp.append_raw(&[PACKET_HELLO as u8], 0);
		rlp.begin_list(5)
			.append(&host.protocol_version)
			.append(&host.client_version)
			.append(&host.capabilities)
			.append(&host.local_endpoint.address.port())
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
		trace!(target: "network", "Hello: {} v{} {} {:?}", client_version, protocol, id, caps);
		self.info.client_version = client_version;
		self.info.capabilities = caps;
		if self.info.capabilities.is_empty() {
			trace!(target: "network", "No common capabilities with peer.");
			return Err(From::from(self.disconnect(DisconnectReason::UselessPeer)));
		}
		if protocol != host.protocol_version {
			trace!(target: "network", "Peer protocol version mismatch: {}", protocol);
			return Err(From::from(self.disconnect(DisconnectReason::UselessPeer)));
		}
		self.had_hello = true;
		Ok(())
	}

	/// Senf ping packet
	pub fn send_ping(&mut self) -> Result<(), UtilError>  {
		try!(self.send(try!(Session::prepare(PACKET_PING))));
		self.ping_time_ns = time::precise_time_ns();
		self.pong_time_ns = None;
		Ok(())
	}

	fn send_pong(&mut self) -> Result<(), UtilError>  {
		self.send(try!(Session::prepare(PACKET_PONG)))
	}

	/// Disconnect this session
	pub fn disconnect(&mut self, reason: DisconnectReason) -> NetworkError {
		let mut rlp = RlpStream::new();
		rlp.append(&(PACKET_DISCONNECT as u32));
		rlp.begin_list(1);
		rlp.append(&(reason as u32));
		self.connection.send_packet(&rlp.out()).ok();
		NetworkError::Disconnect(reason)
	}

	fn prepare(packet_id: u8) -> Result<RlpStream, UtilError> {
		let mut rlp = RlpStream::new();
		rlp.append(&(packet_id as u32));
		rlp.begin_list(0);
		Ok(rlp)
	}

	fn send(&mut self, rlp: RlpStream) -> Result<(), UtilError> {
		self.connection.send_packet(&rlp.out())
	}
}

