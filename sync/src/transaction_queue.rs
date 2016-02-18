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
use util::hash::Address;
use ethcore::transaction::*;

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

#[derive(Debug)]
pub struct TxQueueStats {
	pub pending: usize,
	pub queued: usize,
	pub future: usize,
}

pub struct TxQueue<'a> {
	executor: &'a Executor<ImportTxTask>,
	limit: usize,
	current: Arc<RwLock<BTreeSet<VerifiedTransaction>>>,
	last_nonces: Arc<RwLock<HashMap<Address, U256>>>,
	future: Arc<RwLock<Table<Address, U256, SignedTransaction>>>,
}

impl<'a> TxQueue<'a> {
	/// Creates new instance of this Queue
	pub fn new(executor: &'a Executor<ImportTxTask>) -> Self {
		let limit = 1024;
		let current = BTreeSet::new();
		let future = Table::new();
		let nonces = HashMap::new();

		TxQueue {
			executor: executor,
			limit: limit,
			current: Arc::new(RwLock::new(current)),
			future: Arc::new(RwLock::new(future)),
			last_nonces: Arc::new(RwLock::new(nonces)),
		}
	}

	/// Returns current stats for this queue
	pub fn stats(&self) -> TxQueueStats {
		let current = self.current.read().unwrap();
		let future = self.future.read().unwrap();
		TxQueueStats {
			pending: current.len(),
			future: future.len(),
			queued: self.executor.queued()
		}
	}

	/// Add signed transaction to queue to be verified and imported
	pub fn add(&mut self, tx: SignedTransaction) {
		let current = self.current.clone();
		let future = self.future.clone();
		let last_nonces = self.last_nonces.clone();

		self.executor.execute(ImportTxTask {
			tx: tx,
			current: current,
			future: future,
			last_nonces: last_nonces,
		}).fire();
	}

	/// Returns top transactions from the queue
	pub fn top_transactions(&self, size: usize) -> Vec<SignedTransaction> {
		let current = self.current.read().unwrap();
		current.iter().take(size).map(|t| t.tx.clone()).collect()
	}

	/// Removes all elements (in any state) from the queue
	pub fn clear(&self) {
		let mut current = self.current.write().unwrap();
		let mut future = self.future.write().unwrap();
		let mut nonces = self.last_nonces.write().unwrap();

		current.clear();
		future.clear();
		nonces.clear();
		self.executor.clear();
	}
}
struct Table<Row, Col, Val>
	where Row: Eq + Hash + Clone,
		  Col: Eq + Hash {
	map: HashMap<Row, HashMap<Col, Val>>,
	len: usize,
}
impl<Row, Col, Val> Table<Row, Col, Val>
	where Row: Eq + Hash + Clone,
		  Col: Eq + Hash {
	fn new() -> Table<Row, Col, Val> {
		Table {
			map: HashMap::new(),
			len: 0,
		}
	}

	fn clear(&mut self) {
		self.map.clear();
	}

	fn len(&self) -> usize {
		self.map.iter().fold(0, |acc, (_k, v)| acc + v.len())
	}

	fn is_empty(&self) -> bool {
		self.len() == 0
	}

	fn get_row_mut(&mut self, row: &Row) -> Option<&mut HashMap<Col, Val>> {
		self.map.get_mut(row)
	}

	fn clear_if_empty(&mut self, row: &Row) {
		let is_empty = self.map.get(row).map_or(false, |m| m.is_empty());
		if is_empty {
			self.map.remove(row);
		}
	}

	fn insert(&mut self, row: Row, col: Col, val: Val) {
		if !self.map.contains_key(&row) {
			let m = HashMap::new();
			self.map.insert(row.clone(), m);
		}

		let mut columns = self.map.get_mut(&row).unwrap();
		let result = columns.insert(col, val);

		if let None = result {
			self.len += 1;
		}
	}
}

pub struct ImportTxTask {
	tx: SignedTransaction,
	last_nonces: Arc<RwLock<HashMap<Address, U256>>>,
	current: Arc<RwLock<BTreeSet<VerifiedTransaction>>>,
	future: Arc<RwLock<Table<Address, U256, SignedTransaction>>>,
}

impl ImportTxTask {
	fn move_future_txs(&self, address: Address, nonce: U256) {
		let mut future = self.future.write().unwrap();
		{
			let txs_by_nonce = future.get_row_mut(&address);
			if let None = txs_by_nonce {
				return;
			}
			let mut txs_by_nonce = txs_by_nonce.unwrap();

			let mut current_nonce = nonce + U256::one();
			let mut queue = self.current.write().unwrap();

			while let Some(tx) = txs_by_nonce.remove(&current_nonce) {
				let height = current_nonce - nonce;
				queue.insert(VerifiedTransaction::new(tx, U256::from(height)));
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
			// We have a gap safe to insert
			let mut queue = self.current.write().unwrap();
			queue.insert(VerifiedTransaction::new(tx, height));
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
		let tx = new_unsigned_tx(U256::zero());
		let tx2 = new_unsigned_tx(second_nonce);

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
		// let stats = txq.stats();
		// assert_eq!(stats.pending, 1);
		// assert_eq!(stats.queued, 0);
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
