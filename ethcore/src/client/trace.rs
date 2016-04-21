
//! Bridge between Tracedb and Blockchain.

use util::H256;
use header::BlockNumber;
use trace::DatabaseExtras as TraceDatabaseExtras;
use blockchain::BlockChain;

impl TraceDatabaseExtras for BlockChain {
	fn block_hash(&self, block_number: BlockNumber) -> Option<H256> {
		unimplemented!();
	}

	fn transaction_hash(&self, block_number: BlockNumber, tx_position: usize) -> Option<H256> {
		unimplemented!();
	}
}
