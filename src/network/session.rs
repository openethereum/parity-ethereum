//#![allow(dead_code)] //TODO: remove this after everything is done

use mio::*;
use hash::*;
use rlp::*;
use network::connection::{EncryptedConnection, Packet};
use network::handshake::Handshake;
use network::{Error, DisconnectReason};
use network::host::*;

pub struct Session {
	pub info: SessionInfo,
	connection: EncryptedConnection,
	had_hello: bool,
}

pub enum SessionData {
	None,
	Ready,
	Packet {
		data: Vec<u8>,
		protocol: &'static str,
		packet_id: u8,
	},
}

pub struct SessionInfo {
	pub id: NodeId,
	pub client_version: String,
	pub protocol_version: u32,
	pub capabilities: Vec<SessionCapabilityInfo>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PeerCapabilityInfo {
	pub protocol: String,
	pub version: u8,
}

impl Decodable for PeerCapabilityInfo {
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		Ok(PeerCapabilityInfo {
			protocol: try!(String::decode_untrusted(&try!(rlp.at(0)))),
			version: try!(u32::decode_untrusted(&try!(rlp.at(1)))) as u8,
		})
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct SessionCapabilityInfo {
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
	pub fn new(h: Handshake, event_loop: &mut EventLoop<Host>, host: &HostInfo) -> Result<Session, Error> {
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
		try!(session.connection.register(event_loop));
		Ok(session)
	}

	pub fn readable(&mut self, event_loop: &mut EventLoop<Host>, host: &HostInfo) -> Result<SessionData, Error> {
		match try!(self.connection.readable(event_loop)) {
			Some(data)  => self.read_packet(data, host),
			None => Ok(SessionData::None)
		}
	}

	pub fn writable(&mut self, event_loop: &mut EventLoop<Host>, _host: &HostInfo) -> Result<(), Error> {
		self.connection.writable(event_loop)
	}

	pub fn have_capability(&self, protocol: &str) -> bool {
		self.info.capabilities.iter().any(|c| c.protocol == protocol)
	}

	pub fn send_packet(&mut self, protocol: &str, packet_id: u8, data: &[u8]) -> Result<(), Error> {
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

	fn read_packet(&mut self, packet: Packet, host: &HostInfo) -> Result<SessionData, Error> {
		if packet.data.len() < 2 {
			return Err(Error::BadProtocol);
		}
		let packet_id = packet.data[0];
		if packet_id != PACKET_HELLO && packet_id != PACKET_DISCONNECT && !self.had_hello {
			return Err(Error::BadProtocol);
		}
		match packet_id {
			PACKET_HELLO => {
			let rlp = UntrustedRlp::new(&packet.data[1..]); //TODO: validate rlp expected size
				try!(self.read_hello(&rlp, host));
				Ok(SessionData::Ready)
			}
			PACKET_DISCONNECT => Err(Error::Disconnect(DisconnectReason::DisconnectRequested)),
			PACKET_PING => {
				try!(self.write_pong());
				Ok(SessionData::None)
			}
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
				return Ok(SessionData::Packet { data: packet.data, protocol: protocol, packet_id: pid } )
			},
			_ => {
				debug!(target: "net", "Unkown packet: {:?}", packet_id);
				Ok(SessionData::None)
			}
		}
	}

	fn write_hello(&mut self, host: &HostInfo) -> Result<(), Error>  {
		let mut rlp = RlpStream::new();
		rlp.append(&(PACKET_HELLO as u32));
		rlp.append_list(5)
			.append(&host.protocol_version)
			.append(&host.client_version)
			.append(&host.capabilities)
			.append(&host.listen_port)
			.append(host.id());
		self.connection.send_packet(&rlp.out())
	}

	fn read_hello(&mut self, rlp: &UntrustedRlp, host: &HostInfo) -> Result<(), Error> {
		let protocol = try!(u32::decode_untrusted(&try!(rlp.at(0))));
		let client_version = try!(String::decode_untrusted(&try!(rlp.at(1))));
		let peer_caps: Vec<PeerCapabilityInfo> = try!(Decodable::decode_untrusted(&try!(rlp.at(2))));
		let id = try!(NodeId::decode_untrusted(&try!(rlp.at(4))));

		// Intersect with host capabilities
		// Leave only highset mutually supported capability version
		let mut caps: Vec<SessionCapabilityInfo> = Vec::new();
		for hc in host.capabilities.iter() {
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
		self.info.capabilities = caps;
		if protocol != host.protocol_version {
			return Err(self.disconnect(DisconnectReason::UselessPeer));
		}
		self.had_hello = true;
		Ok(())
	}

	fn write_ping(&mut self) -> Result<(), Error>  {
		self.send(try!(Session::prepare(PACKET_PING, 0)))
	}

	fn write_pong(&mut self) -> Result<(), Error>  {
		self.send(try!(Session::prepare(PACKET_PONG, 0)))
	}


	fn disconnect(&mut self, reason: DisconnectReason) -> Error {
		let mut rlp = RlpStream::new();
		rlp.append(&(PACKET_DISCONNECT as u32));
		rlp.append_list(1);
		rlp.append(&(reason.clone() as u32));
		self.connection.send_packet(&rlp.out()).ok();
		Error::Disconnect(reason)
	}

	fn prepare(packet_id: u8, items: usize) -> Result<RlpStream, Error> {
		let mut rlp = RlpStream::new_list(1);
		rlp.append(&(packet_id as u32));
		rlp.append_list(items);
		Ok(rlp)
	}

	fn send(&mut self, rlp: RlpStream) -> Result<(), Error> {
		self.connection.send_packet(&rlp.out())
	}
}

