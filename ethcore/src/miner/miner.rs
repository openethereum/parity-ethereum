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

use std::fmt;
use std::time::{Instant, Duration};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use account_provider::{AccountProvider, SignError as AccountError};
use ansi_term::Colour;
use ethereum_types::{H256, U256, Address};
use parking_lot::{Mutex, RwLock};
use bytes::Bytes;
use engines::{EthEngine, Seal};
use error::*;
// use ethcore_miner::banning_queue::{BanningTransactionQueue, Threshold};
// use ethcore_miner::local_transactions::{Status as LocalTransactionStatus};
// use ethcore_miner::transaction_queue::{
// 	TransactionQueue,
// 	RemovalReason,
// 	TransactionDetailsProvider as TransactionQueueDetailsProvider,
// 	PrioritizationStrategy,
// 	AccountDetails,
// 	TransactionOrigin,
// };
use ethcore_miner::pool::{self, TransactionQueue};
use ethcore_miner::work_notify::{WorkPoster, NotifyWork};
use ethcore_miner::gas_pricer::{GasPricer, GasPriceCalibratorOptions};
// use ethcore_miner::service_transaction_checker::ServiceTransactionChecker;
use miner::MinerService;
use timer::PerfTimer;
use transaction::{
	self,
	Action,
	UnverifiedTransaction,
	PendingTransaction,
	SignedTransaction,
};
use using_queue::{UsingQueue, GetAction};

use block::{ClosedBlock, IsBlock, Block};
use client::{MiningBlockChainClient, BlockId, TransactionId};
use executive::contract_address;
use header::{Header, BlockNumber};
use receipt::{Receipt, RichReceipt};
use spec::Spec;
use state::State;

/// Different possible definitions for pending transaction set.
#[derive(Debug, PartialEq)]
pub enum PendingSet {
	/// Always just the transactions in the queue. These have had only cheap checks.
	AlwaysQueue,
	/// Always just the transactions in the sealing block. These have had full checks but
	/// may be empty if the node is not actively mining or has force_sealing enabled.
	AlwaysSealing,
	/// Try the sealing block, but if it is not currently sealing, fallback to the queue.
	SealingOrElseQueue,
}

// /// Type of the gas limit to apply to the transaction queue.
// #[derive(Debug, PartialEq)]
// pub enum GasLimit {
// 	/// Depends on the block gas limit and is updated with every block.
// 	Auto,
// 	/// No limit.
// 	None,
// 	/// Set to a fixed gas value.
// 	Fixed(U256),
// }
//
// /// Transaction queue banning settings.
// #[derive(Debug, PartialEq, Clone)]
// pub enum Banning {
// 	/// Banning in transaction queue is disabled
// 	Disabled,
// 	/// Banning in transaction queue is enabled
// 	Enabled {
// 		/// Upper limit of transaction processing time before banning.
// 		offend_threshold: Duration,
// 		/// Number of similar offending transactions before banning.
// 		min_offends: u16,
// 		/// Number of seconds the offender is banned for.
// 		ban_duration: Duration,
// 	},
// }

/// Configures the behaviour of the miner.
#[derive(Debug, PartialEq)]
pub struct MinerOptions {
	/// URLs to notify when there is new work.
	pub new_work_notify: Vec<String>,
	/// Force the miner to reseal, even when nobody has asked for work.
	pub force_sealing: bool,
	/// Reseal on receipt of new external transactions.
	pub reseal_on_external_tx: bool,
	/// Reseal on receipt of new local transactions.
	pub reseal_on_own_tx: bool,
	/// Reseal when new uncle block has been imported.
	pub reseal_on_uncle: bool,
	/// Minimum period between transaction-inspired reseals.
	pub reseal_min_period: Duration,
	/// Maximum period between blocks (enables force sealing after that).
	pub reseal_max_period: Duration,
	// /// Maximum amount of gas to bother considering for block insertion.
	// pub tx_gas_limit: U256,
	// /// Maximum size of the transaction queue.
	// pub tx_queue_size: usize,
	// /// Maximum memory usage of transactions in the queue (current / future).
	// pub tx_queue_memory_limit: Option<usize>,
	// / Strategy to use for prioritizing transactions in the queue.
	// pub tx_queue_strategy: PrioritizationStrategy,
	/// Whether we should fallback to providing all the queue's transactions or just pending.
	pub pending_set: PendingSet,
	/// How many historical work packages can we store before running out?
	pub work_queue_size: usize,
	/// Can we submit two different solutions for the same block and expect both to result in an import?
	pub enable_resubmission: bool,
	// / Global gas limit for all transaction in the queue except for local and retracted.
	// pub tx_queue_gas_limit: GasLimit,
	// / Banning settings.
	// pub tx_queue_banning: Banning,
	// / Do we refuse to accept service transactions even if sender is certified.
	// pub refuse_service_transactions: bool,
	/// Create a pending block with maximal possible gas limit.
	/// NOTE: Such block will contain all pending transactions but
	/// will be invalid if mined.
	pub infinite_pending_block: bool,
}

impl Default for MinerOptions {
	fn default() -> Self {
		MinerOptions {
			new_work_notify: vec![],
			force_sealing: false,
			reseal_on_external_tx: false,
			reseal_on_own_tx: true,
			reseal_on_uncle: false,
			// tx_gas_limit: !U256::zero(),
			// tx_queue_size: 8192,
			// tx_queue_memory_limit: Some(2 * 1024 * 1024),
			// tx_queue_gas_limit: GasLimit::None,
			// tx_queue_strategy: PrioritizationStrategy::GasPriceOnly,
			pending_set: PendingSet::AlwaysQueue,
			reseal_min_period: Duration::from_secs(2),
			reseal_max_period: Duration::from_secs(120),
			work_queue_size: 20,
			enable_resubmission: true,
			// tx_queue_banning: Banning::Disabled,
			// refuse_service_transactions: false,
			infinite_pending_block: false,
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct MiningParams {
	pub gas_range_target: (U256, U256),
	pub author: Address,
	pub extra_data: Bytes,
}

struct SealingWork {
	queue: UsingQueue<ClosedBlock>,
	enabled: bool,
	next_allowed_reseal: Instant,
	next_mandatory_reseal: Instant,
	sealing_block_last_request: u64,
}

/// Keeps track of transactions using priority queue and holds currently mined block.
/// Handles preparing work for "work sealing" or seals "internally" if Engine does not require work.
pub struct Miner {
	// NOTE [ToDr]  When locking always lock in this order!
	sealing: Mutex<SealingWork>,
	params: RwLock<MiningParams>,
	listeners: RwLock<Vec<Box<NotifyWork>>>,
	gas_pricer: Mutex<GasPricer>,
	options: MinerOptions,
	transaction_queue: TransactionQueue,
	engine: Arc<EthEngine>,
	accounts: Option<Arc<AccountProvider>>,
	// TODO [ToDr] Check lock order
	// service_transaction_action: ServiceTransactionAction,
}

impl Miner {
	/// Push notifier that will handle new jobs
	pub fn add_work_listener(&self, notifier: Box<NotifyWork>) {
		self.sealing.lock().enabled = true;
		self.listeners.write().push(notifier);
	}

	pub fn add_work_listener_url(&self, urls: &[String]) {
		self.add_work_listener(Box::new(WorkPoster::new(&urls)));
	}

	/// Creates new instance of miner Arc.
	pub fn new(options: MinerOptions, gas_pricer: GasPricer, spec: &Spec, accounts: Option<Arc<AccountProvider>>) -> Arc<Miner> {
		Arc::new(Miner::new_raw(options, gas_pricer, spec, accounts))
	}

	/// Creates new instance of miner.
	fn new_raw(options: MinerOptions, gas_pricer: GasPricer, spec: &Spec, accounts: Option<Arc<AccountProvider>>) -> Miner {
		// let mem_limit = options.tx_queue_memory_limit.unwrap_or_else(usize::max_value);

		let txq = unimplemented!();
		// let txq = TransactionQueue::with_limits(
		// 	// options.tx_queue_strategy,
		// 	options.tx_queue_size,
		// 	mem_limit,
		// 	// gas_limit,
		// 	options.tx_gas_limit
		// );
		// let txq = match options.tx_queue_banning {
		// 	Banning::Disabled => BanningTransactionQueue::new(txq, Threshold::NeverBan, Duration::from_secs(180)),
		// 	Banning::Enabled { ban_duration, min_offends, .. } => BanningTransactionQueue::new(
		// 		txq,
		// 		Threshold::BanAfter(min_offends),
		// 		ban_duration,
		// 	),
		// };

		let notifiers: Vec<Box<NotifyWork>> = match options.new_work_notify.is_empty() {
			true => Vec::new(),
			false => vec![)],
		};

		let (limits, verifier_options) = unimplemented!();

		Miner {
			sealing: RwLock::new(SealingWork{
				queue: UsingQueue::new(options.work_queue_size),
				enabled: options.force_sealing
					|| !options.new_work_notify.is_empty()
					|| spec.engine.seals_internally().is_some(),
				next_allowed_reseal: Instant::now(),
				next_mandatory_reseal: Instant::now() + options.reseal_max_period,
				sealing_block_last_request: 0,
			}),
			params: RwLock::new(MiningParams::default()),
			notifiers: RwLock::new(vec![]),
			gas_pricer: Mutex::new(gas_pricer),
			options,
			transaction_queue: TransactionQueue::new(limits, verifier_options),
			accounts,
			engine: spec.engine.clone(),
		}
	}

	/// Creates new instance of miner with accounts and with given spec.
	pub fn with_spec_and_accounts(spec: &Spec, accounts: Option<Arc<AccountProvider>>) -> Miner {
		Miner::new_raw(Default::default(), GasPricer::new_fixed(20_000_000_000u64.into()), spec, accounts)
	}

	/// Creates new instance of miner without accounts, but with given spec.
	pub fn with_spec(spec: &Spec) -> Miner {
		Miner::new_raw(Default::default(), GasPricer::new_fixed(20_000_000_000u64.into()), spec, None)
	}

	fn forced_sealing(&self) -> bool {
		self.options.force_sealing || !self.notifiers.read().is_empty()
	}

	/// Clear all pending block states
	pub fn clear(&self) {
		self.sealing.lock().queue.reset();
	}

	/// Get `Some` `clone()` of the current pending block's state or `None` if we're not sealing.
	pub fn pending_state(&self, latest_block_number: BlockNumber) -> Option<State<::state_db::StateDB>> {
		self.map_pending_block(|b| b.state().clone(), latest_block_number)
	}

	/// Get `Some` `clone()` of the current pending block or `None` if we're not sealing.
	pub fn pending_block(&self, latest_block_number: BlockNumber) -> Option<Block> {
		self.map_pending_block(|b| b.to_base(), latest_block_number)
	}

	/// Get `Some` `clone()` of the current pending block header or `None` if we're not sealing.
	pub fn pending_block_header(&self, latest_block_number: BlockNumber) -> Option<Header> {
		self.map_pending_block(|b| b.header().clone(), latest_block_number)
	}

	fn map_pending_block<F, T>(&self, f: F, latest_block_number: BlockNumber) -> Option<T> where
		F: FnOnce(&ClosedBlock) -> T,
	{
		self.from_pending_block(
			latest_block_number,
			|| None,
			|block| Some(f(block)),
		)
	}

	/// Prepares new block for sealing including top transactions from queue.
	fn prepare_block(&self, chain: &MiningBlockChainClient) -> (ClosedBlock, Option<H256>) {
		let _timer = PerfTimer::new("prepare_block");
		let chain_info = chain.chain_info();
		let (pending, mut open_block, original_work_hash) = {
			let nonce_cap = if chain_info.best_block_number + 1 >= self.engine.params().dust_protection_transition {
				Some((self.engine.params().nonce_cap_increment * (chain_info.best_block_number + 1)).into())
			} else { None };

			let mut sealing = self.sealing.lock();
			let last_work_hash = sealing.queue.peek_last_ref().map(|pb| pb.block().fields().header.hash());
			let best_hash = chain_info.best_block_hash;

			// check to see if last ClosedBlock in would_seals is actually same parent block.
			// if so
			//   duplicate, re-open and push any new transactions.
			//   if at least one was pushed successfully, close and enqueue new ClosedBlock;
			//   otherwise, leave everything alone.
			// otherwise, author a fresh block.
			let mut open_block = match sealing.queue.pop_if(|b| b.block().fields().header.parent_hash() == &best_hash) {
				Some(old_block) => {
					trace!(target: "miner", "prepare_block: Already have previous work; updating and returning");
					// add transactions to old_block
					chain.reopen_block(old_block)
				}
				None => {
					// block not found - create it.
					trace!(target: "miner", "prepare_block: No existing work - making new block");
					let params = self.params.read().clone();
					chain.prepare_open_block(
						params.author,
						params.gas_range_target,
						params.extra_data,
					)
				}
			};

			if self.options.infinite_pending_block {
				open_block.set_gas_limit(!U256::zero());
			}

			let pending = self.transaction_queue.pending(
				chain_info.best_block_number,
				chain_info.best_block_timestamp,
				nonce_cap,
			);

			(pending, open_block, last_work_hash)
		};

		let mut invalid_transactions = HashSet::new();
		let mut non_allowed_transactions = HashSet::new();
		let mut transactions_to_penalize = HashSet::new();
		let block_number = open_block.block().fields().header.number();

		let mut tx_count = 0usize;
		let mut skipped_transactions = 0usize;
		let mut max_gas = open_block.header();

		for tx in pending.transactions() {
			let hash = tx.hash();
			let start = Instant::now();
			// Check whether transaction type is allowed for sender
			let result = match self.engine.machine().verify_transaction(&tx, open_block.header(), chain.as_block_chain_client()) {
				Err(Error::Transaction(transaction::Error::NotAllowed)) => Err(transaction::Error::NotAllowed.into()),
				_ => open_block.push_transaction(tx, None),
			};
			let took = start.elapsed();

			// Check for heavy transactions
			// match self.options.tx_queue_banning {
			// 	Banning::Enabled { ref offend_threshold, .. } if &took > offend_threshold => {
			// 		match self.transaction_queue.write().ban_transaction(&hash) {
			// 			true => {
			// 				warn!(target: "miner", "Detected heavy transaction. Banning the sender and recipient/code.");
			// 			},
			// 			false => {
			// 				transactions_to_penalize.insert(hash);
			// 				debug!(target: "miner", "Detected heavy transaction. Penalizing sender.")
			// 			}
			// 		}
			// 	},
			// 	_ => {},
			// }
			trace!(target: "miner", "Adding tx {:?} took {:?}", hash, took);
			match result {
				Err(Error::Execution(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, gas })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of gas limit: {:?} (limit: {:?}, used: {:?}, gas: {:?})", hash, gas_limit, gas_used, gas);

					// Penalize transaction if it's above current gas limit
					if gas > gas_limit {
						invalid_transactions.insert(hash);
					}

					// Exit early if gas left is smaller then min_tx_gas
					let min_tx_gas: U256 = 21000.into();	// TODO: figure this out properly.
					let gas_left = gas_limit - gas_used;
					if gas_left < min_tx_gas {
						break;
					}

					// Avoid iterating over the entire queue in case block is almost full.
					skipped_transactions += 1;
					if skipped_transactions > 8 {
						break;
					}
				},
				// Invalid nonce error can happen only if previous transaction is skipped because of gas limit.
				// If there is errornous state of transaction queue it will be fixed when next block is imported.
				Err(Error::Execution(ExecutionError::InvalidNonce { expected, got })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of invalid nonce: {:?} (expected: {:?}, got: {:?})", hash, expected, got);
				},
				// already have transaction - ignore
				Err(Error::Transaction(transaction::Error::AlreadyImported)) => {},
				Err(Error::Transaction(transaction::Error::NotAllowed)) => {
					non_allowed_transactions.insert(hash);
					debug!(target: "miner", "Skipping non-allowed transaction for sender {:?}", hash);
				},
				Err(e) => {
					invalid_transactions.insert(hash);
					debug!(
						target: "miner", "Error adding transaction to block: number={}. transaction_hash={:?}, Error: {:?}", block_number, hash, e
					);
				},
				// imported ok
				_ => tx_count += 1,
			}
		}
		trace!(target: "miner", "Pushed {} transactions", tx_count);

		let block = open_block.close();

		// let fetch_nonce = |a: &Address| chain.latest_nonce(a);

		{
			let mut queue = self.transaction_queue.write();
			for hash in invalid_transactions {
				// queue.remove(&hash, &fetch_nonce, RemovalReason::Invalid);
				queue.remove(&hash, true)
			}
			for hash in non_allowed_transactions {
				// queue.remove(&hash, &fetch_nonce, RemovalReason::NotAllowed);
				queue.remove(&hash, false)
			}
			// for hash in transactions_to_penalize {
				// queue.penalize(&hash);
			// }
		}

		(block, original_work_hash)
	}

	/// Asynchronously updates minimal gas price for transaction queue
	fn recalibrate_minimal_gas_price(&self) {
		debug!(target: "miner", "minimal_gas_price: recalibrating...");
		let txq = self.transaction_queue.clone();
		self.gas_pricer.lock().recalibrate(move |price| {
			debug!(target: "miner", "minimal_gas_price: Got gas price! {}", price);
			txq.write().set_minimal_gas_price(price);
		});
	}

	/// Check is reseal is allowed and necessary.
	fn requires_reseal(&self, best_block: BlockNumber) -> bool {
		let mut sealing = self.sealing.lock();
		if sealing.enabled {
			trace!(target: "miner", "requires_reseal: sealing is disabled");
			return false
		}

		let has_local_transactions = self.transaction_queue.has_local_pending_transactions();
		trace!(target: "miner", "requires_reseal: sealing enabled");

		let last_request = sealing.sealing_block_last_request;
		let should_disable_sealing = !self.forced_sealing()
			&& !has_local_transactions
			&& self.engine.seals_internally().is_none()
			&& best_block > last_request
			&& best_block - last_request > SEALING_TIMEOUT_IN_BLOCKS;

		trace!(target: "miner", "requires_reseal: should_disable_sealing={}; best_block={}, last_request={}", should_disable_sealing, best_block, last_request);

		if should_disable_sealing {
			trace!(target: "miner", "Miner sleeping (current {}, last {})", best_block, last_request);
			sealing.enabled = false;
			sealing.queue.reset();
			false
		} else {
			// sealing enabled and we don't want to sleep.
			sealing.next_allowed_reseal = Instant::now() + self.options.reseal_min_period;
			true
		}
	}

	/// Attempts to perform internal sealing (one that does not require work) and handles the result depending on the type of Seal.
	fn seal_and_import_block_internally(&self, chain: &MiningBlockChainClient, block: ClosedBlock) -> bool {
		let mut sealing = self.sealing.lock();
		if block.transactions().is_empty()
			&& !self.forced_sealing()
			&& Instant::now() <= sealing.next_mandatory_reseal
		{
			return false
		}

		trace!(target: "miner", "seal_block_internally: attempting internal seal.");

		let parent_header = match chain.block_header(BlockId::Hash(*block.header().parent_hash())) {
			Some(hdr) => hdr.decode(),
			None => return false,
		};

		match self.engine.generate_seal(block.block(), &parent_header) {
			// Save proposal for later seal submission and broadcast it.
			Seal::Proposal(seal) => {
				trace!(target: "miner", "Received a Proposal seal.");
				sealing.next_mandatory_reseal = Instant::now() + self.options.reseal_max_period;
				sealing.queue.push(block.clone());
				sealing.queue.use_last_ref();

				block
					.lock()
					.seal(&*self.engine, seal)
					.map(|sealed| {
						chain.broadcast_proposal_block(sealed);
						true
					})
					.unwrap_or_else(|e| {
						warn!("ERROR: seal failed when given internally generated seal: {}", e);
						false
					})
			},
			// Directly import a regular sealed block.
			Seal::Regular(seal) => {
				sealing.next_mandatory_reseal = Instant::now() + self.options.reseal_max_period;
				block
					.lock()
					.seal(&*self.engine, seal)
					.map(|sealed| chain.import_sealed_block(sealed).is_ok())
					.unwrap_or_else(|e| {
						warn!("ERROR: seal failed when given internally generated seal: {}", e);
						false
					})
			},
			Seal::None => false,
		}
	}

	/// Prepares work which has to be done to seal.
	fn prepare_work(&self, block: ClosedBlock, original_work_hash: Option<H256>) {
		let (work, is_new) = {
			let block_header = block.block().fields().header;
			let block_hash = block_header.hash();
			let mut sealing = self.sealing.lock();
			let last_work_hash = sealing.queue.peek_last_ref().map(|pb| pb.block().fields().header.hash());

			trace!(
				target: "miner",
				"prepare_work: Checking whether we need to reseal: orig={:?} last={:?}, this={:?}",
				original_work_hash, last_work_hash, block_hash
			);

			let (work, is_new) = if last_work_hash.map_or(true, |h| h != block_hash) {
				trace!(
					target: "miner",
					"prepare_work: Pushing a new, refreshed or borrowed pending {}...",
					block_hash
				);
				let is_new = original_work_hash.map_or(true, |h| h != block_hash);

				sealing.queue.push(block);
				// If push notifications are enabled we assume all work items are used.
				if is_new && !self.notifiers.read().is_empty() {
					sealing.queue.use_last_ref();
				}

				(Some((block_hash, block_header.difficulty(), block_header.number())), is_new)
			} else {
				(None, false)
			};
			trace!(
				target: "miner",
				"prepare_work: leaving (last={:?})",
				sealing.queue.peek_last_ref().map(|b| b.block().fields().header.hash())
			);
			(work, is_new)
		};
		if is_new {
			work.map(|(pow_hash, difficulty, number)| {
				for notifier in self.notifiers.read().iter() {
					notifier.notify(pow_hash, difficulty, number)
				}
			});
		}
	}

	fn update_gas_limit(&self, client: &MiningBlockChainClient) {
		let gas_limit = client.best_block_header().gas_limit();
		// let mut queue = self.transaction_queue.write();
		// queue.set_gas_limit(gas_limit);
		// if let GasLimit::Auto = self.options.tx_queue_gas_limit {
		// 	// Set total tx queue gas limit to be 20x the block gas limit.
		// 	queue.set_total_gas_limit(gas_limit * 20.into());
		// }
		unimplemented!()
	}

	/// Returns true if we had to prepare new pending block.
	fn prepare_work_sealing(&self, client: &MiningBlockChainClient) -> bool {
		trace!(target: "miner", "prepare_work_sealing: entering");
		let prepare_new = {
			let mut sealing = self.sealing.lock();
			let have_work = sealing.queue.peek_last_ref().is_some();
			trace!(target: "miner", "prepare_work_sealing: have_work={}", have_work);
			if !have_work {
				sealing.enabled = true;
				true
			} else {
				false
			}
		};

		if prepare_new {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing locks.          |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			let (block, original_work_hash) = self.prepare_block(client);
			self.prepare_work(block, original_work_hash);
		}

		let best_number = client.chain_info().best_block_number;
		let mut sealing = self.sealing.lock();
		if sealing.sealing_block_last_request != best_number {
			trace!(
				target: "miner",
				"prepare_work_sealing: Miner received request (was {}, now {}) - waking up.",
				sealing.sealing_block_last_request, best_number
			);
			sealing.sealing_block_last_request = best_number;
		}

		// Return if we restarted
		prepare_new
	}

	/// Are we allowed to do a non-mandatory reseal?
	fn tx_reseal_allowed(&self) -> bool {
		Instant::now() > self.sealing.lock().next_allowed_reseal
	}

	fn from_pending_block<H, F, G>(&self, latest_block_number: BlockNumber, from_chain: F, map_block: G) -> H
		where F: Fn() -> H, G: FnOnce(&ClosedBlock) -> H {
		let sealing = self.sealing.lock();
		sealing.queue.peek_last_ref().map_or_else(
			|| from_chain(),
			|b| {
				if b.block().header().number() > latest_block_number {
					map_block(b)
				} else {
					from_chain()
				}
			}
		)
	}
}

const SEALING_TIMEOUT_IN_BLOCKS : u64 = 5;

impl MinerService for Miner {

	fn clear_and_reset(&self, chain: &MiningBlockChainClient) {
		self.transaction_queue.clear();
		// --------------------------------------------------------------------------
		// | NOTE Code below requires sealing lock.                            |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
		self.update_sealing(chain);
	}

	fn mining_params(&self) -> MiningParams {
		self.params.read().clone()
	}

	fn set_gas_range_target(&self, gas_range_target: (U256, U256)) {
		self.params.write().gas_range_target = gas_range_target;
	}

	fn set_extra_data(&self, extra_data: Bytes) {
		self.params.write().extra_data = extra_data;
	}

	fn set_author(&self, address: Address, password: Option<String>) -> Result<(), AccountError> {
		self.params.write().author = address;

		if self.engine.seals_internally().is_some() {
			if let Some(ref ap) = self.accounts {
				let password = password.unwrap_or_default();
				// Sign test message
				ap.sign(address.clone(), Some(password.clone()), Default::default())?;
				// Enable sealing
				self.sealing.lock().enabled = true;
				// --------------------------------------------------------------------------
				// | NOTE Code below may require author and sealing locks                   |
				// | (some `Engine`s call `EngineClient.update_sealing()`)                  |
				// | Make sure to release the locks before calling that method.             |
				// --------------------------------------------------------------------------
				self.engine.set_signer(ap.clone(), address, password);
				Ok(())
			} else {
				warn!(target: "miner", "No account provider");
				Err(AccountError::NotFound)
			}
		}
	}

	fn sensible_gas_price(&self) -> U256 {
		// 10% above our minimum.
		*self.transaction_queue.minimal_gas_price() * 110u32 / 100.into()
	}

	fn sensible_gas_limit(&self) -> U256 {
		self.params.read().0 / 5.into()
	}

	fn import_external_transactions(
		&self,
		chain: &MiningBlockChainClient,
		transactions: Vec<UnverifiedTransaction>
	) -> Vec<Result<transaction::ImportResult, Error>> {
		trace!(target: "external_tx", "Importing external transactions");
		let results = self.transaction_queue.import(
			chain,
			transactions.into_iter().map(pool::verifier::Transaction::Unverified).collect(),
		);

		if !results.is_empty() && self.options.reseal_on_external_tx &&	self.tx_reseal_allowed() {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing locks.          |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			self.update_sealing(chain);
		}
		results
	}

	fn import_own_transaction(
		&self,
		chain: &MiningBlockChainClient,
		pending: PendingTransaction,
	) -> Result<transaction::ImportResult, Error> {

		trace!(target: "own_tx", "Importing transaction: {:?}", pending);

		let imported = self.transaction_queue.import(
			chain,
			vec![pool::verifier::Transaction::Pending(pending)]
		).pop().expect("one result returned per added transaction; one added => one result; qed");

		match imported {
			Ok(_) => {
				trace!(target: "own_tx", "Status: {:?}", self.transaction_queue.status());
			},
			Err(ref e) => {
				trace!(target: "own_tx", "Status: {:?}", self.transaction_queue.status());
				warn!(target: "own_tx", "Error importing transaction: {:?}", e);
			},
		}

		// --------------------------------------------------------------------------
		// | NOTE Code below requires transaction_queue and sealing locks.          |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
		if imported.is_ok() && self.options.reseal_on_own_tx && self.tx_reseal_allowed() {
			// Make sure to do it after transaction is imported and lock is droped.
			// We need to create pending block and enable sealing.
			if self.engine.seals_internally().unwrap_or(false) || !self.prepare_work_sealing(chain) {
				// If new block has not been prepared (means we already had one)
				// or Engine might be able to seal internally,
				// we need to update sealing.
				self.update_sealing(chain);
			}
		}

		imported
	}

	fn pending_transactions(&self) -> Vec<PendingTransaction> {
		let queue = self.transaction_queue.read();
		queue.pending_transactions(BlockNumber::max_value(), u64::max_value())
	}

	// fn local_transactions(&self) -> BTreeMap<H256, LocalTransactionStatus> {
	// 	let queue = self.transaction_queue.read();
	// 	queue.local_transactions()
	// 		.iter()
	// 		.map(|(hash, status)| (*hash, status.clone()))
	// 		.collect()
	// }

	fn future_transactions(&self) -> Vec<PendingTransaction> {
		self.transaction_queue.read().future_transactions()
	}

	fn ready_transactions(&self, best_block: BlockNumber, best_block_timestamp: u64) -> Vec<PendingTransaction> {
		let queue = self.transaction_queue.read();
		match self.options.pending_set {
			PendingSet::AlwaysQueue => queue.pending_transactions(best_block, best_block_timestamp),
			PendingSet::SealingOrElseQueue => {
				self.from_pending_block(
					best_block,
					|| queue.pending_transactions(best_block, best_block_timestamp),
					|sealing| sealing.transactions().iter().map(|t| t.clone().into()).collect()
				)
			},
			PendingSet::AlwaysSealing => {
				self.from_pending_block(
					best_block,
					|| vec![],
					|sealing| sealing.transactions().iter().map(|t| t.clone().into()).collect()
				)
			},
		}
	}

	fn pending_transactions_hashes(&self, best_block: BlockNumber) -> Vec<H256> {
		let queue = self.transaction_queue.read();
		match self.options.pending_set {
			PendingSet::AlwaysQueue => queue.pending_hashes(),
			PendingSet::SealingOrElseQueue => {
				self.from_pending_block(
					best_block,
					|| queue.pending_hashes(),
					|sealing| sealing.transactions().iter().map(|t| t.hash()).collect()
				)
			},
			PendingSet::AlwaysSealing => {
				self.from_pending_block(
					best_block,
					|| vec![],
					|sealing| sealing.transactions().iter().map(|t| t.hash()).collect()
				)
			},
		}
	}

	fn transaction(&self, best_block: BlockNumber, hash: &H256) -> Option<PendingTransaction> {
		let queue = self.transaction_queue.read();
		match self.options.pending_set {
			PendingSet::AlwaysQueue => queue.find(hash),
			PendingSet::SealingOrElseQueue => {
				self.from_pending_block(
					best_block,
					|| queue.find(hash),
					|sealing| sealing.transactions().iter().find(|t| &t.hash() == hash).cloned().map(Into::into)
				)
			},
			PendingSet::AlwaysSealing => {
				self.from_pending_block(
					best_block,
					|| None,
					|sealing| sealing.transactions().iter().find(|t| &t.hash() == hash).cloned().map(Into::into)
				)
			},
		}
	}

	fn pending_receipt(&self, best_block: BlockNumber, hash: &H256) -> Option<RichReceipt> {
		self.from_pending_block(
			best_block,
			|| None,
			|pending| {
				let txs = pending.transactions();
				txs.iter()
					.map(|t| t.hash())
					.position(|t| t == *hash)
					.map(|index| {
						let prev_gas = if index == 0 { Default::default() } else { pending.receipts()[index - 1].gas_used };
						let tx = &txs[index];
						let receipt = &pending.receipts()[index];
						RichReceipt {
							transaction_hash: hash.clone(),
							transaction_index: index,
							cumulative_gas_used: receipt.gas_used,
							gas_used: receipt.gas_used - prev_gas,
							contract_address: match tx.action {
								Action::Call(_) => None,
								Action::Create => {
									let sender = tx.sender();
									Some(contract_address(self.engine.create_address_scheme(pending.header().number()), &sender, &tx.nonce, &tx.data).0)
								}
							},
							logs: receipt.logs.clone(),
							log_bloom: receipt.log_bloom,
							outcome: receipt.outcome.clone(),
						}
					})
			}
		)
	}

	fn pending_receipts(&self, best_block: BlockNumber) -> BTreeMap<H256, Receipt> {
		self.from_pending_block(
			best_block,
			BTreeMap::new,
			|pending| {
				let hashes = pending.transactions()
					.iter()
					.map(|t| t.hash());

				let receipts = pending.receipts().iter().cloned();

				hashes.zip(receipts).collect()
			}
		)
	}

	fn last_nonce(&self, address: &Address) -> Option<U256> {
		// TODO [ToDr] missing!
		unimplemented!()
	}

	fn can_produce_work_package(&self) -> bool {
		self.engine.seals_internally().is_none()
	}

	/// Update sealing if required.
	/// Prepare the block and work if the Engine does not seal internally.
	fn update_sealing(&self, chain: &MiningBlockChainClient) {
		trace!(target: "miner", "update_sealing");
		const NO_NEW_CHAIN_WITH_FORKS: &str = "Your chain specification contains one or more hard forks which are required to be \
			on by default. Please remove these forks and start your chain again.";

		if self.requires_reseal(chain.chain_info().best_block_number) {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing locks.          |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			trace!(target: "miner", "update_sealing: preparing a block");
			let (block, original_work_hash) = self.prepare_block(chain);

			// refuse to seal the first block of the chain if it contains hard forks
			// which should be on by default.
			if block.block().fields().header.number() == 1 && self.engine.params().contains_bugfix_hard_fork() {
				warn!("{}", NO_NEW_CHAIN_WITH_FORKS);
				return;
			}

			match self.engine.seals_internally() {
				Some(true) => {
					trace!(target: "miner", "update_sealing: engine indicates internal sealing");
					if self.seal_and_import_block_internally(chain, block) {
						trace!(target: "miner", "update_sealing: imported internally sealed block");
					}
				},
				Some(false) => trace!(target: "miner", "update_sealing: engine is not keen to seal internally right now"),
				None => {
					trace!(target: "miner", "update_sealing: engine does not seal internally, preparing work");
					self.prepare_work(block, original_work_hash)
				},
			}
		}
	}

	fn is_currently_sealing(&self) -> bool {
		self.sealing.lock().queue.is_in_use()
	}

	fn map_pending_block<F, T>(&self, chain: &MiningBlockChainClient, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		trace!(target: "miner", "map_sealing_work: entering");
		self.prepare_work_sealing(chain);
		trace!(target: "miner", "map_sealing_work: sealing prepared");
		let mut sealing = self.sealing.lock();
		let ret = sealing.queue.use_last_ref();
		trace!(target: "miner", "map_sealing_work: leaving use_last_ref={:?}", ret.as_ref().map(|b| b.block().fields().header.hash()));
		ret.map(f)
	}

	fn submit_seal(&self, chain: &MiningBlockChainClient, block_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		let result =
			if let Some(b) = self.sealing.lock().queue.get_used_if(
				if self.options.enable_resubmission {
					GetAction::Clone
				} else {
					GetAction::Take
				},
				|b| &b.hash() == &block_hash
			) {
				trace!(target: "miner", "Submitted block {}={}={} with seal {:?}", block_hash, b.hash(), b.header().bare_hash(), seal);
				b.lock().try_seal(&*self.engine, seal).or_else(|(e, _)| {
					warn!(target: "miner", "Mined solution rejected: {}", e);
					Err(Error::PowInvalid)
				})
			} else {
				warn!(target: "miner", "Submitted solution rejected: Block unknown or out of date.");
				Err(Error::PowHashInvalid)
			};
		result.and_then(|sealed| {
			let n = sealed.header().number();
			let h = sealed.header().hash();
			chain.import_sealed_block(sealed)?;
			info!(target: "miner", "Submitted block imported OK. #{}: {}", Colour::White.bold().paint(format!("{}", n)), Colour::White.bold().paint(format!("{:x}", h)));
			Ok(())
		})
	}

	fn chain_new_blocks(&self, chain: &MiningBlockChainClient, imported: &[H256], _invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		trace!(target: "miner", "chain_new_blocks");

		// 1. We ignore blocks that were `imported` unless resealing on new uncles is enabled.
		// 2. We ignore blocks that are `invalid` because it doesn't have any meaning in terms of the transactions that
		//    are in those blocks

		// First update gas limit in transaction queue
		self.update_gas_limit(chain);

		// Update minimal gas price
		self.recalibrate_minimal_gas_price();

		// Then import all transactions...
		{
			// TODO [ToDr] Parallelize
			for hash in retracted {
				let block = chain.block(BlockId::Hash(*hash))
					.expect("Client is sending message after commit to db and inserting to chain; the block is available; qed");
				let txs = block.transactions()
					.into_iter()
					.map(|transaction| pool::verifier::Transaction::Pending(transaction.into()))
					.collect();
				let _ = self.transaction_queue.import(
					chain,
					txs,
				);
			}
		}

		// ...and at the end remove the old ones
		let client = BlockChainClient { chain, engine: &*self.engine };
		self.transaction_queue.cull(client);

		if enacted.len() > 0 || (imported.len() > 0 && self.options.reseal_on_uncle) {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing locks.          |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			self.update_sealing(chain);
		}
	}
}

struct BlockChainClient<'a> {
	chain: &'a MiningBlockChainClient,
	engine: &'a EthEngine,
}

impl<'a> fmt::Debug for BlockChainClient<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "BlockChainClient")
	}
}

impl<'a> pool::client::Client for BlockChainClient<'a> {
	fn transaction_already_included(&self, hash: &H256) -> bool {
		self.chain.transaction_block(TransactionId::Hash(*hash)).is_some()
	}

	fn verify_transaction(&self, tx: UnverifiedTransaction)
		-> Result<SignedTransaction, transaction::Error>
	{
		let best_block_header = self.chain.best_block_header().decode();

		self.engine.verify_transaction_basic(&tx, &best_block_header)?;
		self.engine.verify_transaction_unordered(tx, &best_block_header)
	}

	fn account_details(&self, address: &Address) -> pool::client::AccountDetails {
		pool::client::AccountDetails {
			nonce: self.chain.latest_nonce(address),
			balance: self.chain.latest_balance(address),
		}
	}

	fn account_nonce(&self, address: &Address) -> U256 {
		self.chain.latest_nonce(address)
	}

	/// Estimate minimal gas requirurement for given transaction.
	fn required_gas(&self, tx: &SignedTransaction) -> U256 {
		tx.gas_required(&self.chain.latest_schedule()).into()
	}

	/// Classify transaction (check if transaction is filtered by some contracts).
	fn transaction_type(&self, tx: &SignedTransaction) -> pool::client::TransactionType {
		// TODO [ToDr] Transaction checker
		// self.service_transaction_action.check(self.client, tx)
		unimplemented!()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use ethcore_miner::transaction_queue::PrioritizationStrategy;
	use ethereum_types::U256;
	use ethkey::{Generator, Random};
	use hash::keccak;
	use rustc_hex::FromHex;
	use transaction::Transaction;

	use client::{BlockChainClient, TestBlockChainClient, EachBlockWith};
	use miner::MinerService;
	use tests::helpers::{generate_dummy_client, generate_dummy_client_with_spec_and_accounts};

	#[test]
	fn should_prepare_block_to_seal() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::with_spec(&Spec::new_test());

		// when
		let sealing_work = miner.map_sealing_work(&client, |_| ());
		assert!(sealing_work.is_some(), "Expected closed block");
	}

	#[test]
	fn should_still_work_after_a_couple_of_blocks() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::with_spec(&Spec::new_test());

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

	fn miner() -> Miner {
		Arc::try_unwrap(Miner::new(
			MinerOptions {
				new_work_notify: Vec::new(),
				force_sealing: false,
				reseal_on_external_tx: false,
				reseal_on_own_tx: true,
				reseal_on_uncle: false,
				reseal_min_period: Duration::from_secs(5),
				reseal_max_period: Duration::from_secs(120),
				tx_gas_limit: !U256::zero(),
				tx_queue_size: 1024,
				tx_queue_memory_limit: None,
				tx_queue_gas_limit: GasLimit::None,
				tx_queue_strategy: PrioritizationStrategy::GasFactorAndGasPrice,
				pending_set: PendingSet::AlwaysSealing,
				work_queue_size: 5,
				enable_resubmission: true,
				tx_queue_banning: Banning::Disabled,
				refuse_service_transactions: false,
				infinite_pending_block: false,
			},
			GasPricer::new_fixed(0u64.into()),
			&Spec::new_test(),
			None, // accounts provider
		)).ok().expect("Miner was just created.")
	}

	fn transaction() -> SignedTransaction {
		transaction_with_chain_id(2)
	}

	fn transaction_with_chain_id(chain_id: u64) -> SignedTransaction {
		let keypair = Random.generate().unwrap();
		Transaction {
			action: Action::Create,
			value: U256::zero(),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::zero(),
			nonce: U256::zero(),
		}.sign(keypair.secret(), Some(chain_id))
	}

	#[test]
	fn should_make_pending_block_when_importing_own_transaction() {
		// given
		let client = TestBlockChainClient::default();
		let miner = miner();
		let transaction = transaction();
		let best_block = 0;
		// when
		let res = miner.import_own_transaction(&client, PendingTransaction::new(transaction, None));

		// then
		assert_eq!(res.unwrap(), transaction::ImportResult::Current);
		assert_eq!(miner.pending_transactions().len(), 1);
		assert_eq!(miner.ready_transactions(best_block, 0).len(), 1);
		assert_eq!(miner.pending_transactions_hashes(best_block).len(), 1);
		assert_eq!(miner.pending_receipts(best_block).len(), 1);
		// This method will let us know if pending block was created (before calling that method)
		assert!(!miner.prepare_work_sealing(&client));
	}

	#[test]
	fn should_not_use_pending_block_if_best_block_is_higher() {
		// given
		let client = TestBlockChainClient::default();
		let miner = miner();
		let transaction = transaction();
		let best_block = 10;
		// when
		let res = miner.import_own_transaction(&client, PendingTransaction::new(transaction, None));

		// then
		assert_eq!(res.unwrap(), transaction::ImportResult::Current);
		assert_eq!(miner.pending_transactions().len(), 1);
		assert_eq!(miner.ready_transactions(best_block, 0).len(), 0);
		assert_eq!(miner.pending_transactions_hashes(best_block).len(), 0);
		assert_eq!(miner.pending_receipts(best_block).len(), 0);
	}

	#[test]
	fn should_import_external_transaction() {
		// given
		let client = TestBlockChainClient::default();
		let miner = miner();
		let transaction = transaction().into();
		let best_block = 0;
		// when
		let res = miner.import_external_transactions(&client, vec![transaction]).pop().unwrap();

		// then
		assert_eq!(res.unwrap(), transaction::ImportResult::Current);
		assert_eq!(miner.pending_transactions().len(), 1);
		assert_eq!(miner.pending_transactions_hashes(best_block).len(), 0);
		assert_eq!(miner.ready_transactions(best_block, 0).len(), 0);
		assert_eq!(miner.pending_receipts(best_block).len(), 0);
		// This method will let us know if pending block was created (before calling that method)
		assert!(miner.prepare_work_sealing(&client));
	}

	#[test]
	fn should_not_seal_unless_enabled() {
		let miner = miner();
		let client = TestBlockChainClient::default();
		// By default resealing is not required.
		assert!(!miner.requires_reseal(1u8.into()));

		miner.import_external_transactions(&client, vec![transaction().into()]).pop().unwrap().unwrap();
		assert!(miner.prepare_work_sealing(&client));
		// Unless asked to prepare work.
		assert!(miner.requires_reseal(1u8.into()));
	}

	#[test]
	fn internal_seals_without_work() {
		let spec = Spec::new_instant();
		let miner = Miner::with_spec(&spec);

		let client = generate_dummy_client(2);

		assert_eq!(miner.import_external_transactions(&*client, vec![transaction_with_chain_id(spec.chain_id()).into()]).pop().unwrap().unwrap(), transaction::ImportResult::Current);

		miner.update_sealing(&*client);
		client.flush_queue();
		assert!(miner.pending_block(0).is_none());
		assert_eq!(client.chain_info().best_block_number, 3 as BlockNumber);

		assert_eq!(miner.import_own_transaction(&*client, PendingTransaction::new(transaction_with_chain_id(spec.chain_id()).into(), None)).unwrap(), transaction::ImportResult::Current);

		miner.update_sealing(&*client);
		client.flush_queue();
		assert!(miner.pending_block(0).is_none());
		assert_eq!(client.chain_info().best_block_number, 4 as BlockNumber);
	}

	#[test]
	fn should_fail_setting_engine_signer_on_pow() {
		let spec = Spec::new_pow_test_spec;
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr = tap.insert_account(keccak("1").into(), "").unwrap();
		let client = generate_dummy_client_with_spec_and_accounts(spec, Some(tap.clone()));
		assert!(match client.miner().set_engine_signer(addr, "".into()) { Err(AccountError::InappropriateChain) => true, _ => false })
	}

	#[test]
	fn should_fail_setting_engine_signer_without_account_provider() {
		let spec = Spec::new_instant;
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr = tap.insert_account(keccak("1").into(), "").unwrap();
		let client = generate_dummy_client_with_spec_and_accounts(spec, None);
		assert!(match client.miner().set_engine_signer(addr, "".into()) { Err(AccountError::NotFound) => true, _ => false });
	}
}
