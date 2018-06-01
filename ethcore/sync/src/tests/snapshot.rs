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
use ethcore::client::EachBlockWith;
use super::helpers::*;
use {SyncConfig, WarpSync};

pub struct TestSnapshot {
	manifest: ManifestData,
	chunks: HashMap<H256, Bytes>,
}

impl TestSnapshot {
	pub fn new(num_chunks: usize, block_hash: H256, block_number: BlockNumber) -> TestSnapshot {
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

		TestSnapshot {
			manifest,
			chunks,
		}
	}
}

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

	pub fn new_with_snapshot(snapshot: &TestSnapshot) -> TestSnapshotService {
		TestSnapshotService {
			manifest: Some(snapshot.manifest.clone()),
			chunks: snapshot.chunks.clone(),
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

	fn partial_manifest(&self) -> Option<ManifestData> {
		self.restoration_manifest.lock().as_ref().cloned()
	}

	fn supported_versions(&self) -> Option<(u64, u64)> {
		Some((1, 2))
	}

	fn completed_chunks(&self) -> Option<Vec<H256>> {
		if self.restoration_manifest.lock().is_none() {
			return None;
		}

		let chunks = self.state_restoration_chunks.lock().keys()
			.chain(self.block_restoration_chunks.lock().keys())
			.map(|h| *h).collect();

		Some(chunks)
	}

	fn chunk(&self, hash: H256) -> Option<Bytes> {
		if let Some(bytes) = self.chunks.get(&hash) {
			return Some(bytes.clone());
		}
		if let Some(bytes) = self.block_restoration_chunks.lock().get(&hash) {
			return Some(bytes.clone());
		}
		if let Some(bytes) = self.state_restoration_chunks.lock().get(&hash) {
			return Some(bytes.clone());
		}

		None
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
		let mut restoration_manifest = self.restoration_manifest.lock();

		if let Some(ref c_manifest) = *restoration_manifest {
			if c_manifest.state_root == manifest.state_root {
				return;
			}
		}

		*restoration_manifest = Some(manifest);
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
	let num_peers = 5;
	let mut config = SyncConfig::default();
	config.warp_sync = WarpSync::Enabled;
	let mut net = TestNet::new_with_config(num_peers, config);
	let snapshot = TestSnapshot::new(16, H256::new(), 30_050);
	for i in 0..(num_peers-1) {
		// The first peers needs to have at least `snapshot_block - 30_000` blocks
		// so that they don't try to sync a snapshot with the other peers
		net.peer_mut(i).snapshot_service = Arc::new(TestSnapshotService::new_with_snapshot(&snapshot));
		net.peer(i).chain.add_blocks(60, EachBlockWith::Nothing);
	}
	net.sync_steps(50);
	assert_eq!(net.peer(num_peers-1).snapshot_service.state_restoration_chunks.lock().len(), net.peer(0).snapshot_service.manifest.as_ref().unwrap().state_hashes.len());
	assert_eq!(net.peer(num_peers-1).snapshot_service.block_restoration_chunks.lock().len(), net.peer(0).snapshot_service.manifest.as_ref().unwrap().block_hashes.len());
}

#[test]
/// This test will first create a network of peers with a full snapshot and empty peers,
/// that will partially restore the snapshot ; Then with the partial peers and an empty one.
/// The last empty peer should be able to get some chunks from the partially recovered ones.
/// Note that we need at least 3 peers with the Snapshot Manifest per network, for the recovery
/// to start
fn snapshot_partial_sync() {
	::env_logger::init().ok();
	let num_peers_full = 3;
	let num_peers_partial = 3;
	let mut config = SyncConfig::default();
	config.warp_sync = WarpSync::Enabled;
	let mut net = TestNet::new_with_config(num_peers_full + num_peers_partial, config);
	let snapshot = TestSnapshot::new(100, H256::new(), 30_050);
	for i in 0..num_peers_full {
		// The first peers needs to have at least `snapshot_block - 30_000` blocks
		// so that they don't try to sync a snapshot with the other peers
		net.peer_mut(i).snapshot_service = Arc::new(TestSnapshotService::new_with_snapshot(&snapshot));
		net.peer(i).chain.add_blocks(60, EachBlockWith::Nothing);
	}
	// First, let's partially sync the snapshot
	net.sync_steps(50);
	{
		let partial_snapshot_service = &net.peer(num_peers_full).snapshot_service;
		assert!(partial_snapshot_service.state_restoration_chunks.lock().len() > 0);
		assert!(partial_snapshot_service.state_restoration_chunks.lock().len() < net.peer(0).snapshot_service.manifest.as_ref().unwrap().state_hashes.len());
		assert!(partial_snapshot_service.block_restoration_chunks.lock().len() > 0);
		assert!(partial_snapshot_service.block_restoration_chunks.lock().len() < net.peer(0).snapshot_service.manifest.as_ref().unwrap().block_hashes.len());
	}

	// Now, create a new network with an empty peer, and the partially synced one
	let mut net_partial = TestNet::new_with_config(num_peers_partial + 1, config);

	// Re-use the partially restored snapshot service
	for i in 0..num_peers_partial {
		net_partial.set_peer(i, net.raw_peer(num_peers_full + i));
	}

	let partial_block_chunks = net_partial.peer(0).snapshot_service.block_restoration_chunks.lock().len();
	let partial_state_chunks = net_partial.peer(0).snapshot_service.state_restoration_chunks.lock().len();

	net_partial.sync_steps(partial_block_chunks + partial_state_chunks - 5);

	// First, ensure that after 50 steps, the empty node started getting some chunks, only partially
	{
		let partial_snapshot_service = &net_partial.peer(num_peers_partial).snapshot_service;
		let restored_block_chunks = partial_snapshot_service.block_restoration_chunks.lock().len();
		let restored_state_chunks = partial_snapshot_service.state_restoration_chunks.lock().len();

		assert!(restored_block_chunks > 0);
		assert!(restored_state_chunks > 0);

		assert!(restored_block_chunks < partial_block_chunks);
		assert!(restored_state_chunks < partial_state_chunks);
	}

	net_partial.sync_steps(100);

	// Second, ensure that after enough steps the empty node fetched all available chunks
	// (note that it could be greater than the number of chunks of the first partial peer,
	// since other partial peers could have more chunks)
	{
		let partial_snapshot_service = &net_partial.peer(num_peers_partial).snapshot_service;
		let restored_block_chunks = partial_snapshot_service.block_restoration_chunks.lock().len();
		let restored_state_chunks = partial_snapshot_service.state_restoration_chunks.lock().len();

		assert!(restored_block_chunks >= partial_block_chunks);
		assert!(restored_state_chunks >= partial_state_chunks);
	}
}
