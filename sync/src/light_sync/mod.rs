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

//! Light client synchronization.
//!
//! This will synchronize the header chain using LES messages.
//! Dataflow is largely one-directional as headers are pushed into
//! the light client queue for import. Where possible, they are batched
//! in groups.
//!
//! This is written assuming that the client and sync service are running
//! in the same binary; unlike a full node

use std::collections::{BinaryHeap, HashMap};
use std::fmt;
use std::sync::Arc;

use light::client::{Client, BlockDescriptor};
use light::net::{Error as NetError, Handler, EventContext, Capabilities, ReqId};
use light::request;
use network::PeerId;
use rlp::{UntrustedRlp, View};
use util::{U256, H256};

// How many headers we request at a time when searching for best
// common ancestor with peer.
const UNCONFIRMED_SEARCH_SIZE: u64 = 128;

#[derive(Debug)]
enum Error {
	// Peer is useless for now.
	UselessPeer,
	// Peer returned a malformed response.
	MalformedResponse,
	// Peer returned known bad block.
	BadBlock,
	// Peer had a prehistoric common ancestor.
	PrehistoricAncestor,
	// Protocol-level error.
	ProtocolLevel(NetError),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::UselessPeer => write!(f, "Peer is useless"),
			Error::MalformedResponse => write!(f, "Response malformed"),
			Error::BadBlock => write!(f, "Block known to be bad"),
			Error::PrehistoricAncestor => write!(f, "Common ancestor is prehistoric"),
			Error::ProtocolLevel(ref err) => write!(f, "Protocol level error: {}", err),
		}
	}
}

/// Peer chain info.
#[derive(Clone)]
struct ChainInfo {
	head_td: U256,
	head_hash: H256,
	head_num: u64,
}

/// A peer we haven't found a common ancestor for yet.
struct UnconfirmedPeer {
	chain_info: ChainInfo,
	last_batched: u64,
	req_id: ReqId,
}

impl UnconfirmedPeer {
	/// Create an unconfirmed peer. Returns `None` if we cannot make a
	/// common ancestors request for some reason. The event context provided
	/// should be associated with this peer.
	fn create(ctx: &EventContext, chain_info: ChainInfo, best_num: u64) -> Result<Self, Error> {
		let this = ctx.peer();

		if ctx.max_requests(this, request::Kind::Headers) < UNCONFIRMED_SEARCH_SIZE {
			return Err(Error::UselessPeer); // a peer which allows this few header reqs isn't useful anyway.
		}

		let req_id = try!(ctx.request_from(this, request::Request::Headers(request::Headers {
			start: best_num.into(),
			max: ::std::cmp::min(best_num, UNCONFIRMED_SEARCH_SIZE),
			skip: 0,
			reverse: true,
		})));

		Ok(UnconfirmedPeer {
			chain_info: chain_info,
			last_batched: best_num,
			req_id: ReqId,
		})
	}

	/// Feed in the result of the headers query. If an error occurs, the request
	/// is malformed. If a common (hash, number) pair is returned then this is
	/// the common ancestor. If not, then another request for headers has been
	/// dispatched.
	fn check_batch(&mut self, ctx: &EventContext, client: &Client, headers: &[Bytes]) -> Result<Option<H256>, Error> {
		use ethcore::block_status::BlockStatus;

		let mut cur_num = self.last_batched;
		let chain_info = client.chain_info();
		for raw_header in headers {
			let header: Header = try!(UntrustedRlp::new(&raw_header).as_val().map_err(|_| Error::MalformedResponse));
			if header.number() != cur_num { return Err(Error::MalformedResponse) }

			if chain_info.first_block_number.map_or(false, |f| header.number() < f) {
				return Err(Error::PrehistoricAncestor);
			}

			let hash = header.hash();

			match client.status(&hash) {
				BlockStatus::InChain => return Ok(Some(hash)),
				BlockStatus::Bad => return Err(Error::BadBlock),
				BlockStatus::Unknown | BlockStatus::Queued => {},
			}

			cur_num -= 1;
		}
		let this = ctx.peer();

		if cur_num == 0 {
			trace!(target: "sync", "Peer {}: genesis as common ancestor", this);
			return Ok(Some(chain_info.genesis_hash));
		}

		// nothing found, nothing prehistoric.
		// send the next request.
		let req_id = try!(ctx.request_from(this, request::Request::Headers(request::Headers {
			start: cur_num,
			max: ::std::cmp::min(cur_num, UNCONFIRMED_SEARCH_SIZE),
			skip: 0,
			reverse: true,
		})));

		self.req_id = req_id;
	}
}

/// Connected peers as state machines.
///
/// On connection, we'll search for a common ancestor to their chain.
/// Once that's found, we can sync to this peer.
enum Peer {
	// Searching for a common ancestor.
	SearchCommon(Mutex<UnconfirmedPeer>),
	// A peer we can sync to.
	SyncTo(ChainInfo),
}

/// Light client synchronization manager. See module docs for more details.
pub struct LightSync {
	best_seen: Mutex<Option<(H256, U256)>>, // best seen block on the network.
	peers: RwLock<HashMap<PeerId, Peer>>, // peers which are relevant to synchronization.
	client: Arc<Client>,
	downloader: Downloader,
	assigned_requests: HashMap<ReqId, HeaderRequest>,
}

impl Handler for LightSync {
	fn on_connect(&self, ctx: &EventContext, status: &Status, capabilities: &Capabilities) {
		if !capabilities.serve_headers || status.head_num <= self.client.best_block().number {
			trace!(target: "sync", "Ignoring irrelevant peer: {}", ctx.peer());
			return;
		}

		let chain_info = ChainInfo {
			head_td: status.head_td,
			head_hash: status.head_hash,
			head_num: status.head_num,
		};
		let our_best = self.client.chain_info().best_block_number;
		let unconfirmed = match UnconfirmedPeer::create(ctx, chain_info, our_best) {
			Ok(unconfirmed) => unconfirmed,
			Err(e) => {
				trace!(target: "sync", "Failed to create unconfirmed peer: {}", e);
				return;
			}
		};

		self.peers.write().insert(ctx.peer(), Mutex::new(unconfirmed));
	}
}

// public API
impl LightSync {
	/// Create a new instance of `LightSync`.
	///
	/// This won't do anything until registered as a handler
	/// so it can receive
	pub fn new(client: Arc<Client>) -> Self {
		LightSync {
			best_seen: Mutex::new(None),
			peers: HashMap::new(),
			client: client,
		}
	}
}
