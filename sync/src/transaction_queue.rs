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

// TODO [todr] - own transactions should have higher priority

use std::vec::Vec;
use std::cmp::{Ordering};
use std::collections::{HashMap, BTreeSet};
use std::sync::{RwLock, Arc};
use std::hash::Hash;
use util::executor::*;
use util::uint::{Uint, U256};
use util::hash::{Address, H256};
use util::table::*;
use ethcore::transaction::*;

#[derive(Clone, Debug)]
struct VerifiedTransaction {
	tx: SignedTransaction,
	nonce_height: U256
}

impl VerifiedTransaction {
	pub fn new(tx: SignedTransaction, nonce_height: U256) -> VerifiedTransaction {
		VerifiedTransaction {
			tx: tx,
			nonce_height: nonce_height
		}
	}

	pub fn sender(&self) -> Address {
		self.tx.sender().unwrap()
	}
}
trivial_ordering!(VerifiedTransaction by |a: &VerifiedTransaction, b: &VerifiedTransaction| {
	// First check nonce_height
	if a.nonce_height != b.nonce_height {
		return a.nonce_height.cmp(&b.nonce_height);
	}

	// Then compare gas_prices
	let a_gas = a.tx.gas_price;
	let b_gas = b.tx.gas_price;
	if a_gas != b_gas {
		return a_gas.cmp(&b_gas);
	}

	// Compare nonce
	let a_nonce = a.tx.nonce;
	let b_nonce = b.tx.nonce;
	if a_nonce != b_nonce {
		return a_nonce.cmp(&b_nonce);
	}

	// and senders
	let a_sender = a.sender();
	let b_sender = b.sender();
	a_sender.cmp(&b_sender)
});

struct CurrentByPriorityAndAddress {
	priority: BTreeSet<VerifiedTransaction>,
	address: Table<Address, U256, VerifiedTransaction>
}

impl CurrentByPriorityAndAddress {
	fn insert(&mut self, address: Address, nonce: U256, verified_tx: VerifiedTransaction) {
		self.priority.insert(verified_tx.clone());
		self.address.insert(address, nonce, verified_tx);
	}

	fn remove_by_address(&mut self, sender: &Address, nonce: &U256) -> Option<VerifiedTransaction> {
		if let Some(verified_tx) = self.address.remove(sender, nonce) {
			self.priority.remove(&verified_tx);
			return Some(verified_tx);
		}
		None
	}

	fn remove(&mut self, tx: &SignedTransaction) -> Option<VerifiedTransaction> {
		// First find the transaction by address
		let address = tx.sender().unwrap();
		let verified_tx = self.address.remove(&address, &tx.nonce);
		if let Some(verified_tx) = verified_tx {
			self.priority.remove(&verified_tx);
			return Some(verified_tx)
		}
		None
	}
}

#[derive(Debug)]
pub struct TxQueueStats {
	pub pending: usize,
	pub queued: usize,
	pub future: usize,
}

pub struct TxQueue<'a> {
	executor: &'a Executor<ImportTxTask>,
	limit: usize,
	current: Arc<RwLock<CurrentByPriorityAndAddress>>,
	future: Arc<RwLock<Table<Address, U256, SignedTransaction>>>,
	avoid: Arc<RwLock<HashMap<H256, SignedTransaction>>>,
	last_nonces: Arc<RwLock<HashMap<Address, U256>>>,
}

impl<'a> TxQueue<'a> {
	/// Creates new instance of this Queue
	pub fn new(executor: &'a Executor<ImportTxTask>) -> Self {
		let limit = 1024;
		let current = CurrentByPriorityAndAddress {
			address: Table::new(),
			priority: BTreeSet::new()
		};
		let future = Table::new();
		let avoid = HashMap::new();
		let nonces = HashMap::new();

		TxQueue {
			executor: executor,
			limit: limit,
			current: Arc::new(RwLock::new(current)),
			future: Arc::new(RwLock::new(future)),
			avoid: Arc::new(RwLock::new(avoid)),
			last_nonces: Arc::new(RwLock::new(nonces)),
		}
	}

	/// Returns current stats for this queue
	pub fn stats(&self) -> TxQueueStats {
		let current = self.current.read().unwrap();
		let future = self.future.read().unwrap();
		TxQueueStats {
			pending: current.priority.len(),
			future: future.len(),
			queued: self.executor.queued()
		}
	}

	/// Add signed transaction to queue to be verified and imported
	pub fn add(&mut self, tx: SignedTransaction) {
		let current = self.current.clone();
		let future = self.future.clone();
		let avoid = self.avoid.clone();
		let last_nonces = self.last_nonces.clone();

		self.executor.execute(ImportTxTask {
			tx: tx,
			current: current,
			future: future,
			avoid: avoid,
			last_nonces: last_nonces,
		}).fire();
	}

	/// Removes transaction from queue.
	///
	/// If gap is introduced marks subsequent transactions as future
	pub fn remove(&mut self, tx: &SignedTransaction) {
		// Remove from current
		{
			let mut current = self.current.write().unwrap();
			let removed = current.remove(tx);
			if let Some(verified_tx) = removed {
				let sender = verified_tx.sender();
				// Are there any other transactions from this sender?
				if !current.address.has_row(&sender) {
					return;
				}

				// Let's find those with higher nonce
				let to_move_to_future : Vec<U256> = {
					let row_map = current.address.get_row(&sender).unwrap();
					let tx_nonce = verified_tx.tx.nonce;
					row_map
						.iter()
						.filter_map(|(nonce, _)| {
							if nonce > &tx_nonce {
								Some(nonce.clone())
							} else {
								None
							}
						})
						.collect()
				};
				let mut future = self.future.write().unwrap();
				for  k in to_move_to_future {
					if let Some(v) = current.remove_by_address(&sender, &k) {
						future.insert(sender.clone(), v.tx.nonce, v.tx.clone());
					}
				}
				return;
			}
		}

		// Remove from future
		{
			let mut future = self.future.write().unwrap();
			let sender = tx.sender().unwrap();
			if let Some(_) = future.remove(&sender, &tx.nonce) {
				return;
			}
		}

		// Avoid transaction - do not verify (happens only if it's in queue)
		{
			let mut avoid = self.avoid.write().unwrap();
			avoid.insert(tx.hash(), tx.clone());
		}
	}

	/// Returns top transactions from the queue
	pub fn top_transactions(&self, size: usize) -> Vec<SignedTransaction> {
		let current = self.current.read().unwrap();
		current.priority.iter().take(size).map(|t| t.tx.clone()).collect()
	}

	/// Removes all elements (in any state) from the queue
	pub fn clear(&self) {
		let mut current = self.current.write().unwrap();
		let mut future = self.future.write().unwrap();
		let mut nonces = self.last_nonces.write().unwrap();

		current.priority.clear();
		current.address.clear();
		future.clear();
		nonces.clear();
		self.executor.clear();
	}
}


pub struct ImportTxTask {
	tx: SignedTransaction,
	last_nonces: Arc<RwLock<HashMap<Address, U256>>>,
	current: Arc<RwLock<CurrentByPriorityAndAddress>>,
	future: Arc<RwLock<Table<Address, U256, SignedTransaction>>>,
	avoid: Arc<RwLock<HashMap<H256, SignedTransaction>>>,
}

impl ImportTxTask {
	fn move_future_txs(&self, address: Address, nonce: U256) {
		let mut queue = self.current.write().unwrap();
		let mut future = self.future.write().unwrap();
		{
			let txs_by_nonce = future.get_row_mut(&address);
			if let None = txs_by_nonce {
				return;
			}
			let mut txs_by_nonce = txs_by_nonce.unwrap();

			let mut current_nonce = nonce + U256::one();

			while let Some(tx) = txs_by_nonce.remove(&current_nonce) {
				let height = current_nonce - nonce;
				let verified_tx = VerifiedTransaction::new(tx, U256::from(height));
				queue.insert(address.clone(), nonce, verified_tx);
				current_nonce = current_nonce + U256::one();
			}
		}
		future.clear_if_empty(&address)
	}
}

impl Task for ImportTxTask {
	type Result = ();
	type Error = ();

	fn call(self) -> Result<Self::Result, Self::Error> {
		let tx = self.tx.clone();

		let nonce = tx.nonce;
		let address = tx.sender().unwrap();

		let (height, last_nonce, is_new) = {
			let nonces = self.last_nonces.read().unwrap();
			let (is_new, last_nonce) = nonces
				.get(&address)
				.map_or_else(|| (true, nonce), |last_nonce| (false, last_nonce.clone()));

			let height = if nonce > last_nonce { nonce - last_nonce } else { U256::zero() };
			(height, last_nonce, is_new)
		};

		// Check height
		if height > U256::from(1) {
			// We have a gap - we put to future
			let mut future = self.future.write().unwrap();
			future.insert(address, nonce, tx);
			return Ok(())
		}

		// Insert to queue
		{
			let mut queue = self.current.write().unwrap();
			let mut avoid = self.avoid.write().unwrap();
			// This transaction should not be inserted
			// because it has been removed before that task run
			if let Some(_tx) = avoid.remove(&tx.hash()) {
				return Ok(())
			}

			// We can insert the transaction
			let verified_tx = VerifiedTransaction::new(tx, height);
			queue.insert(address.clone(), nonce, verified_tx);

			// Update last_nonce
			if nonce > last_nonce || is_new {
				let mut nonces = self.last_nonces.write().unwrap();
				nonces.insert(address.clone(), nonce);
			}
		}

		// But maybe there are some more items waiting in future?
		self.move_future_txs(address, nonce);
		Ok(())
	}
}

#[cfg(test)]
mod test {
	extern crate rustc_serialize;
	use self::rustc_serialize::hex::FromHex;

	use util::crypto::KeyPair;
	use util::executor::Executors;
	use util::uint::{U256, Uint};
	use ethcore::transaction::*;
	use super::*;

	fn new_unsigned_tx(nonce: U256) -> Transaction {
		Transaction {
			action: Action::Create,
			value: U256::from(100),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::one(),
			nonce: nonce
		}
	}

	fn new_tx() -> SignedTransaction {
		let keypair = KeyPair::create().unwrap();
		new_unsigned_tx(U256::zero()).sign(&keypair.secret())
	}

	fn new_txs(second_nonce: U256) -> (SignedTransaction, SignedTransaction) {
		let keypair = KeyPair::create().unwrap();
		let secret = &keypair.secret();
		let nonce = U256::from(123);
		let tx = new_unsigned_tx(nonce);
		let tx2 = new_unsigned_tx(nonce + second_nonce);

		(tx.sign(secret), tx2.sign(secret))
	}

	#[test]
	fn should_enqueue_tx() {
		// given
		let exec = Executors::manual();
		let mut txq = TxQueue::new(&exec);
		let tx = new_tx();

		// when
		txq.add(tx);

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.queued, 1);
	}

	#[test]
	fn should_return_no_of_pending_txs() {
		// given
		let exec = Executors::manual();
		let mut txq = TxQueue::new(&exec);
		let tx = new_tx();
		txq.add(tx);
		assert_eq!(txq.stats().pending, 0);

		// when
		exec.consume(1);

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.queued, 0);
	}

	#[test]
	fn should_import_txs_from_same_sender() {
		// given
		let exec = Executors::same_thread();
		let mut txq = TxQueue::new(&exec);

		let (tx, tx2) = new_txs(U256::from(1));

		// when
		txq.add(tx.clone());
		txq.add(tx2.clone());

		// then
		let top = txq.top_transactions(5);
		assert_eq!(top[0], tx);
		assert_eq!(top[1], tx2);
		assert_eq!(top.len(), 2);
	}

	#[test]
	fn should_put_transaction_to_futures_if_gap_detected() {
		// given
		let exec = Executors::same_thread();
		let mut txq = TxQueue::new(&exec);

		let (tx, tx2) = new_txs(U256::from(2));

		// when
		txq.add(tx.clone());
		txq.add(tx2.clone());

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.queued, 0);
		assert_eq!(stats.future, 1);
		let top = txq.top_transactions(5);
		assert_eq!(top.len(), 1);
		assert_eq!(top[0], tx);
	}

	#[test]
	fn should_move_transactions_if_gap_filled() {
		// given
		let exec = Executors::same_thread();
		let mut txq = TxQueue::new(&exec);
		let kp = KeyPair::create().unwrap();
		let secret = kp.secret();
		let tx = new_unsigned_tx(U256::from(3)).sign(&secret);
		let tx1 = new_unsigned_tx(U256::from(4)).sign(&secret);
		let tx2 = new_unsigned_tx(U256::from(5)).sign(&secret);

		txq.add(tx);
		assert_eq!(txq.stats().pending, 1);
		txq.add(tx2);
		assert_eq!(txq.stats().future, 1);

		// when
		txq.add(tx1);

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 3);
		assert_eq!(stats.queued, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_remove_transaction() {
		// given
		let exec2 = Executors::same_thread();
		let mut txq2 = TxQueue::new(&exec2);
		let (tx, tx2) = new_txs(U256::from(3));
		txq2.add(tx.clone());
		txq2.add(tx2.clone());
		assert_eq!(txq2.stats().pending, 1);
		assert_eq!(txq2.stats().future, 1);

		// when
		txq2.remove(&tx);
		txq2.remove(&tx2);


		// then
		let stats = txq2.stats();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_not_import_transaction_if_removed() {
		// given
		let exec = Executors::manual();
		let mut txq = TxQueue::new(&exec);
		let tx = new_tx();
		txq.add(tx.clone());
		assert_eq!(txq.stats().queued, 1);

		// when
		txq.remove(&tx);
		exec.consume(1);

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
		assert_eq!(stats.queued, 0);
	}


	// TODO [todr] Not sure if this test is actually valid
	#[test]
	#[ignore]
	fn should_put_transaction_to_future_if_older_is_removed() {
		// given
		let exec = Executors::manual();
		let mut txq = TxQueue::new(&exec);
		let (tx, tx2) = new_txs(U256::from(1));
		txq.add(tx.clone());
		txq.add(tx2.clone());

		// when
		exec.consume(1);
		txq.remove(&tx);
		exec.consume(1);

		// then
		let stats = txq.stats();
		assert_eq!(stats.future, 1);
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.queued, 0);

	}

	#[test]
	fn should_move_transactions_to_future_if_gap_introduced() {
		// given
		let exec = Executors::same_thread();
		let mut txq = TxQueue::new(&exec);
		let (tx, tx2) = new_txs(U256::from(1));
		let tx3 = new_tx();
		txq.add(tx2.clone());
		txq.add(tx3.clone());
		txq.add(tx.clone());
		assert_eq!(txq.stats().pending, 3);

		// when
		txq.remove(&tx);

		// then
		let stats = txq.stats();
		assert_eq!(stats.future, 1);
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.queued, 0);
	}

	#[test]
	fn should_clear_queue() {
		// given
		let exec = Executors::manual();
		let mut txq = TxQueue::new(&exec);
		let (tx, tx2) = new_txs(U256::one());

		// add
		txq.add(tx2.clone());
		txq.add(tx.clone());
		exec.consume(1);
		let stats = txq.stats();
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.queued, 1);

		// when
		txq.clear();

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.queued, 0);
	}

}
