extern crate smallvec;
extern crate ethcore_bigint as bigint;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

mod error;
mod pool;

pub use self::pool::Pool;

use std::sync::Arc;
use std::{cmp, fmt};

use self::bigint::prelude::{H256, U256};
type Address = bigint::hash::H160;

// Types
#[derive(Debug)]
pub struct UnverifiedTransaction;
#[derive(Debug, PartialEq)]
pub struct VerifiedTransaction {
	pub hash: H256,
	pub nonce: U256,
	pub gas_price: U256,
	pub gas: U256,
	pub sender: Address,
	pub insertion_id: u64,
}
impl VerifiedTransaction {
	pub fn hash(&self) -> H256 {
		self.hash.clone()
	}

	pub fn mem_usage(&self) -> usize {
		self.nonce.low_u64() as usize
	}

	pub fn sender(&self) -> Address {
		self.sender.clone()
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

pub trait Listener {
	fn added(&mut self, _tx: &SharedTransaction, _old: Option<&SharedTransaction>) {}
	fn rejected(&mut self, _tx: VerifiedTransaction) {}
	fn dropped(&mut self, _tx: &SharedTransaction) {}
	fn invalid(&mut self, _tx: &SharedTransaction) {}
	fn cancelled(&mut self, _tx: &SharedTransaction) {}
	fn mined(&mut self, _tx: &SharedTransaction) {}
}

pub struct NoopListener;
impl Listener for NoopListener {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringChoice {
	RejectNew,
	ReplaceOld,
	InsertNew,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
	/// Given number of stalled transactions has been culled from the beginning.
	/// Usually the score will have to be re-computed from scratch.
	Culled(usize),
}

pub trait Scoring {
	type Score: cmp::Ord + Clone + Default + fmt::Debug;

	fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering;

	fn choose(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> ScoringChoice;

	fn update_scores(&self, txs: &[SharedTransaction], scores: &mut [Self::Score], change: ScoringChange);

	fn should_replace(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
	Stalled,
	Ready,
	Future,
}

impl From<bool> for Readiness {
	fn from(b: bool) -> Self {
		if b { Readiness::Ready } else { Readiness::Future }
	}
}

pub trait Ready {
	/// Returns true if transaction is ready to be included in pending block,
	/// given all previous transactions that were ready are included.
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> Readiness;
}

impl<F> Ready for F where F: FnMut(&VerifiedTransaction) -> Readiness {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> Readiness {
		(*self)(tx)
	}
}
