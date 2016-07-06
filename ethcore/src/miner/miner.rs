// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use rayon::prelude::*;
use std::sync::atomic::AtomicBool;
use std::time::{Instant, Duration};

use util::*;
use util::using_queue::{UsingQueue, GetAction};
use util::Colour::White;
use account_provider::AccountProvider;
use views::{BlockView, HeaderView};
use client::{MiningBlockChainClient, Executive, Executed, EnvInfo, TransactOptions, BlockID, CallAnalytics};
use block::{ClosedBlock, IsBlock};
use error::*;
use transaction::SignedTransaction;
use receipt::{Receipt};
use spec::Spec;
use engine::Engine;
use miner::{MinerService, MinerStatus, TransactionQueue, AccountDetails, TransactionOrigin};
use miner::work_notify::WorkPoster;
use client::TransactionImportResult;


/// Different possible definitions for pending transaction set.
#[derive(Debug)]
pub enum PendingSet {
	/// Always just the transactions in the queue. These have had only cheap checks.
	AlwaysQueue,
	/// Always just the transactions in the sealing block. These have had full checks but
	/// may be empty if the node is not actively mining or has force_sealing enabled.
	AlwaysSealing,
	/// Try the sealing block, but if it is not currently sealing, fallback to the queue.
	SealingOrElseQueue,
}

/// Configures the behaviour of the miner.
#[derive(Debug)]
pub struct MinerOptions {
	/// URLs to notify when there is new work.
	pub new_work_notify: Vec<String>,
	/// Force the miner to reseal, even when nobody has asked for work.
	pub force_sealing: bool,
	/// Reseal on receipt of new external transactions.
	pub reseal_on_external_tx: bool,
	/// Reseal on receipt of new local transactions.
	pub reseal_on_own_tx: bool,
	/// Minimum period between transaction-inspired reseals.
	pub reseal_min_period: Duration,
	/// Maximum amount of gas to bother considering for block insertion.
	pub tx_gas_limit: U256,
	/// Maximum size of the transaction queue.
	pub tx_queue_size: usize,
	/// Whether we should fallback to providing all the queue's transactions or just pending.
	pub pending_set: PendingSet,
	/// How many historical work packages can we store before running out?
	pub work_queue_size: usize,
	/// Can we submit two different solutions for the same block and expect both to result in an import?
	pub enable_resubmission: bool,
}

impl Default for MinerOptions {
	fn default() -> Self {
		MinerOptions {
			new_work_notify: vec![],
			force_sealing: false,
			reseal_on_external_tx: true,
			reseal_on_own_tx: true,
			tx_gas_limit: !U256::zero(),
			tx_queue_size: 1024,
			pending_set: PendingSet::AlwaysQueue,
			reseal_min_period: Duration::from_secs(0),
			work_queue_size: 20,
			enable_resubmission: true,
		}
	}
}

/// Keeps track of transactions using priority queue and holds currently mined block.
pub struct Miner {
	// NOTE [ToDr]  When locking always lock in this order!
	transaction_queue: Mutex<TransactionQueue>,
	sealing_work: Mutex<UsingQueue<ClosedBlock>>,

	// for sealing...
	options: MinerOptions,
	sealing_enabled: AtomicBool,
	next_allowed_reseal: Mutex<Instant>,
	sealing_block_last_request: Mutex<u64>,
	gas_range_target: RwLock<(U256, U256)>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,
	spec: Spec,

	accounts: Option<Arc<AccountProvider>>,
	work_poster: Option<WorkPoster>,
}

impl Miner {
	/// Creates new instance of miner without accounts, but with given spec.
	pub fn with_spec(spec: Spec) -> Miner {
		Miner {
			transaction_queue: Mutex::new(TransactionQueue::new()),
			options: Default::default(),
			sealing_enabled: AtomicBool::new(false),
			next_allowed_reseal: Mutex::new(Instant::now()),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(UsingQueue::new(20)),
			gas_range_target: RwLock::new((U256::zero(), U256::zero())),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			accounts: None,
			spec: spec,
			work_poster: None,
		}
	}

	/// Creates new instance of miner
	pub fn new(options: MinerOptions, spec: Spec, accounts: Option<Arc<AccountProvider>>) -> Arc<Miner> {
		let work_poster = if !options.new_work_notify.is_empty() { Some(WorkPoster::new(&options.new_work_notify)) } else { None };
		Arc::new(Miner {
			transaction_queue: Mutex::new(TransactionQueue::with_limits(options.tx_queue_size, options.tx_gas_limit)),
			sealing_enabled: AtomicBool::new(options.force_sealing || !options.new_work_notify.is_empty()),
			next_allowed_reseal: Mutex::new(Instant::now()),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(UsingQueue::new(options.work_queue_size)),
			gas_range_target: RwLock::new((U256::zero(), U256::zero())),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			options: options,
			accounts: accounts,
			spec: spec,
			work_poster: work_poster,
		})
	}

	fn engine(&self) -> &Engine {
		self.spec.engine.deref()
	}

	fn forced_sealing(&self) -> bool {
		self.options.force_sealing || !self.options.new_work_notify.is_empty()
	}

	/// Prepares new block for sealing including top transactions from queue.
	#[cfg_attr(feature="dev", allow(match_same_arms))]
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn prepare_sealing(&self, chain: &MiningBlockChainClient) {
		trace!(target: "miner", "prepare_sealing: entering");

		let (transactions, mut open_block, original_work_hash) = {
			let transactions = {self.transaction_queue.lock().unwrap().top_transactions()};
			let mut sealing_work = self.sealing_work.lock().unwrap();
			let last_work_hash = sealing_work.peek_last_ref().map(|pb| pb.block().fields().header.hash());
			let best_hash = chain.best_block_header().sha3();
/*
			// check to see if last ClosedBlock in would_seals is actually same parent block.
			// if so
			//   duplicate, re-open and push any new transactions.
			//   if at least one was pushed successfully, close and enqueue new ClosedBlock;
			//   otherwise, leave everything alone.
			// otherwise, author a fresh block.
*/
			let open_block = match sealing_work.pop_if(|b| b.block().fields().header.parent_hash() == &best_hash) {
				Some(old_block) => {
					trace!(target: "miner", "Already have previous work; updating and returning");
					// add transactions to old_block
					let e = self.engine();
					old_block.reopen(e, chain.vm_factory())
				}
				None => {
					// block not found - create it.
					trace!(target: "miner", "No existing work - making new block");
					chain.prepare_open_block(
						self.author(),
						(self.gas_floor_target(), self.gas_ceil_target()),
						self.extra_data()
					)
				}
			};
			(transactions, open_block, last_work_hash)
		};

		let mut invalid_transactions = HashSet::new();
		let block_number = open_block.block().fields().header.number();
		// TODO: push new uncles, too.
		for tx in transactions {
			let hash = tx.hash();
			match open_block.push_transaction(tx, None) {
				Err(Error::Execution(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, .. })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of gas limit: {:?}", hash);
					// Exit early if gas left is smaller then min_tx_gas
					let min_tx_gas: U256 = 21000.into();	// TODO: figure this out properly.
					if gas_limit - gas_used < min_tx_gas {
						break;
					}
				},
				// Invalid nonce error can happen only if previous transaction is skipped because of gas limit.
				// If there is errornous state of transaction queue it will be fixed when next block is imported.
				Err(Error::Execution(ExecutionError::InvalidNonce { .. })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of invalid nonce: {:?}", hash);
				},
				// already have transaction - ignore
				Err(Error::Transaction(TransactionError::AlreadyImported)) => {},
				Err(e) => {
					invalid_transactions.insert(hash);
					debug!(target: "miner",
						   "Error adding transaction to block: number={}. transaction_hash={:?}, Error: {:?}",
						   block_number, hash, e);
				},
				_ => {}	// imported ok
			}
		}

		let block = open_block.close();

		let fetch_account = |a: &Address| AccountDetails {
			nonce: chain.latest_nonce(a),
			balance: chain.latest_balance(a),
		};

		{
			let mut queue = self.transaction_queue.lock().unwrap();
			for hash in invalid_transactions.into_iter() {
				queue.remove_invalid(&hash, &fetch_account);
			}
		}

		if !block.transactions().is_empty() {
			trace!(target: "miner", "prepare_sealing: block has transaction - attempting internal seal.");
			// block with transactions - see if we can seal immediately.
			let s = self.engine().generate_seal(block.block(), match self.accounts {
				Some(ref x) => Some(&**x),
				None => None,
			});
			if let Some(seal) = s {
				trace!(target: "miner", "prepare_sealing: managed internal seal. importing...");
				if let Ok(sealed) = block.lock().try_seal(self.engine(), seal) {
					if let Ok(_) = chain.import_block(sealed.rlp_bytes()) {
						trace!(target: "miner", "prepare_sealing: sealed internally and imported. leaving.");
					} else {
						warn!("prepare_sealing: ERROR: could not import internally sealed block. WTF?");
					}
				} else {
					warn!("prepare_sealing: ERROR: try_seal failed when given internally generated seal. WTF?");
				}
				return;
			} else {
				trace!(target: "miner", "prepare_sealing: unable to generate seal internally");
			}
		}

		let (work, is_new) = {
			let mut sealing_work = self.sealing_work.lock().unwrap();
			let last_work_hash = sealing_work.peek_last_ref().map(|pb| pb.block().fields().header.hash());
			trace!(target: "miner", "Checking whether we need to reseal: orig={:?} last={:?}, this={:?}", original_work_hash, last_work_hash, block.block().fields().header.hash());
			let (work, is_new) = if last_work_hash.map_or(true, |h| h != block.block().fields().header.hash()) {
				trace!(target: "miner", "Pushing a new, refreshed or borrowed pending {}...", block.block().fields().header.hash());
				let pow_hash = block.block().fields().header.hash();
				let number = block.block().fields().header.number();
				let difficulty = *block.block().fields().header.difficulty();
				let is_new = original_work_hash.map_or(true, |h| block.block().fields().header.hash() != h);
				sealing_work.push(block);
				// If push notifications are enabled we assume all work items are used.
				if self.work_poster.is_some() && is_new {
					sealing_work.use_last_ref();
				}
				(Some((pow_hash, difficulty, number)), is_new)
			} else {
				(None, false)
			};
			trace!(target: "miner", "prepare_sealing: leaving (last={:?})", sealing_work.peek_last_ref().map(|b| b.block().fields().header.hash()));
			(work, is_new)
		};
		if is_new {
			work.map(|(pow_hash, difficulty, number)| self.work_poster.as_ref().map(|ref p| p.notify(pow_hash, difficulty, number)));
		}
	}

	fn update_gas_limit(&self, chain: &MiningBlockChainClient) {
		let gas_limit = HeaderView::new(&chain.best_block_header()).gas_limit();
		let mut queue = self.transaction_queue.lock().unwrap();
		queue.set_gas_limit(gas_limit);
	}

	/// Returns true if we had to prepare new pending block
	fn enable_and_prepare_sealing(&self, chain: &MiningBlockChainClient) -> bool {
		trace!(target: "miner", "enable_and_prepare_sealing: entering");
		let have_work = self.sealing_work.lock().unwrap().peek_last_ref().is_some();
		trace!(target: "miner", "enable_and_prepare_sealing: have_work={}", have_work);
		if !have_work {
			self.sealing_enabled.store(true, atomic::Ordering::Relaxed);
			self.prepare_sealing(chain);
		}
		let mut sealing_block_last_request = self.sealing_block_last_request.lock().unwrap();
		let best_number = chain.chain_info().best_block_number;
		if *sealing_block_last_request != best_number {
			trace!(target: "miner", "enable_and_prepare_sealing: Miner received request (was {}, now {}) - waking up.", *sealing_block_last_request, best_number);
			*sealing_block_last_request = best_number;
		}

		// Return if
		!have_work
	}

	fn add_transactions_to_queue(&self, chain: &MiningBlockChainClient, transactions: Vec<SignedTransaction>, origin: TransactionOrigin, transaction_queue: &mut TransactionQueue) ->
		Vec<Result<TransactionImportResult, Error>> {

		let fetch_account = |a: &Address| AccountDetails {
			nonce: chain.latest_nonce(a),
			balance: chain.latest_balance(a),
		};

		transactions.into_iter()
			.map(|tx| transaction_queue.add(tx, &fetch_account, origin))
			.collect()
	}

	/// Are we allowed to do a non-mandatory reseal?
	fn tx_reseal_allowed(&self) -> bool { Instant::now() > *self.next_allowed_reseal.lock().unwrap() }
}

const SEALING_TIMEOUT_IN_BLOCKS : u64 = 5;

impl MinerService for Miner {

	fn clear_and_reset(&self, chain: &MiningBlockChainClient) {
		self.transaction_queue.lock().unwrap().clear();
		self.update_sealing(chain);
	}

	fn status(&self) -> MinerStatus {
		let status = self.transaction_queue.lock().unwrap().status();
		let sealing_work = self.sealing_work.lock().unwrap();
		MinerStatus {
			transactions_in_pending_queue: status.pending,
			transactions_in_future_queue: status.future,
			transactions_in_pending_block: sealing_work.peek_last_ref().map_or(0, |b| b.transactions().len()),
		}
	}

	fn call(&self, chain: &MiningBlockChainClient, t: &SignedTransaction, analytics: CallAnalytics) -> Result<Executed, ExecutionError> {
		let sealing_work = self.sealing_work.lock().unwrap();
		match sealing_work.peek_last_ref() {
			Some(work) => {
				let block = work.block();

				// TODO: merge this code with client.rs's fn call somwhow.
				let header = block.header();
				let last_hashes = chain.last_hashes();
				let env_info = EnvInfo {
					number: header.number(),
					author: *header.author(),
					timestamp: header.timestamp(),
					difficulty: *header.difficulty(),
					last_hashes: last_hashes,
					gas_used: U256::zero(),
					gas_limit: U256::max_value(),
				};
				// that's just a copy of the state.
				let mut state = block.state().clone();
				let sender = try!(t.sender().map_err(|e| {
					let message = format!("Transaction malformed: {:?}", e);
					ExecutionError::TransactionMalformed(message)
				}));
				let balance = state.balance(&sender);
				let needed_balance = t.value + t.gas * t.gas_price;
				if balance < needed_balance {
					// give the sender a sufficient balance
					state.add_balance(&sender, &(needed_balance - balance));
				}
				let options = TransactOptions { tracing: analytics.transaction_tracing, vm_tracing: analytics.vm_tracing, check_nonce: false };
				let mut ret = Executive::new(&mut state, &env_info, self.engine(), chain.vm_factory()).transact(t, options);

				// TODO gav move this into Executive.
				if analytics.state_diffing {
					if let Ok(ref mut x) = ret {
						x.state_diff = Some(state.diff_from(block.state().clone()));
					}
				}
				ret
			},
			None => {
				chain.call(t, analytics)
			}
		}
	}

	fn balance(&self, chain: &MiningBlockChainClient, address: &Address) -> U256 {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(
			|| chain.latest_balance(address),
			|b| b.block().fields().state.balance(address)
		)
	}

	fn storage_at(&self, chain: &MiningBlockChainClient, address: &Address, position: &H256) -> H256 {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(
			|| chain.latest_storage_at(address, position),
			|b| b.block().fields().state.storage_at(address, position)
		)
	}

	fn nonce(&self, chain: &MiningBlockChainClient, address: &Address) -> U256 {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(|| chain.latest_nonce(address), |b| b.block().fields().state.nonce(address))
	}

	fn code(&self, chain: &MiningBlockChainClient, address: &Address) -> Option<Bytes> {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(|| chain.code(address), |b| b.block().fields().state.code(address))
	}

	fn set_author(&self, author: Address) {
		*self.author.write().unwrap() = author;
	}

	fn set_extra_data(&self, extra_data: Bytes) {
		*self.extra_data.write().unwrap() = extra_data;
	}

	/// Set the gas limit we wish to target when sealing a new block.
	fn set_gas_floor_target(&self, target: U256) {
		self.gas_range_target.write().unwrap().0 = target;
	}

	fn set_gas_ceil_target(&self, target: U256) {
		self.gas_range_target.write().unwrap().1 = target;
	}

	fn set_minimal_gas_price(&self, min_gas_price: U256) {
		self.transaction_queue.lock().unwrap().set_minimal_gas_price(min_gas_price);
	}

	fn minimal_gas_price(&self) -> U256 {
		*self.transaction_queue.lock().unwrap().minimal_gas_price()
	}

	fn sensible_gas_price(&self) -> U256 {
		// 10% above our minimum.
		*self.transaction_queue.lock().unwrap().minimal_gas_price() * 110.into() / 100.into()
	}

	fn sensible_gas_limit(&self) -> U256 {
		self.gas_range_target.read().unwrap().0 / 5.into()
	}

	fn transactions_limit(&self) -> usize {
		self.transaction_queue.lock().unwrap().limit()
	}

	fn set_transactions_limit(&self, limit: usize) {
		self.transaction_queue.lock().unwrap().set_limit(limit)
	}

	fn set_tx_gas_limit(&self, limit: U256) {
		self.transaction_queue.lock().unwrap().set_tx_gas_limit(limit)
	}

	/// Get the author that we will seal blocks as.
	fn author(&self) -> Address {
		*self.author.read().unwrap()
	}

	/// Get the extra_data that we will seal blocks with.
	fn extra_data(&self) -> Bytes {
		self.extra_data.read().unwrap().clone()
	}

	/// Get the gas limit we wish to target when sealing a new block.
	fn gas_floor_target(&self) -> U256 {
		self.gas_range_target.read().unwrap().0
	}

	/// Get the gas limit we wish to target when sealing a new block.
	fn gas_ceil_target(&self) -> U256 {
		self.gas_range_target.read().unwrap().1
	}

	fn import_external_transactions(&self, chain: &MiningBlockChainClient, transactions: Vec<SignedTransaction>) ->
		Vec<Result<TransactionImportResult, Error>> {

		let mut transaction_queue = self.transaction_queue.lock().unwrap();
		let results = self.add_transactions_to_queue(chain, transactions, TransactionOrigin::External,
													 &mut transaction_queue);

		if !results.is_empty() && self.options.reseal_on_external_tx &&	self.tx_reseal_allowed() {
			self.update_sealing(chain);
		}
		results
	}

	fn import_own_transaction(
		&self,
		chain: &MiningBlockChainClient,
		transaction: SignedTransaction,
	) -> Result<TransactionImportResult, Error> {

		let hash = transaction.hash();
		trace!(target: "own_tx", "Importing transaction: {:?}", transaction);

		let imported = {
			// Be sure to release the lock before we call enable_and_prepare_sealing
			let mut transaction_queue = self.transaction_queue.lock().unwrap();
			let import = self.add_transactions_to_queue(chain, vec![transaction], TransactionOrigin::Local, &mut transaction_queue).pop().unwrap();

			match import {
				Ok(ref res) => {
					trace!(target: "own_tx", "Imported transaction to {:?} (hash: {:?})", res, hash);
					trace!(target: "own_tx", "Status: {:?}", transaction_queue.status());
				},
				Err(ref e) => {
					trace!(target: "own_tx", "Failed to import transaction {:?} (hash: {:?})", e, hash);
					trace!(target: "own_tx", "Status: {:?}", transaction_queue.status());
					warn!(target: "own_tx", "Error importing transaction: {:?}", e);
				},
			}
			import
		};

		if imported.is_ok() && self.options.reseal_on_own_tx && self.tx_reseal_allowed() {
			// Make sure to do it after transaction is imported and lock is droped.
			// We need to create pending block and enable sealing
			let prepared = self.enable_and_prepare_sealing(chain);
			// If new block has not been prepared (means we already had one)
			// we need to update sealing
			if !prepared {
				self.update_sealing(chain);
			}
		}

		imported
	}

	fn all_transactions(&self) -> Vec<SignedTransaction> {
		let queue = self.transaction_queue.lock().unwrap();
		queue.top_transactions()
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		let queue = self.transaction_queue.lock().unwrap();
		let sw = self.sealing_work.lock().unwrap();
		// TODO: should only use the sealing_work when it's current (it could be an old block)
		let sealing_set = match self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			true => sw.peek_last_ref(),
			false => None,
		};
		match (&self.options.pending_set, sealing_set) {
			(&PendingSet::AlwaysQueue, _) | (&PendingSet::SealingOrElseQueue, None) => queue.top_transactions(),
			(_, sealing) => sealing.map_or_else(Vec::new, |s| s.transactions().clone()),
		}
	}

	fn pending_transactions_hashes(&self) -> Vec<H256> {
		let queue = self.transaction_queue.lock().unwrap();
		let sw = self.sealing_work.lock().unwrap();
		let sealing_set = match self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			true => sw.peek_last_ref(),
			false => None,
		};
		match (&self.options.pending_set, sealing_set) {
			(&PendingSet::AlwaysQueue, _) | (&PendingSet::SealingOrElseQueue, None) => queue.pending_hashes(),
			(_, sealing) => sealing.map_or_else(Vec::new, |s| s.transactions().iter().map(|t| t.hash()).collect()),
		}
	}

	fn transaction(&self, hash: &H256) -> Option<SignedTransaction> {
		let queue = self.transaction_queue.lock().unwrap();
		let sw = self.sealing_work.lock().unwrap();
		let sealing_set = match self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			true => sw.peek_last_ref(),
			false => None,
		};
		match (&self.options.pending_set, sealing_set) {
			(&PendingSet::AlwaysQueue, _) | (&PendingSet::SealingOrElseQueue, None) => queue.find(hash),
			(_, sealing) => sealing.and_then(|s| s.transactions().iter().find(|t| &t.hash() == hash).cloned()),
		}
	}

	fn pending_receipts(&self) -> BTreeMap<H256, Receipt> {
		match (self.sealing_enabled.load(atomic::Ordering::Relaxed), self.sealing_work.lock().unwrap().peek_last_ref()) {
			(true, Some(pending)) => {
				let hashes = pending.transactions()
					.iter()
					.map(|t| t.hash());

				let receipts = pending.receipts().clone().into_iter();

				hashes.zip(receipts).collect()
			},
			_ => BTreeMap::new()
		}
	}

	fn last_nonce(&self, address: &Address) -> Option<U256> {
		self.transaction_queue.lock().unwrap().last_nonce(address)
	}

	fn update_sealing(&self, chain: &MiningBlockChainClient) {
		if self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			let current_no = chain.chain_info().best_block_number;
			let has_local_transactions = self.transaction_queue.lock().unwrap().has_local_pending_transactions();
			let last_request = *self.sealing_block_last_request.lock().unwrap();
			let should_disable_sealing = !self.forced_sealing()
				&& !has_local_transactions
				&& current_no > last_request
				&& current_no - last_request > SEALING_TIMEOUT_IN_BLOCKS;

			if should_disable_sealing {
				trace!(target: "miner", "Miner sleeping (current {}, last {})", current_no, last_request);
				self.sealing_enabled.store(false, atomic::Ordering::Relaxed);
				self.sealing_work.lock().unwrap().reset();
			} else {
				*self.next_allowed_reseal.lock().unwrap() = Instant::now() + self.options.reseal_min_period;
				self.prepare_sealing(chain);
			}
		}
	}

	fn map_sealing_work<F, T>(&self, chain: &MiningBlockChainClient, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		trace!(target: "miner", "map_sealing_work: entering");
		self.enable_and_prepare_sealing(chain);
		trace!(target: "miner", "map_sealing_work: sealing prepared");
		let mut sealing_work = self.sealing_work.lock().unwrap();
		let ret = sealing_work.use_last_ref();
		trace!(target: "miner", "map_sealing_work: leaving use_last_ref={:?}", ret.as_ref().map(|b| b.block().fields().header.hash()));
		ret.map(f)
	}

	fn submit_seal(&self, chain: &MiningBlockChainClient, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		let result = if let Some(b) = self.sealing_work.lock().unwrap().get_used_if(if self.options.enable_resubmission { GetAction::Clone } else { GetAction::Take }, |b| &b.hash() == &pow_hash) {
			b.lock().try_seal(self.engine(), seal).or_else(|_| {
				warn!(target: "miner", "Mined solution rejected: Invalid.");
				Err(Error::PowInvalid)
			})
		} else {
			warn!(target: "miner", "Mined solution rejected: Block unknown or out of date.");
			Err(Error::PowHashInvalid)
		};
		result.and_then(|sealed| {
			let n = sealed.header().number();
			let h = sealed.header().hash();
			try!(chain.import_sealed_block(sealed));
			info!(target: "miner", "Mined block imported OK. #{}: {}", paint(White.bold(), format!("{}", n)), paint(White.bold(), h.hex()));
			Ok(())
		})
	}

	fn chain_new_blocks(&self, chain: &MiningBlockChainClient, _imported: &[H256], _invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		fn fetch_transactions(chain: &MiningBlockChainClient, hash: &H256) -> Vec<SignedTransaction> {
			let block = chain
				.block(BlockID::Hash(*hash))
				// Client should send message after commit to db and inserting to chain.
				.expect("Expected in-chain blocks.");
			let block = BlockView::new(&block);
			let txs = block.transactions();
			// populate sender
			for tx in &txs {
				let _sender = tx.sender();
			}
			txs
		}

		// 1. We ignore blocks that were `imported` (because it means that they are not in canon-chain, and transactions
		//	  should be still available in the queue.
		// 2. We ignore blocks that are `invalid` because it doesn't have any meaning in terms of the transactions that
		//    are in those blocks

		// First update gas limit in transaction queue
		self.update_gas_limit(chain);

		// Then import all transactions...
		{
			let out_of_chain = retracted
				.par_iter()
				.map(|h| fetch_transactions(chain, h));
			out_of_chain.for_each(|txs| {
				let mut transaction_queue = self.transaction_queue.lock().unwrap();
				let _ = self.add_transactions_to_queue(
					chain, txs, TransactionOrigin::External, &mut transaction_queue
				);
			});
		}

		// ...and at the end remove old ones
		{
			let in_chain = enacted
				.par_iter()
				.map(|h: &H256| fetch_transactions(chain, h));

			in_chain.for_each(|mut txs| {
				let mut transaction_queue = self.transaction_queue.lock().unwrap();

				let to_remove = txs.drain(..)
						.map(|tx| {
							tx.sender().expect("Transaction is in block, so sender has to be defined.")
						})
						.collect::<HashSet<Address>>();
				for sender in to_remove.into_iter() {
					transaction_queue.remove_all(sender, chain.latest_nonce(&sender));
				}
			});
		}

		self.update_sealing(chain);
	}
}

#[cfg(test)]
mod tests {

	use super::super::MinerService;
	use super::Miner;
	use util::*;
	use client::{TestBlockChainClient, EachBlockWith};
	use block::*;
	use spec::Spec;

	// TODO [ToDr] To uncomment` when TestBlockChainClient can actually return a ClosedBlock.
	#[ignore]
	#[test]
	fn should_prepare_block_to_seal() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::with_spec(Spec::new_test());

		// when
		let sealing_work = miner.map_sealing_work(&client, |_| ());
		assert!(sealing_work.is_some(), "Expected closed block");
	}

	#[ignore]
	#[test]
	fn should_still_work_after_a_couple_of_blocks() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::with_spec(Spec::new_test());

		let res = miner.map_sealing_work(&client, |b| b.block().fields().header.hash());
		assert!(res.is_some());
		assert!(miner.submit_seal(&client, res.unwrap(), vec![]).is_ok());

		// two more blocks mined, work requested.
		client.add_blocks(1, EachBlockWith::Uncle);
		miner.map_sealing_work(&client, |b| b.block().fields().header.hash());

		client.add_blocks(1, EachBlockWith::Uncle);
		miner.map_sealing_work(&client, |b| b.block().fields().header.hash());

		// solution to original work submitted.
		assert!(miner.submit_seal(&client, res.unwrap(), vec![]).is_ok());
	}
}
