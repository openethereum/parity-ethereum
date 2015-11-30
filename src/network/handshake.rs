use mio::*;
use mio::tcp::*;
use hash::*;
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
	WritingHello,
	ReadingHello,
	StartSession,
}

pub struct Handshake {
	pub id: NodeId,
	pub connection: Connection,
	state: HandshakeState,
	idle_timeout: Option<Timeout>,
	ecdhe: KeyPair,
	nonce: H256,
	remote_public: Public,
	remote_nonce: H256
}

impl Handshake {
	pub fn new(token: Token, id: &NodeId, socket: TcpStream, nonce: &H256) -> Result<Handshake, Error> {
		Ok(Handshake {
			id: id.clone(),
			connection: Connection::new(token, socket),
			state: HandshakeState::New,
			idle_timeout: None,
			ecdhe: try!(KeyPair::create()),
			nonce: nonce.clone(),
			remote_public: Public::new(),
			remote_nonce: H256::new()
		})
	}

	pub fn start(&mut self, host: &HostInfo, originated: bool) {
		if originated {
			self.write_auth(host);
		} 
		else {
			self.read_auth();
		};
	}

	pub fn done(&self) -> bool {
		self.state == HandshakeState::StartSession
	}

	pub fn readable(&mut self, event_loop: &mut EventLoop<Host>, host: &HostInfo) -> Result<(), Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
		Ok(())
	}

	pub fn writable(&mut self, event_loop: &mut EventLoop<Host>, host: &HostInfo) -> Result<(), Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
		match self.state {
			HandshakeState::WritingAuth => {
				match (try!(self.connection.writable())) {
					WriteStatus::Complete => { try!(self.read_ack()); },
					_ => {}
				};
				try!(self.connection.reregister(event_loop));
			},
			HandshakeState::WritingAck => {
				match (try!(self.connection.writable())) {
					WriteStatus::Complete => { try!(self.read_hello()); },
					_ => {}
				};
				try!(self.connection.reregister(event_loop));
			},
			HandshakeState::WritingHello => {
				match (try!(self.connection.writable())) {
					WriteStatus::Complete => { self.state = HandshakeState::StartSession; },
					_ => { try!(self.connection.reregister(event_loop)); }
				};
			},
			_ => { panic!("Unexpected state") }
		}
		Ok(())
	}

	pub fn register(&mut self, event_loop: &mut EventLoop<Host>) -> Result<(), Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
        self.idle_timeout = event_loop.timeout_ms(self.connection.token, 1800).ok();
		self.connection.register(event_loop);
		Ok(())
	}

	fn read_auth(&mut self) -> Result<(), Error> {
		Ok(())
	}

	fn read_ack(&mut self) -> Result<(), Error> {
		Ok(())
	}

	fn read_hello(&mut self) -> Result<(), Error> {
		Ok(())
	}

	fn write_auth(&mut self, host: &HostInfo) -> Result<(), Error> {
		trace!(target:"net", "Sending auth to {:?}", self.connection.socket.peer_addr());
		let mut data = [0u8; /*Signature::SIZE*/ 65 + /*H256::SIZE*/ 32 + /*Public::SIZE*/ 64 + /*H256::SIZE*/ 32 + 1]; //TODO: use associated constants
		let len = data.len();
		{
			data[len - 1] = 0x0;
			let (sig, rest) = data.split_at_mut(65);
			let (hepubk, rest) = rest.split_at_mut(32);
			let (mut pubk, rest) = rest.split_at_mut(64);
			let (nonce, rest) = rest.split_at_mut(32);
			
			// E(remote-pubk, S(ecdhe-random, ecdh-shared-secret^nonce) || H(ecdhe-random-pubk) || pubk || nonce || 0x0)
			let shared = try!(crypto::ecdh::agree(host.secret(), &self.id));
			let signature = try!(crypto::ec::sign(self.ecdhe.secret(), &(&shared ^ &self.nonce))).copy_to(sig);
			self.ecdhe.public().sha3_into(hepubk);
			host.id().copy_to(&mut pubk);
			self.nonce.copy_to(nonce);
		}
		let message = try!(crypto::ecies::encrypt(&self.id, &data));
		self.connection.send(&message[..]);
		self.state = HandshakeState::WritingAuth;
		Ok(())
	}

	fn write_ack(&mut self) -> Result<(), Error> {
		Ok(())
	}

	fn write_hello(&mut self) -> Result<(), Error> {
		Ok(())
	}


}
