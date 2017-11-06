use std::{cmp, fmt};
use {VerifiedTransaction, SharedTransaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
	RejectNew,
	ReplaceOld,
	InsertNew,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
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
	/// Usually the score will have to be re-computed from scratch.
	Culled(usize),
}

pub trait Scoring {
	type Score: cmp::Ord + Clone + Default + fmt::Debug;

	fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering;

	fn choose(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> Choice;

	fn update_scores(&self, txs: &[SharedTransaction], scores: &mut [Self::Score], change: Change);

	fn should_replace(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> bool;
}

#[derive(Debug, Clone)]
pub struct ScoreWithRef<T> {
	pub score: T,
	pub transaction: SharedTransaction,
}
impl<T> ScoreWithRef<T> {
	pub fn new(score: T, transaction: SharedTransaction) -> Self {
		ScoreWithRef { score, transaction }
	}
}
impl<T: cmp::Ord> Ord for ScoreWithRef<T> {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		other.score.cmp(&self.score)
			.then(other.transaction.insertion_id.cmp(&self.transaction.insertion_id))
	}
}
impl<T: cmp::Ord> PartialOrd for ScoreWithRef<T> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl<T: cmp::Ord>  PartialEq for ScoreWithRef<T> {
	fn eq(&self, other: &Self) -> bool {
		self.score == other.score && self.transaction.insertion_id == other.transaction.insertion_id
	}
}
impl<T: cmp::Ord> Eq for ScoreWithRef<T> {}
