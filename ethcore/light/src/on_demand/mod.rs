// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! On-demand chain requests over LES. This is a major building block for RPCs.
//! The request service is implemented using Futures. Higher level request handlers
//! will take the raw data received here and extract meaningful results from it.

use std::cmp;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use ethcore::executed::{Executed, ExecutionError};
use futures::{Poll, Future, Async};
use futures::sync::oneshot::{self, Receiver};
use network::PeerId;
use parking_lot::{RwLock, Mutex};
use rand;
use rand::Rng;

use net::{
	Handler, PeerStatus, Status, Capabilities,
	Announcement, EventContext, BasicContext, ReqId,
};

use cache::Cache;
use request::{self as basic_request, Request as NetworkRequest};
use self::request::CheckedRequest;

pub use self::request::{Request, Response, HeaderRef, Error as ValidityError};
pub use self::request_guard::{RequestGuard, Error as RequestError};
pub use self::response_guard::{ResponseGuard, Error as ResponseGuardError, Inner as ResponseGuardInner};

pub use types::request::ResponseError;

#[cfg(test)]
mod tests;

pub mod request;
mod request_guard;
mod response_guard;

/// The result of execution
pub type ExecutionResult = Result<Executed, ExecutionError>;

/// The initial backoff interval for OnDemand queries
pub const DEFAULT_REQUEST_MIN_BACKOFF_DURATION: Duration = Duration::from_secs(10);
/// The maximum request interval for OnDemand queries
pub const DEFAULT_REQUEST_MAX_BACKOFF_DURATION: Duration = Duration::from_secs(100);
/// The default window length a response is evaluated
pub const DEFAULT_RESPONSE_TIME_TO_LIVE: Duration = Duration::from_secs(10);
/// The default number of maximum backoff iterations
pub const DEFAULT_MAX_REQUEST_BACKOFF_ROUNDS: usize = 10;
/// The default number failed request to be regarded as failure
pub const DEFAULT_NUM_CONSECUTIVE_FAILED_REQUESTS: usize = 1;

/// OnDemand related errors
pub mod error {
	// Silence: `use of deprecated item 'std::error::Error::cause': replaced by Error::source, which can support downcasting`
	// https://github.com/paritytech/parity-ethereum/issues/10302
	#![allow(deprecated)]

	use futures::sync::oneshot::Canceled;

	error_chain! {

		foreign_links {
			ChannelCanceled(Canceled) #[doc = "Canceled oneshot channel"];
		}

		errors {
			#[doc = "Timeout bad response"]
			BadResponse(err: String) {
				description("Max response evaluation time exceeded")
				display("{}", err)
			}

			#[doc = "OnDemand requests limit exceeded"]
			RequestLimit {
				description("OnDemand request maximum backoff iterations exceeded")
				display("OnDemand request maximum backoff iterations exceeded")
			}
		}
	}
}

// relevant peer info.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Peer {
	status: Status,
	capabilities: Capabilities,
}

impl Peer {
	// whether this peer can fulfill the necessary capabilities for the given
	// request.
	fn can_fulfill(&self, request: &Capabilities) -> bool {
		let local_caps = &self.capabilities;
		let can_serve_since = |req, local| {
			match (req, local) {
				(Some(request_block), Some(serve_since)) => request_block >= serve_since,
				(Some(_), None) => false,
				(None, _) => true,
			}
		};

		local_caps.serve_headers >= request.serve_headers &&
			can_serve_since(request.serve_chain_since, local_caps.serve_chain_since) &&
			can_serve_since(request.serve_state_since, local_caps.serve_state_since)
	}
}

/// Either an array of responses or a single error.
type PendingResponse = self::error::Result<Vec<Response>>;

// Attempted request info and sender to put received value.
struct Pending {
	requests: basic_request::Batch<CheckedRequest>,
	net_requests: basic_request::Batch<NetworkRequest>,
	required_capabilities: Capabilities,
	responses: Vec<Response>,
	sender: oneshot::Sender<PendingResponse>,
	request_guard: RequestGuard,
	response_guard: ResponseGuard,
}

impl Pending {
	// answer as many of the given requests from the supplied cache as possible.
	// TODO: support re-shuffling.
	fn answer_from_cache(&mut self, cache: &Mutex<Cache>) {
		while !self.requests.is_complete() {
			let idx = self.requests.num_answered();
			match self.requests[idx].respond_local(cache) {
				Some(response) => {
					self.requests.supply_response_unchecked(&response);

					// update header and back-references after each from-cache
					// response to ensure that the requests are left in a consistent
					// state and increase the likelihood of being able to answer
					// the next request from cache.
					self.update_header_refs(idx, &response);
					self.fill_unanswered();

					self.responses.push(response);
				}
				None => break,
			}
		}
	}

	// update header refs if the given response contains a header future requests require for
	// verification.
	// `idx` is the index of the request the response corresponds to.
	fn update_header_refs(&mut self, idx: usize, response: &Response) {
		if let Response::HeaderByHash(ref hdr) = *response {
				// fill the header for all requests waiting on this one.
				// TODO: could be faster if we stored a map usize => Vec<usize>
				// but typical use just has one header request that others
				// depend on.
			for r in self.requests.iter_mut().skip(idx + 1) {
				if r.needs_header().map_or(false, |(i, _)| i == idx) {
					r.provide_header(hdr.clone())
				}
			}
		}
	}

	// supply a response.
	fn supply_response(&mut self, cache: &Mutex<Cache>, response: &basic_request::Response)
		-> Result<(), basic_request::ResponseError<self::request::Error>>
	{
		match self.requests.supply_response(&cache, response) {
			Ok(response) => {
				let idx = self.responses.len();
				self.update_header_refs(idx, &response);
				self.responses.push(response);
				Ok(())
			}
			Err(e) => Err(e),
		}
	}

	// if the requests are complete, send the result and consume self.
	fn try_complete(self) -> Option<Self> {
		if self.requests.is_complete() {
			if self.sender.send(Ok(self.responses)).is_err() {
				debug!(target: "on_demand", "Dropped oneshot channel receiver on request");
			}
			None
		} else {
			Some(self)
		}
	}

	fn fill_unanswered(&mut self) {
		self.requests.fill_unanswered();
	}

	// update the cached network requests.
	fn update_net_requests(&mut self) {
		use request::IncompleteRequest;

		let mut builder = basic_request::Builder::default();
		let num_answered = self.requests.num_answered();
		let mut mapping = move |idx| idx - num_answered;

		for request in self.requests.iter().skip(num_answered) {
			let mut net_req = request.clone().into_net_request();

			// all back-references with request index less than `num_answered` have
			// been filled by now. all remaining requests point to nothing earlier
			// than the next unanswered request.
			net_req.adjust_refs(&mut mapping);
			builder.push(net_req)
				.expect("all back-references to answered requests have been filled; qed");
		}

		// update pending fields.
		let capabilities = guess_capabilities(&self.requests[num_answered..]);
		self.net_requests = builder.build();
		self.required_capabilities = capabilities;
	}

	// received too many empty responses, may be away to indicate a faulty request
	fn bad_response(self, response_err: ResponseGuardError) {
		let reqs: Vec<&str> = self.requests.requests().iter().map(|req| {
			match req {
				CheckedRequest::HeaderProof(_, _) => "HeaderProof",
				CheckedRequest::HeaderByHash(_, _) => "HeaderByHash",
				CheckedRequest::HeaderWithAncestors(_, _) => "HeaderWithAncestors",
				CheckedRequest::TransactionIndex(_, _) => "TransactionIndex",
				CheckedRequest::Receipts(_, _) => "Receipts",
				CheckedRequest::Body(_, _) => "Body",
				CheckedRequest::Account(_, _) => "Account",
				CheckedRequest::Code(_, _) => "Code",
				CheckedRequest::Execution(_, _) => "Execution",
				CheckedRequest::Signal(_, _) => "Signal",
			}
		}).collect();

		let err = format!("Bad response on {}: [ {} ]. {}",
			if reqs.len() > 1 { "requests" } else { "request" },
			reqs.join(", "),
			response_err
		);

		let err = self::error::ErrorKind::BadResponse(err);
		if self.sender.send(Err(err.into())).is_err() {
			debug!(target: "on_demand", "Dropped oneshot channel receiver on no response");
		}
	}

	// returning a peer discovery timeout during query attempts
	fn request_limit_reached(self) {
		let err = self::error::ErrorKind::RequestLimit;
		if self.sender.send(Err(err.into())).is_err() {
			debug!(target: "on_demand", "Dropped oneshot channel receiver on time out");
		}
	}
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
			CheckedRequest::HeaderWithAncestors(_, _) =>
				caps.serve_headers = true,
			CheckedRequest::TransactionIndex(_, _) => {} // hashes yield no info.
			CheckedRequest::Signal(_, _) =>
				caps.serve_headers = true,
			CheckedRequest::Body(ref req, _) => if let Ok(ref hdr) = req.0.as_ref() {
				update_since(&mut caps.serve_chain_since, hdr.number());
			},
			CheckedRequest::Receipts(ref req, _) => if let Ok(ref hdr) = req.0.as_ref() {
				update_since(&mut caps.serve_chain_since, hdr.number());
			},
			CheckedRequest::Account(ref req, _) => if let Ok(ref hdr) = req.header.as_ref() {
				update_since(&mut caps.serve_state_since, hdr.number());
			},
			CheckedRequest::Code(ref req, _) => if let Ok(ref hdr) = req.header.as_ref() {
				update_since(&mut caps.serve_state_since, hdr.number());
			},
			CheckedRequest::Execution(ref req, _) => if let Ok(ref hdr) = req.header.as_ref() {
				update_since(&mut caps.serve_state_since, hdr.number());
			},
		}
	}

	caps
}

/// A future extracting the concrete output type of the generic adapter
/// from a vector of responses.
pub struct OnResponses<T: request::RequestAdapter> {
	receiver: Receiver<PendingResponse>,
	_marker: PhantomData<T>,
}

impl<T: request::RequestAdapter> Future for OnResponses<T> {
	type Item = T::Out;
	type Error = self::error::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		match self.receiver.poll() {
			Ok(Async::Ready(Ok(v))) => Ok(Async::Ready(T::extract_from(v))),
			Ok(Async::Ready(Err(e))) => Err(e),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(e) => Err(e.into()),
		}
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
	no_immediate_dispatch: bool,
	response_time_window: Duration,
	request_backoff_start: Duration,
	request_backoff_max: Duration,
	request_backoff_rounds_max: usize,
	request_number_of_consecutive_errors: usize
}

impl OnDemand {

	/// Create a new `OnDemand` service with the given cache.
	pub fn new(
		cache: Arc<Mutex<Cache>>,
		response_time_window: Duration,
		request_backoff_start: Duration,
		request_backoff_max: Duration,
		request_backoff_rounds_max: usize,
		request_number_of_consecutive_errors: usize,
	) -> Self {

		Self {
			pending: RwLock::new(Vec::new()),
			peers: RwLock::new(HashMap::new()),
			in_transit: RwLock::new(HashMap::new()),
			cache,
			no_immediate_dispatch: false,
			response_time_window: Self::sanitize_circuit_breaker_input(response_time_window, "Response time window"),
			request_backoff_start: Self::sanitize_circuit_breaker_input(request_backoff_start, "Request initial backoff time window"),
			request_backoff_max: Self::sanitize_circuit_breaker_input(request_backoff_max, "Request maximum backoff time window"),
			request_backoff_rounds_max,
			request_number_of_consecutive_errors,
		}
	}

	fn sanitize_circuit_breaker_input(dur: Duration, name: &'static str) -> Duration {
		if dur.as_secs() < 1 {
			warn!(target: "on_demand",
				"{} is too short must be at least 1 second, configuring it to 1 second", name);
			Duration::from_secs(1)
		} else {
			dur
		}
	}

	// make a test version: this doesn't dispatch pending requests
	// until you trigger it manually.
	#[cfg(test)]
	fn new_test(
		cache: Arc<Mutex<Cache>>,
		request_ttl: Duration,
		request_backoff_start: Duration,
		request_backoff_max: Duration,
		request_backoff_rounds_max: usize,
		request_number_of_consecutive_errors: usize,
	) -> Self {
		let mut me = OnDemand::new(
			cache,
			request_ttl,
			request_backoff_start,
			request_backoff_max,
			request_backoff_rounds_max,
			request_number_of_consecutive_errors,
		);
		me.no_immediate_dispatch = true;

		me
	}

	/// Submit a vector of requests to be processed together.
	///
	/// Fails if back-references are not coherent.
	/// The returned vector of responses will correspond to the requests exactly.
	pub fn request_raw(&self, ctx: &BasicContext, requests: Vec<Request>)
		-> Result<Receiver<PendingResponse>, basic_request::NoSuchOutput>
	{
		let (sender, receiver) = oneshot::channel();
		if requests.is_empty() {
			assert!(sender.send(Ok(Vec::new())).is_ok(), "receiver still in scope; qed");
			return Ok(receiver);
		}

		let mut builder = basic_request::Builder::default();

		let responses = Vec::with_capacity(requests.len());

		let mut header_producers = HashMap::new();
		for (i, request) in requests.into_iter().enumerate() {
			let request = CheckedRequest::from(request);

			// ensure that all requests needing headers will get them.
			if let Some((idx, field)) = request.needs_header() {
				// a request chain with a header back-reference is valid only if it both
				// points to a request that returns a header and has the same back-reference
				// for the block hash.
				match header_producers.get(&idx) {
					Some(ref f) if &field == *f => {}
					_ => return Err(basic_request::NoSuchOutput),
				}
			}
			if let CheckedRequest::HeaderByHash(ref req, _) = request {
				header_producers.insert(i, req.0);
			}

			builder.push(request)?;
		}

		let requests = builder.build();
		let net_requests = requests.clone().map_requests(|req| req.into_net_request());
		let capabilities = guess_capabilities(requests.requests());

		self.submit_pending(ctx, Pending {
			requests,
			net_requests,
			required_capabilities: capabilities,
			responses,
			sender,
			request_guard: RequestGuard::new(
				self.request_number_of_consecutive_errors as u32,
				self.request_backoff_rounds_max,
				self.request_backoff_start,
				self.request_backoff_max,
			),
			response_guard: ResponseGuard::new(self.response_time_window),
		});

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

	// maybe dispatch pending requests.
	// sometimes
	fn attempt_dispatch(&self, ctx: &BasicContext) {
		if !self.no_immediate_dispatch {
			self.dispatch_pending(ctx)
		}
	}

	// dispatch pending requests, and discard those for which the corresponding
	// receiver has been dropped.
	fn dispatch_pending(&self, ctx: &BasicContext) {
		if self.pending.read().is_empty() {
			return
		}

		let mut pending = self.pending.write();

		// iterate over all pending requests, and check them for hang-up.
		// then, try and find a peer who can serve it.
		let peers = self.peers.read();

		*pending = ::std::mem::replace(&mut *pending, Vec::new())
			.into_iter()
			.filter(|pending| !pending.sender.is_canceled())
			.filter_map(|mut pending| {

				let num_peers = peers.len();
				// The first peer to dispatch the request is chosen at random
				let rand = rand::thread_rng().gen_range(0, cmp::max(1, num_peers));

				for (peer_id, peer) in peers
					.iter()
					.cycle()
					.skip(rand)
					.take(num_peers)
				{

					if !peer.can_fulfill(&pending.required_capabilities) {
						trace!(target: "on_demand", "Peer {} without required capabilities, skipping", peer_id);
						continue
					}

					if pending.request_guard.is_call_permitted() {
						if let Ok(req_id) = ctx.request_from(*peer_id, pending.net_requests.clone()) {
							self.in_transit.write().insert(req_id, pending);
							return None;
						}
					}
				}

				// Register that the request round failed
				if let RequestError::ReachedLimit = pending.request_guard.register_error() {
					pending.request_limit_reached();
					None
				} else {
					Some(pending)
				}
		})
		.collect(); // `pending` now contains all requests we couldn't dispatch

		trace!(target: "on_demand", "Was unable to dispatch {} requests.", pending.len());
	}

	// submit a pending request set. attempts to answer from cache before
	// going to the network. if complete, sends response and consumes the struct.
	fn submit_pending(&self, ctx: &BasicContext, mut pending: Pending) {
		// answer as many requests from cache as we can, and schedule for dispatch
		// if incomplete.

		pending.answer_from_cache(&*self.cache);
		if let Some(mut pending) = pending.try_complete() {
			// update cached requests
			pending.update_net_requests();
			// push into `pending` buffer
			self.pending.write().push(pending);
			// try to dispatch
			self.attempt_dispatch(ctx);
		}
	}
}

impl Handler for OnDemand {
	fn on_connect(
		&self,
		ctx: &EventContext,
		status: &Status,
		capabilities: &Capabilities
	) -> PeerStatus {
		self.peers.write().insert(
			ctx.peer(),
			Peer { status: status.clone(), capabilities: *capabilities }
		);
		self.attempt_dispatch(ctx.as_basic());
		PeerStatus::Kept
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

		self.attempt_dispatch(ctx);
	}

	fn on_announcement(&self, ctx: &EventContext, announcement: &Announcement) {
		{
			let mut peers = self.peers.write();
			if let Some(ref mut peer) = peers.get_mut(&ctx.peer()) {
				peer.status.update_from(&announcement);
				peer.capabilities.update_from(&announcement);
			}
		}

		self.attempt_dispatch(ctx.as_basic());
	}

	fn on_responses(&self, ctx: &EventContext, req_id: ReqId, responses: &[basic_request::Response]) {
		let mut pending = match self.in_transit.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		if responses.is_empty() {
			// Max number of `bad` responses reached, drop the request
			if let Err(e) = pending.response_guard.register_error(&ResponseError::Validity(ValidityError::Empty)) {
				pending.bad_response(e);
				return;
			}
		}

		// for each incoming response
		//   1. ensure verification data filled.
		//   2. pending.requests.supply_response
		//   3. if extracted on-demand response, keep it for later.
		for response in responses {
			if let Err(e) = pending.supply_response(&*self.cache, response) {
				let peer = ctx.peer();
				debug!(target: "on_demand", "Peer {} gave bad response: {:?}", peer, e);
				ctx.disable_peer(peer);

				// Max number of `bad` responses reached, drop the request
				if let Err(err) = pending.response_guard.register_error(&e) {
					pending.bad_response(err);
					return;
				}
			}
		}

		pending.fill_unanswered();
		self.submit_pending(ctx.as_basic(), pending);
	}

	fn tick(&self, ctx: &BasicContext) {
		self.attempt_dispatch(ctx)
	}
}
