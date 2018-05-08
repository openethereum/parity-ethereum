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

use std::{fmt, mem};

use smallvec::SmallVec;

use ready::{Ready, Readiness};
use scoring::{self, Scoring};
use pool::Transaction;

#[derive(Debug)]
pub enum AddResult<T, S> {
	Ok(T),
	TooCheapToEnter(T, S),
	TooCheap {
		old: T,
		new: T,
	},
	Replaced {
		old: T,
		new: T,
	},
	PushedOut {
		old: T,
		new: T,
	},
}

/// Represents all transactions from a particular sender ordered by nonce.
const PER_SENDER: usize = 8;
#[derive(Debug)]
pub struct Transactions<T, S: Scoring<T>> {
	// TODO [ToDr] Consider using something that doesn't require shifting all records.
	transactions: SmallVec<[Transaction<T>; PER_SENDER]>,
	scores: SmallVec<[S::Score; PER_SENDER]>,
}

impl<T, S: Scoring<T>> Default for Transactions<T, S> {
	fn default() -> Self {
		Transactions {
			transactions: Default::default(),
			scores: Default::default(),
		}
	}
}

impl<T: fmt::Debug, S: Scoring<T>> Transactions<T, S> {
	pub fn is_empty(&self) -> bool {
		self.transactions.is_empty()
	}

	pub fn len(&self) -> usize {
		self.transactions.len()
	}

	pub fn iter(&self) -> ::std::slice::Iter<Transaction<T>> {
		self.transactions.iter()
	}

	pub fn worst_and_best(&self) -> Option<((S::Score, Transaction<T>), (S::Score, Transaction<T>))> {
		let len = self.scores.len();
		self.scores.get(0).cloned().map(|best| {
			let worst = self.scores[len - 1].clone();
			let best_tx = self.transactions[0].clone();
			let worst_tx = self.transactions[len - 1].clone();

			((worst, worst_tx), (best, best_tx))
		})
	}

	pub fn find_next(&self, tx: &T, scoring: &S) -> Option<(S::Score, Transaction<T>)> {
		self.transactions.binary_search_by(|old| scoring.compare(old, &tx)).ok().and_then(|index| {
			let index = index + 1;
			if index < self.scores.len() {
				Some((self.scores[index].clone(), self.transactions[index].clone()))
			} else {
				None
			}
		})
	}

	fn push_cheapest_transaction(&mut self, tx: Transaction<T>, scoring: &S, max_count: usize) -> AddResult<Transaction<T>, S::Score> {
		let index = self.transactions.len();
		if index == max_count {
			let min_score = self.scores[index - 1].clone();
			AddResult::TooCheapToEnter(tx, min_score)
		} else {
			self.transactions.push(tx.clone());
			self.scores.push(Default::default());
			scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::InsertedAt(index));

			AddResult::Ok(tx)
		}
	}

	pub fn update_scores(&mut self, scoring: &S, event: S::Event) {
		scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::Event(event));
	}

	pub fn add(&mut self, new: Transaction<T>, scoring: &S, max_count: usize) -> AddResult<Transaction<T>, S::Score> {
		let index = match self.transactions.binary_search_by(|old| scoring.compare(old, &new)) {
			Ok(index) => index,
			Err(index) => index,
		};

		// Insert at the end.
		if index == self.transactions.len() {
			return self.push_cheapest_transaction(new, scoring, max_count)
		}

		// Decide if the transaction should replace some other.
		match scoring.choose(&self.transactions[index], &new) {
			// New transaction should be rejected
			scoring::Choice::RejectNew => AddResult::TooCheap {
				old: self.transactions[index].clone(),
				new,
			},
			// New transaction should be kept along with old ones.
			scoring::Choice::InsertNew => {
				self.transactions.insert(index, new.clone());
				self.scores.insert(index, Default::default());
				scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::InsertedAt(index));

				if self.transactions.len() > max_count {
					let old = self.transactions.pop().expect("len is non-zero");
					self.scores.pop();
					scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::RemovedAt(self.transactions.len()));

					AddResult::PushedOut {
						old,
						new,
					}
				} else {
					AddResult::Ok(new)
				}
			},
			// New transaction is replacing some other transaction already in the queue.
			scoring::Choice::ReplaceOld => {
				let old = mem::replace(&mut self.transactions[index], new.clone());
				scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::ReplacedAt(index));

				AddResult::Replaced {
					old,
					new,
				}
			},
		}
	}

	pub fn remove(&mut self, tx: &T, scoring: &S) -> bool {
		let index = match self.transactions.binary_search_by(|old| scoring.compare(old, tx)) {
			Ok(index) => index,
			Err(_) => {
				warn!("Attempting to remove non-existent transaction {:?}", tx);
				return false;
			},
		};

		self.transactions.remove(index);
		self.scores.remove(index);
		// Update scoring
		scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::RemovedAt(index));
		return true;
	}

	pub fn cull<R: Ready<T>>(&mut self, ready: &mut R, scoring: &S) -> SmallVec<[Transaction<T>; PER_SENDER]> {
		let mut result = SmallVec::new();
		if self.is_empty() {
			return result;
		}

		let mut first_non_stalled = 0;
		for tx in &self.transactions {
			match ready.is_ready(tx) {
				Readiness::Stale => {
					first_non_stalled += 1;
				},
				Readiness::Ready | Readiness::Future => break,
			}
		}

		if first_non_stalled == 0 {
			return result;
		}

		// reverse the vectors to easily remove first elements.
		self.transactions.reverse();
		self.scores.reverse();

		for _ in 0..first_non_stalled {
			self.scores.pop();
			result.push(
				self.transactions.pop().expect("first_non_stalled is never greater than transactions.len(); qed")
			);
		}

		self.transactions.reverse();
		self.scores.reverse();

		// update scoring
		scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::Culled(result.len()));

		// reverse the result to maintain correct order.
		result.reverse();
		result
	}
}
