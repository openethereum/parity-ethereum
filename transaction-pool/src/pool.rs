use std::{cmp, mem, fmt};
use std::sync::Arc;
use std::collections::{HashMap, BTreeSet};

use smallvec::SmallVec;

use error;
use {Readiness};
use {Listener, NoopListener};
use {Scoring, ScoringChoice, ScoringChange};
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
	pub fn import(&mut self, transaction: VerifiedTransaction) -> error::Result<SharedTransaction> {
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
				self.listener.rejected(&new);
				bail!(error::ErrorKind::TooCheapToReplace(old, new))
			},
			AddResult::TooCheapToEnter(new) => {
				self.listener.rejected(&new);
				bail!(error::ErrorKind::TooCheapToEnter(new.hash()))
			}
		}
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
				// all transactions from that sender has been removed.
				// We can clear a hashmap entry.
				self.transactions.remove(&worst.1.sender());
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
			Some(old) => if self.scoring.should_replace(&old.transaction, transaction) {
				// New transaction is better than the worst one so we can replace it.
				old.clone()
			} else {
				// otherwise fail
				return Err(error::ErrorKind::TooCheapToEnter(transaction.hash()).into())
			},
		};

		// Remove from transaction set
		self.remove_transaction(&to_remove.transaction);
		Ok(to_remove.transaction)
	}

	/// Removes transaction from sender's transaction `HashMap`.
	fn remove_transaction(&mut self, transaction: &VerifiedTransaction) {
		let sender = transaction.sender();
		let (prev, next) = if let Some(set) = self.transactions.get_mut(&sender) {
			let prev = set.worst_and_best();
			set.remove(&transaction, &self.scoring);
			(prev, set.worst_and_best())
		} else { (None, None) };
		self.update_senders_worst_and_best(prev, next);
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

	pub fn pending<R: Readiness>(&self, ready: R) -> PendingIterator<R, S, L> {
		PendingIterator {
			ready,
			best_transactions: self.best_transactions.clone(),
			pool: self,
		}
	}

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
}

pub struct PendingIterator<'a, R, S, L> where
	S: Scoring + 'a,
	L: 'a
{
	ready: R,
	best_transactions: BTreeSet<ScoreWithRef<S::Score>>,
	pool: &'a Pool<S, L>,
}

impl<'a, R, S, L> Iterator for PendingIterator<'a, R, S, L> where
	R: Readiness,
	S: Scoring,
{
	type Item = SharedTransaction;

	fn next(&mut self) -> Option<Self::Item> {
		while !self.best_transactions.is_empty() {
			println!("Best: {:?}", self.best_transactions);
			let best = {
				let best = self.best_transactions.iter().next().expect("current_best is not empty; qed").clone();
				self.best_transactions.take(&best).expect("Just taken from iterator; qed")
			};

			if self.ready.is_ready(&best.transaction) {
				let sender = best.transaction.sender();

				// retrieve next one from that sender.
				let next = self.pool.transactions
					.get(&sender)
					.and_then(|s| s.find_next(&best.transaction, &self.pool.scoring));
				if let Some((score, tx)) = next {
					self.best_transactions.insert(ScoreWithRef::new(score, tx));
				}

				return Some(best.transaction)
			}
		}

		None
	}
}

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
	// TODO [ToDr] Consider using something that doesn't require shifting all records.
	transactions: SmallVec<[SharedTransaction; PER_SENDER]>,
	scores: SmallVec<[T; PER_SENDER]>,
}

impl<T: Clone> Transactions<T> {
	pub fn worst_and_best(&self) -> Option<((T, SharedTransaction), (T, SharedTransaction))> {
		let len = self.scores.len();
		self.scores.get(0).cloned().map(|best| {
			let worst = self.scores[len - 1].clone();
			let best_tx = self.transactions[0].clone();
			let worst_tx = self.transactions[len - 1].clone();

			((worst, worst_tx), (best, best_tx))
		})
	}
}

impl<T: cmp::Ord + Clone + Default + fmt::Debug> Transactions<T> {
	pub fn find_next<S>(&self, tx: &VerifiedTransaction, scoring: &S) -> Option<(T, SharedTransaction)> where
		S: Scoring<Score=T>,
	{
		self.transactions.binary_search_by(|old| scoring.compare(old, &tx)).ok().and_then(|index| {
			let index = index + 1;
			if index >= self.scores.len() {
				None
			} else {
				Some((self.scores[index].clone(), self.transactions[index].clone()))
			}
		})
	}

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
		match scoring.choose(&self.transactions[index], &tx) {
			ScoringChoice::RejectNew => AddResult::TooCheap {
				old: self.transactions[index].clone(),
				new: tx,
			},
			ScoringChoice::InsertNew => {
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
			ScoringChoice::ReplaceOld => {
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
	use std::rc::Rc;
	use std::cell::Cell;
	use super::*;
	use U256;

	#[derive(Default)]
	struct DummyScoring;

	impl Scoring for DummyScoring {
		type Score = u64;

		fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering {
			old.nonce.cmp(&other.nonce)
		}

		fn choose(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> ScoringChoice {
			let decision = if old.nonce == new.nonce {
				if new.gas_price > old.gas_price {
					ScoringChoice::ReplaceOld
				} else {
					ScoringChoice::RejectNew
				}
			} else {
				ScoringChoice::InsertNew
			};

			decision
		}

		fn update_scores(&self, txs: &[SharedTransaction], scores: &mut [Self::Score], _change: ScoringChange) {
			for i in 0..txs.len() {
				scores[i] = txs[i].gas_price.0;
			}
		}

		fn should_replace(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> bool {
			new.gas_price.0 > old.gas_price.0
		}
	}

	#[derive(Default)]
	struct NonceReadiness(HashMap<Sender, U256>);
	impl Readiness for NonceReadiness {
		fn is_ready(&mut self, tx: &VerifiedTransaction) -> bool {
			let nonce = self.0.entry(tx.sender()).or_insert_with(|| U256::from(0));
			if tx.nonce == *nonce {
				*nonce = U256::from(nonce.0 + 1);
				true
			} else {
				false
			}
		}
	}

	type TestPool = Pool<DummyScoring>;

	#[derive(Debug, Default, Clone)]
	struct TransactionBuilder {
		nonce: U256,
		gas_price: U256,
		gas: U256,
		sender: Sender,
		insertion_id: Rc<Cell<u64>>,
	}

	impl TransactionBuilder {
		pub fn tx(&self) -> Self {
			self.clone()
		}

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
			let insertion_id = {
				let id = self.insertion_id.get() + 1;
				self.insertion_id.set(id);
				id
			};
			VerifiedTransaction {
				hash: (self.nonce.0 ^ (100 * self.gas_price.0) ^ (100_000 * self.sender.0)).into(),
				nonce: self.nonce,
				gas_price: self.gas_price,
				gas: 21_000.into(),
				sender: self.sender,
				insertion_id,
			}
		}
	}

	#[test]
	fn should_clear_queue() {
		// given
		let b = TransactionBuilder::default();
		let mut txq = TestPool::default();
		assert_eq!(txq.status(), Status {
			mem_usage: 0,
			count: 0,
		});
		let tx1 = b.tx().nonce(0).new();
		let tx2 = b.tx().nonce(1).new();

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
		let b = TransactionBuilder::default();
		let mut txq = TestPool::default();
		let tx1 = b.tx().nonce(0).new();
		let tx2 = b.tx().nonce(0).new();

		// when
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap_err();

		// then
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_replace_transaction() {
		// given
		let b = TransactionBuilder::default();
		let mut txq = TestPool::default();
		let tx1 = b.tx().nonce(0).gas_price(1).new();
		let tx2 = b.tx().nonce(0).gas_price(2).new();

		// when
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap();

		// then
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_reject_if_above_count() {
		let b = TransactionBuilder::default();
		let mut txq = TestPool::with_options(Options {
			max_count: 1,
			..Default::default()
		});

		// Reject second
		let tx1 = b.tx().nonce(0).new();
		let tx2 = b.tx().nonce(1).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);

		txq.clear();

		// Replace first
		let tx1 = b.tx().nonce(0).new();
		let tx2 = b.tx().nonce(0).sender(1).gas_price(2).new();
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap();
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_reject_if_above_mem_usage() {
		let b = TransactionBuilder::default();
		let mut txq = TestPool::with_options(Options {
			max_mem_usage: 1,
			..Default::default()
		});

		// Reject second
		let tx1 = b.tx().nonce(1).new();
		let tx2 = b.tx().nonce(2).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);

		txq.clear();

		// Replace first
		let tx1 = b.tx().nonce(1).new();
		let tx2 = b.tx().nonce(1).sender(1).gas_price(2).new();
		txq.import(tx1).unwrap();
		txq.import(tx2).unwrap();
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_reject_if_above_sender_count() {
		let b = TransactionBuilder::default();
		let mut txq = TestPool::with_options(Options {
			max_per_sender: 1,
			..Default::default()
		});

		// Reject second
		let tx1 = b.tx().nonce(1).new();
		let tx2 = b.tx().nonce(2).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);

		txq.clear();

		// Replace first
		let tx1 = b.tx().nonce(1).new();
		let tx2 = b.tx().nonce(2).gas_price(2).new();
		let hash = tx2.hash();
		txq.import(tx1).unwrap();
		// This results in error because we also compare nonces
		assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash));
		assert_eq!(txq.status().count, 1);
	}

	#[test]
	fn should_construct_pending() {
		// given
		let b = TransactionBuilder::default();
		let mut txq = TestPool::default();

		let tx0 = txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
		let tx1 = txq.import(b.tx().nonce(1).gas_price(5).new()).unwrap();
		let tx2 = txq.import(b.tx().nonce(2).new()).unwrap();
		// this transaction doesn't get to the block despite high gas price
		// because of block gas limit and simplistic ordering algorithm.
		txq.import(b.tx().nonce(3).gas_price(4).new()).unwrap();
		//gap
		txq.import(b.tx().nonce(5).new()).unwrap();

		let tx5 = txq.import(b.tx().sender(1).nonce(0).new()).unwrap();
		let tx6 = txq.import(b.tx().sender(1).nonce(1).new()).unwrap();
		let tx7 = txq.import(b.tx().sender(1).nonce(2).new()).unwrap();
		let tx8 = txq.import(b.tx().sender(1).nonce(3).gas_price(4).new()).unwrap();
		// gap
		txq.import(b.tx().sender(1).nonce(5).new()).unwrap();

		let tx9 = txq.import(b.tx().sender(2).nonce(0).new()).unwrap();
		assert_eq!(txq.status().count, 11);

		// when
		let mut current_gas = 0;
		let mut pending = txq.pending(NonceReadiness::default()).take_while(|tx| {
			let should_take = tx.gas.0 + current_gas <= 21_000 * 8;
			if should_take {
				current_gas += tx.gas.0
			}
			should_take
		});

		assert_eq!(pending.next(), Some(tx0));
		assert_eq!(pending.next(), Some(tx1));
		assert_eq!(pending.next(), Some(tx9));
		assert_eq!(pending.next(), Some(tx5));
		assert_eq!(pending.next(), Some(tx6));
		assert_eq!(pending.next(), Some(tx7));
		assert_eq!(pending.next(), Some(tx8));
		assert_eq!(pending.next(), Some(tx2));
		assert_eq!(pending.next(), None);
	}

	#[test]
	fn should_remove_transaction() {
		// given
		let b = TransactionBuilder::default();
		let mut txq = TestPool::default();

		let tx1 = txq.import(b.tx().nonce(0).new()).unwrap();
		let tx2 = txq.import(b.tx().nonce(1).new()).unwrap();
		txq.import(b.tx().nonce(2).new()).unwrap();
		assert_eq!(txq.status().count, 3);

		// when
		assert!(txq.remove(&tx2.hash(), false));

		// then
		assert_eq!(txq.status().count, 2);
		let mut pending = txq.pending(NonceReadiness::default());
		assert_eq!(pending.next(), Some(tx1));
		assert_eq!(pending.next(), None);
	}

	#[test]
	fn should_notify_listener() {
		// all 3 cases
		assert_eq!(false, true)
	}
}
