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

//! Ethereum Transaction Queue

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::collections::BTreeMap;

use ethereum_types::{H256, U256, Address};
use parking_lot::RwLock;
use transaction;
use txpool::{self, Verifier};

use pool::{self, scoring, verifier, client, ready, listener};
use pool::local_transactions::LocalTransactionsList;

type Listener = (LocalTransactionsList, (listener::Notifier, listener::Logger));
type Pool = txpool::Pool<pool::VerifiedTransaction, scoring::GasPrice, Listener>;

/// Transaction queue status
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
	/// Verifier options
	pub options: verifier::Options,
	/// Current status of the transaction pool.
	pub status: txpool::LightStatus,
	/// Current limits of the transaction pool.
	pub limits: txpool::Options,
}

impl fmt::Display for Status {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		writeln!(
			fmt,
			"Pool: {current}/{max} ({senders} senders; {mem}/{mem_max} kB) [minGasPrice: {gp} Mwei, maxGas: {max_gas}]",
			current = self.status.transaction_count,
			max = self.limits.max_count,
			senders = self.status.senders,
			mem = self.status.mem_usage / 1024,
			mem_max = self.limits.max_mem_usage / 1024,
			gp = self.options.minimal_gas_price / 1_000_000.into(),
			max_gas = ::std::cmp::min(self.options.block_gas_limit, self.options.tx_gas_limit),
		)
	}
}

/// Ethereum Transaction Queue
///
/// Responsible for:
/// - verifying incoming transactions
/// - maintaining a pool of verified transactions.
/// - returning an iterator for transactions that are ready to be included in block (pending)
#[derive(Debug)]
pub struct TransactionQueue {
	insertion_id: Arc<AtomicUsize>,
	pool: RwLock<Pool>,
	options: RwLock<verifier::Options>,
	cached_pending: RwLock<Option<(u64, Vec<Arc<pool::VerifiedTransaction>>)>>,
}

impl TransactionQueue {
	/// Create new queue with given pool limits and initial verification options.
	pub fn new(limits: txpool::Options, verification_options: verifier::Options) -> Self {
		TransactionQueue {
			insertion_id: Default::default(),
			pool: RwLock::new(txpool::Pool::new(Default::default(), scoring::GasPrice, limits)),
			options: RwLock::new(verification_options),
			cached_pending: RwLock::new(None),
		}
	}

	/// Update verification options
	///
	/// Some parameters of verification may vary in time (like block gas limit or minimal gas price).
	pub fn set_verifier_options(&self, options: verifier::Options) {
		*self.options.write() = options;
	}

	/// Import a set of transactions to the pool.
	///
	/// Given blockchain and state access (Client)
	/// verifies and imports transactions to the pool.
	pub fn import<C: client::Client>(
		&self,
		client: C,
		transactions: Vec<verifier::Transaction>,
	) -> Vec<Result<(), transaction::Error>> {
		// Run verification
		let _timer = ::trace_time::PerfTimer::new("queue::verifyAndImport");
		let options = self.options.read().clone();

		let verifier = verifier::Verifier::new(client, options, self.insertion_id.clone());
		let results = transactions
			.into_iter()
			.map(|transaction| verifier.verify_transaction(transaction))
			.map(|result| match result {
				Ok(verified) => match self.pool.write().import(verified) {
					Ok(_imported) => Ok(()),
					Err(err) => Err(convert_error(err)),
				},
				Err(err) => Err(err),
			})
			.collect::<Vec<_>>();

		if results.iter().any(|r| r.is_ok()) {
			*self.cached_pending.write() = None;
		}

		results
	}

	pub fn pending<C>(
		&self,
		client: C,
		block_number: u64,
		current_timestamp: u64,
	) -> Vec<Arc<pool::VerifiedTransaction>> where
		C: client::Client,
	{
		// TODO [ToDr] Check if timestamp is within limits.
		let is_valid = |bn| bn == block_number;
		{
			let cached_pending = self.cached_pending.read();
			match *cached_pending {
				Some((bn, ref pending)) if is_valid(bn) => {
					return pending.clone()
				},
				_ => {},
			}
		}

		let mut cached_pending = self.cached_pending.write();
		match *cached_pending {
			Some((bn, ref pending)) if is_valid(bn) => {
				return pending.clone()
			},
			_ => {},
		}

		let pending: Vec<_> = self.collect_pending(client, block_number, current_timestamp, |i| i.collect());
		*cached_pending = Some((block_number, pending.clone()));
		pending
	}

	/// Collect pending transactions.
	///
	/// NOTE This is re-computing the pending set and it might be expensive to do so.
	/// Prefer using cached pending set using `#pending` method.
	pub fn collect_pending<C, F, T>(
		&self,
		client: C,
		block_number: u64,
		current_timestamp: u64,
		// TODO [ToDr] Support nonce_cap
		collect: F,
	) -> T where
		C: client::Client,
		F: FnOnce(txpool::PendingIterator<
			pool::VerifiedTransaction,
			(ready::Condition, ready::State<C>),
			scoring::GasPrice,
			Listener,
		>) -> T,
	{
		let pending_readiness = ready::Condition::new(block_number, current_timestamp);
		let state_readiness = ready::State::new(client);

		let ready = (pending_readiness, state_readiness);

		collect(self.pool.read().pending(ready))
	}

	/// Culls all stalled transactions from the pool.
	pub fn cull<C: client::Client>(
		&self,
		client: C,
	) {
		let state_readiness = ready::State::new(client);

		let removed = self.pool.write().cull(None, state_readiness);
		debug!(target: "txqueue", "Removed {} stalled transactions. {}", removed, self.status());
	}

	/// Returns next valid nonce for given sender
	/// or `None` if there are no pending transactions from that sender.
	pub fn next_nonce<C: client::Client>(
		&self,
		client: C,
		address: &Address,
	) -> Option<U256> {
		let state_readiness = ready::State::new(client);

		self.pool.read().pending_from_sender(state_readiness, address)
			.last()
			.map(|tx| tx.signed().nonce + 1.into())
	}

	/// Retrieve a transaction from the pool.
	///
	/// Given transaction hash looks up that transaction in the pool
	/// and returns a shared pointer to it or `None` if it's not present.
	pub fn find(
		&self,
		hash: &H256,
	) -> Option<Arc<pool::VerifiedTransaction>> {
		self.pool.read().find(hash)
	}

	/// Remove a set of transactions from the pool.
	///
	/// Given an iterator of transaction hashes
	/// removes them from the pool.
	/// That method should be used if invalid transactions are detected
	/// or you want to cancel a transaction.
	pub fn remove<'a, T: IntoIterator<Item = &'a H256>>(
		&self,
		hashes: T,
		is_invalid: bool,
	) -> Vec<Option<Arc<pool::VerifiedTransaction>>> {
		let mut pool = self.pool.write();

		hashes
			.into_iter()
			.map(|hash| pool.remove(hash, is_invalid))
			.collect()
	}

	/// Clear the entire pool.
	pub fn clear(&self) {
		self.pool.write().clear();
	}

	/// Returns gas price of currently the worst transaction in the pool.
	pub fn current_worst_gas_price(&self) -> U256 {
		match self.pool.read().worst_transaction() {
			Some(tx) => tx.signed().gas_price,
			None => self.options.read().minimal_gas_price,
		}
	}

	/// Returns a status of the queue.
	pub fn status(&self) -> Status {
		let pool = self.pool.read();
		let status = pool.light_status();
		let limits = pool.options();
		let options = self.options.read().clone();

		Status {
			options,
			status,
			limits,
		}
	}

	/// Check if there are any local transactions in the pool.
	///
	/// Returns `true` if there are any transactions in the pool
	/// that has been marked as local.
	///
	/// Local transactions are the ones from accounts managed by this node
	/// and transactions submitted via local RPC (`eth_sendRawTransaction`)
	pub fn has_local_pending_transactions(&self) -> bool {
		self.pool.read().listener().0.has_pending()
	}

	/// Returns status of recently seen local transactions.
	pub fn local_transactions(&self) -> BTreeMap<H256, pool::local_transactions::Status> {
		self.pool.read().listener().0.all_transactions().iter().map(|(a, b)| (*a, b.clone())).collect()
	}

	/// Add a callback to be notified about all transactions entering the pool.
	pub fn add_listener(&self, f: Box<Fn(&[H256]) + Send + Sync>) {
		let mut pool = self.pool.write();
		(pool.listener_mut().1).0.add(f);
	}
}


fn convert_error(err: txpool::Error) -> transaction::Error {
	use self::txpool::ErrorKind;

	match *err.kind() {
		ErrorKind::AlreadyImported(..) => transaction::Error::AlreadyImported,
		ErrorKind::TooCheapToEnter(..) => transaction::Error::LimitReached,
		ErrorKind::TooCheapToReplace(..) => transaction::Error::TooCheapToReplace,
		ref e => {
			warn!(target: "txqueue", "Unknown import error: {:?}", e);
			transaction::Error::NotAllowed
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct TestClient;
	impl client::Client for TestClient {
		fn transaction_already_included(&self, _hash: &H256) -> bool {
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
		fn transaction_type(&self, _tx: &transaction::SignedTransaction) -> client::TransactionType {
			client::TransactionType::Regular
		}
	}

	#[test]
	fn should_get_pending_transactions() {
		let queue = TransactionQueue::new(txpool::Options::default(), verifier::Options::default());

		let pending: Vec<_> = queue.pending(TestClient, 0, 0, |x| x.collect());

		for tx in pending {
			assert!(tx.signed().nonce > 0.into());
		}
	}
}
