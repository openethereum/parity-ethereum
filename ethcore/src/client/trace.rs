
//! Bridge between Tracedb and Blockchain.

use std::ops::Range;
use util::{Address, H256};
use header::BlockNumber;
use trace::DatabaseExtras as TraceDatabaseExtras;
use blockchain::{BlockChain, BlockProvider};
use extras::TransactionAddress;
use super::BlockId;

impl TraceDatabaseExtras for BlockChain {
	fn block_hash(&self, block_number: BlockNumber) -> Option<H256> {
		(self as &BlockProvider).block_hash(block_number)
	}

	fn transaction_hash(&self, block_number: BlockNumber, tx_position: usize) -> Option<H256> {
		(self as &BlockProvider).block_hash(block_number)
			.and_then(|block_hash| {
				let tx_address = TransactionAddress {
					block_hash: block_hash,
					index: tx_position
				};
				self.transaction(&tx_address)
			})
			.map(|tx| tx.hash())
	}
}

/// Easy to use trace filter.
pub struct Filter {
	/// Range of filtering.
	pub range: Range<BlockId>,
	/// From address.
	pub from_address: Vec<Address>,
	/// To address.
	pub to_address: Vec<Address>,
}
