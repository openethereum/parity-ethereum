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

//! Header download state machine.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::mem;

use ethcore::header::Header;

use light::client::LightChainClient;
use light::net::{EventContext, ReqId};
use light::request::Headers as HeadersRequest;

use network::PeerId;
use rlp::{UntrustedRlp, View};
use util::{Bytes, H256, Mutex};

use super::response;

// amount of blocks between each scaffold entry.
// TODO: move these into paraeters for `RoundStart::new`?
const ROUND_SKIP: usize = 255;

// amount of scaffold frames: these are the blank spaces in "X___X___X"
const ROUND_FRAMES: usize = 255;

// number of attempts to make to get a full scaffold for a sync round.
const SCAFFOLD_ATTEMPTS: usize = 3;

/// Context for a headers response.
pub trait ResponseContext {
	/// Get the peer who sent this response.
	fn responder(&self) ->	PeerId;
	/// Get the request ID this response corresponds to.
	fn req_id(&self) -> ReqId;
	/// Get the (unverified) response data.
	fn data(&self) -> &[Bytes];
	/// Punish the responder.
	fn punish_responder(&self);
}

/// Reasons for sync round abort.
#[derive(Debug, Clone)]
pub enum AbortReason {
	/// Bad sparse header chain along with a list of peers who contributed to it.
	BadScaffold(Vec<PeerId>),
	/// No incoming data.
	NoResponses,
}

// A request for headers with a known starting header hash.
// and a known parent hash for the last block.
#[derive(PartialEq, Eq)]
struct SubchainRequest {
	subchain_parent: (u64, H256),
	headers_request: HeadersRequest,
	subchain_end: (u64, H256),
	downloaded: VecDeque<Header>,
}

// ordered by subchain parent number so pending requests towards the
// front of the round are dispatched first.
impl PartialOrd for SubchainRequest {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.subchain_parent.0.partial_cmp(&other.subchain_parent.0)
	}
}

impl Ord for SubchainRequest {
	fn cmp(&self, other: &Self) -> Ordering {
		self.subchain_parent.0.cmp(&other.subchain_parent.0)
	}
}

/// Manages downloading of interior blocks of a sparse header chain.
pub struct Fetcher {
	sparse: VecDeque<Header>, // sparse header chain.
	requests: BinaryHeap<SubchainRequest>,
	complete_requests: HashMap<H256, SubchainRequest>,
	pending: HashMap<ReqId, SubchainRequest>,
	scaffold_contributors: Vec<PeerId>,
}

impl Fetcher {
	// Produce a new fetcher given a sparse headerchain, in ascending order along
	// with a list of peers who helped produce the chain.
	// The headers must be valid RLP at this point.
	fn new(sparse_headers: Vec<Header>, contributors: Vec<PeerId>) -> Self {
		let mut requests = BinaryHeap::with_capacity(sparse_headers.len() - 1);

		for pair in sparse_headers.windows(2) {
			let low_rung = &pair[0];
			let high_rung = &pair[1];

			let diff = high_rung.number() - low_rung.number();

			// should never happen as long as we verify the gaps
			// gotten from SyncRound::Start
			if diff < 2 { continue }

			let needed_headers = HeadersRequest {
				start: high_rung.parent_hash().clone().into(),
				max: diff as usize - 1,
				skip: 0,
				reverse: true,
			};

			requests.push(SubchainRequest {
				headers_request: needed_headers,
				subchain_end: (high_rung.number() - 1, *high_rung.parent_hash()),
				downloaded: VecDeque::new(),
				subchain_parent: (low_rung.number(), low_rung.hash()),
			});
		}

		Fetcher {
			sparse: sparse_headers.into(),
			requests: requests,
			complete_requests: HashMap::new(),
			pending: HashMap::new(),
			scaffold_contributors: contributors,
		}
	}

	fn process_response<R: ResponseContext>(mut self, ctx: &R) -> SyncRound {
		let mut request = match self.pending.remove(&ctx.req_id()) {
			Some(request) => request,
			None => return SyncRound::Fetch(self),
		};

		let headers = ctx.data();

		if headers.len() == 0 {
			trace!(target: "sync", "Punishing peer {} for empty response", ctx.responder());
			ctx.punish_responder();
			return SyncRound::Fetch(self);
		}

		match response::decode_and_verify(headers, &request.headers_request) {
			Err(e) => {
				trace!(target: "sync", "Punishing peer {} for invalid response ({})", ctx.responder(), e);
				ctx.punish_responder();

				// TODO: track number of attempts per request,
				// abort if failure rate too high.
				self.requests.push(request);
				SyncRound::Fetch(self)
			}
			Ok(headers) => {
				let mut parent_hash = None;
				for header in headers {
					if parent_hash.as_ref().map_or(false, |h| h != &header.hash()) {
						trace!(target: "sync", "Punishing peer {} for parent mismatch", ctx.responder());
						ctx.punish_responder();

						self.requests.push(request);
						return SyncRound::Fetch(self);
					}

					// incrementally update the frame request as we go so we can
					// return at any time in the loop.
					parent_hash = Some(header.parent_hash().clone());
					request.headers_request.start = header.parent_hash().clone().into();
					request.headers_request.max -= 1;

					request.downloaded.push_front(header);
				}

				let subchain_parent = request.subchain_parent.1;

				if request.headers_request.max == 0 {
					if parent_hash.map_or(true, |hash| hash != subchain_parent) {
						let abort = AbortReason::BadScaffold(self.scaffold_contributors);
						return SyncRound::Abort(abort);
					}

					self.complete_requests.insert(subchain_parent, request);
				}

				// state transition not triggered until drain is finished.
				(SyncRound::Fetch(self))
			}
		}
	}

	fn requests_abandoned(mut self, abandoned: &[ReqId]) -> SyncRound {
		for abandoned in abandoned {
			match self.pending.remove(abandoned) {
				None => {},
				Some(req) => self.requests.push(req),
			}
		}

		// TODO: track failure rate and potentially abort.
		SyncRound::Fetch(self)
	}
}

/// Round started: get stepped header chain.
/// from a start block with number X we request 256 headers stepped by 256 from
/// block X + 1.
pub struct RoundStart {
	start_block: (u64, H256),
	pending_req: Option<(ReqId, HeadersRequest)>,
	sparse_headers: Vec<Header>,
	contributors: HashSet<PeerId>,
	attempt: usize,
}

impl RoundStart {
	fn new(start: (u64, H256)) -> Self {
		RoundStart {
			start_block: start.clone(),
			pending_req: None,
			sparse_headers: Vec::new(),
			contributors: HashSet::new(),
			attempt: 0,
		}
	}

	// called on failed attempt. may trigger a transition.
	fn failed_attempt(mut self) -> SyncRound {
		self.attempt += 1;

		if self.attempt >= SCAFFOLD_ATTEMPTS {
			if self.sparse_headers.len() > 1 {
				let fetcher = Fetcher::new(self.sparse_headers, self.contributors.into_iter().collect());
				SyncRound::Fetch(fetcher)
			} else {
				SyncRound::Abort(AbortReason::NoResponses)
			}
		} else {
			SyncRound::Start(self)
		}
	}

	fn process_response<R: ResponseContext>(mut self, ctx: &R) -> SyncRound {
		let req = match self.pending_req.take() {
			Some((id, ref req)) if ctx.req_id() == id => { req.clone() }
			other => {
				self.pending_req = other;
				return SyncRound::Start(self);
			}
		};

		match response::decode_and_verify(ctx.data(), &req) {
			Ok(headers) => {
				self.contributors.insert(ctx.responder());
				self.sparse_headers.extend(headers);

				if self.sparse_headers.len() == ROUND_FRAMES + 1 {
					trace!(target: "sync", "Beginning fetch of blocks between {} sparse headers",
						self.sparse_headers.len());

					let fetcher = Fetcher::new(self.sparse_headers, self.contributors.into_iter().collect());
					return SyncRound::Fetch(fetcher);
				}
			}
			Err(e) => {
				trace!(target: "sync", "Punishing peer {} for malformed response ({})", ctx.responder(), e);
				ctx.punish_responder();
			}
		};

		self.failed_attempt()
	}

	fn requests_abandoned(mut self, abandoned: &[ReqId]) -> SyncRound {
		match self.pending_req.take() {
			Some((id, req)) => {
				if abandoned.iter().any(|r| r == &id) {
					self.pending_req = None;
					self.failed_attempt()
				} else {
					self.pending_req = Some((id, req));
					SyncRound::Start(self)
				}
			}
			None => SyncRound::Start(self),
		}
	}
}

/// Sync round state machine.
pub enum SyncRound {
	/// Beginning a sync round.
	Start(RoundStart),
	/// Fetching intermediate blocks during a sync round.
	Fetch(Fetcher),
	/// Aborted.
	Abort(AbortReason),
}

impl SyncRound {
	fn abort(reason: AbortReason) -> Self {
		trace!(target: "sync", "Aborting sync round: {:?}", reason);

		SyncRound::Abort(reason)
	}

	/// Process an answer to a request. Unknown requests will be ignored.
	pub fn process_response<R: ResponseContext>(self, ctx: &R) -> Self {
		match self {
			SyncRound::Start(round_start) => round_start.process_response(ctx),
			SyncRound::Fetch(fetcher) => fetcher.process_response(ctx),
			other => other,
		}
	}

	/// Return unfulfilled requests from disconnected peer. Unknown requests will be ignored.
	pub fn requests_abandoned(self, abandoned: &[ReqId]) -> Self {
		match self {
			SyncRound::Start(round_start) => round_start.requests_abandoned(abandoned),
			SyncRound::Fetch(fetcher) => fetcher.requests_abandoned(abandoned),
			other => other,
		}
	}

	/// Dispatch pending requests. The dispatcher provided will attempt to
	/// find a suitable peer to serve the request.
	// TODO: have dispatcher take capabilities argument?
	pub fn dispatch_requests<D>(self, dispatcher: D) -> Self
		where D: Fn(HeadersRequest) -> Option<ReqId>
	{
		unimplemented!()
	}
}
