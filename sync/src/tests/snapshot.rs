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

use std::collections::HashMap;
use std::sync::Arc;
use hash::keccak;
use ethereum_types::H256;
use parking_lot::Mutex;
use bytes::Bytes;
use ethcore::snapshot::{SnapshotService, ManifestData, RestorationStatus};
use ethcore::header::BlockNumber;
use ethcore::client::{EachBlockWith};
use super::helpers::*;
use SyncConfig;

pub struct TestSnapshotService {
	manifest: Option<ManifestData>,
	chunks: HashMap<H256, Bytes>,

	restoration_manifest: Mutex<Option<ManifestData>>,
	state_restoration_chunks: Mutex<HashMap<H256, Bytes>>,
	block_restoration_chunks: Mutex<HashMap<H256, Bytes>>,
}

impl TestSnapshotService {
	pub fn new() -> TestSnapshotService {
		TestSnapshotService {
			manifest: None,
			chunks: HashMap::new(),
			restoration_manifest: Mutex::new(None),
			state_restoration_chunks: Mutex::new(HashMap::new()),
			block_restoration_chunks: Mutex::new(HashMap::new()),
		}
	}

	pub fn new_with_snapshot(num_chunks: usize, block_hash: H256, block_number: BlockNumber) -> TestSnapshotService {
		let num_state_chunks = num_chunks / 2;
		let num_block_chunks = num_chunks - num_state_chunks;
		let state_chunks: Vec<Bytes> = (0..num_state_chunks).map(|_| H256::random().to_vec()).collect();
		let block_chunks: Vec<Bytes> = (0..num_block_chunks).map(|_| H256::random().to_vec()).collect();
		let manifest = ManifestData {
			version: 2,
			state_hashes: state_chunks.iter().map(|data| keccak(data)).collect(),
			block_hashes: block_chunks.iter().map(|data| keccak(data)).collect(),
			state_root: H256::new(),
			block_number: block_number,
			block_hash: block_hash,
		};
		let mut chunks: HashMap<H256, Bytes> = state_chunks.into_iter().map(|data| (keccak(&data), data)).collect();
		chunks.extend(block_chunks.into_iter().map(|data| (keccak(&data), data)));
		TestSnapshotService {
			manifest: Some(manifest),
			chunks: chunks,
			restoration_manifest: Mutex::new(None),
			state_restoration_chunks: Mutex::new(HashMap::new()),
			block_restoration_chunks: Mutex::new(HashMap::new()),
		}
	}
}

impl SnapshotService for TestSnapshotService {
	fn manifest(&self) -> Option<ManifestData> {
		self.manifest.as_ref().cloned()
	}

	fn supported_versions(&self) -> Option<(u64, u64)> {
		Some((1, 2))
	}

	fn chunk(&self, hash: H256) -> Option<Bytes> {
		self.chunks.get(&hash).cloned()
	}

	fn status(&self) -> RestorationStatus {
		match *self.restoration_manifest.lock() {
			Some(ref manifest) if self.state_restoration_chunks.lock().len() == manifest.state_hashes.len() &&
				self.block_restoration_chunks.lock().len() == manifest.block_hashes.len() => RestorationStatus::Inactive,
			Some(ref manifest) => RestorationStatus::Ongoing {
				state_chunks: manifest.state_hashes.len() as u32,
				block_chunks: manifest.block_hashes.len() as u32,
				state_chunks_done: self.state_restoration_chunks.lock().len() as u32,
				block_chunks_done: self.block_restoration_chunks.lock().len() as u32,
			},
			None => RestorationStatus::Inactive,
		}
	}

	fn begin_restore(&self, manifest: ManifestData) {
		*self.restoration_manifest.lock() = Some(manifest);
		self.state_restoration_chunks.lock().clear();
		self.block_restoration_chunks.lock().clear();
	}

	fn abort_restore(&self) {
		*self.restoration_manifest.lock() = None;
		self.state_restoration_chunks.lock().clear();
		self.block_restoration_chunks.lock().clear();
	}

	fn restore_state_chunk(&self, hash: H256, chunk: Bytes) {
		if self.restoration_manifest.lock().as_ref().map_or(false, |m| m.state_hashes.iter().any(|h| h == &hash)) {
			self.state_restoration_chunks.lock().insert(hash, chunk);
		}
	}

	fn restore_block_chunk(&self, hash: H256, chunk: Bytes) {
		if self.restoration_manifest.lock().as_ref().map_or(false, |m| m.block_hashes.iter().any(|h| h == &hash)) {
			self.block_restoration_chunks.lock().insert(hash, chunk);
		}
	}
}

#[test]
fn snapshot_sync() {
	::env_logger::init().ok();
	let mut config = SyncConfig::default();
	config.warp_sync = true;
	let mut net = TestNet::new_with_config(5, config);
	let snapshot_service = Arc::new(TestSnapshotService::new_with_snapshot(16, H256::new(), 500000));
	for i in 0..4 {
		net.peer_mut(i).snapshot_service = snapshot_service.clone();
		net.peer(i).chain.add_blocks(1, EachBlockWith::Nothing);
	}
	net.sync_steps(50);
	assert_eq!(net.peer(4).snapshot_service.state_restoration_chunks.lock().len(), net.peer(0).snapshot_service.manifest.as_ref().unwrap().state_hashes.len());
	assert_eq!(net.peer(4).snapshot_service.block_restoration_chunks.lock().len(), net.peer(0).snapshot_service.manifest.as_ref().unwrap().block_hashes.len());
}

