// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use ethcore::engine::Engine;
use ethcore::client::{BlockChainClient, BlockId};
use ethcore::block::OpenBlock;
use ethcore::error::ImportResult;
use ethcore::transaction::SignedTransaction;
use ethcore::views::{BlockView, HeaderView};

use util::{H256, U256, Address, Bytes};

use super::{MinerBlockChain, AccountDetails};

impl<C : BlockChainClient> MinerBlockChain for C {

	fn open_block(&self, author: Address, gas_floor_target: U256, extra_data: Bytes) -> Option<OpenBlock> {
		BlockChainClient::open_block(self, author, gas_floor_target, extra_data)
	}

	fn import_block(&self, bytes: Bytes) -> ImportResult {
		BlockChainClient::import_block(self, bytes)
	}

	fn block_transactions(&self, hash: &H256) -> Vec<SignedTransaction> {
		let block = self
				.block(BlockId::Hash(*hash))
				// Client should send message after commit to db and inserting to chain.
				.expect("Expected in-chain blocks.");
		let block = BlockView::new(&block);
		block.transactions()
	}

	fn best_block_gas_limit(&self) -> U256 {
		HeaderView::new(&self.best_block_header()).gas_limit()
	}

	fn best_block_number(&self) -> u64 {
		self.chain_info().best_block_number
	}

	fn best_block_hash(&self) -> H256 {
		self.chain_info().best_block_hash
	}

	fn account_details(&self, address: &Address) -> AccountDetails {
		AccountDetails {
			nonce: self.nonce(address),
			balance: self.balance(address),
		}
	}

	fn engine(&self) -> &Engine {
		BlockChainClient::engine(self)
	}
}
