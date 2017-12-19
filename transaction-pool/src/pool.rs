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

use std::sync::Arc;
use std::collections::{HashMap, BTreeSet};

use bigint::hash::{H160, H256};

use error;
use listener::{Listener, NoopListener};
use options::Options;
use ready::{Ready, Readiness};
use scoring::{Scoring, ScoreWithRef};
use status::{LightStatus, Status};
use transactions::{AddResult, Transactions};

use {VerifiedTransaction};

type Sender = H160;

/// A transaction pool.
#[derive(Debug)]
pub struct Pool<T, S: Scoring<T>, L = NoopListener> {
	listener: L,
	scoring: S,
	options: Options,
	mem_usage: usize,

	transactions: HashMap<Sender, Transactions<T, S>>,
	by_hash: HashMap<H256, Arc<T>>,

	best_transactions: BTreeSet<ScoreWithRef<T, S::Score>>,
	worst_transactions: BTreeSet<ScoreWithRef<T, S::Score>>,
}

impl<T: VerifiedTransaction, S: Scoring<T> + Default> Default for Pool<T, S> {
	fn default() -> Self {
		Self::with_scoring(S::default(), Options::default())
	}
}

impl<T: VerifiedTransaction, S: Scoring<T> + Default> Pool<T, S> {
	/// Creates a new `Pool` with given options
	/// and default `Scoring` and `Listener`.
	pub fn with_options(options: Options) -> Self {
		Self::with_scoring(S::default(), options)
	}
}

impl<T: VerifiedTransaction, S: Scoring<T>> Pool<T, S> {
	/// Creates a new `Pool` with given `Scoring` and options.
	pub fn with_scoring(scoring: S, options: Options) -> Self {
		Self::new(NoopListener, scoring, options)
	}
}


const INITIAL_NUMBER_OF_SENDERS: usize = 16;

impl<T, S, L> Pool<T, S, L> where
	T: VerifiedTransaction,
	S: Scoring<T>,
	L: Listener<T>,
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

	/// Attempts to import new transaction to the pool, returns a `Arc<T>` or an `Error`.
	///
	/// NOTE: Since `Ready`ness is separate from the pool it's possible to import stalled transactions.
	/// It's the caller responsibility to make sure that's not the case.
	///
	/// NOTE: The transaction may push out some other transactions from the pool
	/// either because of limits (see `Options`) or because `Scoring` decides that the transaction
	/// replaces an existing transaction from that sender.
	/// If any limit is reached the transaction with the lowest `Score` is evicted to make room.
	///
	/// The `Listener` will be informed on any drops or rejections.
	pub fn import(&mut self, mut transaction: T) -> error::Result<Arc<T>> {
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
						s.finalize_remove(removed.hash());
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

		let (result, prev_state, current_state) = {
			let transactions = self.transactions.entry(*transaction.sender()).or_insert_with(Transactions::default);
			// get worst and best transactions for comparison
			let prev = transactions.worst_and_best();
			let result = transactions.add(transaction, &self.scoring, self.options.max_per_sender);
			let current = transactions.worst_and_best();
			(result, prev, current)
		};

		// update best and worst transactions from this sender (if required)
		self.update_senders_worst_and_best(prev_state, current_state);

		match result {
			AddResult::Ok(tx) => {
				self.listener.added(&tx, None);
				self.finalize_insert(&tx, None);
				Ok(tx)
			},
			AddResult::PushedOut { new, old } |
			AddResult::Replaced { new, old } => {
				self.listener.added(&new, Some(&old));
				self.finalize_insert(&new, Some(&old));
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
	fn finalize_insert(&mut self, new: &Arc<T>, old: Option<&Arc<T>>) {
		self.mem_usage += new.mem_usage();
		self.by_hash.insert(*new.hash(), new.clone());

		if let Some(old) = old {
			self.finalize_remove(old.hash());
		}
	}

	/// Updates the pool statistics if transaction was removed.
	fn finalize_remove(&mut self, hash: &H256) -> Option<Arc<T>> {
		self.by_hash.remove(hash).map(|old| {
			self.mem_usage -= old.mem_usage();
			old
		})
	}

	/// Updates best and worst transactions from a sender.
	fn update_senders_worst_and_best(
		&mut self,
		previous: Option<((S::Score, Arc<T>), (S::Score, Arc<T>))>,
		current: Option<((S::Score, Arc<T>), (S::Score, Arc<T>))>,
	) {
		let worst_collection = &mut self.worst_transactions;
		let best_collection = &mut self.best_transactions;

		let is_same = |a: &(S::Score, Arc<T>), b: &(S::Score, Arc<T>)| {
			a.0 == b.0 && a.1.hash() == b.1.hash()
		};

		let update = |collection: &mut BTreeSet<_>, (score, tx), remove| if remove {
			collection.remove(&ScoreWithRef::new(score, tx));
		} else {
			collection.insert(ScoreWithRef::new(score, tx));
		};

		match (previous, current) {
			(None, Some((worst, best))) => {
				update(worst_collection, worst, false);
				update(best_collection, best, false);
			},
			(Some((worst, best)), None) => {
				// all transactions from that sender has been removed.
				// We can clear a hashmap entry.
				self.transactions.remove(worst.1.sender());
				update(worst_collection, worst, true);
				update(best_collection, best, true);
			},
			(Some((w1, b1)), Some((w2, b2))) => {
				if !is_same(&w1, &w2) {
					update(worst_collection, w1, true);
					update(worst_collection, w2, false);
				}
				if !is_same(&b1, &b2) {
					update(best_collection, b1, true);
					update(best_collection, b2, false);
				}
			},
			(None, None) => {},
		}
	}

	/// Attempts to remove the worst transaction from the pool if it's worse than the given one.
	fn remove_worst(&mut self, transaction: &T) -> error::Result<Arc<T>> {
		let to_remove = match self.worst_transactions.iter().next_back() {
			// No elements to remove? and the pool is still full?
			None => {
				warn!("The pool is full but there are no transactions to remove.");
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
		self.remove_from_set(to_remove.transaction.sender(), |set, scoring| {
			set.remove(&to_remove.transaction, scoring)
		});
		Ok(to_remove.transaction)
	}

	/// Removes transaction from sender's transaction `HashMap`.
	fn remove_from_set<R, F: FnOnce(&mut Transactions<T, S>, &S) -> R>(&mut self, sender: &Sender, f: F) -> Option<R> {
		let (prev, next, result) = if let Some(set) = self.transactions.get_mut(sender) {
			let prev = set.worst_and_best();
			let result = f(set, &self.scoring);
			(prev, set.worst_and_best(), result)
		} else {
			return None;
		};

		self.update_senders_worst_and_best(prev, next);
		Some(result)
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
		if let Some(tx) = self.finalize_remove(hash) {
			self.remove_from_set(tx.sender(), |set, scoring| {
				set.remove(&tx, scoring)
			});
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
	fn remove_stalled<R: Ready<T>>(&mut self, sender: &Sender, ready: &mut R) -> usize {
		let removed_from_set = self.remove_from_set(sender, |transactions, scoring| {
			transactions.cull(ready, scoring)
		});

		match removed_from_set {
			Some(removed) => {
				let len = removed.len();
				for tx in removed {
					self.finalize_remove(tx.hash());
					self.listener.mined(&tx);
				}
				len
			},
			None => 0,
		}
	}

	/// Removes all stalled transactions from given sender list (or from all senders).
	pub fn cull<R: Ready<T>>(&mut self, senders: Option<&[Sender]>, mut ready: R) -> usize {
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
	pub fn pending<R: Ready<T>>(&self, ready: R) -> PendingIterator<T, R, S, L> {
		PendingIterator {
			ready,
			best_transactions: self.best_transactions.clone(),
			pool: self,
		}
	}

	/// Computes the full status of the pool (including readiness).
	pub fn status<R: Ready<T>>(&self, mut ready: R) -> Status {
		let mut status = Status::default();

		for (_sender, transactions) in &self.transactions {
			let len = transactions.len();
			for (idx, tx) in transactions.iter().enumerate() {
				match ready.is_ready(tx) {
					Readiness::Stalled => status.stalled += 1,
					Readiness::Ready => status.pending += 1,
					Readiness::Future => {
						status.future += len - idx;
						break;
					}
				}
			}
		}

		status
	}

	/// Returns light status of the pool.
	pub fn light_status(&self) -> LightStatus {
		LightStatus {
			mem_usage: self.mem_usage,
			transaction_count: self.by_hash.len(),
			senders: self.transactions.len(),
		}
	}
}

/// An iterator over all pending (ready) transactions.
/// NOTE: the transactions are not removed from the queue.
/// You might remove them later by calling `cull`.
pub struct PendingIterator<'a, T, R, S, L> where
	T: VerifiedTransaction + 'a,
	S: Scoring<T> + 'a,
	L: 'a,
{
	ready: R,
	best_transactions: BTreeSet<ScoreWithRef<T, S::Score>>,
	pool: &'a Pool<T, S, L>,
}

impl<'a, T, R, S, L> Iterator for PendingIterator<'a, T, R, S, L> where
	T: VerifiedTransaction,
	R: Ready<T>,
	S: Scoring<T>,
{
	type Item = Arc<T>;

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
