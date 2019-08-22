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

//! Snapshot type definitions

use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering};

use ethereum_types::H256;
use rlp::{Rlp, RlpStream, DecoderError};
use bytes::Bytes;

/// Modes of snapshotting
pub enum Snapshotting {
	/// Snapshotting and warp sync is not supported
	Unsupported,
	/// Snapshots for proof-of-work chains
	PoW {
		/// Number of blocks from the head of the chain
		/// to include in the snapshot.
		blocks: u64,
		/// Number of blocks to allow in the snapshot when restoring.
		max_restore_blocks: u64
	},
	/// Snapshots for proof-of-authority chains
	PoA,
}

/// A progress indicator for snapshots.
#[derive(Debug, Default)]
pub struct Progress {
	/// Number of accounts processed so far
	pub accounts: AtomicUsize,
	/// Number of blocks processed so far
	pub blocks: AtomicUsize,
	/// Size in bytes of a all compressed chunks processed so far
	pub size: AtomicU64,
	/// Signals that the snapshotting process is completed
	pub done: AtomicBool,
	/// Signal snapshotting process to abort
	pub abort: AtomicBool,
}

impl Progress {
	/// Reset the progress.
	pub fn reset(&self) {
		self.accounts.store(0, Ordering::Release);
		self.blocks.store(0, Ordering::Release);
		self.size.store(0, Ordering::Release);
		self.abort.store(false, Ordering::Release);

		// atomic fence here to ensure the others are written first?
		// logs might very rarely get polluted if not.
		self.done.store(false, Ordering::Release);
	}

	/// Get the number of accounts snapshotted thus far.
	pub fn accounts(&self) -> usize { self.accounts.load(Ordering::Acquire) }

	/// Get the number of blocks snapshotted thus far.
	pub fn blocks(&self) -> usize { self.blocks.load(Ordering::Acquire) }

	/// Get the written size of the snapshot in bytes.
	pub fn size(&self) -> u64 { self.size.load(Ordering::Acquire) }

	/// Whether the snapshot is complete.
	pub fn done(&self) -> bool  { self.done.load(Ordering::Acquire) }
}

/// Manifest data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestData {
	/// Snapshot format version.
	pub version: u64,
	/// List of state chunk hashes.
	pub state_hashes: Vec<H256>,
	/// List of block chunk hashes.
	pub block_hashes: Vec<H256>,
	/// The final, expected state root.
	pub state_root: H256,
	/// Block number this snapshot was taken at.
	pub block_number: u64,
	/// Block hash this snapshot was taken at.
	pub block_hash: H256,
}

impl ManifestData {
	/// Encode the manifest data to rlp.
	pub fn into_rlp(self) -> Bytes {
		let mut stream = RlpStream::new_list(6);
		stream.append(&self.version);
		stream.append_list(&self.state_hashes);
		stream.append_list(&self.block_hashes);
		stream.append(&self.state_root);
		stream.append(&self.block_number);
		stream.append(&self.block_hash);

		stream.out()
	}

	/// Try to restore manifest data from raw bytes, interpreted as RLP.
	pub fn from_rlp(raw: &[u8]) -> Result<Self, DecoderError> {
		let decoder = Rlp::new(raw);
		let (start, version) = if decoder.item_count()? == 5 {
			(0, 1)
		} else {
			(1, decoder.val_at(0)?)
		};

		let state_hashes: Vec<H256> = decoder.list_at(start + 0)?;
		let block_hashes: Vec<H256> = decoder.list_at(start + 1)?;
		let state_root: H256 = decoder.val_at(start + 2)?;
		let block_number: u64 = decoder.val_at(start + 3)?;
		let block_hash: H256 = decoder.val_at(start + 4)?;

		Ok(ManifestData {
			version,
			state_hashes,
			block_hashes,
			state_root,
			block_number,
			block_hash,
		})
	}
}

/// A sink for produced chunks.
pub type ChunkSink<'a> = dyn FnMut(&[u8]) -> std::io::Result<()> + 'a;

/// Statuses for snapshot restoration.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RestorationStatus {
	///	No restoration.
	Inactive,
	/// Restoration is initializing
	Initializing {
		/// Total number of state chunks.
		state_chunks: u32,
		/// Total number of block chunks.
		block_chunks: u32,
		/// Number of chunks done/imported
		chunks_done: u32,
	},
	/// Ongoing restoration.
	Ongoing {
		/// Total number of state chunks.
		state_chunks: u32,
		/// Total number of block chunks.
		block_chunks: u32,
		/// Number of state chunks completed.
		state_chunks_done: u32,
		/// Number of block chunks completed.
		block_chunks_done: u32,
	},
	/// Finalizing restoration
	Finalizing,
	/// Failed restoration.
	Failed,
}
