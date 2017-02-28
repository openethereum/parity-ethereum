// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use util::{Address, H256, Bytes, U256, FixedHash, Uint};
use util::standard::*;
use ethcore::error::{Error, CallError};
use ethcore::client::{MiningBlockChainClient, Executed, CallAnalytics};
use ethcore::block::{ClosedBlock, IsBlock};
use ethcore::header::BlockNumber;
use ethcore::transaction::{UnverifiedTransaction, SignedTransaction, PendingTransaction};
use ethcore::receipt::{Receipt, RichReceipt};
use ethcore::miner::{MinerService, MinerStatus, TransactionImportResult, LocalTransactionStatus};
use ethcore::account_provider::SignError as AccountError;

/// Test miner service.
pub struct TestMinerService {
	/// Imported transactions.
	pub imported_transactions: Mutex<Vec<SignedTransaction>>,
	/// Latest closed block.
	pub latest_closed_block: Mutex<Option<ClosedBlock>>,
	/// Pre-existed pending transactions
	pub pending_transactions: Mutex<HashMap<H256, SignedTransaction>>,
	/// Pre-existed local transactions
	pub local_transactions: Mutex<BTreeMap<H256, LocalTransactionStatus>>,
	/// Pre-existed pending receipts
	pub pending_receipts: Mutex<BTreeMap<H256, Receipt>>,
	/// Last nonces.
	pub last_nonces: RwLock<HashMap<Address, U256>>,
	/// Password held by Engine.
	pub password: RwLock<String>,

	min_gas_price: RwLock<U256>,
	gas_range_target: RwLock<(U256, U256)>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,
	limit: RwLock<usize>,
	tx_gas_limit: RwLock<U256>,
}

impl Default for TestMinerService {
	fn default() -> TestMinerService {
		TestMinerService {
			imported_transactions: Mutex::new(Vec::new()),
			latest_closed_block: Mutex::new(None),
			pending_transactions: Mutex::new(HashMap::new()),
			local_transactions: Mutex::new(BTreeMap::new()),
			pending_receipts: Mutex::new(BTreeMap::new()),
			last_nonces: RwLock::new(HashMap::new()),
			min_gas_price: RwLock::new(U256::from(20_000_000)),
			gas_range_target: RwLock::new((U256::from(12345), U256::from(54321))),
			author: RwLock::new(Address::zero()),
			password: RwLock::new(String::new()),
			extra_data: RwLock::new(vec![1, 2, 3, 4]),
			limit: RwLock::new(1024),
			tx_gas_limit: RwLock::new(!U256::zero()),
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
		*self.author.write() = author;
	}

	fn set_engine_signer(&self, address: Address, password: String) -> Result<(), AccountError> {
		*self.author.write() = address;
		*self.password.write() = password;
		Ok(())
	}

	fn set_extra_data(&self, extra_data: Bytes) {
		*self.extra_data.write() = extra_data;
	}

	/// Set the lower gas limit we wish to target when sealing a new block.
	fn set_gas_floor_target(&self, target: U256) {
		self.gas_range_target.write().0 = target;
	}

	/// Set the upper gas limit we wish to target when sealing a new block.
	fn set_gas_ceil_target(&self, target: U256) {
		self.gas_range_target.write().1 = target;
	}

	fn set_minimal_gas_price(&self, min_gas_price: U256) {
		*self.min_gas_price.write() = min_gas_price;
	}

	fn set_transactions_limit(&self, limit: usize) {
		*self.limit.write() = limit;
	}

	fn set_tx_gas_limit(&self, limit: U256) {
		*self.tx_gas_limit.write() = limit;
	}

	fn transactions_limit(&self) -> usize {
		*self.limit.read()
	}

	fn author(&self) -> Address {
		*self.author.read()
	}

	fn minimal_gas_price(&self) -> U256 {
		*self.min_gas_price.read()
	}

	fn extra_data(&self) -> Bytes {
		self.extra_data.read().clone()
	}

	fn gas_floor_target(&self) -> U256 {
		self.gas_range_target.read().0
	}

	fn gas_ceil_target(&self) -> U256 {
		self.gas_range_target.read().1
	}

	/// Imports transactions to transaction queue.
	fn import_external_transactions(&self, _chain: &MiningBlockChainClient, transactions: Vec<UnverifiedTransaction>) ->
		Vec<Result<TransactionImportResult, Error>> {
		// lets assume that all txs are valid
		let transactions: Vec<_> = transactions.into_iter().map(|tx| SignedTransaction::new(tx).unwrap()).collect();
		self.imported_transactions.lock().extend_from_slice(&transactions);

		for sender in transactions.iter().map(|tx| tx.sender()) {
			let nonce = self.last_nonce(&sender).expect("last_nonce must be populated in tests");
			self.last_nonces.write().insert(sender, nonce + U256::from(1));
		}
		transactions
			.iter()
			.map(|_| Ok(TransactionImportResult::Current))
			.collect()
	}

	/// Imports transactions to transaction queue.
	fn import_own_transaction(&self, chain: &MiningBlockChainClient, pending: PendingTransaction) ->
		Result<TransactionImportResult, Error> {

		// keep the pending nonces up to date
		let sender = pending.transaction.sender();
		let nonce = self.last_nonce(&sender).unwrap_or(chain.latest_nonce(&sender));
		self.last_nonces.write().insert(sender, nonce + U256::from(1));

		// lets assume that all txs are valid
		self.imported_transactions.lock().push(pending.transaction);

		Ok(TransactionImportResult::Current)
	}

	/// Returns hashes of transactions currently in pending
	fn pending_transactions_hashes(&self, _best_block: BlockNumber) -> Vec<H256> {
		vec![]
	}

	/// Removes all transactions from the queue and restart mining operation.
	fn clear_and_reset(&self, _chain: &MiningBlockChainClient) {
		unimplemented!();
	}

	/// Called when blocks are imported to chain, updates transactions queue.
	fn chain_new_blocks(&self, _chain: &MiningBlockChainClient, _imported: &[H256], _invalid: &[H256], _enacted: &[H256], _retracted: &[H256]) {
		unimplemented!();
	}

	/// New chain head event. Restart mining operation.
	fn update_sealing(&self, _chain: &MiningBlockChainClient) {
		unimplemented!();
	}

	fn map_sealing_work<F, T>(&self, chain: &MiningBlockChainClient, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		let open_block = chain.prepare_open_block(self.author(), *self.gas_range_target.write(), self.extra_data());
		Some(f(&open_block.close()))
	}

	fn transaction(&self, _best_block: BlockNumber, hash: &H256) -> Option<PendingTransaction> {
		self.pending_transactions.lock().get(hash).cloned().map(Into::into)
	}

	fn pending_transactions(&self) -> Vec<PendingTransaction> {
		self.pending_transactions.lock().values().cloned().map(Into::into).collect()
	}

	fn local_transactions(&self) -> BTreeMap<H256, LocalTransactionStatus> {
		self.local_transactions.lock().iter().map(|(hash, stats)| (*hash, stats.clone())).collect()
	}

	fn ready_transactions(&self, _best_block: BlockNumber, _best_timestamp: u64) -> Vec<PendingTransaction> {
		self.pending_transactions.lock().values().cloned().map(Into::into).collect()
	}

	fn future_transactions(&self) -> Vec<PendingTransaction> {
		vec![]
	}

	fn pending_receipt(&self, _best_block: BlockNumber, hash: &H256) -> Option<RichReceipt> {
		// Not much point implementing this since the logic is complex and the only thing it relies on is pending_receipts, which is already tested.
		self.pending_receipts(0).get(hash).map(|r|
			RichReceipt {
				transaction_hash: Default::default(),
				transaction_index: Default::default(),
				cumulative_gas_used: r.gas_used.clone(),
				gas_used: r.gas_used.clone(),
				contract_address: None,
				logs: r.logs.clone(),
				log_bloom: r.log_bloom,
				state_root: r.state_root,
			}
		)
	}

	fn pending_receipts(&self, _best_block: BlockNumber) -> BTreeMap<H256, Receipt> {
		self.pending_receipts.lock().clone()
	}

	fn last_nonce(&self, address: &Address) -> Option<U256> {
		self.last_nonces.read().get(address).cloned()
	}

	fn is_sealing(&self) -> bool {
		false
	}

	/// Submit `seal` as a valid solution for the header of `pow_hash`.
	/// Will check the seal, but not actually insert the block into the chain.
	fn submit_seal(&self, _chain: &MiningBlockChainClient, _pow_hash: H256, _seal: Vec<Bytes>) -> Result<(), Error> {
		unimplemented!();
	}

	fn balance(&self, _chain: &MiningBlockChainClient, address: &Address) -> Option<U256> {
		self.latest_closed_block.lock()
			.as_ref()
			.map(|b| b.block().fields().state.balance(address))
			.map(|b| b.ok())
			.unwrap_or(Some(U256::default()))
	}

	fn call(&self, _chain: &MiningBlockChainClient, _t: &SignedTransaction, _analytics: CallAnalytics) -> Result<Executed, CallError> {
		unimplemented!();
	}

	fn storage_at(&self, _chain: &MiningBlockChainClient, address: &Address, position: &H256) -> Option<H256> {
		self.latest_closed_block.lock()
			.as_ref()
			.map(|b| b.block().fields().state.storage_at(address, position))
			.map(|s| s.ok())
			.unwrap_or(Some(H256::default()))
	}

	fn nonce(&self, _chain: &MiningBlockChainClient, address: &Address) -> Option<U256> {
		// we assume all transactions are in a pending block, ignoring the
		// reality of gas limits.
		Some(self.last_nonce(address).unwrap_or(U256::zero()))
	}

	fn code(&self, _chain: &MiningBlockChainClient, address: &Address) -> Option<Option<Bytes>> {
		self.latest_closed_block.lock()
			.as_ref()
			.map(|b| b.block().fields().state.code(address))
			.map(|c| c.ok())
			.unwrap_or(None)
			.map(|c| c.map(|c| (&*c).clone()))
	}

	fn sensible_gas_price(&self) -> U256 {
		20000000000u64.into()
	}
}
