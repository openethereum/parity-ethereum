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

//! Secondary chunk creation and restoration, implementation for proof-of-work
//! chains.
//!
//! The secondary chunks in this instance are 30,000 "abridged blocks" from the head
//! of the chain, which serve as an indication of valid chain.

use super::{SnapshotComponents, Rebuilder, ChunkSink};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use blockchain::{BlockChain, BlockProvider};
use engines::Engine;
use snapshot::{Error, ManifestData};
use snapshot::block::AbridgedBlock;
use bigint::hash::H256;
use util::KeyValueDB;
use bytes::Bytes;
use rlp::{RlpStream, UntrustedRlp};
use rand::OsRng;

/// Snapshot creation and restoration for PoW chains.
/// This includes blocks from the head of the chain as a
/// loose assurance that the chain is valid.
#[derive(Clone, Copy, PartialEq)]
pub struct PowSnapshot {
	/// Number of blocks from the head of the chain
	/// to include in the snapshot.
	pub blocks: u64,
	/// Number of to allow in the snapshot when restoring.
	pub max_restore_blocks: u64,
}

impl PowSnapshot {
	/// Create a new instance.
	pub fn new(blocks: u64, max_restore_blocks: u64) -> PowSnapshot {
		PowSnapshot {
			blocks: blocks,
			max_restore_blocks: max_restore_blocks,
		}
	}
}

impl SnapshotComponents for PowSnapshot {
	fn chunk_all(
		&mut self,
		chain: &BlockChain,
		block_at: H256,
		chunk_sink: &mut ChunkSink,
		preferred_size: usize,
	) -> Result<(), Error> {
		PowWorker {
			chain: chain,
			rlps: VecDeque::new(),
			current_hash: block_at,
			writer: chunk_sink,
			preferred_size: preferred_size,
		}.chunk_all(self.blocks)
	}

	fn rebuilder(
		&self,
		chain: BlockChain,
		db: Arc<KeyValueDB>,
		manifest: &ManifestData,
	) -> Result<Box<Rebuilder>, ::error::Error> {
		PowRebuilder::new(chain, db, manifest, self.max_restore_blocks).map(|r| Box::new(r) as Box<_>)
	}

	fn min_supported_version(&self) -> u64 { ::snapshot::MIN_SUPPORTED_STATE_CHUNK_VERSION }
	fn current_version(&self) -> u64 { ::snapshot::STATE_CHUNK_VERSION }
}

/// Used to build block chunks.
struct PowWorker<'a> {
	chain: &'a BlockChain,
	// block, receipt rlp pairs.
	rlps: VecDeque<Bytes>,
	current_hash: H256,
	writer: &'a mut ChunkSink<'a>,
	preferred_size: usize,
}

impl<'a> PowWorker<'a> {
	// Repeatedly fill the buffers and writes out chunks, moving backwards from starting block hash.
	// Loops until we reach the first desired block, and writes out the remainder.
	fn chunk_all(&mut self, snapshot_blocks: u64) -> Result<(), Error> {
		let mut loaded_size = 0;
		let mut last = self.current_hash;

		let genesis_hash = self.chain.genesis_hash();

		for _ in 0..snapshot_blocks {
			if self.current_hash == genesis_hash { break }

			let (block, receipts) = self.chain.block(&self.current_hash)
				.and_then(|b| self.chain.block_receipts(&self.current_hash).map(|r| (b, r)))
				.ok_or(Error::BlockNotFound(self.current_hash))?;

			let abridged_rlp = AbridgedBlock::from_block_view(&block.view()).into_inner();

			let pair = {
				let mut pair_stream = RlpStream::new_list(2);
				pair_stream.append_raw(&abridged_rlp, 1).append(&receipts);
				pair_stream.out()
			};

			let new_loaded_size = loaded_size + pair.len();

			// cut off the chunk if too large.

			if new_loaded_size > self.preferred_size && !self.rlps.is_empty() {
				self.write_chunk(last)?;
				loaded_size = pair.len();
			} else {
				loaded_size = new_loaded_size;
			}

			self.rlps.push_front(pair);

			last = self.current_hash;
			self.current_hash = block.header_view().parent_hash();
		}

		if loaded_size != 0 {
			self.write_chunk(last)?;
		}

		Ok(())
	}

	// write out the data in the buffers to a chunk on disk
	//
	// we preface each chunk with the parent of the first block's details,
	// obtained from the details of the last block written.
	fn write_chunk(&mut self, last: H256) -> Result<(), Error> {
		trace!(target: "snapshot", "prepared block chunk with {} blocks", self.rlps.len());

		let (last_header, last_details) = self.chain.block_header(&last)
			.and_then(|n| self.chain.block_details(&last).map(|d| (n, d)))
			.ok_or(Error::BlockNotFound(last))?;

		let parent_number = last_header.number() - 1;
		let parent_hash = last_header.parent_hash();
		let parent_total_difficulty = last_details.total_difficulty - *last_header.difficulty();

		trace!(target: "snapshot", "parent last written block: {}", parent_hash);

		let num_entries = self.rlps.len();
		let mut rlp_stream = RlpStream::new_list(3 + num_entries);
		rlp_stream.append(&parent_number).append(parent_hash).append(&parent_total_difficulty);

		for pair in self.rlps.drain(..) {
			rlp_stream.append_raw(&pair, 1);
		}

		let raw_data = rlp_stream.out();

		(self.writer)(&raw_data)?;

		Ok(())
	}
}

/// Rebuilder for proof-of-work chains.
/// Does basic verification for all blocks, but `PoW` verification for some.
/// Blocks must be fed in-order.
///
/// The first block in every chunk is disconnected from the last block in the
/// chunk before it, as chunks may be submitted out-of-order.
///
/// After all chunks have been submitted, we "glue" the chunks together.
pub struct PowRebuilder {
	chain: BlockChain,
	db: Arc<KeyValueDB>,
	rng: OsRng,
	disconnected: Vec<(u64, H256)>,
	best_number: u64,
	best_hash: H256,
	best_root: H256,
	fed_blocks: u64,
	snapshot_blocks: u64,
}

impl PowRebuilder {
	/// Create a new PowRebuilder.
	fn new(chain: BlockChain, db: Arc<KeyValueDB>, manifest: &ManifestData, snapshot_blocks: u64) -> Result<Self, ::error::Error> {
		Ok(PowRebuilder {
			chain: chain,
			db: db,
			rng: OsRng::new()?,
			disconnected: Vec::new(),
			best_number: manifest.block_number,
			best_hash: manifest.block_hash,
			best_root: manifest.state_root,
			fed_blocks: 0,
			snapshot_blocks: snapshot_blocks,
		})
	}
}

impl Rebuilder for PowRebuilder {
	/// Feed the rebuilder an uncompressed block chunk.
	/// Returns the number of blocks fed or any errors.
	fn feed(&mut self, chunk: &[u8], engine: &Engine, abort_flag: &AtomicBool) -> Result<(), ::error::Error> {
		use basic_types::Seal::With;
		use views::BlockView;
		use snapshot::verify_old_block;
		use bigint::prelude::U256;
		use triehash::ordered_trie_root;

		let rlp = UntrustedRlp::new(chunk);
		let item_count = rlp.item_count()?;
		let num_blocks = (item_count - 3) as u64;

		trace!(target: "snapshot", "restoring block chunk with {} blocks.", item_count - 3);

		if self.fed_blocks + num_blocks > self.snapshot_blocks {
			return Err(Error::TooManyBlocks(self.snapshot_blocks, self.fed_blocks + num_blocks).into())
		}

		// todo: assert here that these values are consistent with chunks being in order.
		let mut cur_number = rlp.val_at::<u64>(0)? + 1;
		let mut parent_hash = rlp.val_at::<H256>(1)?;
		let parent_total_difficulty = rlp.val_at::<U256>(2)?;

		for idx in 3..item_count {
			if !abort_flag.load(Ordering::SeqCst) { return Err(Error::RestorationAborted.into()) }

			let pair = rlp.at(idx)?;
			let abridged_rlp = pair.at(0)?.as_raw().to_owned();
			let abridged_block = AbridgedBlock::from_raw(abridged_rlp);
			let receipts: Vec<::receipt::Receipt> = pair.list_at(1)?;
			let receipts_root = ordered_trie_root(
				pair.at(1)?.iter().map(|r| r.as_raw().to_owned())
			);

			let block = abridged_block.to_block(parent_hash, cur_number, receipts_root)?;
			let block_bytes = block.rlp_bytes(With);
			let is_best = cur_number == self.best_number;

			if is_best {
				if block.header.hash() != self.best_hash {
					return Err(Error::WrongBlockHash(cur_number, self.best_hash, block.header.hash()).into())
				}

				if block.header.state_root() != &self.best_root {
					return Err(Error::WrongStateRoot(self.best_root, *block.header.state_root()).into())
				}
			}

			verify_old_block(
				&mut self.rng,
				&block.header,
				engine,
				&self.chain,
				Some(&block_bytes),
				is_best
			)?;

			let mut batch = self.db.transaction();

			// special-case the first block in each chunk.
			if idx == 3 {
				if self.chain.insert_unordered_block(&mut batch, &block_bytes, receipts, Some(parent_total_difficulty), is_best, false) {
					self.disconnected.push((cur_number, block.header.hash()));
				}
			} else {
				self.chain.insert_unordered_block(&mut batch, &block_bytes, receipts, None, is_best, false);
			}
			self.db.write_buffered(batch);
			self.chain.commit();

			parent_hash = BlockView::new(&block_bytes).hash();
			cur_number += 1;
		}

		self.fed_blocks += num_blocks;

		Ok(())
	}

	/// Glue together any disconnected chunks and check that the chain is complete.
	fn finalize(&mut self, _: &Engine) -> Result<(), ::error::Error> {
		let mut batch = self.db.transaction();

		for (first_num, first_hash) in self.disconnected.drain(..) {
			let parent_num = first_num - 1;

			// check if the parent is even in the chain.
			// since we don't restore every single block in the chain,
			// the first block of the first chunks has nothing to connect to.
			if let Some(parent_hash) = self.chain.block_hash(parent_num) {
				// if so, add the child to it.
				self.chain.add_child(&mut batch, parent_hash, first_hash);
			}
		}

		let genesis_hash = self.chain.genesis_hash();
		self.chain.insert_epoch_transition(&mut batch, 0, ::engines::EpochTransition {
			block_number: 0,
			block_hash: genesis_hash,
			proof: vec![],
		});

		self.db.write_buffered(batch);
		Ok(())
	}
}
