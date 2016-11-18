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

use io::TimerToken;
use network::{NetworkProtocolHandler, NetworkContext, NetworkError, PeerId};
use rlp::{RlpStream, Stream, UntrustedRlp, View};
use util::hash::H256;
use util::{Mutex, RwLock, U256};

use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicUsize;

use light::provider::Provider;
use light::request::{self, Request};
use transaction::SignedTransaction;

use self::buffer_flow::{Buffer, FlowParams};
use self::error::{Error, Punishment};

mod buffer_flow;
mod error;
mod status;

pub use self::status::{Status, Capabilities, Announcement};

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

// A pending peer: one we've sent our status to but
// may not have received one for.
struct PendingPeer {
	sent_head: H256,
}

// data about each peer.
struct Peer {
	local_buffer: Mutex<Buffer>, // their buffer relative to us
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
	fn deduct_max(&self, flow_params: &FlowParams, kind: request::Kind, max: usize) -> Result<U256, Error> {
		let mut local_buffer = self.local_buffer.lock();
		flow_params.recharge(&mut local_buffer);

		let max_cost = flow_params.compute_cost(kind, max);
		try!(local_buffer.deduct_cost(max_cost));
		Ok(max_cost)
	}

	// refund buffer for a request. returns new buffer amount.
	fn refund(&self, flow_params: &FlowParams, amount: U256) -> U256 {
		let mut local_buffer = self.local_buffer.lock();
		flow_params.refund(&mut local_buffer, amount);

		local_buffer.current()
	}
}

/// An LES event handler.
pub trait Handler: Send + Sync {
	/// Called when a peer connects.
	fn on_connect(&self, _id: PeerId, _status: &Status, _capabilities: &Capabilities) { }
	/// Called when a peer disconnects
	fn on_disconnect(&self, _id: PeerId) { }
	/// Called when a peer makes an announcement.
	fn on_announcement(&self, _id: PeerId, _announcement: &Announcement) { }
	/// Called when a peer requests relay of some transactions.
	fn on_transactions(&self, _id: PeerId, _relay: &[SignedTransaction]) { }
}

/// This is an implementation of the light ethereum network protocol, abstracted
/// over a `Provider` of data and a p2p network.
///
/// This is simply designed for request-response purposes. Higher level uses
/// of the protocol, such as synchronization, will function as wrappers around
/// this system.
pub struct LightProtocol {
	provider: Box<Provider>,
	genesis_hash: H256,
	network_id: status::NetworkId,
	pending_peers: RwLock<HashMap<PeerId, PendingPeer>>,
	peers: RwLock<HashMap<PeerId, Peer>>,
	pending_requests: RwLock<HashMap<usize, Request>>,
	capabilities: RwLock<Capabilities>,
	flow_params: FlowParams, // assumed static and same for every peer.
	handlers: Vec<Box<Handler>>,
	req_id: AtomicUsize,
}

impl LightProtocol {
	/// Make an announcement of new chain head and capabilities to all peers.
	/// The announcement is expected to be valid.
	pub fn make_announcement(&self, mut announcement: Announcement, io: &NetworkContext) {
		let mut reorgs_map = HashMap::new();

		// update stored capabilities
		self.capabilities.write().update_from(&announcement);

		// calculate reorg info and send packets
		for (peer_id, peer_info) in self.peers.write().iter_mut() {
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
	/// These are intended to be added at the beginning of the 
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
	fn on_disconnect(&self, peer: PeerId) {
		// TODO: reassign all requests assigned to this peer.
		self.pending_peers.write().remove(&peer);
		if self.peers.write().remove(&peer).is_some() {
			for handler in &self.handlers {
				handler.on_disconnect(peer)
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
	fn status(&self, peer: &PeerId, data: UntrustedRlp) -> Result<(), Error> {
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

		self.peers.write().insert(*peer, Peer {
			local_buffer: Mutex::new(self.flow_params.create_buffer()),
			remote_buffer: flow_params.create_buffer(),
			current_asking: HashSet::new(),
			status: status.clone(),
			capabilities: capabilities.clone(),
			remote_flow: flow_params,
			sent_head: pending.sent_head,
		});

		for handler in &self.handlers {
			handler.on_connect(*peer, &status, &capabilities)
		}

		Ok(())
	}

	// Handle an announcement.
	fn announcement(&self, peer: &PeerId, data: UntrustedRlp) -> Result<(), Error> {
		if !self.peers.read().contains_key(peer) {
			debug!(target: "les", "Ignoring announcement from unknown peer");
			return Ok(())
		}

		let announcement = try!(status::parse_announcement(data));
		let mut peers = self.peers.write();

		let peer_info = match peers.get_mut(peer) {
			Some(info) => info,
			None => return Ok(()),
		};

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

		for handler in &self.handlers {
			handler.on_announcement(*peer, &announcement);
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
	fn relay_transactions(&self, peer: &PeerId, data: UntrustedRlp) -> Result<(), Error> {
		const MAX_TRANSACTIONS: usize = 256;

		let txs: Vec<_> = try!(data.iter().take(MAX_TRANSACTIONS).map(|x| x.as_val::<SignedTransaction>()).collect());

		debug!(target: "les", "Received {} transactions to relay from peer {}", txs.len(), peer);

		for handler in &self.handlers {
			handler.on_transactions(*peer, &txs);
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
			packet::STATUS => self.status(peer, rlp),
			packet::ANNOUNCE => self.announcement(peer, rlp),

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

			packet::SEND_TRANSACTIONS => self.relay_transactions(peer, rlp),

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

	fn disconnected(&self, _io: &NetworkContext, peer: &PeerId) {
		self.on_disconnect(*peer);
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