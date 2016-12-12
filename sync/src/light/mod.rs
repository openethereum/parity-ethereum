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
use std::sync::Arc;

use light::Client;
use light::net::{Handler, EventContext, Capabilities};
use light::request;
use network: PeerId;
use util::{U256, H256};

struct Peer {
	head_td: U256,
	head_hash: H256,
	head_num: u64,
}

// The block downloader.
// This is instantiated with a starting and a target block
// and produces a priority queue of requests for headers which should be
// fulfilled.
struct Downloader {
	start: u64,
	target: (H256, u64),
	requests: BinaryHeap<Request>,
}

impl Downloader {
	// create a new downloader.
	fn new(start: u64, target: (H256, u64)) -> Self {
		Downloader {
			start: start,
			target: target,
			requests: BinaryHeap::new(),
		}
	}
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
		if !capabilities.serve_headers {
			trace!(target: "sync", "Ignoring irrelevant peer: {}", ctx.peer());
			return;
		}

		{
			let mut best = self.best_seen.lock();
			if best_seen.as_ref().map_or(true, |ref best| status.head_td > best.1) {
				*best_seen = Some(status.head_hash, status.head_td)
			}
		}

		self.peers.write().insert(ctx.peer(), Peer {
			head_td: status.head_td,
			head_hash: status.head_hash,
			head_num: status.head_num,
		});
	}
}

impl LightSync {
	fn assign_request(&self, p-eer: PeerId);
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
