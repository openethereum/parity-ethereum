// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! PLP Protocol Version 1 implementation.
//!
//! This uses a "Provider" to answer requests.

use transaction::UnverifiedTransaction;

use io::TimerToken;
use network::{NetworkProtocolHandler, NetworkContext, PeerId};
use rlp::{RlpStream, Rlp};
use ethereum_types::{H256, U256};
use kvdb::DBValue;
use parking_lot::{Mutex, RwLock};
use std::time::{Duration, Instant};

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::{BitOr, BitAnd, Not};

use provider::Provider;
use request::{Request, NetworkRequests as Requests, Response};

use self::request_credits::{Credits, FlowParams};
use self::context::{Ctx, TickCtx};
use self::error::Punishment;
use self::load_timer::{LoadDistribution, NullStore};
use self::request_set::RequestSet;
use self::id_guard::IdGuard;

mod context;
mod error;
mod load_timer;
mod status;
mod request_set;

#[cfg(test)]
mod tests;

pub mod request_credits;

pub use self::context::{BasicContext, EventContext, IoContext};
pub use self::error::Error;
pub use self::load_timer::{SampleStore, FileStore};
pub use self::status::{Status, Capabilities, Announcement};

const TIMEOUT: TimerToken = 0;
const TIMEOUT_INTERVAL: Duration = Duration::from_secs(1);

const TICK_TIMEOUT: TimerToken = 1;
const TICK_TIMEOUT_INTERVAL: Duration = Duration::from_secs(5);

const PROPAGATE_TIMEOUT: TimerToken = 2;
const PROPAGATE_TIMEOUT_INTERVAL: Duration = Duration::from_secs(5);

const RECALCULATE_COSTS_TIMEOUT: TimerToken = 3;
const RECALCULATE_COSTS_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Max number of transactions in a single packet.
const MAX_TRANSACTIONS_TO_PROPAGATE: usize = 64;

// minimum interval between updates.
const UPDATE_INTERVAL: Duration = Duration::from_millis(5000);

/// Packet count for PIP.
const PACKET_COUNT_V1: u8 = 9;

/// Supported protocol versions.
pub const PROTOCOL_VERSIONS: &'static [(u8, u8)] = &[
	(1, PACKET_COUNT_V1),
];

/// Max protocol version.
pub const MAX_PROTOCOL_VERSION: u8 = 1;

// packet ID definitions.
mod packet {
	// the status packet.
	pub const STATUS: u8 = 0x00;

	// announcement of new block hashes or capabilities.
	pub const ANNOUNCE: u8 = 0x01;

	// request and response.
	pub const REQUEST: u8 = 0x02;
	pub const RESPONSE: u8 = 0x03;

	// request credits update and acknowledgement.
	pub const UPDATE_CREDITS: u8 = 0x04;
	pub const ACKNOWLEDGE_UPDATE: u8 = 0x05;

	// relay transactions to peers.
	pub const SEND_TRANSACTIONS: u8 = 0x06;

	// two packets were previously meant to be reserved for epoch proofs.
	// these have since been moved to requests.
}

// timeouts for different kinds of requests. all values are in milliseconds.
mod timeout {
	use std::time::Duration;

	pub const HANDSHAKE: Duration = Duration::from_millis(4_000);
	pub const ACKNOWLEDGE_UPDATE: Duration = Duration::from_millis(5_000);
	pub const BASE: u64 = 2_500; // base timeout for packet.

	// timeouts per request within packet.
	pub const HEADERS: u64 = 250; // per header?
	pub const TRANSACTION_INDEX: u64 = 100;
	pub const BODY: u64 = 50;
	pub const RECEIPT: u64 = 50;
	pub const PROOF: u64 = 100; // state proof
	pub const CONTRACT_CODE: u64 = 100;
	pub const HEADER_PROOF: u64 = 100;
	pub const TRANSACTION_PROOF: u64 = 1000; // per gas?
	pub const EPOCH_SIGNAL: u64 = 200;
}

/// A request id.
#[cfg(not(test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ReqId(usize);

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ReqId(pub usize);

impl fmt::Display for ReqId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Request #{}", self.0)
	}
}

// A pending peer: one we've sent our status to but
// may not have received one for.
struct PendingPeer {
	sent_head: H256,
	last_update: Instant,
}

/// Relevant data to each peer. Not accessible publicly, only `pub` due to
/// limitations of the privacy system.
pub struct Peer {
	local_credits: Credits, // their credits relative to us
	status: Status,
	capabilities: Capabilities,
	remote_flow: Option<(Credits, FlowParams)>,
	sent_head: H256, // last chain head we've given them.
	last_update: Instant,
	pending_requests: RequestSet,
	failed_requests: Vec<ReqId>,
	propagated_transactions: HashSet<H256>,
	skip_update: bool,
	local_flow: Arc<FlowParams>,
	awaiting_acknowledge: Option<(Instant, Arc<FlowParams>)>,
}

/// Whether or not a peer was kept by a handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerStatus {
	/// The peer was kept
	Kept,
	/// The peer was not kept
	Unkept,
}

impl Not for PeerStatus {
	type Output = Self;

	fn not(self) -> Self {
		use self::PeerStatus::*;

		match self {
			Kept => Unkept,
			Unkept => Kept,
		}
	}
}

impl BitAnd for PeerStatus {
	type Output = Self;

	fn bitand(self, other: Self) -> Self {
		use self::PeerStatus::*;

		match (self, other) {
			(Kept, Kept) => Kept,
			_ => Unkept,
		}
	}
}

impl BitOr for PeerStatus {
	type Output = Self;

	fn bitor(self, other: Self) -> Self {
		use self::PeerStatus::*;

		match (self, other) {
			(_, Kept) | (Kept, _) => Kept,
			_ => Unkept,
		}
	}
}

/// A light protocol event handler.
///
/// Each handler function takes a context which describes the relevant peer
/// and gives references to the IO layer and protocol structure so new messages
/// can be dispatched immediately.
///
/// Request responses are not guaranteed to be complete or valid, but passed IDs will be correct.
/// Response handlers are not given a copy of the original request; it is assumed
/// that relevant data will be stored by interested handlers.
pub trait Handler: Send + Sync {
	/// Called when a peer connects.
	fn on_connect(
		&self,
		_ctx: &EventContext,
		_status: &Status,
		_capabilities: &Capabilities
	) -> PeerStatus { PeerStatus::Kept }
	/// Called when a peer disconnects, with a list of unfulfilled request IDs as
	/// of yet.
	fn on_disconnect(&self, _ctx: &EventContext, _unfulfilled: &[ReqId]) { }
	/// Called when a peer makes an announcement.
	fn on_announcement(&self, _ctx: &EventContext, _announcement: &Announcement) { }
	/// Called when a peer requests relay of some transactions.
	fn on_transactions(&self, _ctx: &EventContext, _relay: &[UnverifiedTransaction]) { }
	/// Called when a peer responds to requests.
	/// Responses not guaranteed to contain valid data and are not yet checked against
	/// the requests they correspond to.
	fn on_responses(&self, _ctx: &EventContext, _req_id: ReqId, _responses: &[Response]) { }
	/// Called when a peer responds with a transaction proof. Each proof is a vector of state items.
	fn on_transaction_proof(&self, _ctx: &EventContext, _req_id: ReqId, _state_items: &[DBValue]) { }
	/// Called to "tick" the handler periodically.
	fn tick(&self, _ctx: &BasicContext) { }
	/// Called on abort. This signals to handlers that they should clean up
	/// and ignore peers.
	// TODO: coreresponding `on_activate`?
	fn on_abort(&self) { }
}

/// Configuration.
pub struct Config {
	/// How many stored seconds of credits peers should be able to accumulate.
	pub max_stored_seconds: u64,
	/// How much of the total load capacity each peer should be allowed to take.
	pub load_share: f64,
}

impl Default for Config {
	fn default() -> Self {
		const LOAD_SHARE: f64 = 1.0 / 25.0;
		const MAX_ACCUMULATED: u64 = 60 * 5; // only charge for 5 minutes.

		Config {
			max_stored_seconds: MAX_ACCUMULATED,
			load_share: LOAD_SHARE,
		}
	}
}

/// Protocol initialization parameters.
pub struct Params {
	/// Network id.
	pub network_id: u64,
	/// Config.
	pub config: Config,
	/// Initial capabilities.
	pub capabilities: Capabilities,
	/// The sample store (`None` if data shouldn't persist between runs).
	pub sample_store: Option<Box<SampleStore>>,
}

/// Type alias for convenience.
pub type PeerMap = HashMap<PeerId, Mutex<Peer>>;

mod id_guard {

	use network::PeerId;
	use parking_lot::RwLockReadGuard;

	use super::{PeerMap, ReqId};

	// Guards success or failure of given request.
	// On drop, inserts the req_id into the "failed requests"
	// set for the peer unless defused. In separate module to enforce correct usage.
	pub struct IdGuard<'a> {
		peers: RwLockReadGuard<'a, PeerMap>,
		peer_id: PeerId,
		req_id: ReqId,
		active: bool,
	}

	impl<'a> IdGuard<'a> {
		/// Create a new `IdGuard`, which will prevent access of the inner ReqId
		/// (for forming responses, triggering handlers) until defused
		pub fn new(peers: RwLockReadGuard<'a, PeerMap>, peer_id: PeerId, req_id: ReqId) -> Self {
			IdGuard {
				peers: peers,
				peer_id: peer_id,
				req_id: req_id,
				active: true,
			}
		}

		/// Defuse the guard, signalling that the request has been successfully decoded.
		pub fn defuse(mut self) -> ReqId {
			// can't use the mem::forget trick here since we need the
			// read guard to drop.
			self.active = false;
			self.req_id
		}
	}

	impl<'a> Drop for IdGuard<'a> {
		fn drop(&mut self) {
			if !self.active { return }
			if let Some(p) = self.peers.get(&self.peer_id) {
				p.lock().failed_requests.push(self.req_id);
			}
		}
	}
}

/// This is an implementation of the light ethereum network protocol, abstracted
/// over a `Provider` of data and a p2p network.
///
/// This is simply designed for request-response purposes. Higher level uses
/// of the protocol, such as synchronization, will function as wrappers around
/// this system.
//
// LOCK ORDER:
//   Locks must be acquired in the order declared, and when holding a read lock
//   on the peers, only one peer may be held at a time.
pub struct LightProtocol {
	provider: Arc<Provider>,
	config: Config,
	genesis_hash: H256,
	network_id: u64,
	pending_peers: RwLock<HashMap<PeerId, PendingPeer>>,
	peers: RwLock<PeerMap>,
	capabilities: RwLock<Capabilities>,
	flow_params: RwLock<Arc<FlowParams>>,
	handlers: Vec<Arc<Handler>>,
	req_id: AtomicUsize,
	sample_store: Box<SampleStore>,
	load_distribution: LoadDistribution,
}

impl LightProtocol {
	/// Create a new instance of the protocol manager.
	pub fn new(provider: Arc<Provider>, params: Params) -> Self {
		debug!(target: "pip", "Initializing light protocol handler");

		let genesis_hash = provider.chain_info().genesis_hash;
		let sample_store = params.sample_store.unwrap_or_else(|| Box::new(NullStore));
		let load_distribution = LoadDistribution::load(&*sample_store);
		let flow_params = FlowParams::from_request_times(
			|kind| load_distribution.expected_time(kind),
			params.config.load_share,
			Duration::from_secs(params.config.max_stored_seconds),
		);

		LightProtocol {
			provider: provider,
			config: params.config,
			genesis_hash: genesis_hash,
			network_id: params.network_id,
			pending_peers: RwLock::new(HashMap::new()),
			peers: RwLock::new(HashMap::new()),
			capabilities: RwLock::new(params.capabilities),
			flow_params: RwLock::new(Arc::new(flow_params)),
			handlers: Vec::new(),
			req_id: AtomicUsize::new(0),
			sample_store: sample_store,
			load_distribution: load_distribution,
		}
	}

	/// Attempt to get peer status.
	pub fn peer_status(&self, peer: &PeerId) -> Option<Status> {
		self.peers.read().get(&peer)
			.map(|peer| peer.lock().status.clone())
	}

	/// Get number of (connected, active) peers.
	pub fn peer_count(&self) -> (usize, usize) {
		let num_pending = self.pending_peers.read().len();
		let peers = self.peers.read();
		(
			num_pending + peers.len(),
			peers.values().filter(|p| !p.lock().pending_requests.is_empty()).count(),
		)
	}

	/// Make a request to a peer.
	///
	/// Fails on: nonexistent peer, network error, peer not server,
	/// insufficient credits. Does not check capabilities before sending.
	/// On success, returns a request id which can later be coordinated
	/// with an event.
	pub fn request_from(&self, io: &IoContext, peer_id: &PeerId, requests: Requests) -> Result<ReqId, Error> {
		let peers = self.peers.read();
		let peer = match peers.get(peer_id) {
			Some(peer) => peer,
			None => return Err(Error::UnknownPeer),
		};

		let mut peer = peer.lock();
		let peer = &mut *peer;
		match peer.remote_flow {
			None => Err(Error::NotServer),
			Some((ref mut creds, ref params)) => {
				// apply recharge to credits if there's no pending requests.
				if peer.pending_requests.is_empty() {
					params.recharge(creds);
				}

				// compute and deduct cost.
				let pre_creds = creds.current();
				let cost = match params.compute_cost_multi(requests.requests()) {
					Some(cost) => cost,
					None => return Err(Error::NotServer),
				};

				creds.deduct_cost(cost)?;

				trace!(target: "pip", "requesting from peer {}. Cost: {}; Available: {}",
					peer_id, cost, pre_creds);

				let req_id = ReqId(self.req_id.fetch_add(1, Ordering::SeqCst));
				io.send(*peer_id, packet::REQUEST, {
					let mut stream = RlpStream::new_list(2);
					stream.append(&req_id.0).append_list(&requests.requests());
					stream.out()
				});

				// begin timeout.
				peer.pending_requests.insert(req_id, requests, cost, Instant::now());
				Ok(req_id)
			}
		}
	}

	/// Make an announcement of new chain head and capabilities to all peers.
	/// The announcement is expected to be valid.
	pub fn make_announcement(&self, io: &IoContext, mut announcement: Announcement) {
		let mut reorgs_map = HashMap::new();
		let now = Instant::now();

		// update stored capabilities
		self.capabilities.write().update_from(&announcement);

		// calculate reorg info and send packets
		for (peer_id, peer_info) in self.peers.read().iter() {
			let mut peer_info = peer_info.lock();

			// TODO: "urgent" announcements like new blocks?
			// the timer approach will skip 1 (possibly 2) in rare occasions.
			if peer_info.sent_head == announcement.head_hash ||
				peer_info.status.head_num >= announcement.head_num  ||
				now - peer_info.last_update < UPDATE_INTERVAL {
				continue
			}

			peer_info.last_update = now;

			let reorg_depth = reorgs_map.entry(peer_info.sent_head)
				.or_insert_with(|| {
					match self.provider.reorg_depth(&announcement.head_hash, &peer_info.sent_head) {
						Some(depth) => depth,
						None => {
							// both values will always originate locally -- this means something
							// has gone really wrong
							debug!(target: "pip", "couldn't compute reorganization depth between {:?} and {:?}",
								&announcement.head_hash, &peer_info.sent_head);
							0
						}
					}
				});

			peer_info.sent_head = announcement.head_hash;
			announcement.reorg_depth = *reorg_depth;

			io.send(*peer_id, packet::ANNOUNCE, status::write_announcement(&announcement));
		}
	}

	/// Add an event handler.
	///
	/// These are intended to be added when the protocol structure
	/// is initialized as a means of customizing its behavior,
	/// and dispatching requests immediately upon events.
	pub fn add_handler(&mut self, handler: Arc<Handler>) {
		self.handlers.push(handler);
	}

	/// Signal to handlers that network activity is being aborted
	/// and clear peer data.
	pub fn abort(&self) {
		for handler in &self.handlers {
			handler.on_abort();
		}

		// acquire in order and hold.
		let mut pending_peers = self.pending_peers.write();
		let mut peers = self.peers.write();

		pending_peers.clear();
		peers.clear();
	}

	// Does the common pre-verification of responses before the response itself
	// is actually decoded:
	//   - check whether peer exists
	//   - check whether request was made
	//   - check whether request kinds match
	fn pre_verify_response(&self, peer: &PeerId, raw: &Rlp) -> Result<IdGuard, Error> {
		let req_id = ReqId(raw.val_at(0)?);
		let cur_credits: U256 = raw.val_at(1)?;

		trace!(target: "pip", "pre-verifying response for {} from peer {}", req_id, peer);

		let peers = self.peers.read();
		let res = match peers.get(peer) {
			Some(peer_info) => {
				let mut peer_info = peer_info.lock();
				let peer_info: &mut Peer = &mut *peer_info;
				let req_info = peer_info.pending_requests.remove(&req_id, Instant::now());
				let last_batched = peer_info.pending_requests.is_empty();
				let flow_info = peer_info.remote_flow.as_mut();

				match (req_info, flow_info) {
					(Some(_), Some(flow_info)) => {
						let &mut (ref mut c, ref mut flow) = flow_info;

						// only update if the cumulative cost of the request set is zero.
						// and this response wasn't from before request costs were updated.
						if !peer_info.skip_update && last_batched {
							let actual_credits = ::std::cmp::min(cur_credits, *flow.limit());
							c.update_to(actual_credits);
						}

						if last_batched { peer_info.skip_update = false }

						Ok(())
					}
					(None, _) => Err(Error::UnsolicitedResponse),
					(_, None) => Err(Error::NotServer), // really should be impossible.
				}
			}
			None => Err(Error::UnknownPeer), // probably only occurs in a race of some kind.
		};

		res.map(|_| IdGuard::new(peers, *peer, req_id))
	}

	/// Handle a packet using the given io context.
	/// Packet data is _untrusted_, which means that invalid data won't lead to
	/// issues.
	pub fn handle_packet(&self, io: &IoContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let rlp = Rlp::new(data);

		trace!(target: "pip", "Incoming packet {} from peer {}", packet_id, peer);

		// handle the packet
		let res = match packet_id {
			packet::STATUS => self.status(peer, io, rlp),
			packet::ANNOUNCE => self.announcement(peer, io, rlp),

			packet::REQUEST => self.request(peer, io, rlp),
			packet::RESPONSE => self.response(peer, io, rlp),

			packet::UPDATE_CREDITS => self.update_credits(peer, io, rlp),
			packet::ACKNOWLEDGE_UPDATE => self.acknowledge_update(peer, io, rlp),

			packet::SEND_TRANSACTIONS => self.relay_transactions(peer, io, rlp),

			other => {
				Err(Error::UnrecognizedPacket(other))
			}
		};

		if let Err(e) = res {
			punish(*peer, io, e);
		}
	}

	// check timeouts and punish peers.
	fn timeout_check(&self, io: &IoContext) {
		let now = Instant::now();

		// handshake timeout
		{
			let mut pending = self.pending_peers.write();
			let slowpokes: Vec<_> = pending.iter()
				.filter(|&(_, ref peer)| {
					peer.last_update + timeout::HANDSHAKE <= now
				})
				.map(|(&p, _)| p)
				.collect();

			for slowpoke in slowpokes {
				debug!(target: "pip", "Peer {} handshake timed out", slowpoke);
				pending.remove(&slowpoke);
				io.disconnect_peer(slowpoke);
			}
		}

		// request and update ack timeouts
		let ack_duration = timeout::ACKNOWLEDGE_UPDATE;
		{
			for (peer_id, peer) in self.peers.read().iter() {
				let peer = peer.lock();
				if peer.pending_requests.check_timeout(now) {
					debug!(target: "pip", "Peer {} request timeout", peer_id);
					io.disconnect_peer(*peer_id);
				}

				if let Some((ref start, _)) = peer.awaiting_acknowledge {
					if *start + ack_duration <= now {
						debug!(target: "pip", "Peer {} update acknowledgement timeout", peer_id);
						io.disconnect_peer(*peer_id);
					}
				}
			}
		}
	}

	// propagate transactions to relay peers.
	// if we aren't on the mainnet, we just propagate to all relay peers
	fn propagate_transactions(&self, io: &IoContext) {
		if self.capabilities.read().tx_relay { return }

		let ready_transactions = self.provider.ready_transactions(MAX_TRANSACTIONS_TO_PROPAGATE);
		if ready_transactions.is_empty() { return }

		trace!(target: "pip", "propagate transactions: {} ready", ready_transactions.len());

		let all_transaction_hashes: HashSet<_> = ready_transactions.iter().map(|tx| tx.hash()).collect();
		let mut buf = Vec::new();

		let peers = self.peers.read();
		for (peer_id, peer_info) in peers.iter() {
			let mut peer_info = peer_info.lock();
			if !peer_info.capabilities.tx_relay { continue }

			let prop_filter = &mut peer_info.propagated_transactions;
			*prop_filter = &*prop_filter & &all_transaction_hashes;

			// fill the buffer with all non-propagated transactions.
			let to_propagate = ready_transactions.iter()
				.filter(|tx| prop_filter.insert(tx.hash()))
				.map(|tx| &tx.transaction);

			buf.extend(to_propagate);

			// propagate to the given peer.
			if buf.is_empty() { continue }
			io.send(*peer_id, packet::SEND_TRANSACTIONS, {
				let mut stream = RlpStream::new_list(buf.len());
				for pending_tx in buf.drain(..) {
					stream.append(pending_tx);
				}

				stream.out()
			})
		}
	}

	/// called when a peer connects.
	pub fn on_connect(&self, peer: &PeerId, io: &IoContext) {
		let proto_version = match io.protocol_version(*peer).ok_or(Error::WrongNetwork) {
			Ok(pv) => pv,
			Err(e) => { punish(*peer, io, e); return }
		};

		if PROTOCOL_VERSIONS.iter().find(|x| x.0 == proto_version).is_none() {
			punish(*peer, io, Error::UnsupportedProtocolVersion(proto_version));
			return;
		}

		let chain_info = self.provider.chain_info();

		let status = Status {
			head_td: chain_info.total_difficulty,
			head_hash: chain_info.best_block_hash,
			head_num: chain_info.best_block_number,
			genesis_hash: chain_info.genesis_hash,
			protocol_version: proto_version as u32, // match peer proto version
			network_id: self.network_id,
			last_head: None,
		};

		let capabilities = self.capabilities.read().clone();
		let local_flow = self.flow_params.read();
		let status_packet = status::write_handshake(&status, &capabilities, Some(&**local_flow));

		self.pending_peers.write().insert(*peer, PendingPeer {
			sent_head: chain_info.best_block_hash,
			last_update: Instant::now(),
		});

		trace!(target: "pip", "Sending status to peer {}", peer);
		io.send(*peer, packet::STATUS, status_packet);
	}

	/// called when a peer disconnects.
	pub fn on_disconnect(&self, peer: PeerId, io: &IoContext) {
		trace!(target: "pip", "Peer {} disconnecting", peer);

		self.pending_peers.write().remove(&peer);
		let unfulfilled = match self.peers.write().remove(&peer) {
			None => return,
			Some(peer_info) => {
				let peer_info = peer_info.into_inner();
				let mut unfulfilled: Vec<_> = peer_info.pending_requests.collect_ids();
				unfulfilled.extend(peer_info.failed_requests);

				unfulfilled
			}
		};

		for handler in &self.handlers {
			handler.on_disconnect(&Ctx {
				peer: peer,
				io: io,
				proto: self,
			}, &unfulfilled)
		}
	}

	/// Execute the given closure with a basic context derived from the I/O context.
	pub fn with_context<F, T>(&self, io: &IoContext, f: F) -> T
		where F: FnOnce(&BasicContext) -> T
	{
		f(&TickCtx {
			io: io,
			proto: self,
		})
	}

	fn tick_handlers(&self, io: &IoContext) {
		for handler in &self.handlers {
			handler.tick(&TickCtx {
				io: io,
				proto: self,
			})
		}
	}

	fn begin_new_cost_period(&self, io: &IoContext) {
		self.load_distribution.end_period(&*self.sample_store);

		let new_params = Arc::new(FlowParams::from_request_times(
			|kind| self.load_distribution.expected_time(kind),
			self.config.load_share,
			Duration::from_secs(self.config.max_stored_seconds),
		));
		*self.flow_params.write() = new_params.clone();

		let peers = self.peers.read();
		let now = Instant::now();

		let packet_body = {
			let mut stream = RlpStream::new_list(3);
			stream.append(new_params.limit())
				.append(new_params.recharge_rate())
				.append(new_params.cost_table());
			stream.out()
		};

		for (peer_id, peer_info) in peers.iter() {
			let mut peer_info = peer_info.lock();

			io.send(*peer_id, packet::UPDATE_CREDITS, packet_body.clone());
			peer_info.awaiting_acknowledge = Some((now.clone(), new_params.clone()));
		}
	}
}

impl LightProtocol {
	// Handle status message from peer.
	fn status(&self, peer: &PeerId, io: &IoContext, data: Rlp) -> Result<(), Error> {
		let pending = match self.pending_peers.write().remove(peer) {
			Some(pending) => pending,
			None => {
				return Err(Error::UnexpectedHandshake);
			}
		};

		let (status, capabilities, flow_params) = status::parse_handshake(data)?;

		trace!(target: "pip", "Connected peer with chain head {:?}", (status.head_hash, status.head_num));

		if (status.network_id, status.genesis_hash) != (self.network_id, self.genesis_hash) {
			trace!(target: "pip", "peer {} wrong network: network_id is {} vs our {}, gh is {} vs our {}",
				peer, status.network_id, self.network_id, status.genesis_hash, self.genesis_hash);

			return Err(Error::WrongNetwork);
		}

		if Some(status.protocol_version as u8) != io.protocol_version(*peer) {
			return Err(Error::BadProtocolVersion);
		}

		let remote_flow = flow_params.map(|params| (params.create_credits(), params));
		let local_flow = self.flow_params.read().clone();

		self.peers.write().insert(*peer, Mutex::new(Peer {
			local_credits: local_flow.create_credits(),
			status: status.clone(),
			capabilities: capabilities.clone(),
			remote_flow: remote_flow,
			sent_head: pending.sent_head,
			last_update: pending.last_update,
			pending_requests: RequestSet::default(),
			failed_requests: Vec::new(),
			propagated_transactions: HashSet::new(),
			skip_update: false,
			local_flow: local_flow,
			awaiting_acknowledge: None,
		}));

		let any_kept = self.handlers.iter().map(
			|handler| handler.on_connect(
				&Ctx {
					peer: *peer,
					io: io,
					proto: self,
				},
				&status,
				&capabilities
			)
		).fold(PeerStatus::Kept, PeerStatus::bitor);

		if any_kept == PeerStatus::Unkept {
			Err(Error::RejectedByHandlers)
		} else {
			Ok(())
		}
	}

	// Handle an announcement.
	fn announcement(&self, peer: &PeerId, io: &IoContext, data: Rlp) -> Result<(), Error> {
		if !self.peers.read().contains_key(peer) {
			debug!(target: "pip", "Ignoring announcement from unknown peer");
			return Ok(())
		}

		let announcement = status::parse_announcement(data)?;

		// scope to ensure locks are dropped before moving into handler-space.
		{
			let peers = self.peers.read();
			let peer_info = match peers.get(peer) {
				Some(info) => info,
				None => return Ok(()),
			};

			let mut peer_info = peer_info.lock();

			// update status.
			{
				// TODO: punish peer if they've moved backwards.
				let status = &mut peer_info.status;
				let last_head = status.head_hash;
				status.head_hash = announcement.head_hash;
				status.head_td = announcement.head_td;
				status.head_num = announcement.head_num;
				status.last_head = Some((last_head, announcement.reorg_depth));
			}

			// update capabilities.
			peer_info.capabilities.update_from(&announcement);
		}

		for handler in &self.handlers {
			handler.on_announcement(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, &announcement);
		}

		Ok(())
	}

	// Receive requests from a peer.
	fn request(&self, peer_id: &PeerId, io: &IoContext, raw: Rlp) -> Result<(), Error> {
		// the maximum amount of requests we'll fill in a single packet.
		const MAX_REQUESTS: usize = 256;

		use ::request::Builder;
		use ::request::CompleteRequest;

		let peers = self.peers.read();
		let peer = match peers.get(peer_id) {
			Some(peer) => peer,
			None => {
				debug!(target: "pip", "Ignoring request from unknown peer");
				return Ok(())
			}
		};
		let mut peer = peer.lock();
		let peer: &mut Peer = &mut *peer;

		let req_id: u64 = raw.val_at(0)?;
		let mut request_builder = Builder::default();

		trace!(target: "pip", "Received requests (id: {}) from peer {}", req_id, peer_id);

		// deserialize requests, check costs and request validity.
		peer.local_flow.recharge(&mut peer.local_credits);

		peer.local_credits.deduct_cost(peer.local_flow.base_cost())?;
		for request_rlp in raw.at(1)?.iter().take(MAX_REQUESTS) {
			let request: Request = request_rlp.as_val()?;
			let cost = peer.local_flow.compute_cost(&request).ok_or(Error::NotServer)?;
			peer.local_credits.deduct_cost(cost)?;
			request_builder.push(request).map_err(|_| Error::BadBackReference)?;
		}

		let requests = request_builder.build();
		let num_requests = requests.requests().len();
		trace!(target: "pip", "Beginning to respond to requests (id: {}) from peer {}", req_id, peer_id);

		// respond to all requests until one fails.
		let responses = requests.respond_to_all(|complete_req| {
			let _timer = self.load_distribution.begin_timer(&complete_req);
			match complete_req {
				CompleteRequest::Headers(req) => self.provider.block_headers(req).map(Response::Headers),
				CompleteRequest::HeaderProof(req) => self.provider.header_proof(req).map(Response::HeaderProof),
				CompleteRequest::TransactionIndex(req) => self.provider.transaction_index(req).map(Response::TransactionIndex),
				CompleteRequest::Body(req) => self.provider.block_body(req).map(Response::Body),
				CompleteRequest::Receipts(req) => self.provider.block_receipts(req).map(Response::Receipts),
				CompleteRequest::Account(req) => self.provider.account_proof(req).map(Response::Account),
				CompleteRequest::Storage(req) => self.provider.storage_proof(req).map(Response::Storage),
				CompleteRequest::Code(req) => self.provider.contract_code(req).map(Response::Code),
				CompleteRequest::Execution(req) => self.provider.transaction_proof(req).map(Response::Execution),
				CompleteRequest::Signal(req) => self.provider.epoch_signal(req).map(Response::Signal),
			}
		});

		trace!(target: "pip", "Responded to {}/{} requests in packet {}", responses.len(), num_requests, req_id);
		trace!(target: "pip", "Peer {} has {} credits remaining.", peer_id, peer.local_credits.current());

		io.respond(packet::RESPONSE, {
			let mut stream = RlpStream::new_list(3);
			let cur_credits = peer.local_credits.current();
			stream.append(&req_id).append(&cur_credits).append_list(&responses);
			stream.out()
		});
		Ok(())
	}

	// handle a packet with responses.
	fn response(&self, peer: &PeerId, io: &IoContext, raw: Rlp) -> Result<(), Error> {
		let (req_id, responses) = {
			let id_guard = self.pre_verify_response(peer, &raw)?;
			let responses: Vec<Response> = raw.list_at(2)?;
			(id_guard.defuse(), responses)
		};

		for handler in &self.handlers {
			handler.on_responses(&Ctx {
				io: io,
				proto: self,
				peer: *peer,
			}, req_id, &responses);
		}

		Ok(())
	}

	// handle an update of request credits parameters.
	fn update_credits(&self, peer_id: &PeerId, io: &IoContext, raw: Rlp) -> Result<(), Error> {
		let peers = self.peers.read();

		let peer = peers.get(peer_id).ok_or(Error::UnknownPeer)?;
		let mut peer = peer.lock();

		trace!(target: "pip", "Received an update to request credit params from peer {}", peer_id);

		{
			let &mut (ref mut credits, ref mut old_params) = peer.remote_flow.as_mut().ok_or(Error::NotServer)?;
			old_params.recharge(credits);

			let new_params = FlowParams::new(
				raw.val_at(0)?, // limit
				raw.val_at(2)?, // cost table
				raw.val_at(1)?, // recharge.
			);

			// preserve ratio of current : limit when updating params.
			credits.maintain_ratio(*old_params.limit(), *new_params.limit());
			*old_params = new_params;
		}

		// set flag to true when there is an in-flight request
		// corresponding to old flow params.
		if !peer.pending_requests.is_empty() {
			peer.skip_update = true;
		}

		// let peer know we've acknowledged the update.
		io.respond(packet::ACKNOWLEDGE_UPDATE, Vec::new());
		Ok(())
	}

	// handle an acknowledgement of request credits update.
	fn acknowledge_update(&self, peer_id: &PeerId, _io: &IoContext, _raw: Rlp) -> Result<(), Error> {
		let peers = self.peers.read();
		let peer = peers.get(peer_id).ok_or(Error::UnknownPeer)?;
		let mut peer = peer.lock();

		trace!(target: "pip", "Received an acknowledgement for new request credit params from peer {}", peer_id);

		let (_, new_params) = match peer.awaiting_acknowledge.take() {
			Some(x) => x,
			None => return Err(Error::UnsolicitedResponse),
		};

		let old_limit = *peer.local_flow.limit();
		peer.local_credits.maintain_ratio(old_limit, *new_params.limit());
		peer.local_flow = new_params;
		Ok(())
	}

	// Receive a set of transactions to relay.
	fn relay_transactions(&self, peer: &PeerId, io: &IoContext, data: Rlp) -> Result<(), Error> {
		const MAX_TRANSACTIONS: usize = 256;

		let txs: Vec<_> = data.iter()
			.take(MAX_TRANSACTIONS)
			.map(|x| x.as_val::<UnverifiedTransaction>())
			.collect::<Result<_,_>>()?;

		debug!(target: "pip", "Received {} transactions to relay from peer {}", txs.len(), peer);

		for handler in &self.handlers {
			handler.on_transactions(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, &txs);
		}

		Ok(())
	}
}

// if something went wrong, figure out how much to punish the peer.
fn punish(peer: PeerId, io: &IoContext, e: Error) {
	match e.punishment() {
		Punishment::None => {}
		Punishment::Disconnect => {
			debug!(target: "pip", "Disconnecting peer {}: {}", peer, e);
			io.disconnect_peer(peer)
		}
		Punishment::Disable => {
			debug!(target: "pip", "Disabling peer {}: {}", peer, e);
			io.disable_peer(peer)
		}
	}
}

impl NetworkProtocolHandler for LightProtocol {
	fn initialize(&self, io: &NetworkContext) {
		io.register_timer(TIMEOUT, TIMEOUT_INTERVAL)
			.expect("Error registering sync timer.");
		io.register_timer(TICK_TIMEOUT, TICK_TIMEOUT_INTERVAL)
			.expect("Error registering sync timer.");
		io.register_timer(PROPAGATE_TIMEOUT, PROPAGATE_TIMEOUT_INTERVAL)
			.expect("Error registering sync timer.");
		io.register_timer(RECALCULATE_COSTS_TIMEOUT, RECALCULATE_COSTS_INTERVAL)
			.expect("Error registering request timer interval token.");
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		self.handle_packet(&io, peer, packet_id, data);
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_connect(peer, &io);
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_disconnect(*peer, &io);
	}

	fn timeout(&self, io: &NetworkContext, timer: TimerToken) {
		match timer {
			TIMEOUT => self.timeout_check(&io),
			TICK_TIMEOUT => self.tick_handlers(&io),
			PROPAGATE_TIMEOUT => self.propagate_transactions(&io),
			RECALCULATE_COSTS_TIMEOUT => self.begin_new_cost_period(&io),
			_ => warn!(target: "pip", "received timeout on unknown token {}", timer),
		}
	}
}
