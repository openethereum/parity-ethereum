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


use util::*;
use ethcore::snapshot::ManifestData;

pub enum ChunkType {
	State(H256),
	Block(H256),
}

pub struct Snapshot {
	/// Heads of subchains to download
	pending_state_chunks: Vec<H256>,
	pending_block_chunks: Vec<H256>,
	/// Set of snapshot chunks being downloaded.
	downloading_chunks: HashSet<H256>,
	completed_chunks: HashSet<H256>,
	snapshot_hash: Option<H256>,
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
		}
	}

	/// Clear everything.
	pub fn clear(&mut self) {
		self.pending_state_chunks.clear();
		self.pending_block_chunks.clear();
		self.downloading_chunks.clear();
		self.completed_chunks.clear();
		self.snapshot_hash = None;
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
		let hash = chunk.sha3();
		if self.completed_chunks.contains(&hash) {
			trace!(target: "sync", "Ignored proccessed chunk: {}", hash.hex());
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
		trace!(target: "sync", "Ignored unknown chunk: {}", hash.hex());
		Err(())
	}

	/// Find a chunk to download
	pub fn needed_chunk(&mut self) -> Option<H256> {
		// check state chunks first
		let mut chunk = self.pending_state_chunks.iter()
			.find(|&h| !self.downloading_chunks.contains(h))
			.cloned();
		if chunk.is_none() {
			chunk = self.pending_block_chunks.iter()
				.find(|&h| !self.downloading_chunks.contains(h))
				.cloned();
		}

		if let Some(hash) = chunk {
			self.downloading_chunks.insert(hash.clone());
		}
		chunk
	}

	pub fn clear_chunk_download(&mut self, hash: &H256) {
		self.downloading_chunks.remove(hash);
	}

	pub fn snapshot_hash(&self) -> Option<H256> {
		self.snapshot_hash
	}

	pub fn total_chunks(&self) -> usize {
		self.pending_block_chunks.len() + self.pending_state_chunks.len()
	}

	pub fn done_chunks(&self) -> usize {
		self.total_chunks() - self.completed_chunks.len()
	}

	pub fn is_complete(&self) -> bool {
		self.is_empty()
	}
	pub fn is_empty(&self) -> bool {
		self.pending_state_chunks.is_empty() && self.pending_block_chunks.is_empty()
	}
}

#[cfg(test)]
mod test {
}

