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

use ethereum_types::H256;
use ethcore::header::BlockNumber;
use network::{PeerId};

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{
	PAR_PROTOCOL_VERSION_4,
	SyncConfig,
	WarpSync,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
/// Snapshot Sync state
pub enum SnapshotSyncState {
	/// Collecting enough peers to start syncing.
	WaitingPeers,
	/// Waiting for snapshot manifest download
	Manifest,
	/// Snapshot service is initializing
	Init,
	/// Downloading snapshot data
	Data,
	/// Waiting for snapshot restoration progress.
	Waiting,
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Peer data type requested
pub enum SnapshotPeerAsking {
	Nothing,
	Bitfield,
	Manifest,
	Data,
}

#[derive(Clone)]
/// Syncing peer information
pub struct PeerInfo {
	/// eth protocol version
	protocol_version: u8,
	/// Type of data currenty being requested from peer.
	asking: SnapshotPeerAsking,
	/// Holds requested snapshot chunk hash if any.
	asking_snapshot_data: Option<H256>,
	/// Last bitfield request
	ask_bitfield_time: Instant,
	/// Peer's current snapshot bitfield
	snapshot_bitfield: Option<Vec<u8>>,
	/// Best snapshot hash
	snapshot_hash: H256,
	/// Best snapshot block number
	snapshot_number: BlockNumber,
}

impl PeerInfo {
	/// Check if the peer has the chunk available
	pub fn chunk_index_available(&self, index: usize) -> bool {
		// If no bitfield, assume every chunk is available
		match self.snapshot_bitfield {
			None => true,
			Some(ref bitfield) => {
				let byte_index = index / 8;

				if byte_index >= bitfield.len() {
					return false;
				}

				let bit_index = index % 8;

				(bitfield[byte_index] >> (7 - bit_index)) & 1 != 0
			}
		}
	}

	/// Check whether the peer supports partial snapshots
	pub fn supports_partial_snapshots(protocol_version: u8) -> bool {
		match protocol_version {
			PAR_PROTOCOL_VERSION_4 => true,
			_ => false,
		}
	}
}

pub type SnapshotPeers = HashMap<PeerId, PeerInfo>;

pub trait NetworkSync {
	/// Maintain other peers. Send out any new blocks and transactions
	pub fn maintain_sync(&mut self, io: &mut SyncIo);

	/// Abort all sync activity
	pub fn abort(&mut self, io: &mut SyncIo);
}

/// Snapshot sync handler.
pub struct SnapshotSync {
	// Whether snapshot sync is enabled
	enabled: bool,
	// Current state of the snapshot sync
	state: SnapshotSyncState,
	// List of connected snapshot peers
	peers: SnapshotPeers,
	/// Warp sync mode.
	warp_sync: WarpSync,
}

impl SnapshotSync {
	pub fn new(config: SyncConfig, io: &SyncIo) -> SnapshotSync {
		let enabled = self.warp_sync.is_enabled() || io.snapshot_service().supported_versions().is_none();

		SnapshotSync {
			enabled: enabled,
			peers: HashMap::new(),
			state: SnapshotSyncState::WaitingPeers,
			warp_sync: config.warp_sync,
		}
	}

	/// Find something to do for a peer. Called for a new peer or when a peer is done with its task.
	fn sync_peer(&mut self, io: &mut SyncIo, peer_id: PeerId, force: bool);

	/// Abort all sync activity
	pub fn abort(&mut self, io: &mut SyncIo);

	/// Reset sync. Clear all downloaded data but keep the queue
	fn reset(&mut self, io: &mut SyncIo);

	/// Restart sync disregarding the block queue status. May end up re-downloading up to QUEUE_SIZE blocks
	pub fn restart(&mut self, io: &mut SyncIo);

	/// Resume downloading
	fn continue_sync(&mut self, io: &mut SyncIo);

	fn maybe_start_snapshot_sync(&mut self, io: &mut SyncIo) {
		match self.state {
			SnapshotSyncState::WaitingPeers |
			SnapshotSyncState::Init |
			SnapshotSyncState::Manifest |
			SnapshotSyncState::Waiting => {
				trace!(target: "sync", "Skipping warp sync. State: {:?}", self.state);
				return;
			},
			SnapshotSyncState::Data => {
				self.continue_sync(io);
			},
		}
		// Make sure the snapshot block is not too far away from best block and network best block and
		// that it is higher than fork detection block
		let our_best_block = io.chain().chain_info().best_block_number;
		let fork_block = self.fork_block.map_or(0, |(n, _)| n);

		let (best_hash, max_peers, snapshot_peers) = {
			let expected_warp_block = match self.warp_sync {
				WarpSync::OnlyAndAfter(block) => block,
				_ => 0,
			};
			//collect snapshot infos from peers
			let snapshots = self.peers.iter()
				.filter(|&(_, p)| p.is_allowed() && p.snapshot_number.map_or(false, |sn|
					// Snapshot must be old enough that it's usefull to sync with it
					our_best_block < sn && (sn - our_best_block) > SNAPSHOT_RESTORE_THRESHOLD &&
					// Snapshot must have been taken after the Fork
					sn > fork_block &&
					// Snapshot must be greater than the warp barrier if any
					sn > expected_warp_block &&
					// If we know a highest block, snapshot must be recent enough
					self.highest_block.map_or(true, |highest| {
						highest < sn || (highest - sn) <= SNAPSHOT_RESTORE_THRESHOLD
					})
				))
				.filter_map(|(p, peer)| peer.snapshot_hash.map(|hash| (p, hash.clone())))
				.filter(|&(_, ref hash)| !self.snapshot.is_known_bad(hash));

			let mut snapshot_peers = HashMap::new();
			let mut max_peers: usize = 0;
			let mut best_hash = None;
			for (p, hash) in snapshots {
				let peers = snapshot_peers.entry(hash).or_insert_with(Vec::new);
				peers.push(*p);
				if peers.len() > max_peers {
					max_peers = peers.len();
					best_hash = Some(hash);
				}
			}
			(best_hash, max_peers, snapshot_peers)
		};

		let timeout = (self.state == SyncState::WaitingPeers) && self.sync_start_time.map_or(false, |t| t.elapsed() > WAIT_PEERS_TIMEOUT);

		if let (Some(hash), Some(peers)) = (best_hash, best_hash.map_or(None, |h| snapshot_peers.get(&h))) {
			if max_peers >= SNAPSHOT_MIN_PEERS {
				trace!(target: "sync", "Starting confirmed snapshot sync {:?} with {:?}", hash, peers);
				self.start_snapshot_sync(io, peers);
			} else if timeout {
				trace!(target: "sync", "Starting unconfirmed snapshot sync {:?} with {:?}", hash, peers);
				self.start_snapshot_sync(io, peers);
			}
		} else if timeout && !self.warp_sync.is_warp_only() {
			trace!(target: "sync", "No snapshots found, starting full sync");
			self.state = SyncState::Idle;
			self.continue_sync(io);
		}
	}

	fn start_snapshot_sync(&mut self, io: &mut SyncIo, peers: &[PeerId]) {
		if !self.snapshot.have_manifest() {
			for p in peers {
				if self.peers.get(p).map_or(false, |p| p.asking == PeerAsking::Nothing) {
					SyncRequester::request_snapshot_manifest(self, io, *p);
				}
			}
			self.state = SyncState::SnapshotManifest;
			trace!(target: "sync", "New snapshot sync with {:?}", peers);
		} else {
			self.state = SyncState::SnapshotData;
			trace!(target: "sync", "Resumed snapshot sync with {:?}", peers);
		}
	}
}
