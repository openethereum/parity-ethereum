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

//! Blockchain generator for tests.

use std::collections::VecDeque;
use ethereum_types::{U256, H256, Bloom};

use bytes::Bytes;
use header::Header;
use rlp::encode;
use transaction::SignedTransaction;
use views::BlockView;

/// Helper structure, used for encoding blocks.
#[derive(Default, Clone, RlpEncodable)]
pub struct Block {
	pub header: Header,
	pub transactions: Vec<SignedTransaction>,
	pub uncles: Vec<Header>
}

impl Block {
	#[inline]
	pub fn header(&self) -> Header {
		self.header.clone()
	}

	#[inline]
	pub fn hash(&self) -> H256 {
		view!(BlockView, &self.encoded()).header_view().hash()
	}

	#[inline]
	pub fn number(&self) -> u64 {
		self.header.number()
	}

	#[inline]
	pub fn encoded(&self) -> Bytes {
		encode(self).into_vec()
	}

	#[inline]
	pub fn difficulty(&self) -> U256 {
		*self.header.difficulty()
	}
}

#[derive(Debug)]
pub struct BlockOptions {
	pub difficulty: U256,
	pub bloom: Bloom,
	pub transactions: Vec<SignedTransaction>,
}

impl Default for BlockOptions {
	fn default() -> Self {
		BlockOptions {
			difficulty: 10.into(),
			bloom: Bloom::default(),
			transactions: Vec::new(),
		}
	}
}

#[derive(Clone)]
pub struct BlockBuilder {
	blocks: VecDeque<Block>,
}

impl BlockBuilder {
	pub fn genesis() -> Self {
		let mut blocks = VecDeque::with_capacity(1);
		blocks.push_back(Block::default());

		BlockBuilder {
			blocks,
		}
	}

	#[inline]
	pub fn add_block(&self) -> Self {
		self.add_block_with(|| BlockOptions::default())
	}

	#[inline]
	pub fn add_blocks(&self, count: usize) -> Self {
		self.add_blocks_with(count, || BlockOptions::default())
	}

	#[inline]
	pub fn add_block_with<T>(&self, get_metadata: T) -> Self where T: Fn() -> BlockOptions {
		self.add_blocks_with(1, get_metadata)
	}

	#[inline]
	pub fn add_block_with_difficulty<T>(&self, difficulty: T) -> Self where T: Into<U256> {
		let difficulty = difficulty.into();
		self.add_blocks_with(1, move || BlockOptions {
			difficulty,
			..Default::default()
		})
	}

	#[inline]
	pub fn add_block_with_transactions<T>(&self, transactions: T) -> Self
		where T: IntoIterator<Item = SignedTransaction> {
		let transactions = transactions.into_iter().collect::<Vec<_>>();
		self.add_blocks_with(1, || BlockOptions {
			transactions: transactions.clone(),
			..Default::default()
		})
	}

	#[inline]
	pub fn add_block_with_bloom(&self, bloom: Bloom) -> Self {
		self.add_blocks_with(1, move || BlockOptions {
			bloom,
			..Default::default()
		})
	}

	pub fn add_blocks_with<T>(&self, count: usize, get_metadata: T) -> Self where T: Fn() -> BlockOptions {
		assert!(count > 0, "There must be at least 1 block");
		let mut parent_hash = self.last().hash();
		let mut parent_number = self.last().number();
		let mut blocks = VecDeque::with_capacity(count);
		for _ in 0..count {
			let mut block = Block::default();
			let metadata = get_metadata();
			let block_number = parent_number + 1;
			block.header.set_parent_hash(parent_hash);
			block.header.set_number(block_number);
			block.header.set_log_bloom(metadata.bloom);
			block.header.set_difficulty(metadata.difficulty);
			block.transactions = metadata.transactions;

			parent_hash = block.hash();
			parent_number = block_number;

			blocks.push_back(block);
		}

		BlockBuilder {
			blocks,
		}
	}

	#[inline]
	pub fn last(&self) -> &Block {
		self.blocks.back().expect("There is always at least 1 block")
	}
}

#[derive(Clone)]
pub struct BlockGenerator {
	builders: VecDeque<BlockBuilder>,
}

impl BlockGenerator {
	pub fn new<T>(builders: T) -> Self where T: IntoIterator<Item = BlockBuilder> {
		BlockGenerator {
			builders: builders.into_iter().collect(),
		}
	}
}

impl Iterator for BlockGenerator {
	type Item = Block;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.builders.front_mut() {
				Some(ref mut builder) => {
					if let Some(block) = builder.blocks.pop_front() {
						return Some(block);
					}
				},
				None => return None,
			}
			self.builders.pop_front();
		}

	}
}

#[cfg(test)]
mod tests {
	use super::{BlockBuilder, BlockOptions, BlockGenerator};

	#[test]
	fn test_block_builder() {
		let genesis = BlockBuilder::genesis();
		let block_1 = genesis.add_block();
		let block_1001 = block_1.add_blocks(1000);
		let block_1002 = block_1001.add_block_with(|| BlockOptions::default());
		let generator = BlockGenerator::new(vec![genesis, block_1, block_1001, block_1002]);
		assert_eq!(generator.count(), 1003);
	}

	#[test]
	fn test_block_builder_fork() {
		let genesis = BlockBuilder::genesis();
		let block_10a = genesis.add_blocks(10);
		let block_11b = genesis.add_blocks(11);
		assert_eq!(block_10a.last().number(), 10);
		assert_eq!(block_11b.last().number(), 11);
	}
}
