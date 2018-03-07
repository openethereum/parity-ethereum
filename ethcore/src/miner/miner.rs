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
use ethcore_miner::banning_queue::{BanningTransactionQueue, Threshold};
use ethcore_miner::local_transactions::{Status as LocalTransactionStatus};
use ethcore_miner::transaction_queue::{
	TransactionQueue,
	RemovalReason,
	TransactionDetailsProvider as TransactionQueueDetailsProvider,
	PrioritizationStrategy,
	AccountDetails,
	TransactionOrigin,
};
use futures_cpupool::CpuPool;
use ethcore_miner::work_notify::{WorkPoster, NotifyWork};
use miner::service_transaction_checker::ServiceTransactionChecker;
use miner::{MinerService, MinerStatus};
use price_info::fetch::Client as FetchClient;
use price_info::{Client as PriceInfoClient, PriceInfo};
use transaction::{
	Action,
	UnverifiedTransaction,
	PendingTransaction,
	SignedTransaction,
	Condition as TransactionCondition,
	ImportResult as TransactionImportResult,
	Error as TransactionError,
};
use using_queue::{UsingQueue, GetAction};
use block::{ClosedBlock, IsBlock, Block};
use client::{
	AccountData, BlockChain, RegistryInfo, ScheduleInfo, CallContract, BlockProducer, SealedBlockImporter
};
use client::{BlockId, TransactionId, MiningBlockChainClient};
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

/// Type of the gas limit to apply to the transaction queue.
#[derive(Debug, PartialEq)]
pub enum GasLimit {
	/// Depends on the block gas limit and is updated with every block.
	Auto,
	/// No limit.
	None,
	/// Set to a fixed gas value.
	Fixed(U256),
}

/// Transaction queue banning settings.
#[derive(Debug, PartialEq, Clone)]
pub enum Banning {
	/// Banning in transaction queue is disabled
	Disabled,
	/// Banning in transaction queue is enabled
	Enabled {
		/// Upper limit of transaction processing time before banning.
		offend_threshold: Duration,
		/// Number of similar offending transactions before banning.
		min_offends: u16,
		/// Number of seconds the offender is banned for.
		ban_duration: Duration,
	},
}

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
	/// Maximum amount of gas to bother considering for block insertion.
	pub tx_gas_limit: U256,
	/// Maximum size of the transaction queue.
	pub tx_queue_size: usize,
	/// Maximum memory usage of transactions in the queue (current / future).
	pub tx_queue_memory_limit: Option<usize>,
	/// Strategy to use for prioritizing transactions in the queue.
	pub tx_queue_strategy: PrioritizationStrategy,
	/// Whether we should fallback to providing all the queue's transactions or just pending.
	pub pending_set: PendingSet,
	/// How many historical work packages can we store before running out?
	pub work_queue_size: usize,
	/// Can we submit two different solutions for the same block and expect both to result in an import?
	pub enable_resubmission: bool,
	/// Global gas limit for all transaction in the queue except for local and retracted.
	pub tx_queue_gas_limit: GasLimit,
	/// Banning settings.
	pub tx_queue_banning: Banning,
	/// Do we refuse to accept service transactions even if sender is certified.
	pub refuse_service_transactions: bool,
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
			tx_gas_limit: !U256::zero(),
			tx_queue_size: 8192,
			tx_queue_memory_limit: Some(2 * 1024 * 1024),
			tx_queue_gas_limit: GasLimit::None,
			tx_queue_strategy: PrioritizationStrategy::GasPriceOnly,
			pending_set: PendingSet::AlwaysQueue,
			reseal_min_period: Duration::from_secs(2),
			reseal_max_period: Duration::from_secs(120),
			work_queue_size: 20,
			enable_resubmission: true,
			tx_queue_banning: Banning::Disabled,
			refuse_service_transactions: false,
			infinite_pending_block: false,
		}
	}
}

/// Options for the dynamic gas price recalibrator.
#[derive(Debug, PartialEq)]
pub struct GasPriceCalibratorOptions {
	/// Base transaction price to match against.
	pub usd_per_tx: f32,
	/// How frequently we should recalibrate.
	pub recalibration_period: Duration,
}

/// The gas price validator variant for a `GasPricer`.
#[derive(Debug, PartialEq)]
pub struct GasPriceCalibrator {
	options: GasPriceCalibratorOptions,
	next_calibration: Instant,
	price_info: PriceInfoClient,
}

impl GasPriceCalibrator {
	fn recalibrate<F: Fn(U256) + Sync + Send + 'static>(&mut self, set_price: F) {
		trace!(target: "miner", "Recalibrating {:?} versus {:?}", Instant::now(), self.next_calibration);
		if Instant::now() >= self.next_calibration {
			let usd_per_tx = self.options.usd_per_tx;
			trace!(target: "miner", "Getting price info");

			self.price_info.get(move |price: PriceInfo| {
				trace!(target: "miner", "Price info arrived: {:?}", price);
				let usd_per_eth = price.ethusd;
				let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
				let gas_per_tx: f32 = 21000.0;
				let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
				info!(target: "miner", "Updated conversion rate to Ξ1 = {} ({} wei/gas)", Colour::White.bold().paint(format!("US${:.2}", usd_per_eth)), Colour::Yellow.bold().paint(format!("{}", wei_per_gas)));
				set_price(U256::from(wei_per_gas as u64));
			});

			self.next_calibration = Instant::now() + self.options.recalibration_period;
		}
	}
}

/// Struct to look after updating the acceptable gas price of a miner.
#[derive(Debug, PartialEq)]
pub enum GasPricer {
	/// A fixed gas price in terms of Wei - always the argument given.
	Fixed(U256),
	/// Gas price is calibrated according to a fixed amount of USD.
	Calibrated(GasPriceCalibrator),
}

impl GasPricer {
	/// Create a new Calibrated `GasPricer`.
	pub fn new_calibrated(options: GasPriceCalibratorOptions, fetch: FetchClient, p: CpuPool) -> GasPricer {
		GasPricer::Calibrated(GasPriceCalibrator {
			options: options,
			next_calibration: Instant::now(),
			price_info: PriceInfoClient::new(fetch, p),
		})
	}

	/// Create a new Fixed `GasPricer`.
	pub fn new_fixed(gas_price: U256) -> GasPricer {
		GasPricer::Fixed(gas_price)
	}

	fn recalibrate<F: Fn(U256) + Sync + Send + 'static>(&mut self, set_price: F) {
		match *self {
			GasPricer::Fixed(ref max) => set_price(max.clone()),
			GasPricer::Calibrated(ref mut cal) => cal.recalibrate(set_price),
		}
	}
}

struct SealingWork {
	queue: UsingQueue<ClosedBlock>,
	enabled: bool,
}

/// Keeps track of transactions using priority queue and holds currently mined block.
/// Handles preparing work for "work sealing" or seals "internally" if Engine does not require work.
pub struct Miner {
	// NOTE [ToDr]  When locking always lock in this order!
	transaction_queue: Arc<RwLock<BanningTransactionQueue>>,
	transaction_listener: RwLock<Vec<Box<Fn(&[H256]) + Send + Sync>>>,
	sealing_work: Mutex<SealingWork>,
	next_allowed_reseal: Mutex<Instant>,
	next_mandatory_reseal: RwLock<Instant>,
	sealing_block_last_request: Mutex<u64>,
	// for sealing...
	options: MinerOptions,

	gas_range_target: RwLock<(U256, U256)>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,
	engine: Arc<EthEngine>,

	accounts: Option<Arc<AccountProvider>>,
	notifiers: RwLock<Vec<Box<NotifyWork>>>,
	gas_pricer: Mutex<GasPricer>,
	service_transaction_action: ServiceTransactionAction,
}

impl Miner {
	/// Push notifier that will handle new jobs
	pub fn push_notifier(&self, notifier: Box<NotifyWork>) {
		self.notifiers.write().push(notifier);
		self.sealing_work.lock().enabled = true;
	}

	/// Creates new instance of miner Arc.
	pub fn new(options: MinerOptions, gas_pricer: GasPricer, spec: &Spec, accounts: Option<Arc<AccountProvider>>) -> Arc<Miner> {
		Arc::new(Miner::new_raw(options, gas_pricer, spec, accounts))
	}

	/// Creates new instance of miner.
	fn new_raw(options: MinerOptions, gas_pricer: GasPricer, spec: &Spec, accounts: Option<Arc<AccountProvider>>) -> Miner {
		let gas_limit = match options.tx_queue_gas_limit {
			GasLimit::Fixed(ref limit) => *limit,
			_ => !U256::zero(),
		};
		let mem_limit = options.tx_queue_memory_limit.unwrap_or_else(usize::max_value);

		let txq = TransactionQueue::with_limits(
			options.tx_queue_strategy,
			options.tx_queue_size,
			mem_limit,
			gas_limit,
			options.tx_gas_limit
		);
		let txq = match options.tx_queue_banning {
			Banning::Disabled => BanningTransactionQueue::new(txq, Threshold::NeverBan, Duration::from_secs(180)),
			Banning::Enabled { ban_duration, min_offends, .. } => BanningTransactionQueue::new(
				txq,
				Threshold::BanAfter(min_offends),
				ban_duration,
			),
		};

		let notifiers: Vec<Box<NotifyWork>> = match options.new_work_notify.is_empty() {
			true => Vec::new(),
			false => vec![Box::new(WorkPoster::new(&options.new_work_notify))],
		};

		let service_transaction_action = match options.refuse_service_transactions {
			true => ServiceTransactionAction::Refuse,
			false => ServiceTransactionAction::Check(ServiceTransactionChecker::default()),
		};

		Miner {
			transaction_queue: Arc::new(RwLock::new(txq)),
			transaction_listener: RwLock::new(vec![]),
			next_allowed_reseal: Mutex::new(Instant::now()),
			next_mandatory_reseal: RwLock::new(Instant::now() + options.reseal_max_period),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(SealingWork{
				queue: UsingQueue::new(options.work_queue_size),
				enabled: options.force_sealing
					|| !options.new_work_notify.is_empty()
					|| spec.engine.seals_internally().is_some()
			}),
			gas_range_target: RwLock::new((U256::zero(), U256::zero())),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			options: options,
			accounts: accounts,
			engine: spec.engine.clone(),
			notifiers: RwLock::new(notifiers),
			gas_pricer: Mutex::new(gas_pricer),
			service_transaction_action: service_transaction_action,
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
		self.sealing_work.lock().queue.reset();
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

	/// Set a callback to be notified about imported transactions' hashes.
	pub fn add_transactions_listener(&self, f: Box<Fn(&[H256]) + Send + Sync>) {
		self.transaction_listener.write().push(f);
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
	fn prepare_block<C: AccountData + BlockChain + BlockProducer + CallContract>(&self, chain: &C) -> (ClosedBlock, Option<H256>) {
		trace_time!("prepare_block");
		let chain_info = chain.chain_info();
		let (transactions, mut open_block, original_work_hash) = {
			let nonce_cap = if chain_info.best_block_number + 1 >= self.engine.params().dust_protection_transition {
				Some((self.engine.params().nonce_cap_increment * (chain_info.best_block_number + 1)).into())
			} else { None };
			let transactions = {self.transaction_queue.read().top_transactions_at(chain_info.best_block_number, chain_info.best_block_timestamp, nonce_cap)};
			let mut sealing_work = self.sealing_work.lock();
			let last_work_hash = sealing_work.queue.peek_last_ref().map(|pb| pb.block().header().hash());
			let best_hash = chain_info.best_block_hash;

			// check to see if last ClosedBlock in would_seals is actually same parent block.
			// if so
			//   duplicate, re-open and push any new transactions.
			//   if at least one was pushed successfully, close and enqueue new ClosedBlock;
			//   otherwise, leave everything alone.
			// otherwise, author a fresh block.
			let mut open_block = match sealing_work.queue.pop_if(|b| b.block().header().parent_hash() == &best_hash) {
				Some(old_block) => {
					trace!(target: "miner", "prepare_block: Already have previous work; updating and returning");
					// add transactions to old_block
					chain.reopen_block(old_block)
				}
				None => {
					// block not found - create it.
					trace!(target: "miner", "prepare_block: No existing work - making new block");
					chain.prepare_open_block(
						self.author(),
						(self.gas_floor_target(), self.gas_ceil_target()),
						self.extra_data()
					)
				}
			};

			if self.options.infinite_pending_block {
				open_block.remove_gas_limit();
			}

			(transactions, open_block, last_work_hash)
		};

		let mut invalid_transactions = HashSet::new();
		let mut non_allowed_transactions = HashSet::new();
		let mut transactions_to_penalize = HashSet::new();
		let block_number = open_block.block().header().number();

		let mut tx_count: usize = 0;
		let tx_total = transactions.len();
		for tx in transactions {
			let hash = tx.hash();
			let start = Instant::now();
			// Check whether transaction type is allowed for sender
			let result = match self.engine.machine().verify_transaction(&tx, open_block.header(), chain) {
				Err(Error::Transaction(TransactionError::NotAllowed)) => {
					Err(TransactionError::NotAllowed.into())
				}
				_ => {
					open_block.push_transaction(tx, None)
				}
			};
			let took = start.elapsed();

			// Check for heavy transactions
			match self.options.tx_queue_banning {
				Banning::Enabled { ref offend_threshold, .. } if &took > offend_threshold => {
					match self.transaction_queue.write().ban_transaction(&hash) {
						true => {
							warn!(target: "miner", "Detected heavy transaction. Banning the sender and recipient/code.");
						},
						false => {
							transactions_to_penalize.insert(hash);
							debug!(target: "miner", "Detected heavy transaction. Penalizing sender.")
						}
					}
				},
				_ => {},
			}
			trace!(target: "miner", "Adding tx {:?} took {:?}", hash, took);
			match result {
				Err(Error::Execution(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, gas })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of gas limit: {:?} (limit: {:?}, used: {:?}, gas: {:?})", hash, gas_limit, gas_used, gas);

					// Penalize transaction if it's above current gas limit
					if gas > gas_limit {
						transactions_to_penalize.insert(hash);
					}

					// Exit early if gas left is smaller then min_tx_gas
					let min_tx_gas: U256 = 21000.into();	// TODO: figure this out properly.
					if gas_limit - gas_used < min_tx_gas {
						break;
					}
				},
				// Invalid nonce error can happen only if previous transaction is skipped because of gas limit.
				// If there is errornous state of transaction queue it will be fixed when next block is imported.
				Err(Error::Execution(ExecutionError::InvalidNonce { expected, got })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of invalid nonce: {:?} (expected: {:?}, got: {:?})", hash, expected, got);
				},
				// already have transaction - ignore
				Err(Error::Transaction(TransactionError::AlreadyImported)) => {},
				Err(Error::Transaction(TransactionError::NotAllowed)) => {
					non_allowed_transactions.insert(hash);
					debug!(target: "miner",
						   "Skipping non-allowed transaction for sender {:?}",
						   hash);
				},
				Err(e) => {
					invalid_transactions.insert(hash);
					debug!(target: "miner",
						   "Error adding transaction to block: number={}. transaction_hash={:?}, Error: {:?}",
						   block_number, hash, e);
				},
				_ => {
					tx_count += 1;
				}	// imported ok
			}
		}
		trace!(target: "miner", "Pushed {}/{} transactions", tx_count, tx_total);

		let block = open_block.close();

		let fetch_nonce = |a: &Address| chain.latest_nonce(a);

		{
			let mut queue = self.transaction_queue.write();
			for hash in invalid_transactions {
				queue.remove(&hash, &fetch_nonce, RemovalReason::Invalid);
			}
			for hash in non_allowed_transactions {
				queue.remove(&hash, &fetch_nonce, RemovalReason::NotAllowed);
			}
			for hash in transactions_to_penalize {
				queue.penalize(&hash);
			}
		}
		(block, original_work_hash)
	}

	/// Asynchronously updates minimal gas price for transaction queue
	pub fn recalibrate_minimal_gas_price(&self) {
		debug!(target: "miner", "minimal_gas_price: recalibrating...");
		let txq = self.transaction_queue.clone();
		self.gas_pricer.lock().recalibrate(move |price| {
			debug!(target: "miner", "minimal_gas_price: Got gas price! {}", price);
			txq.write().set_minimal_gas_price(price);
		});
	}

	/// Check is reseal is allowed and necessary.
	fn requires_reseal(&self, best_block: BlockNumber) -> bool {
		let has_local_transactions = self.transaction_queue.read().has_local_pending_transactions();
		let mut sealing_work = self.sealing_work.lock();
		if sealing_work.enabled {
			trace!(target: "miner", "requires_reseal: sealing enabled");
			let last_request = *self.sealing_block_last_request.lock();
			let should_disable_sealing = !self.forced_sealing()
				&& !has_local_transactions
				&& self.engine.seals_internally().is_none()
				&& best_block > last_request
				&& best_block - last_request > SEALING_TIMEOUT_IN_BLOCKS;

			trace!(target: "miner", "requires_reseal: should_disable_sealing={}; best_block={}, last_request={}", should_disable_sealing, best_block, last_request);

			if should_disable_sealing {
				trace!(target: "miner", "Miner sleeping (current {}, last {})", best_block, last_request);
				sealing_work.enabled = false;
				sealing_work.queue.reset();
				false
			} else {
				// sealing enabled and we don't want to sleep.
				*self.next_allowed_reseal.lock() = Instant::now() + self.options.reseal_min_period;
				true
			}
		} else {
			trace!(target: "miner", "requires_reseal: sealing is disabled");
			false
		}
	}

	/// Attempts to perform internal sealing (one that does not require work) and handles the result depending on the type of Seal.
	fn seal_and_import_block_internally<C>(&self, chain: &C, block: ClosedBlock) -> bool
		where C: BlockChain + SealedBlockImporter
	{
		if !block.transactions().is_empty() || self.forced_sealing() || Instant::now() > *self.next_mandatory_reseal.read() {
			trace!(target: "miner", "seal_block_internally: attempting internal seal.");

			let parent_header = match chain.block_header(BlockId::Hash(*block.header().parent_hash())) {
				Some(hdr) => hdr.decode(),
				None => return false,
			};

			match self.engine.generate_seal(block.block(), &parent_header) {
				// Save proposal for later seal submission and broadcast it.
				Seal::Proposal(seal) => {
					trace!(target: "miner", "Received a Proposal seal.");
					*self.next_mandatory_reseal.write() = Instant::now() + self.options.reseal_max_period;
					{
						let mut sealing_work = self.sealing_work.lock();
						sealing_work.queue.push(block.clone());
						sealing_work.queue.use_last_ref();
					}
					block
						.lock()
						.seal(&*self.engine, seal)
						.map(|sealed| { chain.broadcast_proposal_block(sealed); true })
						.unwrap_or_else(|e| {
							warn!("ERROR: seal failed when given internally generated seal: {}", e);
							false
						})
				},
				// Directly import a regular sealed block.
				Seal::Regular(seal) => {
					*self.next_mandatory_reseal.write() = Instant::now() + self.options.reseal_max_period;
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
		} else {
			false
		}
	}

	/// Prepares work which has to be done to seal.
	fn prepare_work(&self, block: ClosedBlock, original_work_hash: Option<H256>) {
		let (work, is_new) = {
			let mut sealing_work = self.sealing_work.lock();
			let last_work_hash = sealing_work.queue.peek_last_ref().map(|pb| pb.block().header().hash());
			trace!(target: "miner", "prepare_work: Checking whether we need to reseal: orig={:?} last={:?}, this={:?}", original_work_hash, last_work_hash, block.block().header().hash());
			let (work, is_new) = if last_work_hash.map_or(true, |h| h != block.block().header().hash()) {
				trace!(target: "miner", "prepare_work: Pushing a new, refreshed or borrowed pending {}...", block.block().header().hash());
				let pow_hash = block.block().header().hash();
				let number = block.block().header().number();
				let difficulty = *block.block().header().difficulty();
				let is_new = original_work_hash.map_or(true, |h| block.block().header().hash() != h);
				sealing_work.queue.push(block);
				// If push notifications are enabled we assume all work items are used.
				if !self.notifiers.read().is_empty() && is_new {
					sealing_work.queue.use_last_ref();
				}
				(Some((pow_hash, difficulty, number)), is_new)
			} else {
				(None, false)
			};
			trace!(target: "miner", "prepare_work: leaving (last={:?})", sealing_work.queue.peek_last_ref().map(|b| b.block().header().hash()));
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

	fn update_gas_limit<C: BlockChain>(&self, client: &C) {
		let gas_limit = client.best_block_header().gas_limit();
		let mut queue = self.transaction_queue.write();
		queue.set_gas_limit(gas_limit);
		if let GasLimit::Auto = self.options.tx_queue_gas_limit {
			// Set total tx queue gas limit to be 20x the block gas limit.
			queue.set_total_gas_limit(gas_limit * 20u32);
		}
	}

	/// Returns true if we had to prepare new pending block.
	fn prepare_work_sealing<C: AccountData + BlockChain + BlockProducer + CallContract>(&self, client: &C) -> bool {
		trace!(target: "miner", "prepare_work_sealing: entering");
		let prepare_new = {
			let mut sealing_work = self.sealing_work.lock();
			let have_work = sealing_work.queue.peek_last_ref().is_some();
			trace!(target: "miner", "prepare_work_sealing: have_work={}", have_work);
			if !have_work {
				sealing_work.enabled = true;
				true
			} else {
				false
			}
		};
		if prepare_new {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing_work locks.     |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			let (block, original_work_hash) = self.prepare_block(client);
			self.prepare_work(block, original_work_hash);
		}
		let mut sealing_block_last_request = self.sealing_block_last_request.lock();
		let best_number = client.chain_info().best_block_number;
		if *sealing_block_last_request != best_number {
			trace!(target: "miner", "prepare_work_sealing: Miner received request (was {}, now {}) - waking up.", *sealing_block_last_request, best_number);
			*sealing_block_last_request = best_number;
		}

		// Return if we restarted
		prepare_new
	}

	fn add_transactions_to_queue<C: AccountData + BlockChain + CallContract + RegistryInfo + ScheduleInfo>(
		&self,
		client: &C,
		transactions: Vec<UnverifiedTransaction>,
		default_origin: TransactionOrigin,
		condition: Option<TransactionCondition>,
		transaction_queue: &mut BanningTransactionQueue,
	) -> Vec<Result<TransactionImportResult, Error>> {
		let best_block_header = client.best_block_header().decode();
		let insertion_time = client.chain_info().best_block_number;
		let mut inserted = Vec::with_capacity(transactions.len());

		let results = transactions.into_iter()
			.map(|tx| {
				let hash = tx.hash();
				if client.transaction_block(TransactionId::Hash(hash)).is_some() {
					debug!(target: "miner", "Rejected tx {:?}: already in the blockchain", hash);
					return Err(Error::Transaction(TransactionError::AlreadyImported));
				}
				match self.engine.verify_transaction_basic(&tx, &best_block_header)
					.and_then(|_| self.engine.verify_transaction_unordered(tx, &best_block_header))
				{
					Err(e) => {
						debug!(target: "miner", "Rejected tx {:?} with invalid signature: {:?}", hash, e);
						Err(e)
					},
					Ok(transaction) => {
						// This check goes here because verify_transaction takes SignedTransaction parameter
						self.engine.machine().verify_transaction(&transaction, &best_block_header, client)?;

						let origin = self.accounts.as_ref().and_then(|accounts| {
							match accounts.has_account(transaction.sender()).unwrap_or(false) {
								true => Some(TransactionOrigin::Local),
								false => None,
							}
						}).unwrap_or(default_origin);

						let details_provider = TransactionDetailsProvider::new(client, &self.service_transaction_action);
						let hash = transaction.hash();
						let result = match origin {
							TransactionOrigin::Local | TransactionOrigin::RetractedBlock => {
								transaction_queue.add(transaction, origin, insertion_time, condition.clone(), &details_provider)?
							},
							TransactionOrigin::External => {
								transaction_queue.add_with_banlist(transaction, insertion_time, &details_provider)?
							},
						};

						inserted.push(hash);
						Ok(result)
					},
				}
			})
			.collect();

		for listener in &*self.transaction_listener.read() {
			listener(&inserted);
		}

		results
	}

	/// Are we allowed to do a non-mandatory reseal?
	fn tx_reseal_allowed(&self) -> bool { Instant::now() > *self.next_allowed_reseal.lock() }

	fn from_pending_block<H, F, G>(&self, latest_block_number: BlockNumber, from_chain: F, map_block: G) -> H
		where F: Fn() -> H, G: FnOnce(&ClosedBlock) -> H {
		let sealing_work = self.sealing_work.lock();
		sealing_work.queue.peek_last_ref().map_or_else(
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
	type State = State<::state_db::StateDB>;

	fn clear_and_reset<C: MiningBlockChainClient>(&self, chain: &C) {
		self.transaction_queue.write().clear();
		// --------------------------------------------------------------------------
		// | NOTE Code below requires transaction_queue and sealing_work locks.     |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
		self.update_sealing(chain);
	}

	fn status(&self) -> MinerStatus {
		let status = self.transaction_queue.read().status();
		let sealing_work = self.sealing_work.lock();
		MinerStatus {
			transactions_in_pending_queue: status.pending,
			transactions_in_future_queue: status.future,
			transactions_in_pending_block: sealing_work.queue.peek_last_ref().map_or(0, |b| b.transactions().len()),
		}
	}

	fn set_author(&self, author: Address) {
		if self.engine.seals_internally().is_some() {
			let mut sealing_work = self.sealing_work.lock();
			sealing_work.enabled = true;
		}
		*self.author.write() = author;
	}

	fn set_engine_signer(&self, address: Address, password: String) -> Result<(), AccountError> {
		if self.engine.seals_internally().is_some() {
			if let Some(ref ap) = self.accounts {
				ap.sign(address.clone(), Some(password.clone()), Default::default())?;
				// Limit the scope of the locks.
				{
					let mut sealing_work = self.sealing_work.lock();
					sealing_work.enabled = true;
					*self.author.write() = address;
				}
				// --------------------------------------------------------------------------
				// | NOTE Code below may require author and sealing_work locks              |
				// | (some `Engine`s call `EngineClient.update_sealing()`)                  |.
				// | Make sure to release the locks before calling that method.             |
				// --------------------------------------------------------------------------
				self.engine.set_signer(ap.clone(), address, password);
				Ok(())
			} else {
				warn!(target: "miner", "No account provider");
				Err(AccountError::NotFound)
			}
		} else {
			warn!(target: "miner", "Cannot set engine signer on a PoW chain.");
			Err(AccountError::InappropriateChain)
		}
	}

	fn set_extra_data(&self, extra_data: Bytes) {
		*self.extra_data.write() = extra_data;
	}

	/// Set the gas limit we wish to target when sealing a new block.
	fn set_gas_floor_target(&self, target: U256) {
		self.gas_range_target.write().0 = target;
	}

	fn set_gas_ceil_target(&self, target: U256) {
		self.gas_range_target.write().1 = target;
	}

	fn set_minimal_gas_price(&self, min_gas_price: U256) {
		self.transaction_queue.write().set_minimal_gas_price(min_gas_price);
	}

	fn minimal_gas_price(&self) -> U256 {
		*self.transaction_queue.read().minimal_gas_price()
	}

	fn sensible_gas_price(&self) -> U256 {
		// 10% above our minimum.
		*self.transaction_queue.read().minimal_gas_price() * 110u32 / 100.into()
	}

	fn sensible_gas_limit(&self) -> U256 {
		self.gas_range_target.read().0 / 5.into()
	}

	fn transactions_limit(&self) -> usize {
		self.transaction_queue.read().limit()
	}

	fn set_transactions_limit(&self, limit: usize) {
		self.transaction_queue.write().set_limit(limit)
	}

	fn set_tx_gas_limit(&self, limit: U256) {
		self.transaction_queue.write().set_tx_gas_limit(limit)
	}

	/// Get the author that we will seal blocks as.
	fn author(&self) -> Address {
		*self.author.read()
	}

	/// Get the extra_data that we will seal blocks with.
	fn extra_data(&self) -> Bytes {
		self.extra_data.read().clone()
	}

	/// Get the gas limit we wish to target when sealing a new block.
	fn gas_floor_target(&self) -> U256 {
		self.gas_range_target.read().0
	}

	/// Get the gas limit we wish to target when sealing a new block.
	fn gas_ceil_target(&self) -> U256 {
		self.gas_range_target.read().1
	}

	fn import_external_transactions<C: MiningBlockChainClient>(
		&self,
		client: &C,
		transactions: Vec<UnverifiedTransaction>
	) -> Vec<Result<TransactionImportResult, Error>> {
		trace!(target: "external_tx", "Importing external transactions");
		let results = {
			let mut transaction_queue = self.transaction_queue.write();
			self.add_transactions_to_queue(
				client, transactions, TransactionOrigin::External, None, &mut transaction_queue
			)
		};

		if !results.is_empty() && self.options.reseal_on_external_tx &&	self.tx_reseal_allowed() {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing_work locks.     |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			self.update_sealing(client);
		}
		results
	}

	fn import_own_transaction<C: MiningBlockChainClient>(
		&self,
		chain: &C,
		pending: PendingTransaction,
	) -> Result<TransactionImportResult, Error> {

		trace!(target: "own_tx", "Importing transaction: {:?}", pending);

		let imported = {
			// Be sure to release the lock before we call prepare_work_sealing
			let mut transaction_queue = self.transaction_queue.write();
			// We need to re-validate transactions
			let import = self.add_transactions_to_queue(
				chain, vec![pending.transaction.into()], TransactionOrigin::Local, pending.condition, &mut transaction_queue
			).pop().expect("one result returned per added transaction; one added => one result; qed");

			match import {
				Ok(_) => {
					trace!(target: "own_tx", "Status: {:?}", transaction_queue.status());
				},
				Err(ref e) => {
					trace!(target: "own_tx", "Status: {:?}", transaction_queue.status());
					warn!(target: "own_tx", "Error importing transaction: {:?}", e);
				},
			}
			import
		};

		// --------------------------------------------------------------------------
		// | NOTE Code below requires transaction_queue and sealing_work locks.     |
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

	fn local_transactions(&self) -> BTreeMap<H256, LocalTransactionStatus> {
		let queue = self.transaction_queue.read();
		queue.local_transactions()
			.iter()
			.map(|(hash, status)| (*hash, status.clone()))
			.collect()
	}

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

	fn remove_pending_transaction<C: AccountData>(&self, chain: &C, hash: &H256) -> Option<PendingTransaction> {
		let mut queue = self.transaction_queue.write();
		let tx = queue.find(hash);
		if tx.is_some() {
			let fetch_nonce = |a: &Address| chain.latest_nonce(a);
			queue.remove(hash, &fetch_nonce, RemovalReason::Canceled);
		}
		tx
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
		self.transaction_queue.read().last_nonce(address)
	}

	fn can_produce_work_package(&self) -> bool {
		self.engine.seals_internally().is_none()
	}

	/// Update sealing if required.
	/// Prepare the block and work if the Engine does not seal internally.
	fn update_sealing<C>(&self, chain: &C)
		where C: AccountData + BlockChain + RegistryInfo
		         + CallContract + BlockProducer + SealedBlockImporter
	{
		trace!(target: "miner", "update_sealing");
		const NO_NEW_CHAIN_WITH_FORKS: &str = "Your chain specification contains one or more hard forks which are required to be \
			on by default. Please remove these forks and start your chain again.";

		if self.requires_reseal(chain.chain_info().best_block_number) {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing_work locks.     |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			trace!(target: "miner", "update_sealing: preparing a block");
			let (block, original_work_hash) = self.prepare_block(chain);

			// refuse to seal the first block of the chain if it contains hard forks
			// which should be on by default.
			if block.block().header().number() == 1 && self.engine.params().contains_bugfix_hard_fork() {
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
		self.sealing_work.lock().queue.is_in_use()
	}

	fn map_sealing_work<C, F, T>(&self, client: &C, f: F) -> Option<T>
		where C: AccountData + BlockChain + BlockProducer + CallContract,
		      F: FnOnce(&ClosedBlock) -> T
	{
		trace!(target: "miner", "map_sealing_work: entering");
		self.prepare_work_sealing(client);
		trace!(target: "miner", "map_sealing_work: sealing prepared");
		let mut sealing_work = self.sealing_work.lock();
		let ret = sealing_work.queue.use_last_ref();
		trace!(target: "miner", "map_sealing_work: leaving use_last_ref={:?}", ret.as_ref().map(|b| b.block().header().hash()));
		ret.map(f)
	}

	fn submit_seal<C: SealedBlockImporter>(&self, chain: &C, block_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		let result =
			if let Some(b) = self.sealing_work.lock().queue.get_used_if(
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

	fn chain_new_blocks<C>(&self, chain: &C, imported: &[H256], _invalid: &[H256], enacted: &[H256], retracted: &[H256])
		where C: AccountData + BlockChain + CallContract + RegistryInfo
		         + BlockProducer + ScheduleInfo + SealedBlockImporter
	{
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

			let mut transaction_queue = self.transaction_queue.write();
			for hash in retracted {
				let block = chain.block(BlockId::Hash(*hash))
					.expect("Client is sending message after commit to db and inserting to chain; the block is available; qed");
				let txs = block.transactions();
				let _ = self.add_transactions_to_queue(
					chain, txs, TransactionOrigin::RetractedBlock, None, &mut transaction_queue
				);
			}
		}

		// ...and at the end remove the old ones
		{
			let fetch_account = |a: &Address| AccountDetails {
				nonce: chain.latest_nonce(a),
				balance: chain.latest_balance(a),
			};
			let time = chain.chain_info().best_block_number;
			let mut transaction_queue = self.transaction_queue.write();
			transaction_queue.remove_old(&fetch_account, time);
		}

		if enacted.len() > 0 || (imported.len() > 0 && self.options.reseal_on_uncle) {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing_work locks.     |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			self.update_sealing(chain);
		}
	}

	fn pending_state(&self, latest_block_number: BlockNumber) -> Option<Self::State> {
		Miner::pending_state(self, latest_block_number)
	}

	fn pending_block_header(&self, latest_block_number: BlockNumber) -> Option<Header> {
		Miner::pending_block_header(self, latest_block_number)
	}

	fn pending_block(&self, latest_block_number: BlockNumber) -> Option<Block> {
		Miner::pending_block(self, latest_block_number)
	}
}

/// Action when service transaction is received
enum ServiceTransactionAction {
	/// Refuse service transaction immediately
	Refuse,
	/// Accept if sender is certified to send service transactions
	Check(ServiceTransactionChecker),
}

impl ServiceTransactionAction {
	pub fn check<C: CallContract + RegistryInfo>(&self, client: &C, tx: &SignedTransaction) -> Result<bool, String>
	{
		match *self {
			ServiceTransactionAction::Refuse => Err("configured to refuse service transactions".to_owned()),
			ServiceTransactionAction::Check(ref checker) => checker.check(client, tx),
		}
	}
}

struct TransactionDetailsProvider<'a, C: 'a> {
	client: &'a C,
	service_transaction_action: &'a ServiceTransactionAction,
}

impl<'a, C> TransactionDetailsProvider<'a, C> {
	pub fn new(client: &'a C, service_transaction_action: &'a ServiceTransactionAction) -> Self {
		TransactionDetailsProvider {
			client: client,
			service_transaction_action: service_transaction_action,
		}
	}
}

impl<'a, C> TransactionQueueDetailsProvider for TransactionDetailsProvider<'a, C>
	where C: AccountData + CallContract + RegistryInfo + ScheduleInfo
{
	fn fetch_account(&self, address: &Address) -> AccountDetails {
		AccountDetails {
			nonce: self.client.latest_nonce(address),
			balance: self.client.latest_balance(address),
		}
	}

	fn estimate_gas_required(&self, tx: &SignedTransaction) -> U256 {
		tx.gas_required(&self.client.latest_schedule()).into()
	}

	fn is_service_transaction_acceptable(&self, tx: &SignedTransaction) -> Result<bool, String> {
		self.service_transaction_action.check(self.client, tx)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use ethcore_miner::transaction_queue::PrioritizationStrategy;
	use ethereum_types::U256;
	use ethkey::{Generator, Random};
	use client::{TestBlockChainClient, EachBlockWith, ChainInfo};
	use hash::keccak;
	use header::BlockNumber;
	use rustc_hex::FromHex;
	use spec::Spec;
	use transaction::{SignedTransaction, Transaction, PendingTransaction, Action};
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

		let res = miner.map_sealing_work(&client, |b| b.block().header().hash());
		assert!(res.is_some());
		assert!(miner.submit_seal(&client, res.unwrap(), vec![]).is_ok());

		// two more blocks mined, work requested.
		client.add_blocks(1, EachBlockWith::Uncle);
		miner.map_sealing_work(&client, |b| b.block().header().hash());

		client.add_blocks(1, EachBlockWith::Uncle);
		miner.map_sealing_work(&client, |b| b.block().header().hash());

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
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
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
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
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
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
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

		assert_eq!(miner.import_external_transactions(&*client, vec![transaction_with_chain_id(spec.chain_id()).into()]).pop().unwrap().unwrap(), TransactionImportResult::Current);

		miner.update_sealing(&*client);
		client.flush_queue();
		assert!(miner.pending_block(0).is_none());
		assert_eq!(client.chain_info().best_block_number, 3 as BlockNumber);

		assert_eq!(miner.import_own_transaction(&*client, PendingTransaction::new(transaction_with_chain_id(spec.chain_id()).into(), None)).unwrap(), TransactionImportResult::Current);

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
