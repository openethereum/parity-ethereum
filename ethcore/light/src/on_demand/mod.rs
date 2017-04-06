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
use std::marker::PhantomData;
use std::sync::Arc;

use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::receipt::Receipt;
use ethcore::executed::{Executed, ExecutionError};

use futures::{future, Async, Poll, Future, BoxFuture};
use futures::sync::oneshot::{self, Sender, Receiver, Canceled};
use network::PeerId;
use rlp::RlpStream;
use util::{Bytes, RwLock, Mutex, U256, H256};
use util::sha3::{SHA3_NULL_RLP, SHA3_EMPTY, SHA3_EMPTY_LIST_RLP};

use net::{self, Handler, Status, Capabilities, Announcement, EventContext, BasicContext, ReqId};
use cache::Cache;
use request::{self as basic_request, Request as NetworkRequest};

pub mod request;

pub use self::request::{CheckedRequest, Request, Response};

/// The result of execution
pub type ExecutionResult = Result<Executed, ExecutionError>;

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

/// A future extracting the concrete output type of the generic adapter
/// from a vector of responses.
pub struct OnResponses<T: request::RequestAdapter> {
	receiver: Receiver<Vec<Response>>,
	_marker: PhantomData<T>,
}

impl<T: request::RequestAdapter> Future for OnResponses<T> {
	type Item = T::Out;
	type Error = Canceled;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		self.receiver.poll().map(|async| async.map(T::extract_from))
	}
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

	/// Request a header's hash by block number and CHT root hash.
	/// Returns the hash.
	pub fn hash_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> BoxFuture<H256, Canceled> {
		let cached = {
			let mut cache = self.cache.lock();
			cache.block_hash(&req.num())
		};

		match cached {
			Some(hash) => future::ok(hash).boxed(),
			None => {
				self.request(ctx, req)
					.expect("request given fully fleshed out; qed")
					.map(|(h, _)| h)
					.boxed()
			},
		}
	}

	/// Request a canonical block's chain score.
	/// Returns the chain score.
	pub fn chain_score_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> BoxFuture<U256, Canceled> {
		let cached = {
			let mut cache = self.cache.lock();
			cache.block_hash(&req.num()).and_then(|hash| cache.chain_score(&hash))
		};

		match cached {
			Some(score) => future::ok(score).boxed(),
			None => {
				self.request(ctx, req)
					.expect("request given fully fleshed out; qed")
					.map(|(_, s)| s)
					.boxed()
			},
		}
	}

	/// Request a canonical block's hash and chain score by number.
	/// Returns the hash and chain score.
	pub fn hash_and_score_by_number(&self, ctx: &BasicContext, req: request::HeaderProof) -> BoxFuture<(H256, U256), Canceled> {
		let cached = {
			let mut cache = self.cache.lock();
			let hash = cache.block_hash(&req.num());
			(
				hash.clone(),
				hash.and_then(|hash| cache.chain_score(&hash)),
			)
		};

		match cached {
			(Some(hash), Some(score)) => future::ok((hash, score)).boxed(),
			_ => {
				self.request(ctx, req)
					.expect("request given fully fleshed out; qed")
					.boxed()
			},
		}
	}

	/// Request a header by hash. This is less accurate than by-number because we don't know
	/// where in the chain this header lies, and therefore can't find a peer who is supposed to have
	/// it as easily.
	pub fn header_by_hash(&self, ctx: &BasicContext, req: request::HeaderByHash) -> BoxFuture<encoded::Header, Canceled> {
		match { self.cache.lock().block_header(&req.0) } {
			Some(hdr) => future::ok(hdr).boxed(),
			None => {
				self.request(ctx, req)
					.expect("request given fully fleshed out; qed")
					.boxed()
			},
		}
	}

	/// Request a block, given its header. Block bodies are requestable by hash only,
	/// and the header is required anyway to verify and complete the block body
	/// -- this just doesn't obscure the network query.
	pub fn block(&self, ctx: &BasicContext, req: request::Body) -> BoxFuture<encoded::Block, Canceled> {
		// fast path for empty body.
		if req.header.transactions_root() == SHA3_NULL_RLP && req.header.uncles_hash() == SHA3_EMPTY_LIST_RLP {
			let mut stream = RlpStream::new_list(3);
			stream.append_raw(&req.header.into_inner(), 1);
			stream.begin_list(0);
			stream.begin_list(0);

			future::ok(encoded::Block::new(stream.out())).boxed()
		} else {
			match { self.cache.lock().block_body(&req.hash) } {
				Some(body) => {
					let mut stream = RlpStream::new_list(3);
					let body = body.rlp();
					stream.append_raw(&req.header.into_inner(), 1);
					stream.append_raw(&body.at(0).as_raw(), 1);
					stream.append_raw(&body.at(1).as_raw(), 1);

					future::ok(encoded::Block::new(stream.out())).boxed()
				}
				None => {
					self.request(ctx, req)
						.expect("request given fully fleshed out; qed")
						.boxed()
				}
			}
		}
	}

	/// Request the receipts for a block. The header serves two purposes:
	/// provide the block hash to fetch receipts for, and for verification of the receipts root.
	pub fn block_receipts(&self, ctx: &BasicContext, req: request::BlockReceipts) -> BoxFuture<Vec<Receipt>, Canceled> {
		// fast path for empty receipts.
		if req.0.receipts_root() == SHA3_NULL_RLP {
			return future::ok(Vec::new()).boxed()
		}

		match { self.cache.lock().block_receipts(&req.0.hash()) } {
			Some(receipts) => future::ok(receipts).boxed(),
			None => {
				self.request(ctx, req)
					.expect("request given fully fleshed out; qed")
					.boxed()
			},
		}
	}

	/// Request an account by address and block header -- which gives a hash to query and a state root
	/// to verify against.
	/// `None` here means that no account by the queried key exists in the queried state.
	pub fn account(&self, ctx: &BasicContext, req: request::Account) -> BoxFuture<Option<BasicAccount>, Canceled> {
		self.request(ctx, req)
			.expect("request given fully fleshed out; qed")
			.boxed()
	}

	/// Request code by address, known code hash, and block header.
	pub fn code(&self, ctx: &BasicContext, req: request::Code) -> BoxFuture<Bytes, Canceled> {
		// fast path for no code.
		if req.code_hash == SHA3_EMPTY {
			future::ok(Vec::new()).boxed()
		} else {
			self.request(ctx, req)
				.expect("request given fully fleshed out; qed")
				.boxed()
		}
	}

	/// Request proof-of-execution for a transaction.
	pub fn transaction_proof(&self, ctx: &BasicContext, req: request::TransactionProof) -> BoxFuture<ExecutionResult, Canceled> {
		self.request(ctx, req)
			.expect("request given fully fleshed out; qed")
			.boxed()
	}

	/// Submit a vector of requests to be processed together.
	///
	/// Fails if back-references are not coherent.
	/// The returned vector of responses will correspond to the requests exactly.
	pub fn request_raw(&self, ctx: &BasicContext, requests: Vec<Request>)
		-> Result<Receiver<Vec<Response>>, basic_request::NoSuchOutput>
	{
		let (sender, receiver) = oneshot::channel();

		if requests.is_empty() {
			assert!(sender.send(Vec::new()).is_ok(), "receiver still in scope; qed");
			return Ok(receiver);
		}

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

		self.dispatch_pending(ctx);

		Ok(receiver)
	}

	/// Submit a strongly-typed batch of requests.
	///
	/// Fails if back-reference are not coherent.
	pub fn request<T>(&self, ctx: &BasicContext, requests: T) -> Result<OnResponses<T>, basic_request::NoSuchOutput>
		where T: request::RequestAdapter
	{
		self.request_raw(ctx, requests.make_requests()).map(|recv| OnResponses {
			receiver: recv,
			_marker: PhantomData,
		})
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
				false => Some(pending),
				true => None,
			})
			.filter_map(|pending| {
				for (peer_id, peer) in peers.iter() { // .shuffle?
					// TODO: see which requests can be answered by the cache?

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

		let mut pending = match self.in_transit.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		// for each incoming response
		//   1. ensure verification data filled. (still TODO since on_demand doesn't use back-references yet)
		//   2. pending.requests.supply_response
		//   3. if extracted on-demand response
		for response in responses {
			match pending.requests.supply_response(&*self.cache, response) {
				Ok(response) => {
					pending.responses.push(response)
				}
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
		let on_demand = OnDemand::new(cache);
		let result = on_demand.header_by_hash(&FakeContext, request::HeaderByHash(H256::default()));

		assert!(on_demand.pending.read().len() == 1);
		drop(result);

		on_demand.dispatch_pending(&FakeContext);
		assert!(on_demand.pending.read().is_empty());
	}
}
