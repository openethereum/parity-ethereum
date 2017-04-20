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

//! Secondary chunk creation and restoration, implementation for proof-of-authority
//! based engines.
//!
//! The chunks here contain state proofs of transitions, along with validator proofs.

use super::{SnapshotComponents, Rebuilder, ChunkSink};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use blockchain::{BlockChain, BlockProvider};
use engines::Engine;
use ids::BlockId;
use snapshot::{Error, ManifestData};
use snapshot::block::AbridgedBlock;
use util::{Bytes, H256, KeyValueDB};
use rlp::{RlpStream, UntrustedRlp};

/// Snapshot creation and restoration for PoA chains.
/// Chunk format:
///
/// [[abridged, epoch data, state proof, last hashes], ...]
///   - Abridged block at which transition occurred,
///   - epoch data (list of validators)
///   - state items required to check epoch data
pub struct PoaSnapshot;

impl SnapshotComponents for PoaSnapshot {
	fn chunk_all(
		&mut self,
		chain: &BlockChain,
		block_at: H256,
		chunk_sink: &mut ChunkSink,
		preferred_size: usize,
	) -> Result<(), Error> {
		let number = chain.block_number(block_at)
			.ok_or_else(|| Error::InvalidStartingBlock(BlockId::Hash(block_at)))?;

		let mut written_size = 0;
		for transition in chain.epoch_transitions().take_while(|t| t.block_number <= number) {

		}
	}

	fn rebuilder(
		&self,
		chain: BlockChain,
		db: Arc<KeyValueDB>,
		manifest: &ManifestData,
	) -> Result<Box<Rebuilder>, ::error::Error> {
		PowRebuilder::new(chain, db, manifest).map(|r| Box::new(r) as Box<_>)
	}
}
