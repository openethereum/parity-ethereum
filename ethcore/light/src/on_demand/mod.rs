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
use ethcore::state::ProvedExecution;
use ethcore::executed::{Executed, ExecutionError};

use futures::{Async, Poll, Future};
use futures::sync::oneshot::{self, Sender, Receiver};
use network::PeerId;
use rlp::RlpStream;
use util::{Bytes, RwLock, Mutex, U256, H256};
use util::sha3::{SHA3_NULL_RLP, SHA3_EMPTY_LIST_RLP};

use net::{Handler, Status, Capabilities, Announcement, EventContext, BasicContext, ReqId};
use cache::Cache;
use request::{self as basic_request, Request as NetworkRequest, Response as NetworkResponse};

pub mod request;

// relevant peer info.
struct Peer {
	status: Status,
	capabilities: Capabilities,
}

impl Peer {
	// Whether a given peer can handle a specific request.
	fn can_handle(&self, pending: &Pending) -> bool {
		match *pending {
			Pending::HeaderProof(ref req, _) =>
				self.capabilities.serve_headers && self.status.head_num > req.num(),
			Pending::HeaderByHash(_, _) => self.capabilities.serve_headers,
			Pending::Block(ref req, _) =>
				self.capabilities.serve_chain_since.as_ref().map_or(false, |x| *x >= req.header.number()),
			Pending::BlockReceipts(ref req, _) =>
				self.capabilities.serve_chain_since.as_ref().map_or(false, |x| *x >= req.0.number()),
			Pending::Account(ref req, _) =>
				self.capabilities.serve_state_since.as_ref().map_or(false, |x| *x >= req.header.number()),
			Pending::Code(ref req, _) =>
				self.capabilities.serve_state_since.as_ref().map_or(false, |x| *x >= req.block_id.1),
			Pending::TxProof(ref req, _) =>
				self.capabilities.serve_state_since.as_ref().map_or(false, |x| *x >= req.header.number()),
		}
	}
}

// Which portions of a CHT proof should be sent.
enum ChtProofSender {
	Both(Sender<(H256, U256)>),
	Hash(Sender<H256>),
	ChainScore(Sender<U256>),
}

// Attempted request info and sender to put received value.
enum Pending {
	HeaderProof(request::HeaderProof, ChtProofSender),
	HeaderByHash(request::HeaderByHash, Sender<encoded::Header>),
	Block(request::Body, Sender<encoded::Block>),
	BlockReceipts(request::BlockReceipts, Sender<Vec<Receipt>>),
	Account(request::Account, Sender<Option<BasicAccount>>),
	Code(request::Code, Sender<Bytes>),
	TxProof(request::TransactionProof, Sender<Result<Executed, ExecutionError>>),
}

impl Pending {
	// Create a network request.
	fn make_request(&self) -> NetworkRequest {
		match *self {
			Pending::HeaderByHash(ref req, _) => NetworkRequest::Headers(basic_request::IncompleteHeadersRequest {
				start: basic_request::HashOrNumber::Hash(req.0).into(),
				skip: 0,
				max: 1,
				reverse: false,
			}),
			Pending::HeaderProof(ref req, _) => NetworkRequest::HeaderProof(basic_request::IncompleteHeaderProofRequest {
				num: req.num().into(),
			}),
			Pending::Block(ref req, _) => NetworkRequest::Body(basic_request::IncompleteBodyRequest {
				hash: req.hash.into(),
			}),
			Pending::BlockReceipts(ref req, _) => NetworkRequest::Receipts(basic_request::IncompleteReceiptsRequest {
				hash: req.0.hash().into(),
			}),
			Pending::Account(ref req, _) => NetworkRequest::Account(basic_request::IncompleteAccountRequest {
				block_hash: req.header.hash().into(),
				address_hash: ::util::Hashable::sha3(&req.address).into(),
			}),
			Pending::Code(ref req, _) => NetworkRequest::Code(basic_request::IncompleteCodeRequest {
				block_hash: req.block_id.0.into(),
				code_hash: req.code_hash.into(),
			}),
			Pending::TxProof(ref req, _) => NetworkRequest::Execution(basic_request::IncompleteExecutionRequest {
				block_hash: req.header.hash().into(),
				from: req.tx.sender(),
				gas: req.tx.gas,
				gas_price: req.tx.gas_price,
				action: req.tx.action.clone(),
				value: req.tx.value,
				data: req.tx.data.clone(),
			}),
		}
	}
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

const RECEIVER_IN_SCOPE: &'static str = "Receiver is still in scope, so it's not dropped; qed";

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

	/// Request a header's hash by block number and CHT root hash.
	/// Returns the hash.
	pub fn hash_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> Receiver<H256> {
		let (sender, receiver) = oneshot::channel();
		let cached = {
			let mut cache = self.cache.lock();
			cache.block_hash(&req.num())
		};

		match cached {
			Some(hash) => sender.send(hash).expect(RECEIVER_IN_SCOPE),
			None => self.dispatch(ctx, Pending::HeaderProof(req, ChtProofSender::Hash(sender))),
		}
		receiver
	}

	/// Request a canonical block's chain score.
	/// Returns the chain score.
	pub fn chain_score_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> Receiver<U256> {
		let (sender, receiver) = oneshot::channel();
		let cached = {
			let mut cache = self.cache.lock();
			cache.block_hash(&req.num()).and_then(|hash| cache.chain_score(&hash))
		};

		match cached {
			Some(score) => sender.send(score).expect(RECEIVER_IN_SCOPE),
			None => self.dispatch(ctx, Pending::HeaderProof(req, ChtProofSender::ChainScore(sender))),
		}

		receiver
	}

	/// Request a canonical block's hash and chain score by number.
	/// Returns the hash and chain score.
	pub fn hash_and_score_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> Receiver<(H256, U256)> {
		let (sender, receiver) = oneshot::channel();
		let cached = {
			let mut cache = self.cache.lock();
			let hash = cache.block_hash(&req.num());
			(
				hash.clone(),
				hash.and_then(|hash| cache.chain_score(&hash)),
			)
		};

		match cached {
			(Some(hash), Some(score)) => sender.send((hash, score)).expect(RECEIVER_IN_SCOPE),
			_ => self.dispatch(ctx, Pending::HeaderProof(req, ChtProofSender::Both(sender))),
		}

		receiver
	}

	/// Request a header by hash. This is less accurate than by-number because we don't know
	/// where in the chain this header lies, and therefore can't find a peer who is supposed to have
	/// it as easily.
	pub fn header_by_hash(&self, ctx: &BasicContext, req: request::HeaderByHash) -> Receiver<encoded::Header> {
		let (sender, receiver) = oneshot::channel();
		match self.cache.lock().block_header(&req.0) {
			Some(hdr) => sender.send(hdr).expect(RECEIVER_IN_SCOPE),
			None => self.dispatch(ctx, Pending::HeaderByHash(req, sender)),
		}
		receiver
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

			sender.send(encoded::Block::new(stream.out())).expect(RECEIVER_IN_SCOPE);
		} else {
			match self.cache.lock().block_body(&req.hash) {
				Some(body) => {
					let mut stream = RlpStream::new_list(3);
					stream.append_raw(&req.header.into_inner(), 1);
					stream.append_raw(&body.into_inner(), 2);

					sender.send(encoded::Block::new(stream.out())).expect(RECEIVER_IN_SCOPE);
				}
				None => self.dispatch(ctx, Pending::Block(req, sender)),
			}
		}
		receiver
	}

	/// Request the receipts for a block. The header serves two purposes:
	/// provide the block hash to fetch receipts for, and for verification of the receipts root.
	pub fn block_receipts(&self, ctx: &BasicContext, req: request::BlockReceipts) -> Receiver<Vec<Receipt>> {
		let (sender, receiver) = oneshot::channel();

		// fast path for empty receipts.
		if req.0.receipts_root() == SHA3_NULL_RLP {
			sender.send(Vec::new()).expect(RECEIVER_IN_SCOPE);
		} else {
			match self.cache.lock().block_receipts(&req.0.hash()) {
				Some(receipts) => sender.send(receipts).expect(RECEIVER_IN_SCOPE),
				None => self.dispatch(ctx, Pending::BlockReceipts(req, sender)),
			}
		}

		receiver
	}

	/// Request an account by address and block header -- which gives a hash to query and a state root
	/// to verify against.
	pub fn account(&self, ctx: &BasicContext, req: request::Account) -> Receiver<Option<BasicAccount>> {
		let (sender, receiver) = oneshot::channel();
		self.dispatch(ctx, Pending::Account(req, sender));
		receiver
	}

	/// Request code by address, known code hash, and block header.
	pub fn code(&self, ctx: &BasicContext, req: request::Code) -> Receiver<Bytes> {
		let (sender, receiver) = oneshot::channel();

		// fast path for no code.
		if req.code_hash == ::util::sha3::SHA3_EMPTY {
			sender.send(Vec::new()).expect(RECEIVER_IN_SCOPE)
		} else {
			self.dispatch(ctx, Pending::Code(req, sender));
		}

		receiver
	}

	/// Request proof-of-execution for a transaction.
	pub fn transaction_proof(&self, ctx: &BasicContext, req: request::TransactionProof) -> Receiver<Result<Executed, ExecutionError>> {
		let (sender, receiver) = oneshot::channel();

		self.dispatch(ctx, Pending::TxProof(req, sender));

		receiver
	}

	// dispatch the request, with a "suitability" function to filter acceptable peers.
	fn dispatch(&self, ctx: &BasicContext, pending: Pending) {
		let mut builder = basic_request::RequestBuilder::default();
		builder.push(pending.make_request())
			.expect("make_request always returns fully complete request; qed");

		let complete = builder.build();

		for (id, peer) in self.peers.read().iter() {
			if !peer.can_handle(&pending) { continue }
			match ctx.request_from(*id, complete.clone()) {
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
		self.orphaned_requests.write().push(pending);
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

		for mut orphaned in to_dispatch {
			let hung_up = match orphaned {
				Pending::HeaderProof(_, ref mut sender) => match *sender {
						ChtProofSender::Both(ref mut s) => check_hangup(s),
						ChtProofSender::Hash(ref mut s) => check_hangup(s),
						ChtProofSender::ChainScore(ref mut s) => check_hangup(s),
				},
				Pending::HeaderByHash(_, ref mut sender) => check_hangup(sender),
				Pending::Block(_, ref mut sender) => check_hangup(sender),
				Pending::BlockReceipts(_, ref mut sender) => check_hangup(sender),
				Pending::Account(_, ref mut sender) => check_hangup(sender),
				Pending::Code(_, ref mut sender) => check_hangup(sender),
				Pending::TxProof(_, ref mut sender) => check_hangup(sender),
			};

			if !hung_up { self.dispatch(ctx, orphaned) }
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

	fn on_responses(&self, ctx: &EventContext, req_id: ReqId, responses: &[basic_request::Response]) {
		let peer = ctx.peer();
		let req = match self.pending_requests.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		let response = match responses.get(0) {
			Some(response) => response,
			None => {
				trace!(target: "on_demand", "Ignoring empty response for request {}", req_id);
				self.dispatch(ctx.as_basic(), req);
				return;
			}
		};

		// handle the response appropriately for the request.
		// all branches which do not return early lead to disabling of the peer
		// due to misbehavior.
		match req {
			Pending::HeaderProof(req, sender) => {
				if let NetworkResponse::HeaderProof(ref response) = *response {
					match req.check_response(&response.proof) {
						Ok((hash, score)) => {
							let mut cache = self.cache.lock();
							cache.insert_block_hash(req.num(), hash);
							cache.insert_chain_score(hash, score);

							match sender {
								ChtProofSender::Both(sender) => { let _ = sender.send((hash, score)); }
								ChtProofSender::Hash(sender) => { let _ = sender.send(hash); }
								ChtProofSender::ChainScore(sender) => { let _ = sender.send(score); }
							}
							return
						}
						Err(e) => warn!("Error handling response for header request: {:?}", e),
					}
				}
			}
			Pending::HeaderByHash(req, sender) => {
				if let NetworkResponse::Headers(ref response) = *response {
					if let Some(header) = response.headers.get(0) {
						match req.check_response(header) {
							Ok(header) => {
								self.cache.lock().insert_block_header(req.0, header.clone());
								let _ = sender.send(header);
								return
							}
							Err(e) => warn!("Error handling response for header request: {:?}", e),
						}
					}
				}
			}
			Pending::Block(req, sender) => {
				if let NetworkResponse::Body(ref response) = *response {
					match req.check_response(&response.body) {
						Ok(block) => {
							self.cache.lock().insert_block_body(req.hash, response.body.clone());
							let _ = sender.send(block);
							return
						}
						Err(e) => warn!("Error handling response for block request: {:?}", e),
					}
				}
			}
			Pending::BlockReceipts(req, sender) => {
				if let NetworkResponse::Receipts(ref response) = *response {
					match req.check_response(&response.receipts) {
						Ok(receipts) => {
							let hash = req.0.hash();
							self.cache.lock().insert_block_receipts(hash, receipts.clone());
							let _ = sender.send(receipts);
							return
						}
						Err(e) => warn!("Error handling response for receipts request: {:?}", e),
					}
				}
			}
			Pending::Account(req, sender) => {
				if let NetworkResponse::Account(ref response) = *response {
					match req.check_response(&response.proof) {
						Ok(maybe_account) => {
							// TODO: validate against request outputs.
							// needs engine + env info as part of request.
							let _ = sender.send(maybe_account);
							return
						}
						Err(e) => warn!("Error handling response for state request: {:?}", e),
					}
				}
			}
			Pending::Code(req, sender) => {
				if let NetworkResponse::Code(ref response) = *response {
					match req.check_response(response.code.as_slice()) {
						Ok(()) => {
							let _ = sender.send(response.code.clone());
							return
						}
						Err(e) => warn!("Error handling response for code request: {:?}", e),
					}
				}
			}
			Pending::TxProof(req, sender) => {
				if let NetworkResponse::Execution(ref response) = *response {
					match req.check_response(&response.items) {
						ProvedExecution::Complete(executed) => {
							let _ = sender.send(Ok(executed));
							return
						}
						ProvedExecution::Failed(err) => {
							let _ = sender.send(Err(err));
							return
						}
						ProvedExecution::BadProof => warn!("Error handling response for transaction proof request"),
					}
				}
			}
		}

		ctx.disable_peer(peer);
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
	use request::Requests;

	use network::{PeerId, NodeId};
	use time::Duration;
	use util::{H256, Mutex};

	struct FakeContext;

	impl BasicContext for FakeContext {
		fn persistent_peer_id(&self, _: PeerId) -> Option<NodeId> { None }
		fn request_from(&self, _: PeerId, _: Requests) -> Result<ReqId, LesError> {
			unimplemented!()
		}
		fn make_announcement(&self, _: Announcement) { }
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
