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

//! Light Transaction Queue.
//!
//! Manages local transactions,
//! but stores all local transactions, removing only on invalidated nonce.
//!
//! Under the assumption that light nodes will have a relatively limited set of
//! accounts for which they create transactions, this queue is structured in an
//! address-wise manner.

use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::Entry;

use ethcore::transaction::{Condition, PendingTransaction, SignedTransaction};
use util::{Address, U256, H256, H256FastMap};

// Knowledge of an account's current nonce.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CurrentNonce {
	// Assumed current nonce.
	Assumed(U256),
	// Known current nonce.
	Known(U256),
}

impl CurrentNonce {
	// whether this nonce is assumed
	fn is_assumed(&self) -> bool {
		match *self {
			CurrentNonce::Assumed(_) => true,
			CurrentNonce::Known(_) => false,
		}
	}

	// whether this nonce is known for certain from an external source.
	fn is_known(&self) -> bool {
		!self.is_assumed()
	}

	// the current nonce's value.
	fn value(&self) -> &U256 {
		match *self {
			CurrentNonce::Assumed(ref val) => val,
			CurrentNonce::Known(ref val) => val,
		}
	}
}

// transactions associated with a specific account.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AccountTransactions {
	// believed current nonce (gotten from initial given TX or `cull` calls).
	cur_nonce: CurrentNonce,
	current: Vec<PendingTransaction>, // ordered "current" transactions (cur_nonce onwards)
	future: BTreeMap<U256, PendingTransaction>, // "future" transactions.
}

impl AccountTransactions {
	fn is_empty(&self) -> bool {
		self.current.is_empty() && self.future.is_empty()
	}

	fn next_nonce(&self) -> U256 {
		self.current.last().map(|last| last.nonce + 1.into())
			.unwrap_or_else(|| *self.cur_nonce.value())
	}

	// attempt to move transactions from the future queue into the current queue.
	fn adjust_future(&mut self) {
		let mut next_nonce = self.next_nonce();

		loop {
			match self.future.remove(&next_nonce) {
				Some(tx) => self.current.push(tx),
				None => break,
			}

			next_nonce = next_nonce + 1.into();
		}
	}
}

/// Light transaction queue. See module docs for more details.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TransactionQueue {
	by_account: HashMap<Address, AccountTransactions>,
	by_hash: H256FastMap<PendingTransaction>,
}

impl TransactionQueue {
	/// Insert a pending transaction to be queued.
	pub fn insert(&mut self, tx: PendingTransaction) {
		let sender = tx.sender();
		let hash = tx.hash();
		let nonce = tx.nonce;

	    match self.by_account.entry(sender) {
			Entry::Vacant(entry) => {
				entry.insert(AccountTransactions {
					cur_nonce: CurrentNonce::Assumed(nonce),
					current: vec![tx.clone()],
					future: BTreeMap::new(),
				});
			}
			Entry::Occupied(mut entry) => {
				let acct_txs = entry.get_mut();
				if &nonce < acct_txs.cur_nonce.value() {
					// don't accept txs from before known current nonce.
					if acct_txs.cur_nonce.is_known() { return }

					// lower our assumption until corrected later.
					acct_txs.cur_nonce = CurrentNonce::Assumed(nonce);
				}

				match acct_txs.current.binary_search_by(|x| x.nonce.cmp(&nonce)) {
					Ok(idx) => {
						trace!(target: "txqueue", "Replacing existing transaction from {} with nonce {}",
							sender, nonce);

						acct_txs.current[idx] = tx.clone();
					}
					Err(idx) => {
						let cur_len = acct_txs.current.len();
						let incr_nonce = nonce + 1.into();

						// current is sorted with one tx per nonce,
						// so if a tx with given nonce wasn't found that means it is either
						// earlier in nonce than all other "current" transactions or later.
						debug_assert!(idx == 0 || idx == cur_len);

						if idx == 0 && acct_txs.current.first().map_or(false, |f| f.nonce != incr_nonce) {
							let old_cur = ::std::mem::replace(&mut acct_txs.current, vec![tx.clone()]);

							trace!(target: "txqueue", "Moving {} transactions with nonce > {} to future",
								old_cur.len(), incr_nonce);

							for future in old_cur {
								let future_nonce = future.nonce;
								acct_txs.future.insert(future_nonce, future);
							}
						} else if idx == cur_len && acct_txs.current.last().map_or(false, |f| f.nonce + 1.into() != nonce) {
							trace!(target: "txqueue", "Queued future transaction for {}, nonce={}", sender, nonce);
							let future_nonce = nonce;
							acct_txs.future.insert(future_nonce, tx.clone());
						} else {
							trace!(target: "txqueue", "Queued current transaction for {}, nonce={}", sender, nonce);

							// insert, then check if we've filled any gaps.
							acct_txs.current.insert(idx, tx.clone());
							acct_txs.adjust_future();
						}
					}
				}
			}
		}

		self.by_hash.insert(hash, tx);
	}

	/// Get pending transaction by hash.
	pub fn transaction(&self, hash: &H256) -> Option<SignedTransaction> {
		self.by_hash.get(hash).map(|tx| (&**tx).clone())
	}

	/// Get the next nonce for a given address based on what's within the queue.
	/// If the address has no queued transactions, then `None` will be returned
	/// and the next nonce will have to be deduced via other means.
	pub fn next_nonce(&self, address: &Address) -> Option<U256> {
		self.by_account.get(address).map(AccountTransactions::next_nonce)
	}

	/// Get all transactions ready to be propagated.
	/// `best_block_number` and `best_block_timestamp` are used to filter out conditionally
	/// propagated transactions.
	pub fn ready_transactions(&self, best_block_number: u64, best_block_timestamp: u64) -> Vec<PendingTransaction> {
		self.by_account.values().flat_map(|acct_txs| {
			acct_txs.current.iter().take_while(|tx| match tx.condition {
				None => true,
				Some(Condition::Number(blk_num)) => blk_num <= best_block_number,
				Some(Condition::Timestamp(time)) => time <= best_block_timestamp,
			}).cloned()
		}).collect()
	}

	/// Addresses for which we store transactions.
	pub fn queued_senders(&self) -> Vec<Address> {
		self.by_account.keys().cloned().collect()
	}

	/// Cull out all transactions by the given address which are invalidated by the given nonce.
	pub fn cull(&mut self, address: Address, cur_nonce: U256) {
		let mut removed_hashes = vec![];
		if let Entry::Occupied(mut entry) = self.by_account.entry(address) {
			{
				let acct_txs = entry.get_mut();
				acct_txs.cur_nonce = CurrentNonce::Known(cur_nonce);

				// cull old "future" keys.
				let old_future: Vec<_> = acct_txs.future.keys().take_while(|&&k| k < cur_nonce).cloned().collect();

				for old in old_future {
					let hash = acct_txs.future.remove(&old)
						.expect("key extracted from keys iterator; known to exist; qed")
						.hash();
					removed_hashes.push(hash);
				}

				// then cull from "current".
				let valid_pos = acct_txs.current.iter().position(|tx| tx.nonce >= cur_nonce);
				match valid_pos {
					None =>
						removed_hashes.extend(acct_txs.current.drain(..).map(|tx| tx.hash())),
					Some(valid) =>
						removed_hashes.extend(acct_txs.current.drain(..valid).map(|tx| tx.hash())),
				}

				// now try and move stuff out of future into current.
				acct_txs.adjust_future();
			}

			if entry.get_mut().is_empty() {
				trace!(target: "txqueue", "No more queued transactions for {} after nonce {}",
					address, cur_nonce);
				entry.remove();
			}
		}

		trace!(target: "txqueue", "Culled {} old transactions from sender {} (nonce={})",
			removed_hashes.len(), address, cur_nonce);

		for hash in removed_hashes {
			self.by_hash.remove(&hash);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::TransactionQueue;
	use util::Address;
	use ethcore::transaction::{Transaction, PendingTransaction, Condition};

	#[test]
	fn queued_senders() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();
		let tx = Transaction::default().fake_sign(sender);

		txq.insert(tx.into());

		assert_eq!(txq.queued_senders(), vec![sender]);

		txq.cull(sender, 1.into());

		assert_eq!(txq.queued_senders(), vec![]);
		assert!(txq.by_hash.is_empty());
	}

	#[test]
	fn next_nonce() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();

		for i in (0..5).chain(10..15) {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.insert(tx.into());
		}

		// current: 0..5, future: 10..15
		assert_eq!(txq.ready_transactions(0, 0).len(), 5);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 5.into());

		txq.cull(sender, 8.into());

		// current: empty, future: 10..15
		assert_eq!(txq.ready_transactions(0, 0).len(), 0);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 8.into());

		txq.cull(sender, 10.into());

		// current: 10..15, future: empty
		assert_eq!(txq.ready_transactions(0, 0).len(), 5);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 15.into());
	}

	#[test]
	fn current_to_future() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();

		for i in 5..10 {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.insert(tx.into());
		}

		assert_eq!(txq.ready_transactions(0, 0).len(), 5);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 10.into());

		for i in 0..3 {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.insert(tx.into());
		}

		assert_eq!(txq.ready_transactions(0, 0).len(), 3);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 3.into());

		for i in 3..5 {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.insert(tx.into());
		}

		assert_eq!(txq.ready_transactions(0, 0).len(), 10);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 10.into());
	}

	#[test]
	fn conditional() {
		let mut txq = TransactionQueue::default();
		let sender = Address::default();

		for i in 0..5 {
			let mut tx = Transaction::default();
			tx.nonce = i.into();
			let tx = tx.fake_sign(sender);

			txq.insert(match i {
				3 => PendingTransaction::new(tx, Some(Condition::Number(100))),
				4 => PendingTransaction::new(tx, Some(Condition::Timestamp(1234))),
				_ => tx.into(),
			});
		}

		assert_eq!(txq.ready_transactions(0, 0).len(), 3);
		assert_eq!(txq.ready_transactions(0, 1234).len(), 3);
		assert_eq!(txq.ready_transactions(100, 0).len(), 4);
		assert_eq!(txq.ready_transactions(100, 1234).len(), 5);
	}

	#[test]
	fn cull_from_future() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();

		for i in (0..1).chain(3..10) {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.insert(tx.into());
		}

		txq.cull(sender, 6.into());

		assert_eq!(txq.ready_transactions(0, 0).len(), 4);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 10.into());
	}
}
