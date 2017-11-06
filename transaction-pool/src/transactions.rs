use std::mem;
use std::marker::PhantomData;
use std::sync::Arc;

use smallvec::SmallVec;

use ready::{Ready, Readiness};
use scoring::{self, Scoring};
use {SharedTransaction, VerifiedTransaction};

#[derive(Debug)]
pub enum AddResult {
	Ok(SharedTransaction),
	TooCheapToEnter(VerifiedTransaction),
	TooCheap {
		old: SharedTransaction,
		new: VerifiedTransaction,
	},
	Replaced {
		old: SharedTransaction,
		new: SharedTransaction,
	},
	PushedOut {
		old: SharedTransaction,
		new: SharedTransaction,
	},
}

/// Represents all transactions from a particular sender ordered by nonce.
const PER_SENDER: usize = 8;
#[derive(Debug)]
pub struct Transactions<S: Scoring> {
	// TODO [ToDr] Consider using something that doesn't require shifting all records.
	transactions: SmallVec<[SharedTransaction; PER_SENDER]>,
	scores: SmallVec<[S::Score; PER_SENDER]>,
	_score: PhantomData<S>,
}

impl<S: Scoring> Default for Transactions<S> {
	fn default() -> Self {
		Transactions {
			transactions: Default::default(),
			scores: Default::default(),
			_score: PhantomData,
		}
	}
}

impl<S: Scoring> Transactions<S> {
	pub fn is_empty(&self) -> bool {
		self.transactions.is_empty()
	}

	pub fn len(&self) -> usize {
		self.transactions.len()
	}

	pub fn iter(&self) -> ::std::slice::Iter<SharedTransaction> {
		self.transactions.iter()
	}

	pub fn worst_and_best(&self) -> Option<((S::Score, SharedTransaction), (S::Score, SharedTransaction))> {
		let len = self.scores.len();
		self.scores.get(0).cloned().map(|best| {
			let worst = self.scores[len - 1].clone();
			let best_tx = self.transactions[0].clone();
			let worst_tx = self.transactions[len - 1].clone();

			((worst, worst_tx), (best, best_tx))
		})
	}

	pub fn find_next(&self, tx: &VerifiedTransaction, scoring: &S) -> Option<(S::Score, SharedTransaction)> {
		self.transactions.binary_search_by(|old| scoring.compare(old, &tx)).ok().and_then(|index| {
			let index = index + 1;
			if index >= self.scores.len() {
				None
			} else {
				Some((self.scores[index].clone(), self.transactions[index].clone()))
			}
		})
	}

	pub fn add(&mut self, tx: VerifiedTransaction, scoring: &S, max_count: usize) -> AddResult {
		let index = match self.transactions.binary_search_by(|old| scoring.compare(old, &tx)) {
			Ok(index) => index,
			Err(index) => index,
		};

		// Insert at the end.
		let len = self.transactions.len();
		if index == len {
			// trying to insert least worth transaction
			return if len == max_count {
				AddResult::TooCheapToEnter(tx)
			} else {
				let shared = Arc::new(tx);
				self.transactions.push(shared.clone());
				self.scores.push(Default::default());
				scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::InsertedAt(index));

				AddResult::Ok(shared)
			}
		}

		// Decide if the transaction should be replaced
		match scoring.choose(&self.transactions[index], &tx) {
			scoring::Choice::RejectNew => AddResult::TooCheap {
				old: self.transactions[index].clone(),
				new: tx,
			},
			scoring::Choice::InsertNew => {
				let new = Arc::new(tx);

				self.transactions.insert(index, new.clone());
				self.scores.insert(index, Default::default());
				scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::InsertedAt(index));

				if self.transactions.len() > max_count {
					let old = self.transactions.pop().expect("len is non-zero");
					self.scores.pop();
					scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::RemovedAt(self.transactions.len()));

					return AddResult::PushedOut {
						old,
						new,
					};
				}

				AddResult::Ok(new)
			},
			scoring::Choice::ReplaceOld => {
				let new = Arc::new(tx);
				let old = mem::replace(&mut self.transactions[index], new.clone());
				scoring.update_scores(&self.transactions, &mut self.scores, scoring::Change::ReplacedAt(index));

				AddResult::Replaced {
					old,
					new,
				}
			},
		}
	}

	pub fn remove(&mut self, tx: &VerifiedTransaction, scoring: &S) -> bool {
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

	pub fn cull<R: Ready>(&mut self, ready: &mut R, scoring: &S) -> SmallVec<[SharedTransaction; PER_SENDER]> {
		let mut result = SmallVec::new();
		if self.is_empty() {
			return result;
		}

		let mut first_non_stalled = 0;
		for tx in &self.transactions {
			match ready.is_ready(tx) {
				Readiness::Stalled => {
					first_non_stalled += 1;
				},
				Readiness::Ready | Readiness::Future => break,
			}
		}

		// reverse the vectors to allow
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
