
//! Bridge between Tracedb and Blockchain.

use util::{H256};
use header::BlockNumber;
use trace::DatabaseExtras as TraceDatabaseExtras;
use blockchain::{BlockChain, BlockProvider};
use blockchain::extras::TransactionAddress;
pub use types::trace_filter::Filter;

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
