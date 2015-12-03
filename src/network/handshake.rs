use mio::*;
use mio::tcp::*;
use hash::*;
use bytes::Bytes;
use crypto::*;
use crypto;
use network::connection::{Connection, WriteStatus};
use network::host::{NodeId, Host, HostInfo};
use network::Error;

#[derive(PartialEq, Eq)]
enum HandshakeState {
	New,
	ReadingAuth,
	WritingAuth,
	ReadingAck,
	WritingAck,
	StartSession,
}

pub struct Handshake {
	pub id: NodeId,
	pub connection: Connection,
	state: HandshakeState,
	pub originated: bool,
	idle_timeout: Option<Timeout>,
	pub ecdhe: KeyPair,
	pub nonce: H256,
	pub remote_public: Public,
	pub remote_nonce: H256,
	pub auth_cipher: Bytes,
	pub ack_cipher: Bytes
}

const AUTH_PACKET_SIZE:usize = 307;
const ACK_PACKET_SIZE:usize = 210;

impl Handshake {
	pub fn new(token: Token, id: &NodeId, socket: TcpStream, nonce: &H256) -> Result<Handshake, Error> {
		Ok(Handshake {
			id: id.clone(),
			connection: Connection::new(token, socket),
			originated: false,
			state: HandshakeState::New,
			idle_timeout: None,
			ecdhe: try!(KeyPair::create()),
			nonce: nonce.clone(),
			remote_public: Public::new(),
			remote_nonce: H256::new(),
			auth_cipher: Bytes::new(),
			ack_cipher: Bytes::new(),
		})
	}

	pub fn start(&mut self, host: &HostInfo, originated: bool) -> Result<(), Error> {
		self.originated = originated;
		if originated {
			try!(self.write_auth(host));
		}
		else {
			self.state = HandshakeState::ReadingAuth;
			self.connection.expect(AUTH_PACKET_SIZE);
		};
		Ok(())
	}

	pub fn done(&self) -> bool {
		self.state == HandshakeState::StartSession
	}

	pub fn readable(&mut self, event_loop: &mut EventLoop<Host>, host: &HostInfo) -> Result<(), Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
		match self.state {
			HandshakeState::ReadingAuth => {
				match try!(self.connection.readable()) {
					Some(data)  => {
						try!(self.read_auth(host, &data));
						try!(self.write_ack());
					},
					None => {}
				};
			},
			HandshakeState::ReadingAck => {
				match try!(self.connection.readable()) {
					Some(data)  => {
						try!(self.read_ack(host, &data));
						self.state = HandshakeState::StartSession;
					},
					None => {}
				};
			},
			_ => { panic!("Unexpected state") }
		}
		if self.state != HandshakeState::StartSession {
			try!(self.connection.reregister(event_loop));
		}
		Ok(())
	}

	pub fn writable(&mut self, event_loop: &mut EventLoop<Host>, _host: &HostInfo) -> Result<(), Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
		match self.state {
			HandshakeState::WritingAuth => {
				match try!(self.connection.writable()) {
					WriteStatus::Complete => {
						self.connection.expect(ACK_PACKET_SIZE);
						self.state = HandshakeState::ReadingAck;
					},
					_ => {}
				};
			},
			HandshakeState::WritingAck => {
				match try!(self.connection.writable()) {
					WriteStatus::Complete => {
						self.connection.expect(32);
						self.state = HandshakeState::StartSession;
					},
					_ => {}
				};
			},
			_ => { panic!("Unexpected state") }
		}
		if self.state != HandshakeState::StartSession {
			try!(self.connection.reregister(event_loop));
		}
		Ok(())
	}

	pub fn register(&mut self, event_loop: &mut EventLoop<Host>) -> Result<(), Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
        self.idle_timeout = event_loop.timeout_ms(self.connection.token, 1800).ok();
		try!(self.connection.register(event_loop));
		Ok(())
	}

	fn read_auth(&mut self, host: &HostInfo, data: &[u8]) -> Result<(), Error> {
		trace!(target:"net", "Received handshake auth to {:?}", self.connection.socket.peer_addr());
		assert!(data.len() == AUTH_PACKET_SIZE);
		self.auth_cipher = data.to_vec();
		let auth = try!(ecies::decrypt(host.secret(), data));
		let (sig, rest) = auth.split_at(65);
		let (hepubk, rest) = rest.split_at(32);
		let (pubk, rest) = rest.split_at(64);
		let (nonce, _) = rest.split_at(32);
		self.remote_public.clone_from_slice(pubk);
		self.remote_nonce.clone_from_slice(nonce);
		let shared = try!(ecdh::agree(host.secret(), &self.remote_public));
		let signature = ec::Signature::from_slice(sig);
		let spub = try!(ec::recover(&signature, &(&shared ^ &self.remote_nonce)));
		if &spub.sha3()[..] != hepubk {
			trace!(target:"net", "Handshake hash mismath with {:?}", self.connection.socket.peer_addr());
			return Err(Error::Auth);
		};
		self.write_ack()
	}

	fn read_ack(&mut self, host: &HostInfo, data: &[u8]) -> Result<(), Error> {
		trace!(target:"net", "Received handshake auth to {:?}", self.connection.socket.peer_addr());
		assert!(data.len() == ACK_PACKET_SIZE);
		self.ack_cipher = data.to_vec();
		let ack = try!(ecies::decrypt(host.secret(), data));
		self.remote_public.clone_from_slice(&ack[0..64]);
		self.remote_nonce.clone_from_slice(&ack[64..(64+32)]);
		Ok(())
	}

	fn write_auth(&mut self, host: &HostInfo) -> Result<(), Error> {
		trace!(target:"net", "Sending handshake auth to {:?}", self.connection.socket.peer_addr());
		let mut data = [0u8; /*Signature::SIZE*/ 65 + /*H256::SIZE*/ 32 + /*Public::SIZE*/ 64 + /*H256::SIZE*/ 32 + 1]; //TODO: use associated constants
		let len = data.len();
		{
			data[len - 1] = 0x0;
			let (sig, rest) = data.split_at_mut(65);
			let (hepubk, rest) = rest.split_at_mut(32);
			let (pubk, rest) = rest.split_at_mut(64);
			let (nonce, _) = rest.split_at_mut(32);

			// E(remote-pubk, S(ecdhe-random, ecdh-shared-secret^nonce) || H(ecdhe-random-pubk) || pubk || nonce || 0x0)
			let shared = try!(crypto::ecdh::agree(host.secret(), &self.id));
			try!(crypto::ec::sign(self.ecdhe.secret(), &(&shared ^ &self.nonce))).copy_to(sig);
			self.ecdhe.public().sha3_into(hepubk);
			host.id().copy_to(pubk);
			self.nonce.copy_to(nonce);
		}
		let message = try!(crypto::ecies::encrypt(&self.id, &data));
		self.auth_cipher = message.clone();
		self.connection.send(message);
		self.state = HandshakeState::WritingAuth;
		Ok(())
	}

	fn write_ack(&mut self) -> Result<(), Error> {
		trace!(target:"net", "Sending handshake ack to {:?}", self.connection.socket.peer_addr());
		let mut data = [0u8; 1 + /*Public::SIZE*/ 64 + /*H256::SIZE*/ 32]; //TODO: use associated constants
		let len = data.len();
		{
			data[len - 1] = 0x0;
			let (epubk, rest) = data.split_at_mut(64);
			let (nonce, _) = rest.split_at_mut(32);
			self.ecdhe.public().copy_to(epubk);
			self.nonce.copy_to(nonce);
		}
		let message = try!(crypto::ecies::encrypt(&self.id, &data));
		self.ack_cipher = message.clone();
		self.connection.send(message);
		self.state = HandshakeState::WritingAck;
		Ok(())
	}
}
