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
//! in the same binary; unlike a full node which might communicate via IPC.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use ethcore::header::Header;

use light::client::LightChainClient;
use light::net::{Announcement, Error as NetError, Handler, EventContext, Capabilities, ReqId, Status};
use light::request;
use network::PeerId;
use rlp::{DecoderError, UntrustedRlp, View};
use util::{Bytes, U256, H256, Mutex, RwLock};

mod response;
mod sync_round;

#[derive(Debug)]
enum Error {
	// Peer returned a malformed response.
	MalformedResponse(response::BasicError),
	// Peer returned known bad block.
	BadBlock,
	// Protocol-level error.
	ProtocolLevel(NetError),
}

impl From<NetError> for Error {
	fn from(net_error: NetError) -> Self {
		Error::ProtocolLevel(net_error)
	}
}

impl From<response::BasicError> for Error {
	fn from(err: response::BasicError) -> Self {
		Error::MalformedResponse(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::MalformedResponse(ref err) => write!(f, "{}", err),
			Error::BadBlock => write!(f, "Block known to be bad"),
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

struct Peer {
	first_status: ChainInfo,
	status: ChainInfo,
}

impl Peer {
	/// Create a peer object.
	fn new(chain_info: ChainInfo) -> Self {
		Peer {
			first_status: chain_info.clone(),
			status: chain_info.clone(),
		}
	}
}

/// Light client synchronization manager. See module docs for more details.
pub struct LightSync<L: LightChainClient> {
	best_seen: Mutex<Option<(H256, U256)>>, // best seen block on the network.
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>, // peers which are relevant to synchronization.
	client: Arc<L>,
}

impl<L: LightChainClient> Handler for LightSync<L> {
	fn on_connect(&self, ctx: &EventContext, status: &Status, capabilities: &Capabilities) {
		let our_best = self.client.chain_info().best_block_number;

		if !capabilities.serve_headers || status.head_num <= our_best {
			trace!(target: "sync", "Ignoring irrelevant peer: {}", ctx.peer());
			return;
		}

		let chain_info = ChainInfo {
			head_td: status.head_td,
			head_hash: status.head_hash,
			head_num: status.head_num,
		};

		self.peers.write().insert(ctx.peer(), Mutex::new(Peer::new(chain_info)));
	}

	fn on_disconnect(&self, ctx: &EventContext, _unfulfilled: &[ReqId]) {
		let peer_id = ctx.peer();

	}

	fn on_announcement(&self, ctx: &EventContext, announcement: &Announcement) {
		// restart search for common ancestor if necessary.
		// restart download if necessary.
		// if this is a peer we found irrelevant earlier, we may want to
		// re-evaluate their usefulness.
		if !self.peers.read().contains_key(&ctx.peer()) { return }

		trace!(target: "sync", "Announcement from peer {}: new chain head {:?}, reorg depth {}",
			ctx.peer(), (announcement.head_hash, announcement.head_num), announcement.reorg_depth);
	}

	fn on_block_headers(&self, ctx: &EventContext, req_id: ReqId, headers: &[Bytes]) {
		let peer_id = ctx.peer();
	}
}

// public API
impl<L: LightChainClient> LightSync<L> {
	/// Create a new instance of `LightSync`.
	///
	/// This won't do anything until registered as a handler
	/// so it can act on events.
	pub fn new(client: Arc<L>) -> Self {
		LightSync {
			best_seen: Mutex::new(None),
			peers: RwLock::new(HashMap::new()),
			client: client,
		}
	}
}
