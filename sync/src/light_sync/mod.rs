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
//!
//!
//! Sync strategy:
//! - Find a common ancestor with peers.
//! - Split the chain up into subchains, which are downloaded in parallel from various peers in rounds.
//! - When within a certain distance of the head of the chain, aggressively download all
//!   announced blocks.
//! - On bad block/response, punish peer and reset.

use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use light::client::LightChainClient;
use light::net::{
	Announcement, Handler, BasicContext, EventContext,
	Capabilities, ReqId, Status,
};
use light::request;
use network::PeerId;
use util::{Bytes, U256, H256, Mutex, RwLock};
use rand::{Rng, OsRng};

use self::sync_round::{AbortReason, SyncRound, ResponseContext};

mod response;
mod sync_round;

#[cfg(test)]
mod tests;

/// Peer chain info.
#[derive(Clone)]
struct ChainInfo {
	head_td: U256,
	head_hash: H256,
	head_num: u64,
}

struct Peer {
	status: ChainInfo,
}

impl Peer {
	// Create a new peer.
	fn new(chain_info: ChainInfo) -> Self {
		Peer {
			status: chain_info,
		}
	}
}
// search for a common ancestor with the best chain.
#[derive(Debug)]
enum AncestorSearch {
	Queued(u64), // queued to search for blocks starting from here.
	Awaiting(ReqId, u64, request::Headers), // awaiting response for this request.
	Prehistoric, // prehistoric block found. TODO: start to roll back CHTs.
	FoundCommon(u64, H256), // common block found.
	Genesis, // common ancestor is the genesis.
}

impl AncestorSearch {
	fn begin(best_num: u64) -> Self {
		match best_num {
			0 => AncestorSearch::Genesis,
			_ => AncestorSearch::Queued(best_num),
		}
	}

	fn process_response<L>(self, ctx: &ResponseContext, client: &L) -> AncestorSearch
		where L: LightChainClient
	{
		let first_num = client.chain_info().first_block_number.unwrap_or(0);
		match self {
			AncestorSearch::Awaiting(id, start, req) => {
				if &id == ctx.req_id() {
					match response::decode_and_verify(ctx.data(), &req) {
						Ok(headers) => {
							for header in &headers {
								if client.is_known(&header.hash()) {
									debug!(target: "sync", "Found common ancestor with best chain");
									return AncestorSearch::FoundCommon(header.number(), header.hash());
								}

								if header.number() <= first_num {
									debug!(target: "sync", "Prehistoric common ancestor with best chain.");
									return AncestorSearch::Prehistoric;
								}
							}

							AncestorSearch::Queued(start - headers.len() as u64)
						}
						Err(e) => {
							trace!(target: "sync", "Bad headers response from {}: {}", ctx.responder(), e);

							ctx.punish_responder();
							AncestorSearch::Queued(start)
						}
					}
				} else {
					AncestorSearch::Awaiting(id, start, req)
				}
			}
			other => other,
		}
	}

	fn dispatch_request<F>(self, mut dispatcher: F) -> AncestorSearch
		where F: FnMut(request::Headers) -> Option<ReqId>
	{
		const BATCH_SIZE: usize = 64;

		match self {
			AncestorSearch::Queued(start) => {
				trace!(target: "sync", "Requesting {} reverse headers from {} to find common ancestor",
					BATCH_SIZE, start);

				let req = request::Headers {
					start: start.into(),
					max: ::std::cmp::min(start as usize, BATCH_SIZE),
					skip: 0,
					reverse: true,
				};

				match dispatcher(req.clone()) {
					Some(req_id) => AncestorSearch::Awaiting(req_id, start, req),
					None => AncestorSearch::Queued(start),
				}
			}
			other => other,
		}
	}
}

// synchronization state machine.
#[derive(Debug)]
enum SyncState {
	// Idle (waiting for peers) or at chain head.
	Idle,
	// searching for common ancestor with best chain.
	// queue should be cleared at this phase.
	AncestorSearch(AncestorSearch),
	// Doing sync rounds.
	Rounds(SyncRound),
}

struct ResponseCtx<'a> {
	peer: PeerId,
	req_id: ReqId,
	ctx: &'a BasicContext,
	data: &'a [Bytes],
}

impl<'a> ResponseContext for ResponseCtx<'a> {
	fn responder(&self) -> PeerId { self.peer }
	fn req_id(&self) -> &ReqId { &self.req_id }
	fn data(&self) -> &[Bytes] { self.data }
	fn punish_responder(&self) { self.ctx.disable_peer(self.peer) }
}

/// Light client synchronization manager. See module docs for more details.
pub struct LightSync<L: LightChainClient> {
	best_seen: Mutex<Option<(H256, U256)>>, // best seen block on the network.
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>, // peers which are relevant to synchronization.
	client: Arc<L>,
	rng: Mutex<OsRng>,
	state: Mutex<SyncState>,
}

impl<L: LightChainClient> Handler for LightSync<L> {
	fn on_connect(&self, ctx: &EventContext, status: &Status, capabilities: &Capabilities) {
		let our_best = self.client.chain_info().best_block_number;

		if !capabilities.serve_headers || status.head_num <= our_best {
			trace!(target: "sync", "Disconnecting irrelevant peer: {}", ctx.peer());
			ctx.disconnect_peer(ctx.peer());
			return;
		}

		let chain_info = ChainInfo {
			head_td: status.head_td,
			head_hash: status.head_hash,
			head_num: status.head_num,
		};

		{
			let mut best = self.best_seen.lock();
			if best.as_ref().map_or(true, |b| status.head_td > b.1) {
				*best = Some((status.head_hash, status.head_td));
			}
		}

		self.peers.write().insert(ctx.peer(), Mutex::new(Peer::new(chain_info)));
		self.maintain_sync(ctx.as_basic());
	}

	fn on_disconnect(&self, ctx: &EventContext, unfulfilled: &[ReqId]) {
		let peer_id = ctx.peer();

		let peer = match self.peers.write().remove(&peer_id).map(|p| p.into_inner()) {
			Some(peer) => peer,
			None => return,
		};

		trace!(target: "sync", "peer {} disconnecting", peer_id);

		let new_best = {
			let mut best = self.best_seen.lock();
			let peer_best = (peer.status.head_hash, peer.status.head_td);

			if best.as_ref().map_or(false, |b| b == &peer_best) {
				// search for next-best block.
				let next_best: Option<(H256, U256)> = self.peers.read().values()
					.map(|p| p.lock())
					.map(|p| (p.status.head_hash, p.status.head_td))
					.fold(None, |acc, x| match acc {
						Some(acc) => if x.1 > acc.1 { Some(x) } else { Some(acc) },
						None => Some(x),
					});

				*best = next_best;
			}

			best.clone()
		};

		if new_best.is_none() {
			debug!(target: "sync", "No peers remain. Reverting to idle");
			*self.state.lock() = SyncState::Idle;
		} else {
			let mut state = self.state.lock();

			*state = match mem::replace(&mut *state, SyncState::Idle) {
				SyncState::Idle => SyncState::Idle,
				SyncState::AncestorSearch(search) => SyncState::AncestorSearch(search),
				SyncState::Rounds(round) => SyncState::Rounds(round.requests_abandoned(unfulfilled)),
			};
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn on_announcement(&self, ctx: &EventContext, announcement: &Announcement) {
		let last_td = {
			let peers = self.peers.read();
			match peers.get(&ctx.peer()) {
				None => return,
				Some(peer) => {
					let mut peer = peer.lock();
					let last_td = peer.status.head_td;
					peer.status = ChainInfo {
						head_td: announcement.head_td,
						head_hash: announcement.head_hash,
						head_num: announcement.head_num,
					};
					last_td
				}
			}
		};

		trace!(target: "sync", "Announcement from peer {}: new chain head {:?}, reorg depth {}",
			ctx.peer(), (announcement.head_hash, announcement.head_num), announcement.reorg_depth);

		if last_td > announcement.head_td {
			trace!(target: "sync", "Peer {} moved backwards.", ctx.peer());
			self.peers.write().remove(&ctx.peer());
			ctx.disconnect_peer(ctx.peer());
		}

		{
			let mut best = self.best_seen.lock();
			if best.as_ref().map_or(true, |b| announcement.head_td > b.1) {
				*best = Some((announcement.head_hash, announcement.head_td));
			}
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn on_block_headers(&self, ctx: &EventContext, req_id: ReqId, headers: &[Bytes]) {
		if !self.peers.read().contains_key(&ctx.peer()) {
			return
		}

		{
			let mut state = self.state.lock();

			let ctx = ResponseCtx {
				peer: ctx.peer(),
				req_id: req_id,
				ctx: ctx.as_basic(),
				data: headers,
			};

			*state = match mem::replace(&mut *state, SyncState::Idle) {
				SyncState::Idle => SyncState::Idle,
				SyncState::AncestorSearch(search) =>
					SyncState::AncestorSearch(search.process_response(&ctx, &*self.client)),
				SyncState::Rounds(round) => SyncState::Rounds(round.process_response(&ctx)),
			};
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn tick(&self, ctx: &BasicContext) {
		self.maintain_sync(ctx);
	}
}

// private helpers
impl<L: LightChainClient> LightSync<L> {
	// Begins a search for the common ancestor and our best block.
	// does not lock state, instead has a mutable reference to it passed.
	fn begin_search(&self, state: &mut SyncState) {
		if let None =  *self.best_seen.lock() {
			// no peers.
			*state = SyncState::Idle;
			return;
		}

		self.client.flush_queue();
		let chain_info = self.client.chain_info();

		trace!(target: "sync", "Beginning search for common ancestor from {:?}",
			(chain_info.best_block_number, chain_info.best_block_hash));
		*state = SyncState::AncestorSearch(AncestorSearch::begin(chain_info.best_block_number));
	}

	fn maintain_sync(&self, ctx: &BasicContext) {
		const DRAIN_AMOUNT: usize = 128;

		let mut state = self.state.lock();
		debug!(target: "sync", "Maintaining sync ({:?})", &*state);

		// drain any pending blocks into the queue.
		{
			let mut sink = Vec::with_capacity(DRAIN_AMOUNT);

			'a:
			loop {
				let queue_info = self.client.queue_info();
				if queue_info.is_full() { break }

				*state = match mem::replace(&mut *state, SyncState::Idle) {
					SyncState::Rounds(round)
						=> SyncState::Rounds(round.drain(&mut sink, Some(DRAIN_AMOUNT))),
					other => other,
				};

				if sink.is_empty() { break }
				trace!(target: "sync", "Drained {} headers to import", sink.len());

				for header in sink.drain(..) {
					if let Err(e) = self.client.queue_header(header) {
						debug!(target: "sync", "Found bad header ({:?}). Reset to search state.", e);

						self.begin_search(&mut state);
						break 'a;
					}
				}
			}
		}

		// handle state transitions.
		{
			let chain_info = self.client.chain_info();
			let best_td = chain_info.total_difficulty;
			match mem::replace(&mut *state, SyncState::Idle) {
				_ if self.best_seen.lock().as_ref().map_or(true, |&(_, td)| best_td >= td)
					=> *state = SyncState::Idle,
				SyncState::Rounds(SyncRound::Abort(reason, _)) => {
					match reason {
						AbortReason::BadScaffold(bad_peers) => {
							debug!(target: "sync", "Disabling peers responsible for bad scaffold");
							for peer in bad_peers {
								ctx.disable_peer(peer);
							}
						}
						AbortReason::NoResponses => {}
					}

					debug!(target: "sync", "Beginning search after aborted sync round");
					self.begin_search(&mut state);
				}
				SyncState::AncestorSearch(AncestorSearch::FoundCommon(num, hash)) => {
					// TODO: compare to best block and switch to another downloading
					// method when close.
					*state = SyncState::Rounds(SyncRound::begin(num, hash));
				}
				SyncState::AncestorSearch(AncestorSearch::Genesis) => {
					// Same here.
					let g_hash = chain_info.genesis_hash;
					*state = SyncState::Rounds(SyncRound::begin(0, g_hash));
				}
				SyncState::Idle => self.begin_search(&mut state),
				other => *state = other, // restore displaced state.
			}
		}

		// allow dispatching of requests.
		// TODO: maybe wait until the amount of cumulative requests remaining is high enough
		// to avoid pumping the failure rate.
		{
			let peers = self.peers.read();
			let mut peer_ids: Vec<_> = peers.keys().cloned().collect();
			let mut rng = self.rng.lock();

			// naive request dispatcher: just give to any peer which says it will
			// give us responses.
			let dispatcher = move |req: request::Headers| {
				rng.shuffle(&mut peer_ids);

				for peer in &peer_ids {
					if ctx.max_requests(*peer, request::Kind::Headers) >= req.max {
						match ctx.request_from(*peer, request::Request::Headers(req.clone())) {
							Ok(id) => {
								return Some(id)
							}
							Err(e) =>
								trace!(target: "sync", "Error requesting headers from viable peer: {}", e),
						}
					}
				}

				None
			};

			*state = match mem::replace(&mut *state, SyncState::Idle) {
				SyncState::Rounds(round) =>
					SyncState::Rounds(round.dispatch_requests(dispatcher)),
				SyncState::AncestorSearch(search) =>
					SyncState::AncestorSearch(search.dispatch_request(dispatcher)),
				other => other,
			};
		}
	}
}

// public API
impl<L: LightChainClient> LightSync<L> {
	/// Create a new instance of `LightSync`.
	///
	/// This won't do anything until registered as a handler
	/// so it can act on events.
	pub fn new(client: Arc<L>) -> Result<Self, ::std::io::Error> {
		Ok(LightSync {
			best_seen: Mutex::new(None),
			peers: RwLock::new(HashMap::new()),
			client: client,
			rng: Mutex::new(try!(OsRng::new())),
			state: Mutex::new(SyncState::Idle),
		})
	}
}
