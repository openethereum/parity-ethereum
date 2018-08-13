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

//! Warp sync implementation for light client.

use ethcore::snapshot::SnapshotService;
use ethcore::header::BlockNumber;
use super::WarpSync;
use chain;
use snapshot::{Snapshot, ChunkType};
use ethereum_types::{H256, U256};
use io::TimerToken;
use itertools::Itertools;
use network::{self, PeerId, PacketId, NetworkProtocolHandler, NetworkContext};
use light::net::Punishment;
use std::mem;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use rlp::{self, Rlp, RlpStream};
use api::WARP_SYNC_PROTOCOL_ID;
use ethcore::snapshot::ManifestData;

const TICK_TIMEOUT: TimerToken = 0;
const TICK_TIMEOUT_INTERVAL: Duration = Duration::from_secs(5);
const WAIT_PEERS_TIMEOUT: Duration = Duration::from_secs(15);

/// Snapshot sync network handler for light client.
pub trait SnapshotSyncHandler: Send + Sync {
	/// Action on a new peer is connected.
	fn on_connect(&self, event: &SnapshotSyncEvent);
	/// Action on a previously connected peer disconnects.
	fn on_disconnect(&self, event: &SnapshotSyncEvent);
	/// Action on status packet received.
	fn on_warp_peer_status(&self, event: &SnapshotSyncEvent, rlp: Rlp) -> Result<(), WarpSyncError>;
	/// Action on snapshot manifest received.
	fn on_snap_manifest(&self, event: &SnapshotSyncEvent, rlp: Rlp) -> Result<(), WarpSyncError>;
	/// Action on snapshot block chunk received.
	fn on_snap_data(&self, event: &SnapshotSyncEvent, rlp: Rlp) -> Result<(), WarpSyncError>;
	/// On tick handler.
	fn on_tick(&self, ctx: &SnapshotSyncContext);
}

/// Snapshot sync context.
pub trait SnapshotSyncContext {
	/// Send a warp sync request to a specific peer.
	fn send(&self, peer: PeerId, packet_id: u8, packet_body: Vec<u8>);
	/// Disconnect peer. Reconnect can be attempted later.
	fn disconnect_peer(&self, peer: PeerId);
	/// Disconnect a peer and prevent it from connecting again.
	fn disable_peer(&self, peer: PeerId);
	/// Return max version for the warp sync protocol.
	fn protocol_version(&self, peer: PeerId) -> Option<u8>;
	/// Return a reference to a snapshot service.
	fn snapshot_service(&self) -> &SnapshotService;
	/// Get network id.
	fn network_id(&self) -> u64;
}

/// Snapshot sync event context.
pub trait SnapshotSyncEvent {
	/// Return event's peer id.
	fn peer(&self) -> PeerId;
	/// Treat the event context as a context.
	fn as_context(&self) -> &SnapshotSyncContext;
}

/// Light client warp sync network protocol handler.
pub struct SnapshotSyncLightHandler {
	network_id: u64,
	sync: Arc<SnapshotSyncHandler>,
	snapshot_service: Arc<SnapshotService>,
}

impl SnapshotSyncLightHandler {
	/// Creates a new instance of `SnapshotSyncLightHandler`.
	pub fn new(network_id: u64, handler: Arc<SnapshotSyncHandler>, service: Arc<SnapshotService>) -> Self {
		Self {
			network_id: network_id,
			sync: handler,
			snapshot_service: service,
		}
	}
}

/// `SnapshotSyncContext` implementation.
struct SyncCtx<'a> {
	network: &'a NetworkContext,
	network_id: u64,
	snapshot_service: &'a SnapshotService,
}
/// `SnapshotSyncEvent` implementation.
struct SyncEvent<'a> {
	context: SyncCtx<'a>,
	peer_id: PeerId,
}


impl<'a> SyncCtx<'a> {
	/// Creates a new instance of `SyncCtx`.
	pub fn new(
		network: &'a NetworkContext,
		network_id: u64,
		snapshot_service: &'a SnapshotService,
	) -> Self {
		Self {
			network: network,
			network_id: network_id,
			snapshot_service: snapshot_service,
		}
	}
}

impl<'a> SyncEvent<'a> {
	/// Creates a new instance of `SyncEvent`.
	pub fn new(
		network: &'a NetworkContext,
		network_id: u64,
		snapshot_service: &'a SnapshotService,
		peer_id: PeerId,
	) -> Self {
		Self {
			context: SyncCtx::new(network, network_id, snapshot_service),
			peer_id: peer_id,
		}
	}
}

impl<'a> SnapshotSyncContext for SyncCtx<'a> {
	fn send(&self, peer: PeerId, packet_id: u8, packet_body: Vec<u8>) {
		if let Err(e) = self.network.send_protocol(WARP_SYNC_PROTOCOL_ID, peer, packet_id, packet_body) {
			debug!(target: "warp", "Error sending sync packet to peer {}: {}", peer, e);
		}
	}

	fn disconnect_peer(&self, peer: PeerId) {
		trace!(target: "warp", "Initiating disconnect of peer {}", peer);
		self.network.disconnect_peer(peer);
	}

	fn disable_peer(&self, peer: PeerId) {
		trace!(target: "warp", "Initiating disable of peer {}", peer);
		self.network.disable_peer(peer);
	}

	fn protocol_version(&self, peer: PeerId) -> Option<u8> {
		self.network.protocol_version(WARP_SYNC_PROTOCOL_ID, peer)
	}

	fn snapshot_service(&self) -> &SnapshotService {
		self.snapshot_service
	}

	fn network_id(&self) -> u64 {
		self.network_id
	}
}

impl<'a> SnapshotSyncEvent for SyncEvent<'a> {
	fn peer(&self) -> PeerId {
		self.peer_id
	}

	fn as_context(&self) -> &SnapshotSyncContext {
		&self.context
	}
}

fn send_empty_packet(io: &NetworkContext, peer: &PeerId, packet_id: PacketId) -> Result<(), WarpSyncError> {
	let packet = RlpStream::new_list(0).out();
	io.send_protocol(WARP_SYNC_PROTOCOL_ID, *peer, packet_id, packet).map_err(|e| e.into())
}

impl NetworkProtocolHandler for SnapshotSyncLightHandler {
	fn initialize(&self, io: &NetworkContext) {
		io.register_timer(TICK_TIMEOUT, TICK_TIMEOUT_INTERVAL)
			.expect("Error registering sync timer for a light client.");
	}

	fn read(&self, io: &NetworkContext, peer: &PeerId, packet_id: u8, data: &[u8]) {
		let event = SyncEvent::new(io, self.network_id, &*self.snapshot_service, *peer);

		let rlp = Rlp::new(data);

		let result = match packet_id {
			chain::STATUS_PACKET => self.sync.on_warp_peer_status(&event, rlp),
			chain::SNAPSHOT_MANIFEST_PACKET => self.sync.on_snap_manifest(&event, rlp),
			chain::SNAPSHOT_DATA_PACKET => self.sync.on_snap_data(&event, rlp),
			chain::GET_SNAPSHOT_MANIFEST_PACKET => send_empty_packet(io, peer, chain::SNAPSHOT_MANIFEST_PACKET),
			chain::GET_SNAPSHOT_DATA_PACKET => send_empty_packet(io, peer, chain::SNAPSHOT_DATA_PACKET),
			chain::GET_BLOCK_HEADERS_PACKET => send_empty_packet(io, peer, chain::BLOCK_HEADERS_PACKET),
			other => {
				trace!(target: "warp", "Unrecognized packet {} from peer {}", other, peer);
				Err(WarpSyncError::UnrecognizedPacket(other))
			},
		};

		if let Err(e) = result {
			punish(&event, e);
		}
	}

	fn connected(&self, io: &NetworkContext, peer: &PeerId) {
		let protocol = io.protocol_version(WARP_SYNC_PROTOCOL_ID, *peer).unwrap_or(0);
		if protocol == 0 {
			trace!(target: "warp", "Connected to a peer that doesn't support warp protocol: {}", peer);
			return;
		}
		let event = SyncEvent::new(io, self.network_id, &*self.snapshot_service, *peer);
		self.sync.on_connect(&event);
	}

	fn disconnected(&self, io: &NetworkContext, peer: &PeerId) {
		let event = SyncEvent::new(io, self.network_id, &*self.snapshot_service, *peer);
		self.sync.on_disconnect(&event);
	}

	fn timeout(&self, io: &NetworkContext, timer: TimerToken) {
		assert_eq!(timer, TICK_TIMEOUT, "warp: unexpected timer token {}", timer);
		let context = SyncCtx::new(io, self.network_id, &*self.snapshot_service);
		self.sync.on_tick(&context);
	}
}

fn punish(event: &SnapshotSyncEvent, e: WarpSyncError) {
	match e.punishment() {
		Punishment::None => {}
		Punishment::Disconnect => {
			trace!(target: "warp", "Disconnecting peer {}: {:?}", event.peer(), e);
			event.as_context().disconnect_peer(event.peer())
		}
		Punishment::Disable => {
			trace!(target: "warp", "Disabling peer {}: {:?}", event.peer(), e);
			event.as_context().disable_peer(event.peer())
		}
	}
}

/// Snapshot download state machine.
#[derive(Debug)]
pub struct SnapshotManager {
	snapshot: Snapshot,
	peers: HashMap<PeerId, SnapshotPeer>,
	warp_sync: WarpSync,
	sync_start_time: Option<Instant>,
}

/// Data type requested from a peer with a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotPeerAsking {
	/// Asking peer for snapshot block chunk with given hash.
	SnapshotData(H256),
	/// Asking peer for snapshot manifest.
	SnapshotManifest,
}

/// Peer with snapshot info.
#[derive(Debug, Clone)]
pub struct SnapshotPeer {
	asking: Option<SnapshotPeerAsking>,
	/// Request timestamp.
	ask_time: Instant,
	/// Best snapshot hash.
	snapshot_hash: H256,
	/// Best snapshot block number.
	snapshot_number: BlockNumber,
}

impl SnapshotPeer {
	/// Create a new instance of `SnapshotPeer`
	pub fn new(snapshot_hash: H256, snapshot_number: BlockNumber) -> Self {
		Self {
			asking: None,
			ask_time: Instant::now(),
			snapshot_hash: snapshot_hash,
			snapshot_number: snapshot_number,
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct GroupedPeers {
	pub snapshot_hash: H256,
	pub peers: Vec<PeerId>,
}

impl SnapshotManager {
	/// Create a new instance of `SnapshotManager`.
	pub fn new(warp_sync: WarpSync) -> Self {
		Self {
			snapshot: Snapshot::new_light(),
			peers: HashMap::new(),
			warp_sync: warp_sync,
			sync_start_time: None,
		}
	}

	pub fn on_peer_aborting(&mut self, peer_id: &PeerId) -> Option<SnapshotPeerAsking> {
		self.clear_peer_download(&peer_id);
		self.peers.remove(&peer_id).and_then(|p| p.asking)
	}

	pub fn clear_peer_download(&mut self, peer: &PeerId) {
		let asking = self.peers.get(peer).and_then(|p| p.asking.clone());
		if let Some(SnapshotPeerAsking::SnapshotData(hash)) = asking {
			self.snapshot.clear_chunk_download(&hash);
		}
	}

	/// Reset peer asking status after request is complete.
	/// Returns the old asking status.
	pub fn reset_peer_asking(&mut self, peer: &PeerId) -> Option<SnapshotPeerAsking> {
		self.reset_peer_asking_to(peer, None)
	}

	fn reset_peer_asking_to(&mut self, peer: &PeerId, to: Option<SnapshotPeerAsking>) -> Option<SnapshotPeerAsking> {
		self.peers
			.get_mut(peer)
			.and_then(|p| mem::replace(&mut p.asking, to))
	}

	/// Count the number of peers.
	pub fn peers_count(&self) -> usize {
		self.peers.len()
	}

	/// Get peer by id.
	pub fn get(&self, peer: &PeerId) -> Option<&SnapshotPeer> {
		self.peers.get(peer)
	}

	/// Returns the largest group of peers with the same snapshot hash.
	/// If we already have a manifest, prefer peers with that manifest.
	/// If there are several largest groups, choose the one with the highest snapshot number.
	pub fn best_peer_group(&self, best_block: BlockNumber, best_seen: Option<u64>) -> Option<GroupedPeers> {
		self.grouped_peers_max_by_key(|p| {
			let snapshot_number = p.peers
				.first()
				.and_then(|id| self.peers.get(id).map(|p| p.snapshot_number))
				.unwrap_or(0);

			let same_snapshot = Some(p.snapshot_hash) == self.snapshot.snapshot_hash();
			(same_snapshot, p.peers.len(), snapshot_number)
		}, best_block, best_seen)
	}

	/// Returns peers grouped by snapshot hash maximizing the `key`.
	fn grouped_peers_max_by_key<B, K>(
		&self,
		key: K,
		best_block: BlockNumber,
		best_seen: Option<u64>,
	) -> Option<GroupedPeers>
		where
			B: Ord,
			K: FnMut(&GroupedPeers) -> B
	{
		let expected_warp_block = match self.warp_sync {
			WarpSync::OnlyAndAfter(block) => block,
			_ => 0,
		};

		self.peers
			.iter()
			.filter(|&(_, p)|
					p.snapshot_number > expected_warp_block &&
					best_block < p.snapshot_number &&
					(p.snapshot_number - best_block) > chain::SNAPSHOT_RESTORE_THRESHOLD &&
					best_seen.map_or(true, |highest|
									 highest >= p.snapshot_number &&
									 (highest - p.snapshot_number) <= chain::SNAPSHOT_RESTORE_THRESHOLD)
			)
			.map(|(id, p)| (p.snapshot_hash.clone(), id))
			.filter(|&(ref hash, _)| !self.snapshot.is_known_bad(hash))
			.sorted_by_key(|&(hash, _)| hash)
			.into_iter()
			.group_by(|&(hash, _)| hash)
			.into_iter()
			.map(|(hash, peers)| GroupedPeers {
				snapshot_hash: hash,
				peers: peers.map(|(_, id)| *id).collect(),
			})
			.max_by_key(key)
	}

	pub fn initialize(&mut self, service: &SnapshotService) {
		self.snapshot.initialize(service);
	}

	pub fn request_manifest(&mut self, ctx: &SnapshotSyncContext, peers: &[PeerId]) -> Option<PeerId> {
		if self.snapshot.have_manifest() {
			return None;
		}
		for peer_id in peers {
			if let Some(ref mut peer) = self.peers.get_mut(peer_id) {
				if peer.asking.is_some() {
					continue;
				}
				peer.asking = Some(SnapshotPeerAsking::SnapshotManifest);
				peer.ask_time = Instant::now();

				trace!(target: "warp", "Requesting a snapshot manifest from {}", peer_id);
				let rlp = RlpStream::new_list(0);

				ctx.send(*peer_id, chain::GET_SNAPSHOT_MANIFEST_PACKET, rlp.out());
				return Some(*peer_id);
			}
		}
		None
	}

	pub fn request_snapshot_blocks(&mut self, ctx: &SnapshotSyncContext, peers: &[PeerId]) {
		use super::MAX_BLOCK_CHUNKS_DOWNLOAD_AHEAD;

		let snapshot_ref = &mut self.snapshot;
		let mut needed_chunks = (0..MAX_BLOCK_CHUNKS_DOWNLOAD_AHEAD)
			.into_iter()
			.filter_map(|_| snapshot_ref.needed_chunk());

		for peer_id in peers.iter() {
			let mut maybe_peer = self.peers.get_mut(peer_id);
			let peer = match maybe_peer {
				Some(ref mut peer) if peer.asking.is_none() => peer,
				_ => continue,
			};
			let hash = match needed_chunks.next() {
				Some(hash) => hash,
				None => return,
			};
			peer.asking = Some(SnapshotPeerAsking::SnapshotData(hash));
			peer.ask_time = Instant::now();

			trace!(target: "warp", "Requesting a snapshot chunk {:?} from peer {}", &hash, peer_id);
			let mut rlp = RlpStream::new_list(1);
			rlp.append(&hash);

			ctx.send(*peer_id, chain::GET_SNAPSHOT_DATA_PACKET, rlp.out());
		}
	}

	pub fn disconnect_slowpokes(&mut self, ctx: &SnapshotSyncContext) {
		let now = Instant::now();
		let aborting: Vec<PeerId> = self.peers
			.iter()
			.filter_map(|(id, peer)| {
				let elapsed = now - peer.ask_time;
				let timeout = match peer.asking {
					None => false,
					Some(SnapshotPeerAsking::SnapshotData(_)) => elapsed > chain::SNAPSHOT_DATA_TIMEOUT,
					Some(SnapshotPeerAsking::SnapshotManifest) => elapsed > chain::SNAPSHOT_MANIFEST_TIMEOUT,
				};
				match timeout {
					true => Some(*id),
					false => None,
				}
			})
			.collect();
		for peer_id in &aborting {
			trace!(target: "warp", "Timeout {}", peer_id);
			ctx.disconnect_peer(*peer_id);
			self.on_peer_aborting(peer_id);
		}
	}

	pub fn timeout(&self) -> bool {
		self.sync_start_time.map_or(false, |t| t.elapsed() > WAIT_PEERS_TIMEOUT)
	}

	/// Get the warp sync settings.
	pub fn warp_sync(&self) -> WarpSync {
		self.warp_sync
	}

	/// Clear snapshot state.
	pub fn clear(&mut self) {
		self.snapshot.clear();
	}

	pub fn has_manifest(&self) -> bool {
		return self.snapshot.have_manifest()
	}

	pub fn is_complete(&self) -> bool {
		self.snapshot.is_complete()
	}

	pub fn done_chunks(&self) -> usize {
		self.snapshot.done_chunks()
	}

	/// Validate chunk and mark it as downloaded.
	pub fn validate_chunk(&mut self, chunk: &[u8]) -> Result<ChunkType, ()> {
		self.snapshot.validate_chunk(chunk)
	}

	/// Get the snapshot hash.
	pub fn snapshot_hash(&self) -> Option<H256> {
		self.snapshot.snapshot_hash()
	}

	/// Note snapshot hash as bad.
	pub fn note_bad(&mut self, hash: H256) {
		self.snapshot.note_bad(hash)
	}

	pub fn reset_manifest_to(&mut self, manifest: &ManifestData, hash: &H256) {
		self.snapshot.reset_to(manifest, hash);
	}

	pub fn on_peer_status(
		&mut self,
		event: &SnapshotSyncEvent,
		rlp: Rlp,
		genesis_hash: H256,
	) -> Result<(), WarpSyncError> {
		use chain::{PAR_PROTOCOL_VERSION_1, PAR_PROTOCOL_VERSION_2, PAR_PROTOCOL_VERSION_3};

		let supported_versions = [PAR_PROTOCOL_VERSION_1.0, PAR_PROTOCOL_VERSION_2.0, PAR_PROTOCOL_VERSION_3.0];
		let peer_id = event.peer();
		let warp_protocol = event.as_context().protocol_version(event.peer()).unwrap_or(0) != 0;

		if !warp_protocol {
			trace!(target: "warp", "Peer doesn't support warp sync {}", peer_id);
			return Err(WarpSyncError::NotServer);
		}

		if self.peers.contains_key(&peer_id) {
			debug!(target: "warp", "Unexpected status packet from a known peer {}", peer_id);
			return Ok(());
		}

		let protocol_version: u8 = rlp.val_at(0)?;
		let network_id: u64 = rlp.val_at(1)?;

		// make sure the status message is well formed
		// even though we're not going to use these fields
		let _difficulty: U256 = rlp.val_at(2)?;
		let _latest_hash: H256 = rlp.val_at(3)?;

		let genesis: H256 = rlp.val_at(4)?;
		let snapshot_hash: H256 = rlp.val_at(5)?;
		let snapshot_number: BlockNumber = rlp.val_at(6)?;

		if !supported_versions.contains(&protocol_version)  {
			trace!(target: "warp", "Peer {} unsupported protocol version ({})", peer_id, protocol_version);
			return Err(WarpSyncError::UnsupportedProtocolVersion(protocol_version));
		}

		let expected_network_id = event.as_context().network_id();
		if network_id != expected_network_id || genesis != genesis_hash {
			trace!(
				target: "warp",
				"Wrong network: peer {} (genesis hash, network id): found ({}, {}), expected ({}, {})",
				peer_id,
				genesis, network_id,
				genesis_hash, expected_network_id,
			);
			return Err(WarpSyncError::WrongNetwork);
		}

		// may be a light client
		if snapshot_number == 0 {
			trace!(target: "warp", "Received a status message with snapshot number 0 from {}", peer_id);
			return Ok(());
		}

		let peer = SnapshotPeer::new(snapshot_hash, snapshot_number);
		self.peers.insert(peer_id, peer);

		if self.sync_start_time.is_none() {
			self.sync_start_time = Some(Instant::now());
		}

		debug!(target: "warp", "Connected {}", peer_id);

		Ok(())
	}
}

/// Warp sync download state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpSyncState {
	/// Collecting enough peers to start warp sync.
	WaitingPeers,
	/// Downloading manifest.
	Manifest,
	/// Downloading block chunks.
	Blocks,
	/// Waiting for snapshot restoration progress.
	WaitingService,
}

/// Kinds of errors which can be encountered in the course of warp sync.
#[derive(Debug)]
pub enum WarpSyncError {
	/// An RLP decoding error.
	Rlp(rlp::DecoderError),
	/// A network error.
	Network(network::Error),
	/// Unrecognized packet code.
	UnrecognizedPacket(u8),
	/// Peer on wrong network (wrong NetworkId or genesis hash)
	WrongNetwork,
	/// Not a server,
	NotServer,
	/// Unsupported protocol version.
	UnsupportedProtocolVersion(u8),
	/// Unsupported manifest version.
	UnsupportedManifestVersion(u64),
	/// Invalid manifest packet.
	BadManifest,
	/// Invalid block chunk.
	BadBlockChunk,
}

impl WarpSyncError {
	/// What level of punishment does this error warrant?
	pub fn punishment(&self) -> Punishment {
		match *self {
			WarpSyncError::Rlp(_) => Punishment::Disable,
			WarpSyncError::Network(_) => Punishment::None,
			WarpSyncError::UnrecognizedPacket(_) => Punishment::None,
			WarpSyncError::WrongNetwork => Punishment::Disable,
			WarpSyncError::NotServer => Punishment::Disconnect,
			WarpSyncError::UnsupportedProtocolVersion(_) => Punishment::Disable,
			WarpSyncError::UnsupportedManifestVersion(_) => Punishment::Disable,
			WarpSyncError::BadManifest => Punishment::Disable,
			WarpSyncError::BadBlockChunk => Punishment::Disable,
		}
	}
}

impl From<rlp::DecoderError> for WarpSyncError {
	fn from(err: rlp::DecoderError) -> Self {
		WarpSyncError::Rlp(err)
	}
}

impl From<network::Error> for WarpSyncError {
	fn from(err: network::Error) -> Self {
		WarpSyncError::Network(err)
	}
}
