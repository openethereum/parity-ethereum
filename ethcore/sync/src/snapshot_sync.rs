// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

use std::collections::HashSet;
use std::iter::FromIterator;

use ethereum_types::H256;
use keccak_hash::keccak;
use log::trace;
use snapshot::SnapshotService;
use common_types::snapshot::ManifestData;
use indexmap::IndexSet;

#[derive(PartialEq, Eq, Debug)]
/// The type of data contained in a chunk: state or block.
pub enum ChunkType {
	/// The chunk contains state data (aka account data).
	State(H256),
	/// The chunk contains block data.
	Block(H256),
}

#[derive(Default, MallocSizeOf)]
pub struct Snapshot {
	/// List of hashes of the state chunks we need to complete the warp sync from this snapshot.
	/// These hashes are contained in the Manifest we downloaded from the peer(s).
	/// Note: this is an ordered set so that state restoration happens in order, which keeps
	/// memory usage down.
	// See https://github.com/paritytech/parity-common/issues/255
	#[ignore_malloc_size_of = "no impl for IndexSet (yet)"]
	pending_state_chunks: IndexSet<H256>,
	/// List of hashes of the block chunks we need to complete the warp sync from this snapshot.
	/// These hashes are contained in the Manifest we downloaded from the peer(s).
	/// Note: this is an ordered set so that state restoration happens in order, which keeps
	/// memory usage down.
	// See https://github.com/paritytech/parity-common/issues/255
	#[ignore_malloc_size_of = "no impl for IndexSet (yet)"]
	pending_block_chunks: IndexSet<H256>,
	/// Set of hashes of chunks we are currently downloading.
	downloading_chunks: HashSet<H256>,
	/// The set of chunks (block or state) that we have successfully downloaded.
	completed_chunks: HashSet<H256>,
	/// The hash of the the `ManifestData` RLP that we're downloading.
	snapshot_hash: Option<H256>,
	/// Total number of chunks in the current snapshot.
	total_chunks: Option<usize>,
	/// Set of snapshot hashes we failed to import. We will not try to sync with
	/// this snapshot again until restart.
	bad_hashes: HashSet<H256>,
	initialized: bool,
}

impl Snapshot {
	/// Create a new instance.
	pub fn new() -> Self {
		Default::default()
	}

	/// Sync the Snapshot completed chunks with the Snapshot Service
	pub fn initialize(&mut self, snapshot_service: &dyn SnapshotService, total_chunks: usize) {
		if self.initialized {
			return;
		}

		if let Some(completed_chunks) = snapshot_service.completed_chunks() {
			self.completed_chunks = HashSet::from_iter(completed_chunks);
		}

		trace!(
			target: "snapshot_sync",
			"Snapshot initialized. {}/{} completed chunks.",
			self.completed_chunks.len(), total_chunks
		);
		self.total_chunks = Some(total_chunks);
		self.initialized = true;
	}

	/// Clear everything and set `initialized` to false.
	pub fn clear(&mut self) {
		self.pending_state_chunks.clear();
		self.pending_block_chunks.clear();
		self.downloading_chunks.clear();
		self.completed_chunks.clear();
		self.snapshot_hash = None;
		self.total_chunks = None;
		self.initialized = false;
	}

	/// Check if we're currently downloading a snapshot.
	pub fn have_manifest(&self) -> bool {
		self.snapshot_hash.is_some()
	}

	/// Clear the `Snapshot` and reset it with data from a `ManifestData` (i.e. the lists of
	/// block&state chunk hashes contained in the `ManifestData`).
	pub fn reset_to(&mut self, manifest: &ManifestData, hash: &H256) {
		self.clear();
		self.pending_state_chunks = IndexSet::from_iter(manifest.state_hashes.clone());
		self.pending_block_chunks = IndexSet::from_iter(manifest.block_hashes.clone());
		self.total_chunks = Some(self.pending_block_chunks.len() + self.pending_state_chunks.len());
		self.snapshot_hash = Some(hash.clone());
	}

	/// Check if the the chunk is known, i.e. downloaded already or currently downloading; if so add
	/// it to the `completed_chunks` set.
	/// Returns a `ChunkType` with the hash of the chunk.
	pub fn validate_chunk(&mut self, chunk: &[u8]) -> Result<ChunkType, ()> {
		let hash = keccak(chunk);
		if self.completed_chunks.contains(&hash) {
			trace!(target: "snapshot_sync", "Already proccessed chunk {:x}. Ignoring.", hash);
			return Err(());
		}
		self.downloading_chunks.remove(&hash);

		self.pending_block_chunks.take(&hash)
			.and_then(|h| {
				self.completed_chunks.insert(h);
				Some(ChunkType::Block(hash))
			})
			.or(
				self.pending_state_chunks.take(&hash)
					.and_then(|h| {
						self.completed_chunks.insert(h);
						Some(ChunkType::State(hash))
					})
			).ok_or_else(|| {
				trace!(target: "snapshot_sync", "Ignoring unknown chunk: {:x}", hash);
				()
			})
	}

	/// Pick a chunk to download.
	/// Note: the order in which chunks are processed is somewhat important. The account state
	/// sometimes spills over into more than one chunk and the parts of state that are missing
	/// pieces are held in memory while waiting for the next chunk(s) to show up. This means that
	/// when chunks are processed out-of-order, memory usage goes up, sometimes significantly (see
	/// e.g. https://github.com/paritytech/parity-ethereum/issues/8825).
	pub fn needed_chunk(&mut self) -> Option<H256> {
		// Find next needed chunk: first block, then state chunks
		let chunk = {
			let filter = |h| !self.downloading_chunks.contains(h) && !self.completed_chunks.contains(h);
			self.pending_block_chunks.iter()
				.find(|&h| filter(h))
				.or(self.pending_state_chunks.iter()
					.find(|&h| filter(h))
				)
				.map(|h| *h)
		};
		if let Some(hash) = chunk {
			self.downloading_chunks.insert(hash.clone());
		}
		chunk
	}

	/// Remove a chunk from the set of chunks we're interested in downloading.
	pub fn clear_chunk_download(&mut self, hash: &H256) {
		self.downloading_chunks.remove(hash);
	}

	/// Mark a snapshot hash as bad.
	pub fn note_bad(&mut self, hash: H256) {
		self.bad_hashes.insert(hash);
	}

	/// Whether a snapshot hash is known to be bad.
	pub fn is_known_bad(&self, hash: &H256) -> bool {
		self.bad_hashes.contains(hash)
	}

	/// Hash of the snapshot we're currently downloading/importing.
	pub fn snapshot_hash(&self) -> Option<H256> {
		self.snapshot_hash
	}

	/// Total number of chunks in the snapshot we're currently working on (state + block chunks).
	pub fn total_chunks(&self) -> usize {
		self.total_chunks.unwrap_or_default()
	}

	/// Number of chunks we've processed so far (state and block chunks).
	pub fn done_chunks(&self) -> usize {
		self.completed_chunks.len()
	}

	/// Are we done downloading all chunks?
	pub fn is_complete(&self) -> bool {
		self.total_chunks() == self.completed_chunks.len()
	}
}

#[cfg(test)]
mod test {
	use super::{ChunkType, H256, Snapshot};

	use bytes::Bytes;
	use keccak_hash::keccak;
	use common_types::snapshot::ManifestData;

	fn is_empty(snapshot: &Snapshot) -> bool {
		snapshot.pending_block_chunks.is_empty() &&
		snapshot.pending_state_chunks.is_empty() &&
		snapshot.completed_chunks.is_empty() &&
		snapshot.downloading_chunks.is_empty() &&
		snapshot.snapshot_hash.is_none()
	}

	fn test_manifest() -> (ManifestData, H256, Vec<Bytes>, Vec<Bytes>) {
		let state_chunks: Vec<Bytes> = (0..20).map(|_| H256::random().as_bytes().to_vec()).collect();
		let block_chunks: Vec<Bytes> = (0..20).map(|_| H256::random().as_bytes().to_vec()).collect();
		let manifest = ManifestData {
			version: 2,
			state_hashes: state_chunks.iter().map(|data| keccak(data)).collect(),
			block_hashes: block_chunks.iter().map(|data| keccak(data)).collect(),
			state_root: H256::zero(),
			block_number: 42,
			block_hash: H256::zero(),
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
		assert_eq!(snapshot.done_chunks(), 0, "no chunks done at outset");
		assert!(snapshot.validate_chunk(&H256::random().as_bytes().to_vec()).is_err(), "random chunk is invalid");

		// request all 20 + 20 chunks
		let requested: Vec<H256> = (0..40).map(|_| snapshot.needed_chunk().unwrap()).collect();
		assert!(snapshot.needed_chunk().is_none(), "no chunks left after all are drained");

		let requested_all_block_chunks = manifest.block_hashes.iter()
			.all(|h| requested.iter().any(|rh| rh == h));
		assert!(requested_all_block_chunks, "all block chunks in the manifest accounted for");

		let requested_all_state_chunks = manifest.state_hashes.iter()
			.all(|h| requested.iter().any(|rh| rh == h));
		assert!(requested_all_state_chunks, "all state chunks in the manifest accounted for");

		assert_eq!(snapshot.downloading_chunks.len(), 40, "all requested chunks are downloading");

		assert_eq!(
			snapshot.validate_chunk(&state_chunks[4]),
			Ok(ChunkType::State(manifest.state_hashes[4].clone())),
			"4th state chunk hash validates as such"
		);
		assert_eq!(snapshot.completed_chunks.len(), 1, "after validating a chunk, it's in the completed set");
		assert_eq!(snapshot.downloading_chunks.len(), 39, "after validating a chunk, there's one less in the downloading set");

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

		assert!(snapshot.is_complete(), "when all chunks have been validated, we're done");
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
