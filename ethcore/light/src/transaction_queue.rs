// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Light Transaction Queue.
//!
//! Manages local transactions,
//! but stores all local transactions, removing only on invalidated nonce.
//!
//! Under the assumption that light nodes will have a relatively limited set of
//! accounts for which they create transactions, this queue is structured in an
//! address-wise manner.

use std::fmt;
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::Entry;

use common_types::transaction::{self, Condition, PendingTransaction, SignedTransaction};
use ethereum_types::{H256, U256, Address};
use fastmap::H256FastMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransactionInfo {
	hash: H256,
	nonce: U256,
	condition: Option<Condition>,
}

impl<'a> From<&'a PendingTransaction> for TransactionInfo {
	fn from(tx: &'a PendingTransaction) -> Self {
		TransactionInfo {
			hash: tx.hash(),
			nonce: tx.nonce,
			condition: tx.condition.clone(),
		}
	}
}

// transactions associated with a specific account.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AccountTransactions {
	// believed current nonce (gotten from initial given TX or `cull` calls).
	cur_nonce: CurrentNonce,
	current: Vec<TransactionInfo>, // ordered "current" transactions (cur_nonce onwards)
	future: BTreeMap<U256, TransactionInfo>, // "future" transactions.
}

impl AccountTransactions {
	fn is_empty(&self) -> bool {
		self.current.is_empty() && self.future.is_empty()
	}

	fn next_nonce(&self) -> U256 {
		self.current.last().map(|last| last.nonce + 1)
			.unwrap_or_else(|| *self.cur_nonce.value())
	}

	// attempt to move transactions from the future queue into the current queue.
	fn adjust_future(&mut self) -> Vec<H256> {
		let mut promoted = Vec::new();
		let mut next_nonce = self.next_nonce();

		while let Some(tx) = self.future.remove(&next_nonce) {
			promoted.push(tx.hash);
			self.current.push(tx);
			next_nonce = next_nonce + 1;
		}

		promoted
	}
}

/// Transaction import result.
pub enum ImportDestination {
	/// Transaction has been imported to the current queue.
	///
	/// It's going to be propagated to peers.
	Current,
	/// Transaction has been imported to future queue.
	///
	/// It means it won't be propagated until the gap is filled.
	Future,
}

type Listener = Box<Fn(&[H256]) + Send + Sync>;

/// Light transaction queue. See module docs for more details.
#[derive(Default)]
pub struct TransactionQueue {
	by_account: HashMap<Address, AccountTransactions>,
	by_hash: H256FastMap<PendingTransaction>,
	listeners: Vec<Listener>,
}

impl fmt::Debug for TransactionQueue {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("TransactionQueue")
			.field("by_account", &self.by_account)
			.field("by_hash", &self.by_hash)
			.field("listeners", &self.listeners.len())
			.finish()
	}
}

impl TransactionQueue {
	/// Import a pending transaction to be queued.
	pub fn import(&mut self, tx: PendingTransaction) -> Result<ImportDestination, transaction::Error> {
		let sender = tx.sender();
		let hash = tx.hash();
		let nonce = tx.nonce;
		let tx_info = TransactionInfo::from(&tx);

		if self.by_hash.contains_key(&hash) { return Err(transaction::Error::AlreadyImported) }

		let (res, promoted) = match self.by_account.entry(sender) {
			Entry::Vacant(entry) => {
				entry.insert(AccountTransactions {
					cur_nonce: CurrentNonce::Assumed(nonce),
					current: vec![tx_info],
					future: BTreeMap::new(),
				});

				(ImportDestination::Current, vec![hash])
			}
			Entry::Occupied(mut entry) => {
				let acct_txs = entry.get_mut();
				if nonce < *acct_txs.cur_nonce.value() {
					// don't accept txs from before known current nonce.
					if acct_txs.cur_nonce.is_known() {
						return Err(transaction::Error::Old)
					}

					// lower our assumption until corrected later.
					acct_txs.cur_nonce = CurrentNonce::Assumed(nonce);
				}

				match acct_txs.current.binary_search_by(|x| x.nonce.cmp(&nonce)) {
					Ok(idx) => {
						trace!(target: "txqueue", "Replacing existing transaction from {} with nonce {}",
							sender, nonce);

						let old = ::std::mem::replace(&mut acct_txs.current[idx], tx_info);
						self.by_hash.remove(&old.hash);

						(ImportDestination::Current, vec![hash])
					}
					Err(idx) => {
						let cur_len = acct_txs.current.len();
						let incr_nonce = nonce + 1;

						// current is sorted with one tx per nonce,
						// so if a tx with given nonce wasn't found that means it is either
						// earlier in nonce than all other "current" transactions or later.
						assert!(idx == 0 || idx == cur_len);

						if idx == 0 && acct_txs.current.first().map_or(false, |f| f.nonce != incr_nonce) {
							let old_cur = ::std::mem::replace(&mut acct_txs.current, vec![tx_info]);

							trace!(target: "txqueue", "Moving {} transactions with nonce > {} to future",
								old_cur.len(), incr_nonce);

							for future in old_cur {
								let future_nonce = future.nonce;
								acct_txs.future.insert(future_nonce, future);
							}

							(ImportDestination::Current, vec![hash])
						} else if idx == cur_len && acct_txs.current.last().map_or(false, |f| f.nonce + 1 != nonce) {
							trace!(target: "txqueue", "Queued future transaction for {}, nonce={}", sender, nonce);
							let future_nonce = nonce;
							acct_txs.future.insert(future_nonce, tx_info);

							(ImportDestination::Future, vec![])
						} else {
							trace!(target: "txqueue", "Queued current transaction for {}, nonce={}", sender, nonce);

							// insert, then check if we've filled any gaps.
							acct_txs.current.insert(idx, tx_info);
							let mut promoted = acct_txs.adjust_future();
							promoted.insert(0, hash);

							(ImportDestination::Current, promoted)
						}
					}
				}
			}
		};

		self.by_hash.insert(hash, tx);
		self.notify(&promoted);
		Ok(res)
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
	///
	/// Returned transactions are batched by sender, in order of ascending nonce.
	pub fn ready_transactions(&self, best_block_number: u64, best_block_timestamp: u64) -> Vec<PendingTransaction> {
		self.by_account.values()
			.flat_map(|acct_txs| {
				acct_txs.current.iter().take_while(|tx| match tx.condition {
					None => true,
					Some(Condition::Number(blk_num)) => blk_num <= best_block_number,
					Some(Condition::Timestamp(time)) => time <= best_block_timestamp,
				}).map(|info| info.hash)
			})
			.filter_map(|hash| match self.by_hash.get(&hash) {
				Some(tx) => Some(tx.clone()),
				None => {
					warn!(target: "txqueue", "Inconsistency detected between `by_hash` and `by_account`: {} not stored.",
						hash);
					None
				}
			})
			.collect()
	}

	/// Get all transactions not ready to be propagated.
	/// `best_block_number` and `best_block_timestamp` are used to filter out conditionally
	/// propagated transactions.
	///
	/// Returned transactions are batched by sender, in order of ascending nonce.
	pub fn future_transactions(&self, best_block_number: u64, best_block_timestamp: u64) -> Vec<PendingTransaction> {
		self.by_account.values()
			.flat_map(|acct_txs| {
				acct_txs.current.iter().skip_while(|tx| match tx.condition {
					None => true,
					Some(Condition::Number(blk_num)) => blk_num <= best_block_number,
					Some(Condition::Timestamp(time)) => time <= best_block_timestamp,
				}).chain(acct_txs.future.values()).map(|info| info.hash)
			})
			.filter_map(|hash| match self.by_hash.get(&hash) {
				Some(tx) => Some(tx.clone()),
				None => {
					warn!(target: "txqueue", "Inconsistency detected between `by_hash` and `by_account`: {} not stored.",
						hash);
					None
				}
			})
			.collect()
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
						.hash;
					removed_hashes.push(hash);
				}

				// then cull from "current".
				let valid_pos = acct_txs.current.iter().position(|tx| tx.nonce >= cur_nonce);
				match valid_pos {
					None =>
						removed_hashes.extend(acct_txs.current.drain(..).map(|tx| tx.hash)),
					Some(valid) =>
						removed_hashes.extend(acct_txs.current.drain(..valid).map(|tx| tx.hash)),
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

	/// Get a transaction by hash.
	pub fn get(&self, hash: &H256) -> Option<&PendingTransaction> {
		self.by_hash.get(&hash)
	}

	/// Add a transaction queue listener.
	pub fn add_listener(&mut self, f: Listener) {
		self.listeners.push(f);
	}

	/// Notifies all listeners about new pending transaction.
	fn notify(&self, hashes: &[H256]) {
		for listener in &self.listeners {
			listener(hashes)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::TransactionQueue;
	use ethereum_types::Address;
	use common_types::transaction::{Transaction, PendingTransaction, Condition};

	#[test]
	fn queued_senders() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();
		let tx = Transaction::default().fake_sign(sender);

		txq.import(tx.into()).unwrap();

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

			txq.import(tx.into()).unwrap();
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

			txq.import(tx.into()).unwrap();
		}

		assert_eq!(txq.ready_transactions(0, 0).len(), 5);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 10.into());

		for i in 0..3 {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.import(tx.into()).unwrap();
		}

		assert_eq!(txq.ready_transactions(0, 0).len(), 3);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 3.into());

		for i in 3..5 {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.import(tx.into()).unwrap();
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

			txq.import(match i {
				3 => PendingTransaction::new(tx, Some(Condition::Number(100))),
				4 => PendingTransaction::new(tx, Some(Condition::Timestamp(1234))),
				_ => tx.into(),
			}).unwrap();
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

			txq.import(tx.into()).unwrap();
		}

		txq.cull(sender, 6.into());

		assert_eq!(txq.ready_transactions(0, 0).len(), 4);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 10.into());
	}

	#[test]
	fn import_old() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();

		let mut tx_a = Transaction::default();
		tx_a.nonce = 3.into();

		let mut tx_b = Transaction::default();
		tx_b.nonce = 2.into();

		txq.import(tx_a.fake_sign(sender).into()).unwrap();
		txq.cull(sender, 3.into());

		assert!(txq.import(tx_b.fake_sign(sender).into()).is_err())
	}

	#[test]
	fn replace_is_removed() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();

		let tx_b: PendingTransaction = Transaction::default().fake_sign(sender).into();
		let tx_a: PendingTransaction = {
			let mut tx_a = Transaction::default();
			tx_a.gas_price = tx_b.gas_price + 1;
			tx_a.fake_sign(sender).into()
		};

		let hash = tx_a.hash();

		txq.import(tx_a).unwrap();
		txq.import(tx_b).unwrap();

		assert!(txq.transaction(&hash).is_none());
	}

	#[test]
	fn future_transactions() {
		let sender = Address::default();
		let mut txq = TransactionQueue::default();

		for i in (0..1).chain(3..10) {
			let mut tx = Transaction::default();
			tx.nonce = i.into();

			let tx = tx.fake_sign(sender);

			txq.import(tx.into()).unwrap();
		}

		assert_eq!(txq.future_transactions(0, 0).len(), 7);
		assert_eq!(txq.next_nonce(&sender).unwrap(), 1.into());
	}
}
