
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use ethereum_types::H256;
use parking_lot::{RwLock, RwLockReadGuard};
use transaction;
use txpool::{self, Verifier};

use pool::{self, scoring, verifier, client, ready};

type Pool = txpool::Pool<pool::VerifiedTransaction, scoring::GasPrice>;

#[derive(Debug)]
pub struct TransactionQueue {
	insertion_id: Arc<AtomicUsize>,
	pool: RwLock<Pool>,
	options: RwLock<verifier::Options>,
}

impl TransactionQueue {
	pub fn new(limits: txpool::Options, verification_options: verifier::Options) -> Self {
		TransactionQueue {
			insertion_id: Default::default(),
			pool: RwLock::new(txpool::Pool::with_scoring(scoring::GasPrice, limits)),
			options: RwLock::new(verification_options),
		}
	}

	pub fn set_verifier_options(&self, options: verifier::Options) {
		*self.options.write() = options;
	}

	pub fn import<C: client::Client>(
		&self,
		client: C,
		transactions: Vec<verifier::Transaction>,
	) -> Vec<Result<(), transaction::Error>> {
		// Run verification
		let options = self.options.read().clone();

		// TODO [ToDr] parallelize
		let verifier = verifier::Verifier::new(client, options, self.insertion_id.clone());
		transactions
			.into_iter()
			.map(|transaction| verifier.verify_transaction(transaction))
			.map(|result| match result {
				Ok(verified) => match self.pool.write().import(verified) {
					Ok(imported) => Ok(()),
					Err(txpool::Error(kind, _)) => unimplemented!(),
				},
				Err(err) => Err(err),
			})
			.collect()
	}

	pub fn pending<C: client::Client>(
		&self,
		client: C,
		block_number: u64,
		current_timestamp: u64,
		// TODO [ToDr] Support nonce_cap
	) -> PendingReader<(ready::Condition, ready::State<C>)> {
		let pending_readiness = ready::Condition::new(block_number, current_timestamp);
		let state_readiness = ready::State::new(client);

		PendingReader {
			guard: self.pool.read(),
			ready: Some((pending_readiness, state_readiness)),
		}
	}

	pub fn cull<C: client::Client>(
		&self,
		client: C,
	) {
		let state_readiness = ready::State::new(client);
		let removed = self.pool.write().cull(None, state_readiness);
		debug!(target: "txqueue", "Removed {} stalled transactions.", removed);
	}

	pub fn find(
		&self,
		hash: &H256,
	) -> Option<Arc<pool::VerifiedTransaction>> {
		self.pool.read().find(hash)
	}

	pub fn remove<'a, T: IntoIterator<Item = &'a H256>>(
		&self,
		hashes: T,
		is_invalid: bool,
	) {
		let mut pool = self.pool.write();
		for hash in hashes {
			pool.remove(hash, is_invalid);
		}
	}
}

pub struct PendingReader<'a, R> {
	guard: RwLockReadGuard<'a, Pool>,
	ready: Option<R>,
}

impl<'a, R: txpool::Ready<pool::VerifiedTransaction>> PendingReader<'a, R> {
	pub fn transactions(&'a mut self) -> txpool::PendingIterator<pool::VerifiedTransaction, R, scoring::GasPrice, txpool::NoopListener> {
		self.guard.pending(self.ready.take().unwrap())
	}
}
