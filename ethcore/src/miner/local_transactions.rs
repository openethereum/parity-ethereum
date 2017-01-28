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

//! Local Transactions List.

use linked_hash_map::LinkedHashMap;
use transaction::SignedTransaction;
use error::TransactionError;
use util::{U256, H256};

/// Status of local transaction.
/// Can indicate that the transaction is currently part of the queue (`Pending/Future`)
/// or gives a reason why the transaction was removed.
#[derive(Debug, PartialEq, Clone)]
pub enum Status {
	/// The transaction is currently in the transaction queue.
	Pending,
	/// The transaction is in future part of the queue.
	Future,
	/// Transaction is already mined.
	Mined(SignedTransaction),
	/// Transaction is dropped because of limit
	Dropped(SignedTransaction),
	/// Replaced because of higher gas price of another transaction.
	Replaced(SignedTransaction, U256, H256),
	/// Transaction was never accepted to the queue.
	Rejected(SignedTransaction, TransactionError),
	/// Transaction is invalid.
	Invalid(SignedTransaction),
}

impl Status {
	fn is_current(&self) -> bool {
		*self == Status::Pending || *self == Status::Future
	}
}

/// Keeps track of local transactions that are in the queue or were mined/dropped recently.
#[derive(Debug)]
pub struct LocalTransactionsList {
	max_old: usize,
	transactions: LinkedHashMap<H256, Status>,
}

impl Default for LocalTransactionsList {
	fn default() -> Self {
		Self::new(10)
	}
}

impl LocalTransactionsList {
	pub fn new(max_old: usize) -> Self {
		LocalTransactionsList {
			max_old: max_old,
			transactions: Default::default(),
		}
	}

	pub fn mark_pending(&mut self, hash: H256) {
		debug!(target: "own_tx", "Imported to Current (hash {:?})", hash);
		self.clear_old();
		self.transactions.insert(hash, Status::Pending);
	}

	pub fn mark_future(&mut self, hash: H256) {
		debug!(target: "own_tx", "Imported to Future (hash {:?})", hash);
		self.transactions.insert(hash, Status::Future);
		self.clear_old();
	}

	pub fn mark_rejected(&mut self, tx: SignedTransaction, err: TransactionError) {
		debug!(target: "own_tx", "Transaction rejected (hash {:?}): {:?}", tx.hash(), err);
		self.transactions.insert(tx.hash(), Status::Rejected(tx, err));
		self.clear_old();
	}

	pub fn mark_replaced(&mut self, tx: SignedTransaction, gas_price: U256, hash: H256) {
		debug!(target: "own_tx", "Transaction replaced (hash {:?}) by {:?} (new gas price: {:?})", tx.hash(), hash, gas_price);
		self.transactions.insert(tx.hash(), Status::Replaced(tx, gas_price, hash));
		self.clear_old();
	}

	pub fn mark_invalid(&mut self, tx: SignedTransaction) {
		warn!(target: "own_tx", "Transaction marked invalid (hash {:?})", tx.hash());
		self.transactions.insert(tx.hash(), Status::Invalid(tx));
		self.clear_old();
	}

	pub fn mark_dropped(&mut self, tx: SignedTransaction) {
		warn!(target: "own_tx", "Transaction dropped (hash {:?})", tx.hash());
		self.transactions.insert(tx.hash(), Status::Dropped(tx));
		self.clear_old();
	}

	pub fn mark_mined(&mut self, tx: SignedTransaction) {
		info!(target: "own_tx", "Transaction mined (hash {:?})", tx.hash());
		self.transactions.insert(tx.hash(), Status::Mined(tx));
		self.clear_old();
	}

	pub fn contains(&self, hash: &H256) -> bool {
		self.transactions.contains_key(hash)
	}

	pub fn all_transactions(&self) -> &LinkedHashMap<H256, Status> {
		&self.transactions
	}

	fn clear_old(&mut self) {
		let number_of_old = self.transactions
			.values()
			.filter(|status| !status.is_current())
			.count();

		if self.max_old >= number_of_old {
			return;
		}

		let to_remove = self.transactions
			.iter()
			.filter(|&(_, status)| !status.is_current())
			.map(|(hash, _)| *hash)
			.take(number_of_old - self.max_old)
			.collect::<Vec<_>>();

		for hash in to_remove {
			self.transactions.remove(&hash);
		}
	}
}

#[cfg(test)]
mod tests {
	use util::U256;
	use ethkey::{Random, Generator};
	use transaction::{Action, Transaction, SignedTransaction};
	use super::{LocalTransactionsList, Status};

	#[test]
	fn should_add_transaction_as_pending() {
		// given
		let mut list = LocalTransactionsList::default();

		// when
		list.mark_pending(10.into());
		list.mark_future(20.into());

		// then
		assert!(list.contains(&10.into()), "Should contain the transaction.");
		assert!(list.contains(&20.into()), "Should contain the transaction.");
		let statuses = list.all_transactions().values().cloned().collect::<Vec<Status>>();
		assert_eq!(statuses, vec![Status::Pending, Status::Future]);
	}

	#[test]
	fn should_clear_old_transactions() {
		// given
		let mut list = LocalTransactionsList::new(1);
		let tx1 = new_tx(10.into());
		let tx1_hash = tx1.hash();
		let tx2 = new_tx(50.into());
		let tx2_hash = tx2.hash();

		list.mark_pending(10.into());
		list.mark_invalid(tx1);
		list.mark_dropped(tx2);
		assert!(list.contains(&tx2_hash));
		assert!(!list.contains(&tx1_hash));
		assert!(list.contains(&10.into()));

		// when
		list.mark_future(15.into());

		// then
		assert!(list.contains(&10.into()));
		assert!(list.contains(&15.into()));
	}

	fn new_tx(nonce: U256) -> SignedTransaction {
		let keypair = Random.generate().unwrap();
		Transaction {
			action: Action::Create,
			value: U256::from(100),
			data: Default::default(),
			gas: U256::from(10),
			gas_price: U256::from(1245),
			nonce: nonce
		}.sign(keypair.secret(), None)
	}
}
