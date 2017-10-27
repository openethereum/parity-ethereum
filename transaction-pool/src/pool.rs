use std::sync::Arc;
use std::collections::{HashMap, BTreeSet};

use smallvec::SmallVec;

use {Listener, NoopListener};
use {VerifiedTransaction, H256};

type Sender = ::Address;
type SharedTransaction = Arc<VerifiedTransaction>;

#[derive(Debug)]
pub struct Options {
	max_count: usize,
	max_per_sender: usize,
	memory_limit: usize,
}

impl Default for Options {
	fn default() -> Self {
		Options {
			max_count: 1024,
			max_per_sender: 16,
			memory_limit: 8 * 1024 * 1024,
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct Status {
	pub mem_usage: usize,
	pub count: usize,
}

#[derive(Debug)]
pub struct Pool<L> {
	listener: L,
	options: Options,
	mem_usage: usize,

	transactions: HashMap<Sender, Transactions>,
	by_hash: HashMap<H256, SharedTransaction>,

	best_transactions: BTreeSet<Score>,
	worst_transactions: BTreeSet<Score>,
}

impl Default for Pool<NoopListener> {
	fn default() -> Self {
		Self::new(Options::default())
	}
}

impl Pool<NoopListener> {
	pub fn new(options: Options) -> Self {
		Self::with_listener(NoopListener, options)
	}
}

impl<L: Listener> Pool<L> {
	pub fn with_listener(listener: L, options: Options) -> Self {
		let transactions = HashMap::with_capacity(16);
		let by_hash = HashMap::with_capacity(options.max_count / 16);

		Pool {
			listener,
			options,
			mem_usage: 0,
			transactions,
			by_hash,
			best_transactions: Default::default(),
			worst_transactions: Default::default(),
		}

	}
	pub fn import(&mut self, transaction: VerifiedTransaction) -> Result<(), ()> {
		let sender = transaction.sender();
		let mem_usage = transaction.mem_usage();
		let hash = transaction.hash();

		if self.by_hash.contains_key(&hash) {
			return Err(())
		}

		while self.by_hash.len() >= self.options.max_count {
			self.remove_worst_transaction()
		}

		while self.mem_usage + mem_usage >= self.options.memory_limit {
			self.remove_worst_transaction()
		}

		let transactions = self.transactions.entry(sender).or_insert_with(Transactions::default);

		let result = transactions.add(transaction, self.options.max_per_sender);

		match result {
			AddResult::Ok(ref tx) => {
				self.listener.added(tx, None);

				self.mem_usage += mem_usage;
				self.by_hash.insert(hash, tx.clone());
			},
			AddResult::Rejected(ref tx) => {
				self.listener.rejected(tx);
			},
			AddResult::Replaced { ref new, ref old } => {
				self.listener.added(new, Some(old));

				self.mem_usage += mem_usage;
				self.by_hash.insert(hash, new.clone());

				self.mem_usage -= old.mem_usage();
				self.by_hash.remove(&old.hash());
			},
		}

		Ok(())
	}

	fn remove_worst_transaction(&mut self) {

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

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Score;

#[derive(Debug)]
enum AddResult {
	Ok(SharedTransaction),
	Rejected(VerifiedTransaction),
	Replaced {
		old: SharedTransaction,
		new: SharedTransaction,
	}
}

/// Represents all transactions from a particular sender ordered by nonce.
const PER_SENDER: usize = 8;
#[derive(Default, Debug)]
struct Transactions {
	transactions: SmallVec<[SharedTransaction; PER_SENDER]>,
	best: Option<Score>,
	worst: Option<Score>,
}

impl Transactions {
	pub fn add(&mut self, tx: VerifiedTransaction, max_count: usize) -> AddResult {
		AddResult::Ok(Arc::new(tx))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use U256;

	#[derive(Debug, Default)]
	struct TransactionBuilder {
		nonce: U256
	}

	impl TransactionBuilder {
		pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
			self.nonce = nonce.into();
			self
		}

		pub fn new(self) -> VerifiedTransaction {
			VerifiedTransaction {
				hash: self.nonce.0.into()
			}
		}
	}

	fn tx() -> TransactionBuilder {
		TransactionBuilder::default()
	}

	#[test]
	fn should_clear_queue() {
		// given
		let mut txq = Pool::default();
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
		let mut txq = Pool::default();
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
		assert_eq!(false, true)
	}

	#[test]
	fn should_reject_if_above_count() {
		assert_eq!(false, true)
	}

	#[test]
	fn should_reject_if_above_mem_usage() {
		assert_eq!(false, true)
	}

	#[test]
	fn should_reject_if_above_sender_count() {
		assert_eq!(false, true)
	}

	#[test]
	fn should_notify_listener() {
		// all 3 cases
		assert_eq!(false, true)
	}
}
