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

use std::cmp::{Ordering};
use std::collections::{HashMap, BTreeSet};
use util::numbers::{Uint, U256};
use util::hash::{Address, H256};
use util::table::*;
use ethcore::transaction::*;


#[derive(Clone, Debug)]
struct TransactionOrder {
	nonce_height: U256,
	gas_price: U256,
	hash: H256,
}

impl TransactionOrder {
	pub fn for_transaction(tx: &VerifiedTransaction, base_nonce: U256) -> Self {
		TransactionOrder {
			nonce_height: tx.nonce() - base_nonce,
			gas_price: tx.transaction.gas_price,
			hash: tx.hash(),
		}
	}
}

impl Eq for TransactionOrder {}
impl PartialEq for TransactionOrder {
	fn eq(&self, other: &TransactionOrder) -> bool {
		self.cmp(other) == Ordering::Equal
	}
}
impl PartialOrd for TransactionOrder {
	fn partial_cmp(&self, other: &TransactionOrder) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for TransactionOrder {
	fn cmp(&self, b: &TransactionOrder) -> Ordering {
		// First check nonce_height
		if self.nonce_height != b.nonce_height {
			return self.nonce_height.cmp(&b.nonce_height);
		}

		// Then compare gas_prices
		let a_gas = self.gas_price;
		let b_gas = b.gas_price;
		if a_gas != b_gas {
			return a_gas.cmp(&b_gas);
		}

		// Compare hashes
		self.hash.cmp(&b.hash)
	}
}

struct VerifiedTransaction {
	transaction: SignedTransaction
}
impl VerifiedTransaction {
	fn new(transaction: SignedTransaction) -> Self {
		VerifiedTransaction {
			transaction: transaction
		}
	}

	fn hash(&self) -> H256 {
		self.transaction.hash()
	}

	fn nonce(&self) -> U256 {
		self.transaction.nonce
	}

	fn sender(&self) -> Address {
		self.transaction.sender().unwrap()
	}
}

struct TransactionSet {
	by_priority: BTreeSet<TransactionOrder>,
	by_address: Table<Address, U256, TransactionOrder>,
	limit: usize,
}

impl TransactionSet {
	fn insert(&mut self, sender: Address, nonce: U256, order: TransactionOrder) {
		self.by_priority.insert(order.clone());
		self.by_address.insert(sender, nonce, order);
	}

	fn enforce_limit(&mut self, by_hash: &HashMap<H256, VerifiedTransaction>) {
		let len = self.by_priority.len();
		if len <= self.limit {
			return;
		}

		let to_drop : Vec<&VerifiedTransaction> = {
			self.by_priority
				.iter()
				.skip(self.limit)
				.map(|order| by_hash.get(&order.hash).expect("Inconsistency in queue detected."))
				.collect()
		};

		for tx in to_drop {
			self.drop(&tx.sender(), &tx.nonce());
		}
	}

	fn drop(&mut self, sender: &Address, nonce: &U256) -> Option<TransactionOrder> {
		if let Some(tx_order) = self.by_address.remove(sender, nonce) {
			self.by_priority.remove(&tx_order);
			return Some(tx_order);
		}
		None
	}

	fn clear(&mut self) {
		self.by_priority.clear();
		self.by_address.clear();
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
	current: TransactionSet,
	/// Priority queue for transactions that has been received but are not yet valid to go to block
	future: TransactionSet,
	/// All transactions managed by queue indexed by hash
	by_hash: HashMap<H256, VerifiedTransaction>,
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
		let current = TransactionSet {
			by_priority: BTreeSet::new(),
			by_address: Table::new(),
			limit: current_limit,
		};
		let future = TransactionSet {
			by_priority: BTreeSet::new(),
			by_address: Table::new(),
			limit: future_limit,
		};

		TransactionQueue {
			current: current,
			future: future,
			by_hash: HashMap::new(),
			last_nonces: HashMap::new(),
			first_nonces: HashMap::new(),
		}
	}

	/// Returns current status for this queue
	pub fn status(&self) -> TransactionQueueStatus {
		TransactionQueueStatus {
			pending: self.current.by_priority.len(),
			future: self.future.by_priority.len(),
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
		self.import_tx(VerifiedTransaction::new(tx), fetch_nonce);
	}

	/// Removes all transactions identified by hashes given in slice
	///
	/// If gap is introduced marks subsequent transactions as future
	pub fn remove_all(&mut self, txs: &[H256]) {
		for tx in txs {
			self.remove(&tx);
		}
	}

	/// Removes transaction identified by hashes from queue.
	///
	/// If gap is introduced marks subsequent transactions as future
	pub fn remove(&mut self, hash: &H256) {
		let transaction = self.by_hash.remove(hash);
		if transaction.is_none() {
			// We don't know this transaction
			return;
		}
		let transaction = transaction.unwrap();
		let sender = transaction.sender();
		let nonce = transaction.nonce();

		// Remove from future
		self.future.drop(&sender, &nonce);

		// Remove from current
		let order = self.current.drop(&sender, &nonce);
		if order.is_none() {
			return;
		}

		// Are there any other transactions from this sender?
		if !self.current.by_address.has_row(&sender) {
			// Clear last & first nonces
			self.last_nonces.remove(&sender);
			self.first_nonces.remove(&sender);
			return;
		}

		// Let's find those with higher nonce (TODO [todr] optimize?)
		let to_move_to_future = {
			let row_map = self.current.by_address.row(&sender).unwrap();
			let mut to_future = Vec::new();
			let mut highest = U256::zero();
			let mut lowest = nonce.clone();

			// Search nonces to remove and track lowest and highest
			for (current_nonce, _) in row_map.iter() {
				if current_nonce > &nonce {
					to_future.push(current_nonce.clone());
				} else if current_nonce > &highest {
					highest = current_nonce.clone();
				} else if current_nonce < &lowest {
					lowest = current_nonce.clone();
				}
			}

			// Update first_nonces and last_nonces
			if highest == U256::zero() {
				self.last_nonces.remove(&sender);
			} else {
				self.last_nonces.insert(sender.clone(), highest);
			}

			if lowest == nonce {
				self.first_nonces.remove(&sender);
			} else {
				self.first_nonces.insert(sender.clone(), lowest);
			}

			// return to future
			to_future
		};

		for k in to_move_to_future {
			if let Some(v) = self.current.drop(&sender, &k) {
				// TODO [todr] Recalculate height?
				self.future.insert(sender.clone(), k, v);
			}
		}
		self.future.enforce_limit(&self.by_hash);
	}

	/// Returns top transactions from the queue
	pub fn top_transactions(&self, size: usize) -> Vec<SignedTransaction> {
		self.current.by_priority
			.iter()
			.take(size)
			.map(|t| self.by_hash.get(&t.hash).expect("Transaction Queue Inconsistency"))
			.map(|t| t.transaction.clone())
			.collect()
	}

	/// Removes all elements (in any state) from the queue
	pub fn clear(&mut self) {
		self.current.clear();
		self.future.clear();
		self.by_hash.clear();
		self.last_nonces.clear();
		self.first_nonces.clear();
	}

	fn move_future_txs(&mut self, address: Address, current_nonce: U256, first_nonce: U256) -> Option<U256> {
		let mut current_nonce = current_nonce + U256::one();
		{
			let by_nonce = self.future.by_address.row_mut(&address);
			if let None = by_nonce {
				return None;
			}
			let mut by_nonce = by_nonce.unwrap();
			while let Some(order) = by_nonce.remove(&current_nonce) {
				// remove also from priority and hash
				self.future.by_priority.remove(&order);
				// Put to current
				let transaction = self.by_hash.get(&order.hash).expect("TransactionQueue Inconsistency");
				let order = TransactionOrder::for_transaction(transaction, first_nonce);
				self.current.insert(address.clone(), transaction.nonce(), order);
				current_nonce = current_nonce + U256::one();
			}
		}
		self.future.by_address.clear_if_empty(&address);
		// Returns last inserted nonce
		Some(current_nonce - U256::one())
	}

	fn import_tx<T>(&mut self, tx: VerifiedTransaction, fetch_nonce: &T)
		where T: Fn(&Address) -> U256 {
		let nonce = tx.nonce();
		let address = tx.sender();

		let next_nonce = U256::one() + self.last_nonces
			.get(&address)
			.cloned()
			.unwrap_or_else(|| fetch_nonce(&address));

		// Check height
		if nonce > next_nonce {
			let order = TransactionOrder::for_transaction(&tx, next_nonce);
			// Insert to by_hash
			self.by_hash.insert(tx.hash(), tx);
			// We have a gap - put to future
			self.future.insert(address, nonce, order);
			self.future.enforce_limit(&self.by_hash);
			return;
		} else if next_nonce > nonce {
			// Droping transaction
			return;
		}

		let first_nonce = self.first_nonces
			.get(&address)
			.cloned()
			.unwrap_or_else(|| nonce.clone());

		let order = TransactionOrder::for_transaction(&tx, first_nonce);
		// Insert to by_hash
		self.by_hash.insert(tx.hash(), tx);

		// Insert to current
		self.current.insert(address.clone(), nonce, order);
		// But maybe there are some more items waiting in future?
		let new_last_nonce = self.move_future_txs(address.clone(), nonce, first_nonce);
		self.first_nonces.insert(address.clone(), first_nonce);
		self.last_nonces.insert(address.clone(), new_last_nonce.unwrap_or(nonce));
		// Enforce limit
		self.current.enforce_limit(&self.by_hash);
	}
}


#[cfg(test)]
mod test {
	extern crate rustc_serialize;
	use self::rustc_serialize::hex::FromHex;
	use std::collections::{HashMap, BTreeSet};
	use util::crypto::KeyPair;
	use util::numbers::{U256, Uint};
	use util::hash::{Address};
	use util::table::*;
	use ethcore::transaction::*;
	use super::*;
	use super::{TransactionSet, TransactionOrder, VerifiedTransaction};

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
	fn should_create_transaction_set() {
		// given
		let mut set = TransactionSet {
			by_priority: BTreeSet::new(),
			by_address: Table::new(),
			limit: 1
		};
		let (tx1, tx2) = new_txs(U256::from(1));
		let tx1 = VerifiedTransaction::new(tx1);
		let tx2 = VerifiedTransaction::new(tx2);
		let by_hash = {
			let mut x = HashMap::new();
			let tx1 = VerifiedTransaction::new(tx1.transaction.clone());
			let tx2 = VerifiedTransaction::new(tx2.transaction.clone());
			x.insert(tx1.hash(), tx1);
			x.insert(tx2.hash(), tx2);
			x
		};
		// Insert both transactions
		let order1 = TransactionOrder::for_transaction(&tx1, U256::zero());
		set.insert(tx1.sender(), tx1.nonce(), order1.clone());
		let order2 = TransactionOrder::for_transaction(&tx2, U256::zero());
		set.insert(tx2.sender(), tx2.nonce(), order2.clone());
		assert_eq!(set.by_priority.len(), 2);
		assert_eq!(set.by_address.len(), 2);

		// when
		set.enforce_limit(&by_hash);

		// then
		assert_eq!(set.by_priority.len(), 1);
		assert_eq!(set.by_address.len(), 1);
		assert_eq!(set.by_priority.iter().next().unwrap().clone(), order1);
		set.clear();
		assert_eq!(set.by_priority.len(), 0);
		assert_eq!(set.by_address.len(), 0);
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
		txq2.remove(&tx.hash());
		txq2.remove(&tx2.hash());


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
		txq.remove(&tx.hash());

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
		txq.remove(&tx1.hash());
		assert_eq!(txq.status().pending, 0);
		assert_eq!(txq.status().future, 1);
		txq.add(tx1.clone(), &default_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.future, 0);
		assert_eq!(stats.pending, 2);

	}

}
