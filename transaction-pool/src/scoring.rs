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

//! A transactions ordering abstraction.

use std::{cmp, fmt};
use pool::Transaction;

/// Represents a decision what to do with
/// a new transaction that tries to enter the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
	/// New transaction should be rejected
	/// (i.e. the old transaction that occupies the same spot
	/// is better).
	RejectNew,
	/// The old transaction should be dropped
	/// in favour of the new one.
	ReplaceOld,
	/// The new transaction should be inserted
	/// and both (old and new) should stay in the pool.
	InsertNew,
}

/// Describes a reason why the `Score` of transactions
/// should be updated.
/// The `Scoring` implementations can use this information
/// to update the `Score` table more efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change<T = ()> {
	/// New transaction has been inserted at given index.
	/// The Score at that index is initialized with default value
	/// and needs to be filled in.
	InsertedAt(usize),
	/// The transaction has been removed at given index and other transactions
	/// shifted to it's place.
	/// The scores were removed and shifted as well.
	/// For simple scoring algorithms no action is required here.
	RemovedAt(usize),
	/// The transaction at given index has replaced a previous transaction.
	/// The score at that index needs to be update (it contains value from previous transaction).
	ReplacedAt(usize),
	/// Given number of stalled transactions has been culled from the beginning.
	/// The scores has been removed from the beginning as well.
	/// For simple scoring algorithms no action is required here.
	Culled(usize),
	/// Custom event to update the score triggered outside of the pool.
	/// Handling this event is up to scoring implementation.
	Event(T),
}

/// A transaction ordering.
///
/// The implementation should decide on order of transactions in the pool.
/// Each transaction should also get assigned a `Score` which is used to later
/// prioritize transactions in the pending set.
///
/// Implementation notes:
/// - Returned `Score`s should match ordering of `compare` method.
/// - `compare` will be called only within a context of transactions from the same sender.
/// - `choose` may be called even if `compare` returns `Ordering::Equal`
/// - `should_replace` is used to decide if new transaction should push out an old transaction already in the queue.
/// - `Score`s and `compare` should align with `Ready` implementation.
///
/// Example: Natural ordering of Ethereum transactions.
/// - `compare`: compares transaction `nonce` ()
/// - `choose`: compares transactions `gasPrice` (decides if old transaction should be replaced)
/// - `update_scores`: score defined as `gasPrice` if `n==0` and `max(scores[n-1], gasPrice)` if `n>0`
/// - `should_replace`: compares `gasPrice` (decides if transaction from a different sender is more valuable)
///
pub trait Scoring<T>: fmt::Debug {
	/// A score of a transaction.
	type Score: cmp::Ord + Clone + Default + fmt::Debug;
	/// Custom scoring update event type.
	type Event: fmt::Debug;

	/// Decides on ordering of `T`s from a particular sender.
	fn compare(&self, old: &T, other: &T) -> cmp::Ordering;

	/// Decides how to deal with two transactions from a sender that seem to occupy the same slot in the queue.
	fn choose(&self, old: &T, new: &T) -> Choice;

	/// Updates the transaction scores given a list of transactions and a change to previous scoring.
	/// NOTE: you can safely assume that both slices have the same length.
	/// (i.e. score at index `i` represents transaction at the same index)
	fn update_scores(&self, txs: &[Transaction<T>], scores: &mut [Self::Score], change: Change<Self::Event>);

	/// Decides if `new` should push out `old` transaction from the pool.
	fn should_replace(&self, old: &T, new: &T) -> bool;
}

/// A score with a reference to the transaction.
#[derive(Debug)]
pub struct ScoreWithRef<T, S> {
	/// Score
	pub score: S,
	/// Shared transaction
	pub transaction: Transaction<T>,
}

impl<T, S> ScoreWithRef<T, S> {
	/// Creates a new `ScoreWithRef`
	pub fn new(score: S, transaction: Transaction<T>) -> Self {
		ScoreWithRef { score, transaction }
	}
}

impl<T, S: Clone> Clone for ScoreWithRef<T, S> {
	fn clone(&self) -> Self {
		ScoreWithRef {
			score: self.score.clone(),
			transaction: self.transaction.clone(),
		}
	}
}

impl<S: cmp::Ord, T> Ord for ScoreWithRef<T, S> {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		other.score.cmp(&self.score)
			.then(other.transaction.insertion_id.cmp(&self.transaction.insertion_id))
	}
}

impl<S: cmp::Ord, T> PartialOrd for ScoreWithRef<T, S> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<S: cmp::Ord, T>  PartialEq for ScoreWithRef<T, S> {
	fn eq(&self, other: &Self) -> bool {
		self.score == other.score && self.transaction.insertion_id == other.transaction.insertion_id
	}
}

impl<S: cmp::Ord, T> Eq for ScoreWithRef<T, S> {}
