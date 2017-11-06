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

use std::collections::{HashMap, BTreeSet};

use bigint::hash::{H160, H256};

use error;
use listener::{Listener, NoopListener};
use options::Options;
use ready::{Ready, Readiness};
use scoring::{Scoring, ScoreWithRef};
use status::{LightStatus, Status};
use transactions::{AddResult, Transactions};

use {VerifiedTransaction, SharedTransaction};

type Sender = H160;

/// A transaction pool.
#[derive(Debug)]
pub struct Pool<S: Scoring, L = NoopListener> {
	listener: L,
	scoring: S,
	options: Options,
	mem_usage: usize,

	transactions: HashMap<Sender, Transactions<S>>,
	by_hash: HashMap<H256, SharedTransaction>,

	best_transactions: BTreeSet<ScoreWithRef<S::Score>>,
	worst_transactions: BTreeSet<ScoreWithRef<S::Score>>,
}

impl<S: Scoring + Default> Default for Pool<S> {
	fn default() -> Self {
		Self::with_scoring(S::default(), Options::default())
	}
}

impl<S: Scoring + Default> Pool<S> {
	/// Creates a new `Pool` with given options
	/// and default `Scoring` and `Listener`.
	pub fn with_options(options: Options) -> Self {
		Self::with_scoring(S::default(), options)
	}
}

impl<S: Scoring> Pool<S> {
	/// Creates a new `Pool` with given `Scoring` and options.
	pub fn with_scoring(scoring: S, options: Options) -> Self {
		Self::new(NoopListener, scoring, options)
	}
}


const INITIAL_NUMBER_OF_SENDERS: usize = 16;

impl<S, L> Pool<S, L> where
	S: Scoring,
	L: Listener,
{
	/// Creates new `Pool` with given `Scoring`, `Listener` and options.
	pub fn new(listener: L, scoring: S, options: Options) -> Self {
		let transactions = HashMap::with_capacity(INITIAL_NUMBER_OF_SENDERS);
		let by_hash = HashMap::with_capacity(options.max_count / 16);

		Pool {
			listener,
			scoring,
			options,
			mem_usage: 0,
			transactions,
			by_hash,
			best_transactions: Default::default(),
			worst_transactions: Default::default(),
		}

	}

	/// Attempts to import new transaction to the pool, returns a `SharedTransaction` or an `Error`.
	///
	/// NOTE: The transaction may push out some other transactions from the pool
	/// either because of limits (see `Options`) or because `Scoring` decides that the transaction
	/// replaces an existing transaction from that sender.
	/// If any limit is reached the transaction with the lowest `Score` is evicted to make room.
	///
	/// The `Listener` will be informed on any drops or rejections.
	pub fn import(&mut self, mut transaction: VerifiedTransaction) -> error::Result<SharedTransaction> {
		let mem_usage = transaction.mem_usage();

		ensure!(!self.by_hash.contains_key(transaction.hash()), error::ErrorKind::AlreadyImported(*transaction.hash()));

		{
			let remove_worst = |s: &mut Self, transaction| {
				match s.remove_worst(&transaction) {
					Err(err) => {
						s.listener.rejected(transaction);
						Err(err)
					},
					Ok(removed) => {
						s.listener.dropped(&removed);
						s.removed(&removed);
						Ok(transaction)
					},
				}
			};

			while self.by_hash.len() + 1 > self.options.max_count {
				transaction = remove_worst(self, transaction)?;
			}

			while self.mem_usage + mem_usage > self.options.max_mem_usage {
				transaction = remove_worst(self, transaction)?;
			}
		}

		let result = {
			let transactions = self.transactions.entry(*transaction.sender()).or_insert_with(Transactions::default);
			// get worst and best transactions for comparison
			let prev = transactions.worst_and_best();
			let result = transactions.add(transaction, &self.scoring, self.options.max_per_sender);
			let current = transactions.worst_and_best();
			(result, prev, current)
		};

		// update best and worst transactions from this sender (if required)
		self.update_senders_worst_and_best(result.1, result.2);

		match result.0 {
			AddResult::Ok(tx) => {
				self.listener.added(&tx, None);
				self.added(&tx, None);
				Ok(tx)
			},
			AddResult::PushedOut { new, old } |
			AddResult::Replaced { new, old } => {
				self.listener.added(&new, Some(&old));
				self.added(&new, Some(&old));
				Ok(new)
			},
			AddResult::TooCheap { new, old } => {
				let hash = *new.hash();
				self.listener.rejected(new);
				bail!(error::ErrorKind::TooCheapToReplace(*old.hash(), hash))
			},
			AddResult::TooCheapToEnter(new) => {
				let hash = *new.hash();
				self.listener.rejected(new);
				bail!(error::ErrorKind::TooCheapToEnter(hash))
			}
		}
	}

	/// Updates state of the pool statistics if the transaction was added to a set.
	fn added(&mut self, new: &SharedTransaction, old: Option<&SharedTransaction>) {
		self.mem_usage += new.mem_usage();
		self.by_hash.insert(*new.hash(), new.clone());

		if let Some(old) = old {
			self.removed(old)
		}
	}

	/// Updates the pool statistics if transaction was removed.
	fn removed(&mut self, old: &SharedTransaction) {
		self.mem_usage -= old.mem_usage();
		self.by_hash.remove(old.hash());
	}

	/// Updates best and worst transactions from a sender.
	fn update_senders_worst_and_best(
		&mut self,
		previous: Option<((S::Score, SharedTransaction), (S::Score, SharedTransaction))>,
		current: Option<((S::Score, SharedTransaction), (S::Score, SharedTransaction))>,
	) {
		let worst = &mut self.worst_transactions;
		let best = &mut self.best_transactions;

		let mut update_worst = |(score, tx), remove| if remove {
			worst.remove(&ScoreWithRef::new(score, tx));
		} else {
			worst.insert(ScoreWithRef::new(score, tx));
		};

		let mut update_best = |(score, tx), remove| if remove {
			best.remove(&ScoreWithRef::new(score, tx));
		} else {
			best.insert(ScoreWithRef::new(score, tx));
		};

		match (previous, current) {
			(None, Some((worst, best))) => {
				update_worst(worst, false);
				update_best(best, false);
			},
			(Some((worst, best)), None) => {
				// all transactions from that sender has been removed.
				// We can clear a hashmap entry.
				self.transactions.remove(worst.1.sender());
				update_worst(worst, true);
				update_best(best, true);
			},
			(Some((w1, b1)), Some((w2, b2))) => {
				if w1 != w2 {
					update_worst(w1, true);
					update_worst(w2, false);
				}
				if b1 != b2 {
					update_best(b1, true);
					update_best(b2, false);
				}
			},
			(None, None) => {},
		}
	}

	/// Attempts to remove the worst transaction from the pool if it's worse than the given one.
	fn remove_worst(&mut self, transaction: &VerifiedTransaction) -> error::Result<SharedTransaction> {
		let to_remove = match self.worst_transactions.iter().next_back() {
			// No elements to remove? and the pool is still full?
			None => {
				warn!("The pool is full but there is no transaction to remove.");
				return Err(error::ErrorKind::TooCheapToEnter(*transaction.hash()).into());
			},
			Some(old) => if self.scoring.should_replace(&old.transaction, transaction) {
				// New transaction is better than the worst one so we can replace it.
				old.clone()
			} else {
				// otherwise fail
				return Err(error::ErrorKind::TooCheapToEnter(*transaction.hash()).into())
			},
		};

		// Remove from transaction set
		self.remove_transaction(&to_remove.transaction);
		Ok(to_remove.transaction)
	}

	/// Removes transaction from sender's transaction `HashMap`.
	fn remove_transaction(&mut self, transaction: &VerifiedTransaction) {
		let (prev, next) = if let Some(set) = self.transactions.get_mut(transaction.sender()) {
			let prev = set.worst_and_best();
			set.remove(&transaction, &self.scoring);
			(prev, set.worst_and_best())
		} else { (None, None) };
		self.update_senders_worst_and_best(prev, next);
	}

	/// Clears pool from all transactions.
	/// This causes a listener notification that all transactions were dropped.
	/// NOTE: the drop-notification order will be arbitrary.
	pub fn clear(&mut self) {
		self.mem_usage = 0;
		self.transactions.clear();
		self.best_transactions.clear();
		self.worst_transactions.clear();

		for (_hash, tx) in self.by_hash.drain() {
			self.listener.dropped(&tx)
		}
	}

	/// Removes single transaction from the pool.
	/// Depending on the `is_invalid` flag the listener
	/// will either get a `cancelled` or `invalid` notification.
	pub fn remove(&mut self, hash: &H256, is_invalid: bool) -> bool {
		if let Some(tx) = self.by_hash.remove(hash) {
			self.remove_transaction(&tx);
			if is_invalid {
				self.listener.invalid(&tx);
			} else {
				self.listener.cancelled(&tx);
			}
			true
		} else {
			false
		}
	}

	/// Removes all stalled transactions from given sender.
	fn remove_stalled<R: Ready>(&mut self, sender: &Sender, ready: &mut R) -> usize {
		// TODO [ToDr] Does not update by_hash nor best_and_wrost
		let (sender_empty, removed) = match self.transactions.get_mut(sender) {
			None => (false, 0),
			Some(ref mut transactions) => {
				let removed = transactions.cull(ready, &mut self.scoring);
				let len = removed.len();
				for tx in removed {
					self.listener.mined(&tx);
				}
				(transactions.is_empty(), len)
			}
		};

		if sender_empty {
			self.transactions.remove(sender);
		}

		removed
	}

	/// Removes all stalled transactions from given sender list (or from all senders).
	pub fn cull<R: Ready>(&mut self, senders: Option<&[Sender]>, mut ready: R) -> usize {
		let mut removed = 0;
		match senders {
			Some(senders) => {
				for sender in senders {
					removed += self.remove_stalled(sender, &mut ready);
				}
			},
			None => {
				let senders = self.transactions.keys().cloned().collect::<Vec<_>>();
				for sender in senders {
					removed += self.remove_stalled(&sender, &mut ready);
				}
			},
		}

		removed
	}

	/// Returns an iterator of pending (ready) transactions.
	pub fn pending<R: Ready>(&self, ready: R) -> PendingIterator<R, S, L> {
		PendingIterator {
			ready,
			best_transactions: self.best_transactions.clone(),
			pool: self,
		}
	}

	/// Computes the full status of the pool (including readiness).
	pub fn status<R: Ready>(&self, mut ready: R) -> Status {
		let mut stalled = 0;
		let mut pending = 0;
		let mut future = 0;

		for (_sender, transactions) in &self.transactions {
			let len = transactions.len();
			for (idx, tx) in transactions.iter().enumerate() {
				match ready.is_ready(tx) {
					Readiness::Stalled => stalled += 1,
					Readiness::Ready => pending += 1,
					Readiness::Future => {
						future += len - idx;
						break;
					}
				}
			}
		}

		Status {
			stalled,
			pending,
			future,
		}
	}

	/// Returns light status of the pool.
	pub fn light_status(&self) -> LightStatus {
		LightStatus {
			mem_usage: self.mem_usage,
			count: self.by_hash.len(),
			senders: self.transactions.len(),
		}
	}
}

/// An iterator over all pending (ready) transactions.
/// NOTE: the transactions are not removed from the queue.
/// You might remove them later by calling `cull`.
pub struct PendingIterator<'a, R, S, L> where
	S: Scoring + 'a,
	L: 'a
{
	ready: R,
	best_transactions: BTreeSet<ScoreWithRef<S::Score>>,
	pool: &'a Pool<S, L>,
}

impl<'a, R, S, L> Iterator for PendingIterator<'a, R, S, L> where
	R: Ready,
	S: Scoring,
{
	type Item = SharedTransaction;

	fn next(&mut self) -> Option<Self::Item> {
		while !self.best_transactions.is_empty() {
			let best = {
				let best = self.best_transactions.iter().next().expect("current_best is not empty; qed").clone();
				self.best_transactions.take(&best).expect("Just taken from iterator; qed")
			};

			match self.ready.is_ready(&best.transaction) {
				Readiness::Ready => {
					// retrieve next one from that sender.
					let next = self.pool.transactions
						.get(best.transaction.sender())
						.and_then(|s| s.find_next(&best.transaction, &self.pool.scoring));
					if let Some((score, tx)) = next {
						self.best_transactions.insert(ScoreWithRef::new(score, tx));
					}

					return Some(best.transaction)
				},
				state => warn!("[{:?}] Ignoring {:?} transaction.", best.transaction.hash(), state),
			}
		}

		None
	}
}
