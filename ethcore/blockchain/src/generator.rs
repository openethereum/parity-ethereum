// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Blockchain generator for tests.

use std::collections::VecDeque;
use ethereum_types::{U256, H256, Bloom};

use common_types::encoded;
use common_types::header::Header;
use common_types::transaction::{SignedTransaction, Transaction, Action};
use common_types::view;
use common_types::views::BlockView;
use keccak_hash::keccak;
use rlp::encode;
use rlp_derive::RlpEncodable;
use triehash_ethereum::ordered_trie_root;

/// Helper structure, used for encoding blocks.
#[derive(Default, Clone, RlpEncodable)]
pub struct Block {
	/// Block header
	pub header: Header,
	/// Block transactions
	pub transactions: Vec<SignedTransaction>,
	/// Block uncles
	pub uncles: Vec<Header>
}

impl Block {
	/// Get a copy of the header
	#[inline]
	pub fn header(&self) -> Header {
		self.header.clone()
	}

	/// Get block hash
	#[inline]
	pub fn hash(&self) -> H256 {
		view!(BlockView, &self.encoded().raw()).header_view().hash()
	}

	/// Get block number
	#[inline]
	pub fn number(&self) -> u64 {
		self.header.number()
	}

	/// Get RLP encoding of this block
	#[inline]
	pub fn encoded(&self) -> encoded::Block {
		encoded::Block::new(encode(self))
	}

	/// Get block difficulty
	#[inline]
	pub fn difficulty(&self) -> U256 {
		*self.header.difficulty()
	}
}

/// Specify block options for generator
#[derive(Debug)]
pub struct BlockOptions {
	/// Difficulty
	pub difficulty: U256,
	/// Set bloom filter
	pub bloom: Bloom,
	/// Transactions included in blocks
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

/// Utility to create blocks
#[derive(Clone)]
pub struct BlockBuilder {
	blocks: VecDeque<Block>,
}

impl BlockBuilder {
	/// Create new BlockBuilder starting at genesis.
	pub fn genesis() -> Self {
		let mut blocks = VecDeque::with_capacity(1);
		blocks.push_back(Block::default());

		BlockBuilder {
			blocks,
		}
	}

	/// Add new block with default options.
	#[inline]
	pub fn add_block(&self) -> Self {
		self.add_block_with(|| BlockOptions::default())
	}

	/// Add `count` number of blocks with default options.
	#[inline]
	pub fn add_blocks(&self, count: usize) -> Self {
		self.add_blocks_with(count, || BlockOptions::default())
	}

	/// Add block with specified options.
	#[inline]
	pub fn add_block_with<T>(&self, get_metadata: T) -> Self where T: Fn() -> BlockOptions {
		self.add_blocks_with(1, get_metadata)
	}

	/// Add a block with given difficulty
	#[inline]
	pub fn add_block_with_difficulty<T>(&self, difficulty: T) -> Self where T: Into<U256> {
		let difficulty = difficulty.into();
		self.add_blocks_with(1, move || BlockOptions {
			difficulty,
			..Default::default()
		})
	}

	/// Add a block with randomly generated transactions.
	#[inline]
	pub fn add_block_with_random_transactions(&self) -> Self {
		// Maximum of ~50 transactions
		let count = rand::random::<u8>() as usize / 5;
		let transactions = std::iter::repeat_with(|| {
			let data_len = rand::random::<u8>();
			let data = std::iter::repeat_with(|| rand::random::<u8>())
				.take(data_len as usize)
				.collect::<Vec<_>>();
			Transaction {
				nonce: 0.into(),
				gas_price: 0.into(),
				gas: 100_000.into(),
				action: Action::Create,
				value: 100.into(),
				data,
			}.sign(&keccak("").into(), None)
		}).take(count);

		self.add_block_with_transactions(transactions)
	}

	/// Add a block with given transactions.
	#[inline]
	pub fn add_block_with_transactions<T>(&self, transactions: T) -> Self
		where T: IntoIterator<Item = SignedTransaction> {
		let transactions = transactions.into_iter().collect::<Vec<_>>();
		self.add_blocks_with(1, || BlockOptions {
			transactions: transactions.clone(),
			..Default::default()
		})
	}

	/// Add a block with given bloom filter.
	#[inline]
	pub fn add_block_with_bloom(&self, bloom: Bloom) -> Self {
		self.add_blocks_with(1, move || BlockOptions {
			bloom,
			..Default::default()
		})
	}

	/// Add a bunch of blocks with given metadata.
	pub fn add_blocks_with<T>(&self, count: usize, get_metadata: T) -> Self where T: Fn() -> BlockOptions {
		assert!(count > 0, "There must be at least 1 block");
		let mut parent_hash = self.last().hash();
		let mut parent_number = self.last().number();
		let mut blocks = VecDeque::with_capacity(count);
		for _ in 0..count {
			let mut block = Block::default();
			let metadata = get_metadata();
			let block_number = parent_number + 1;
			let transactions = metadata.transactions;
			let transactions_root = ordered_trie_root(transactions.iter().map(rlp::encode));

			block.header.set_parent_hash(parent_hash);
			block.header.set_number(block_number);
			block.header.set_log_bloom(metadata.bloom);
			block.header.set_difficulty(metadata.difficulty);
			block.header.set_transactions_root(transactions_root);
			block.transactions = transactions;

			parent_hash = block.hash();
			parent_number = block_number;

			blocks.push_back(block);
		}

		BlockBuilder {
			blocks,
		}
	}

	/// Get a reference to the last generated block.
	#[inline]
	pub fn last(&self) -> &Block {
		self.blocks.back().expect("There is always at least 1 block")
	}
}

/// Generates a blockchain from given block builders (blocks will be concatenated).
#[derive(Clone)]
pub struct BlockGenerator {
	builders: VecDeque<BlockBuilder>,
}

impl BlockGenerator {
	/// Create new block generator.
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
