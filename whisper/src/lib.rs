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

//! Whisper messaging system as a DevP2P subprotocol and RPC interface.

extern crate ethcore_bigint as bigint;
extern crate ethcore_network as network;
extern crate parking_lot;
extern crate rlp;
extern crate slab;
extern crate smallvec;
extern crate tiny_keccak;

#[macro_use]
extern crate log;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::sync::mpsc;
use std::time::{self, Duration, SystemTime};

use bigint::hash::{H256, H512};
use network::{NetworkContext, NetworkError, PeerId, TimerToken};
use parking_lot::{Mutex, RwLock};
use rlp::{DecoderError, RlpStream, UntrustedRlp};
use smallvec::SmallVec;
use tiny_keccak::keccak256;

const RALLY_TOKEN: TimerToken = 1;
const RALLY_TIMEOUT_MS: u64 = 750; // supposed to be at least once per second.

mod packet {
	pub const STATUS: u8 = 0;
	pub const MESSAGES: u8 = 1;
	pub const TOPIC_FILTER: u8 = 2;
}

#[derive(Debug)]
struct Topic([u8; 4]);

impl Topic {
	fn bloom(&self) -> H512 {
		unimplemented!()
	}
}

impl rlp::Encodable for Topic {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(&self.0);
	}
}

impl rlp::Decodable for Topic {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		use std::cmp;

		rlp.decoder().decode_value(|bytes| match bytes.len().cmp(&4) {
			cmp::Ordering::Less => Err(DecoderError::RlpIsTooShort),
			cmp::Ordering::Greater => Err(DecoderError::RlpIsTooBig),
			cmp::Ordering::Equal => {
				let mut t = [0u8; 4];
				t.copy_from_slice(bytes);
				Ok(Topic(t))
			}
		})
	}
}

// raw envelope struct.
#[derive(Debug)]
struct Envelope {
	expiry: u64,
	ttl: u64,
	topics: SmallVec<[Topic; 4]>,
	data: Vec<u8>,
	nonce: u64,
}

impl rlp::Encodable for Envelope {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(5)
			.append(&self.expiry)
			.append(&self.ttl)
			.append_list(&self.topics)
			.append(&self.data)
			.append(&self.nonce);
	}
}

impl rlp::Decodable for Envelope {
	fn decode(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		if rlp.item_count()? != 5 { return Err(DecoderError::RlpIncorrectListLen) }

		Ok(Envelope {
			expiry: rlp.val_at(0)?,
			ttl: rlp.val_at(1)?,
			topics: rlp.at(2)?.iter().map(|x| x.as_val()).collect::<Result<_, _>>()?,
			data: rlp.val_at(3)?,
			nonce: rlp.val_at(4)?,
		})
	}
}

#[derive(Debug)]
struct Message {
	envelope: Envelope,
	bloom: H512,
	hash: H256,
}

impl Message {
	fn expiry(&self) -> SystemTime {
		time::UNIX_EPOCH + Duration::from_secs(self.envelope.expiry)
	}
}

// errors in importing a whisper message.
#[derive(Debug)]
enum Error {
	Decoder(DecoderError),
	LivesTooLong,
	Network(NetworkError),
	UnknownPacket(u8),
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Self {
		Error::Decoder(err)
	}
}

impl From<NetworkError> for Error {
	fn from(err: NetworkError) -> Self {
		Error::Network(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Decoder(ref err) => write!(f, "Failed to decode message: {}", err),
			Error::LivesTooLong => write!(f, "Message claims to be issued before the unix epoch."),
			Error::Network(ref err) => write!(f, "Network error: {}", err),
			Error::UnknownPacket(ref id) => write!(f, "Unknown packet kind: {}", id),
		}
	}
}

// stores messages by two metrics: expiry and PoW rating
struct Messages {
	slab: ::slab::Slab<Message>,
	known: HashSet<H256>,
	by_expiry: BTreeMap<SystemTime, SmallVec<[usize; 8]>>,
}

impl Messages {
	fn new() -> Self {
		Messages {
			slab: ::slab::Slab::with_capacity(0),
			known: HashSet::new(),
			by_expiry: BTreeMap::new(),
		}
	}

	// reserve space for additional elements.
	fn reserve(&mut self, additional: usize) {
		self.slab.reserve_exact(additional);
		self.known.reserve(additional);
	}


	// insert a message into the store. for best performance,
	// call `reserve` before inserting a bunch.
	fn insert(&mut self, message: Message) {
		if !self.known.insert(message.hash) { return }

		let expiry = message.expiry();
		let id = self.slab.insert(message).unwrap_or_else(|message| {
			self.slab.reserve_exact(1);
			self.slab.insert(message).expect("just reserved space; qed")
		});

		self.by_expiry.entry(expiry)
			.or_insert_with(|| SmallVec::new())
			.push(id);
	}

	// prune expired messages.
	fn prune_expired(&mut self, now: &SystemTime) {
		let mut expired_times = Vec::new();

		{
			let expired = self.by_expiry.iter()
				.take_while(|&(time, _)| time <= now)
				.flat_map(|(time, v)| {
					expired_times.push(*time);
					v.iter().cloned()
				});

			for expired_id in expired {
				let message = self.slab.remove(expired_id).expect("only live ids are kept; qed");
				self.known.remove(&message.hash);
			}
		}

		for expired_time in expired_times {
			self.by_expiry.remove(&expired_time);
		}
	}
}

struct Peer;

/// The whisper network protocol handler.
pub struct Handler {
	incoming: Mutex<mpsc::Receiver<Message>>,
	async_sender: Mutex<mpsc::Sender<Message>>,
	messages: Mutex<Messages>,
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>,
}

impl Handler {
	fn rally(&self) {
		// accumulate incoming messages.
		let incoming_messages: Vec<_> = self.incoming.lock().try_iter().collect();
		let mut messages = self.messages.lock();

		messages.reserve(incoming_messages.len());

		for message in incoming_messages {
			messages.insert(message);
		}

		// 1. prune expired messages.
		// 2. loop over peers and send messages that they haven't seen yet
		unimplemented!()
	}

	// handle status packet from peer.
	fn on_status(&self, io: &NetworkContext, peer: &PeerId, status: UntrustedRlp)
		-> Result<(), Error>
	{
		Ok(())
	}

	fn on_messages(&self, io: &NetworkContext, peer: &PeerId, messages: UntrustedRlp)
		-> Result<(), Error>
	{
		let sender = self.async_sender.lock().clone();

		// decode messages packet and put to message store.
		// check for messages that match our "listeners"
		// broadcast using async sender.

		Ok(())
	}

	fn on_topic_filter(&self, io: &NetworkContext, peer: &PeerId, filter: UntrustedRlp)
		-> Result<(), Error>
	{
		Ok(())
	}

	fn on_connect(&self, io: &NetworkContext, peer: &PeerId) {
	}

	fn on_disconnect(&self, io: &NetworkContext, peer: &PeerId) {
	}
}

impl ::network::NetworkProtocolHandler for Handler {
	fn initialize(&self, io: &NetworkContext) {
		// set up broadcast timer (< 1s)
		io.register_timer(RALLY_TOKEN, RALLY_TIMEOUT_MS)
			.expect("Failed to initialize message rally timer");
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);
		let res = match packet_id {
			packet::STATUS => self.on_status(io, peer, rlp),
			packet::MESSAGES => self.on_messages(io, peer, rlp),
			packet::TOPIC_FILTER => self.on_topic_filter(io, peer, rlp),
			other => Err(Error::UnknownPacket(other)),
		};

		if let Err(e) = res {
			trace!(target: "whisper", "Disabling peer due to misbehavior: {}", e);
			io.disable_peer(*peer);
		}
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		// peer with higher ID should begin rallying.
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
	}

	fn timeout(&self, io: &NetworkContext, timer: TimerToken) {
		// rally with each peer and handle timeouts.
	}
}

#[cfg(test)]
mod tests {
}
