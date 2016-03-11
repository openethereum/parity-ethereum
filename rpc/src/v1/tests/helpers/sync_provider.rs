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

use ethcore::transaction::SignedTransaction;
use ethsync::{SyncProvider, SyncStatus, SyncState};

pub struct Config {
	pub protocol_version: u8,
	pub num_peers: usize,
}

pub struct TestSyncProvider {
	status: SyncStatus,
}

impl TestSyncProvider {
	pub fn new(config: Config) -> Self {
		TestSyncProvider {
			status: SyncStatus {
				state: SyncState::NotSynced,
				protocol_version: config.protocol_version,
				start_block_number: 0,
				last_imported_block_number: None,
				highest_block_number: None,
				blocks_total: 0,
				blocks_received: 0,
				num_peers: config.num_peers,
				num_active_peers: 0,
				mem_used: 0,
				transaction_queue_pending: 0,
			},
		}
	}
}

impl SyncProvider for TestSyncProvider {
	fn status(&self) -> SyncStatus {
		self.status.clone()
	}

	fn insert_transaction(&self, _transaction: SignedTransaction) {
		unimplemented!()
	}
}

