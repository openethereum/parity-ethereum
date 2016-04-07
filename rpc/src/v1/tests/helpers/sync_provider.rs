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

//! Test implementation of SyncProvider.

use util::{U256};
use ethsync::{SyncProvider, SyncStatus, SyncState};
use std::sync::{RwLock};

/// TestSyncProvider config.
pub struct Config {
	/// Protocol version.
	pub network_id: U256,
	/// Number of peers.
	pub num_peers: usize,
}

/// Test sync provider.
pub struct TestSyncProvider {
	/// Sync status.
	pub status: RwLock<SyncStatus>,
}

impl TestSyncProvider {
	/// Creates new sync provider.
	pub fn new(config: Config) -> Self {
		TestSyncProvider {
			status: RwLock::new(SyncStatus {
				state: SyncState::NotSynced,
				network_id: config.network_id,
				protocol_version: 63,
				start_block_number: 0,
				last_imported_block_number: None,
				highest_block_number: None,
				blocks_total: 0,
				blocks_received: 0,
				num_peers: config.num_peers,
				num_active_peers: 0,
				mem_used: 0,
			}),
		}
	}
}

impl SyncProvider for TestSyncProvider {
	fn status(&self) -> SyncStatus {
		self.status.read().unwrap().clone()
	}
}

