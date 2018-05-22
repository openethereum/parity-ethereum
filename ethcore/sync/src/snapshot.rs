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

use ethcore::snapshot::{ManifestData, SnapshotService};
use ethereum_types::H256;
use hash::keccak;
use rand::{thread_rng, Rng};

use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(PartialEq, Eq, Debug)]
pub enum ChunkType {
	State(H256),
	Block(H256),
}

pub struct Snapshot {
	pending_state_chunks: Vec<H256>,
	pending_block_chunks: Vec<H256>,
	downloading_chunks: HashSet<H256>,
	completed_chunks: HashSet<H256>,
	snapshot_hash: Option<H256>,
	bad_hashes: HashSet<H256>,
	initialized: bool,
}

impl Snapshot {
	/// Create a new instance.
	pub fn new() -> Snapshot {
		Snapshot {
			pending_state_chunks: Vec::new(),
			pending_block_chunks: Vec::new(),
			downloading_chunks: HashSet::new(),
			completed_chunks: HashSet::new(),
			snapshot_hash: None,
			bad_hashes: HashSet::new(),
			initialized: false,
		}
	}

	/// Sync the Snapshot completed chunks with the Snapshot Service
	pub fn initialize(&mut self, snapshot_service: &SnapshotService) {
		if self.initialized {
			return;
		}

		if let Some(completed_chunks) = snapshot_service.completed_chunks() {
			self.completed_chunks = HashSet::from_iter(completed_chunks);
		}

		trace!(
			target: "snapshot",
			"Snapshot is now initialized with {} completed chunks.",
			self.completed_chunks.len(),
		);

		self.initialized = true;
	}

	/// Clear everything.
	pub fn clear(&mut self) {
		self.pending_state_chunks.clear();
		self.pending_block_chunks.clear();
		self.downloading_chunks.clear();
		self.completed_chunks.clear();
		self.snapshot_hash = None;
		self.initialized = false;
	}

	/// Check if currently downloading a snapshot.
	pub fn have_manifest(&self) -> bool {
		self.snapshot_hash.is_some()
	}

	/// Reset collection for a manifest RLP
	pub fn reset_to(&mut self, manifest: &ManifestData, hash: &H256) {
		self.clear();
		self.pending_state_chunks = manifest.state_hashes.clone();
		self.pending_block_chunks = manifest.block_hashes.clone();
		self.snapshot_hash = Some(hash.clone());
	}

	/// Validate chunk and mark it as downloaded
	pub fn validate_chunk(&mut self, chunk: &[u8]) -> Result<ChunkType, ()> {
		let hash = keccak(chunk);
		if self.completed_chunks.contains(&hash) {
			trace!(target: "sync", "Ignored proccessed chunk: {:x}", hash);
			return Err(());
		}
		self.downloading_chunks.remove(&hash);
		if self.pending_block_chunks.iter().any(|h| h == &hash) {
			self.completed_chunks.insert(hash.clone());
			return Ok(ChunkType::Block(hash));
		}
		if self.pending_state_chunks.iter().any(|h| h == &hash) {
			self.completed_chunks.insert(hash.clone());
			return Ok(ChunkType::State(hash));
		}
		trace!(target: "sync", "Ignored unknown chunk: {:x}", hash);
		Err(())
	}

	/// Find a random chunk to download
	pub fn needed_chunk(&mut self) -> Option<H256> {
		// Find all random chunks: first blocks, then state
		let needed_chunks = {
			let chunk_filter = |h| !self.downloading_chunks.contains(h) && !self.completed_chunks.contains(h);

			let needed_block_chunks = self.pending_block_chunks.iter()
				.filter(|&h| chunk_filter(h))
				.map(|h| *h)
				.collect::<Vec<H256>>();

			// If no block chunks to download, get the state chunks
			if needed_block_chunks.len() == 0 {
				self.pending_state_chunks.iter()
					.filter(|&h| chunk_filter(h))
					.map(|h| *h)
					.collect::<Vec<H256>>()
			} else {
				needed_block_chunks
			}
		};

		// Get a random chunk
		let chunk = thread_rng().choose(&needed_chunks);

		if let Some(hash) = chunk {
			self.downloading_chunks.insert(hash.clone());
		}
		chunk.map(|h| *h)
	}

	pub fn clear_chunk_download(&mut self, hash: &H256) {
		self.downloading_chunks.remove(hash);
	}

	// note snapshot hash as bad.
	pub fn note_bad(&mut self, hash: H256) {
		self.bad_hashes.insert(hash);
	}

	// whether snapshot hash is known to be bad.
	pub fn is_known_bad(&self, hash: &H256) -> bool {
		self.bad_hashes.contains(hash)
	}

	pub fn snapshot_hash(&self) -> Option<H256> {
		self.snapshot_hash
	}

	pub fn total_chunks(&self) -> usize {
		self.pending_block_chunks.len() + self.pending_state_chunks.len()
	}

	pub fn done_chunks(&self) -> usize {
		self.completed_chunks.len()
	}

	pub fn is_complete(&self) -> bool {
		self.total_chunks() == self.completed_chunks.len()
	}
}

#[cfg(test)]
mod test {
	use hash::keccak;
	use bytes::Bytes;
	use super::*;
	use ethcore::snapshot::ManifestData;

	fn is_empty(snapshot: &Snapshot) -> bool {
		snapshot.pending_block_chunks.is_empty() &&
		snapshot.pending_state_chunks.is_empty() &&
		snapshot.completed_chunks.is_empty() &&
		snapshot.downloading_chunks.is_empty() &&
		snapshot.snapshot_hash.is_none()
	}

	fn test_manifest() -> (ManifestData, H256, Vec<Bytes>, Vec<Bytes>) {
		let state_chunks: Vec<Bytes> = (0..20).map(|_| H256::random().to_vec()).collect();
		let block_chunks: Vec<Bytes> = (0..20).map(|_| H256::random().to_vec()).collect();
		let manifest = ManifestData {
			version: 2,
			state_hashes: state_chunks.iter().map(|data| keccak(data)).collect(),
			block_hashes: block_chunks.iter().map(|data| keccak(data)).collect(),
			state_root: H256::new(),
			block_number: 42,
			block_hash: H256::new(),
		};
		let mhash = keccak(manifest.clone().into_rlp());
		(manifest, mhash, state_chunks, block_chunks)
	}

	#[test]
	fn create_clear() {
		let mut snapshot = Snapshot::new();
		assert!(is_empty(&snapshot));
		let (manifest, mhash, _, _,) = test_manifest();
		snapshot.reset_to(&manifest, &mhash);
		assert!(!is_empty(&snapshot));
		snapshot.clear();
		assert!(is_empty(&snapshot));
	}

	#[test]
	fn validate_chunks() {
		let mut snapshot = Snapshot::new();
		let (manifest, mhash, state_chunks, block_chunks) = test_manifest();
		snapshot.reset_to(&manifest, &mhash);
		assert_eq!(snapshot.done_chunks(), 0);
		assert!(snapshot.validate_chunk(&H256::random().to_vec()).is_err());

		let requested: Vec<H256> = (0..40).map(|_| snapshot.needed_chunk().unwrap()).collect();
		assert!(snapshot.needed_chunk().is_none());

		let requested_all_block_chunks = manifest.block_hashes.iter()
			.all(|h| requested.iter().any(|rh| rh == h));
		assert!(requested_all_block_chunks);

		let requested_all_state_chunks = manifest.state_hashes.iter()
			.all(|h| requested.iter().any(|rh| rh == h));
		assert!(requested_all_state_chunks);

		assert_eq!(snapshot.downloading_chunks.len(), 40);

		assert_eq!(snapshot.validate_chunk(&state_chunks[4]), Ok(ChunkType::State(manifest.state_hashes[4].clone())));
		assert_eq!(snapshot.completed_chunks.len(), 1);
		assert_eq!(snapshot.downloading_chunks.len(), 39);

		assert_eq!(snapshot.validate_chunk(&block_chunks[10]), Ok(ChunkType::Block(manifest.block_hashes[10].clone())));
		assert_eq!(snapshot.completed_chunks.len(), 2);
		assert_eq!(snapshot.downloading_chunks.len(), 38);

		for (i, data) in state_chunks.iter().enumerate() {
			if i != 4 {
				assert!(snapshot.validate_chunk(data).is_ok());
			}
		}

		for (i, data) in block_chunks.iter().enumerate() {
			if i != 10 {
				assert!(snapshot.validate_chunk(data).is_ok());
			}
		}

		assert!(snapshot.is_complete());
		assert_eq!(snapshot.done_chunks(), 40);
		assert_eq!(snapshot.done_chunks(), snapshot.total_chunks());
		assert_eq!(snapshot.snapshot_hash(), Some(keccak(manifest.into_rlp())));
	}

	#[test]
	fn tracks_known_bad() {
		let mut snapshot = Snapshot::new();
		let hash = H256::random();

		assert_eq!(snapshot.is_known_bad(&hash), false);
		snapshot.note_bad(hash);
		assert_eq!(snapshot.is_known_bad(&hash), true);
	}
}

