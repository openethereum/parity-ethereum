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

//! Verification test helpers.

use std::collections::HashMap;

use blockchain::{BlockProvider, BlockChain, BlockDetails, TransactionAddress, BlockReceipts};
use common_types::{
	BlockNumber,
	encoded,
	verification::Unverified,
	log_entry::{LogEntry, LocalizedLogEntry},
};
use ethereum_types::{BloomRef, H256};
use parity_bytes::Bytes;

#[derive(Default)]
pub struct TestBlockChain {
	blocks: HashMap<H256, Bytes>,
	numbers: HashMap<BlockNumber, H256>,
}

impl TestBlockChain {
	pub fn new() -> Self { TestBlockChain::default() }

	pub fn insert(&mut self, bytes: Bytes) {
		let header = Unverified::from_rlp(bytes.clone()).unwrap().header;
		let hash = header.hash();
		self.blocks.insert(hash, bytes);
		self.numbers.insert(header.number(), hash);
	}
}

impl BlockProvider for TestBlockChain {
	fn is_known(&self, hash: &H256) -> bool {
		self.blocks.contains_key(hash)
	}

	fn first_block(&self) -> Option<H256> {
		unimplemented!()
	}

	fn best_ancient_block(&self) -> Option<H256> {
		None
	}

	/// Get raw block data
	fn block(&self, hash: &H256) -> Option<encoded::Block> {
		self.blocks.get(hash).cloned().map(encoded::Block::new)
	}

	/// Get the familial details concerning a block.
	fn block_details(&self, hash: &H256) -> Option<BlockDetails> {
		self.blocks.get(hash).map(|bytes| {
			let header = Unverified::from_rlp(bytes.to_vec()).unwrap().header;
			BlockDetails {
				number: header.number(),
				total_difficulty: *header.difficulty(),
				parent: *header.parent_hash(),
				children: Vec::new(),
				is_finalized: false,
			}
		})
	}

	/// Get the hash of given block's number.
	fn block_hash(&self, index: BlockNumber) -> Option<H256> {
		self.numbers.get(&index).cloned()
	}

	fn transaction_address(&self, _hash: &H256) -> Option<TransactionAddress> {
		unimplemented!()
	}

	fn block_receipts(&self, _hash: &H256) -> Option<BlockReceipts> {
		unimplemented!()
	}

	fn block_header_data(&self, hash: &H256) -> Option<encoded::Header> {
		self.block(hash)
			.map(|b| b.header_view().rlp().as_raw().to_vec())
			.map(encoded::Header::new)
	}

	fn block_body(&self, hash: &H256) -> Option<encoded::Body> {
		self.block(hash)
			.map(|b| BlockChain::block_to_body(&b.into_inner()))
			.map(encoded::Body::new)
	}

	fn blocks_with_bloom<'a, B, I, II>(&self, _blooms: II, _from_block: BlockNumber, _to_block: BlockNumber) -> Vec<BlockNumber>
		where BloomRef<'a>: From<B>, II: IntoIterator<Item = B, IntoIter = I> + Copy, I: Iterator<Item = B>, Self: Sized {
		unimplemented!()
	}

	fn logs<F>(&self, _blocks: Vec<H256>, _matches: F, _limit: Option<usize>) -> Vec<LocalizedLogEntry>
		where F: Fn(&LogEntry) -> bool, Self: Sized {
		unimplemented!()
	}
}
