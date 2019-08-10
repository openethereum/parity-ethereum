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

//! Block import analysis functions.

use types::verification::VerificationQueueInfo as BlockQueueInfo;
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
