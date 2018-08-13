// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Secondary chunk creation and restoration, implementations for different consensus
//! engines.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use types::{BlockNumber, receipt::Receipt};
use blockchain::{BlockChain, BlockChainDB, BlockProvider};
use encoded;
use engines::{EthEngine, EpochTransition};
use header::Header;
use snapshot::{Error, ManifestData};

use ethereum_types::{H256, U256};
use kvdb::DBTransaction;

mod authority;
mod work;

pub use self::authority::*;
pub use self::work::*;

/// A sink for produced chunks.
pub type ChunkSink<'a> = FnMut(&[u8]) -> ::std::io::Result<()> + 'a;

/// Components necessary for snapshot creation and restoration.
pub trait SnapshotComponents: RebuilderFactory + Send {
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
		preferred_size: usize,
	) -> Result<(), Error>;
}

/// A common trait for `BlockChain` (full node) and `HeaderChain` (light client)
/// used in snapshot restoration.
pub trait RestorationTargetChain: Send {
	/// Returns reference to genesis hash.
	fn genesis_hash(&self) -> H256;

	/// Returns the header of the genesis block.
	fn genesis_header(&self) -> Header;

	/// Get the hash of given block's number.
	fn block_hash(&self, index: BlockNumber) -> Option<H256>;

	/// Get the header of a block.
	fn block_header_data(&self, hash: &H256) -> Option<Header>;

	/// Add a child to a given block. Assumes that the block hash is in
	/// the chain and the child's parent is this block.
	///
	/// Used in snapshots to glue the chunks together at the end.
	fn add_child(&self, batch: &mut DBTransaction, block_hash: H256, child_hash: H256);

	/// Insert an epoch transition. Provide an epoch number being transitioned to
	/// and epoch transition object.
	///
	/// The block the transition occurred at should have already been inserted into the chain.
	fn insert_epoch_transition(
		&self,
		batch: &mut DBTransaction,
		header: Header,
		transition: EpochTransition,
	);

	/// Inserts a verified, known block from the canonical chain.
	///
	/// Can be performed out-of-order, but care must be taken that the final chain is in a correct state.
	/// This is used by snapshot restoration and when downloading missing blocks for the chain gap.
	/// `is_best` forces the best block to be updated to this block.
	/// `is_ancient` forces the best block of the first block sequence to be updated to this block.
	/// `parent_td` is a parent total diffuculty
	/// Supply a dummy parent total difficulty when the parent block may not be in the chain.
	/// Returns true if the block is disconnected.
	fn insert_unordered_block(
		&self,
		batch: &mut DBTransaction,
		block: encoded::Block,
		receipts: Vec<Receipt>,
		parent_td: Option<U256>,
		is_best: bool,
		is_ancient: bool,
	) -> bool;

	/// Apply pending insertion updates.
	fn commit(&self);
}

/// A factory producing `Rebuilder`s needed for snapshot restoration.
pub trait RebuilderFactory: Send {
	/// Create a `Rebuilder`, which will have chunks fed into it in arbitrary
	/// order and then be finalized.
	///
	/// The manifest, a database, and fresh `RestorationTargetChain` are supplied.
	///
	/// The engine passed to the `Rebuilder` methods will be the same instance
	/// that created the `SnapshotComponents`.
	fn rebuilder(
		&self,
		chain: Box<RestorationTargetChain>,
		db: Arc<BlockChainDB>,
		manifest: &ManifestData,
	) -> Result<Box<Rebuilder>, ::error::Error>;

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
		engine: &EthEngine,
		abort_flag: &AtomicBool,
	) -> Result<(), ::error::Error>;

	/// Finalize the restoration. Will be done after all chunks have been
	/// fed successfully.
	///
	/// This should apply the necessary "glue" between chunks,
	/// and verify against the restored state.
	fn finalize(&mut self, engine: &EthEngine) -> Result<(), ::error::Error>;
}

impl RestorationTargetChain for BlockChain {
	fn genesis_hash(&self) -> H256 {
		BlockProvider::genesis_hash(self)
	}

	fn genesis_header(&self) -> Header {
		BlockProvider::genesis_header(self).decode().expect("genesis header is always decodable; qed")
	}

	fn block_hash(&self, index: BlockNumber) -> Option<H256> {
		BlockProvider::block_hash(self, index)
	}

	fn block_header_data(&self, hash: &H256) -> Option<Header> {
		BlockProvider::block_header_data(self, hash).and_then(|h| h.decode().ok())
	}

	fn add_child(&self, batch: &mut DBTransaction, block_hash: H256, child_hash: H256) {
		BlockChain::add_child(self, batch, block_hash, child_hash);
	}

	fn insert_epoch_transition(
		&self, batch: &mut DBTransaction,
		header: Header, transition: EpochTransition
	) {
		BlockChain::insert_epoch_transition(self, batch, header.number(), transition);
	}

	fn insert_unordered_block(
		&self,
		batch: &mut DBTransaction,
		block: encoded::Block,
		receipts: Vec<Receipt>,
		parent_td: Option<U256>,
		is_best: bool,
		is_ancient: bool,
	) -> bool {
		BlockChain::insert_unordered_block(self, batch, block, receipts, parent_td, is_best, is_ancient)
	}

	fn commit(&self) {
		BlockChain::commit(self);
	}
}
