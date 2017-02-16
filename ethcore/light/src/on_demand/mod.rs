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

//! On-demand chain requests over LES. This is a major building block for RPCs.
//! The request service is implemented using Futures. Higher level request handlers
//! will take the raw data received here and extract meaningful results from it.

use std::collections::HashMap;
use std::sync::Arc;

use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::receipt::Receipt;

use futures::{Async, Poll, Future};
use futures::sync::oneshot::{self, Sender, Receiver};
use network::PeerId;
use rlp::{RlpStream, Stream};
use util::{Bytes, RwLock, Mutex, U256};
use util::sha3::{SHA3_NULL_RLP, SHA3_EMPTY_LIST_RLP};

use net::{Handler, Status, Capabilities, Announcement, EventContext, BasicContext, ReqId};
use cache::Cache;
use types::les_request::{self as les_request, Request as LesRequest};

pub mod request;

// relevant peer info.
struct Peer {
	status: Status,
	capabilities: Capabilities,
}

// Which portions of a CHT proof should be sent.
enum ChtProofSender {
	Both(Sender<(encoded::Header, U256)>),
	Header(Sender<encoded::Header>),
	ChainScore(Sender<U256>),
}

// Attempted request info and sender to put received value.
enum Pending {
	HeaderByNumber(request::HeaderByNumber, ChtProofSender),
	HeaderByHash(request::HeaderByHash, Sender<encoded::Header>),
	Block(request::Body, Sender<encoded::Block>),
	BlockReceipts(request::BlockReceipts, Sender<Vec<Receipt>>),
	Account(request::Account, Sender<BasicAccount>),
	Code(request::Code, Sender<Bytes>),
}

/// On demand request service. See module docs for more details.
/// Accumulates info about all peers' capabilities and dispatches
/// requests to them accordingly.
pub struct OnDemand {
	peers: RwLock<HashMap<PeerId, Peer>>,
	pending_requests: RwLock<HashMap<ReqId, Pending>>,
	cache: Arc<Mutex<Cache>>,
	orphaned_requests: RwLock<Vec<Pending>>,
}

impl OnDemand {
	/// Create a new `OnDemand` service with the given cache.
	pub fn new(cache: Arc<Mutex<Cache>>) -> Self {
		OnDemand {
			peers: RwLock::new(HashMap::new()),
			pending_requests: RwLock::new(HashMap::new()),
			cache: cache,
			orphaned_requests: RwLock::new(Vec::new()),
		}
	}

	/// Request a header by block number and CHT root hash.
	/// Returns the header.
	pub fn header_by_number(&self, ctx: &BasicContext, req: request::HeaderByNumber) -> Receiver<encoded::Header> {
		let (sender, receiver) = oneshot::channel();
		let cached = {
			let mut cache = self.cache.lock();
			cache.block_hash(&req.num()).and_then(|hash| cache.block_header(&hash))
		};

		match cached {
			Some(hdr) => sender.complete(hdr),
			None => self.dispatch_header_by_number(ctx, req, ChtProofSender::Header(sender)),
		}
		receiver
	}

	/// Request a canonical block's chain score.
	/// Returns the chain score.
	pub fn chain_score_by_number(&self, ctx: &BasicContext, req: request::HeaderByNumber) -> Receiver<U256> {
		let (sender, receiver) = oneshot::channel();
		let cached = {
			let mut cache = self.cache.lock();
			cache.block_hash(&req.num()).and_then(|hash| cache.chain_score(&hash))
		};

		match cached {
			Some(score) => sender.complete(score),
			None => self.dispatch_header_by_number(ctx, req, ChtProofSender::ChainScore(sender)),
		}

		receiver
	}

	/// Request a canonical block's chain score.
	/// Returns the header and chain score.
	pub fn header_and_score_by_number(&self, ctx: &BasicContext, req: request::HeaderByNumber) -> Receiver<(encoded::Header, U256)> {
		let (sender, receiver) = oneshot::channel();
		let cached = {
			let mut cache = self.cache.lock();
			let hash = cache.block_hash(&req.num());
			(
				hash.clone().and_then(|hash| cache.block_header(&hash)),
				hash.and_then(|hash| cache.chain_score(&hash)),
			)
		};

		match cached {
			(Some(hdr), Some(score)) => sender.complete((hdr, score)),
			_ => self.dispatch_header_by_number(ctx, req, ChtProofSender::Both(sender)),
		}

		receiver
	}

	// dispatch the request, completing the request if no peers available.
	fn dispatch_header_by_number(&self, ctx: &BasicContext, req: request::HeaderByNumber, sender: ChtProofSender) {
		let num = req.num();
		let cht_num = req.cht_num();

		let les_req = LesRequest::HeaderProofs(les_request::HeaderProofs {
			requests: vec![les_request::HeaderProof {
				cht_number: cht_num,
				block_number: num,
				from_level: 0,
			}],
		});

		let pending = Pending::HeaderByNumber(req, sender);

		// we're looking for a peer with serveHeaders who's far enough along in the
		// chain.
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_headers && peer.status.head_num >= num {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							pending,
						);
						return
					},
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		trace!(target: "on_demand", "No suitable peer for request");
		self.orphaned_requests.write().push(pending)
	}

	/// Request a header by hash. This is less accurate than by-number because we don't know
	/// where in the chain this header lies, and therefore can't find a peer who is supposed to have
	/// it as easily.
	pub fn header_by_hash(&self, ctx: &BasicContext, req: request::HeaderByHash) -> Receiver<encoded::Header> {
		let (sender, receiver) = oneshot::channel();
		match self.cache.lock().block_header(&req.0) {
			Some(hdr) => sender.complete(hdr),
			None => self.dispatch_header_by_hash(ctx, req, sender),
		}
		receiver
	}

	fn dispatch_header_by_hash(&self, ctx: &BasicContext, req: request::HeaderByHash, sender: Sender<encoded::Header>) {
		let les_req = LesRequest::Headers(les_request::Headers {
			start: req.0.into(),
			max: 1,
			skip: 0,
			reverse: false,
		});

		// all we've got is a hash, so we'll just guess at peers who might have
		// it randomly.
		let mut potential_peers = self.peers.read().iter()
			.filter(|&(_, peer)| peer.capabilities.serve_headers)
			.map(|(id, _)| *id)
			.collect::<Vec<_>>();

		let mut rng = ::rand::thread_rng();
		::rand::Rng::shuffle(&mut rng, &mut potential_peers);

		let pending = Pending::HeaderByHash(req, sender);

		for id in potential_peers {
			match ctx.request_from(id, les_req.clone()) {
				Ok(req_id) => {
					trace!(target: "on_demand", "Assigning request to peer {}", id);
					self.pending_requests.write().insert(
						req_id,
						pending,
					);
					return
				}
				Err(e) =>
					trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
			}
		}

		trace!(target: "on_demand", "No suitable peer for request");
		self.orphaned_requests.write().push(pending)
	}

	/// Request a block, given its header. Block bodies are requestable by hash only,
	/// and the header is required anyway to verify and complete the block body
	/// -- this just doesn't obscure the network query.
	pub fn block(&self, ctx: &BasicContext, req: request::Body) -> Receiver<encoded::Block> {
		let (sender, receiver) = oneshot::channel();

		// fast path for empty body.
		if req.header.transactions_root() == SHA3_NULL_RLP && req.header.uncles_hash() == SHA3_EMPTY_LIST_RLP {
			let mut stream = RlpStream::new_list(3);
			stream.append_raw(&req.header.into_inner(), 1);
			stream.begin_list(0);
			stream.begin_list(0);

			sender.complete(encoded::Block::new(stream.out()))
		} else {
			match self.cache.lock().block_body(&req.hash) {
				Some(body) => {
					let mut stream = RlpStream::new_list(3);
					stream.append_raw(&req.header.into_inner(), 1);
					stream.append_raw(&body.into_inner(), 2);

					sender.complete(encoded::Block::new(stream.out()));
				}
				None => self.dispatch_block(ctx, req, sender),
			}
		}
		receiver
	}

	fn dispatch_block(&self, ctx: &BasicContext, req: request::Body, sender: Sender<encoded::Block>) {
		let num = req.header.number();
		let les_req = LesRequest::Bodies(les_request::Bodies {
			block_hashes: vec![req.hash],
		});
		let pending = Pending::Block(req, sender);

		// we're looking for a peer with serveChainSince(num)
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_chain_since.as_ref().map_or(false, |x| *x >= num) {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							pending,
						);
						return
					}
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		trace!(target: "on_demand", "No suitable peer for request");
		self.orphaned_requests.write().push(pending)
	}

	/// Request the receipts for a block. The header serves two purposes:
	/// provide the block hash to fetch receipts for, and for verification of the receipts root.
	pub fn block_receipts(&self, ctx: &BasicContext, req: request::BlockReceipts) -> Receiver<Vec<Receipt>> {
		let (sender, receiver) = oneshot::channel();

		// fast path for empty receipts.
		if req.0.receipts_root() == SHA3_NULL_RLP {
			sender.complete(Vec::new())
		} else {
			match self.cache.lock().block_receipts(&req.0.hash()) {
				Some(receipts) => sender.complete(receipts),
				None => self.dispatch_block_receipts(ctx, req, sender),
			}
		}

		receiver
	}

	fn dispatch_block_receipts(&self, ctx: &BasicContext, req: request::BlockReceipts, sender: Sender<Vec<Receipt>>) {
		let num = req.0.number();
		let les_req = LesRequest::Receipts(les_request::Receipts {
			block_hashes: vec![req.0.hash()],
		});
		let pending = Pending::BlockReceipts(req, sender);

		// we're looking for a peer with serveChainSince(num)
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_chain_since.as_ref().map_or(false, |x| *x >= num) {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							pending,
						);
						return
					}
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		trace!(target: "on_demand", "No suitable peer for request");
		self.orphaned_requests.write().push(pending)
	}

	/// Request an account by address and block header -- which gives a hash to query and a state root
	/// to verify against.
	pub fn account(&self, ctx: &BasicContext, req: request::Account) -> Receiver<BasicAccount> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch_account(ctx, req, sender);
		receiver
	}

	fn dispatch_account(&self, ctx: &BasicContext, req: request::Account, sender: Sender<BasicAccount>) {
		let num = req.header.number();
		let les_req = LesRequest::StateProofs(les_request::StateProofs {
			requests: vec![les_request::StateProof {
				block: req.header.hash(),
				key1: ::util::Hashable::sha3(&req.address),
				key2: None,
				from_level: 0,
			}],
		});
		let pending = Pending::Account(req, sender);

		// we're looking for a peer with serveStateSince(num)
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_state_since.as_ref().map_or(false, |x| *x >= num) {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							pending,
						);
						return
					}
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		trace!(target: "on_demand", "No suitable peer for request");
		self.orphaned_requests.write().push(pending)
	}

	/// Request code by address, known code hash, and block header.
	pub fn code(&self, ctx: &BasicContext, req: request::Code) -> Receiver<Bytes> {
		let (sender, receiver) = oneshot::channel();

		// fast path for no code.
		if req.code_hash == ::util::sha3::SHA3_EMPTY {
			sender.complete(Vec::new())
		} else {
			self.dispatch_code(ctx, req, sender);
		}

		receiver
	}

	fn dispatch_code(&self, ctx: &BasicContext, req: request::Code, sender: Sender<Bytes>) {
		let num = req.block_id.1;
		let les_req = LesRequest::Codes(les_request::ContractCodes {
			code_requests: vec![les_request::ContractCode {
				block_hash: req.block_id.0,
				account_key: ::util::Hashable::sha3(&req.address),
			}]
		});
		let pending = Pending::Code(req, sender);

		// we're looking for a peer with serveStateSince(num)
		for (id, peer) in self.peers.read().iter() {
			if peer.capabilities.serve_state_since.as_ref().map_or(false, |x| *x >= num) {
				match ctx.request_from(*id, les_req.clone()) {
					Ok(req_id) => {
						trace!(target: "on_demand", "Assigning request to peer {}", id);
						self.pending_requests.write().insert(
							req_id,
							pending
						);
						return
					}
					Err(e) =>
						trace!(target: "on_demand", "Failed to make request of peer {}: {:?}", id, e),
				}
			}
		}

		trace!(target: "on_demand", "No suitable peer for request");
		self.orphaned_requests.write().push(pending)
	}

	// dispatch orphaned requests, and discard those for which the corresponding
	// receiver has been dropped.
	fn dispatch_orphaned(&self, ctx: &BasicContext) {
		// wrapper future for calling `poll_cancel` on our `Senders` to preserve
		// the invariant that it's always within a task.
		struct CheckHangup<'a, T: 'a>(&'a mut Sender<T>);

		impl<'a, T: 'a> Future for CheckHangup<'a, T> {
			type Item = bool;
			type Error = ();

			fn poll(&mut self) -> Poll<bool, ()> {
				Ok(Async::Ready(match self.0.poll_cancel() {
					Ok(Async::NotReady) => false, // hasn't hung up.
					_ => true, // has hung up.
				}))
			}
		}

		// check whether a sender's hung up (using `wait` to preserve the task invariant)
		// returns true if has hung up, false otherwise.
		fn check_hangup<T>(send: &mut Sender<T>) -> bool {
			CheckHangup(send).wait().expect("CheckHangup always returns ok; qed")
		}

		if self.orphaned_requests.read().is_empty() { return }

		let to_dispatch = ::std::mem::replace(&mut *self.orphaned_requests.write(), Vec::new());

		for orphaned in to_dispatch {
			match orphaned {
				Pending::HeaderByNumber(req, mut sender) => {
					let hangup = match sender {
						ChtProofSender::Both(ref mut s) => check_hangup(s),
						ChtProofSender::Header(ref mut s) => check_hangup(s),
						ChtProofSender::ChainScore(ref mut s) => check_hangup(s),
					};

					if !hangup { self.dispatch_header_by_number(ctx, req, sender) }
				}
				Pending::HeaderByHash(req, mut sender) =>
					if !check_hangup(&mut sender) {	self.dispatch_header_by_hash(ctx, req, sender) },
				Pending::Block(req, mut sender) =>
					if !check_hangup(&mut sender) { self.dispatch_block(ctx, req, sender) },
				Pending::BlockReceipts(req, mut sender) =>
					if !check_hangup(&mut sender) { self.dispatch_block_receipts(ctx, req, sender) },
				Pending::Account(req, mut sender) =>
					if !check_hangup(&mut sender) { self.dispatch_account(ctx, req, sender) },
				Pending::Code(req, mut sender) =>
					if !check_hangup(&mut sender) { self.dispatch_code(ctx, req, sender) },
			}
		}
	}
}

impl Handler for OnDemand {
	fn on_connect(&self, ctx: &EventContext, status: &Status, capabilities: &Capabilities) {
		self.peers.write().insert(ctx.peer(), Peer { status: status.clone(), capabilities: capabilities.clone() });
		self.dispatch_orphaned(ctx.as_basic());
	}

	fn on_disconnect(&self, ctx: &EventContext, unfulfilled: &[ReqId]) {
		self.peers.write().remove(&ctx.peer());
		let ctx = ctx.as_basic();

		{
			let mut orphaned = self.orphaned_requests.write();
			for unfulfilled in unfulfilled {
				if let Some(pending) = self.pending_requests.write().remove(unfulfilled) {
					trace!(target: "on_demand", "Attempting to reassign dropped request");
					orphaned.push(pending);
				}
			}
		}

		self.dispatch_orphaned(ctx);
	}

	fn on_announcement(&self, ctx: &EventContext, announcement: &Announcement) {
		let mut peers = self.peers.write();
		if let Some(ref mut peer) = peers.get_mut(&ctx.peer()) {
			peer.status.update_from(&announcement);
			peer.capabilities.update_from(&announcement);
		}

		self.dispatch_orphaned(ctx.as_basic());
	}

	fn on_header_proofs(&self, ctx: &EventContext, req_id: ReqId, proofs: &[(Bytes, Vec<Bytes>)]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::HeaderByNumber(req, sender) => {
				if let Some(&(ref header, ref proof)) = proofs.get(0) {
					match req.check_response(header, proof) {
						Ok((header, score)) => {
							let mut cache = self.cache.lock();
							let hash = header.hash();
							cache.insert_block_header(hash, header.clone());
							cache.insert_block_hash(header.number(), hash);
							cache.insert_chain_score(hash, score);

							match sender {
								ChtProofSender::Both(sender) => sender.complete((header, score)),
								ChtProofSender::Header(sender) => sender.complete(header),
								ChtProofSender::ChainScore(sender) => sender.complete(score),
							}

							return
						}
						Err(e) => {
							warn!("Error handling response for header request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_header_by_number(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only header by number request fetches header proofs; qed"),
		}
	}

	fn on_block_headers(&self, ctx: &EventContext, req_id: ReqId, headers: &[Bytes]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::HeaderByHash(req, sender) => {
				if let Some(ref header) = headers.get(0) {
					match req.check_response(header) {
						Ok(header) => {
							self.cache.lock().insert_block_header(req.0, header.clone());
							sender.complete(header);
							return
						}
						Err(e) => {
							warn!("Error handling response for header request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_header_by_hash(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only header by hash request fetches headers; qed"),
		}
	}

	fn on_block_bodies(&self, ctx: &EventContext, req_id: ReqId, bodies: &[Bytes]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::Block(req, sender) => {
				if let Some(ref body) = bodies.get(0) {
					match req.check_response(body) {
						Ok(block) => {
							let body = encoded::Body::new(body.to_vec());
							self.cache.lock().insert_block_body(req.hash, body);
							sender.complete(block);
							return
						}
						Err(e) => {
							warn!("Error handling response for block request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_block(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only block request fetches bodies; qed"),
		}
	}

	fn on_receipts(&self, ctx: &EventContext, req_id: ReqId, receipts: &[Vec<Receipt>]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::BlockReceipts(req, sender) => {
				if let Some(ref receipts) = receipts.get(0) {
					match req.check_response(receipts) {
						Ok(receipts) => {
							let hash = req.0.hash();
							self.cache.lock().insert_block_receipts(hash, receipts.clone());
							sender.complete(receipts);
							return
						}
						Err(e) => {
							warn!("Error handling response for receipts request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_block_receipts(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only receipts request fetches receipts; qed"),
		}
	}

	fn on_state_proofs(&self, ctx: &EventContext, req_id: ReqId, proofs: &[Vec<Bytes>]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::Account(req, sender) => {
				if let Some(ref proof) = proofs.get(0) {
					match req.check_response(proof) {
						Ok(proof) => {
							sender.complete(proof);
							return
						}
						Err(e) => {
							warn!("Error handling response for state request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}
				}

				self.dispatch_account(ctx.as_basic(), req, sender);
			}
			_ => panic!("Only account request fetches state proof; qed"),
		}
	}

	fn on_code(&self, ctx: &EventContext, req_id: ReqId, codes: &[Bytes]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		match req {
			Pending::Code(req, sender) => {
				if let Some(code) = codes.get(0) {
					match req.check_response(code.as_slice()) {
						Ok(()) => {
							sender.complete(code.clone());
							return
						}
						Err(e) => {
							warn!("Error handling response for code request: {:?}", e);
							ctx.disable_peer(peer);
						}
					}

					self.dispatch_code(ctx.as_basic(), req, sender);
				}
			}
			_ => panic!("Only code request fetches code; qed"),
		}
	}

	fn tick(&self, ctx: &BasicContext) {
		self.dispatch_orphaned(ctx)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::sync::Arc;

	use cache::Cache;
	use net::{Announcement, BasicContext, ReqId, Error as LesError};
	use request::{Request as LesRequest, Kind as LesRequestKind};

	use network::{PeerId, NodeId};
	use time::Duration;
	use util::{H256, Mutex};

	struct FakeContext;

	impl BasicContext for FakeContext {
		fn persistent_peer_id(&self, _: PeerId) -> Option<NodeId> { None }
		fn request_from(&self, _: PeerId, _: LesRequest) -> Result<ReqId, LesError> {
			unimplemented!()
		}
		fn make_announcement(&self, _: Announcement) { }
		fn max_requests(&self, _: PeerId, _: LesRequestKind) -> usize { 0 }
		fn disconnect_peer(&self, _: PeerId) { }
		fn disable_peer(&self, _: PeerId) { }
	}

	#[test]
	fn detects_hangup() {
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));
		let on_demand = OnDemand::new(cache);
		let result = on_demand.header_by_hash(&FakeContext, request::HeaderByHash(H256::default()));

		assert!(on_demand.orphaned_requests.read().len() == 1);
		drop(result);

		on_demand.dispatch_orphaned(&FakeContext);
		assert!(on_demand.orphaned_requests.read().is_empty());
	}
}
