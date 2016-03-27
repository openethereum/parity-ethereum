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

//! Test implementation of miner service.

use util::{Address, H256, Bytes};
use util::standard::*;
use ethcore::error::Error;
use ethcore::client::BlockChainClient;
use ethcore::block::ClosedBlock;
use ethcore::transaction::SignedTransaction;
use ethminer::{MinerService, MinerStatus, AccountDetails};

/// Test miner service.
pub struct TestMinerService {
	/// Imported transactions.
	pub imported_transactions: RwLock<Vec<H256>>,
	/// Latest closed block.
	pub latest_closed_block: Mutex<Option<ClosedBlock>>,
	/// Pre-existed pending transactions
	pub pending_transactions: Mutex<HashMap<H256, SignedTransaction>>,
}

impl Default for TestMinerService {
	fn default() -> TestMinerService {
		TestMinerService {
			imported_transactions: RwLock::new(Vec::new()),
			latest_closed_block: Mutex::new(None),
			pending_transactions: Mutex::new(HashMap::new()),
		}
	}
}

impl MinerService for TestMinerService {

	/// Returns miner's status.
	fn status(&self) -> MinerStatus {
		MinerStatus {
			transactions_in_pending_queue: 0,
			transactions_in_future_queue: 0,
			transactions_in_pending_block: 1
		}
	}

	/// Imports transactions to transaction queue.
	fn import_transactions<T>(&self, _transactions: Vec<SignedTransaction>, _fetch_account: T) -> Vec<Result<(), Error>>
		where T: Fn(&Address) -> AccountDetails { unimplemented!(); }

	/// Returns hashes of transactions currently in pending
	fn pending_transactions_hashes(&self) -> Vec<H256> { vec![] }

	/// Removes all transactions from the queue and restart mining operation.
	fn clear_and_reset(&self, _chain: &BlockChainClient) { unimplemented!(); }

	/// Called when blocks are imported to chain, updates transactions queue.
	fn chain_new_blocks(&self, _chain: &BlockChainClient, _imported: &[H256], _invalid: &[H256], _enacted: &[H256], _retracted: &[H256]) { unimplemented!(); }

	/// New chain head event. Restart mining operation.
	fn update_sealing(&self, _chain: &BlockChainClient) { unimplemented!(); }

	fn map_sealing_work<F, T>(&self, _chain: &BlockChainClient, _f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T { unimplemented!(); }

	fn transaction(&self, hash: &H256) -> Option<SignedTransaction> {
		self.pending_transactions.lock().unwrap().get(hash).and_then(|tx_ref| Some(tx_ref.clone()))
	}

	/// Submit `seal` as a valid solution for the header of `pow_hash`.
	/// Will check the seal, but not actually insert the block into the chain.
	fn submit_seal(&self, _chain: &BlockChainClient, _pow_hash: H256, _seal: Vec<Bytes>) -> Result<(), Error> { unimplemented!(); }
}
