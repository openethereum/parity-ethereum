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
use util::uint::{Uint, U256};
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

struct TxsByPriorityAndAddress {
	priority: BTreeSet<VerifiedTransaction>,
	address: Table<Address, U256, VerifiedTransaction>,
	limit: usize,
}

impl TxsByPriorityAndAddress {
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
pub struct TxQueueStats {
	pub pending: usize,
	pub future: usize,
}

pub struct TxQueue {
	current: TxsByPriorityAndAddress,
	future: TxsByPriorityAndAddress,
	last_nonces: HashMap<Address, U256>,
}

impl TxQueue {
	/// Creates new instance of this Queue
	pub fn new() -> Self {
		Self::with_limits(1024, 1024)
	}

	/// Create new instance of this Queue with specified limits
	pub fn with_limits(current_limit: usize, future_limit: usize) -> Self {
		let current = TxsByPriorityAndAddress {
			address: Table::new(),
			priority: BTreeSet::new(),
			limit: current_limit,
		};
		let future = TxsByPriorityAndAddress {
			address: Table::new(),
			priority: BTreeSet::new(),
			limit: future_limit,
		};
		let nonces = HashMap::new();

		TxQueue {
			current: current,
			future: future,
			last_nonces: nonces,
		}
	}

	/// Returns current stats for this queue
	pub fn stats(&self) -> TxQueueStats {
		TxQueueStats {
			pending: self.current.priority.len(),
			future: self.future.priority.len(),
		}
	}

	/// Adds all signed transactions to queue to be verified and imported
	pub fn add_all(&mut self, txs: Vec<SignedTransaction>) {
		for tx in txs.into_iter() {
			self.add(tx);
		}
	}

	/// Add signed transaction to queue to be verified and imported
	pub fn add(&mut self, tx: SignedTransaction) {
		self.import_tx(tx);
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
				return;
			}

			// Let's find those with higher nonce
			let to_move_to_future : Vec<U256> = {
				let row_map = self.current.address.get_row(&sender).unwrap();
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
	}

	fn move_future_txs(&mut self, address: Address, nonce: U256) {
		{
			let txs_by_nonce = self.future.address.get_row_mut(&address);
			if let None = txs_by_nonce {
				return;
			}
			let mut txs_by_nonce = txs_by_nonce.unwrap();

			let mut current_nonce = nonce + U256::one();

			while let Some(tx) = txs_by_nonce.remove(&current_nonce) {
				// remove also from priority
				self.future.priority.remove(&tx);
				// Put to current
				let height = current_nonce - nonce;
				let verified_tx = VerifiedTransaction::new(tx.tx, U256::from(height));
				self.current.insert(address.clone(), nonce, verified_tx);
				current_nonce = current_nonce + U256::one();
			}
		}
		self.future.address.clear_if_empty(&address)
	}

	fn import_tx(&mut self, tx: SignedTransaction) {
		let nonce = tx.nonce;
		let address = tx.sender().unwrap();

		let (height, last_nonce, is_new) = {
			let (is_new, last_nonce) = self.last_nonces
				.get(&address)
				.map_or_else(|| (true, nonce), |last_nonce| (false, last_nonce.clone()));

			let height = if nonce > last_nonce { nonce - last_nonce } else { U256::zero() };
			(height, last_nonce, is_new)
		};

		// We can insert the transaction
		let verified_tx = VerifiedTransaction::new(tx, height);

		// Check height
		if height > U256::from(1) {
			// We have a gap - put to future
			self.future.insert(address, nonce, verified_tx);
			self.future.enforce_limit();
			return;
		}

		{
			// Insert to current
			self.current.insert(address.clone(), nonce, verified_tx);

			// Update last_nonce
			if nonce > last_nonce || is_new {
				self.last_nonces.insert(address.clone(), nonce);
			}
		}

		// But maybe there are some more items waiting in future?
		self.move_future_txs(address, nonce);
		// Enforce limit
		self.current.enforce_limit();
	}
}

#[cfg(test)]
mod test {
	extern crate rustc_serialize;
	use self::rustc_serialize::hex::FromHex;

	use util::crypto::KeyPair;
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
	fn should_import_tx() {
		// given
		let mut txq = TxQueue::new();
		let tx = new_tx();

		// when
		txq.add(tx);

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 1);
	}

	#[test]
	fn should_import_txs_from_same_sender() {
		// given
		let mut txq = TxQueue::new();

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
		let mut txq = TxQueue::new();

		let (tx, tx2) = new_txs(U256::from(2));

		// when
		txq.add(tx.clone());
		txq.add(tx2.clone());

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.future, 1);
		let top = txq.top_transactions(5);
		assert_eq!(top.len(), 1);
		assert_eq!(top[0], tx);
	}

	#[test]
	fn should_move_transactions_if_gap_filled() {
		// given
		let mut txq = TxQueue::new();
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
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_remove_transaction() {
		// given
		let mut txq2 = TxQueue::new();
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
	fn should_move_transactions_to_future_if_gap_introduced() {
		// given
		let mut txq = TxQueue::new();
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
	}

	#[test]
	fn should_clear_queue() {
		// given
		let mut txq = TxQueue::new();
		let (tx, tx2) = new_txs(U256::one());

		// add
		txq.add(tx2.clone());
		txq.add(tx.clone());
		let stats = txq.stats();
		assert_eq!(stats.pending, 2);

		// when
		txq.clear();

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 0);
	}

	#[test]
	fn should_drop_old_transactions_when_hitting_the_limit() {
		// given
		let mut txq = TxQueue::with_limits(1, 1);
		let (tx, tx2) = new_txs(U256::one());
		txq.add(tx.clone());
		assert_eq!(txq.stats().pending, 1);

		// when
		txq.add(tx2.clone());

		// then
		let t = txq.top_transactions(2);
		assert_eq!(txq.stats().pending, 1);
		assert_eq!(t.len(), 1);
		assert_eq!(t[0], tx);
	}

	#[test]
	fn should_limit_future_transactions() {
		let mut txq = TxQueue::with_limits(10, 1);
		let (tx1, tx2) = new_txs(U256::from(4));
		let (tx3, tx4) = new_txs(U256::from(4));
		txq.add(tx1.clone());
		txq.add(tx3.clone());
		assert_eq!(txq.stats().pending, 2);

		// when
		txq.add(tx2.clone());
		assert_eq!(txq.stats().future, 1);
		txq.add(tx4.clone());

		// then
		assert_eq!(txq.stats().future, 1);
	}

}
