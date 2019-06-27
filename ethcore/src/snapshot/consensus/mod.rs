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

//! Secondary chunk creation and restoration, implementations for different consensus
//! engines.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use blockchain::{BlockChain, BlockChainDB};
use engines::Engine;
use snapshot::{Error, ManifestData, Progress};

use ethereum_types::H256;

mod authority;
mod work;

pub use self::authority::*;
pub use self::work::*;

/// A sink for produced chunks.
pub type ChunkSink<'a> = dyn FnMut(&[u8]) -> ::std::io::Result<()> + 'a;

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
	) -> Result<(), Error>;

	/// Create a rebuilder, which will have chunks fed into it in aribtrary
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
	) -> Result<Box<dyn Rebuilder>, ::error::Error>;

	/// Minimum supported snapshot version number.
	fn min_supported_version(&self) -> u64;

	/// Current version number
	fn current_version(&self) -> u64;
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
	) -> Result<(), ::error::Error>;

	/// Finalize the restoration. Will be done after all chunks have been
	/// fed successfully.
	///
	/// This should apply the necessary "glue" between chunks,
	/// and verify against the restored state.
	fn finalize(&mut self, engine: &dyn Engine) -> Result<(), ::error::Error>;
}
