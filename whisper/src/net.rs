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
use std::time::{Duration, SystemTime};
use std::sync::Arc;

use bigint::hash::{H256, H512};
use network::{HostInfo, NetworkContext, NetworkError, NodeId, PeerId, TimerToken};
use ordered_float::OrderedFloat;
use parking_lot::{Mutex, RwLock};
use rlp::{DecoderError, RlpStream, UntrustedRlp};

use message::{Message, Error as MessageError};

const RALLY_TOKEN: TimerToken = 1;
const RALLY_TIMEOUT_MS: u64 = 750; // supposed to be at least once per second.

const PROTOCOL_VERSION: usize = 2;

/// Supported protocol versions.
pub const SUPPORTED_VERSIONS: &'static [u8] = &[PROTOCOL_VERSION as u8];

// maximum tolerated delay between messages packets.
const MAX_TOLERATED_DELAY_MS: u64 = 2000;

/// Number of packets.
pub const PACKET_COUNT: u8 = 3;

mod packet {
	pub const STATUS: u8 = 0;
	pub const MESSAGES: u8 = 1;
	pub const TOPIC_FILTER: u8 = 2;
}

/// Handles messages within a single packet.
pub trait MessageHandler: Send + Sync {
	/// Evaluate the message and handle it.
	///
	/// The same message will not be passed twice.
	/// Heavy handling should be done asynchronously.
	/// If there is a significant overhead in this thread, then an attacker
	/// can determine which kinds of messages we are listening for.
	fn handle_messages(&self, message: &[Message]);
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

// sorts by work proved, descending.
#[derive(PartialEq, Eq)]
struct SortedEntry {
	slab_id: usize,
	work_proved: OrderedFloat<f64>,
	expiry: SystemTime,
}

impl Ord for SortedEntry {
	fn cmp(&self, other: &SortedEntry) -> Ordering {
		self.work_proved.cmp(&other.work_proved)
	}
}

impl PartialOrd for SortedEntry {
	fn partial_cmp(&self, other: &SortedEntry) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

// stores messages by two metrics: expiry and PoW rating
// when full, will accept messages above the minimum stored.
struct Messages {
	slab: ::slab::Slab<Message>,
	sorted: Vec<SortedEntry>,
	known: HashSet<H256>,
	removed_hashes: Vec<H256>,
	cumulative_size: usize,
	ideal_size: usize,
}

impl Messages {
	fn new(ideal_size: usize) -> Self {
		Messages {
			slab: ::slab::Slab::with_capacity(0),
			sorted: Vec::new(),
			known: HashSet::new(),
			removed_hashes: Vec::new(),
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

	// whether a message is not known and within the bounds of PoW.
	fn may_accept(&self, message: &Message) -> bool {
		!self.known.contains(message.hash()) && {
			self.sorted.last().map_or(true, |entry| {
				let work_proved = OrderedFloat(message.work_proved());
				OrderedFloat(self.slab[entry.slab_id].work_proved()) < work_proved
			})
		}
	}

	// insert a message into the store. for best performance,
	// call `reserve` before inserting a bunch.
	//
	fn insert(&mut self, message: Message) -> bool {
		if !self.known.insert(message.hash().clone()) { return false }

		let work_proved = OrderedFloat(message.work_proved());

		// pop off entries by low PoW until we have enough space for the higher
		// PoW message being inserted.
		let size_upon_insertion = self.cumulative_size + message.encoded_size();
		if size_upon_insertion >= self.ideal_size {
			let diff = size_upon_insertion - self.ideal_size;
			let mut found_diff = 0;
			for entry in self.sorted.iter().rev() {
				if found_diff >= diff { break }

				// if we encounter a message with at least the PoW we're looking
				// at, don't push that message out.
				if entry.work_proved >= work_proved { return false }
				found_diff += self.slab[entry.slab_id].encoded_size();
			}

			// message larger than ideal size.
			if found_diff < diff { return false }

			while found_diff > 0 {
				let entry = self.sorted.pop()
					.expect("found_diff built by traversing entries; therefore that many entries exist; qed");

				let message = self.slab.remove(entry.slab_id)
					.expect("sorted entry slab IDs always filled; qed");

				found_diff -= message.encoded_size();

				self.cumulative_size -= message.encoded_size();
				self.known.remove(message.hash());
				self.removed_hashes.push(message.hash().clone());
			}
		}

		let expiry = message.expiry();

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

		true
	}

	// prune expired messages, and then prune low proof-of-work messages
	// until below ideal size.
	fn prune(&mut self, now: SystemTime) -> Vec<H256> {
		{
			let slab = &mut self.slab;
			let known = &mut self.known;
			let cumulative_size = &mut self.cumulative_size;
			let ideal_size = &self.ideal_size;
			let removed = &mut self.removed_hashes;

			// first pass, we look just at expired entries.
			let all_expired = self.sorted.iter()
				.filter(|entry| entry.expiry <= now)
				.map(|x| (true, x));

			// second pass, we look at entries which aren't expired but in order
			// by PoW
			let low_proof = self.sorted.iter().rev()
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
				removed.push(message.hash().clone());

				*cumulative_size -= message.encoded_size();
			}
		}

		// clear all the sorted entries we removed from slab.
		let slab = &self.slab;
		self.sorted.retain(|entry| slab.contains(entry.slab_id));

		::std::mem::replace(&mut self.removed_hashes, Vec::new())
	}

	fn iter(&self) -> ::slab::Iter<Message, usize> {
		self.slab.iter()
	}

	fn is_full(&self) -> bool {
		self.cumulative_size >= self.ideal_size
	}

	fn status(&self) -> PoolStatus {
		PoolStatus {
			required_pow: if self.is_full() {
				self.sorted.last().map(|entry| entry.work_proved.0)
			} else {
				None
			},
			message_count: self.sorted.len(),
			cumulative_size: self.cumulative_size,
			target_size: self.ideal_size,
		}
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
	fn note_evicted(&mut self, messages: &[H256]) {
		for message_hash in messages {
			self.known_messages.remove(message_hash);
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

/// Pool status.
pub struct PoolStatus {
	/// Required PoW to be accepted into the pool
	pub required_pow: Option<f64>,
	/// Number of messages in the pool.
	pub message_count: usize,
	/// Cumulative size of the messages in the pool
	pub cumulative_size: usize,
	/// Target size of the pool.
	pub target_size: usize,
}

/// Handle to the pool, for posting messages or getting info.
#[derive(Clone)]
pub struct PoolHandle {
	messages: Arc<RwLock<Messages>>,
}

impl PoolHandle {
	/// Post a message to the whisper network to be relayed.
	pub fn post_message(&self, message: Message) -> bool {
		self.messages.write().insert(message)
	}

	/// Get number of messages and amount of memory used by them.
	pub fn pool_status(&self) -> PoolStatus {
		self.messages.read().status()
	}
}

/// The whisper network protocol handler.
pub struct Network<T> {
	messages: Arc<RwLock<Messages>>,
	handler: T,
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>,
	node_key: RwLock<NodeId>,
}

// public API.
impl<T> Network<T> {
	/// Create a new network handler.
	pub fn new(messages_size_bytes: usize, handler: T) -> Self {
		Network {
			messages: Arc::new(RwLock::new(Messages::new(messages_size_bytes))),
			handler: handler,
			peers: RwLock::new(HashMap::new()),
			node_key: RwLock::new(Default::default()),
		}
	}

	/// Acquire a sender to asynchronously feed messages to the whisper
	/// network.
	pub fn handle(&self) -> PoolHandle {
		PoolHandle { messages: self.messages.clone() }
	}
}

impl<T: MessageHandler> Network<T> {
	fn rally(&self, io: &NetworkContext) {
		// cannot be greater than 16MB (protocol limitation)
		const MAX_MESSAGES_PACKET_SIZE: usize = 8 * 1024 * 1024;

		// prune messages.
		let now = SystemTime::now();
		let pruned_hashes = self.messages.write().prune(now);

		let messages = self.messages.read();
		let peers = self.peers.read();

		// send each peer a packet with new messages it may find relevant.
		for (peer_id, peer) in peers.iter() {
			let mut peer_data = peer.lock();
			peer_data.note_evicted(&pruned_hashes);

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
		let mut messages_vec = {
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
			let mut messages_vec = message_packet.iter().map(|rlp| Message::decode(rlp, now))
				.collect::<Result<Vec<_>, _>>()?;

			if messages_vec.is_empty() { return Ok(()) }

			// disallow duplicates in packet.
			messages_vec.retain(|message| peer.note_known(&message));
			messages_vec
		};

		// import for relaying.
		let mut messages = self.messages.write();

		messages_vec.retain(|message| messages.may_accept(&message));
		messages.reserve(messages_vec.len());

		self.handler.handle_messages(&messages_vec);

		for message in messages_vec {
			messages.insert(message);
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

impl<T: MessageHandler> ::network::NetworkProtocolHandler for Network<T> {
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
