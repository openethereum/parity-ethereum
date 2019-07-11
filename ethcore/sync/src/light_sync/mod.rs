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

//! Light client synchronization.
//!
//! This will synchronize the header chain using PIP messages.
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

use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Instant, Duration};

use types::encoded;
use light::client::{AsLightClient, LightChainClient};
use light::net::{
	PeerStatus, Announcement, Handler, BasicContext,
	EventContext, Capabilities, ReqId, Status,
	Error as NetError,
};
use chain::SyncState as ChainSyncState;
use light::request::{self, CompleteHeadersRequest as HeadersRequest};
use network::PeerId;
use ethereum_types::{H256, U256};
use parking_lot::{Mutex, RwLock};
use rand::{rngs::OsRng, seq::SliceRandom};
use futures::sync::mpsc;

use self::sync_round::{AbortReason, SyncRound, ResponseContext};
use api::Notification;

mod response;
mod sync_round;

#[cfg(test)]
mod tests;

// Base value for the header request timeout.
const REQ_TIMEOUT_BASE: Duration = Duration::from_secs(7);
// Additional value for each requested header.
// If we request N headers, then the timeout will be:
//  REQ_TIMEOUT_BASE + N * REQ_TIMEOUT_PER_HEADER
const REQ_TIMEOUT_PER_HEADER: Duration = Duration::from_millis(10);

/// Peer chain info.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ChainInfo {
	head_td: U256,
	head_hash: H256,
	head_num: u64,
}

impl PartialOrd for ChainInfo {
	fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
		self.head_td.partial_cmp(&other.head_td)
	}
}

impl Ord for ChainInfo {
	fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
		self.head_td.cmp(&other.head_td)
	}
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
	Awaiting(ReqId, u64, HeadersRequest), // awaiting response for this request.
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

	fn process_response<L>(self, ctx: &dyn ResponseContext, client: &L) -> AncestorSearch
		where L: AsLightClient
	{
		let client = client.as_light_client();
		let first_num = client.chain_info().first_block_number.unwrap_or(0);
		match self {
			AncestorSearch::Awaiting(id, start, req) => {
				if &id == ctx.req_id() {
					match response::verify(ctx.data(), &req) {
						Ok(headers) => {
							for header in &headers {
								if client.is_known(&header.hash()) {
									debug!(target: "sync", "Found common ancestor with best chain");
									return AncestorSearch::FoundCommon(header.number(), header.hash());
								}

								if header.number() < first_num {
									debug!(target: "sync", "Prehistoric common ancestor with best chain.");
									return AncestorSearch::Prehistoric;
								}
							}

							let probe = start - headers.len() as u64;
							if probe == 0 {
								AncestorSearch::Genesis
							} else {
								AncestorSearch::Queued(probe)
							}
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

	fn requests_abandoned(self, req_ids: &[ReqId]) -> AncestorSearch {
		match self {
			AncestorSearch::Awaiting(id, start, req) => {
				if req_ids.iter().find(|&x| x == &id).is_some() {
					AncestorSearch::Queued(start)
				} else {
					AncestorSearch::Awaiting(id, start, req)
				}
			}
			other => other,
		}
	}

	fn dispatch_request<F>(self, mut dispatcher: F) -> AncestorSearch
		where F: FnMut(HeadersRequest) -> Option<ReqId>
	{
		const BATCH_SIZE: u64 = 64;

		match self {
			AncestorSearch::Queued(start) => {
				let batch_size = ::std::cmp::min(start, BATCH_SIZE);
				trace!(target: "sync", "Requesting {} reverse headers from {} to find common ancestor",
					batch_size, start);

				let req = HeadersRequest {
					start: start.into(),
					max: batch_size,
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

/// A wrapper around the SyncState that makes sure to
/// update the giving reference to `is_idle`
#[derive(Debug)]
struct SyncStateWrapper {
	state: SyncState,
}

impl SyncStateWrapper {
	/// Create a new wrapper for SyncState::Idle
	pub fn idle() -> Self {
		SyncStateWrapper {
			state: SyncState::Idle,
		}
	}

	/// Set the new state's value, making sure `is_idle` gets updated
	pub fn set(&mut self, state: SyncState, is_idle_handle: &mut bool) {
		*is_idle_handle = match state {
			SyncState::Idle => true,
			_ => false,
		};
		self.state = state;
	}

	/// Returns the internal state's value
	pub fn into_inner(self) -> SyncState {
		self.state
	}
}

impl Deref for SyncStateWrapper {
	type Target = SyncState;

	fn deref(&self) -> &SyncState {
		&self.state
	}
}

struct ResponseCtx<'a> {
	peer: PeerId,
	req_id: ReqId,
	ctx: &'a dyn BasicContext,
	data: &'a [encoded::Header],
}

impl<'a> ResponseContext for ResponseCtx<'a> {
	fn responder(&self) -> PeerId { self.peer }
	fn req_id(&self) -> &ReqId { &self.req_id }
	fn data(&self) -> &[encoded::Header] { self.data }
	fn punish_responder(&self) { self.ctx.disable_peer(self.peer) }
}

/// Light client synchronization manager. See module docs for more details.
pub struct LightSync<L: AsLightClient> {
	start_block_number: u64,
	best_seen: Mutex<Option<ChainInfo>>, // best seen block on the network.
	peers: RwLock<HashMap<PeerId, Mutex<Peer>>>, // peers which are relevant to synchronization.
	pending_reqs: Mutex<HashMap<ReqId, PendingReq>>, // requests from this handler
	client: Arc<L>,
	rng: Mutex<OsRng>,
	state: Mutex<SyncStateWrapper>,
	senders: RwLock<Vec<mpsc::UnboundedSender<ChainSyncState>>>,
	// We duplicate this state tracking to avoid deadlocks in `is_major_importing`.
	is_idle: Mutex<bool>,
}

#[derive(Debug, Clone)]
struct PendingReq {
	started: Instant,
	timeout: Duration,
}

impl<L: AsLightClient + Send + Sync> Handler for LightSync<L> {
	fn on_connect(
		&self,
		ctx: &dyn EventContext,
		status: &Status,
		capabilities: &Capabilities
	) -> PeerStatus {
		use std::cmp;

		if capabilities.serve_headers {
			let chain_info = ChainInfo {
				head_td: status.head_td,
				head_hash: status.head_hash,
				head_num: status.head_num,
			};

			{
				let mut best = self.best_seen.lock();
				*best = cmp::max(best.clone(), Some(chain_info.clone()));
			}

			self.peers.write().insert(ctx.peer(), Mutex::new(Peer::new(chain_info)));
			self.maintain_sync(ctx.as_basic());

			PeerStatus::Kept
		} else {
			PeerStatus::Unkept
		}
	}

	fn on_disconnect(&self, ctx: &dyn EventContext, unfulfilled: &[ReqId]) {
		let peer_id = ctx.peer();

		let peer = match self.peers.write().remove(&peer_id).map(|p| p.into_inner()) {
			Some(peer) => peer,
			None => return,
		};

		trace!(target: "sync", "peer {} disconnecting", peer_id);

		let new_best = {
			let mut best = self.best_seen.lock();

			if best.as_ref().map_or(false, |b| b == &peer.status) {
				// search for next-best block.
				let next_best: Option<ChainInfo> = self.peers.read().values()
					.map(|p| p.lock().status.clone())
					.map(Some)
					.fold(None, ::std::cmp::max);

				*best = next_best;
			}

			best.clone()
		};

		{
			let mut pending_reqs = self.pending_reqs.lock();
			for unfulfilled in unfulfilled {
				pending_reqs.remove(&unfulfilled);
			}
		}

		if new_best.is_none() {
			debug!(target: "sync", "No peers remain. Reverting to idle");
			self.set_state(&mut self.state.lock(), SyncState::Idle);
		} else {
			let mut state = self.state.lock();

			let next_state = match mem::replace(&mut *state, SyncStateWrapper::idle()).into_inner() {
				SyncState::Idle => SyncState::Idle,
				SyncState::AncestorSearch(search) =>
					SyncState::AncestorSearch(search.requests_abandoned(unfulfilled)),
				SyncState::Rounds(round) => SyncState::Rounds(round.requests_abandoned(unfulfilled)),
			};
			self.set_state(&mut state, next_state);
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn on_announcement(&self, ctx: &dyn EventContext, announcement: &Announcement) {
		let (last_td, chain_info) = {
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
					(last_td, peer.status.clone())
				}
			}
		};

		trace!(target: "sync", "Announcement from peer {}: new chain head {:?}, reorg depth {}",
			ctx.peer(), (announcement.head_hash, announcement.head_num), announcement.reorg_depth);

		if last_td > announcement.head_td {
			trace!(target: "sync", "Peer {} moved backwards.", ctx.peer());
			self.peers.write().remove(&ctx.peer());
			ctx.disconnect_peer(ctx.peer());
			return
		}

		{
			let mut best = self.best_seen.lock();
			*best = ::std::cmp::max(best.clone(), Some(chain_info));
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn on_responses(&self, ctx: &dyn EventContext, req_id: ReqId, responses: &[request::Response]) {
		let peer = ctx.peer();
		if !self.peers.read().contains_key(&peer) {
			return
		}

		if self.pending_reqs.lock().remove(&req_id).is_none() {
			return
		}

		let headers = match responses.get(0) {
			Some(&request::Response::Headers(ref response)) => &response.headers[..],
			Some(_) => {
				trace!("Disabling peer {} for wrong response type.", peer);
				ctx.disable_peer(peer);
				&[]
			}
			None => &[],
		};

		{
			let mut state = self.state.lock();

			let ctx = ResponseCtx {
				peer: ctx.peer(),
				req_id: req_id,
				ctx: ctx.as_basic(),
				data: headers,
			};

			let next_state = match mem::replace(&mut *state, SyncStateWrapper::idle()).into_inner() {
				SyncState::Idle => SyncState::Idle,
				SyncState::AncestorSearch(search) =>
					SyncState::AncestorSearch(search.process_response(&ctx, &*self.client)),
				SyncState::Rounds(round) => SyncState::Rounds(round.process_response(&ctx)),
			};
			self.set_state(&mut state, next_state);
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn tick(&self, ctx: &dyn BasicContext) {
		self.maintain_sync(ctx);
	}
}

// private helpers
impl<L: AsLightClient> LightSync<L> {
	/// Sets the LightSync's state, and update
	/// `is_idle`
	fn set_state(&self, state: &mut SyncStateWrapper, next_state: SyncState) {

		match next_state {
			SyncState::Idle => self.notify_senders(ChainSyncState::Idle),
			_ => self.notify_senders(ChainSyncState::Blocks)
		};

		state.set(next_state, &mut self.is_idle.lock());
	}

	fn notify_senders(&self, state: ChainSyncState) {
		self.senders.write().retain(|sender| {
			sender.unbounded_send(state).is_ok()
		})
	}

	// Begins a search for the common ancestor and our best block.
	// does not lock state, instead has a mutable reference to it passed.
	fn begin_search(&self, state: &mut SyncStateWrapper) {
		if let None =  *self.best_seen.lock() {
			// no peers.
			self.set_state(state, SyncState::Idle);
			return;
		}

		self.client.as_light_client().flush_queue();
		let chain_info = self.client.as_light_client().chain_info();

		trace!(target: "sync", "Beginning search for common ancestor from {:?}",
			(chain_info.best_block_number, chain_info.best_block_hash));
		let next_state = SyncState::AncestorSearch(AncestorSearch::begin(chain_info.best_block_number));
		self.set_state(state, next_state);
	}

	// handles request dispatch, block import, state machine transitions, and timeouts.
	fn maintain_sync(&self, ctx: &dyn BasicContext) {
		use ethcore::error::{Error as EthcoreError, ImportError};

		const DRAIN_AMOUNT: usize = 128;

		let client = self.client.as_light_client();
		let chain_info = client.chain_info();

		let mut state = self.state.lock();
		debug!(target: "sync", "Maintaining sync ({:?})", **state);

		// drain any pending blocks into the queue.
		{
			let mut sink = Vec::with_capacity(DRAIN_AMOUNT);

			'a:
			loop {
				if client.queue_info().is_full() { break }

				let next_state = match mem::replace(&mut *state, SyncStateWrapper::idle()).into_inner() {
					SyncState::Rounds(round)
						=> SyncState::Rounds(round.drain(&mut sink, Some(DRAIN_AMOUNT))),
					other => other,
				};
				self.set_state(&mut state, next_state);

				if sink.is_empty() { break }
				trace!(target: "sync", "Drained {} headers to import", sink.len());

				for header in sink.drain(..) {
					match client.queue_header(header) {
						Ok(_) => {}
						Err(EthcoreError::Import(ImportError::AlreadyInChain)) => {
							trace!(target: "sync", "Block already in chain. Continuing.");
						},
						Err(EthcoreError::Import(ImportError::AlreadyQueued)) => {
							trace!(target: "sync", "Block already queued. Continuing.");
						},
						Err(e) => {
							debug!(target: "sync", "Found bad header ({:?}). Reset to search state.", e);

							self.begin_search(&mut state);
							break 'a;
						}
					}
				}
			}
		}

		// handle state transitions.
		{
			let best_td = chain_info.pending_total_difficulty;
			let sync_target = match *self.best_seen.lock() {
				Some(ref target) if target.head_td > best_td => (target.head_num, target.head_hash),
				ref other => {
					let network_score = other.as_ref().map(|target| target.head_td);
					trace!(target: "sync", "No target to sync to. Network score: {:?}, Local score: {:?}",
						network_score, best_td);
					self.set_state(&mut state, SyncState::Idle);
					return;
				}
			};

			match mem::replace(&mut *state, SyncStateWrapper::idle()).into_inner() {
				SyncState::Rounds(SyncRound::Abort(reason, remaining)) => {
					if remaining.len() > 0 {
						self.set_state(&mut state, SyncState::Rounds(SyncRound::Abort(reason, remaining)));
						return;
					}

					match reason {
						AbortReason::BadScaffold(bad_peers) => {
							debug!(target: "sync", "Disabling peers responsible for bad scaffold");
							for peer in bad_peers {
								ctx.disable_peer(peer);
							}
						}
						AbortReason::NoResponses => {}
						AbortReason::TargetReached => {
							debug!(target: "sync", "Sync target reached. Going idle");
							self.set_state(&mut state, SyncState::Idle);
							return;
						}
					}

					debug!(target: "sync", "Beginning search after aborted sync round");
					self.begin_search(&mut state);
				}
				SyncState::AncestorSearch(AncestorSearch::FoundCommon(num, hash)) => {
					self.set_state(&mut state, SyncState::Rounds(SyncRound::begin((num, hash), sync_target)));
				}
				SyncState::AncestorSearch(AncestorSearch::Genesis) => {
					// Same here.
					let g_hash = chain_info.genesis_hash;
					self.set_state(&mut state, SyncState::Rounds(SyncRound::begin((0, g_hash), sync_target)));
				}
				SyncState::Idle => self.begin_search(&mut state),
				other => self.set_state(&mut state, other), // restore displaced state.
			}
		}

		// handle requests timeouts
		{
			let mut pending_reqs = self.pending_reqs.lock();
			let mut unfulfilled = Vec::new();
			for (req_id, info) in pending_reqs.iter() {
				if info.started.elapsed() >= info.timeout {
					debug!(target: "sync", "{} timed out", req_id);
					unfulfilled.push(req_id.clone());
				}
			}

			if !unfulfilled.is_empty() {
				for unfulfilled in unfulfilled.iter() {
					pending_reqs.remove(unfulfilled);
				}
				drop(pending_reqs);

				let next_state = match mem::replace(&mut *state, SyncStateWrapper::idle()).into_inner() {
					SyncState::Idle => SyncState::Idle,
					SyncState::AncestorSearch(search) =>
						SyncState::AncestorSearch(search.requests_abandoned(&unfulfilled)),
					SyncState::Rounds(round) => SyncState::Rounds(round.requests_abandoned(&unfulfilled)),
				};
				self.set_state(&mut state, next_state);
			}
		}

		// allow dispatching of requests.
		{
			let peers = self.peers.read();
			let mut peer_ids: Vec<_> = peers.iter().filter_map(|(id, p)| {
				if p.lock().status.head_td > chain_info.pending_total_difficulty {
					Some(*id)
				} else {
					None
				}
			}).collect();

			let mut rng = self.rng.lock();
			let mut requested_from = HashSet::new();

			// naive request dispatcher: just give to any peer which says it will
			// give us responses. but only one request per peer per state transition.
			let dispatcher = move |req: HeadersRequest| {
				peer_ids.shuffle(&mut *rng);

				let request = {
					let mut builder = request::Builder::default();
					builder.push(request::Request::Headers(request::IncompleteHeadersRequest {
						start: req.start.into(),
						skip: req.skip,
						max: req.max,
						reverse: req.reverse,
					})).expect("request provided fully complete with no unresolved back-references; qed");
					builder.build()
				};
				for peer in &peer_ids {
					if requested_from.contains(peer) { continue }
					match ctx.request_from(*peer, request.clone()) {
						Ok(id) => {
							assert!(req.max <= u32::max_value() as u64,
								"requesting more than 2^32 headers at a time would overflow");
							let timeout = REQ_TIMEOUT_BASE + REQ_TIMEOUT_PER_HEADER * req.max as u32;
							self.pending_reqs.lock().insert(id.clone(), PendingReq {
								started: Instant::now(),
								timeout,
							});
							requested_from.insert(peer.clone());

							return Some(id)
						}
						Err(NetError::NoCredits) => {}
						Err(e) =>
							trace!(target: "sync", "Error requesting headers from viable peer: {}", e),
					}
				}

				None
			};

			let next_state = match mem::replace(&mut *state, SyncStateWrapper::idle()).into_inner() {
				SyncState::Rounds(round) =>
					SyncState::Rounds(round.dispatch_requests(dispatcher)),
				SyncState::AncestorSearch(search) =>
					SyncState::AncestorSearch(search.dispatch_request(dispatcher)),
				other => other,
			};
			self.set_state(&mut state, next_state);
		}
	}

	// returns receiving end of futures::mpsc::unbounded channel
	// poll the channel for changes to sync state.
	fn sync_notification(&self) -> Notification<ChainSyncState> {
		let (sender, receiver) = futures::sync::mpsc::unbounded();
		self.senders.write().push(sender);
		receiver
	}
}

// public API
impl<L: AsLightClient> LightSync<L> {
	/// Create a new instance of `LightSync`.
	///
	/// This won't do anything until registered as a handler
	/// so it can act on events.
	pub fn new(client: Arc<L>) -> Result<Self, ::std::io::Error> {
		Ok(LightSync {
			start_block_number: client.as_light_client().chain_info().best_block_number,
			best_seen: Mutex::new(None),
			peers: RwLock::new(HashMap::new()),
			pending_reqs: Mutex::new(HashMap::new()),
			client: client,
			rng: Mutex::new(OsRng::new()?),
			senders: RwLock::new(Vec::new()),
			state: Mutex::new(SyncStateWrapper::idle()),
			is_idle: Mutex::new(true),
		})
	}
}

/// Trait for erasing the type of a light sync object and exposing read-only methods.
pub trait SyncInfo {
	/// Get the highest block advertised on the network.
	fn highest_block(&self) -> Option<u64>;

	/// Get the block number at the time of sync start.
	fn start_block(&self) -> u64;

	/// Whether major sync is underway.
	fn is_major_importing(&self) -> bool;

	/// returns the receieving end of a futures::mpsc unbounded channel
	/// poll the channel for changes to sync state
	fn sync_notification(&self) -> Notification<ChainSyncState>;
}

impl<L: AsLightClient> SyncInfo for LightSync<L> {
	fn highest_block(&self) -> Option<u64> {
		self.best_seen.lock().as_ref().map(|x| x.head_num)
	}

	fn start_block(&self) -> u64 {
		self.start_block_number
	}

	fn is_major_importing(&self) -> bool {
		const EMPTY_QUEUE: usize = 3;

		let queue_info = self.client.as_light_client().queue_info();
		let is_verifying = queue_info.unverified_queue_size + queue_info.verified_queue_size > EMPTY_QUEUE;
		let is_syncing = !*self.is_idle.lock();

		is_verifying || is_syncing
	}

	fn sync_notification(&self) -> Notification<ChainSyncState> {
		self.sync_notification()
	}
}
