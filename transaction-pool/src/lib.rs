extern crate smallvec;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

mod error;
mod pool;

pub use self::pool::Pool;

use std::sync::Arc;
use std::cmp;

// Types
#[derive(Debug)]
pub struct UnverifiedTransaction;
#[derive(Debug)]
pub struct SignedTransaction;
#[derive(Debug, PartialEq)]
pub struct VerifiedTransaction {
	pub hash: H256,
	pub nonce: U256,
	pub gas_price: U256,
	pub sender: Address,
}
impl VerifiedTransaction {
	pub fn hash(&self) -> H256 {
		self.hash.clone()
	}

	pub fn mem_usage(&self) -> usize {
		self.nonce.0 as usize
	}

	pub fn sender(&self) -> Address {
		self.sender.clone()
	}
}
#[derive(Debug)]
pub struct PendingTransaction;
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u64);
impl From<u64> for Address {
	fn from(x: u64) -> Self {
		Address(x)
	}
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct U256(u64);
impl From<u64> for U256 {
	fn from(x: u64) -> Self {
		U256(x)
	}
}
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct H256(u64);
impl From<u64> for H256 {
	fn from(x: u64) -> Self {
		H256(x)
	}
}

pub type SharedTransaction = Arc<VerifiedTransaction>;

/// Main part of the transaction verification is decoupled from the pool
pub trait Verifier {
	type Error;

	fn verify_transaction(&self, tx: UnverifiedTransaction) -> Result<VerifiedTransaction, Self::Error>;
}

pub struct NoopVerifier;
impl Verifier for NoopVerifier {
	type Error = ();

	fn verify_transaction(&self, _tx: UnverifiedTransaction) -> Result<VerifiedTransaction, Self::Error> {
		unimplemented!()
	}
}

// TODO [ToDr] Should accept SharedTransaction instead.
pub trait Listener {
	fn added(&mut self, _tx: &VerifiedTransaction, _old: Option<&VerifiedTransaction>) {}
	fn rejected(&mut self, _tx: &VerifiedTransaction) {}
	fn dropped(&mut self, _tx: &VerifiedTransaction) {}
	fn invalid(&mut self, _tx: &SignedTransaction) {}
	fn cancelled(&mut self, _tx: &PendingTransaction) {}
}

pub struct NoopListener;
impl Listener for NoopListener {}

#[derive(Debug, Clone, Copy)]
pub enum ScoringDecision {
	Reject,
	Replace,
	Insert,
}

pub enum ScoringChange {
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
}

pub trait Scoring {
	type Score: cmp::Ord + Clone + Default;

	fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering;

	fn decide(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> ScoringDecision;

	fn update_scores(&self, txs: &[SharedTransaction], scores: &mut [Self::Score], change: ScoringChange);
}
