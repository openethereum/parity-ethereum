#![allow(dead_code)] //TODO: remove this after everything is done
use std::collections::VecDeque;
use mio::{Token, EventSet, EventLoop, Timeout, PollOpt, TryRead, TryWrite};
use mio::tcp::*;
use hash::*;
use sha3::*;
use bytes::*;
use rlp::*;
use std::io::{self, Cursor, Read};
use network::host::{Host};
use network::Error;
use network::handshake::Handshake;
use crypto;
use rcrypto::blockmodes::*;
use rcrypto::aessafe::*;
use rcrypto::symmetriccipher::*;
use rcrypto::buffer::*;
use tiny_keccak::Keccak;

const ENCRYPTED_HEADER_LEN: usize = 32;

pub struct Connection {
	pub token: Token,
	pub socket: TcpStream,
	rec_buf: Bytes,
	rec_size: usize,
	send_queue: VecDeque<Cursor<Bytes>>,
	interest: EventSet,
}

#[derive(PartialEq, Eq)]
pub enum WriteStatus {
	Ongoing,
	Complete
}

impl Connection {
	pub fn new(token: Token, socket: TcpStream) -> Connection {
		Connection {
			token: token,
			socket: socket,
			send_queue: VecDeque::new(),
			rec_buf: Bytes::new(),
			rec_size: 0,
			interest: EventSet::hup(),
		}
	}

	pub fn expect(&mut self, size: usize) {
		if self.rec_size != self.rec_buf.len() {
			warn!(target:"net", "Unexpected connection read start");
		}
		unsafe { self.rec_buf.set_len(0) }
		self.rec_size = size;
	}

	//TODO: return a slice
	pub fn readable(&mut self) -> io::Result<Option<Bytes>> {
		if self.rec_size == 0 || self.rec_buf.len() >= self.rec_size {
			warn!(target:"net", "Unexpected connection read");
		}
		let max = self.rec_size - self.rec_buf.len();
		// resolve "multiple applicable items in scope [E0034]" error
		let sock_ref = <TcpStream as Read>::by_ref(&mut self.socket);
		match sock_ref.take(max as u64).try_read_buf(&mut self.rec_buf) {
			Ok(Some(_)) if self.rec_buf.len() == self.rec_size => {
				self.rec_size = 0;
				Ok(Some(::std::mem::replace(&mut self.rec_buf, Bytes::new())))
			},
			Ok(_) => Ok(None),
			Err(e) => Err(e),
		}
	}

	pub fn send(&mut self, data: Bytes) { //TODO: take ownership version
		if data.len() != 0 {
			self.send_queue.push_back(Cursor::new(data));
		}
		if !self.interest.is_writable() {
			self.interest.insert(EventSet::writable());
		}
	}

	pub fn writable(&mut self) -> io::Result<WriteStatus> {
		if self.send_queue.is_empty() {
			return Ok(WriteStatus::Complete)
		}
		{
			let buf = self.send_queue.front_mut().unwrap();
			let send_size = buf.get_ref().len();
			if (buf.position() as usize) >= send_size {
				warn!(target:"net", "Unexpected connection data");
				return Ok(WriteStatus::Complete)
			}
			match self.socket.try_write_buf(buf) {
				Ok(_) if (buf.position() as usize) < send_size => {
					self.interest.insert(EventSet::writable());
					Ok(WriteStatus::Ongoing)
				},
				Ok(_) if (buf.position() as usize) == send_size => {
					Ok(WriteStatus::Complete)
				},
				Ok(_) => { panic!("Wrote past buffer");},
				Err(e) => Err(e)
			}
		}.and_then(|r| {
			if r == WriteStatus::Complete {
				self.send_queue.pop_front();
			};
			if self.send_queue.is_empty() {
				self.interest.remove(EventSet::writable());
			}
			else {
				self.interest.insert(EventSet::writable());
			}
			Ok(r)
		})
	}

	pub fn register(&mut self, event_loop: &mut EventLoop<Host>) -> io::Result<()> {
		trace!(target: "net", "connection register; token={:?}", self.token);
		self.interest.insert(EventSet::readable());
		event_loop.register(&self.socket, self.token, self.interest, PollOpt::edge() | PollOpt::oneshot()).or_else(|e| {
			error!("Failed to reregister {:?}, {:?}", self.token, e);
			Err(e)
		})
	}

	pub fn reregister(&mut self, event_loop: &mut EventLoop<Host>) -> io::Result<()> {
		trace!(target: "net", "connection reregister; token={:?}", self.token);
		event_loop.reregister( &self.socket, self.token, self.interest, PollOpt::edge() | PollOpt::oneshot()).or_else(|e| {
			error!("Failed to reregister {:?}, {:?}", self.token, e);
			Err(e)
		})
	}
}

pub struct Packet {
	pub protocol: u16,
	pub data: Bytes,
}

enum EncryptedConnectionState {
	Header,
	Payload,
}

pub struct EncryptedConnection {
	connection: Connection,
	encoder: CtrMode<AesSafe256Encryptor>,
	decoder: CtrMode<AesSafe256Encryptor>,
	mac_encoder: EcbEncryptor<AesSafe256Encryptor, EncPadding<NoPadding>>,
	egress_mac: Keccak,
	ingress_mac: Keccak,
	read_state: EncryptedConnectionState,
	idle_timeout: Option<Timeout>,
	protocol_id: u16,
	payload_len: u32,
}

impl EncryptedConnection {
	pub fn new(handshake: Handshake) -> Result<EncryptedConnection, Error> {
		let shared = try!(crypto::ecdh::agree(handshake.ecdhe.secret(), &handshake.remote_public));
		let mut nonce_material = H512::new();
		if handshake.originated {
			handshake.remote_nonce.copy_to(&mut nonce_material[0..32]);
			handshake.nonce.copy_to(&mut nonce_material[32..64]);
		}
		else {
			handshake.nonce.copy_to(&mut nonce_material[0..32]);
			handshake.remote_nonce.copy_to(&mut nonce_material[32..64]);
		}
		let mut key_material = H512::new();
		shared.copy_to(&mut key_material[0..32]);
		nonce_material.sha3_into(&mut key_material[32..64]);
		key_material.sha3().copy_to(&mut key_material[32..64]);
		key_material.sha3().copy_to(&mut key_material[32..64]);

		let iv = vec![0u8; 16];
		let encoder = CtrMode::new(AesSafe256Encryptor::new(&key_material[32..64]), iv);
		let iv = vec![0u8; 16];
		let decoder = CtrMode::new(AesSafe256Encryptor::new(&key_material[32..64]), iv);

		key_material.sha3().copy_to(&mut key_material[32..64]);
		let mac_encoder = EcbEncryptor::new(AesSafe256Encryptor::new(&key_material[32..64]), NoPadding);
		println!("SESSION key: {}", H256::from_slice(&key_material[32..64]).hex());

		let mut egress_mac = Keccak::new_keccak256();
		let mut mac_material = &H256::from_slice(&key_material[32..64]) ^ &handshake.remote_nonce;
		egress_mac.update(&mac_material);
		egress_mac.update(if handshake.originated { &handshake.auth_cipher } else { &handshake.ack_cipher });

		let mut ingress_mac = Keccak::new_keccak256();
		mac_material = &H256::from_slice(&key_material[32..64]) ^ &handshake.nonce;
		ingress_mac.update(&mac_material);
		ingress_mac.update(if handshake.originated { &handshake.ack_cipher } else { &handshake.auth_cipher });

		Ok(EncryptedConnection {
			connection: handshake.connection,
			encoder: encoder,
			decoder: decoder,
			mac_encoder: mac_encoder,
			egress_mac: egress_mac,
			ingress_mac: ingress_mac,
			read_state: EncryptedConnectionState::Header,
			idle_timeout: None,
			protocol_id: 0,
			payload_len: 0
		})
	}

	pub fn send_packet(&mut self, payload: &[u8]) -> Result<(), Error> {
		println!("HEADER");
		let mut header = RlpStream::new();
		let len = payload.len() as usize;
		header.append_raw(&[(len >> 16) as u8, (len >> 8) as u8, len as u8], 1);
		header.append_raw(&[0xc2u8, 0x80u8, 0x80u8], 1);
		//TODO: ger rid of vectors here
		let mut header = header.out();
		let padding = (16 - (payload.len() % 16)) % 16;
		header.resize(16, 0u8);

		let mut packet = vec![0u8; (32 + payload.len() + padding + 16)];
		self.encoder.encrypt(&mut RefReadBuffer::new(&header), &mut RefWriteBuffer::new(&mut packet), false).expect("Invalid length or padding");
		EncryptedConnection::update_mac(&mut self.egress_mac, &mut self.mac_encoder,  &packet[0..16]);
		self.egress_mac.clone().finalize(&mut packet[16..32]);
		self.encoder.encrypt(&mut RefReadBuffer::new(&payload), &mut RefWriteBuffer::new(&mut packet[32..(32 + len)]), padding == 0).expect("Invalid length or padding");
		if padding != 0 {
			let pad = [08; 16];
			self.encoder.encrypt(&mut RefReadBuffer::new(&pad[0..padding]), &mut RefWriteBuffer::new(&mut packet[(32 + len)..(32 + len + padding)]), true).expect("Invalid length or padding");
		}
		self.egress_mac.update(&packet[32..(32 + len + padding)]);
		EncryptedConnection::update_mac(&mut self.egress_mac, &mut self.mac_encoder, &[0u8; 0]);
		self.egress_mac.clone().finalize(&mut packet[(32 + len + padding)..]);
		self.connection.send(packet);
		Ok(())
	}

	fn read_header(&mut self, header: &[u8]) -> Result<(), Error> {
		if header.len() != ENCRYPTED_HEADER_LEN {
			return Err(Error::Auth);
		}
		EncryptedConnection::update_mac(&mut self.ingress_mac, &mut self.mac_encoder, &header[0..16]);
		let mac = &header[16..];
		let mut expected = H256::new();
		self.ingress_mac.clone().finalize(&mut expected);
		if mac != &expected[0..16] {
			return Err(Error::Auth);
		}

		let mut hdec = H128::new();
		self.decoder.decrypt(&mut RefReadBuffer::new(&header[0..16]), &mut RefWriteBuffer::new(&mut hdec), false).expect("Invalid length or padding");

		let length = ((((hdec[0] as u32) << 8) + (hdec[1] as u32)) << 8) + (hdec[2] as u32);
		let header_rlp = UntrustedRlp::new(&hdec[3..6]);
		let protocol_id = try!(u16::decode_untrusted(&try!(header_rlp.at(0))));

		self.payload_len = length;
		self.protocol_id = protocol_id;
		self.read_state = EncryptedConnectionState::Payload;

		let padding = (16 - (length % 16)) % 16;
		let full_length = length + padding + 16;
		self.connection.expect(full_length as usize);
		Ok(())
	}

	fn read_payload(&mut self, payload: &[u8]) -> Result<Packet, Error> {
		let padding = (16 - (self.payload_len  % 16)) % 16;
		let full_length = (self.payload_len + padding + 16) as usize;
		if payload.len() != full_length {
			return Err(Error::Auth);
		}
		self.ingress_mac.update(&payload[0..payload.len() - 16]);
		EncryptedConnection::update_mac(&mut self.ingress_mac, &mut self.mac_encoder, &[0u8; 0]);
		let mac = &payload[(payload.len() - 16)..];
		let mut expected = H128::new();
		self.ingress_mac.clone().finalize(&mut expected);
		if mac != &expected[..] {
			return Err(Error::Auth);
		}

		let mut packet = vec![0u8; self.payload_len as usize];
		self.decoder.decrypt(&mut RefReadBuffer::new(&payload[0..(full_length - 16)]), &mut RefWriteBuffer::new(&mut packet), false).expect("Invalid length or padding");
		packet.resize(self.payload_len as usize, 0u8);
		Ok(Packet {
			protocol: self.protocol_id,
			data: packet
		})
	}

	fn update_mac(mac: &mut Keccak, mac_encoder: &mut EcbEncryptor<AesSafe256Encryptor, EncPadding<NoPadding>>, seed: &[u8]) {
		let mut prev = H128::new();
		mac.clone().finalize(&mut prev);
		let mut enc = H128::new();
		println!("before: {}", prev.hex());
		mac_encoder.encrypt(&mut RefReadBuffer::new(&prev), &mut RefWriteBuffer::new(&mut enc), true).unwrap();
		mac_encoder.reset();
		println!("after {}", enc.hex());

		if !seed.is_empty() {
			enc = enc ^ H128::from_slice(seed);
		}
		else {
			enc = enc ^ prev;
		}
		mac.update(&enc);
	}

	pub fn readable(&mut self, event_loop: &mut EventLoop<Host>) -> Result<Option<Packet>, Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
		try!(self.connection.reregister(event_loop));
		match self.read_state {
			EncryptedConnectionState::Header => {
				match try!(self.connection.readable()) {
					Some(data)  => {
						try!(self.read_header(&data));
					},
					None => {}
				};
				Ok(None)
			},
			EncryptedConnectionState::Payload => {
				match try!(self.connection.readable()) {
					Some(data)  => {
						self.read_state = EncryptedConnectionState::Header;
						self.connection.expect(ENCRYPTED_HEADER_LEN);
						Ok(Some(try!(self.read_payload(&data))))
					},
					None => Ok(None)
				}
			}
		}
	}

	pub fn writable(&mut self, event_loop: &mut EventLoop<Host>) -> Result<(), Error> {
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
		try!(self.connection.writable());
		try!(self.connection.reregister(event_loop));
		Ok(())
	}

	pub fn register(&mut self, event_loop: &mut EventLoop<Host>) -> Result<(), Error> {
		self.connection.expect(ENCRYPTED_HEADER_LEN);
		self.idle_timeout.map(|t| event_loop.clear_timeout(t));
		self.idle_timeout = event_loop.timeout_ms(self.connection.token, 1800).ok();
		try!(self.connection.reregister(event_loop));
		Ok(())
	}
}

#[test]
pub fn ctest() {
	use hash::*;
	use std::str::FromStr;
	let key = H256::from_str("2212767d793a7a3d66f869ae324dd11bd17044b82c9f463b8a541a4d089efec5").unwrap();
	let before = H128::from_str("12532abaec065082a3cf1da7d0136f15").unwrap();
	let before2 = H128::from_str("7e99f682356fdfbc6b67a9562787b18a").unwrap();
	let after = H128::from_str("89464c6b04e7c99e555c81d3f7266a05").unwrap();
	let after2 = H128::from_str("85c070030589ef9c7a2879b3a8489316").unwrap();

	let mut got = H128::new();

	let mut encoder = EcbEncryptor::new(AesSafe256Encryptor::new(&key), NoPadding);
	encoder.encrypt(&mut RefReadBuffer::new(&before), &mut RefWriteBuffer::new(&mut got), true).unwrap();
	encoder.reset();
	println!("got: {} ", got.hex());
	assert_eq!(got, after);
	got = H128::new();
	encoder.encrypt(&mut RefReadBuffer::new(&before2), &mut RefWriteBuffer::new(&mut got), true).unwrap();
	encoder.reset();
	assert_eq!(got, after2);
}


