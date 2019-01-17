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

//! TODO: Something something blockchain info

use ethereum_types::{H256, Address};
use types::blockchain_info::BlockChainInfo;
use types::encoded;
use types::ids::{BlockId, TransactionId};
use types::header::Header;

/// Provides `chain_info` method
pub trait ChainInfo {
	/// Get blockchain information.
	fn chain_info(&self) -> BlockChainInfo;
}

/// Provides various information on a block by it's ID
pub trait BlockInfo {
	/// Get raw block header data by block id.
	fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

	/// Get the best block header.
	fn best_block_header(&self) -> Header;

	/// Get raw block data by block header hash.
	fn block(&self, id: BlockId) -> Option<encoded::Block>;

	/// Get address code hash at given block's state.
	fn code_hash(&self, address: &Address, id: BlockId) -> Option<H256>;
}

/// Provides various information on a transaction by it's ID
pub trait TransactionInfo {
	/// Get the hash of block that contains the transaction, if any.
	fn transaction_block(&self, id: TransactionId) -> Option<H256>;
}
