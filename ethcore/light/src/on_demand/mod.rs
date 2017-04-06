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

// TODO [ToDr] Suppressing deprecation warnings. Rob will fix the API anyway.
#![allow(deprecated)]

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
use util::sha3::{SHA3_NULL_RLP, SHA3_EMPTY, SHA3_EMPTY_LIST_RLP};

use net::{self, Handler, Status, Capabilities, Announcement, EventContext, BasicContext, ReqId};
use cache::Cache;
use request::{self as basic_request, Request as NetworkRequest, Response as NetworkResponse};

pub mod request;

pub use self::request::{CheckedRequest ,Request, Response};

// relevant peer info.
struct Peer {
	status: Status,
	capabilities: Capabilities,
}

impl Peer {
	// whether this peer can fulfill the
	fn can_fulfill(&self, c: &Capabilities) -> bool {
		let caps = &self.capabilities;

		caps.serve_headers == c.serve_headers &&
			caps.serve_chain_since >= c.serve_chain_since &&
			caps.serve_state_since >= c.serve_chain_since
	}
}

// Which portions of a CHT proof should be sent.
enum ChtProofSender {
	Both(Sender<(H256, U256)>),
	Hash(Sender<H256>),
	ChainScore(Sender<U256>),
}

// Attempted request info and sender to put received value.
struct Pending {
	requests: basic_request::Requests<CheckedRequest>,
	net_requests: basic_request::Requests<NetworkRequest>,
	required_capabilities: Capabilities,
	responses: Vec<Response>,
	sender: oneshot::Sender<Vec<Response>>,
}

// helper to guess capabilities required for a given batch of network requests.
fn guess_capabilities(requests: &[CheckedRequest]) -> Capabilities {
	let mut caps = Capabilities {
		serve_headers: false,
		serve_chain_since: None,
		serve_state_since: None,
		tx_relay: false,
	};

	let update_since = |current: &mut Option<u64>, new|
		*current = match *current {
			Some(x) => Some(::std::cmp::min(x, new)),
			None => Some(new),
		};

	for request in requests {
		match *request {
			// TODO: might be worth returning a required block number for this also.
			CheckedRequest::HeaderProof(_, _) =>
				caps.serve_headers = true,
			CheckedRequest::HeaderByHash(_, _) =>
				caps.serve_headers = true,
			CheckedRequest::Body(ref req, _) =>
				update_since(&mut caps.serve_chain_since, req.header.number()),
			CheckedRequest::Receipts(ref req, _) =>
				update_since(&mut caps.serve_chain_since, req.0.number()),
			CheckedRequest::Account(ref req, _) =>
				update_since(&mut caps.serve_state_since, req.header.number()),
			CheckedRequest::Code(ref req, _) =>
				update_since(&mut caps.serve_state_since, req.block_id.1),
			CheckedRequest::Execution(ref req, _) =>
				update_since(&mut caps.serve_state_since, req.header.number()),
		}
	}

	caps
}

/// On demand request service. See module docs for more details.
/// Accumulates info about all peers' capabilities and dispatches
/// requests to them accordingly.
// lock in declaration order.
pub struct OnDemand {
	pending: RwLock<Vec<Pending>>,
	peers: RwLock<HashMap<PeerId, Peer>>,
	in_transit: RwLock<HashMap<ReqId, Pending>>,
	cache: Arc<Mutex<Cache>>,
}

const RECEIVER_IN_SCOPE: &'static str = "Receiver is still in scope, so it's not dropped; qed";

impl OnDemand {
	/// Create a new `OnDemand` service with the given cache.
	pub fn new(cache: Arc<Mutex<Cache>>) -> Self {
		OnDemand {
			pending: RwLock::new(Vec::new()),
			peers: RwLock::new(HashMap::new()),
			in_transit: RwLock::new(HashMap::new()),
			cache: cache,
		}
	}

	// /// Request a header's hash by block number and CHT root hash.
	// /// Returns the hash.
	// pub fn hash_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> Receiver<H256> {
	// 	let (sender, receiver) = oneshot::channel();
	// 	let cached = {
	// 		let mut cache = self.cache.lock();
	// 		cache.block_hash(&req.num())
	// 	};

	// 	match cached {
	// 		Some(hash) => sender.send(hash).expect(RECEIVER_IN_SCOPE),
	// 		None => self.dispatch(ctx, Pending::HeaderProof(req, ChtProofSender::Hash(sender))),
	// 	}
	// 	receiver
	// }

	// /// Request a canonical block's chain score.
	// /// Returns the chain score.
	// pub fn chain_score_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> Receiver<U256> {
	// 	let (sender, receiver) = oneshot::channel();
	// 	let cached = {
	// 		let mut cache = self.cache.lock();
	// 		cache.block_hash(&req.num()).and_then(|hash| cache.chain_score(&hash))
	// 	};

	// 	match cached {
	// 		Some(score) => sender.send(score).expect(RECEIVER_IN_SCOPE),
	// 		None => self.dispatch(ctx, Pending::HeaderProof(req, ChtProofSender::ChainScore(sender))),
	// 	}

	// 	receiver
	// }

	// /// Request a canonical block's hash and chain score by number.
	// /// Returns the hash and chain score.
	// pub fn hash_and_score_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> Receiver<(H256, U256)> {
	// 	let (sender, receiver) = oneshot::channel();
	// 	let cached = {
	// 		let mut cache = self.cache.lock();
	// 		let hash = cache.block_hash(&req.num());
	// 		(
	// 			hash.clone(),
	// 			hash.and_then(|hash| cache.chain_score(&hash)),
	// 		)
	// 	};

	// 	match cached {
	// 		(Some(hash), Some(score)) => sender.send((hash, score)).expect(RECEIVER_IN_SCOPE),
	// 		_ => self.dispatch(ctx, Pending::HeaderProof(req, ChtProofSender::Both(sender))),
	// 	}

	// 	receiver
	// }

	// /// Request a header by hash. This is less accurate than by-number because we don't know
	// /// where in the chain this header lies, and therefore can't find a peer who is supposed to have
	// /// it as easily.
	// pub fn header_by_hash(&self, ctx: &BasicContext, req: request::HeaderByHash) -> Receiver<encoded::Header> {
	// 	let (sender, receiver) = oneshot::channel();
	// 	match { self.cache.lock().block_header(&req.0) } {
	// 		Some(hdr) => sender.send(hdr).expect(RECEIVER_IN_SCOPE),
	// 		None => self.dispatch(ctx, Pending::HeaderByHash(req, sender)),
	// 	}
	// 	receiver
	// }

	// /// Request a block, given its header. Block bodies are requestable by hash only,
	// /// and the header is required anyway to verify and complete the block body
	// /// -- this just doesn't obscure the network query.
	// pub fn block(&self, ctx: &BasicContext, req: request::Body) -> Receiver<encoded::Block> {
	// 	let (sender, receiver) = oneshot::channel();

	// 	// fast path for empty body.
	// 	if req.header.transactions_root() == SHA3_NULL_RLP && req.header.uncles_hash() == SHA3_EMPTY_LIST_RLP {
	// 		let mut stream = RlpStream::new_list(3);
	// 		stream.append_raw(&req.header.into_inner(), 1);
	// 		stream.begin_list(0);
	// 		stream.begin_list(0);

	// 		sender.send(encoded::Block::new(stream.out())).expect(RECEIVER_IN_SCOPE);
	// 	} else {
	// 		match { self.cache.lock().block_body(&req.hash) } {
	// 			Some(body) => {
	// 				let mut stream = RlpStream::new_list(3);
	// 				let body = body.rlp();
	// 				stream.append_raw(&req.header.into_inner(), 1);
	// 				stream.append_raw(&body.at(0).as_raw(), 1);
	// 				stream.append_raw(&body.at(1).as_raw(), 1);

	// 				sender.send(encoded::Block::new(stream.out())).expect(RECEIVER_IN_SCOPE);
	// 			}
	// 			None => self.dispatch(ctx, Pending::Block(req, sender)),
	// 		}
	// 	}
	// 	receiver
	// }

	// /// Request the receipts for a block. The header serves two purposes:
	// /// provide the block hash to fetch receipts for, and for verification of the receipts root.
	// pub fn block_receipts(&self, ctx: &BasicContext, req: request::BlockReceipts) -> Receiver<Vec<Receipt>> {
	// 	let (sender, receiver) = oneshot::channel();

	// 	// fast path for empty receipts.
	// 	if req.0.receipts_root() == SHA3_NULL_RLP {
	// 		sender.send(Vec::new()).expect(RECEIVER_IN_SCOPE);
	// 	} else {
	// 		match { self.cache.lock().block_receipts(&req.0.hash()) } {
	// 			Some(receipts) => sender.send(receipts).expect(RECEIVER_IN_SCOPE),
	// 			None => self.dispatch(ctx, Pending::BlockReceipts(req, sender)),
	// 		}
	// 	}

	// 	receiver
	// }

	// /// Request an account by address and block header -- which gives a hash to query and a state root
	// /// to verify against.
	// pub fn account(&self, ctx: &BasicContext, req: request::Account) -> Receiver<BasicAccount> {
	// 	let (sender, receiver) = oneshot::channel();
	// 	self.dispatch(ctx, Pending::Account(req, sender));
	// 	receiver
	// }

	// /// Request code by address, known code hash, and block header.
	// pub fn code(&self, ctx: &BasicContext, req: request::Code) -> Receiver<Bytes> {
	// 	let (sender, receiver) = oneshot::channel();

	// 	// fast path for no code.
	// 	if req.code_hash == SHA3_EMPTY {
	// 		sender.send(Vec::new()).expect(RECEIVER_IN_SCOPE)
	// 	} else {
	// 		self.dispatch(ctx, Pending::Code(req, sender));
	// 	}

	// 	receiver
	// }

	// /// Request proof-of-execution for a transaction.
	// pub fn transaction_proof(&self, ctx: &BasicContext, req: request::TransactionProof) -> Receiver<Result<Executed, ExecutionError>> {
	// 	let (sender, receiver) = oneshot::channel();

	// 	self.dispatch(ctx, Pending::TxProof(req, sender));

	// 	receiver
	// }

	/// Submit a batch of requests.
	///
	/// Fails if back-references are not coherent.
	/// The returned vector of responses will match the requests exactly.
	pub fn make_requests(&self, ctx: &BasicContext, requests: Vec<Request>)
		-> Result<Receiver<Vec<Response>>, basic_request::NoSuchOutput>
	{
		let (sender, receiver) = oneshot::channel();

		let mut builder = basic_request::RequestBuilder::default();

		let responses = Vec::with_capacity(requests.len());
		for request in requests {
			builder.push(CheckedRequest::from(request))?;
		}

		let requests = builder.build();
		let net_requests = requests.clone().map_requests(|req| req.into_net_request());
		let capabilities = guess_capabilities(requests.requests());

		self.pending.write().push(Pending {
			requests: requests,
			net_requests: net_requests,
			required_capabilities: capabilities,
			responses: responses,
			sender: sender,
		});

		Ok(receiver)
	}

	// dispatch pending requests, and discard those for which the corresponding
	// receiver has been dropped.
	fn dispatch_pending(&self, ctx: &BasicContext) {
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

		if self.pending.read().is_empty() { return }
		let mut pending = self.pending.write();

		// iterate over all pending requests, and check them for hang-up.
		// then, try and find a peer who can serve it.
		let peers = self.peers.read();
		*pending = ::std::mem::replace(&mut *pending, Vec::new()).into_iter()
			.filter_map(|mut pending| match check_hangup(&mut pending.sender) {
				true => Some(pending),
				false => None,
			})
			.filter_map(|pending| {
				for (peer_id, peer) in peers.iter() { // .shuffle?
					if !peer.can_fulfill(&pending.required_capabilities) {
						continue
					}

					match ctx.request_from(*peer_id, pending.net_requests.clone()) {
						Ok(req_id) => {
							self.in_transit.write().insert(req_id, pending);
							return None
						}
						Err(net::Error::NoCredits) => {}
						Err(e) => debug!(target: "on_demand", "Error dispatching request to peer: {}", e),
					}
				}
				Some(pending)
			})
			.collect(); // `pending` now contains all requests we couldn't dispatch.
	}
}

impl Handler for OnDemand {
	fn on_connect(&self, ctx: &EventContext, status: &Status, capabilities: &Capabilities) {
		self.peers.write().insert(ctx.peer(), Peer { status: status.clone(), capabilities: capabilities.clone() });
		self.dispatch_pending(ctx.as_basic());
	}

	fn on_disconnect(&self, ctx: &EventContext, unfulfilled: &[ReqId]) {
		self.peers.write().remove(&ctx.peer());
		let ctx = ctx.as_basic();

		{
			let mut pending = self.pending.write();
			for unfulfilled in unfulfilled {
				if let Some(unfulfilled) = self.in_transit.write().remove(unfulfilled) {
					trace!(target: "on_demand", "Attempting to reassign dropped request");
					pending.push(unfulfilled);
				}
			}
		}

		self.dispatch_pending(ctx);
	}

	fn on_announcement(&self, ctx: &EventContext, announcement: &Announcement) {
		{
			let mut peers = self.peers.write();
			if let Some(ref mut peer) = peers.get_mut(&ctx.peer()) {
				peer.status.update_from(&announcement);
				peer.capabilities.update_from(&announcement);
			}
		}

		self.dispatch_pending(ctx.as_basic());
	}

	fn on_responses(&self, ctx: &EventContext, req_id: ReqId, responses: &[basic_request::Response]) {
		use request::IncompleteRequest;

		let peer = ctx.peer();
		let mut pending = match self.in_transit.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		// for each incoming response
		//   1. ensure verification data filled.
		//   2. pending.requests.supply_response
		//   3. if extracted on-demand response
		for response in responses {
			match pending.requests.supply_response(response) {
				Ok(response) => pending.responses.push(response),
				Err(e) => {
					let peer = ctx.peer();
					debug!(target: "on_demand", "Peer {} gave bad response: {:?}", peer, e);
					ctx.disable_peer(peer);

					break;
				}
			}
		}

		if pending.requests.is_complete() {
			let _ = pending.sender.send(pending.responses);

			return;
		}

		// update network requests (unless we're done, in which case fulfill the future.)
		let mut builder = basic_request::RequestBuilder::default();
		let num_answered = pending.requests.num_answered();
		let mut mapping = move |idx| idx - num_answered;

		for request in pending.requests.requests().iter().skip(num_answered) {
			let mut net_req = request.clone().into_net_request();

			// all back-references with request index less than `num_answered` have
			// been filled by now. all remaining requests point to nothing earlier
			// than the next unanswered request.
			net_req.adjust_refs(&mut mapping);
			builder.push(net_req)
				.expect("all back-references to answered requests have been filled; qed");
		}

		// update pending fields and re-queue.
		let capabilities = guess_capabilities(&pending.requests.requests()[num_answered..]);
		pending.net_requests = builder.build();
		pending.required_capabilities = capabilities;

		self.pending.write().push(pending);
		self.dispatch_pending(ctx.as_basic());
	}

	fn tick(&self, ctx: &BasicContext) {
		self.dispatch_pending(ctx)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::sync::Arc;

	use cache::Cache;
	use net::{Announcement, BasicContext, ReqId, Error as LesError};
	use request::NetworkRequests;

	use network::{PeerId, NodeId};
	use time::Duration;
	use util::{H256, Mutex};

	struct FakeContext;

	impl BasicContext for FakeContext {
		fn persistent_peer_id(&self, _: PeerId) -> Option<NodeId> { None }
		fn request_from(&self, _: PeerId, _: NetworkRequests) -> Result<ReqId, LesError> {
			unimplemented!()
		}
		fn make_announcement(&self, _: Announcement) { }
		fn disconnect_peer(&self, _: PeerId) { }
		fn disable_peer(&self, _: PeerId) { }
	}

	#[test]
	fn detects_hangup() {
		let cache = Arc::new(Mutex::new(Cache::new(Default::default(), Duration::hours(6))));
		let on_demand = OnDemand::new(cache, 0.into());
		let result = on_demand.header_by_hash(&FakeContext, request::HeaderByHash(H256::default()));

		assert!(on_demand.orphaned_requests.read().len() == 1);
		drop(result);

		on_demand.dispatch_pending(&FakeContext);
		assert!(on_demand.orphaned_requests.read().is_empty());
	}
}
