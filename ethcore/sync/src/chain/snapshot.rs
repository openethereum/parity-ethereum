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

/// Snapshot sync handler.
pub struct SnapshotSync {
	// Current state of the snapshot sync
	state: SnapshotSyncState,
	// List of connected snapshot peers
	peers: SnapshotPeers,
	/// Warp sync mode.
	warp_sync: WarpSync,
}

impl SnapshotSync {
	pub fn new(config: SyncConfig) -> SnapshotSync {
		SnapshotSync {
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
}
