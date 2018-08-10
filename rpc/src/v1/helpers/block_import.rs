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

//! Block import analysis functions.

use ethcore::client::BlockQueueInfo;
use sync::SyncState;

/// Check if client is during major sync or during block import and allows defining whether 'waiting for peers' should
/// be considered a syncing state.
pub fn is_major_importing_or_waiting(sync_state: Option<SyncState>, queue_info: BlockQueueInfo, waiting_is_syncing_state: bool) -> bool {
	let is_syncing_state = sync_state.map_or(false, |s| match s {
		SyncState::Idle | SyncState::NewBlocks => false,
		SyncState::WaitingPeers if !waiting_is_syncing_state => false,
		_ => true,
	});
	let is_verifying = queue_info.unverified_queue_size + queue_info.verified_queue_size > 3;
	is_verifying || is_syncing_state
}

/// Check if client is during major sync or during block import.
pub fn is_major_importing(sync_state: Option<SyncState>, queue_info: BlockQueueInfo) -> bool {
	is_major_importing_or_waiting(sync_state, queue_info, true)
}

#[cfg(test)]
mod tests {
	use ethcore::client::BlockQueueInfo;
	use sync::SyncState;
	use super::is_major_importing;

	fn queue_info(unverified: usize, verified: usize) -> BlockQueueInfo {
		BlockQueueInfo {
			unverified_queue_size: unverified,
			verified_queue_size: verified,
			verifying_queue_size: 0,
			max_queue_size: 1000,
			max_mem_use: 1000,
			mem_used: 500
		}
	}

	#[test]
	fn is_still_verifying() {
		assert!(!is_major_importing(None, queue_info(2, 1)));
		assert!(is_major_importing(None, queue_info(2, 2)));
	}

	#[test]
	fn is_synced_state() {
		assert!(is_major_importing(Some(SyncState::Blocks), queue_info(0, 0)));
		assert!(!is_major_importing(Some(SyncState::Idle), queue_info(0, 0)));
	}
}
