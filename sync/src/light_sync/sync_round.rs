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

use std::collections::{HashMap, VecDeque};
use std::mem;

use ethcore::header::Header;

use light::client::LightChainClient;
use light::net::{EventContext, ReqId};
use light::request::Headers as HeadersRequest;

use network::PeerId;
use rlp::{UntrustedRlp, View};
use util::{Bytes, H256, Mutex};

use super::{Error, Peer};
use super::response::{self, Constraint};

// amount of blocks between each scaffold entry.
// TODO: move these into paraeters for `RoundStart::new`?
const ROUND_SKIP: usize = 255;

// amount of scaffold frames: these are the blank spaces in "X___X___X"
const ROUND_FRAMES: usize = 255;

// number of attempts to make to get a full scaffold for a sync round.
const SCAFFOLD_ATTEMPTS: usize = 3;

/// Reasons for sync round abort.
#[derive(Debug, Clone, Copy)]
pub enum AbortReason {
	/// Bad chain downloaded.
	BadChain,
	/// No incoming data.
	NoResponses,
}

// A request for headers with a known starting header
// and a known parent hash for the last block.
struct Request {
	headers: HeadersRequest,
	end_parent: H256,
}

pub struct Fetcher {
	sparse: Vec<Header>, // sparse header chain.
	requests: VecDeque<Request>,
	pending: HashMap<ReqId, Request>,
}

impl Fetcher {
	// Produce a new fetcher given a sparse headerchain, in ascending order.
	// The headers must be valid RLP at this point.
	fn new(sparse_headers: Vec<Header>) -> Self {
		let mut requests = VecDeque::with_capacity(sparse_headers.len() - 1);
		for pair in sparse_headers.windows(2) {
			let low_rung = &pair[0];
			let high_rung = &pair[1];

			let diff = high_rung.number() - low_rung.number();
			if diff < 2 { continue } // these headers are already adjacent.

			let needed_headers = HeadersRequest {
				start: high_rung.parent_hash().clone().into(),
				max: diff as usize - 1,
				skip: 0,
				reverse: true,
			};

			requests.push_back(Request {
				headers: needed_headers,
				end_parent: low_rung.hash(),
			});
		}

		Fetcher {
			sparse: sparse_headers,
			requests: requests,
			pending: HashMap::new(),
		}
	}

	fn process_response(self, req_id: ReqId, headers: &[Bytes]) -> (SyncRound, Result<(), Error>) {
		unimplemented!()
	}
}

// Round started: get stepped header chain.
// from a start block with number X we request 256 headers stepped by 256 from
// block X + 1.
struct RoundStart {
	start_block: (u64, H256),
	pending_req: Option<(ReqId, HeadersRequest)>,
	sparse_headers: Vec<Header>,
	attempt: usize,
}

impl RoundStart {
	fn new(start: (u64, H256)) -> Self {
		RoundStart {
			start_block: start.clone(),
			pending_req: None,
			sparse_headers: Vec::new(),
			attempt: 0,
		}
	}

	fn process_response(mut self, req_id: ReqId, headers: &[Bytes]) -> (SyncRound, Result<(), Error>) {
		let req = match self.pending_req.take() {
			Some((id, ref req)) if req_id == id => { req.clone() }
			other => {
				self.pending_req = other;
				return (SyncRound::Start(self), Ok(()))
			}
		};

		self.attempt += 1;
		let res = match response::decode_and_verify(headers, &req) {
			Ok(headers) => {
				self.sparse_headers.extend(headers);

				if self.sparse_headers.len() == ROUND_FRAMES + 1 {
					trace!(target: "sync", "Beginning fetch of blocks between {} sparse headers",
						self.sparse_headers.len());

					let fetcher = Fetcher::new(self.sparse_headers);
					return (SyncRound::Fetch(fetcher), Ok(()));
				}

				Ok(())
			}
			Err(e) => Err(e),
		};

		if self.attempt >= SCAFFOLD_ATTEMPTS {
			(SyncRound::Abort(AbortReason::NoResponses), res.map_err(Into::into))
		} else {
			(SyncRound::Start(self), res.map_err(Into::into))
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
	pub fn process_response(self, req_id: ReqId, headers: &[Bytes]) -> (Self, Result<(), Error>) {
		match self {
			SyncRound::Start(round_start) => round_start.process_response(req_id, headers),
			SyncRound::Fetch(fetcher) => fetcher.process_response(req_id, headers),
			other => (other, Ok(())),
		}
	}

	/// Return unfulfilled requests from disconnected peer. Unknown requests will be ignored.
	pub fn requests_abandoned(self, abandoned: &[ReqId]) -> (Self, Result<(), Error>) {
		unimplemented!()
	}

	/// Dispatch pending requests. The dispatcher provided will attempt to
	/// find a suitable peer to serve the request.
	// TODO: have dispatcher take capabilities argument?
	pub fn dispatch_requests<D>(self, dispatcher: D) -> (Self, Result<(), Error>)
		where D: Fn(HeadersRequest) -> Option<ReqId>
	{
		unimplemented!()
	}
}
