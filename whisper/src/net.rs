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

use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::fmt;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use bigint::hash::{H256, H512};
use network::{HostInfo, NetworkContext, NetworkError, NodeId, PeerId, TimerToken};
use ordered_float::OrderedFloat;
use parking_lot::{Mutex, RwLock};
use rlp::{DecoderError, RlpStream, UntrustedRlp};

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

/// Handles messages within a single packet.
pub trait MessageHandler {
	/// Evaluate the message and handle it.
	///
	/// The same message will not be passed twice.
	/// Heavy handling should be done asynchronously.
	/// If there is a significant overhead in this thread, then an attacker
	/// can determine which kinds of messages we are listening for.
	fn handle_message(&mut self, message: &Message);
}

/// Creates message handlers.
pub trait CreateHandler: Send + Sync {
	type Handler: MessageHandler;

	/// Create a message handler which will process
	/// messages for a single packet.
	fn create_handler(&self) -> Self::Handler;
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

// sorts by work proved, ascending.
#[derive(PartialEq, Eq)]
struct SortedEntry {
	slab_id: usize,
	work_proved: OrderedFloat<f64>,
	expiry: SystemTime,
}

impl Ord for SortedEntry {
	fn cmp(&self, other: &SortedEntry) -> Ordering {
		self.work_proved.cmp(&other.work_proved).reverse()
	}
}

impl PartialOrd for SortedEntry {
	fn partial_cmp(&self, other: &SortedEntry) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

// stores messages by two metrics: expiry and PoW rating
struct Messages {
	slab: ::slab::Slab<Message>,
	sorted: Vec<SortedEntry>,
	known: HashSet<H256>,
	cumulative_size: usize,
	ideal_size: usize,
}

impl Messages {
	fn new(ideal_size: usize) -> Self {
		Messages {
			slab: ::slab::Slab::with_capacity(0),
			sorted: Vec::new(),
			known: HashSet::new(),
			cumulative_size: 0,
			ideal_size: ideal_size,
		}
	}

	// reserve space for additional elements.
	fn reserve(&mut self, additional: usize) {
		self.slab.reserve_exact(additional);
		self.sorted.reserve(additional);
		self.known.reserve(additional);
	}

	// whether a message is known.
	fn contains(&self, message: &Message) -> bool {
		self.known.contains(message.hash())
	}

	// insert a message into the store. for best performance,
	// call `reserve` before inserting a bunch.
	//
	// does not prune low PoW messages. Call `prune`
	// to do that.
	//
	// TODO: consolidate insertion and pruning under-worked
	// messages with an iterator returned from this method.
	fn insert(&mut self, message: Message) {
		if !self.known.insert(message.hash().clone()) { return }

		let expiry = message.expiry();
		let work_proved = OrderedFloat(message.work_proved());

		self.cumulative_size += message.encoded_size();

		if !self.slab.has_available() { self.slab.reserve_exact(1) }
		let id = self.slab.insert(message).expect("just ensured enough space in slab; qed");

		let sorted_entry = SortedEntry {
			slab_id: id,
			work_proved: work_proved,
			expiry: expiry,
		};

		match self.sorted.binary_search(&sorted_entry) {
			Ok(idx) | Err(idx) => self.sorted.insert(idx, sorted_entry),
		}
	}

	// prune expired messages, and then prune low proof-of-work messages
	// until below ideal size.
	fn prune(&mut self, now: SystemTime) -> Vec<Message> {
		let mut messages = Vec::new();

		{
			let slab = &mut self.slab;
			let known = &mut self.known;
			let cumulative_size = &mut self.cumulative_size;
			let ideal_size = &self.ideal_size;

			// first pass, we look just at expired entries.
			let all_expired = self.sorted.iter()
				.filter(|entry| entry.expiry <= now)
				.map(|x| (true, x));

			// second pass, we look at entries which aren't expired but in order
			let low_proof = self.sorted.iter()
				.filter(|entry| entry.expiry > now)
				.map(|x| (false, x));

			for (is_expired, entry) in all_expired.chain(low_proof) {
				// break once we've removed all expired entries
				// or have taken enough low-work entries.
				if !is_expired && *cumulative_size <= *ideal_size {
					break
				}

				let message = slab.remove(entry.slab_id)
					.expect("references to ID kept upon creation; only destroyed upon removal; qed");

				known.remove(message.hash());

				*cumulative_size -= message.encoded_size();
				messages.push(message);
			}
		}

		// clear all the sorted entries we removed from slab.
		let slab = &self.slab;
		self.sorted.retain(|entry| slab.contains(entry.slab_id));

		messages
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

	// note a message as known. returns true if it was already
	// known, false otherwise.
	fn note_known(&mut self, message: &Message) -> bool {
		self.known_messages.insert(message.hash().clone())
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
pub struct Network<T> {
	incoming: Mutex<mpsc::Receiver<Message>>,
	async_sender: Mutex<mpsc::Sender<Message>>,
	messages: RwLock<Messages>,
	create_handler: T,
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>,
	node_key: RwLock<NodeId>,
}

// public API.
impl<T> Network<T> {
	/// Create a new network handler.
	pub fn new(messages_size_bytes: usize, create_handler: T) -> Self {
		let (tx, rx) = mpsc::channel();

		Network {
			incoming: Mutex::new(rx),
			async_sender: Mutex::new(tx),
			messages: RwLock::new(Messages::new(messages_size_bytes)),
			create_handler: create_handler,
			peers: RwLock::new(HashMap::new()),
			node_key: RwLock::new(Default::default()),
		}
	}

	/// Acquire a sender to asynchronously feed messages.
	pub fn message_sender(&self) -> mpsc::Sender<Message> {
		self.async_sender.lock().clone()
	}
}

impl<T: CreateHandler> Network<T> {
	fn rally(&self, io: &NetworkContext) {
		// cannot be greater than 16MB (protocol limitation)
		const MAX_MESSAGES_PACKET_SIZE: usize = 8 * 1024 * 1024;

		// accumulate incoming messages.
		let incoming_messages: Vec<_> = self.incoming.lock().try_iter().collect();
		let mut messages = self.messages.write();

		messages.reserve(incoming_messages.len());

		for message in incoming_messages {
			messages.insert(message);
		}

		let now = SystemTime::now();
		let pruned_messages = messages.prune(now);

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

	fn on_messages(&self, peer: &PeerId, message_packet: UntrustedRlp)
		-> Result<(), Error>
	{
		// TODO: pinning thread-local data to each I/O worker would
		// optimize this significantly.
		let sender = self.message_sender();
		let mut packet_handler = self.create_handler.create_handler();
		let messages = self.messages.read();

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

		for message_rlp in message_packet.iter() {
			let message = Message::decode(message_rlp, now)?;
			if !peer.note_known(&message) || messages.contains(&message) {
				continue
			}

			packet_handler.handle_message(&message);
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

impl<T: CreateHandler> ::network::NetworkProtocolHandler for Network<T> {
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
