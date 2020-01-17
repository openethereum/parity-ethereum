// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Ethereum Transaction Queue

use std::{cmp, fmt};
use std::sync::Arc;
use std::sync::atomic::{self, AtomicUsize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use ethereum_types::{H256, U256, Address};
use futures::sync::mpsc;
use parking_lot::RwLock;
use txpool::{self, Verifier};
use types::transaction;

use pool::{
	self, replace, scoring, verifier, client, ready, listener,
	PrioritizationStrategy, PendingOrdering, PendingSettings, TxStatus
};
use pool::local_transactions::LocalTransactionsList;

type Listener = (LocalTransactionsList, (listener::TransactionsPoolNotifier, listener::Logger));
type Pool = txpool::Pool<pool::VerifiedTransaction, scoring::NonceAndGasPrice, Listener>;

/// Max cache time in milliseconds for pending transactions.
///
/// Pending transactions are cached and will only be computed again
/// if last cache has been created earler than `TIMESTAMP_CACHE` ms ago.
/// This timeout applies only if there are local pending transactions
/// since it only affects transaction Condition.
const TIMESTAMP_CACHE: u64 = 1000;

/// How many senders at once do we attempt to process while culling.
///
/// When running with huge transaction pools, culling can take significant amount of time.
/// To prevent holding `write()` lock on the pool for this long period, we split the work into
/// chunks and allow other threads to utilize the pool in the meantime.
/// This parameter controls how many (best) senders at once will be processed.
const CULL_SENDERS_CHUNK: usize = 1024;

/// Transaction queue status.
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
	/// Verifier options.
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
			gp = self.options.minimal_gas_price / 1_000_000,
			max_gas = cmp::min(self.options.block_gas_limit, self.options.tx_gas_limit),
		)
	}
}

#[derive(Debug)]
struct CachedPending {
	block_number: u64,
	current_timestamp: u64,
	nonce_cap: Option<U256>,
	has_local_pending: bool,
	pending: Option<Vec<Arc<pool::VerifiedTransaction>>>,
	max_len: usize,
}

impl CachedPending {
	/// Creates new `CachedPending` without cached set.
	pub fn none() -> Self {
		CachedPending {
			block_number: 0,
			current_timestamp: 0,
			has_local_pending: false,
			pending: None,
			nonce_cap: None,
			max_len: 0,
		}
	}

	/// Remove cached pending set.
	pub fn clear(&mut self) {
		self.pending = None;
	}

	/// Returns cached pending set (if any) if it's valid.
	pub fn pending(
		&self,
		block_number: u64,
		current_timestamp: u64,
		nonce_cap: Option<&U256>,
		max_len: usize,
	) -> Option<Vec<Arc<pool::VerifiedTransaction>>> {
		// First check if we have anything in cache.
		let pending = self.pending.as_ref()?;

		if block_number != self.block_number {
			return None;
		}

		// In case we don't have any local pending transactions
		// there is no need to invalidate the cache because of timestamp.
		// Timestamp only affects local `PendingTransactions` with `Condition::Timestamp`.
		if self.has_local_pending && current_timestamp > self.current_timestamp + TIMESTAMP_CACHE {
			return None;
		}

		// It's fine to return limited set even if `nonce_cap` is `None`.
		// The worst thing that may happen is that some transactions won't get propagated in current round,
		// but they are not really valid in current block anyway. We will propagate them in the next round.
		// Also there is no way to have both `Some` with different numbers since it depends on the block number
		// and a constant parameter in schedule (`nonce_cap_increment`)
		if self.nonce_cap.is_none() && nonce_cap.is_some() {
			return None;
		}

		// It's fine to just take a smaller subset, but not other way around.
		if max_len > self.max_len {
			return None;
		}

		Some(pending.iter().take(max_len).cloned().collect())
	}
}

#[derive(Debug)]
struct RecentlyRejected {
	inner: RwLock<HashMap<H256, transaction::Error>>,
	limit: usize,
}

impl RecentlyRejected {
	fn new(limit: usize) -> Self {
		RecentlyRejected {
			limit,
			inner: RwLock::new(HashMap::with_capacity(MIN_REJECTED_CACHE_SIZE)),
		}
	}

	fn clear(&self) {
		self.inner.write().clear();
	}

	fn get(&self, hash: &H256) -> Option<transaction::Error> {
		self.inner.read().get(hash).cloned()
	}

	fn insert(&self, hash: H256, err: &transaction::Error) {
		if self.inner.read().contains_key(&hash) {
			return;
		}

		let mut inner = self.inner.write();
		inner.insert(hash, err.clone());

		// clean up
		if inner.len() > self.limit {
			// randomly remove half of the entries
			let to_remove: Vec<_> = inner.keys().take(self.limit / 2).cloned().collect();
			for key in to_remove {
				inner.remove(&key);
			}
		}
	}
}

/// Minimal size of rejection cache, by default it's equal to queue size.
const MIN_REJECTED_CACHE_SIZE: usize = 2048;

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
	cached_pending: RwLock<CachedPending>,
	recently_rejected: RecentlyRejected,
}

impl TransactionQueue {
	/// Create new queue with given pool limits and initial verification options.
	pub fn new(
		limits: txpool::Options,
		verification_options: verifier::Options,
		strategy: PrioritizationStrategy,
	) -> Self {
		let max_count = limits.max_count;
		TransactionQueue {
			insertion_id: Default::default(),
			pool: RwLock::new(txpool::Pool::new(Default::default(), scoring::NonceAndGasPrice(strategy), limits)),
			options: RwLock::new(verification_options),
			cached_pending: RwLock::new(CachedPending::none()),
			recently_rejected: RecentlyRejected::new(cmp::max(MIN_REJECTED_CACHE_SIZE, max_count / 4)),
		}
	}

	/// Update verification options
	///
	/// Some parameters of verification may vary in time (like block gas limit or minimal gas price).
	pub fn set_verifier_options(&self, options: verifier::Options) {
		*self.options.write() = options;
	}

	/// Sets the in-chain transaction checker for pool listener.
	pub fn set_in_chain_checker<F>(&self, f: F) where
		F: Fn(&H256) -> bool + Send + Sync + 'static
	{
		self.pool.write().listener_mut().0.set_in_chain_checker(f)
	}

	/// Import a set of transactions to the pool.
	///
	/// Given blockchain and state access (Client)
	/// verifies and imports transactions to the pool.
	pub fn import<T: IntoIterator<Item = verifier::Transaction>, C: client::Client + client::NonceClient + Clone>(
		&self,
		client: C,
		transactions: T,
	) -> Vec<Result<(), transaction::Error>> {
		// Run verification
		trace_time!("pool::verify_and_import");
		let options = self.options.read().clone();

		let transaction_to_replace = {
			if options.no_early_reject {
				None
			} else {
				let pool = self.pool.read();
				if pool.is_full() {
					pool.worst_transaction().map(|worst| (pool.scoring().clone(), worst))
				} else {
					None
				}
			}
		};

		let verifier = verifier::Verifier::new(
			client.clone(),
			options,
			self.insertion_id.clone(),
			transaction_to_replace,
		);

		let mut replace = replace::ReplaceByScoreAndReadiness::new(self.pool.read().scoring().clone(), client);

		let results = transactions
			.into_iter()
			.map(|transaction| {
				let hash = transaction.hash();

				if self.pool.read().find(&hash).is_some() {
					return Err(transaction::Error::AlreadyImported);
				}

				if let Some(err) = self.recently_rejected.get(&hash) {
					trace!(target: "txqueue", "[{:?}] Rejecting recently rejected: {:?}", hash, err);
					return Err(err);
				}

				let imported = verifier
					.verify_transaction(transaction)
					.and_then(|verified| {
						self.pool.write().import(verified, &mut replace).map_err(convert_error)
					});

				match imported {
					Ok(_) => Ok(()),
					Err(err) => {
						self.recently_rejected.insert(hash, &err);
						Err(err)
					},
				}
			})
			.collect::<Vec<_>>();

		// Notify about imported transactions.
		(self.pool.write().listener_mut().1).0.notify();

		if results.iter().any(|r| r.is_ok()) {
			self.cached_pending.write().clear();
		}

		results
	}

	/// Returns all transactions in the queue without explicit ordering.
	pub fn all_transactions(&self) -> Vec<Arc<pool::VerifiedTransaction>> {
		let ready = |_tx: &pool::VerifiedTransaction| txpool::Readiness::Ready;
		self.pool.read().unordered_pending(ready).collect()
	}

	/// Returns all transaction hashes in the queue without explicit ordering.
	pub fn all_transaction_hashes(&self) -> Vec<H256> {
		let ready = |_tx: &pool::VerifiedTransaction| txpool::Readiness::Ready;
		self.pool.read().unordered_pending(ready).map(|tx| tx.hash).collect()
	}

	/// Computes unordered set of pending hashes.
	///
	/// Since strict nonce-checking is not required, you may get some false positive future transactions as well.
	pub fn pending_hashes<N>(
		&self,
		nonce: N,
	) -> BTreeSet<H256> where
		N: Fn(&Address) -> Option<U256>,
	{
		let ready = ready::OptionalState::new(nonce);
		self.pool.read().unordered_pending(ready).map(|tx| tx.hash).collect()
	}

	/// Returns current pending transactions ordered by priority.
	///
	/// NOTE: This may return a cached version of pending transaction set.
	/// Re-computing the pending set is possible with `#collect_pending` method,
	/// but be aware that it's a pretty expensive operation.
	pub fn pending<C>(
		&self,
		client: C,
		settings: PendingSettings,
	) -> Vec<Arc<pool::VerifiedTransaction>> where
		C: client::NonceClient,
	{
		let PendingSettings { block_number, current_timestamp, nonce_cap, max_len, ordering } = settings;
		if let Some(pending) = self.cached_pending.read().pending(block_number, current_timestamp, nonce_cap.as_ref(), max_len) {
			return pending;
		}

		// Double check after acquiring write lock
		let mut cached_pending = self.cached_pending.write();
		if let Some(pending) = cached_pending.pending(block_number, current_timestamp, nonce_cap.as_ref(), max_len) {
			return pending;
		}

		// In case we don't have a cached set, but we don't care about order
		// just return the unordered set.
		if let PendingOrdering::Unordered = ordering {
			let ready = Self::ready(client, block_number, current_timestamp, nonce_cap);
			return self.pool.read().unordered_pending(ready).take(max_len).collect();
		}

		let pending: Vec<_> = self.collect_pending(client, block_number, current_timestamp, nonce_cap, |i| {
			i.take(max_len).collect()
		});

		*cached_pending = CachedPending {
			block_number,
			current_timestamp,
			nonce_cap,
			has_local_pending: self.has_local_pending_transactions(),
			pending: Some(pending.clone()),
			max_len,
		};

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
		nonce_cap: Option<U256>,
		collect: F,
	) -> T where
		C: client::NonceClient,
		F: FnOnce(txpool::PendingIterator<
			pool::VerifiedTransaction,
			(ready::Condition, ready::State<C>),
			scoring::NonceAndGasPrice,
			Listener,
		>) -> T,
	{
		debug!(target: "txqueue", "Re-computing pending set for block: {}", block_number);
		trace_time!("pool::collect_pending");
		let ready = Self::ready(client, block_number, current_timestamp, nonce_cap);
		collect(self.pool.read().pending(ready))
	}

	fn ready<C>(
		client: C,
		block_number: u64,
		current_timestamp: u64,
		nonce_cap: Option<U256>,
	) -> (ready::Condition, ready::State<C>) where
		C: client::NonceClient,
	{
		let pending_readiness = ready::Condition::new(block_number, current_timestamp);
		// don't mark any transactions as stale at this point.
		let stale_id = None;
		let state_readiness = ready::State::new(client, stale_id, nonce_cap);

		(pending_readiness, state_readiness)
	}

	/// Culls all stalled transactions from the pool.
	pub fn cull<C: client::NonceClient + Clone>(
		&self,
		client: C,
	) {
		trace_time!("pool::cull");
		// We don't care about future transactions, so nonce_cap is not important.
		let nonce_cap = None;
		// We want to clear stale transactions from the queue as well.
		// (Transactions that are occuping the queue for a long time without being included)
		let stale_id = {
			let current_id = self.insertion_id.load(atomic::Ordering::Relaxed);
			// wait at least for half of the queue to be replaced
			let gap = self.pool.read().options().max_count / 2;
			// but never less than 100 transactions
			let gap = cmp::max(100, gap);

			current_id.checked_sub(gap)
		};

		self.recently_rejected.clear();

		let mut removed = 0;
		let senders: Vec<_> = {
			let pool = self.pool.read();
			let senders = pool.senders().cloned().collect();
			senders
		};
		for chunk in senders.chunks(CULL_SENDERS_CHUNK) {
			trace_time!("pool::cull::chunk");
			let state_readiness = ready::State::new(client.clone(), stale_id, nonce_cap);
			removed += self.pool.write().cull(Some(chunk), state_readiness);
		}
		debug!(target: "txqueue", "Removed {} stalled transactions. {}", removed, self.status());
	}

	/// Returns next valid nonce for given sender
	/// or `None` if there are no pending transactions from that sender.
	pub fn next_nonce<C: client::NonceClient>(
		&self,
		client: C,
		address: &Address,
	) -> Option<U256> {
		// Do not take nonce_cap into account when determining next nonce.
		let nonce_cap = None;
		// Also we ignore stale transactions in the queue.
		let stale_id = None;

		let state_readiness = ready::State::new(client, stale_id, nonce_cap);

		self.pool.read().pending_from_sender(state_readiness, address)
			.last()
			.map(|tx| tx.signed().nonce.saturating_add(U256::from(1)))
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
	pub fn remove<'a, T: IntoIterator<Item=&'a H256>>(
		&self,
		hashes: T,
		is_invalid: bool,
	) -> Vec<Option<Arc<pool::VerifiedTransaction>>> {
		let results = {
			let mut pool = self.pool.write();

			hashes
				.into_iter()
				.map(|hash| pool.remove(hash, is_invalid))
				.collect::<Vec<_>>()
		};

		if results.iter().any(Option::is_some) {
			self.cached_pending.write().clear();
		}

		results
	}

	/// Clear the entire pool.
	pub fn clear(&self) {
		self.pool.write().clear();
	}

	/// Penalize given senders.
	pub fn penalize<'a, T: IntoIterator<Item = &'a Address>>(&self, senders: T) {
		let mut pool = self.pool.write();
		for sender in senders {
			pool.update_scores(sender, ());
		}
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

	/// Add a listener to be notified about all transactions the pool
	pub fn add_pending_listener(&self, f: mpsc::UnboundedSender<Arc<Vec<H256>>>) {
		let mut pool = self.pool.write();
		(pool.listener_mut().1).0.add_pending_listener(f);
	}

	/// Add a listener to be notified about all transactions the pool
	pub fn add_full_listener(&self, f: mpsc::UnboundedSender<Arc<Vec<(H256, TxStatus)>>>) {
		let mut pool = self.pool.write();
		(pool.listener_mut().1).0.add_full_listener(f);
	}

	/// Check if pending set is cached.
	#[cfg(test)]
	pub fn is_pending_cached(&self) -> bool {
		self.cached_pending.read().pending.is_some()
	}
}

fn convert_error<H: fmt::Debug + fmt::LowerHex>(err: txpool::Error<H>) -> transaction::Error {
	use self::txpool::Error;

	match err {
		Error::AlreadyImported(..) => transaction::Error::AlreadyImported,
		Error::TooCheapToEnter(..) => transaction::Error::LimitReached,
		Error::TooCheapToReplace(..) => transaction::Error::TooCheapToReplace { prev: None, new: None }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use pool::tests::client::TestClient;

	#[test]
	fn should_get_pending_transactions() {
		let queue = TransactionQueue::new(txpool::Options::default(), verifier::Options::default(), PrioritizationStrategy::GasPriceOnly);

		let pending: Vec<_> = queue.pending(TestClient::default(), PendingSettings::all_prioritized(0, 0));

		for tx in pending {
			assert!(tx.signed().nonce > 0.into());
		}
	}
}
