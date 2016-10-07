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

//! Blacklisting Queue
//! Transacton Queue wrapper maintaining additional black list of senders and contract hashes.

use std::ops::{Deref, DerefMut};
use std::cell::Cell;
use transaction::SignedTransaction;
use transient_hashmap::TransientHashMap;
use miner::{TransactionQueue, TransactionImportResult, TransactionOrigin, AccountDetails};
use error::{Error, TransactionError};
use util::{Uint, U256, H256, Address};

type Count = usize;
const ITEM_LIFETIME_SEC: u64 = 60;

/// Transaction queue with blacklist.
pub struct BlacklistingTransactionQueue {
	queue: TransactionQueue,
	blacklist_threshold: Count,
	senders_blacklist: TransientHashMap<Address, Cell<Count>>,
	_codes_blacklist: TransientHashMap<H256, Cell<Count>>,
}

impl BlacklistingTransactionQueue {
	/// Creates new blacklisting transaction queue
	pub fn new(queue: TransactionQueue, blacklist_threshold: Count) -> Self {
		BlacklistingTransactionQueue {
			queue: queue,
			blacklist_threshold: blacklist_threshold,
			senders_blacklist: TransientHashMap::new(ITEM_LIFETIME_SEC),
			_codes_blacklist: TransientHashMap::new(ITEM_LIFETIME_SEC),
		}
	}

	/// Borrows internal queue.
	/// NOTE: you can insert transactions to the queue even
	/// if they would be rejected because of blacklist otherwise.
	/// But probably you shouldn't.
	pub fn queue(&mut self) -> &mut TransactionQueue {
		&mut self.queue
	}

	/// Add to the queue taking blacklists into consideration.
	/// May reject transaction because of the blacklist.
	pub fn add_with_blacklist<F>(&mut self, transaction: SignedTransaction, account_details: &F) -> Result<TransactionImportResult, Error>
		where F: Fn(&Address) -> AccountDetails {
		// Check the senders blacklist first, but use direct query to avoid increasing the blacklist timeout.
		if let Ok(sender) = transaction.sender() {
			let count = self.senders_blacklist.direct().get(&sender).map(|v| v.get()).unwrap_or(0);
			if count > self.blacklist_threshold {
				return Err(Error::Transaction(TransactionError::SenderBlacklisted));
			}
		}
		self.queue.add(transaction, account_details, TransactionOrigin::External)
	}

	/// Blacklists given sender.
	/// If blacklist threshold is reached all subsequent transactions from this sender will be rejected.
	/// Reaching blacklist threshold also removes all existsing transaction from this sender that are already in the
	/// queue.
	pub fn blacklist_sender(&mut self, address: Address) -> bool {
		{
			let mut count = self.senders_blacklist.entry(address).or_insert_with(|| Cell::new(0));
			*count.get_mut() += 1;
			if count.get() <= self.blacklist_threshold {
				return false;
			}
		}
		// Blacklist the sender.
		// Remove all transactions from the queue.
		self.remove_all(address, !U256::zero());
		true
	}
}

impl Deref for BlacklistingTransactionQueue {
	type Target = TransactionQueue;

	fn deref(&self) -> &Self::Target {
		&self.queue
	}
}
impl DerefMut for BlacklistingTransactionQueue {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.queue()
	}
}

#[cfg(test)]
mod tests {
	use super::BlacklistingTransactionQueue;
	use ethkey::{Random, Generator};
	use transaction::{Transaction, SignedTransaction, Action};
	use error::{Error, TransactionError};
	use client::TransactionImportResult;
	use miner::{TransactionQueue, TransactionOrigin, AccountDetails};
	use util::{Uint, U256, Address, FromHex};

	fn queue() -> BlacklistingTransactionQueue {
		BlacklistingTransactionQueue::new(TransactionQueue::default(), 1)
	}

	fn default_account_details(_address: &Address) -> AccountDetails {
		AccountDetails {
			nonce: U256::zero(),
			balance: !U256::zero(),
		}
	}

	fn transaction() -> SignedTransaction {
		let keypair = Random.generate().unwrap();
		Transaction {
			action: Action::Create,
			value: U256::from(100),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::from(10),
			nonce: U256::from(0),
		}.sign(keypair.secret())
	}

	fn unwrap_err(res: Result<TransactionImportResult, Error>) -> TransactionError {
		match res {
			Err(Error::Transaction(e)) => e,
			Ok(x) => panic!("Expected error, got: Ok({:?})", x),
			Err(e) => panic!("Unexpected error type returned by queue: {:?}", e),
		}
	}

	#[test]
	fn should_allow_to_borrow_the_queue() {
		// given
		let tx = transaction();
		let mut txq = queue();

		// when
		txq.queue().add(tx, &default_account_details, TransactionOrigin::External).unwrap();

		// then
		// should also deref to queue
		assert_eq!(txq.status().pending, 1);
	}

	#[test]
	fn should_not_accept_transactions_from_blacklisted_sender() {
		// given
		let tx = transaction();
		let mut txq = queue();
		// Blacklist once (threshold not reached)
		let blacklist1 = txq.blacklist_sender(tx.sender().unwrap());
		assert!(!blacklist1, "Threshold not reached yet.");
		// Insert once
		let import1 = txq.add_with_blacklist(tx.clone(), &default_account_details).unwrap();
		assert_eq!(import1, TransactionImportResult::Current);

		// when
		let blacklist2 = txq.blacklist_sender(tx.sender().unwrap());
		let import2 = txq.add_with_blacklist(tx.clone(), &default_account_details);

		// then
		assert!(blacklist2, "Threshold should be reached - blacklist.");
		assert_eq!(unwrap_err(import2), TransactionError::SenderBlacklisted);
		// Should also remove transacion from the queue
		assert_eq!(txq.find(&tx.hash()), None);
	}
}
