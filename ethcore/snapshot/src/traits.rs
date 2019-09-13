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

use std::sync::{Arc, atomic::AtomicBool};

use blockchain::{BlockChain, BlockChainDB};
use bytes::Bytes;
use client_traits::{BlockChainClient, BlockInfo, DatabaseRestore, BlockChainReset};
use common_types::{
	ids::BlockId,
	errors::{EthcoreError as Error, SnapshotError},
	snapshot::{ManifestData, ChunkSink, Progress, RestorationStatus},
};
use engine::Engine;
use ethereum_types::H256;

use crate::io::SnapshotWriter;

/// The interface for a snapshot network service.
/// This handles:
///    - restoration of snapshots to temporary databases.
///    - responding to queries for snapshot manifests and chunks
pub trait SnapshotService : Sync + Send {
	/// Query the most recent manifest data.
	fn manifest(&self) -> Option<ManifestData>;

	/// Get the supported range of snapshot version numbers.
	/// `None` indicates warp sync isn't supported by the consensus engine.
	fn supported_versions(&self) -> Option<(u64, u64)>;

	/// Returns a list of the completed chunks
	fn completed_chunks(&self) -> Option<Vec<H256>>;

	/// Get raw chunk for a given hash.
	fn chunk(&self, hash: H256) -> Option<Bytes>;

	/// Ask the snapshot service for the restoration status.
	fn status(&self) -> RestorationStatus;

	/// Begin snapshot restoration.
	/// If restoration in-progress, this will reset it.
	/// From this point on, any previous snapshot may become unavailable.
	fn begin_restore(&self, manifest: ManifestData);

	/// Abort an in-progress restoration if there is one.
	fn abort_restore(&self);

	/// Feed a raw state chunk to the service to be processed asynchronously.
	/// no-op if not currently restoring.
	fn restore_state_chunk(&self, hash: H256, chunk: Bytes);

	/// Feed a raw block chunk to the service to be processed asynchronously.
	/// no-op if currently restoring.
	fn restore_block_chunk(&self, hash: H256, chunk: Bytes);

	/// Abort in-progress snapshotting if there is one.
	fn abort_snapshot(&self);

	/// Shutdown the Snapshot Service by aborting any ongoing restore
	fn shutdown(&self);
}

/// Restore from secondary snapshot chunks.
pub trait Rebuilder: Send {
	/// Feed a chunk, potentially out of order.
	///
	/// Check `abort_flag` periodically while doing heavy work. If set to `false`, should bail with
	/// `Error::RestorationAborted`.
	fn feed(
		&mut self,
		chunk: &[u8],
		engine: &dyn Engine,
		abort_flag: &AtomicBool,
	) -> Result<(), Error>;

	/// Finalize the restoration. Will be done after all chunks have been
	/// fed successfully.
	///
	/// This should apply the necessary "glue" between chunks,
	/// and verify against the restored state.
	fn finalize(&mut self) -> Result<(), Error>;
}

/// Components necessary for snapshot creation and restoration.
pub trait SnapshotComponents: Send {
	/// Create secondary snapshot chunks; these corroborate the state data
	/// in the state chunks.
	///
	/// Chunks shouldn't exceed the given preferred size, and should be fed
	/// uncompressed into the sink.
	///
	/// This will vary by consensus engine, so it's exposed as a trait.
	fn chunk_all(
		&mut self,
		chain: &BlockChain,
		block_at: H256,
		chunk_sink: &mut ChunkSink,
		progress: &Progress,
		preferred_size: usize,
	) -> Result<(), SnapshotError>;

	/// Create a rebuilder, which will have chunks fed into it in arbitrary
	/// order and then be finalized.
	///
	/// The manifest, a database, and fresh `BlockChain` are supplied.
	///
	/// The engine passed to the `Rebuilder` methods will be the same instance
	/// that created the `SnapshotComponents`.
	fn rebuilder(
		&self,
		chain: BlockChain,
		db: Arc<dyn BlockChainDB>,
		manifest: &ManifestData,
	) -> Result<Box<dyn Rebuilder>, Error>;

	/// Minimum supported snapshot version number.
	fn min_supported_version(&self) -> u64;

	/// Current version number
	fn current_version(&self) -> u64;
}

/// Snapshot related functionality
pub trait SnapshotClient: BlockChainClient + BlockInfo + DatabaseRestore + BlockChainReset {
	/// Take a snapshot at the given block.
	/// If the ID given is "latest", this will default to 1000 blocks behind.
	fn take_snapshot<W: SnapshotWriter + Send>(
		&self,
		writer: W,
		at: BlockId,
		p: &Progress,
	) -> Result<(), Error>;
}

/// Helper trait for broadcasting a block to take a snapshot at.
pub trait Broadcast: Send + Sync {
	/// Start a snapshot from the given block number.
	fn take_at(&self, num: Option<u64>);
}


/// Helper trait for transforming hashes to block numbers and checking if syncing.
pub trait Oracle: Send + Sync {
	/// Maps a block hash to a block number
	fn to_number(&self, hash: H256) -> Option<u64>;

	/// Are we currently syncing?
	fn is_major_importing(&self) -> bool;
}
