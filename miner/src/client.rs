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
use ethcore::client::{BlockChainClient};
use ethcore::block::OpenBlock;
use ethcore::error::ImportResult;
use ethcore::transaction::SignedTransaction;

use util::{H256, U256, Address, Bytes};

use super::{MinerBlockChain, AccountDetails};

impl<C : BlockChainClient> MinerBlockChain for C {

	fn open_block(&self, author: Address, gas_floor_target: U256, extra_data: Bytes) -> Option<OpenBlock> {
		unimplemented!()
	}

	fn import_block(&self, bytes: Bytes) -> ImportResult {
		unimplemented!()
	}

	fn block_transactions(&self, hash: &H256) -> Vec<SignedTransaction> {
		unimplemented!()
	}

	fn best_block_gas_limit(&self) -> U256 {
		unimplemented!()
	}

	fn best_block_number(&self) -> u64 {
		unimplemented!()
	}

	fn best_block_hash(&self) -> H256 {
		unimplemented!()
	}

	fn account_details(&self, address: &Address) -> AccountDetails {
		unimplemented!()
	}

	fn engine(&self) -> &Engine {
		unimplemented!()
	}
}
