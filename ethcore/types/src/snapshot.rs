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

use std::time::Instant;

use bytes::Bytes;
use ethereum_types::H256;
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
	accounts: u64,
	// Number of accounts processed at last tick.
	prev_accounts: u64,
	/// Number of blocks processed so far
	pub blocks: u64,
	/// Size in bytes of a all compressed chunks processed so far
	bytes: u64,
	// Number of bytes processed at last tick.
	prev_bytes: u64,
	/// Signals that the snapshotting process is completed
	pub done: bool,
	/// Signal snapshotting process to abort
	pub abort: bool,

	last_tick: Instant,
}

impl Progress {
	/// Create a new progress tracker.
	pub fn new() -> Progress {
		Progress {
			accounts: 0,
			prev_accounts: 0,
			blocks: 0,
			bytes: 0,
			prev_bytes: 0,
			abort: false,
			done: false,
			last_tick: Instant::now(),
		}
	}

	/// Get the number of accounts snapshotted thus far.
	pub fn accounts(&self) -> u64 { self.accounts }

	/// Get the number of blocks snapshotted thus far.
	pub fn blocks(&self) -> u64 { self.blocks }

	/// Get the written size of the snapshot in bytes.
	pub fn bytes(&self) -> u64 { self.bytes }

	/// Whether the snapshot is complete.
	pub fn done(&self) -> bool  { self.done }

	/// Return the progress rate over the last tick (i.e. since last update).
	pub fn rate(&self) -> (f64, f64) {
		let dt = self.last_tick.elapsed().as_secs_f64();
		if dt < 1.0 {
			return (0f64, 0f64);
		}
		let delta_acc = self.accounts.saturating_sub(self.prev_accounts);
		let delta_bytes = self.bytes.saturating_sub(self.prev_bytes);
		(delta_acc as f64 / dt, delta_bytes as f64 / dt)
	}

	/// Update state progress counters and set the last tick.
	pub fn update(&mut self, accounts_delta: u64, bytes_delta: u64) {
		self.last_tick = Instant::now();
		self.prev_accounts = self.accounts;
		self.prev_bytes = self.bytes;
		self.accounts += accounts_delta;
		self.bytes += bytes_delta;
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
