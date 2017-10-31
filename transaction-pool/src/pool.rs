use std::{cmp, mem};
use std::sync::Arc;
use std::collections::{HashMap, BTreeSet};

use smallvec::SmallVec;

use error;
use {Listener, NoopListener};
use {Scoring, ScoringDecision, ScoringChange};
use {VerifiedTransaction, SharedTransaction, H256};

type Sender = ::Address;

#[derive(Debug)]
pub struct Options {
	max_count: usize,
	max_per_sender: usize,
	max_mem_usage: usize,
}

impl Default for Options {
	fn default() -> Self {
		Options {
			max_count: 1024,
			max_per_sender: 16,
			max_mem_usage: 8 * 1024 * 1024,
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct Status {
	pub mem_usage: usize,
	pub count: usize,
}

#[derive(Debug, Clone)]
struct ScoreWithRef<T> {
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
		self.score.cmp(&other.score)
	}
}
impl<T: cmp::Ord> PartialOrd for ScoreWithRef<T> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl<T: cmp::Ord>  PartialEq for ScoreWithRef<T> {
	fn eq(&self, other: &Self) -> bool {
		self.score == other.score
	}
}
impl<T: cmp::Ord> Eq for ScoreWithRef<T> {}


#[derive(Debug)]
pub struct Pool<S: Scoring, L = NoopListener> {
	listener: L,
	scoring: S,
	options: Options,
	mem_usage: usize,

	transactions: HashMap<Sender, Transactions<S::Score>>,
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
	pub fn with_options(options: Options) -> Self {
		Self::with_scoring(S::default(), options)
	}
}

impl<S: Scoring> Pool<S> {
	pub fn with_scoring(scoring: S, options: Options) -> Self {
		Self::new(NoopListener, scoring, options)
	}
}

impl<S, L> Pool<S, L> where
	S: Scoring,
	L: Listener,
{
	pub fn new(listener: L, scoring: S, options: Options) -> Self {
		let transactions = HashMap::with_capacity(16);
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
	pub fn import(&mut self, transaction: VerifiedTransaction) -> error::Result<()> {
		let sender = transaction.sender();
		let mem_usage = transaction.mem_usage();
		let hash = transaction.hash();

		ensure!(!self.by_hash.contains_key(&hash), error::ErrorKind::AlreadyImported(hash));

		{
			let remove_worst = |s: &mut Self, transaction| {
				s.remove_worst(transaction).map_err(|err| {
					s.listener.rejected(transaction);
					err
				}).map(|removed| {
					s.listener.dropped(&removed);
					s.removed(&removed);
					()
				})
			};

			while self.by_hash.len() + 1 > self.options.max_count {
				remove_worst(self, &transaction)?;
			}

			while self.mem_usage + mem_usage > self.options.max_mem_usage {
				remove_worst(self, &transaction)?;
			}
		}

		let result = {
			let transactions = self.transactions.entry(sender).or_insert_with(Transactions::default);
			// get worst and best transactions for comparison
			let prev = transactions.worst_and_best();
			let result = transactions.add(transaction, &self.scoring, self.options.max_per_sender);
			let current = transactions.worst_and_best();
			(result, prev, current)
		};

		// update best and worst transactions from this sender (if required)
		self.update_senders_worst_and_best(result.1, result.2);

		match result.0 {
			AddResult::Ok(ref tx) => {
				self.listener.added(tx, None);
				self.added(tx, None);
			},
			AddResult::PushedOut { ref new, ref old } |
			AddResult::Replaced { ref new, ref old } => {
				self.listener.added(new, Some(old));
				self.added(new, Some(old));
			},
			AddResult::TooCheap { new, old } => {
				self.listener.rejected(&new);
				bail!(error::ErrorKind::TooCheapToReplace(old, new))
			},
			AddResult::TooCheapToEnter(new) => {
				self.listener.rejected(&new);
				bail!(error::ErrorKind::TooCheapToEnter(new.hash()))
			}
		}

		Ok(())
	}

	fn added(&mut self, new: &SharedTransaction, old: Option<&SharedTransaction>) {
		self.mem_usage += new.mem_usage();
		self.by_hash.insert(new.hash(), new.clone());

		if let Some(old) = old {
			self.removed(old)
		}
	}

	fn removed(&mut self, old: &SharedTransaction) {
		self.mem_usage -= old.mem_usage();
		self.by_hash.remove(&old.hash());
	}

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

	fn remove_worst(&mut self, transaction: &VerifiedTransaction) -> error::Result<SharedTransaction> {
		let to_remove = match self.worst_transactions.iter().next_back() {
			// No elements to remove? and the queue is still full?
			None => {
				warn!("The queue is full but there is no transaction to remove.");
				return Err(error::ErrorKind::TooCheapToEnter(transaction.hash()).into());
			},
			Some(old) => match self.scoring.decide(&old.transaction, transaction) {
				// New transaction is better than the worst one so we can replace it.
				ScoringDecision::Replace | ScoringDecision::Insert => old.clone(),
				ScoringDecision::Reject => return Err(error::ErrorKind::TooCheapToEnter(transaction.hash()).into()),
			},
		};

		// Remove worst transaction from the list
		self.worst_transactions.remove(&to_remove);
		// Might also be on the best_transactions list
		self.best_transactions.remove(&to_remove);
		// Remove from transaction set
		let sender = to_remove.transaction.sender();
		let (prev, next) = if let Some(set) = self.transactions.get_mut(&sender) {
			let prev = set.worst_and_best();
			set.remove(&to_remove.transaction, &self.scoring);
			(prev, set.worst_and_best())
		} else { (None, None) };
		self.update_senders_worst_and_best(prev, next);
		Ok(to_remove.transaction)
	}

	pub fn clear(&mut self) {
		self.mem_usage = 0;
		self.transactions.clear();
		self.best_transactions.clear();
		self.worst_transactions.clear();
		for (_hash, tx) in self.by_hash.drain() {
			self.listener.dropped(&tx)
		}
	}

	pub fn status(&self) -> Status {
		Status {
			mem_usage: self.mem_usage,
			count: self.by_hash.len(),
		}
	}
}

// TODO [ToDr] Add lifetime here (avoid cloning Arcs)
#[derive(Debug)]
enum AddResult {
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
#[derive(Default, Debug)]
struct Transactions<T> {
	// TODO [ToDr] Consider using VecDeque?
	transactions: SmallVec<[SharedTransaction; PER_SENDER]>,
	scores: SmallVec<[T; PER_SENDER]>,
}

impl<T: Clone> Transactions<T> {
	pub fn worst_and_best(&self) -> Option<((T, SharedTransaction), (T, SharedTransaction))> {
		let len = self.scores.len();
		self.scores.get(0).cloned().map(|worst| {
			let best = self.scores[len - 1].clone();
			let worst_tx = self.transactions[0].clone();
			let best_tx = self.transactions[len - 1].clone();

			((worst, worst_tx), (best, best_tx))
		})
	}
}

impl<T: cmp::Ord + Clone + Default> Transactions<T> {
	pub fn add<S>(&mut self, tx: VerifiedTransaction, scoring: &S, max_count: usize) -> AddResult where
		S: Scoring<Score=T>,
	{
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
				scoring.update_scores(&self.transactions, &mut self.scores, ScoringChange::InsertedAt(index));

				AddResult::Ok(shared)
			}
		}

		// Decide if the transaction should be replaced
		match scoring.decide(&self.transactions[index], &tx) {
			ScoringDecision::Reject => AddResult::TooCheap {
				old: self.transactions[index].clone(),
				new: tx,
			},
			ScoringDecision::Insert => {
				let new = Arc::new(tx);

				self.transactions.insert(index, new.clone());
				self.scores.insert(index, Default::default());
				scoring.update_scores(&self.transactions, &mut self.scores, ScoringChange::InsertedAt(index));

				if self.transactions.len() > max_count {
					let old = self.transactions.pop().expect("len is non-zero");
					self.scores.pop();
					scoring.update_scores(&self.transactions, &mut self.scores, ScoringChange::RemovedAt(self.transactions.len()));

					return AddResult::PushedOut {
						old,
						new,
					};
				}

				AddResult::Ok(new)
			},
			ScoringDecision::Replace => {
				let new = Arc::new(tx);
				let old = mem::replace(&mut self.transactions[index], new.clone());
				scoring.update_scores(&self.transactions, &mut self.scores, ScoringChange::ReplacedAt(index));

				AddResult::Replaced {
					old,
					new,
				}
			},
		}
	}
	pub fn remove<S>(&mut self, tx: &VerifiedTransaction, scoring: &S) -> bool where
		S: Scoring<Score=T>,
	{
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
		scoring.update_scores(&self.transactions, &mut self.scores, ScoringChange::RemovedAt(index));
		return true;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use U256;

	#[derive(Default)]
	struct DummyScoring;

	impl Scoring for DummyScoring {
		type Score = u64;

		fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering {
			old.nonce.cmp(&other.nonce)
		}

		fn decide(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> ScoringDecision {
			let decision = match old.gas_price.cmp(&new.gas_price) {
				cmp::Ordering::Greater | cmp::Ordering::Equal => ScoringDecision::Reject,
				_ => if old.nonce == new.nonce {
					ScoringDecision::Replace
				} else {
					ScoringDecision::Insert
				}
			};

			println!("Comparing: {:?} vs {:?}. Decision: {:?}", old, new, decision);
			decision
		}

		fn update_scores(&self, txs: &[SharedTransaction], scores: &mut [Self::Score], _change: ScoringChange) {
			for i in 0..txs.len() {
				scores[i] = txs[i].gas_price.0;
			}
		}
	}

	type TestPool = Pool<DummyScoring>;

	#[derive(Debug, Default)]
	struct TransactionBuilder {
		nonce: U256,
		gas_price: U256,
		sender: Sender,
	}

	impl TransactionBuilder {
		pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
			self.nonce = nonce.into();
			self
		}

		pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
			self.gas_price = gas_price.into();
			self
		}

		pub fn sender<T: Into<Sender>>(mut self, sender: T) -> Self {
			self.sender = sender.into();
			self
		}

		pub fn new(self) -> VerifiedTransaction {
			VerifiedTransaction {
				hash: (self.nonce.0 ^ self.gas_price.0 ^ self.sender.0).into(),
				nonce: self.nonce,
				gas_price: self.gas_price,
				sender: self.sender,
			}
		}
	}

	fn tx() -> TransactionBuilder {
		TransactionBuilder::default()
	}

	#[test]
	fn should_clear_queue() {
		// given
		let mut txq = TestPool::default();
		assert_eq!(txq.status(), Status {
			mem_usage: 0,
			count: 0,
		});
		let tx1 = tx().nonce(0).new();
		let tx2 = tx().nonce(1).new();

		// add
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap();
		assert_eq!(txq.status(), Status {
			mem_usage: 1,
			count: 2,
		});

		// when
		txq.clear();

		// then
		assert_eq!(txq.status(), Status {
			mem_usage: 0,
			count: 0,
		});
	}

	#[test]
	fn should_not_allow_same_transaction_twice() {
		// given
		let mut txq = TestPool::default();
		let tx1 = tx().nonce(0).new();
		let tx2 = tx().nonce(0).new();

		// when
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap_err();

		// then
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_replace_transaction() {
		// given
		let mut txq = TestPool::default();
		let tx1 = tx().nonce(0).gas_price(1).new();
		let tx2 = tx().nonce(0).gas_price(2).new();

		// when
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap();

		// then
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_reject_if_above_count() {
		// given
		let mut txq = TestPool::with_options(Options {
			max_count: 1,
			..Default::default()
		});

		// Reject second
		let tx1 = tx().nonce(0).new();
		let tx2 = tx().nonce(1).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);

		txq.clear();

		// Replace first
		let tx1 = tx().nonce(0).new();
		let tx2 = tx().nonce(0).sender(1).gas_price(2).new();
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap();
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_reject_if_above_mem_usage() {
		// given
		let mut txq = TestPool::with_options(Options {
			max_mem_usage: 1,
			..Default::default()
		});

		// Reject second
		let tx1 = tx().nonce(1).new();
		let tx2 = tx().nonce(2).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);

		txq.clear();

		// Replace first
		let tx1 = tx().nonce(1).new();
		let tx2 = tx().nonce(1).sender(1).gas_price(2).new();
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap();
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_reject_if_above_sender_count() {
		// given
		let mut txq = TestPool::with_options(Options {
			max_per_sender: 1,
			..Default::default()
		});

		// Reject second
		let tx1 = tx().nonce(1).new();
		let tx2 = tx().nonce(2).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);

		txq.clear();

		// Replace first
		let tx1 = tx().nonce(1).new();
		let tx2 = tx().nonce(2).gas_price(2).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		// This results in error because we also compare nonces
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_notify_listener() {
		// all 3 cases
		assert_eq!(false, true)
	}
}
