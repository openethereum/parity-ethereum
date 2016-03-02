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

//! Transaction Queue

use std::vec::Vec;
use std::cmp::{Ordering};
use std::collections::{HashMap, BTreeSet};
use util::numbers::{Uint, U256};
use util::hash::{Address};
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

impl Eq for VerifiedTransaction {}
impl PartialEq for VerifiedTransaction {
	fn eq(&self, other: &VerifiedTransaction) -> bool {
		self.cmp(other) == Ordering::Equal
	}
}
impl PartialOrd for VerifiedTransaction {
	fn partial_cmp(&self, other: &VerifiedTransaction) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for VerifiedTransaction {
	fn cmp(&self, b: &VerifiedTransaction) -> Ordering {
		// First check nonce_height
		if self.nonce_height != b.nonce_height {
			return self.nonce_height.cmp(&b.nonce_height);
		}

		// Then compare gas_prices
		let a_gas = self.tx.gas_price;
		let b_gas = b.tx.gas_price;
		if a_gas != b_gas {
			return a_gas.cmp(&b_gas);
		}

		// Compare nonce
		let a_nonce = self.tx.nonce;
		let b_nonce = b.tx.nonce;
		if a_nonce != b_nonce {
			return a_nonce.cmp(&b_nonce);
		}

		// and senders
		let a_sender = self.sender();
		let b_sender = b.sender();
		a_sender.cmp(&b_sender)
	}
}

struct TransactionsByPriorityAndAddress {
	priority: BTreeSet<VerifiedTransaction>,
	address: Table<Address, U256, VerifiedTransaction>,
	limit: usize,
}

impl TransactionsByPriorityAndAddress {
	fn insert(&mut self, address: Address, nonce: U256, verified_tx: VerifiedTransaction) {
		self.priority.insert(verified_tx.clone());
		self.address.insert(address, nonce, verified_tx);
	}

	fn enforce_limit(&mut self) {
		let len = self.priority.len();
		if len <= self.limit {
			return;
		}

		let to_remove : Vec<SignedTransaction> = {
			self.priority
				.iter()
				.skip(self.limit)
				.map(|v_tx| v_tx.tx.clone())
				.collect()
		};

		for tx in to_remove {
			self.remove(&tx);
		}
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
		self.remove_by_address(&address, &tx.nonce)
	}

	fn clear(&mut self) {
		self.priority.clear();
		self.address.clear();
	}
}

#[derive(Debug)]
/// Current status of the queue
pub struct TransactionQueueStatus {
	/// Number of pending transactions (ready to go to block)
	pub pending: usize,
	/// Number of future transactions (waiting for transactions with lower nonces first)
	pub future: usize,
}

/// TransactionQueue implementation
pub struct TransactionQueue {
	/// Priority queue for transactions that can go to block
	current: TransactionsByPriorityAndAddress,
	/// Priority queue for transactions that has been received but are not yet valid to go to block
	future: TransactionsByPriorityAndAddress,
	/// Last nonce of transaction in current
	last_nonces: HashMap<Address, U256>,
	/// First nonce of transaction in current (used to determine priority)
	first_nonces: HashMap<Address, U256>,
}

impl TransactionQueue {
	/// Creates new instance of this Queue
	pub fn new() -> Self {
		Self::with_limits(1024, 1024)
	}

	/// Create new instance of this Queue with specified limits
	pub fn with_limits(current_limit: usize, future_limit: usize) -> Self {
		let current = TransactionsByPriorityAndAddress {
			address: Table::new(),
			priority: BTreeSet::new(),
			limit: current_limit,
		};
		let future = TransactionsByPriorityAndAddress {
			address: Table::new(),
			priority: BTreeSet::new(),
			limit: future_limit,
		};

		TransactionQueue {
			current: current,
			future: future,
			last_nonces: HashMap::new(),
			first_nonces: HashMap::new(),
		}
	}

	/// Returns current status for this queue
	pub fn status(&self) -> TransactionQueueStatus {
		TransactionQueueStatus {
			pending: self.current.priority.len(),
			future: self.future.priority.len(),
		}
	}

	/// Adds all signed transactions to queue to be verified and imported
	pub fn add_all<T>(&mut self, txs: Vec<SignedTransaction>, fetch_nonce: T)
		where T: Fn(&Address) -> U256 {
		for tx in txs.into_iter() {
			self.add(tx, &fetch_nonce);
		}
	}

	/// Add signed transaction to queue to be verified and imported
	pub fn add<T>(&mut self, tx: SignedTransaction, fetch_nonce: &T)
		where T: Fn(&Address) -> U256 {
		self.import_tx(tx, fetch_nonce);
	}

	/// Removes all transactions in given slice
	///
	/// If gap is introduced marks subsequent transactions as future
	pub fn remove_all(&mut self, txs: &[SignedTransaction]) {
		for tx in txs {
			self.remove(&tx);
		}
	}

	/// Removes transaction from queue.
	///
	/// If gap is introduced marks subsequent transactions as future
	pub fn remove(&mut self, tx: &SignedTransaction) {
		// Remove from current
		let removed = self.current.remove(tx);
		if let Some(verified_tx) = removed {
			let sender = verified_tx.sender();

			// Are there any other transactions from this sender?
			if !self.current.address.has_row(&sender) {
				// Clear last & first nonces
				self.last_nonces.remove(&sender);
				self.first_nonces.remove(&sender);
				return;
			}

			// Let's find those with higher nonce (TODO [todr] optimize?)
			let to_move_to_future = {
				let row_map = self.current.address.row(&sender).unwrap();
				let tx_nonce = verified_tx.tx.nonce;
				let mut to_future = Vec::new();
				let mut highest = U256::zero();
				let mut lowest = tx_nonce.clone();

				// Search nonces to remove and track lowest and highest
				for (nonce, _) in row_map.iter() {
					if nonce > &tx_nonce {
						to_future.push(nonce.clone());
					} else if nonce > &highest {
						highest = nonce.clone();
					} else if nonce < &lowest {
						lowest = nonce.clone();
					}
				}

				// Update first_nonces and last_nonces
				if highest == U256::zero() {
					self.last_nonces.remove(&sender);
				} else {
					self.last_nonces.insert(sender.clone(), highest);
				}

				if lowest == tx_nonce {
					self.first_nonces.remove(&sender);
				} else {
					self.first_nonces.insert(sender.clone(), lowest);
				}

				// return to future
				to_future
			};

			for k in to_move_to_future {
				if let Some(v) = self.current.remove_by_address(&sender, &k) {
					self.future.insert(sender.clone(), v.tx.nonce, v);
				}
			}
			self.future.enforce_limit();
			return;
		}

		// Remove from future
		{
			let sender = tx.sender().unwrap();
			if let Some(_) = self.future.remove_by_address(&sender, &tx.nonce) {
				return;
			}
		}
	}

	/// Returns top transactions from the queue
	pub fn top_transactions(&self, size: usize) -> Vec<SignedTransaction> {
		self.current.priority
			.iter()
			.take(size)
			.map(|t| t.tx.clone()).collect()
	}

	/// Removes all elements (in any state) from the queue
	pub fn clear(&mut self) {
		self.current.clear();
		self.future.clear();
		self.last_nonces.clear();
		self.first_nonces.clear();
	}

	fn move_future_txs(&mut self, address: Address, current_nonce: U256, first_nonce: U256) -> Option<U256> {
		let mut current_nonce = current_nonce + U256::one();
		{
			let txs_by_nonce = self.future.address.row_mut(&address);
			if let None = txs_by_nonce {
				return None;
			}
			let mut txs_by_nonce = txs_by_nonce.unwrap();

			while let Some(tx) = txs_by_nonce.remove(&current_nonce) {
				// remove also from priority
				self.future.priority.remove(&tx);
				// Put to current
				let height = current_nonce - first_nonce;
				let verified_tx = VerifiedTransaction::new(tx.tx, U256::from(height));
				self.current.insert(address.clone(), verified_tx.tx.nonce, verified_tx);
				current_nonce = current_nonce + U256::one();
			}
		}
		self.future.address.clear_if_empty(&address);
		// Returns last inserted nonce
		Some(current_nonce - U256::one())
	}

	fn import_tx<T>(&mut self, tx: SignedTransaction, fetch_nonce: &T)
		where T: Fn(&Address) -> U256 {
		let nonce = tx.nonce;
		let address = tx.sender().unwrap();

		let next_nonce = U256::one() + self.last_nonces
			.get(&address)
			.cloned()
			.unwrap_or_else(|| fetch_nonce(&address));

		// Check height
		if nonce > next_nonce {
			let height = nonce - next_nonce;
			let verified_tx = VerifiedTransaction::new(tx, height);
			// We have a gap - put to future
			self.future.insert(address, nonce, verified_tx);
			self.future.enforce_limit();
			return;
		} else if next_nonce > nonce {
			// Droping transaction
			return;
		}

		let first_nonce = self.first_nonces
			.get(&address)
			.cloned()
			.unwrap_or_else(|| nonce.clone());

		let height = nonce - first_nonce;
		let verified_tx = VerifiedTransaction::new(tx, height);
		// Insert to current
		self.current.insert(address.clone(), nonce, verified_tx);
		// But maybe there are some more items waiting in future?
		let new_last_nonce = self.move_future_txs(address.clone(), nonce, first_nonce);
		self.first_nonces.insert(address.clone(), first_nonce);
		self.last_nonces.insert(address.clone(), new_last_nonce.unwrap_or(nonce));
		// Enforce limit
		self.current.enforce_limit();
	}
}

#[cfg(test)]
mod test {
	extern crate rustc_serialize;
	use self::rustc_serialize::hex::FromHex;

	use util::crypto::KeyPair;
	use util::numbers::{U256, Uint};
	use util::hash::{Address};
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
		new_unsigned_tx(U256::from(123)).sign(&keypair.secret())
	}

	fn default_nonce(_address: &Address) -> U256 {
		U256::from(122)
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
	fn should_import_tx() {
		// given
		let mut txq = TransactionQueue::new();
		let tx = new_tx();

		// when
		txq.add(tx, &default_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 1);
	}

	#[test]
	fn should_import_txs_from_same_sender() {
		// given
		let mut txq = TransactionQueue::new();

		let (tx, tx2) = new_txs(U256::from(1));

		// when
		txq.add(tx.clone(), &default_nonce);
		txq.add(tx2.clone(), &default_nonce);

		// then
		let top = txq.top_transactions(5);
		assert_eq!(top[0], tx);
		assert_eq!(top[1], tx2);
		assert_eq!(top.len(), 2);
	}

	#[test]
	fn should_put_transaction_to_futures_if_gap_detected() {
		// given
		let mut txq = TransactionQueue::new();

		let (tx, tx2) = new_txs(U256::from(2));

		// when
		txq.add(tx.clone(), &default_nonce);
		txq.add(tx2.clone(), &default_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.future, 1);
		let top = txq.top_transactions(5);
		assert_eq!(top.len(), 1);
		assert_eq!(top[0], tx);
	}

	#[test]
	fn should_move_transactions_if_gap_filled() {
		// given
		let mut txq = TransactionQueue::new();
		let kp = KeyPair::create().unwrap();
		let secret = kp.secret();
		let tx = new_unsigned_tx(U256::from(123)).sign(&secret);
		let tx1 = new_unsigned_tx(U256::from(124)).sign(&secret);
		let tx2 = new_unsigned_tx(U256::from(125)).sign(&secret);

		txq.add(tx, &default_nonce);
		assert_eq!(txq.status().pending, 1);
		txq.add(tx2, &default_nonce);
		assert_eq!(txq.status().future, 1);

		// when
		txq.add(tx1, &default_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 3);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_remove_transaction() {
		// given
		let mut txq2 = TransactionQueue::new();
		let (tx, tx2) = new_txs(U256::from(3));
		txq2.add(tx.clone(), &default_nonce);
		txq2.add(tx2.clone(), &default_nonce);
		assert_eq!(txq2.status().pending, 1);
		assert_eq!(txq2.status().future, 1);

		// when
		txq2.remove(&tx);
		txq2.remove(&tx2);


		// then
		let stats = txq2.status();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_move_transactions_to_future_if_gap_introduced() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx, tx2) = new_txs(U256::from(1));
		let tx3 = new_tx();
		txq.add(tx2.clone(), &default_nonce);
		assert_eq!(txq.status().future, 1);
		txq.add(tx3.clone(), &default_nonce);
		txq.add(tx.clone(), &default_nonce);
		assert_eq!(txq.status().pending, 3);

		// when
		txq.remove(&tx);

		// then
		let stats = txq.status();
		assert_eq!(stats.future, 1);
		assert_eq!(stats.pending, 1);
	}

	#[test]
	fn should_clear_queue() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx, tx2) = new_txs(U256::one());

		// add
		txq.add(tx2.clone(), &default_nonce);
		txq.add(tx.clone(), &default_nonce);
		let stats = txq.status();
		assert_eq!(stats.pending, 2);

		// when
		txq.clear();

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 0);
	}

	#[test]
	fn should_drop_old_transactions_when_hitting_the_limit() {
		// given
		let mut txq = TransactionQueue::with_limits(1, 1);
		let (tx, tx2) = new_txs(U256::one());
		txq.add(tx.clone(), &default_nonce);
		assert_eq!(txq.status().pending, 1);

		// when
		txq.add(tx2.clone(), &default_nonce);

		// then
		let t = txq.top_transactions(2);
		assert_eq!(txq.status().pending, 1);
		assert_eq!(t.len(), 1);
		assert_eq!(t[0], tx);
	}

	#[test]
	fn should_limit_future_transactions() {
		let mut txq = TransactionQueue::with_limits(10, 1);
		let (tx1, tx2) = new_txs(U256::from(4));
		let (tx3, tx4) = new_txs(U256::from(4));
		txq.add(tx1.clone(), &default_nonce);
		txq.add(tx3.clone(), &default_nonce);
		assert_eq!(txq.status().pending, 2);

		// when
		txq.add(tx2.clone(), &default_nonce);
		assert_eq!(txq.status().future, 1);
		txq.add(tx4.clone(), &default_nonce);

		// then
		assert_eq!(txq.status().future, 1);
	}

	#[test]
	fn should_drop_transactions_with_old_nonces() {
		let mut txq = TransactionQueue::new();
		let tx = new_tx();
		let last_nonce = tx.nonce.clone();
		let fetch_last_nonce = |_a: &Address| last_nonce;

		// when
		txq.add(tx, &fetch_last_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_accept_same_transaction_twice() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx1, tx2) = new_txs(U256::from(1));
		txq.add(tx1.clone(), &default_nonce);
		txq.add(tx2.clone(), &default_nonce);
		assert_eq!(txq.status().pending, 2);

		// when
		txq.remove(&tx1);
		assert_eq!(txq.status().future, 1);
		txq.add(tx1.clone(), &default_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 2);
		assert_eq!(stats.future, 0);

	}

}
