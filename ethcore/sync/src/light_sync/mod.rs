// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
//!
//! When `warp_sync` is enabled (by default), a light client will first try to
//! sync using snapshots. If a client fails to get a snapshot manifest within a certain period of time,
//! it will fall back to normal sync.

use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::Arc;
use std::time::{Instant, Duration};
use bytes::Bytes;

use ethcore::encoded;
use super::WarpSync;
use chain;
use light::client::{AsLightClient, LightChainClient};
use light::net::{
	PeerStatus, Announcement, Handler, BasicContext,
	EventContext, Capabilities, ReqId, Status,
	Error as NetError,
};

use hash::keccak;
use light::request::{self, CompleteHeadersRequest as HeadersRequest};
use network::{PeerId};
use ethereum_types::{H256, U256};
use rlp::Rlp;
use ethcore::snapshot::{ManifestData, RestorationStatus, SnapshotService};
use snapshot::ChunkType;
use self::sync_round::{AbortReason, SyncRound, ResponseContext};
use self::warp_sync::{
	SnapshotSyncHandler, SnapshotSyncContext, SnapshotSyncEvent,
	SnapshotManager, WarpSyncError, SnapshotPeerAsking, WarpSyncState,
	GroupedPeers,
};

pub use self::warp_sync::SnapshotSyncLightHandler;

use parking_lot::{Mutex, RwLock};
use rand::{Rng, OsRng};

mod response;
mod sync_round;
mod warp_sync;

#[cfg(test)]
mod tests;

// Base value for the header request timeout.
const REQ_TIMEOUT_BASE: Duration = Duration::from_secs(7);
// Additional value for each requested header.
// If we request N headers, then the timeout will be:
//  REQ_TIMEOUT_BASE + N * REQ_TIMEOUT_PER_HEADER
const REQ_TIMEOUT_PER_HEADER: Duration = Duration::from_millis(10);
// This number is pretty random.
pub(super) const MAX_BLOCK_CHUNKS_DOWNLOAD_AHEAD: usize = 10;

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

	fn process_response<L>(self, ctx: &ResponseContext, client: &L) -> AncestorSearch
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
	// warp sync
	Snapshot(WarpSyncState),
}

struct ResponseCtx<'a> {
	peer: PeerId,
	req_id: ReqId,
	ctx: &'a BasicContext,
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
	state: Mutex<SyncState>,
	snapshot_manager: RwLock<SnapshotManager>,
}

#[derive(Debug, Clone)]
struct PendingReq {
	started: Instant,
	timeout: Duration,
}

impl<L: AsLightClient + Send + Sync> Handler for LightSync<L> {
	fn on_connect(
		&self,
		ctx: &EventContext,
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

	fn on_disconnect(&self, ctx: &EventContext, unfulfilled: &[ReqId]) {
		match *self.state.lock() {
			SyncState::Snapshot(_) => return,
			_ => {},
		};

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
			*self.state.lock() = SyncState::Idle;
		} else {
			let mut state = self.state.lock();

			*state = match mem::replace(&mut *state, SyncState::Idle) {
				SyncState::Idle =>
					SyncState::Idle,
				SyncState::AncestorSearch(search) =>
					SyncState::AncestorSearch(search.requests_abandoned(unfulfilled)),
				SyncState::Rounds(round) =>
					SyncState::Rounds(round.requests_abandoned(unfulfilled)),
				other => other,
			};
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn on_announcement(&self, ctx: &EventContext, announcement: &Announcement) {
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

	fn on_responses(&self, ctx: &EventContext, req_id: ReqId, responses: &[request::Response]) {
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

			*state = match mem::replace(&mut *state, SyncState::Idle) {
				SyncState::Idle =>
					SyncState::Idle,
				SyncState::AncestorSearch(search) =>
					SyncState::AncestorSearch(search.process_response(&ctx, &*self.client)),
				SyncState::Rounds(round) =>
					SyncState::Rounds(round.process_response(&ctx)),
				other => other,
			};
		}

		self.maintain_sync(ctx.as_basic());
	}

	fn tick(&self, ctx: &BasicContext) {
		self.maintain_sync(ctx);
	}
}

// snapshot sync handler methods
impl<L: AsLightClient + Send + Sync> SnapshotSyncHandler for LightSync<L> {
	fn on_connect(&self, event: &SnapshotSyncEvent) {
		match *self.state.lock() {
			SyncState::Snapshot(_) => {}
			_ => {
				return;
			}
		}
		if self.send_warp_sync_status_packet(event) {
			self.maintain_sync_with_snap(event.as_context());
		}
	}

	fn on_disconnect(&self, event: &SnapshotSyncEvent) {
		match *self.state.lock() {
			SyncState::Snapshot(_) => {}
			_ => {
				return;
			}
		}
		trace!(target: "warp", "Disconnected from peer {}", event.peer());
		let asking = self.snapshot_manager.write().on_peer_aborting(&event.peer());
		if let Some(SnapshotPeerAsking::SnapshotManifest) = asking {
			// TODO: if this was the last peer with the same snapshot,
			//       maybe we should restart warp sync from scratch
			self.restart(event.as_context().snapshot_service());
			self.maintain_sync_with_snap(event.as_context());
		}
	}

	fn on_warp_peer_status(&self, event: &SnapshotSyncEvent, rlp: Rlp) -> Result<(), WarpSyncError> {
		let genesis_hash = self.client.as_light_client().chain_info().genesis_hash;
		self.snapshot_manager.write().on_peer_status(event, rlp, genesis_hash)
	}

	fn on_snap_manifest(&self, event: &SnapshotSyncEvent, rlp: Rlp) -> Result<(), WarpSyncError> {
		let peer_id = event.peer();
		if self.snapshot_manager.read().get(&peer_id).is_none() {
			trace!(target: "warp", "Ignoring snapshot manifest from unknown peer {}", peer_id);
			return Ok(());
		}

		let (manifest, manifest_hash) = {
			let is_manifest_state = {
				match *self.state.lock() {
					SyncState::Snapshot(WarpSyncState::Manifest) => true,
					_ => false
				}
			};

			let asked_for_manifest = match self.snapshot_manager.write().reset_peer_asking(&peer_id) {
				Some(SnapshotPeerAsking::SnapshotManifest) => true,
				_ => false,
			};
			if !asked_for_manifest || !is_manifest_state {
				trace!(target: "warp", "{}: Ignored unexpected manifest", peer_id);
				return Ok(());
			}

			let manifest_rlp = rlp.at(0)?;
			match ManifestData::from_rlp(manifest_rlp.as_raw()) {
				Err(e) => {
					trace!(target: "warp", "{}: Ignored bad manifest: {:?}", peer_id, e);
					return Err(WarpSyncError::BadManifest);
				}
				Ok(manifest) => (manifest, keccak(manifest_rlp.as_raw())),
			}
		};

		let is_supported_version = event.as_context().snapshot_service().supported_versions()
			.map_or(false, |(l, h)| manifest.version >= l && manifest.version <= h);

		if !is_supported_version {
			trace!(target: "warp", "{}: Snapshot manifest version not supported: {}", peer_id, manifest.version);
			return Err(WarpSyncError::UnsupportedManifestVersion(manifest.version));
		}

		trace!(target: "warp", "Received a manifest ({}) from {}", manifest.block_hash, peer_id);

		self.snapshot_manager.write().reset_manifest_to(&manifest, &manifest_hash);
		event.as_context().snapshot_service().begin_restore(manifest);
		*self.state.lock() = SyncState::Snapshot(WarpSyncState::Blocks);

		self.maintain_sync_with_snap(event.as_context());

		Ok(())
	}

	fn on_snap_data(&self, event: &SnapshotSyncEvent, rlp: Rlp) -> Result<(), WarpSyncError> {
		let peer_id = event.peer();
		if self.snapshot_manager.read().get(&peer_id).is_none() {
			trace!(target: "warp", "Ignoring snapshot chunk from unknown peer {}", peer_id);
			return Ok(());
		}

		self.snapshot_manager.write().clear_peer_download(&peer_id);

		let (chunk, hash) = {
			let is_blocks_state = {
				match *self.state.lock() {
					SyncState::Snapshot(WarpSyncState::Blocks) => true,
					_ => false
				}
			};

			// always lock the snapshot manager before the state
			let mut snap = self.snapshot_manager.write();

			let asking = snap.reset_peer_asking(&peer_id);
			let asked_for_snapshot_data = match asking {
				Some(SnapshotPeerAsking::SnapshotData(_)) => true,
				_ => false,
			};

			if !asked_for_snapshot_data || !is_blocks_state {
				trace!(target: "warp", "Peer {}: Ignored unexpected snapshot chunk", peer_id);
				return Ok(());
			}

			let status = event.as_context().snapshot_service().status();
			match status {
				RestorationStatus::Inactive | RestorationStatus::Failed => {
					trace!(target: "warp", "Snapshot restoration aborted");
					*self.state.lock() = SyncState::Snapshot(WarpSyncState::WaitingPeers);

					// only note bad if restoration failed.
					if let (Some(hash), RestorationStatus::Failed) = (snap.snapshot_hash(), status) {
						trace!(target: "warp", "Noting snapshot hash {} as bad", hash);
						snap.note_bad(hash);
					}

					snap.clear();
					return Ok(());
				},
				RestorationStatus::Initializing { .. } => {
					trace!(target: "warp", "{}: Snapshot restoration is initializing", peer_id);
					return Ok(());
				},
				RestorationStatus::Ongoing { .. } => {
					trace!(target: "warp", "{}: Snapshot restoration is ongoing", peer_id);
				},
			}

			let snapshot_data: Bytes = rlp.val_at(0)?;
			match snap.validate_chunk(&snapshot_data) {
				Ok(ChunkType::Block(hash)) => {
					match asking {
						Some(SnapshotPeerAsking::SnapshotData(h)) if h != hash => {
							trace!(target: "warp", "{}: Asked for a different block chunk", peer_id);
						},
						_ => {},
					}
					(snapshot_data, hash)
				}
				_ => {
					trace!(target: "warp", "{}: Got a state or a bad block chunk on light client", peer_id);
					return Err(WarpSyncError::BadBlockChunk);
				}
			}
		};

		trace!(target: "warp", "{}: Processing block chunk", peer_id);
		event.as_context().snapshot_service().restore_block_chunk(hash, chunk);

		if self.snapshot_manager.read().is_complete() {
			// wait for snapshot restoration process to complete
			*self.state.lock() = SyncState::Snapshot(WarpSyncState::WaitingService);
		}

		Ok(())
	}

	fn on_tick(&self, ctx: &SnapshotSyncContext) {
		self.maintain_sync_with_snap(ctx);
		self.snapshot_manager.write().disconnect_slowpokes(ctx);
	}
}

// private warp sync helpers
impl<L: AsLightClient> LightSync<L> {

	fn get_init_state(client: &LightChainClient, warp_sync: WarpSync) -> SyncState {
		let best_block = client.chain_info().best_block_number;
		let waiting_peers = SyncState::Snapshot(WarpSyncState::WaitingPeers);
		match warp_sync {
			WarpSync::Enabled => waiting_peers,
			WarpSync::OnlyAndAfter(block) if block > best_block => waiting_peers,
			_ => SyncState::Idle,
		}
	}

	fn maintain_sync_with_snap(&self, ctx: &SnapshotSyncContext) {
		if !self.warp_sync_enabled(ctx.snapshot_service()) {
			trace!(target: "warp", "Skipping warp sync. Disabled or not supported.");
			return;
		}

		let our_best_block = self.client.as_light_client().chain_info().best_block_number;
		let best_seen = SyncInfo::highest_block(self);
		let peers = self.snapshot_manager.read().best_peer_group(our_best_block, best_seen);

		self.maybe_start_snapshot_sync(ctx, &peers);

		match *self.state.lock() {
			SyncState::Snapshot(_) => {}
			_ => {
				return;
			}
		}

		let old_state = match *self.state.lock() {
			SyncState::Snapshot(s) => s,
			_ => { return; }
		};
		match old_state {
			WarpSyncState::WaitingService => {
				match ctx.snapshot_service().status() {
					RestorationStatus::Initializing { .. } => {
						trace!(target: "warp", "Snapshot restoration is initializing");
						return;
					},
					RestorationStatus::Inactive => {
						trace!(target: "warp", "Snapshot restoration is complete");
						self.restart(ctx.snapshot_service());
						return;
					},
					RestorationStatus::Ongoing { block_chunks_done, .. } => {
						// Initialize the snapshot if not already done
						self.snapshot_manager.write().initialize(ctx.snapshot_service());
						let left_chunks = self.snapshot_manager.read()
							.done_chunks()
							.saturating_sub(block_chunks_done as usize);
						if !self.snapshot_manager.read().is_complete() &&
							left_chunks <= MAX_BLOCK_CHUNKS_DOWNLOAD_AHEAD
						{
							trace!(target: "warp", "Resuming snapshot sync");
							*self.state.lock() = SyncState::Snapshot(WarpSyncState::Blocks);
						}
					},
					RestorationStatus::Failed => {
						trace!(target: "warp", "Snapshot restoration aborted");
						self.snapshot_manager.write().clear();
						*self.state.lock() = SyncState::Snapshot(WarpSyncState::WaitingPeers);
					},
				}
				self.continue_warp_sync(ctx, &peers);
			},
			WarpSyncState::Blocks => {
				self.continue_warp_sync(ctx, &peers);
			}
			_ => {},
		};
	}

	fn restart(&self, service: &SnapshotService) {
		trace!(target: "sync", "Restarting");
		if let SyncState::Snapshot(WarpSyncState::Blocks) = *self.state.lock() {
			debug!(target: "warp", "Aborting snapshot restore");
			service.abort_restore();
		}
		self.snapshot_manager.write().clear();
		let warp_sync = self.snapshot_manager.read().warp_sync();
		let init_state = Self::get_init_state(self.client.as_light_client(), warp_sync);
		*self.state.lock() = init_state;
	}

	fn continue_warp_sync(&self, ctx: &SnapshotSyncContext, peers: &Option<GroupedPeers>) {
		let old_state = match *self.state.lock() {
			SyncState::Snapshot(s) => s,
			_ => { return; }
		};
		match old_state {
			WarpSyncState::Blocks => {
				match ctx.snapshot_service().status() {
					RestorationStatus::Initializing { .. } => {
						self.snapshot_manager.write().initialize(ctx.snapshot_service());
						trace!(target: "warp", "Snapshot service is initializing, pausing sync");
						*self.state.lock() = SyncState::Snapshot(WarpSyncState::WaitingService);
						return;
					}
					RestorationStatus::Ongoing { block_chunks_done, .. } => {
						// Initialize the snapshot if not already done
						self.snapshot_manager.write().initialize(ctx.snapshot_service());
						let processed_chunks = self.snapshot_manager.read()
							.done_chunks()
							.saturating_sub(block_chunks_done as usize);
						if processed_chunks > MAX_BLOCK_CHUNKS_DOWNLOAD_AHEAD {
							trace!(target: "warp", "Snapshot queue full, pausing sync");
							*self.state.lock() = SyncState::Snapshot(WarpSyncState::WaitingService);
							return;
						}
					}
					RestorationStatus::Failed => {
						trace!(target: "warp", "Snapshot restoration aborted");
						self.snapshot_manager.write().clear();
						*self.state.lock() = SyncState::Snapshot(WarpSyncState::WaitingPeers);
						return;
					}
					s => {
						trace!(target: "warp", "Downloading chunks, but snapshot service state is {:?}", s);
					}
				}
				if let Some(peers) = peers {
					self.snapshot_manager.write().request_snapshot_blocks(ctx, &peers.peers);
				} else {
					debug!(target: "warp", "No peers to download snapshot blocks from");
				}
			},
			_ => {},
		}
	}

	fn warp_sync_enabled(&self, service: &SnapshotService) -> bool {
		let warp_sync = self.snapshot_manager.read().warp_sync();
		warp_sync.is_enabled() && service.supported_versions().is_some()
	}

	fn maybe_start_snapshot_sync(&self, ctx: &SnapshotSyncContext, peers: &Option<GroupedPeers>) {
		match *self.state.lock() {
			SyncState::Snapshot(WarpSyncState::WaitingPeers) => {}
			_ => {
				return;
			}
		}

		let has_manifest = self.snapshot_manager.read().has_manifest();
		let timeout = self.snapshot_manager.read().timeout();

		if !has_manifest {
			let requested = match *peers {
				Some(ref p) => {
					if p.peers.len() >= chain::SNAPSHOT_MIN_PEERS || timeout {
						self.maybe_request_manifest(ctx, &p.peers)
					} else {
						false
					}
				},
				None => false,
			};

			if !requested {
				trace!(target: "warp", "No appropriate snapshots found");
			} else {
				return;
			}
		}

		let warp_sync = self.snapshot_manager.read().warp_sync();

		let timeout = match *self.state.lock() {
			SyncState::Snapshot(WarpSyncState::WaitingPeers) if timeout => true,
			_ => false,
		};

		if timeout && !warp_sync.is_warp_only() {
			trace!(target: "warp", "No snapshots found, starting header sync");
			*self.state.lock() = SyncState::Idle;
		}
	}

	fn maybe_request_manifest(&self, ctx: &SnapshotSyncContext, peers: &[PeerId]) -> bool {
		let peer = self.snapshot_manager.write().request_manifest(ctx, peers);
		if let Some(id) = peer {
			*self.state.lock() = SyncState::Snapshot(WarpSyncState::Manifest);
			trace!(target: "warp", "Requested a snapshot manifest from peer {}", id);
		}
		peer.is_some()
	}

	fn send_warp_sync_status_packet(&self, event: &SnapshotSyncEvent) -> bool {
		let protocol = event.as_context().protocol_version(event.peer()).unwrap_or(0);
		let is_warp_protocol = protocol != 0;
		if !is_warp_protocol {
			trace!(target: "warp", "Peer {} doesn't support warp protocol", event.peer());
			return false;
		}

		trace!(target: "warp", "Sending status to {}", event.peer());

		let network_id = event.as_context().network_id();
		let chain_info = self.client.as_light_client().chain_info();

		let manifest_hash = H256::new();
		let manifest_number: u64 = 0;

		let packet = chain::ChainSync::status_packet(
			protocol as u32,
			network_id,
			&chain_info,
			Some(manifest_hash),
			Some(manifest_number),
		);

		event.as_context().send(event.peer(), chain::STATUS_PACKET, packet);
		true
	}
}

// private helpers
impl<L: AsLightClient> LightSync<L> {
	// Begins a search for the common ancestor and our best block.
	// does not lock state, instead has a mutable reference to it passed.
	fn begin_search(&self, state: &mut SyncState) {
		if let None =  *self.best_seen.lock() {
			// no peers.
			*state = SyncState::Idle;
			return;
		}

		self.client.as_light_client().flush_queue();
		let chain_info = self.client.as_light_client().chain_info();

		trace!(target: "sync", "Beginning search for common ancestor from {:?}",
			   (chain_info.best_block_number, chain_info.best_block_hash));
		*state = SyncState::AncestorSearch(AncestorSearch::begin(chain_info.best_block_number));
	}

	// handles request dispatch, block import, state machine transitions, and timeouts.
	fn maintain_sync(&self, ctx: &BasicContext) {
		use ethcore::error::{BlockImportError, BlockImportErrorKind, ImportErrorKind};

		const DRAIN_AMOUNT: usize = 128;

		let client = self.client.as_light_client();
		let chain_info = client.chain_info();

		let mut state = self.state.lock();

		// skip normal sync if we're warp syncing
		match *state {
			SyncState::Snapshot(s) => {
				trace!(target: "sync", "Skipping non-warp sync. State: {:?}", s);
				return;
			},
			_ => {},
		};

		debug!(target: "sync", "Maintaining sync ({:?})", &*state);

		// drain any pending blocks into the queue.
		{
			let mut sink = Vec::with_capacity(DRAIN_AMOUNT);

			'a:
			loop {
				if client.queue_info().is_full() { break }

				*state = match mem::replace(&mut *state, SyncState::Idle) {
					SyncState::Rounds(round)
						=> SyncState::Rounds(round.drain(&mut sink, Some(DRAIN_AMOUNT))),
					other => other,
				};

				if sink.is_empty() { break }
				trace!(target: "sync", "Drained {} headers to import", sink.len());

				for header in sink.drain(..) {
					match client.queue_header(header) {
						Ok(_) => {}
						Err(BlockImportError(BlockImportErrorKind::Import(ImportErrorKind::AlreadyInChain), _)) => {
							trace!(target: "sync", "Block already in chain. Continuing.");
						},
						Err(BlockImportError(BlockImportErrorKind::Import(ImportErrorKind::AlreadyQueued), _)) => {
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
					*state = SyncState::Idle;
					return;
				}
			};

			match mem::replace(&mut *state, SyncState::Idle) {
				SyncState::Rounds(SyncRound::Abort(reason, remaining)) => {
					if remaining.len() > 0 {
						*state = SyncState::Rounds(SyncRound::Abort(reason, remaining));
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
							*state = SyncState::Idle;
							return;
						}
					}

					debug!(target: "sync", "Beginning search after aborted sync round");
					self.begin_search(&mut state);
				}
				SyncState::AncestorSearch(AncestorSearch::FoundCommon(num, hash)) => {
					*state = SyncState::Rounds(SyncRound::begin((num, hash), sync_target));
				}
				SyncState::AncestorSearch(AncestorSearch::Genesis) => {
					// Same here.
					let g_hash = chain_info.genesis_hash;
					*state = SyncState::Rounds(SyncRound::begin((0, g_hash), sync_target));
				}
				SyncState::Idle => self.begin_search(&mut state),
				other => *state = other, // restore displaced state.
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

				*state = match mem::replace(&mut *state, SyncState::Idle) {
					SyncState::Idle =>
						SyncState::Idle,
					SyncState::AncestorSearch(search) =>
						SyncState::AncestorSearch(search.requests_abandoned(&unfulfilled)),
					SyncState::Rounds(round) =>
						SyncState::Rounds(round.requests_abandoned(&unfulfilled)),
					other => other,
				};
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
				rng.shuffle(&mut peer_ids);

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
impl<L: AsLightClient> LightSync<L> {
	/// Create a new instance of `LightSync`.
	///
	/// This won't do anything until registered as a handler
	/// so it can act on events.
	pub fn new(client: Arc<L>, warp_sync: WarpSync) -> Result<Self, ::std::io::Error> {
		let best_block = client.as_light_client().chain_info().best_block_number;
		let state = Self::get_init_state(client.as_light_client(), warp_sync);
		Ok(LightSync {
			start_block_number: best_block,
			best_seen: Mutex::new(None),
			peers: RwLock::new(HashMap::new()),
			pending_reqs: Mutex::new(HashMap::new()),
			client: client,
			rng: Mutex::new(OsRng::new()?),
			state: Mutex::new(state),
			snapshot_manager: RwLock::new(SnapshotManager::new(warp_sync)),
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

	/// Whether warp sync is underway.
	fn is_snapshot_syncing(&self) -> bool;

	/// Count the number of connected peers with snapshots.
	fn connected_snapshot_peers(&self) -> usize;
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

		if self.client.as_light_client().queue_info().unverified_queue_size > EMPTY_QUEUE {
			return true;
		}

		match *self.state.lock() {
			SyncState::Idle => false,
			_ => true,
		}
	}

	fn is_snapshot_syncing(&self) -> bool {
		match *self.state.lock() {
			SyncState::Snapshot(_) => true,
			_ => false,
		}
	}

	fn connected_snapshot_peers(&self) -> usize {
		self.snapshot_manager.read().peers_count()
	}
}
