
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use ethereum_types::{H256, U256};
use parking_lot::{RwLock, RwLockReadGuard};
use transaction;
use txpool::{self, Verifier};

use pool::{self, scoring, verifier, client, ready};

// TODO [ToDr] Support logging listener (and custom listeners)
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

	pub fn clear(&self) {
		self.pool.write().clear();
	}

	pub fn current_worst_gas_price(&self) -> U256 {
		match self.pool.read().worst_transaction() {
			Some(tx) => tx.signed().gas_price,
			None => self.options.read().minimal_gas_price,
		}
	}

	pub fn status(&self) -> txpool::LightStatus {
		self.pool.read().light_status()
	}

	pub fn has_local_transactions(&self) -> bool {
		// TODO [ToDr] Take from the listener
		false
	}
}

pub struct PendingReader<'a, R> {
	guard: RwLockReadGuard<'a, Pool>,
	ready: Option<R>,
}

impl<'a, R: txpool::Ready<pool::VerifiedTransaction>> PendingReader<'a, R> {
	pub fn transactions(&mut self) -> txpool::PendingIterator<pool::VerifiedTransaction, R, scoring::GasPrice, txpool::NoopListener> {
		self.guard.pending(self.ready.take().unwrap())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use ethereum_types::Address;

	#[derive(Debug)]
	struct TestClient;
	impl client::Client for TestClient {
		fn transaction_already_included(&self, hash: &H256) -> bool {
			false
		}

		fn verify_transaction(&self, tx: transaction::UnverifiedTransaction) -> Result<transaction::SignedTransaction, transaction::Error> {
			Ok(transaction::SignedTransaction::new(tx)?)
		}

		/// Fetch account details for given sender.
		fn account_details(&self, _address: &Address) -> client::AccountDetails {
			client::AccountDetails {
				balance: 5_000_000.into(),
				nonce: 0.into(),
				is_local: false,
			}
		}

		/// Fetch only account nonce for given sender.
		fn account_nonce(&self, _address: &Address) -> U256 {
			0.into()
		}

		/// Estimate minimal gas requirurement for given transaction.
		fn required_gas(&self, _tx: &transaction::SignedTransaction) -> U256 {
			0.into()
		}

		/// Classify transaction (check if transaction is filtered by some contracts).
		fn transaction_type(&self, tx: &transaction::SignedTransaction) -> client::TransactionType {
			client::TransactionType::Regular
		}
	}

	#[test]
	fn should_get_pending_transactions() {
		let queue = TransactionQueue::new(txpool::Options::default(), verifier::Options::default());

		let mut pending = queue.pending(TestClient, 0, 0);

		for tx in pending.transactions() {
			assert!(tx.signed().nonce > 0.into());
		}
	}
}
