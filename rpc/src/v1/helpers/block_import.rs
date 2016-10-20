// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
use ethsync::SyncState;

/// Check if client is during major sync or during block import.
pub fn is_major_importing(sync_status: &Option<SyncState>, queue_info: &BlockQueueInfo) -> bool {
	let is_syncing_state = sync_status.map_or(false, |s|
		s != SyncState::Idle && s != SyncState::NewBlocks
	);
	let is_verifying = queue_info.unverified_queue_size + queue_info.verified_queue_size > 3;
	is_verifying || is_syncing_state
}
