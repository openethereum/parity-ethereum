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

//! Whisper messaging system as a DevP2P subprotocol.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use bigint::hash::{H256, H512};
use network::{HostInfo, NetworkContext, NetworkError, NodeId, PeerId, TimerToken};
use parking_lot::{Mutex, RwLock};
use rlp::{DecoderError, RlpStream, UntrustedRlp};
use smallvec::SmallVec;

use message::{Message, Error as MessageError};

const RALLY_TOKEN: TimerToken = 1;
const RALLY_TIMEOUT_MS: u64 = 750; // supposed to be at least once per second.

const PROTOCOL_VERSION: usize = 2;

// maximum tolerated delay between messages packets.
const MAX_TOLERATED_DELAY_MS: u64 = 2000;

mod packet {
	pub const STATUS: u8 = 0;
	pub const MESSAGES: u8 = 1;
	pub const TOPIC_FILTER: u8 = 2;
}

// errors in importing a whisper message.
#[derive(Debug)]
enum Error {
	Decoder(DecoderError),
	Network(NetworkError),
	Message(MessageError),
	UnknownPacket(u8),
	UnknownPeer(PeerId),
	ProtocolVersionMismatch(usize),
	SameNodeKey,
	UnexpectedMessage,
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

impl From<MessageError> for Error {
	fn from(err: MessageError) -> Self {
		Error::Message(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Decoder(ref err) => write!(f, "Failed to decode packet: {}", err),
			Error::Network(ref err) => write!(f, "Network error: {}", err),
			Error::Message(ref err) => write!(f, "Error decoding message: {}", err),
			Error::UnknownPacket(ref id) => write!(f, "Unknown packet kind: {}", id),
			Error::UnknownPeer(ref id) => write!(f, "Message received from unknown peer: {}", id),
			Error::ProtocolVersionMismatch(ref proto) =>
				write!(f, "Unknown protocol version: {}", proto),
			Error::UnexpectedMessage => write!(f, "Unexpected message."),
			Error::SameNodeKey => write!(f, "Peer and us have same node key."),
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
		if !self.known.insert(message.hash().clone()) { return }

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
	fn prune_expired(&mut self, now: &SystemTime) -> Vec<Message> {
		let mut expired_times = Vec::new();
		let mut expired_messages = Vec::new();

		{
			let expired = self.by_expiry.iter()
				.take_while(|&(time, _)| time <= now)
				.flat_map(|(time, v)| {
					expired_times.push(*time);
					v.iter().cloned()
				});

			for expired_id in expired {
				let message = self.slab.remove(expired_id).expect("only live ids are kept; qed");
				self.known.remove(message.hash());

				expired_messages.push(message);
			}
		}

		for expired_time in expired_times {
			self.by_expiry.remove(&expired_time);
		}

		expired_messages
	}

	fn iter(&self) -> ::slab::Iter<Message, usize> {
		self.slab.iter()
	}
}

enum State {
	Unconfirmed(SystemTime), // awaiting status packet.
	TheirTurn(SystemTime), // it has been their turn to send since stored time.
	OurTurn,
}

struct Peer {
	node_key: NodeId,
	state: State,
	known_messages: HashSet<H256>,
	topic_filter: Option<H512>,
}

impl Peer {
	// note that a message has been evicted from the queue.
	fn note_evicted(&mut self, messages: &[Message]) {
		for message in messages {
			self.known_messages.remove(message.hash());
		}
	}

	// whether this peer will accept the message.
	fn will_accept(&self, message: &Message) -> bool {
		let known = self.known_messages.contains(message.hash());

		let matches_bloom = self.topic_filter.as_ref()
			.map_or(true, |topic| topic & message.bloom() == message.bloom().clone());

		!known && matches_bloom
	}

	fn note_known(&mut self, message: &Message) {
		self.known_messages.insert(message.hash().clone());
	}

	fn set_topic_filter(&mut self, topic: H512) {
		self.topic_filter = Some(topic);
	}

	fn can_send_messages(&self) -> bool {
		match self.state {
			State::Unconfirmed(_) | State::OurTurn => false,
			State::TheirTurn(_) => true,
		}
	}
}

/// The whisper network protocol handler.
pub struct Handler {
	incoming: Mutex<mpsc::Receiver<Message>>,
	async_sender: Mutex<mpsc::Sender<Message>>,
	messages: Mutex<Messages>,
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>,
	node_key: RwLock<NodeId>,
}

// public API.
impl Handler {
	/// Create a new network handler.
	pub fn new() -> Self {
		let (tx, rx) = mpsc::channel();

		Handler {
			incoming: Mutex::new(rx),
			async_sender: Mutex::new(tx),
			messages: Mutex::new(Messages::new()),
			peers: RwLock::new(HashMap::new()),
			node_key: RwLock::new(Default::default()),
		}
	}

	/// Acquire a sender to asynchronously feed messages.
	pub fn message_sender(&self) -> mpsc::Sender<Message> {
		self.async_sender.lock().clone()
	}
}

impl Handler {
	fn rally(&self, io: &NetworkContext) {
		// cannot be greater than 16MB (protocol limitation)
		const MAX_MESSAGES_PACKET_SIZE: usize = 8 * 1024 * 1024;

		// accumulate incoming messages.
		let incoming_messages: Vec<_> = self.incoming.lock().try_iter().collect();
		let mut messages = self.messages.lock();

		messages.reserve(incoming_messages.len());

		for message in incoming_messages {
			messages.insert(message);
		}

		let now = SystemTime::now();
		let pruned_messages = messages.prune_expired(&now);

		let peers = self.peers.read();

		// send each peer a packet with new messages it may find relevant.
		for (peer_id, peer) in peers.iter() {
			let mut peer_data = peer.lock();
			peer_data.note_evicted(&pruned_messages);

			let punish_timeout = |last_activity: &SystemTime| {
				if *last_activity + Duration::from_millis(MAX_TOLERATED_DELAY_MS) <= now {
					debug!(target: "whisper", "Disconnecting peer {} due to excessive timeout.", peer_id);
					io.disconnect_peer(*peer_id);
				}
			};

			// check timeouts and skip peers who we can't send a rally to.
			match peer_data.state {
				State::Unconfirmed(ref time) | State::TheirTurn(ref time) => {
					punish_timeout(time);
					continue;
				}
				State::OurTurn => {}
			}

			// construct packet, skipping messages the peer won't accept.
			let mut stream = RlpStream::new();
			stream.begin_unbounded_list();

			for message in messages.iter() {
				if !peer_data.will_accept(message) { continue }

				if stream.estimate_size(message.encoded_size()) > MAX_MESSAGES_PACKET_SIZE {
					break;
				}

				peer_data.note_known(message);
				stream.append(message.envelope());
			}
			stream.complete_unbounded_list();

			peer_data.state = State::TheirTurn(SystemTime::now());
			if let Err(e) = io.send(*peer_id, packet::MESSAGES, stream.out()) {
				debug!(target: "whisper", "Failed to send messages packet to peer {}: {}", peer_id, e);
				io.disconnect_peer(*peer_id);
			}
		}
	}

	// handle status packet from peer.
	fn on_status(&self, peer: &PeerId, status: UntrustedRlp)
		-> Result<(), Error>
	{
		let proto: usize = status.as_val()?;
		if proto != PROTOCOL_VERSION { return Err(Error::ProtocolVersionMismatch(proto)) }

		let peers = self.peers.read();
		match peers.get(peer) {
			Some(peer) => {
				let mut peer = peer.lock();
				let our_node_key = self.node_key.read().clone();

				// handle this basically impossible edge case gracefully.
				if peer.node_key == our_node_key {
					return Err(Error::SameNodeKey);
				}

				// peer with lower node key begins the rally.
				if peer.node_key > our_node_key {
					peer.state = State::OurTurn;
				} else {
					peer.state = State::TheirTurn(SystemTime::now());
				}

				Ok(())
			}
			None => {
				debug!(target: "whisper", "Received message from unknown peer.");
				Err(Error::UnknownPeer(*peer))
			}
		}
	}

	fn on_messages(&self, peer: &PeerId, messages: UntrustedRlp)
		-> Result<(), Error>
	{
		// TODO: pinning thread-local data to each I/O worker would
		// optimize this significantly.
		let sender = self.message_sender();

		let peers = self.peers.read();
		let peer = match peers.get(peer) {
			Some(peer) => peer,
			None => {
				debug!(target: "whisper", "Received message from unknown peer.");
				return Err(Error::UnknownPeer(*peer));
			}
		};

		let mut peer = peer.lock();

		if !peer.can_send_messages() {
			return Err(Error::UnexpectedMessage);
		}

		peer.state = State::OurTurn;

		let now = SystemTime::now();
		for message_rlp in messages.iter() {
			let message = Message::decode(message_rlp, now)?;
			peer.note_known(&message);

			// TODO: check whether the message matches our local filter criteria
			// and pass payload to handler.

			sender.send(message).expect("receiver always kept alive; qed");
		}


		Ok(())
	}

	fn on_topic_filter(&self, peer: &PeerId, filter: UntrustedRlp)
		-> Result<(), Error>
	{
		let peers = self.peers.read();
		match peers.get(peer) {
			Some(peer) => {
				let mut peer = peer.lock();

				if let State::Unconfirmed(_) = peer.state {
					return Err(Error::UnexpectedMessage);
				}

				peer.set_topic_filter(filter.as_val()?)
			}
			None => {
				debug!(target: "whisper", "Received message from unknown peer.");
				return Err(Error::UnknownPeer(*peer));
			}
		}

		Ok(())
	}

	fn on_connect(&self, io: &NetworkContext, peer: &PeerId) {
		trace!(target: "whisper", "Connecting peer {}", peer);

		let node_key = match io.session_info(*peer).and_then(|info| info.id) {
			Some(node_key) => node_key,
			None => {
				debug!(target: "whisper", "Disconnecting peer {}, who has no node key.", peer);
				io.disable_peer(*peer);
				return;
			}
		};

		self.peers.write().insert(*peer, Mutex::new(Peer {
			node_key: node_key,
			state: State::Unconfirmed(SystemTime::now()),
			known_messages: HashSet::new(),
			topic_filter: None,
		}));

		if let Err(e) = io.send(*peer, packet::STATUS, ::rlp::encode(&PROTOCOL_VERSION).to_vec()) {
			debug!(target: "whisper", "Error sending status: {}", e);
			io.disconnect_peer(*peer);
		}
	}

	fn on_disconnect(&self, peer: &PeerId) {
		trace!(target: "whisper", "Disconnecting peer {}", peer);
		let _ = self.peers.write().remove(peer);
	}
}

impl ::network::NetworkProtocolHandler for Handler {
	fn initialize(&self, io: &NetworkContext, host_info: &HostInfo) {
		// set up broadcast timer (< 1s)
		io.register_timer(RALLY_TOKEN, RALLY_TIMEOUT_MS)
			.expect("Failed to initialize message rally timer");

		*self.node_key.write() = host_info.id().clone();
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);
		let res = match packet_id {
			packet::STATUS => self.on_status(peer, rlp),
			packet::MESSAGES => self.on_messages(peer, rlp),
			packet::TOPIC_FILTER => self.on_topic_filter(peer, rlp),
			other => Err(Error::UnknownPacket(other)),
		};

		if let Err(e) = res {
			trace!(target: "whisper", "Disabling peer due to misbehavior: {}", e);
			io.disable_peer(*peer);
		}
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		// peer with higher ID should begin rallying.
		self.on_connect(io, peer)
	}

	fn disconnected(&self, _io: &NetworkContext, peer: &PeerId) {
		self.on_disconnect(peer)
	}

	fn timeout(&self, io: &NetworkContext, timer: TimerToken) {
		// rally with each peer and handle timeouts.
		match timer {
			RALLY_TOKEN => self.rally(io),
			other => debug!(target: "whisper", "Timout triggered on unknown token {}", other),
		}
	}
}

#[cfg(test)]
mod tests {
}
