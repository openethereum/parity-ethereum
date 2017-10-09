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

use std::{str, io};
use std::net::SocketAddr;
use std::cmp::Ordering;
use std::sync::*;
use std::collections::HashMap;

use mio::*;
use mio::deprecated::{Handler, EventLoop};
use mio::tcp::*;
use bigint::hash::*;
use rlp::*;
use connection::{EncryptedConnection, Packet, Connection};
use handshake::Handshake;
use io::{IoContext, StreamToken};
use error::{NetworkError, DisconnectReason};
use host::*;
use node_table::NodeId;
use stats::NetworkStats;
use time;

// Timeout must be less than (interval - 1).
const PING_TIMEOUT_SEC: u64 = 60;
const PING_INTERVAL_SEC: u64 = 120;

#[derive(Debug, Clone)]
enum ProtocolState {
	// Packets pending protocol on_connect event return.
	Pending(Vec<(Vec<u8>, u8)>),
	// Protocol connected.
	Connected,
}

/// Peer session over encrypted connection.
/// When created waits for Hello packet exchange and signals ready state.
/// Sends and receives protocol packets and handles basic packes such as ping/pong and disconnect.
pub struct Session {
	/// Shared session information
	pub info: SessionInfo,
	/// Session ready flag. Set after successfull Hello packet exchange
	had_hello: bool,
	/// Session is no longer active flag.
	expired: bool,
	ping_time_ns: u64,
	pong_time_ns: Option<u64>,
	state: State,
	// Protocol states -- accumulates pending packets until signaled as ready.
	protocol_states: HashMap<ProtocolId, ProtocolState>,
}

enum State {
	Handshake(Handshake),
	Session(EncryptedConnection),
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
		protocol: [u8; 3],
		/// Zero based packet ID
		packet_id: u8,
	},
	/// Session has more data to be read
	Continue,
}

/// Shared session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
	/// Peer public key
	pub id: Option<NodeId>,
	/// Peer client ID
	pub client_version: String,
	/// Peer RLPx protocol version
	pub protocol_version: u32,
	/// Session protocol capabilities
	pub capabilities: Vec<SessionCapabilityInfo>,
	/// Peer protocol capabilities
	pub peer_capabilities: Vec<PeerCapabilityInfo>,
	/// Peer ping delay in milliseconds
	pub ping_ms: Option<u64>,
	/// True if this session was originated by us.
	pub originated: bool,
	/// Remote endpoint address of the session
	pub remote_address: String,
	/// Local endpoint address of the session
	pub local_address: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerCapabilityInfo {
	pub protocol: ProtocolId,
	pub version: u8,
}

impl Decodable for PeerCapabilityInfo {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		let p: Vec<u8> = rlp.val_at(0)?;
		if p.len() != 3 {
			return Err(DecoderError::Custom("Invalid subprotocol string length. Should be 3"));
		}
		let mut p2: ProtocolId = [0u8; 3];
		p2.clone_from_slice(&p);
		Ok(PeerCapabilityInfo {
			protocol: p2,
			version: rlp.val_at(1)?
		})
	}
}

impl ToString for PeerCapabilityInfo {
	fn to_string(&self) -> String {
		format!("{}/{}", str::from_utf8(&self.protocol[..]).unwrap_or("???"), self.version)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCapabilityInfo {
	pub protocol: [u8; 3],
	pub version: u8,
	pub packet_count: u8,
	pub id_offset: u8,
}

impl PartialOrd for SessionCapabilityInfo {
	fn partial_cmp(&self, other: &SessionCapabilityInfo) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for SessionCapabilityInfo {
	fn cmp(&self, b: &SessionCapabilityInfo) -> Ordering {
		// By protocol id first
		if self.protocol != b.protocol {
			return self.protocol.cmp(&b.protocol);
		}
		// By version
		self.version.cmp(&b.version)
	}
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
	pub fn new<Message>(io: &IoContext<Message>, socket: TcpStream, token: StreamToken, id: Option<&NodeId>,
		nonce: &H256, stats: Arc<NetworkStats>, host: &HostInfo) -> Result<Session, NetworkError>
		where Message: Send + Clone + Sync + 'static {
		let originated = id.is_some();
		let mut handshake = Handshake::new(token, id, socket, nonce, stats).expect("Can't create handshake");
		let local_addr = handshake.connection.local_addr_str();
		handshake.start(io, host, originated)?;
		Ok(Session {
			state: State::Handshake(handshake),
			had_hello: false,
			info: SessionInfo {
				id: id.cloned(),
				client_version: String::new(),
				protocol_version: 0,
				capabilities: Vec::new(),
				peer_capabilities: Vec::new(),
				ping_ms: None,
				originated: originated,
				remote_address: "Handshake".to_owned(),
				local_address: local_addr,
			},
			ping_time_ns: 0,
			pong_time_ns: None,
			expired: false,
			protocol_states: HashMap::new(),
		})
	}

	fn complete_handshake<Message>(&mut self, io: &IoContext<Message>, host: &HostInfo) -> Result<(), NetworkError> where Message: Send + Sync + Clone {
		let connection = if let State::Handshake(ref mut h) = self.state {
			self.info.id = Some(h.id.clone());
			self.info.remote_address = h.connection.remote_addr_str();
			EncryptedConnection::new(h)?
		} else {
			panic!("Unexpected state");
		};
		self.state = State::Session(connection);
		self.write_hello(io, host)?;
		self.send_ping(io)?;
		Ok(())
	}

	fn connection(&self) -> &Connection {
		match self.state {
			State::Handshake(ref h) => &h.connection,
			State::Session(ref s) => &s.connection,
		}
	}

	/// Get id of the remote peer
	pub fn id(&self) -> Option<&NodeId> {
		self.info.id.as_ref()
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
		match self.state {
			State::Handshake(ref h) => self.expired || h.expired(),
			_ => self.expired,
		}
	}

	/// Check if this session is over and there is nothing to be sent.
	pub fn done(&self) -> bool {
		self.expired() && !self.connection().is_sending()
	}

	/// Get remote peer address
	pub fn remote_addr(&self) -> io::Result<SocketAddr> {
		self.connection().remote_addr()
	}

	/// Readable IO handler. Returns packet data if available.
	pub fn readable<Message>(&mut self, io: &IoContext<Message>, host: &HostInfo) -> Result<SessionData, NetworkError>  where Message: Send + Sync + Clone {
		if self.expired() {
			return Ok(SessionData::None)
		}
		let mut create_session = false;
		let mut packet_data = None;
		match self.state {
			State::Handshake(ref mut h) => {
				h.readable(io, host)?;
				if h.done() {
					create_session = true;
				}
			}
			State::Session(ref mut c) => {
				match c.readable(io)? {
					data @ Some(_) => packet_data = data,
					None => return Ok(SessionData::None)
				}
			}
		}
		if let Some(data) = packet_data {
			return Ok(self.read_packet(io, data, host)?);
		}
		if create_session {
			self.complete_handshake(io, host)?;
            io.update_registration(self.token()).unwrap_or_else(|e| debug!(target: "network", "Token registration error: {:?}", e));
		}
		Ok(SessionData::None)
	}

	/// Writable IO handler. Sends pending packets.
	pub fn writable<Message>(&mut self, io: &IoContext<Message>, _host: &HostInfo) -> Result<(), NetworkError> where Message: Send + Sync + Clone {
		match self.state {
			State::Handshake(ref mut h) => h.writable(io),
			State::Session(ref mut s) => s.writable(io),
		}
	}

	/// Checks if peer supports given capability
	pub fn have_capability(&self, protocol: [u8; 3]) -> bool {
		self.info.capabilities.iter().any(|c| c.protocol == protocol)
	}

	/// Checks if peer supports given capability
	pub fn capability_version(&self, protocol: [u8; 3]) -> Option<u8> {
		self.info.capabilities.iter().filter_map(|c| if c.protocol == protocol { Some(c.version) } else { None }).max()
	}

	/// Register the session socket with the event loop
	pub fn register_socket<Host:Handler<Timeout = Token>>(&self, reg: Token, event_loop: &mut EventLoop<Host>) -> Result<(), NetworkError> {
		if self.expired() {
			return Ok(());
		}
		self.connection().register_socket(reg, event_loop)?;
		Ok(())
	}

	/// Update registration with the event loop. Should be called at the end of the IO handler.
	pub fn update_socket<Host:Handler>(&self, reg:Token, event_loop: &mut EventLoop<Host>) -> Result<(), NetworkError> {
		self.connection().update_socket(reg, event_loop)?;
		Ok(())
	}

	/// Delete registration
	pub fn deregister_socket<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), NetworkError> {
		self.connection().deregister_socket(event_loop)?;
		Ok(())
	}

	/// Send a protocol packet to peer.
	pub fn send_packet<Message>(&mut self, io: &IoContext<Message>, protocol: [u8; 3], packet_id: u8, data: &[u8]) -> Result<(), NetworkError>
        where Message: Send + Sync + Clone {
		if self.info.capabilities.is_empty() || !self.had_hello {
			debug!(target: "network", "Sending to unconfirmed session {}, protocol: {}, packet: {}", self.token(), str::from_utf8(&protocol[..]).unwrap_or("??"), packet_id);
			return Err(From::from(NetworkError::BadProtocol));
		}
		if self.expired() {
			return Err(From::from(NetworkError::Expired));
		}
		let mut i = 0usize;
		while protocol != self.info.capabilities[i].protocol {
			i += 1;
			if i == self.info.capabilities.len() {
				debug!(target: "network", "Unknown protocol: {:?}", protocol);
				return Ok(())
			}
		}
		let pid = self.info.capabilities[i].id_offset + packet_id;
		let mut rlp = RlpStream::new();
		rlp.append(&(pid as u32));
		rlp.append_raw(data, 1);
		self.send(io, rlp)
	}

	/// Keep this session alive. Returns false if ping timeout happened
	pub fn keep_alive<Message>(&mut self, io: &IoContext<Message>) -> bool where Message: Send + Sync + Clone {
		if let State::Handshake(_) = self.state {
			return true;
		}
		let timed_out = if let Some(pong) = self.pong_time_ns {
			pong - self.ping_time_ns > PING_TIMEOUT_SEC * 1000_000_000
		} else {
			time::precise_time_ns() - self.ping_time_ns > PING_TIMEOUT_SEC * 1000_000_000
		};

		if !timed_out && time::precise_time_ns() - self.ping_time_ns > PING_INTERVAL_SEC * 1000_000_000 {
			if let Err(e) = self.send_ping(io) {
				debug!("Error sending ping message: {:?}", e);
			}
		}
		!timed_out
	}

	pub fn token(&self) -> StreamToken {
		self.connection().token()
	}

	/// Signal that a subprotocol has handled the connection successfully and
	/// get all pending packets in order received.
	pub fn mark_connected(&mut self, protocol: ProtocolId) -> Vec<(ProtocolId, u8, Vec<u8>)> {
		match self.protocol_states.insert(protocol, ProtocolState::Connected) {
			None => Vec::new(),
			Some(ProtocolState::Connected) => {
				debug!(target: "network", "Protocol {:?} marked as connected more than once", protocol);
				Vec::new()
			}
			Some(ProtocolState::Pending(pending)) =>
				pending.into_iter().map(|(data, id)| (protocol, id, data)).collect(),
		}
	}

	fn read_packet<Message>(&mut self, io: &IoContext<Message>, packet: Packet, host: &HostInfo) -> Result<SessionData, NetworkError>
	where Message: Send + Sync + Clone {
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
				self.read_hello(io, &rlp, host)?;
				Ok(SessionData::Ready)
			},
			PACKET_DISCONNECT => {
				let rlp = UntrustedRlp::new(&packet.data[1..]);
				let reason: u8 = rlp.val_at(0)?;
				if self.had_hello {
					debug!(target:"network", "Disconnected: {}: {:?}", self.token(), DisconnectReason::from_u8(reason));
				}
				Err(From::from(NetworkError::Disconnect(DisconnectReason::from_u8(reason))))
			}
			PACKET_PING => {
				self.send_pong(io)?;
				Ok(SessionData::Continue)
			},
			PACKET_PONG => {
				let time = time::precise_time_ns();
				self.pong_time_ns = Some(time);
				self.info.ping_ms = Some((time - self.ping_time_ns) / 1000_000);
				Ok(SessionData::Continue)
			},
			PACKET_GET_PEERS => Ok(SessionData::None), //TODO;
			PACKET_PEERS => Ok(SessionData::None),
			PACKET_USER ... PACKET_LAST => {
				let mut i = 0usize;
				while packet_id >= self.info.capabilities[i].id_offset + self.info.capabilities[i].packet_count {
					i += 1;
					if i == self.info.capabilities.len() {
						debug!(target: "network", "Unknown packet: {:?}", packet_id);
						return Ok(SessionData::Continue)
					}
				}

				// map to protocol
				let protocol = self.info.capabilities[i].protocol;
				let protocol_packet_id = packet_id - self.info.capabilities[i].id_offset;

				match *self.protocol_states.entry(protocol).or_insert_with(|| ProtocolState::Pending(Vec::new())) {
					ProtocolState::Connected => {
						trace!(target: "network", "Packet {} mapped to {:?}:{}, i={}, capabilities={:?}", packet_id, protocol, protocol_packet_id, i, self.info.capabilities);
						Ok(SessionData::Packet { data: packet.data, protocol: protocol, packet_id: protocol_packet_id } )
					}
					ProtocolState::Pending(ref mut pending) => {
						trace!(target: "network", "Packet {} deferred until protocol connection event completion", packet_id);
						pending.push((packet.data, protocol_packet_id));

						Ok(SessionData::Continue)
					}
				}
			},
			_ => {
				debug!(target: "network", "Unknown packet: {:?}", packet_id);
				Ok(SessionData::Continue)
			}
		}
	}

	fn write_hello<Message>(&mut self, io: &IoContext<Message>, host: &HostInfo) -> Result<(), NetworkError> where Message: Send + Sync + Clone {
		let mut rlp = RlpStream::new();
		rlp.append_raw(&[PACKET_HELLO as u8], 0);
		rlp.begin_list(5)
			.append(&host.protocol_version)
			.append(&host.client_version)
			.append_list(&host.capabilities)
			.append(&host.local_endpoint.address.port())
			.append(host.id());
		self.send(io, rlp)
	}

	fn read_hello<Message>(&mut self, io: &IoContext<Message>, rlp: &UntrustedRlp, host: &HostInfo) -> Result<(), NetworkError>
	where Message: Send + Sync + Clone {
		let protocol = rlp.val_at::<u32>(0)?;
		let client_version = rlp.val_at::<String>(1)?;
		let peer_caps: Vec<PeerCapabilityInfo> = rlp.list_at(2)?;
		let id = rlp.val_at::<NodeId>(4)?;

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

		// Sort capabilities alphabeticaly.
		caps.sort();

		i = 0;
		let mut offset: u8 = PACKET_USER;
		while i < caps.len() {
			caps[i].id_offset = offset;
			offset += caps[i].packet_count;
			i += 1;
		}
		debug!(target: "network", "Hello: {} v{} {} {:?}", client_version, protocol, id, caps);
		let protocol = ::std::cmp::min(protocol, host.protocol_version);
		self.info.protocol_version = protocol;
		self.info.client_version = client_version;
		self.info.capabilities = caps;
		self.info.peer_capabilities = peer_caps;
		if self.info.capabilities.is_empty() {
			trace!(target: "network", "No common capabilities with peer.");
			return Err(From::from(self.disconnect(io, DisconnectReason::UselessPeer)));
		}
		if protocol != host.protocol_version {
			trace!(target: "network", "Peer protocol version mismatch: {}", protocol);
			return Err(From::from(self.disconnect(io, DisconnectReason::UselessPeer)));
		}
		self.had_hello = true;
		Ok(())
	}

	/// Senf ping packet
	pub fn send_ping<Message>(&mut self, io: &IoContext<Message>) -> Result<(), NetworkError> where Message: Send + Sync + Clone {
		self.send(io, Session::prepare(PACKET_PING)?)?;
		self.ping_time_ns = time::precise_time_ns();
		self.pong_time_ns = None;
		Ok(())
	}

	fn send_pong<Message>(&mut self, io: &IoContext<Message>) -> Result<(), NetworkError> where Message: Send + Sync + Clone {
		self.send(io, Session::prepare(PACKET_PONG)?)
	}

	/// Disconnect this session
	pub fn disconnect<Message>(&mut self, io: &IoContext<Message>, reason: DisconnectReason) -> NetworkError where Message: Send + Sync + Clone {
		if let State::Session(_) = self.state {
			let mut rlp = RlpStream::new();
			rlp.append(&(PACKET_DISCONNECT as u32));
			rlp.begin_list(1);
			rlp.append(&(reason as u32));
			self.send(io, rlp).ok();
		}
		NetworkError::Disconnect(reason)
	}

	fn prepare(packet_id: u8) -> Result<RlpStream, NetworkError> {
		let mut rlp = RlpStream::new();
		rlp.append(&(packet_id as u32));
		rlp.begin_list(0);
		Ok(rlp)
	}

	fn send<Message>(&mut self, io: &IoContext<Message>, rlp: RlpStream) -> Result<(), NetworkError> where Message: Send + Sync + Clone {
		match self.state {
			State::Handshake(_) => {
				warn!(target:"network", "Unexpected send request");
			},
			State::Session(ref mut s) => {
				s.send_packet(io, &rlp.out())?
			},
		}
		Ok(())
	}
}

