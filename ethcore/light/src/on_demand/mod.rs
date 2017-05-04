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

use ethcore::executed::{Executed, ExecutionError};

use futures::{Async, Poll, Future};
use futures::sync::oneshot::{self, Sender, Receiver, Canceled};
use network::PeerId;
use util::{RwLock, Mutex};

use net::{self, Handler, Status, Capabilities, Announcement, EventContext, BasicContext, ReqId};
use cache::Cache;
use request::{self as basic_request, Request as NetworkRequest};
use self::request::CheckedRequest;

pub use self::request::{Request, Response, HeaderRef};

#[cfg(test)]
mod tests;

pub mod request;

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
	no_immediate_dispatch: bool,
}

impl OnDemand {
	/// Create a new `OnDemand` service with the given cache.
	pub fn new(cache: Arc<Mutex<Cache>>) -> Self {
		OnDemand {
			pending: RwLock::new(Vec::new()),
			peers: RwLock::new(HashMap::new()),
			in_transit: RwLock::new(HashMap::new()),
			cache: cache,
			no_immediate_dispatch: true,
		}
	}

	// make a test version: this doesn't dispatch pending requests
	// until you trigger it manually.
	#[cfg(test)]
	fn new_test(cache: Arc<Mutex<Cache>>) -> Self {
		let mut me = OnDemand::new(cache);
		me.no_immediate_dispatch = true;

		me
	}

	/// Submit a vector of requests to be processed together.
	///
	/// Fails if back-references are not coherent.
	/// The returned vector of responses will correspond to the requests exactly.
	pub fn request_raw(&self, ctx: &BasicContext, requests: Vec<Request>)
		-> Result<Receiver<Vec<Response>>, basic_request::NoSuchOutput>
	{
		use std::collections::HashSet;

		let (sender, receiver) = oneshot::channel();

		if requests.is_empty() {
			assert!(sender.send(Vec::new()).is_ok(), "receiver still in scope; qed");
			return Ok(receiver);
		}

		let mut builder = basic_request::RequestBuilder::default();

		let responses = Vec::with_capacity(requests.len());

		let mut header_producers = HashSet::new();
		for (i, request) in requests.into_iter().enumerate() {
			let request = CheckedRequest::from(request);

			// ensure that all requests needing headers will get them.
			if let Some(idx) = request.needs_header() {
				if !header_producers.contains(&idx) { return Err(basic_request::NoSuchOutput) }
			}
			if let CheckedRequest::HeaderByHash(_, _) = request {
				header_producers.insert(i);
			}

			builder.push(request)?;
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

		self.attempt_dispatch(ctx);

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
		self.attempt_dispatch(ctx.as_basic());
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
		use request::IncompleteRequest;

		let mut pending = match self.in_transit.write().remove(&req_id) {
			Some(req) => req,
			None => return,
		};

		// for each incoming response
		//   1. ensure verification data filled.
		//   2. pending.requests.supply_response
		//   3. if extracted on-demand response
		for response in responses {
			match pending.requests.supply_response(&*self.cache, response) {
				Ok(response) => {
					match response {
						Response::HeaderByHash(ref hdr) => {
							// fill the header for all requests waiting on this one.
							// TODO: could be faster if we stored a map usize => Vec<usize>
							// but typical use just has one header request that others
							// depend on.
							let num_answered = pending.requests.num_answered();
							for r in pending.requests.iter_mut().skip(num_answered) {
								if r.needs_header() == Some(num_answered - 1) {
									r.provide_header(hdr.clone())
								}
							}
						}
						_ => {}, // no other responses produce headers.
					}
					pending.responses.push(response);
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

		// TODO: attempt local responses.
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
		self.attempt_dispatch(ctx.as_basic());
	}

	fn tick(&self, ctx: &BasicContext) {
		self.attempt_dispatch(ctx)
	}
}
