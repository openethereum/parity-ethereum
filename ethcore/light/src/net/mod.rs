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

//! LES Protocol Version 1 implementation.
//!
//! This uses a "Provider" to answer requests.
//! See https://github.com/ethcore/parity/wiki/Light-Ethereum-Subprotocol-(LES)

use ethcore::transaction::SignedTransaction;
use ethcore::receipt::Receipt;

use io::TimerToken;
use network::{NetworkProtocolHandler, NetworkContext, NetworkError, PeerId};
use rlp::{RlpStream, Stream, UntrustedRlp, View};
use util::hash::H256;
use util::{Bytes, Mutex, RwLock, U256};
use time::SteadyTime;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use provider::Provider;
use request::{self, Request};

use self::buffer_flow::{Buffer, FlowParams};
use self::error::{Error, Punishment};

mod buffer_flow;
mod error;
mod status;

pub use self::status::{Status, Capabilities, Announcement, NetworkId};

const TIMEOUT: TimerToken = 0;
const TIMEOUT_INTERVAL_MS: u64 = 1000;

// LPV1
const PROTOCOL_VERSION: u32 = 1;

// TODO [rob] make configurable.
const PROTOCOL_ID: [u8; 3] = *b"les";

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

/// A request id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReqId(usize);

// A pending peer: one we've sent our status to but
// may not have received one for.
struct PendingPeer {
	sent_head: H256,
}

// data about each peer.
struct Peer {
	local_buffer: Buffer, // their buffer relative to us
	remote_buffer: Buffer, // our buffer relative to them
	current_asking: HashSet<usize>, // pending request ids.
	status: Status,
	capabilities: Capabilities,
	remote_flow: FlowParams,
	sent_head: H256, // last head we've given them.
}

impl Peer {
	// check the maximum cost of a request, returning an error if there's
	// not enough buffer left.
	// returns the calculated maximum cost.
	fn deduct_max(&mut self, flow_params: &FlowParams, kind: request::Kind, max: usize) -> Result<U256, Error> {
		flow_params.recharge(&mut self.local_buffer);

		let max_cost = flow_params.compute_cost(kind, max);
		try!(self.local_buffer.deduct_cost(max_cost));
		Ok(max_cost)
	}

	// refund buffer for a request. returns new buffer amount.
	fn refund(&mut self, flow_params: &FlowParams, amount: U256) -> U256 {
		flow_params.refund(&mut self.local_buffer, amount);

		self.local_buffer.current()
	}

	// recharge remote buffer with remote flow params.
	fn recharge_remote(&mut self) {
		let flow = &mut self.remote_flow;
		flow.recharge(&mut self.remote_buffer);
	}
}

/// Context for a network event.
pub struct EventContext<'a> {
	/// Protocol implementation.
	pub proto: &'a LightProtocol,
	/// Network context to enable immediate response to
	/// events.
	pub io: &'a NetworkContext<'a>,
	/// Relevant peer for event.
	pub peer: PeerId,
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
	fn on_connect(&self, _ctx: EventContext, _status: &Status, _capabilities: &Capabilities) { }
	/// Called when a peer disconnects, with a list of unfulfilled request IDs as
	/// of yet.
	fn on_disconnect(&self, _ctx: EventContext, _unfulfilled: &[ReqId]) { }
	/// Called when a peer makes an announcement.
	fn on_announcement(&self, _ctx: EventContext, _announcement: &Announcement) { }
	/// Called when a peer requests relay of some transactions.
	fn on_transactions(&self, _ctx: EventContext, _relay: &[SignedTransaction]) { }
	/// Called when a peer responds with block bodies.
	fn on_block_bodies(&self, _ctx: EventContext, _req_id: ReqId, _bodies: &[Bytes]) { }
	/// Called when a peer responds with block headers.
	fn on_block_headers(&self, _ctx: EventContext, _req_id: ReqId, _headers: &[Bytes]) { }
	/// Called when a peer responds with block receipts.
	fn on_receipts(&self, _ctx: EventContext, _req_id: ReqId, _receipts: &[Vec<Receipt>]) { }
	/// Called when a peer responds with state proofs. Each proof is a series of trie
	/// nodes in ascending order by distance from the root.
	fn on_state_proofs(&self, _ctx: EventContext, _req_id: ReqId, _proofs: &[Vec<Bytes>]) { }
	/// Called when a peer responds with contract code.
	fn on_code(&self, _ctx: EventContext, _req_id: ReqId, _codes: &[Bytes]) { }
	/// Called when a peer responds with header proofs. Each proof is a series of trie
	/// nodes is ascending order by distance from the root.
	fn on_header_proofs(&self, _ctx: EventContext, _req_id: ReqId, _proofs: &[Vec<Bytes>]) { }
}

// a request and the time it was made.
struct Requested {
	request: Request,
	timestamp: SteadyTime,
}

/// Protocol parameters.
pub struct Params {
	/// Genesis hash.
	pub genesis_hash: H256,
	/// Network id.
	pub network_id: NetworkId,
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
	provider: Box<Provider>,
	genesis_hash: H256,
	network_id: NetworkId,
	pending_peers: RwLock<HashMap<PeerId, PendingPeer>>,
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>,
	pending_requests: RwLock<HashMap<usize, Requested>>,
	capabilities: RwLock<Capabilities>,
	flow_params: FlowParams, // assumed static and same for every peer.
	handlers: Vec<Box<Handler>>,
	req_id: AtomicUsize,
}

impl LightProtocol {
	/// Create a new instance of the protocol manager.
	pub fn new(provider: Box<Provider>, params: Params) -> Self {
		LightProtocol {
			provider: provider,
			genesis_hash: params.genesis_hash,
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
	/// which a peer would be able to serve.
	pub fn max_requests(&self, peer: PeerId, kind: request::Kind) -> Option<usize> {
		self.peers.read().get(&peer).map(|peer| {
			let mut peer = peer.lock();
			peer.recharge_remote();
			peer.remote_flow.max_amount(&peer.remote_buffer, kind)
		})
	}

	/// Make a request to a peer. 
	///
	/// Fails on: nonexistent peer, network error,
	/// insufficient buffer. Does not check capabilities before sending.
	/// On success, returns a request id which can later be coordinated 
	/// with an event.
	pub fn request_from(&self, io: &NetworkContext, peer_id: &PeerId, request: Request) -> Result<ReqId, Error> {
		let peers = self.peers.read();
		let peer = try!(peers.get(peer_id).ok_or_else(|| Error::UnknownPeer));
		let mut peer = peer.lock();

		peer.recharge_remote();

		let max = peer.remote_flow.compute_cost(request.kind(), request.amount());
		try!(peer.remote_buffer.deduct_cost(max));

		let req_id = self.req_id.fetch_add(1, Ordering::SeqCst);
		let packet_data = encode_request(&request, req_id);

		let packet_id = match request.kind() {
			request::Kind::Headers => packet::GET_BLOCK_HEADERS,
			request::Kind::Bodies => packet::GET_BLOCK_BODIES,
			request::Kind::Receipts => packet::GET_RECEIPTS,
			request::Kind::StateProofs => packet::GET_PROOFS,
			request::Kind::Codes => packet::GET_CONTRACT_CODES,
			request::Kind::HeaderProofs => packet::GET_HEADER_PROOFS,
		};

		try!(io.send(*peer_id, packet_id, packet_data));

		peer.current_asking.insert(req_id);
		self.pending_requests.write().insert(req_id, Requested {
			request: request,
			timestamp: SteadyTime::now(),
		});

		Ok(ReqId(req_id))
	}

	/// Make an announcement of new chain head and capabilities to all peers.
	/// The announcement is expected to be valid.
	pub fn make_announcement(&self, io: &NetworkContext, mut announcement: Announcement) {
		let mut reorgs_map = HashMap::new();

		// update stored capabilities
		self.capabilities.write().update_from(&announcement);

		// calculate reorg info and send packets
		for (peer_id, peer_info) in self.peers.read().iter() {
			let mut peer_info = peer_info.lock();
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

			if let Err(e) = io.send(*peer_id, packet::ANNOUNCE, status::write_announcement(&announcement)) {
				debug!(target: "les", "Error sending to peer {}: {}", peer_id, e);
			}
		}
	}

	/// Add an event handler.
	/// Ownership will be transferred to the protocol structure,
	/// and the handler will be kept alive as long as it is.
	/// These are intended to be added when the protocol structure 
	/// is initialized as a means of customizing its behavior.
	pub fn add_handler(&mut self, handler: Box<Handler>) {
		self.handlers.push(handler);
	}
}

impl LightProtocol {
	// called when a peer connects.
	fn on_connect(&self, peer: &PeerId, io: &NetworkContext) {
		let peer = *peer;

		match self.send_status(peer, io) {
			Ok(pending_peer) => {
				self.pending_peers.write().insert(peer, pending_peer);
			}
			Err(e) => {
				trace!(target: "les", "Error while sending status: {}", e);
				io.disconnect_peer(peer);
			}
		}
	}

	// called when a peer disconnects.
	fn on_disconnect(&self, peer: PeerId, io: &NetworkContext) {
		self.pending_peers.write().remove(&peer);
		if let Some(peer_info) = self.peers.write().remove(&peer) {
			let unfulfilled: Vec<_> = peer_info.into_inner().current_asking.into_iter().map(ReqId).collect();
			{
				let mut pending = self.pending_requests.write();
				for &ReqId(ref inner) in &unfulfilled {
					pending.remove(inner);
				}
			}

			for handler in &self.handlers {
				handler.on_disconnect(EventContext {
					peer: peer,
					io: io,
					proto: self,
				}, &unfulfilled)
			}	
		}
	}

	// send status to a peer.
	fn send_status(&self, peer: PeerId, io: &NetworkContext) -> Result<PendingPeer, NetworkError> {
		let chain_info = self.provider.chain_info();

		// TODO: could update capabilities here.

		let status = Status {
			head_td: chain_info.total_difficulty,
			head_hash: chain_info.best_block_hash,
			head_num: chain_info.best_block_number,
			genesis_hash: chain_info.genesis_hash,
			protocol_version: PROTOCOL_VERSION,
			network_id: self.network_id,
			last_head: None,
		};

		let capabilities = self.capabilities.read().clone();
		let status_packet = status::write_handshake(&status, &capabilities, &self.flow_params);

		try!(io.send(peer, packet::STATUS, status_packet));

		Ok(PendingPeer {
			sent_head: chain_info.best_block_hash,
		})
	}

	// Handle status message from peer.
	fn status(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
		let pending = match self.pending_peers.write().remove(peer) {
			Some(pending) => pending,
			None => {
				return Err(Error::UnexpectedHandshake);
			}
		};

		let (status, capabilities, flow_params) = try!(status::parse_handshake(data));

		trace!(target: "les", "Connected peer with chain head {:?}", (status.head_hash, status.head_num));

		if (status.network_id, status.genesis_hash) != (self.network_id, self.genesis_hash) {
			return Err(Error::WrongNetwork);
		}

		self.peers.write().insert(*peer, Mutex::new(Peer {
			local_buffer: self.flow_params.create_buffer(),
			remote_buffer: flow_params.create_buffer(),
			current_asking: HashSet::new(),
			status: status.clone(),
			capabilities: capabilities.clone(),
			remote_flow: flow_params,
			sent_head: pending.sent_head,
		}));

		for handler in &self.handlers {
			handler.on_connect(EventContext {
				peer: *peer,
				io: io,	
				proto: self,
			}, &status, &capabilities)
		}

		Ok(())
	}

	// Handle an announcement.
	fn announcement(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
		if !self.peers.read().contains_key(peer) {
			debug!(target: "les", "Ignoring announcement from unknown peer");
			return Ok(())
		}

		let announcement = try!(status::parse_announcement(data));

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
			handler.on_announcement(EventContext {
				peer: *peer,
				io: io,
				proto: self,
			}, &announcement);
		}

		Ok(())
	}

	// Handle a request for block headers.
	fn get_block_headers(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
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

		let req_id: u64 = try!(data.val_at(0));

		let block = {
			let rlp = try!(data.at(1));
			(try!(rlp.val_at(0)), try!(rlp.val_at(1)))
		};

		let req = request::Headers {
			block_num: block.0,
			block_hash: block.1,
			max: ::std::cmp::min(MAX_HEADERS, try!(data.val_at(2))),
			skip: try!(data.val_at(3)),
			reverse: try!(data.val_at(4)),
		};

		let max_cost = try!(peer.deduct_max(&self.flow_params, request::Kind::Headers, req.max));

		let response = self.provider.block_headers(req);
		let actual_cost = self.flow_params.compute_cost(request::Kind::Headers, response.len());
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);
		io.respond(packet::BLOCK_HEADERS, {
			let mut stream = RlpStream::new_list(response.len() + 2);
			stream.append(&req_id).append(&cur_buffer);

			for header in response {
				stream.append_raw(&header, 1);
			}

			stream.out()
		}).map_err(Into::into)
	}

	// Receive a response for block headers.
	fn block_headers(&self, _: &PeerId, _: &NetworkContext, _: UntrustedRlp) -> Result<(), Error> {
		unimplemented!()
	}

	// Handle a request for block bodies.
	fn get_block_bodies(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
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

		let req_id: u64 = try!(data.val_at(0));

		let req = request::Bodies {
			block_hashes: try!(data.iter().skip(1).take(MAX_BODIES).map(|x| x.as_val()).collect())
		};

		let max_cost = try!(peer.deduct_max(&self.flow_params, request::Kind::Bodies, req.block_hashes.len()));

		let response = self.provider.block_bodies(req);
		let response_len = response.iter().filter(|x| &x[..] != &::rlp::EMPTY_LIST_RLP).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::Bodies, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::BLOCK_BODIES, {
			let mut stream = RlpStream::new_list(response.len() + 2);
			stream.append(&req_id).append(&cur_buffer);

			for body in response {
				stream.append_raw(&body, 1);
			}

			stream.out()
		}).map_err(Into::into)
	}

	// Receive a response for block bodies.
	fn block_bodies(&self, _: &PeerId, _: &NetworkContext, _: UntrustedRlp) -> Result<(), Error> {
		unimplemented!()
	}

	// Handle a request for receipts.
	fn get_receipts(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
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

		let req_id: u64 = try!(data.val_at(0));

		let req = request::Receipts {
			block_hashes: try!(data.iter().skip(1).take(MAX_RECEIPTS).map(|x| x.as_val()).collect())
		};

		let max_cost = try!(peer.deduct_max(&self.flow_params, request::Kind::Receipts, req.block_hashes.len()));

		let response = self.provider.receipts(req);
		let response_len = response.iter().filter(|x| &x[..] != &::rlp::EMPTY_LIST_RLP).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::Receipts, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::RECEIPTS, {
			let mut stream = RlpStream::new_list(response.len() + 2);
			stream.append(&req_id).append(&cur_buffer);

			for receipts in response {
				stream.append_raw(&receipts, 1);
			}

			stream.out()
		}).map_err(Into::into)
	}

	// Receive a response for receipts.
	fn receipts(&self, _: &PeerId, _: &NetworkContext, _: UntrustedRlp) -> Result<(), Error> {
		unimplemented!()
	}

	// Handle a request for proofs.
	fn get_proofs(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
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

		let req_id: u64 = try!(data.val_at(0));

		let req = {
			let requests: Result<Vec<_>, Error> = data.iter().skip(1).take(MAX_PROOFS).map(|x| {
				Ok(request::StateProof {
					block: try!(x.val_at(0)),
					key1: try!(x.val_at(1)),
					key2: if try!(x.at(2)).is_empty() { None } else { Some(try!(x.val_at(2))) },
					from_level: try!(x.val_at(3)),
				})
			}).collect();

			request::StateProofs {
				requests: try!(requests),
			}
		};

		let max_cost = try!(peer.deduct_max(&self.flow_params, request::Kind::StateProofs, req.requests.len()));

		let response = self.provider.proofs(req);
		let response_len = response.iter().filter(|x| &x[..] != &::rlp::EMPTY_LIST_RLP).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::StateProofs, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::PROOFS, {
			let mut stream = RlpStream::new_list(response.len() + 2);
			stream.append(&req_id).append(&cur_buffer);

			for proof in response {
				stream.append_raw(&proof, 1);
			}

			stream.out()
		}).map_err(Into::into)
	}

	// Receive a response for proofs.
	fn proofs(&self, _: &PeerId, _: &NetworkContext, _: UntrustedRlp) -> Result<(), Error> {
		unimplemented!()
	}

	// Handle a request for contract code.
	fn get_contract_code(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
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

		let req_id: u64 = try!(data.val_at(0));

		let req = {
			let requests: Result<Vec<_>, Error> = data.iter().skip(1).take(MAX_CODES).map(|x| {
				Ok(request::ContractCode {
					block_hash: try!(x.val_at(0)),
					account_key: try!(x.val_at(1)),
				})
			}).collect();

			request::ContractCodes {
				code_requests: try!(requests),
			}
		};

		let max_cost = try!(peer.deduct_max(&self.flow_params, request::Kind::Codes, req.code_requests.len()));

		let response = self.provider.contract_code(req);
		let response_len = response.iter().filter(|x| !x.is_empty()).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::Codes, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::CONTRACT_CODES, {
			let mut stream = RlpStream::new_list(response.len() + 2);
			stream.append(&req_id).append(&cur_buffer);

			for code in response {
				stream.append_raw(&code, 1);
			}

			stream.out()
		}).map_err(Into::into)
	}

	// Receive a response for contract code.
	fn contract_code(&self, _: &PeerId, _: &NetworkContext, _: UntrustedRlp) -> Result<(), Error> {
		unimplemented!()
	}

	// Handle a request for header proofs
	fn get_header_proofs(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
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

		let req_id: u64 = try!(data.val_at(0));

		let req = {
			let requests: Result<Vec<_>, Error> = data.iter().skip(1).take(MAX_PROOFS).map(|x| {
				Ok(request::HeaderProof {
					cht_number: try!(x.val_at(0)),
					block_number: try!(x.val_at(1)),
					from_level: try!(x.val_at(2)),
				})
			}).collect();

			request::HeaderProofs {
				requests: try!(requests),
			}
		};

		let max_cost = try!(peer.deduct_max(&self.flow_params, request::Kind::HeaderProofs, req.requests.len()));

		let response = self.provider.header_proofs(req);
		let response_len = response.iter().filter(|x| &x[..] != ::rlp::EMPTY_LIST_RLP).count();
		let actual_cost = self.flow_params.compute_cost(request::Kind::HeaderProofs, response_len);
		assert!(max_cost >= actual_cost, "Actual cost exceeded maximum computed cost.");

		let cur_buffer = peer.refund(&self.flow_params, max_cost - actual_cost);

		io.respond(packet::HEADER_PROOFS, {
			let mut stream = RlpStream::new_list(response.len() + 2);
			stream.append(&req_id).append(&cur_buffer);

			for proof in response {
				stream.append_raw(&proof, 1);
			}

			stream.out()
		}).map_err(Into::into)
	}

	// Receive a response for header proofs
	fn header_proofs(&self, _: &PeerId, _: &NetworkContext, _: UntrustedRlp) -> Result<(), Error> {
		unimplemented!()
	}

	// Receive a set of transactions to relay.
	fn relay_transactions(&self, peer: &PeerId, io: &NetworkContext, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_TRANSACTIONS: usize = 256;

		let txs: Vec<_> = try!(data.iter().take(MAX_TRANSACTIONS).map(|x| x.as_val::<SignedTransaction>()).collect());

		debug!(target: "les", "Received {} transactions to relay from peer {}", txs.len(), peer);

		for handler in &self.handlers {
			handler.on_transactions(EventContext {
				peer: *peer,
				io: io,
				proto: self,	
			}, &txs);
		}

		Ok(())
	}
}

impl NetworkProtocolHandler for LightProtocol {
	fn initialize(&self, io: &NetworkContext) {
		io.register_timer(TIMEOUT, TIMEOUT_INTERVAL_MS).expect("Error registering sync timer.");
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let rlp = UntrustedRlp::new(data);

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

		// if something went wrong, figure out how much to punish the peer.
		if let Err(e) = res {
			match e.punishment() {
				Punishment::None => {}
				Punishment::Disconnect => {
					debug!(target: "les", "Disconnecting peer {}: {}", peer, e);
					io.disconnect_peer(*peer)
				}
				Punishment::Disable => {
					debug!(target: "les", "Disabling peer {}: {}", peer, e);
					io.disable_peer(*peer)
				}
			}
		}
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_connect(peer, io);
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
		self.on_disconnect(*peer, io);
	}

	fn timeout(&self, _io: &NetworkContext, timer: TimerToken) {
		match timer {
			TIMEOUT => {
				// broadcast transactions to peers.
			}
			_ => warn!(target: "les", "received timeout on unknown token {}", timer),
		}
	}
}

// Helper for encoding the request to RLP with the given ID.
fn encode_request(req: &Request, req_id: usize) -> Vec<u8> {
	match *req {
		Request::Headers(ref headers) => {
			let mut stream = RlpStream::new_list(5);
			stream
				.append(&req_id)
				.begin_list(2)
					.append(&headers.block_num)
					.append(&headers.block_hash)
				.append(&headers.max)
				.append(&headers.skip)
				.append(&headers.reverse);
			stream.out()
		}
		Request::Bodies(ref request) => {
			let mut stream = RlpStream::new_list(request.block_hashes.len() + 1);
			stream.append(&req_id);

			for hash in &request.block_hashes {
				stream.append(hash);
			}

			stream.out()
		}
		Request::Receipts(ref request) => {
			let mut stream = RlpStream::new_list(request.block_hashes.len() + 1);
			stream.append(&req_id);

			for hash in &request.block_hashes {
				stream.append(hash);
			}

			stream.out()
		}
		Request::StateProofs(ref request) => {
			let mut stream = RlpStream::new_list(request.requests.len() + 1);
			stream.append(&req_id);

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
			let mut stream = RlpStream::new_list(request.code_requests.len() + 1);
			stream.append(&req_id);

			for code_req in &request.code_requests {
				stream.begin_list(2)
					.append(&code_req.block_hash)
					.append(&code_req.account_key);
			}

			stream.out()
		}
		Request::HeaderProofs(ref request) => {
			let mut stream = RlpStream::new_list(request.requests.len() + 1);
			stream.append(&req_id);

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