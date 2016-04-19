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

use util::{Address, H256, Bytes, U256, FixedHash};
use util::standard::*;
use ethcore::error::Error;
use ethcore::client::BlockChainClient;
use ethcore::block::ClosedBlock;
use ethcore::transaction::SignedTransaction;
use ethminer::{MinerService, MinerStatus, AccountDetails, TransactionImportResult};

/// Test miner service.
pub struct TestMinerService {
	/// Imported transactions.
	pub imported_transactions: Mutex<Vec<SignedTransaction>>,
	/// Latest closed block.
	pub latest_closed_block: Mutex<Option<ClosedBlock>>,
	/// Pre-existed pending transactions
	pub pending_transactions: Mutex<HashMap<H256, SignedTransaction>>,
	/// Last nonces.
	pub last_nonces: RwLock<HashMap<Address, U256>>,

	min_gas_price: RwLock<U256>,
	gas_floor_target: RwLock<U256>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,
	limit: RwLock<usize>,
}

impl Default for TestMinerService {
	fn default() -> TestMinerService {
		TestMinerService {
			imported_transactions: Mutex::new(Vec::new()),
			latest_closed_block: Mutex::new(None),
			pending_transactions: Mutex::new(HashMap::new()),
			last_nonces: RwLock::new(HashMap::new()),
			min_gas_price: RwLock::new(U256::from(20_000_000)),
			gas_floor_target: RwLock::new(U256::from(12345)),
			author: RwLock::new(Address::zero()),
			extra_data: RwLock::new(vec![1, 2, 3, 4]),
			limit: RwLock::new(1024),
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

	fn set_author(&self, author: Address) {
		*self.author.write().unwrap() = author;
	}

	fn set_extra_data(&self, extra_data: Bytes) {
		*self.extra_data.write().unwrap() = extra_data;
	}

	/// Set the gas limit we wish to target when sealing a new block.
	fn set_gas_floor_target(&self, target: U256) {
		*self.gas_floor_target.write().unwrap() = target;
	}

	fn set_minimal_gas_price(&self, min_gas_price: U256) {
		*self.min_gas_price.write().unwrap() = min_gas_price;
	}

	fn set_transactions_limit(&self, limit: usize) {
		*self.limit.write().unwrap() = limit;
	}

	fn transactions_limit(&self) -> usize {
		*self.limit.read().unwrap()
	}

	fn author(&self) -> Address {
		*self.author.read().unwrap()
	}

	fn minimal_gas_price(&self) -> U256 {
		*self.min_gas_price.read().unwrap()
	}

	fn extra_data(&self) -> Bytes {
		self.extra_data.read().unwrap().clone()
	}

	fn gas_floor_target(&self) -> U256 {
		*self.gas_floor_target.read().unwrap()
	}

	/// Imports transactions to transaction queue.
	fn import_transactions<T>(&self, transactions: Vec<SignedTransaction>, _fetch_account: T) ->
		Vec<Result<TransactionImportResult, Error>>
		where T: Fn(&Address) -> AccountDetails {
		// lets assume that all txs are valid
		self.imported_transactions.lock().unwrap().extend_from_slice(&transactions);

		transactions
			.iter()
			.map(|_| Ok(TransactionImportResult::Current))
			.collect()
	}

	/// Imports transactions to transaction queue.
	fn import_own_transaction<T>(&self, transaction: SignedTransaction, _fetch_account: T) ->
		Result<TransactionImportResult, Error>
		where T: Fn(&Address) -> AccountDetails {
		// lets assume that all txs are valid
		self.imported_transactions.lock().unwrap().push(transaction);

		Ok(TransactionImportResult::Current)
	}

	/// Returns hashes of transactions currently in pending
	fn pending_transactions_hashes(&self) -> Vec<H256> {
		vec![]
	}

	/// Removes all transactions from the queue and restart mining operation.
	fn clear_and_reset(&self, _chain: &BlockChainClient) {
		unimplemented!();
	}

	/// Called when blocks are imported to chain, updates transactions queue.
	fn chain_new_blocks(&self, _chain: &BlockChainClient, _imported: &[H256], _invalid: &[H256], _enacted: &[H256], _retracted: &[H256]) {
		unimplemented!();
	}

	/// New chain head event. Restart mining operation.
	fn update_sealing(&self, _chain: &BlockChainClient) {
		unimplemented!();
	}

	fn map_sealing_work<F, T>(&self, _chain: &BlockChainClient, _f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		unimplemented!();
	}

	fn transaction(&self, hash: &H256) -> Option<SignedTransaction> {
		self.pending_transactions.lock().unwrap().get(hash).cloned()
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		self.pending_transactions.lock().unwrap().values().cloned().collect()
	}

	fn last_nonce(&self, address: &Address) -> Option<U256> {
		self.last_nonces.read().unwrap().get(address).cloned()
	}

	/// Submit `seal` as a valid solution for the header of `pow_hash`.
	/// Will check the seal, but not actually insert the block into the chain.
	fn submit_seal(&self, _chain: &BlockChainClient, _pow_hash: H256, _seal: Vec<Bytes>) -> Result<(), Error> {
		unimplemented!();
	}
}
