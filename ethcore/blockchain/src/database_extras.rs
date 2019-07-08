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

//! Provides a `DatabaseExtras` trait that defines an interface to query for block data not
//! contained in a TraceDB.

use common_types::BlockNumber;
use ethereum_types::H256;
use ethcore_db::keys::TransactionAddress;

use crate::blockchain::{BlockProvider, BlockChain};

/// `DatabaseExtras` provides an interface to query extra data which is not stored in TraceDB,
/// but necessary to work correctly.
pub trait DatabaseExtras {
	/// Returns hash of given block number.
	fn block_hash(&self, block_number: BlockNumber) -> Option<H256>;

	/// Returns hash of transaction at given position.
	fn transaction_hash(&self, block_number: BlockNumber, tx_position: usize) -> Option<H256>;
}

/// Bridge between TraceDB and Blockchain.
impl DatabaseExtras for BlockChain {
	fn block_hash(&self, block_number: BlockNumber) -> Option<H256> {
		(self as &dyn BlockProvider).block_hash(block_number)
	}

	fn transaction_hash(&self, block_number: BlockNumber, tx_position: usize) -> Option<H256> {
		(self as &dyn BlockProvider).block_hash(block_number)
			.and_then(|block_hash| {
				let tx_address = TransactionAddress {
					block_hash,
					index: tx_position
				};
				self.transaction(&tx_address)
			})
			.map(|tx| tx.hash())
	}
}
