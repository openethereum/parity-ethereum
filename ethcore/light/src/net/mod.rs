// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! LES Protocol Version 1 implementation.
//!
//! This uses a "Provider" to answer requests.
//! See https://github.com/ethcore/parity/wiki/Light-Ethereum-Subprotocol-(LES)

use ethcore::transaction::SignedTransaction;
use ethcore::receipt::Receipt;

use io::TimerToken;
use network::{NetworkProtocolHandler, NetworkContext, PeerId};
use rlp::{RlpStream, Stream, UntrustedRlp, View};
use util::hash::H256;
use util::{Bytes, Mutex, RwLock, U256};
use time::{Duration, SteadyTime};

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use provider::Provider;
use request::{self, HashOrNumber, Request};

use self::buffer_flow::{Buffer, FlowParams};
use self::context::{Ctx, TickCtx};
use self::error::Punishment;

mod context;
mod error;
mod status;

#[cfg(test)]
mod tests;

pub mod buffer_flow;

pub use self::error::Error;
pub use self::context::{BasicContext, EventContext, IoContext};
pub use self::status::{Status, Capabilities, Announcement};

const TIMEOUT: TimerToken = 0;
const TIMEOUT_INTERVAL_MS: u64 = 1000;

const TICK_TIMEOUT: TimerToken = 1;
const TICK_TIMEOUT_INTERVAL_MS: u64 = 5000;

// minimum interval between updates.
const UPDATE_INTERVAL_MS: i64 = 5000;

/// Supported protocol versions.
pub const PROTOCOL_VERSIONS: &'static [u8] = &[1];

/// Max protocol version.
pub const MAX_PROTOCOL_VERSION: u8 = 1;

/// Packet count for LES.
pub const PACKET_COUNT: u8 = 15;

// packet ID definitions.
mod packet {
	// the status packet.
	pub const STATUS: u8 = 0x00;

	// announcement of new block hashes or capabilities.
	pub const ANNOUNCE: u8 = 0x01;

	// request and response for block headers
	pub const GET_BLOCK_HEADERS: u8 = 0x02;
	pub const BLOCK_HEADERS: u8 = 0x03;

	// request and response for block bodies
	pub const GET_BLOCK_BODIES: u8 = 0x04;
	pub const BLOCK_BODIES: u8 = 0x05;

	// request and response for transaction receipts.
	pub const GET_RECEIPTS: u8 = 0x06;
	pub const RECEIPTS: u8 = 0x07;

	// request and response for merkle proofs.
	pub const GET_PROOFS: u8 = 0x08;
	pub const PROOFS: u8 = 0x09;

	// request and response for contract code.
	pub const GET_CONTRACT_CODES: u8 = 0x0a;
	pub const CONTRACT_CODES: u8 = 0x0b;

	// relay transactions to peers.
	pub const SEND_TRANSACTIONS: u8 = 0x0c;

	// request and response for header proofs in a CHT.
	pub const GET_HEADER_PROOFS: u8 = 0x0d;
	pub const HEADER_PROOFS: u8 = 0x0e;
}

// timeouts for different kinds of requests. all values are in milliseconds.
// TODO: variable timeouts based on request count.
mod timeout {
	pub const HANDSHAKE: i64 = 2500;
	pub const HEADERS: i64 = 5000;
	pub const BODIES: i64 = 5000;
	pub const RECEIPTS: i64 = 3500;
	pub const PROOFS: i64 = 4000;
	pub const CONTRACT_CODES: i64 = 5000;
	pub const HEADER_PROOFS: i64 = 3500;
}

/// A request id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReqId(usize);

// A pending peer: one we've sent our status to but
// may not have received one for.
struct PendingPeer {
	sent_head: H256,
	last_update: SteadyTime,
}

// data about each peer.
struct Peer {
	local_buffer: Buffer, // their buffer relative to us
	status: Status,
	capabilities: Capabilities,
	remote_flow: Option<(Buffer, FlowParams)>,
	sent_head: H256, // last chain head we've given them.
	last_update: SteadyTime,
	idle: bool, // make into a current percentage of max buffer being requested?
}

impl Peer {
	// check the maximum cost of a request, returning an error if there's
	// not enough buffer left.
	// returns the calculated maximum cost.
	fn deduct_max(&mut self, flow_params: &FlowParams, kind: request::Kind, max: usize) -> Result<U256, Error> {
		flow_params.recharge(&mut self.local_buffer);

		let max_cost = flow_params.compute_cost(kind, max);
		self.local_buffer.deduct_cost(max_cost)?;
		Ok(max_cost)
	}

	// refund buffer for a request. returns new buffer amount.
	fn refund(&mut self, flow_params: &FlowParams, amount: U256) -> U256 {
		flow_params.refund(&mut self.local_buffer, amount);

		self.local_buffer.current()
	}
}

/// An LES event handler.
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
	fn on_connect(&self, _ctx: &EventContext, _status: &Status, _capabilities: &Capabilities) { }
	/// Called when a peer disconnects, with a list of unfulfilled request IDs as
	/// of yet.
	fn on_disconnect(&self, _ctx: &EventContext, _unfulfilled: &[ReqId]) { }
	/// Called when a peer makes an announcement.
	fn on_announcement(&self, _ctx: &EventContext, _announcement: &Announcement) { }
	/// Called when a peer requests relay of some transactions.
	fn on_transactions(&self, _ctx: &EventContext, _relay: &[SignedTransaction]) { }
	/// Called when a peer responds with block bodies.
	fn on_block_bodies(&self, _ctx: &EventContext, _req_id: ReqId, _bodies: &[Bytes]) { }
	/// Called when a peer responds with block headers.
	fn on_block_headers(&self, _ctx: &EventContext, _req_id: ReqId, _headers: &[Bytes]) { }
	/// Called when a peer responds with block receipts.
	fn on_receipts(&self, _ctx: &EventContext, _req_id: ReqId, _receipts: &[Vec<Receipt>]) { }
	/// Called when a peer responds with state proofs. Each proof is a series of trie
	/// nodes in ascending order by distance from the root.
	fn on_state_proofs(&self, _ctx: &EventContext, _req_id: ReqId, _proofs: &[Vec<Bytes>]) { }
	/// Called when a peer responds with contract code.
	fn on_code(&self, _ctx: &EventContext, _req_id: ReqId, _codes: &[Bytes]) { }
	/// Called when a peer responds with header proofs. Each proof is a block header coupled
	/// with a series of trie nodes is ascending order by distance from the root.
	fn on_header_proofs(&self, _ctx: &EventContext, _req_id: ReqId, _proofs: &[(Bytes, Vec<Bytes>)]) { }
	/// Called to "tick" the handler periodically.
	fn tick(&self, _ctx: &BasicContext) { }
	/// Called on abort. This signals to handlers that they should clean up
	/// and ignore peers.
	// TODO: coreresponding `on_activate`?
	fn on_abort(&self) { }
}

// a request, the peer who it was made to, and the time it was made.
struct Requested {
	request: Request,
	timestamp: SteadyTime,
	peer_id: PeerId,
}

/// Protocol parameters.
pub struct Params {
	/// Network id.
	pub network_id: u64,
	/// Buffer flow parameters.
	pub flow_params: FlowParams,
	/// Initial capabilities.
	pub capabilities: Capabilities,
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
	genesis_hash: H256,
	network_id: u64,
	pending_peers: RwLock<HashMap<PeerId, PendingPeer>>,
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>,
	pending_requests: RwLock<HashMap<usize, Requested>>,
	capabilities: RwLock<Capabilities>,
	flow_params: FlowParams, // assumed static and same for every peer.
	handlers: Vec<Arc<Handler>>,
	req_id: AtomicUsize,
}

impl LightProtocol {
	/// Create a new instance of the protocol manager.
	pub fn new(provider: Arc<Provider>, params: Params) -> Self {
		debug!(target: "les", "Initializing LES handler");

		let genesis_hash = provider.chain_info().genesis_hash;
		LightProtocol {
			provider: provider,
			genesis_hash: genesis_hash,
			network_id: params.network_id,
			pending_peers: RwLock::new(HashMap::new()),
			peers: RwLock::new(HashMap::new()),
			pending_requests: RwLock::new(HashMap::new()),
			capabilities: RwLock::new(params.capabilities),
			flow_params: params.flow_params,
			handlers: Vec::new(),
			req_id: AtomicUsize::new(0),
		}
	}

	/// Check the maximum amount of requests of a specific type
	/// which a peer would be able to serve. Returns zero if the
	/// peer is unknown or has no buffer flow parameters.
	fn max_requests(&self, peer: PeerId, kind: request::Kind) -> usize {
		self.peers.read().get(&peer).and_then(|peer| {
			let mut peer = peer.lock();
			let idle = peer.idle;
			match peer.remote_flow {
				Some((ref mut buf, ref flow)) => {
					flow.recharge(buf);

					if !idle {
						Some(0)
					} else {
						Some(flow.max_amount(&*buf, kind))
					}
				}
				None => None,
			}
		}).unwrap_or(0)
	}

	/// Make a request to a peer.
	///
	/// Fails on: nonexistent peer, network error, peer not server,
	/// insufficient buffer. Does not check capabilities before sending.
	/// On success, returns a request id which can later be coordinated
	/// with an event.
	pub fn request_from(&self, io: &IoContext, peer_id: &PeerId, request: Request) -> Result<ReqId, Error> {
		let peers = self.peers.read();
		let peer = peers.get(peer_id).ok_or_else(|| Error::UnknownPeer)?;
		let mut peer = peer.lock();

		if !peer.idle { return Err(Error::Overburdened) }

		match peer.remote_flow {
			Some((ref mut buf, ref flow)) => {
				flow.recharge(buf);
				let max = flow.compute_cost(request.kind(), request.amount());
				buf.deduct_cost(max)?;
			}
			None => return Err(Error::NotServer),
		}

		let req_id = self.req_id.fetch_add(1, Ordering::SeqCst);
		let packet_data = encode_request(&request, req_id);

		trace!(target: "les", "Dispatching request {} to peer {}", req_id, peer_id);

		let packet_id = match request.kind() {
			request::Kind::Headers => packet::GET_BLOCK_HEADERS,
			request::Kind::Bodies => packet::GET_BLOCK_BODIES,
			request::Kind::Receipts => packet::GET_RECEIPTS,
			request::Kind::StateProofs => packet::GET_PROOFS,
			request::Kind::Codes => packet::GET_CONTRACT_CODES,
			request::Kind::HeaderProofs => packet::GET_HEADER_PROOFS,
		};

		io.send(*peer_id, packet_id, packet_data);

		peer.idle = false;
		self.pending_requests.write().insert(req_id, Requested {
			request: request,
			timestamp: SteadyTime::now(),
			peer_id: *peer_id,
		});

		Ok(ReqId(req_id))
	}

	/// Make an announcement of new chain head and capabilities to all peers.
	/// The announcement is expected to be valid.
	pub fn make_announcement(&self, io: &IoContext, mut announcement: Announcement) {
		let mut reorgs_map = HashMap::new();
		let now = SteadyTime::now();

		// update stored capabilities
		self.capabilities.write().update_from(&announcement);

		// calculate reorg info and send packets
		for (peer_id, peer_info) in self.peers.read().iter() {
			let mut peer_info = peer_info.lock();

			// TODO: "urgent" announcements like new blocks?
			// the timer approach will skip 1 (possibly 2) in rare occasions.
			if peer_info.sent_head == announcement.head_hash ||
				peer_info.status.head_num >= announcement.head_num  ||
				now - peer_info.last_update < Duration::milliseconds(UPDATE_INTERVAL_MS) {
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
							debug!(target: "les", "couldn't compute reorganization depth between {:?} and {:?}",
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
		let mut pending_requests = self.pending_requests.write();

		pending_peers.clear();
		peers.clear();
		pending_requests.clear();
	}

	// Does the common pre-verification of responses before the response itself
	// is actually decoded:
	//   - check whether peer exists
	//   - check whether request was made
	//   - check whether request kinds match
	fn pre_verify_response(&self, peer: &PeerId, kind: request::Kind, raw: &UntrustedRlp) -> Result<ReqId, Error> {
		let req_id: usize = raw.val_at(0)?;
		let cur_buffer: U256 = raw.val_at(1)?;

		trace!(target: "les", "pre-verifying response from peer {}, kind={:?}", peer, kind);

		match self.pending_requests.write().remove(&req_id) {
			None => return Err(Error::UnsolicitedResponse),
			Some(requested) => {
				if requested.peer_id != *peer || requested.request.kind() != kind {
					return Err(Error::UnsolicitedResponse)
				}
			}
		}

		let peers = self.peers.read();
		match peers.get(peer) {
			Some(peer_info) => {
				let mut peer_info = peer_info.lock();
				peer_info.idle = true;

				match peer_info.remote_flow.as_mut() {
					Some(&mut (ref mut buf, ref mut flow)) => {
						let actual_buffer = ::std::cmp::min(cur_buffer, *flow.limit());
						buf.update_to(actual_buffer)
					}
					None => return Err(Error::NotServer), // this really should be impossible.
				}
				Ok(ReqId(req_id))
			}
			None => Err(Error::UnknownPeer), // probably only occurs in a race of some kind.
		}
	}

	/// Handle an LES packet using the given io context.
	/// Packet data is _untrusted_, which means that invalid data won't lead to
	/// issues.
	pub fn handle_packet(&self, io: &IoContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);

		trace!(target: "les", "Incoming packet {} from peer {}", packet_id, peer);

		// handle the packet
		let res = match packet_id {
			packet::STATUS => self.status(peer, io, rlp),
			packet::ANNOUNCE => self.announcement(peer, io, rlp),

			packet::GET_BLOCK_HEADERS => self.get_block_headers(peer, io, rlp),
			packet::BLOCK_HEADERS => self.block_headers(peer, io, rlp),

			packet::GET_BLOCK_BODIES => self.get_block_bodies(peer, io, rlp),
			packet::BLOCK_BODIES => self.block_bodies(peer, io, rlp),

			packet::GET_RECEIPTS => self.get_receipts(peer, io, rlp),
			packet::RECEIPTS => self.receipts(peer, io, rlp),

			packet::GET_PROOFS => self.get_proofs(peer, io, rlp),
			packet::PROOFS => self.proofs(peer, io, rlp),

			packet::GET_CONTRACT_CODES => self.get_contract_code(peer, io, rlp),
			packet::CONTRACT_CODES => self.contract_code(peer, io, rlp),

			packet::GET_HEADER_PROOFS => self.get_header_proofs(peer, io, rlp),
			packet::HEADER_PROOFS => self.header_proofs(peer, io, rlp),

			packet::SEND_TRANSACTIONS => self.relay_transactions(peer, io, rlp),

			other => {
				Err(Error::UnrecognizedPacket(other))
			}
		};

		if let Err(e) = res {
			punish(*peer, io, e);
		}
	}

		/// called when a peer connects.
	pub fn on_connect(&self, peer: &PeerId, io: &IoContext) {
		let proto_version = match io.protocol_version(*peer).ok_or(Error::WrongNetwork) {
			Ok(pv) => pv,
			Err(e) => { punish(*peer, io, e); return }
		};

		if PROTOCOL_VERSIONS.iter().find(|x| **x == proto_version).is_none() {
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
		let status_packet = status::write_handshake(&status, &capabilities, Some(&self.flow_params));

		self.pending_peers.write().insert(*peer, PendingPeer {
			sent_head: chain_info.best_block_hash,
			last_update: SteadyTime::now(),
		});

		io.send(*peer, packet::STATUS, status_packet);
	}

	/// called when a peer disconnects.
	pub fn on_disconnect(&self, peer: PeerId, io: &IoContext) {
		trace!(target: "les", "Peer {} disconnecting", peer);


		self.pending_peers.write().remove(&peer);
		if self.peers.write().remove(&peer).is_some() {
			let unfulfilled: Vec<_> = self.pending_requests.read()
				.iter()
				.filter(|&(_, r)| r.peer_id == peer)
				.map(|(&id, _)| ReqId(id))
				.collect();

			{
				let mut pending = self.pending_requests.write();
				for &ReqId(ref inner) in &unfulfilled {
					pending.remove(inner);
				}
			}

			for handler in &self.handlers {
				handler.on_disconnect(&Ctx {
					peer: peer,
					io: io,
					proto: self,
				}, &unfulfilled)
			}
		}
	}

	// check timeouts and punish peers.
	fn timeout_check(&self, io: &IoContext) {
		let now = SteadyTime::now();

		// handshake timeout
		{
			let mut pending = self.pending_peers.write();
			let slowpokes: Vec<_> = pending.iter()
				.filter(|&(_, ref peer)| {
					peer.last_update + Duration::milliseconds(timeout::HANDSHAKE) <= now
				})
				.map(|(&p, _)| p)
				.collect();

			for slowpoke in slowpokes {
				debug!(target: "les", "Peer {} handshake timed out", slowpoke);
				pending.remove(&slowpoke);
				io.disconnect_peer(slowpoke);
			}
		}

		// request timeouts
		{
			for r in self.pending_requests.read().values() {
				let kind_timeout = match r.request.kind() {
					request::Kind::Headers => timeout::HEADERS,
					request::Kind::Bodies => timeout::BODIES,
					request::Kind::Receipts => timeout::RECEIPTS,
					request::Kind::StateProofs => timeout::PROOFS,
					request::Kind::Codes => timeout::CONTRACT_CODES,
					request::Kind::HeaderProofs => timeout::HEADER_PROOFS,
				};

				if r.timestamp + Duration::milliseconds(kind_timeout) <= now {
					debug!(target: "les", "Request for {:?} from peer {} timed out",
						r.request.kind(), r.peer_id);

					// keep the request in the `pending` set for now so
					// on_disconnect will pass unfulfilled ReqIds to handlers.
					// in the case that a response is received after this, the
					// disconnect won't be cancelled but the ReqId won't be
					// marked as abandoned.
					io.disconnect_peer(r.peer_id);
				}
			}
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
}

impl LightProtocol {
	// Handle status message from peer.
	fn status(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		let pending = match self.pending_peers.write().remove(peer) {
			Some(pending) => pending,
			None => {
				return Err(Error::UnexpectedHandshake);
			}
		};

		let (status, capabilities, flow_params) = status::parse_handshake(data)?;

		trace!(target: "les", "Connected peer with chain head {:?}", (status.head_hash, status.head_num));

		if (status.network_id, status.genesis_hash) != (self.network_id, self.genesis_hash) {
			return Err(Error::WrongNetwork);
		}

		if Some(status.protocol_version as u8) != io.protocol_version(*peer) {
			return Err(Error::BadProtocolVersion);
		}

		let remote_flow = flow_params.map(|params| (params.create_buffer(), params));

		self.peers.write().insert(*peer, Mutex::new(Peer {
			local_buffer: self.flow_params.create_buffer(),
			status: status.clone(),
			capabilities: capabilities.clone(),
			remote_flow: remote_flow,
			sent_head: pending.sent_head,
			last_update: pending.last_update,
			idle: true,
		}));

		for handler in &self.handlers {
			handler.on_connect(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, &status, &capabilities)
		}

		Ok(())
	}

	// Handle an announcement.
	fn announcement(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		if !self.peers.read().contains_key(peer) {
			debug!(target: "les", "Ignoring announcement from unknown peer");
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

	// Handle a request for block headers.
	fn get_block_headers(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_HEADERS: usize = 512;

		let peers = self.peers.read();
		let peer = match peers.get(peer) {
			Some(peer) => peer,
			None => {
				debug!(target: "les", "Ignoring request from unknown peer");
				return Ok(())
			}
		};

		let mut peer = peer.lock();

		let req_id: u64 = data.val_at(0)?;
		let data = data.at(1)?;

		let start_block = {
			if data.at(0)?.size() == 32 {
				HashOrNumber::Hash(data.val_at(0)?)
			} else {
				HashOrNumber::Number(data.val_at(0)?)
			}
		};

		let req = request::Headers {
			start: start_block,
			max: ::std::cmp::min(MAX_HEADERS, data.val_at(1)?),
			skip: data.val_at(2)?,
			reverse: data.val_at(3)?,
		};

		let max_cost = peer.deduct_max(&self.flow_params, request::Kind::Headers, req.max)?;

		let response = self.provider.block_headers(req);
		let actual_cost = self.flow_params.compute_cost(request::Kind::Headers, response.len());
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);
		io.respond(packet::BLOCK_HEADERS, {
			let mut stream = RlpStream::new_list(3);
			stream.append(&req_id).append(&cur_buffer).begin_list(response.len());

			for header in response {
				stream.append_raw(&header.into_inner(), 1);
			}

			stream.out()
		});

		Ok(())
	}

	// Receive a response for block headers.
	fn block_headers(&self, peer: &PeerId, io: &IoContext, raw: UntrustedRlp) -> Result<(), Error> {
		let req_id = self.pre_verify_response(peer, request::Kind::Headers, &raw)?;
		let raw_headers: Vec<_> = raw.at(2)?.iter().map(|x| x.as_raw().to_owned()).collect();

		for handler in &self.handlers {
			handler.on_block_headers(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, req_id, &raw_headers);
		}

		Ok(())
	}

	// Handle a request for block bodies.
	fn get_block_bodies(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_BODIES: usize = 256;

		let peers = self.peers.read();
		let peer = match peers.get(peer) {
			Some(peer) => peer,
			None => {
				debug!(target: "les", "Ignoring request from unknown peer");
				return Ok(())
			}
		};
		let mut peer = peer.lock();

		let req_id: u64 = data.val_at(0)?;

		let req = request::Bodies {
			block_hashes: data.at(1)?.iter()
				.take(MAX_BODIES)
				.map(|x| x.as_val())
				.collect::<Result<_, _>>()?
		};

		let max_cost = peer.deduct_max(&self.flow_params, request::Kind::Bodies, req.block_hashes.len())?;

		let response = self.provider.block_bodies(req);
		let response_len = response.iter().filter(|x| x.is_some()).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::Bodies, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::BLOCK_BODIES, {
			let mut stream = RlpStream::new_list(3);
			stream.append(&req_id).append(&cur_buffer).begin_list(response.len());

			for body in response {
				match body {
					Some(body) => stream.append_raw(&body.into_inner(), 1),
					None => stream.append_empty_data(),
				};
			}

			stream.out()
		});

		Ok(())
	}

	// Receive a response for block bodies.
	fn block_bodies(&self, peer: &PeerId, io: &IoContext, raw: UntrustedRlp) -> Result<(), Error> {
		let req_id = self.pre_verify_response(peer, request::Kind::Bodies, &raw)?;
		let raw_bodies: Vec<Bytes> = raw.at(2)?.iter().map(|x| x.as_raw().to_owned()).collect();

		for handler in &self.handlers {
			handler.on_block_bodies(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, req_id, &raw_bodies);
		}

		Ok(())
	}

	// Handle a request for receipts.
	fn get_receipts(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_RECEIPTS: usize = 256;

		let peers = self.peers.read();
		let peer = match peers.get(peer) {
			Some(peer) => peer,
			None => {
				debug!(target: "les", "Ignoring request from unknown peer");
				return Ok(())
			}
		};
		let mut peer = peer.lock();

		let req_id: u64 = data.val_at(0)?;

		let req = request::Receipts {
			block_hashes: data.at(1)?.iter()
				.take(MAX_RECEIPTS)
				.map(|x| x.as_val())
				.collect::<Result<_,_>>()?
		};

		let max_cost = peer.deduct_max(&self.flow_params, request::Kind::Receipts, req.block_hashes.len())?;

		let response = self.provider.receipts(req);
		let response_len = response.iter().filter(|x| &x[..] != &::rlp::EMPTY_LIST_RLP).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::Receipts, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::RECEIPTS, {
			let mut stream = RlpStream::new_list(3);
			stream.append(&req_id).append(&cur_buffer).begin_list(response.len());

			for receipts in response {
				stream.append_raw(&receipts, 1);
			}

			stream.out()
		});

		Ok(())
	}

	// Receive a response for receipts.
	fn receipts(&self, peer: &PeerId, io: &IoContext, raw: UntrustedRlp) -> Result<(), Error> {
		let req_id = self.pre_verify_response(peer, request::Kind::Receipts, &raw)?;
		let raw_receipts: Vec<Vec<Receipt>> = raw.at(2)?
			.iter()
			.map(|x| x.as_val())
			.collect::<Result<_,_>>()?;

		for handler in &self.handlers {
			handler.on_receipts(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, req_id, &raw_receipts);
		}

		Ok(())
	}

	// Handle a request for proofs.
	fn get_proofs(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_PROOFS: usize = 128;

		let peers = self.peers.read();
		let peer = match peers.get(peer) {
			Some(peer) => peer,
			None => {
				debug!(target: "les", "Ignoring request from unknown peer");
				return Ok(())
			}
		};
		let mut peer = peer.lock();

		let req_id: u64 = data.val_at(0)?;

		let req = {
			let requests: Result<Vec<_>, Error> = data.at(1)?.iter().take(MAX_PROOFS).map(|x| {
				Ok(request::StateProof {
					block: x.val_at(0)?,
					key1: x.val_at(1)?,
					key2: if x.at(2)?.is_empty() { None } else { Some(x.val_at(2)?) },
					from_level: x.val_at(3)?,
				})
			}).collect();

			request::StateProofs {
				requests: requests?,
			}
		};

		let max_cost = peer.deduct_max(&self.flow_params, request::Kind::StateProofs, req.requests.len())?;

		let response = self.provider.proofs(req);
		let response_len = response.iter().filter(|x| &x[..] != &::rlp::EMPTY_LIST_RLP).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::StateProofs, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::PROOFS, {
			let mut stream = RlpStream::new_list(3);
			stream.append(&req_id).append(&cur_buffer).begin_list(response.len());

			for proof in response {
				stream.append_raw(&proof, 1);
			}

			stream.out()
		});

		Ok(())
	}

	// Receive a response for proofs.
	fn proofs(&self, peer: &PeerId, io: &IoContext, raw: UntrustedRlp) -> Result<(), Error> {
		let req_id = self.pre_verify_response(peer, request::Kind::StateProofs, &raw)?;

		let raw_proofs: Vec<Vec<Bytes>> = raw.at(2)?.iter()
			.map(|x| x.iter().map(|node| node.as_raw().to_owned()).collect())
			.collect();

		for handler in &self.handlers {
			handler.on_state_proofs(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, req_id, &raw_proofs);
		}

		Ok(())
	}

	// Handle a request for contract code.
	fn get_contract_code(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_CODES: usize = 256;

		let peers = self.peers.read();
		let peer = match peers.get(peer) {
			Some(peer) => peer,
			None => {
				debug!(target: "les", "Ignoring request from unknown peer");
				return Ok(())
			}
		};
		let mut peer = peer.lock();

		let req_id: u64 = data.val_at(0)?;

		let req = {
			let requests: Result<Vec<_>, Error> = data.at(1)?.iter().take(MAX_CODES).map(|x| {
				Ok(request::ContractCode {
					block_hash: x.val_at(0)?,
					account_key: x.val_at(1)?,
				})
			}).collect();

			request::ContractCodes {
				code_requests: requests?,
			}
		};

		let max_cost = peer.deduct_max(&self.flow_params, request::Kind::Codes, req.code_requests.len())?;

		let response = self.provider.contract_codes(req);
		let response_len = response.iter().filter(|x| !x.is_empty()).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::Codes, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::CONTRACT_CODES, {
			let mut stream = RlpStream::new_list(3);
			stream.append(&req_id).append(&cur_buffer).begin_list(response.len());

			for code in response {
				stream.append(&code);
			}

			stream.out()
		});

		Ok(())
	}

	// Receive a response for contract code.
	fn contract_code(&self, peer: &PeerId, io: &IoContext, raw: UntrustedRlp) -> Result<(), Error> {
		let req_id = self.pre_verify_response(peer, request::Kind::Codes, &raw)?;

		let raw_code: Vec<Bytes> = raw.at(2)?.iter()
			.map(|x| x.as_val())
			.collect::<Result<_,_>>()?;

		for handler in &self.handlers {
			handler.on_code(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, req_id, &raw_code);
		}

		Ok(())
	}

	// Handle a request for header proofs
	fn get_header_proofs(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_PROOFS: usize = 256;

		let peers = self.peers.read();
		let peer = match peers.get(peer) {
			Some(peer) => peer,
			None => {
				debug!(target: "les", "Ignoring request from unknown peer");
				return Ok(())
			}
		};
		let mut peer = peer.lock();

		let req_id: u64 = data.val_at(0)?;

		let req = {
			let requests: Result<Vec<_>, Error> = data.at(1)?.iter().take(MAX_PROOFS).map(|x| {
				Ok(request::HeaderProof {
					cht_number: x.val_at(0)?,
					block_number: x.val_at(1)?,
					from_level: x.val_at(2)?,
				})
			}).collect();

			request::HeaderProofs {
				requests: requests?,
			}
		};

		let max_cost = peer.deduct_max(&self.flow_params, request::Kind::HeaderProofs, req.requests.len())?;

		let response = self.provider.header_proofs(req);
		let response_len = response.iter().filter(|x| &x[..] != ::rlp::EMPTY_LIST_RLP).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::HeaderProofs, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::HEADER_PROOFS, {
			let mut stream = RlpStream::new_list(3);
			stream.append(&req_id).append(&cur_buffer).begin_list(response.len());

			for proof in response {
				stream.append_raw(&proof, 1);
			}

			stream.out()
		});

		Ok(())
	}

	// Receive a response for header proofs
	fn header_proofs(&self, peer: &PeerId, io: &IoContext, raw: UntrustedRlp) -> Result<(), Error> {
		fn decode_res(raw: UntrustedRlp) -> Result<(Bytes, Vec<Bytes>), ::rlp::DecoderError> {
			Ok((
				raw.val_at(0)?,
				raw.at(1)?.iter().map(|x| x.as_raw().to_owned()).collect(),
			))
		}

		let req_id = self.pre_verify_response(peer, request::Kind::HeaderProofs, &raw)?;
		let raw_proofs: Vec<_> = raw.at(2)?.iter()
			.map(decode_res)
			.collect::<Result<_,_>>()?;

		for handler in &self.handlers {
			handler.on_header_proofs(&Ctx {
				peer: *peer,
				io: io,
				proto: self,
			}, req_id, &raw_proofs);
		}

		Ok(())
	}

	// Receive a set of transactions to relay.
	fn relay_transactions(&self, peer: &PeerId, io: &IoContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_TRANSACTIONS: usize = 256;

		let txs: Vec<_> = data.iter()
			.take(MAX_TRANSACTIONS)
			.map(|x| x.as_val::<SignedTransaction>())
			.collect::<Result<_,_>>()?;

		debug!(target: "les", "Received {} transactions to relay from peer {}", txs.len(), peer);

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
			debug!(target: "les", "Disconnecting peer {}: {}", peer, e);
			io.disconnect_peer(peer)
		}
		Punishment::Disable => {
			debug!(target: "les", "Disabling peer {}: {}", peer, e);
			io.disable_peer(peer)
		}
	}
}

impl NetworkProtocolHandler for LightProtocol {
	fn initialize(&self, io: &NetworkContext) {
		io.register_timer(TIMEOUT, TIMEOUT_INTERVAL_MS)
			.expect("Error registering sync timer.");
		io.register_timer(TICK_TIMEOUT, TICK_TIMEOUT_INTERVAL_MS)
			.expect("Error registering sync timer.");
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		self.handle_packet(io, peer, packet_id, data);
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_connect(peer, io);
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_disconnect(*peer, io);
	}

	fn timeout(&self, io: &NetworkContext, timer: TimerToken) {
		match timer {
			TIMEOUT => self.timeout_check(io),
			TICK_TIMEOUT => self.tick_handlers(io),
			_ => warn!(target: "les", "received timeout on unknown token {}", timer),
		}
	}
}

// Helper for encoding the request to RLP with the given ID.
fn encode_request(req: &Request, req_id: usize) -> Vec<u8> {
	match *req {
		Request::Headers(ref headers) => {
			let mut stream = RlpStream::new_list(2);
			stream.append(&req_id).begin_list(4);

			match headers.start {
				HashOrNumber::Hash(ref hash) => stream.append(hash),
				HashOrNumber::Number(ref num) => stream.append(num),
			};

			stream
				.append(&headers.max)
				.append(&headers.skip)
				.append(&headers.reverse);

			stream.out()
		}
		Request::Bodies(ref request) => {
			let mut stream = RlpStream::new_list(2);
			stream.append(&req_id).begin_list(request.block_hashes.len());

			for hash in &request.block_hashes {
				stream.append(hash);
			}

			stream.out()
		}
		Request::Receipts(ref request) => {
			let mut stream = RlpStream::new_list(2);
			stream.append(&req_id).begin_list(request.block_hashes.len());

			for hash in &request.block_hashes {
				stream.append(hash);
			}

			stream.out()
		}
		Request::StateProofs(ref request) => {
			let mut stream = RlpStream::new_list(2);
			stream.append(&req_id).begin_list(request.requests.len());

			for proof_req in &request.requests {
				stream.begin_list(4)
					.append(&proof_req.block)
					.append(&proof_req.key1);

				match proof_req.key2 {
					Some(ref key2) => stream.append(key2),
					None => stream.append_empty_data(),
				};

				stream.append(&proof_req.from_level);
			}

			stream.out()
		}
		Request::Codes(ref request) => {
			let mut stream = RlpStream::new_list(2);
			stream.append(&req_id).begin_list(request.code_requests.len());

			for code_req in &request.code_requests {
				stream.begin_list(2)
					.append(&code_req.block_hash)
					.append(&code_req.account_key);
			}

			stream.out()
		}
		Request::HeaderProofs(ref request) => {
			let mut stream = RlpStream::new_list(2);
			stream.append(&req_id).begin_list(request.requests.len());

			for proof_req in &request.requests {
				stream.begin_list(3)
					.append(&proof_req.cht_number)
					.append(&proof_req.block_number)
					.append(&proof_req.from_level);
			}

			stream.out()
		}
	}
}
