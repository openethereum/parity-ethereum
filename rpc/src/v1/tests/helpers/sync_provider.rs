// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::collections::BTreeMap;
use util::{H256, RwLock};
use ethsync::{SyncProvider, EthProtocolInfo, SyncStatus, SyncState, PeerInfo, TransactionStats};

/// TestSyncProvider config.
pub struct Config {
	/// Protocol version.
	pub network_id: u64,
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
				state: SyncState::Idle,
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
				num_snapshot_chunks: 0,
				snapshot_chunks_done: 0,
				last_imported_old_block_number: None,
			}),
		}
	}

	/// Simulate importing blocks.
	pub fn increase_imported_block_number(&self, count: u64) {
		let mut status =  self.status.write();
		let current_number = status.last_imported_block_number.unwrap_or(0);
		status.last_imported_block_number = Some(current_number + count);
	}
}

impl SyncProvider for TestSyncProvider {
	fn status(&self) -> SyncStatus {
		self.status.read().clone()
	}

	fn peers(&self) -> Vec<PeerInfo> {
		vec![
			PeerInfo {
				id: Some("node1".to_owned()),
    			client_version: "Parity/1".to_owned(),
				capabilities: vec!["eth/62".to_owned(), "eth/63".to_owned()],
    			remote_address: "127.0.0.1:7777".to_owned(),
				local_address: "127.0.0.1:8888".to_owned(),
				eth_info: Some(EthProtocolInfo {
					version: 62,
					difficulty: Some(40.into()),
					head: 50.into(),
				}),
				les_info: None,
			},
			PeerInfo {
				id: None,
    			client_version: "Parity/2".to_owned(),
				capabilities: vec!["eth/63".to_owned(), "eth/64".to_owned()],
    			remote_address: "Handshake".to_owned(),
				local_address: "127.0.0.1:3333".to_owned(),
				eth_info: Some(EthProtocolInfo {
					version: 64,
					difficulty: None,
					head: 60.into()
				}),
				les_info: None,
			}
		]
	}

	fn enode(&self) -> Option<String> {
		None
	}

	fn transactions_stats(&self) -> BTreeMap<H256, TransactionStats> {
		map![
			1.into() => TransactionStats {
				first_seen: 10,
				propagated_to: map![
					128.into() => 16
				],
			},
			5.into() => TransactionStats {
				first_seen: 16,
				propagated_to: map![
					16.into() => 1
				],
			}
		]
	}
}

