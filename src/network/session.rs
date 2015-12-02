#![allow(dead_code)] //TODO: remove this after everything is done
//TODO: hello packet timeout
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

pub struct SessionInfo {
	pub id: NodeId,
	pub client_version: String,
	pub protocol_version: u32,
	pub capabilities: Vec<CapabilityInfo>,
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
		let mut connection = try!(EncryptedConnection::new(h));
		try!(connection.register(event_loop));
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

	pub fn readable(&mut self, event_loop: &mut EventLoop<Host>, host: &HostInfo) -> Result<(), Error> {
		match try!(self.connection.readable(event_loop)) {
			Some(data)  => { 
				try!(self.read_packet(data, host)); 
			},
			None => {}
		};
		Ok(())
	}

	pub fn writable(&mut self, event_loop: &mut EventLoop<Host>, _host: &HostInfo) -> Result<(), Error> {
		self.connection.writable(event_loop)
	}

	pub fn read_packet(&mut self, packet: Packet, host: &HostInfo) -> Result<(), Error> {
		let data = &packet.data;
		if data.len() < 2 {
			return Err(Error::BadProtocol);
		}
		let packet_id = data[0];
		let rlp = UntrustedRlp::new(&data[1..]); //TODO: validate rlp expected size
		if packet_id != PACKET_HELLO && packet_id != PACKET_DISCONNECT && !self.had_hello {
			return Err(Error::BadProtocol);
		}
		match packet_id {
			PACKET_HELLO => self.read_hello(&rlp, host),
			PACKET_DISCONNECT => Err(Error::Disconnect(DisconnectReason::DisconnectRequested)),
			PACKET_PING => self.write_pong(),
			PACKET_GET_PEERS => Ok(()), //TODO;
			PACKET_PEERS => Ok(()),
			PACKET_USER ... PACKET_LAST => {
				warn!(target: "net", "User packet: {:?}", rlp);
				Ok(())
			},
			_ => {
				debug!(target: "net", "Unkown packet: {:?}", rlp);
				Ok(())
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
		let client_version = try!(String::decode_untrusted(&try!(rlp.at(0))));
		let mut caps: Vec<CapabilityInfo> = try!(Decodable::decode_untrusted(&try!(rlp.at(2))));
		let id = try!(NodeId::decode_untrusted(&try!(rlp.at(4))));

		// Intersect with host capabilities
		// Leave only highset mutually supported capability version
		caps.retain(|c| host.capabilities.contains(&c));
		let mut i = 0;
		while i < caps.len() {
			if caps.iter().any(|c| c.protocol == caps[i].protocol && c.version > caps[i].version) {
				caps.remove(i);
			}
			else {
				i += 1;
			}
		}

		trace!(target: "net", "Hello: {} v{} {} {:?}", client_version, protocol, id, caps);
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

