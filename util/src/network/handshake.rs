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

use std::sync::Arc;
use mio::*;
use mio::tcp::*;
use hash::*;
use sha3::Hashable;
use bytes::Bytes;
use crypto::*;
use crypto;
use network::connection::{Connection};
use network::host::{HostInfo};
use network::node::NodeId;
use error::*;
use network::error::NetworkError;
use network::stats::NetworkStats;
use io::{IoContext, StreamToken};

#[derive(PartialEq, Eq, Debug)]
enum HandshakeState {
	/// Just created
	New,
	/// Waiting for auth packet
	ReadingAuth,
	/// Waiting for ack packet
	ReadingAck,
	/// Ready to start a session
	StartSession,
}

/// RLPx protocol handhake. See https://github.com/ethereum/devp2p/blob/master/rlpx.md#encrypted-handshake
pub struct Handshake {
	/// Remote node public key
	pub id: NodeId,
	/// Underlying connection
	pub connection: Connection,
	/// Handshake state
	state: HandshakeState,
	/// Outgoing or incoming connection
	pub originated: bool,
	/// ECDH ephemeral
	pub ecdhe: KeyPair,
	/// Connection nonce
	pub nonce: H256,
	/// Handshake public key
	pub remote_public: Public,
	/// Remote connection nonce.
	pub remote_nonce: H256,
	/// A copy of received encryped auth packet 
	pub auth_cipher: Bytes,
	/// A copy of received encryped ack packet 
	pub ack_cipher: Bytes
}

const AUTH_PACKET_SIZE: usize = 307;
const ACK_PACKET_SIZE: usize = 210;
const HANDSHAKE_TIMEOUT: u64 = 30000;

impl Handshake {
	/// Create a new handshake object
	pub fn new(token: StreamToken, id: Option<&NodeId>, socket: TcpStream, nonce: &H256, stats: Arc<NetworkStats>) -> Result<Handshake, UtilError> {
		Ok(Handshake {
			id: if let Some(id) = id { id.clone()} else { NodeId::new() },
			connection: Connection::new(token, socket, stats),
			originated: false,
			state: HandshakeState::New,
			ecdhe: try!(KeyPair::create()),
			nonce: nonce.clone(),
			remote_public: Public::new(),
			remote_nonce: H256::new(),
			auth_cipher: Bytes::new(),
			ack_cipher: Bytes::new(),
		})
	}

	/// Get id of the remote node if known
	pub fn id(&self) -> &NodeId {
		&self.id
	}

	/// Get stream token id
	pub fn token(&self) -> StreamToken {
		self.connection.token()
	}

	/// Start a handhsake
	pub fn start<Message>(&mut self, io: &IoContext<Message>, host: &HostInfo, originated: bool) -> Result<(), UtilError> where Message: Send + Clone{
		self.originated = originated;
		io.register_timer(self.connection.token, HANDSHAKE_TIMEOUT).ok();
		if originated {
			try!(self.write_auth(host));
		}
		else {
			self.state = HandshakeState::ReadingAuth;
			self.connection.expect(AUTH_PACKET_SIZE);
		};
		Ok(())
	}

	/// Check if handshake is complete
	pub fn done(&self) -> bool {
		self.state == HandshakeState::StartSession
	}

	/// Readable IO handler. Drives the state change.
	pub fn readable<Message>(&mut self, io: &IoContext<Message>, host: &HostInfo) -> Result<(), UtilError> where Message: Send + Clone {
		io.clear_timer(self.connection.token).unwrap();
		match self.state {
			HandshakeState::ReadingAuth => {
				if let Some(data) = try!(self.connection.readable()) {
					try!(self.read_auth(host, &data));
					try!(self.write_ack());
				};
			},
			HandshakeState::ReadingAck => {
				if let Some(data) = try!(self.connection.readable()) {
					try!(self.read_ack(host, &data));
					self.state = HandshakeState::StartSession;
				};
			},
			HandshakeState::StartSession => {},
			_ => { panic!("Unexpected state"); }
		}
		if self.state != HandshakeState::StartSession {
			try!(io.update_registration(self.connection.token));
		}
		Ok(())
	}

	/// Writabe IO handler.
	pub fn writable<Message>(&mut self, io: &IoContext<Message>, _host: &HostInfo) -> Result<(), UtilError> where Message: Send + Clone {
		io.clear_timer(self.connection.token).unwrap();
		try!(self.connection.writable());
		if self.state != HandshakeState::StartSession {
			io.update_registration(self.connection.token).unwrap();
		}
		Ok(())
	}

	/// Register the socket with the event loop
	pub fn register_socket<Host:Handler<Timeout=Token>>(&self, reg: Token, event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		try!(self.connection.register_socket(reg, event_loop));
		Ok(())
	}

	pub fn update_socket<Host:Handler<Timeout=Token>>(&self, reg: Token, event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		try!(self.connection.update_socket(reg, event_loop));
		Ok(())
	}

	/// Delete registration
	pub fn deregister_socket<Host:Handler>(&self, event_loop: &mut EventLoop<Host>) -> Result<(), UtilError> {
		try!(self.connection.deregister_socket(event_loop));
		Ok(())
	}

	/// Parse, validate and confirm auth message
	fn read_auth(&mut self, host: &HostInfo, data: &[u8]) -> Result<(), UtilError> {
		trace!(target:"net", "Received handshake auth to {:?}", self.connection.socket.peer_addr());
		if data.len() != AUTH_PACKET_SIZE {
			debug!(target:"net", "Wrong auth packet size");
			return Err(From::from(NetworkError::BadProtocol));
		}
		self.auth_cipher = data.to_vec();
		let auth = try!(ecies::decrypt(host.secret(), data));
		let (sig, rest) = auth.split_at(65);
		let (hepubk, rest) = rest.split_at(32);
		let (pubk, rest) = rest.split_at(64);
		let (nonce, _) = rest.split_at(32);
		self.id.clone_from_slice(pubk);
		self.remote_nonce.clone_from_slice(nonce);
		let shared = try!(ecdh::agree(host.secret(), &self.id));
		let signature = Signature::from_slice(sig);
		let spub = try!(ec::recover(&signature, &(&shared ^ &self.remote_nonce)));
		self.remote_public = spub.clone();
		if &spub.sha3()[..] != hepubk {
			trace!(target:"net", "Handshake hash mismath with {:?}", self.connection.socket.peer_addr());
			return Err(From::from(NetworkError::Auth));
		};
		Ok(())
	}

	/// Parse and validate ack message
	fn read_ack(&mut self, host: &HostInfo, data: &[u8]) -> Result<(), UtilError> {
		trace!(target:"net", "Received handshake auth to {:?}", self.connection.socket.peer_addr());
		if data.len() != ACK_PACKET_SIZE {
			debug!(target:"net", "Wrong ack packet size");
			return Err(From::from(NetworkError::BadProtocol));
		}
		self.ack_cipher = data.to_vec();
		let ack = try!(ecies::decrypt(host.secret(), data));
		self.remote_public.clone_from_slice(&ack[0..64]);
		self.remote_nonce.clone_from_slice(&ack[64..(64+32)]);
		Ok(())
	}

	/// Sends auth message
	fn write_auth(&mut self, host: &HostInfo) -> Result<(), UtilError> {
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
		self.connection.expect(ACK_PACKET_SIZE);
		self.state = HandshakeState::ReadingAck;
		Ok(())
	}

	/// Sends ack message
	fn write_ack(&mut self) -> Result<(), UtilError> {
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
		self.state = HandshakeState::StartSession;
		Ok(())
	}
}
