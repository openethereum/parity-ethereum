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

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use bytes::Bytes;
use ethereum_types::H256;
use parking_lot::RwLock;
use rlp::{Rlp, RlpStream, DecoderError};

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
#[derive(Debug)]
pub struct Progress {
	/// Number of accounts processed so far
	accounts: AtomicUsize,
	// Number of accounts processed at last tick.
	prev_accounts: AtomicUsize,
	/// Number of blocks processed so far
	pub blocks: AtomicUsize,
	/// Size in bytes of a all compressed chunks processed so far
	bytes: AtomicUsize,
	// Number of bytes processed at last tick.
	prev_bytes: AtomicUsize,
	/// Signals that the snapshotting process is completed
	pub done: AtomicBool,
	/// Signal snapshotting process to abort
	pub abort: AtomicBool,

	last_tick: RwLock<Instant>,
}

impl Progress {
	/// Create a new progress tracker.
	pub fn new() -> Progress {
		Progress {
			accounts: AtomicUsize::new(0),
			prev_accounts: AtomicUsize::new(0),
			blocks: AtomicUsize::new(0),
			bytes: AtomicUsize::new(0),
			prev_bytes: AtomicUsize::new(0),
			abort: AtomicBool::new(false),
			done: AtomicBool::new(false),
			last_tick: RwLock::new(Instant::now()),
		}
	}

	/// Reset the progress.
	pub fn reset(&self) {
		self.accounts.store(0, Ordering::Release);
		self.blocks.store(0, Ordering::Release);
		self.bytes.store(0, Ordering::Release);
		self.abort.store(false, Ordering::Release);

		// atomic fence here to ensure the others are written first?
		// logs might very rarely get polluted if not.
		self.done.store(false, Ordering::Release);

		*self.last_tick.write() = Instant::now();
	}

	/// Get the number of accounts snapshotted thus far.
	pub fn accounts(&self) -> usize { self.accounts.load(Ordering::Acquire) }

	/// Get the number of blocks snapshotted thus far.
	pub fn blocks(&self) -> usize { self.blocks.load(Ordering::Acquire) }

	/// Get the written size of the snapshot in bytes.
	pub fn bytes(&self) -> usize { self.bytes.load(Ordering::Acquire) }

	/// Whether the snapshot is complete.
	pub fn done(&self) -> bool  { self.done.load(Ordering::Acquire) }

	/// Return the progress rate over the last tick (i.e. since last update).
	pub fn rate(&self) -> (f64, f64) {
		let last_tick = *self.last_tick.read();
		let dt = last_tick.elapsed().as_secs_f64();
		if dt < 1.0 {
			return (0f64, 0f64);
		}
		let delta_acc = self.accounts.load(Ordering::Relaxed)
			.saturating_sub(self.prev_accounts.load(Ordering::Relaxed));
		let delta_bytes = self.bytes.load(Ordering::Relaxed)
			.saturating_sub(self.prev_bytes.load(Ordering::Relaxed));
		(delta_acc as f64 / dt, delta_bytes as f64 / dt)
	}

	/// Update state progress counters and set the last tick.
	pub fn update(&self, accounts_delta: usize, bytes_delta: usize) {
		*self.last_tick.write() = Instant::now();
		self.prev_accounts.store(
			self.accounts.fetch_add(accounts_delta, Ordering::SeqCst),
			Ordering::SeqCst
		);
		self.prev_bytes.store(
			self.bytes.fetch_add(bytes_delta, Ordering::SeqCst),
			Ordering::SeqCst
		);
	}
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
