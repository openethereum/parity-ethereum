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
use std::time::{Instant, Duration};

use util::*;
use util::using_queue::{UsingQueue, GetAction};
use account_provider::AccountProvider;
use views::{BlockView, HeaderView};
use state::State;
use client::{MiningBlockChainClient, Executive, Executed, EnvInfo, TransactOptions, BlockID, CallAnalytics};
use executive::contract_address;
use block::{ClosedBlock, IsBlock, Block};
use header::BlockNumber;
use error::*;
use transaction::{Action, SignedTransaction};
use receipt::{Receipt, RichReceipt};
use spec::Spec;
use engines::Engine;
use miner::{MinerService, MinerStatus, TransactionQueue, PrioritizationStrategy, AccountDetails, TransactionOrigin};
use miner::work_notify::WorkPoster;
use client::TransactionImportResult;
use miner::price_info::PriceInfo;

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
	/// Minimum period between transaction-inspired reseals.
	pub reseal_min_period: Duration,
	/// Maximum amount of gas to bother considering for block insertion.
	pub tx_gas_limit: U256,
	/// Maximum size of the transaction queue.
	pub tx_queue_size: usize,
	/// Strategy to use for prioritizing transactions in the queue.
	pub tx_queue_strategy: PrioritizationStrategy,
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
			reseal_on_external_tx: false,
			reseal_on_own_tx: true,
			tx_gas_limit: !U256::zero(),
			tx_queue_size: 2048,
			tx_queue_strategy: PrioritizationStrategy::GasFactorAndGasPrice,
			pending_set: PendingSet::AlwaysQueue,
			reseal_min_period: Duration::from_secs(2),
			work_queue_size: 20,
			enable_resubmission: true,
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
}

impl GasPriceCalibrator {
	fn recalibrate<F: Fn(U256) + Sync + Send + 'static>(&mut self, set_price: F) {
		trace!(target: "miner", "Recalibrating {:?} versus {:?}", Instant::now(), self.next_calibration);
		if Instant::now() >= self.next_calibration {
			let usd_per_tx = self.options.usd_per_tx;
			trace!(target: "miner", "Getting price info");
			if let Ok(_) = PriceInfo::get(move |price: PriceInfo| {
				trace!(target: "miner", "Price info arrived: {:?}", price);
				let usd_per_eth = price.ethusd;
				let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
				let gas_per_tx: f32 = 21000.0;
				let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
				info!(target: "miner", "Updated conversion rate to Ξ1 = {} ({} wei/gas)", Colour::White.bold().paint(format!("US${}", usd_per_eth)), Colour::Yellow.bold().paint(format!("{}", wei_per_gas)));
				set_price(U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap());
			}) {
				self.next_calibration = Instant::now() + self.options.recalibration_period;
			} else {
				warn!(target: "miner", "Unable to update Ether price.");
			}
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
	pub fn new_calibrated(options: GasPriceCalibratorOptions) -> GasPricer {
		GasPricer::Calibrated(GasPriceCalibrator {
			options: options,
			next_calibration: Instant::now(),
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
pub struct Miner {
	// NOTE [ToDr]  When locking always lock in this order!
	transaction_queue: Arc<Mutex<TransactionQueue>>,
	sealing_work: Mutex<SealingWork>,
	next_allowed_reseal: Mutex<Instant>,
	sealing_block_last_request: Mutex<u64>,
	// for sealing...
	options: MinerOptions,

	gas_range_target: RwLock<(U256, U256)>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,
	engine: Arc<Engine>,

	accounts: Option<Arc<AccountProvider>>,
	work_poster: Option<WorkPoster>,
	gas_pricer: Mutex<GasPricer>,
}

impl Miner {
	/// Creates new instance of miner without accounts, but with given spec.
	pub fn with_spec(spec: &Spec) -> Miner {
		Miner {
			transaction_queue: Arc::new(Mutex::new(TransactionQueue::default())),
			options: Default::default(),
			next_allowed_reseal: Mutex::new(Instant::now()),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(SealingWork{queue: UsingQueue::new(20), enabled: false}),
			gas_range_target: RwLock::new((U256::zero(), U256::zero())),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			accounts: None,
			engine: spec.engine.clone(),
			work_poster: None,
			gas_pricer: Mutex::new(GasPricer::new_fixed(20_000_000_000u64.into())),
		}
	}

	/// Creates new instance of miner
	pub fn new(options: MinerOptions, gas_pricer: GasPricer, spec: &Spec, accounts: Option<Arc<AccountProvider>>) -> Arc<Miner> {
		let work_poster = if !options.new_work_notify.is_empty() { Some(WorkPoster::new(&options.new_work_notify)) } else { None };
		let txq = Arc::new(Mutex::new(TransactionQueue::with_limits(
			options.tx_queue_strategy, options.tx_queue_size, !U256::zero(), options.tx_gas_limit
		)));
		Arc::new(Miner {
			transaction_queue: txq,
			next_allowed_reseal: Mutex::new(Instant::now()),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(SealingWork{queue: UsingQueue::new(options.work_queue_size), enabled: options.force_sealing || !options.new_work_notify.is_empty()}),
			gas_range_target: RwLock::new((U256::zero(), U256::zero())),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			options: options,
			accounts: accounts,
			engine: spec.engine.clone(),
			work_poster: work_poster,
			gas_pricer: Mutex::new(gas_pricer),
		})
	}

	fn forced_sealing(&self) -> bool {
		self.options.force_sealing || !self.options.new_work_notify.is_empty()
	}

	/// Get `Some` `clone()` of the current pending block's state or `None` if we're not sealing.
	pub fn pending_state(&self) -> Option<State> {
		self.sealing_work.lock().queue.peek_last_ref().map(|b| b.block().fields().state.clone())
	}

	/// Get `Some` `clone()` of the current pending block's state or `None` if we're not sealing.
	pub fn pending_block(&self) -> Option<Block> {
		self.sealing_work.lock().queue.peek_last_ref().map(|b| b.base().clone())
	}

	/// Prepares new block for sealing including top transactions from queue.
	#[cfg_attr(feature="dev", allow(match_same_arms))]
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn prepare_sealing(&self, chain: &MiningBlockChainClient) {
		trace!(target: "miner", "prepare_sealing: entering");

		let _timer = PerfTimer::new("prepare_sealing");
		{
			trace!(target: "miner", "recalibrating...");
			let txq = self.transaction_queue.clone();
			self.gas_pricer.lock().recalibrate(move |price| {
				trace!(target: "miner", "Got gas price! {}", price);
				txq.lock().set_minimal_gas_price(price);
			});
			trace!(target: "miner", "done recalibration.");
		}

		let (transactions, mut open_block, original_work_hash) = {
			let transactions = {self.transaction_queue.lock().top_transactions()};
			let mut sealing_work = self.sealing_work.lock();
			let last_work_hash = sealing_work.queue.peek_last_ref().map(|pb| pb.block().fields().header.hash());
			let best_hash = chain.best_block_header().sha3();
/*
			// check to see if last ClosedBlock in would_seals is actually same parent block.
			// if so
			//   duplicate, re-open and push any new transactions.
			//   if at least one was pushed successfully, close and enqueue new ClosedBlock;
			//   otherwise, leave everything alone.
			// otherwise, author a fresh block.
*/
			let open_block = match sealing_work.queue.pop_if(|b| b.block().fields().header.parent_hash() == &best_hash) {
				Some(old_block) => {
					trace!(target: "miner", "Already have previous work; updating and returning");
					// add transactions to old_block
					old_block.reopen(&*self.engine, chain.vm_factory())
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
		let mut transactions_to_penalize = HashSet::new();
		let block_number = open_block.block().fields().header.number();
		// TODO: push new uncles, too.
		for tx in transactions {
			let hash = tx.hash();
			match open_block.push_transaction(tx, None) {
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
			let mut queue = self.transaction_queue.lock();
			for hash in invalid_transactions.into_iter() {
				queue.remove_invalid(&hash, &fetch_account);
			}
			for hash in transactions_to_penalize {
				queue.penalize(&hash);
			}
		}

		if !block.transactions().is_empty() {
			trace!(target: "miner", "prepare_sealing: block has transaction - attempting internal seal.");
			// block with transactions - see if we can seal immediately.
			let s = self.engine.generate_seal(block.block(), match self.accounts {
				Some(ref x) => Some(&**x),
				None => None,
			});
			if let Some(seal) = s {
				trace!(target: "miner", "prepare_sealing: managed internal seal. importing...");
				if let Ok(sealed) = block.lock().try_seal(&*self.engine, seal) {
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
			let mut sealing_work = self.sealing_work.lock();
			let last_work_hash = sealing_work.queue.peek_last_ref().map(|pb| pb.block().fields().header.hash());
			trace!(target: "miner", "Checking whether we need to reseal: orig={:?} last={:?}, this={:?}", original_work_hash, last_work_hash, block.block().fields().header.hash());
			let (work, is_new) = if last_work_hash.map_or(true, |h| h != block.block().fields().header.hash()) {
				trace!(target: "miner", "Pushing a new, refreshed or borrowed pending {}...", block.block().fields().header.hash());
				let pow_hash = block.block().fields().header.hash();
				let number = block.block().fields().header.number();
				let difficulty = *block.block().fields().header.difficulty();
				let is_new = original_work_hash.map_or(true, |h| block.block().fields().header.hash() != h);
				sealing_work.queue.push(block);
				// If push notifications are enabled we assume all work items are used.
				if self.work_poster.is_some() && is_new {
					sealing_work.queue.use_last_ref();
				}
				(Some((pow_hash, difficulty, number)), is_new)
			} else {
				(None, false)
			};
			trace!(target: "miner", "prepare_sealing: leaving (last={:?})", sealing_work.queue.peek_last_ref().map(|b| b.block().fields().header.hash()));
			(work, is_new)
		};
		if is_new {
			work.map(|(pow_hash, difficulty, number)| self.work_poster.as_ref().map(|ref p| p.notify(pow_hash, difficulty, number)));
		}
	}

	fn update_gas_limit(&self, chain: &MiningBlockChainClient) {
		let gas_limit = HeaderView::new(&chain.best_block_header()).gas_limit();
		let mut queue = self.transaction_queue.lock();
		queue.set_gas_limit(gas_limit);
		// Set total qx queue gas limit to be 2x the block gas limit.
		queue.set_total_gas_limit(gas_limit << 1);
	}

	/// Returns true if we had to prepare new pending block
	fn enable_and_prepare_sealing(&self, chain: &MiningBlockChainClient) -> bool {
		trace!(target: "miner", "enable_and_prepare_sealing: entering");
		let prepare_new = {
			let mut sealing_work = self.sealing_work.lock();
			let have_work = sealing_work.queue.peek_last_ref().is_some();
			trace!(target: "miner", "enable_and_prepare_sealing: have_work={}", have_work);
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
			self.prepare_sealing(chain);
		}
		let mut sealing_block_last_request = self.sealing_block_last_request.lock();
		let best_number = chain.chain_info().best_block_number;
		if *sealing_block_last_request != best_number {
			trace!(target: "miner", "enable_and_prepare_sealing: Miner received request (was {}, now {}) - waking up.", *sealing_block_last_request, best_number);
			*sealing_block_last_request = best_number;
		}

		// Return if we restarted
		prepare_new
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
	fn tx_reseal_allowed(&self) -> bool { Instant::now() > *self.next_allowed_reseal.lock() }

	fn from_pending_block<H, F, G>(&self, latest_block_number: BlockNumber, from_chain: F, map_block: G) -> H
		where F: Fn() -> H, G: Fn(&ClosedBlock) -> H {
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

	fn clear_and_reset(&self, chain: &MiningBlockChainClient) {
		self.transaction_queue.lock().clear();
		// --------------------------------------------------------------------------
		// | NOTE Code below requires transaction_queue and sealing_work locks.     |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
		self.update_sealing(chain);
	}

	fn status(&self) -> MinerStatus {
		let status = self.transaction_queue.lock().status();
		let sealing_work = self.sealing_work.lock();
		MinerStatus {
			transactions_in_pending_queue: status.pending,
			transactions_in_future_queue: status.future,
			transactions_in_pending_block: sealing_work.queue.peek_last_ref().map_or(0, |b| b.transactions().len()),
		}
	}

	fn call(&self, chain: &MiningBlockChainClient, t: &SignedTransaction, analytics: CallAnalytics) -> Result<Executed, CallError> {
		let sealing_work = self.sealing_work.lock();
		match sealing_work.queue.peek_last_ref() {
			Some(work) => {
				let block = work.block();

				// TODO: merge this code with client.rs's fn call somwhow.
				let header = block.header();
				let last_hashes = Arc::new(chain.last_hashes());
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
				let original_state = if analytics.state_diffing { Some(state.clone()) } else { None };

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
				let mut ret = try!(Executive::new(&mut state, &env_info, &*self.engine, chain.vm_factory()).transact(t, options));

				// TODO gav move this into Executive.
				ret.state_diff = original_state.map(|original| state.diff_from(original));

				Ok(ret)
			},
			None => {
				chain.call(t, BlockID::Latest, analytics)
			}
		}
	}

	fn balance(&self, chain: &MiningBlockChainClient, address: &Address) -> U256 {
		self.from_pending_block(
			chain.chain_info().best_block_number,
			|| chain.latest_balance(address),
			|b| b.block().fields().state.balance(address)
		)
	}

	fn storage_at(&self, chain: &MiningBlockChainClient, address: &Address, position: &H256) -> H256 {
		self.from_pending_block(
			chain.chain_info().best_block_number,
			|| chain.latest_storage_at(address, position),
			|b| b.block().fields().state.storage_at(address, position)
		)
	}

	fn nonce(&self, chain: &MiningBlockChainClient, address: &Address) -> U256 {
		self.from_pending_block(
			chain.chain_info().best_block_number,
			|| chain.latest_nonce(address),
			|b| b.block().fields().state.nonce(address)
		)
	}

	fn code(&self, chain: &MiningBlockChainClient, address: &Address) -> Option<Bytes> {
		self.from_pending_block(
			chain.chain_info().best_block_number,
			|| chain.latest_code(address),
			|b| b.block().fields().state.code(address).map(|c| (*c).clone())
		)
	}

	fn set_author(&self, author: Address) {
		*self.author.write() = author;
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
		self.transaction_queue.lock().set_minimal_gas_price(min_gas_price);
	}

	fn minimal_gas_price(&self) -> U256 {
		*self.transaction_queue.lock().minimal_gas_price()
	}

	fn sensible_gas_price(&self) -> U256 {
		// 10% above our minimum.
		*self.transaction_queue.lock().minimal_gas_price() * 110.into() / 100.into()
	}

	fn sensible_gas_limit(&self) -> U256 {
		self.gas_range_target.read().0 / 5.into()
	}

	fn transactions_limit(&self) -> usize {
		self.transaction_queue.lock().limit()
	}

	fn set_transactions_limit(&self, limit: usize) {
		self.transaction_queue.lock().set_limit(limit)
	}

	fn set_tx_gas_limit(&self, limit: U256) {
		self.transaction_queue.lock().set_tx_gas_limit(limit)
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

	fn import_external_transactions(
		&self,
		chain: &MiningBlockChainClient,
		transactions: Vec<SignedTransaction>
	) -> Vec<Result<TransactionImportResult, Error>> {

		let results = {
			let mut transaction_queue = self.transaction_queue.lock();
			self.add_transactions_to_queue(
				chain, transactions, TransactionOrigin::External, &mut transaction_queue
			)
		};

		if !results.is_empty() && self.options.reseal_on_external_tx &&	self.tx_reseal_allowed() {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing_work locks.     |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
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
			let mut transaction_queue = self.transaction_queue.lock();
			let import = self.add_transactions_to_queue(
				chain, vec![transaction], TransactionOrigin::Local, &mut transaction_queue
			).pop().unwrap();

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

		// --------------------------------------------------------------------------
		// | NOTE Code below requires transaction_queue and sealing_work locks.     |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
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
		let queue = self.transaction_queue.lock();
		queue.top_transactions()
	}

	fn pending_transactions(&self, best_block: BlockNumber) -> Vec<SignedTransaction> {
		let queue = self.transaction_queue.lock();
		match self.options.pending_set {
			PendingSet::AlwaysQueue => queue.top_transactions(),
			PendingSet::SealingOrElseQueue => {
				self.from_pending_block(
					best_block,
					|| queue.top_transactions(),
					|sealing| sealing.transactions().to_owned()
				)
			},
			PendingSet::AlwaysSealing => {
				self.from_pending_block(
					best_block,
					|| vec![],
					|sealing| sealing.transactions().to_owned()
				)
			},
		}
	}

	fn pending_transactions_hashes(&self, best_block: BlockNumber) -> Vec<H256> {
		let queue = self.transaction_queue.lock();
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

	fn transaction(&self, best_block: BlockNumber, hash: &H256) -> Option<SignedTransaction> {
		let queue = self.transaction_queue.lock();
		match self.options.pending_set {
			PendingSet::AlwaysQueue => queue.find(hash),
			PendingSet::SealingOrElseQueue => {
				self.from_pending_block(
					best_block,
					|| queue.find(hash),
					|sealing| sealing.transactions().iter().find(|t| &t.hash() == hash).cloned()
				)
			},
			PendingSet::AlwaysSealing => {
				self.from_pending_block(
					best_block,
					|| None,
					|sealing| sealing.transactions().iter().find(|t| &t.hash() == hash).cloned()
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
						let ref tx = txs[index];
						let ref receipt = pending.receipts()[index];
						RichReceipt {
							transaction_hash: hash.clone(),
							transaction_index: index,
							cumulative_gas_used: receipt.gas_used,
							gas_used: receipt.gas_used - prev_gas,
							contract_address: match tx.action {
								Action::Call(_) => None,
								Action::Create => Some(contract_address(&tx.sender().unwrap(), &tx.nonce)),
							},
							logs: receipt.logs.clone(),
						}
					})
			}
		)
	}

	fn pending_receipts(&self, best_block: BlockNumber) -> BTreeMap<H256, Receipt> {
		self.from_pending_block(
			best_block,
			|| BTreeMap::new(),
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
		self.transaction_queue.lock().last_nonce(address)
	}

	fn update_sealing(&self, chain: &MiningBlockChainClient) {
		trace!(target: "miner", "update_sealing");
		let requires_reseal = {
			let has_local_transactions = self.transaction_queue.lock().has_local_pending_transactions();
			let mut sealing_work = self.sealing_work.lock();
			if sealing_work.enabled {
				trace!(target: "miner", "update_sealing: sealing enabled");
				let current_no = chain.chain_info().best_block_number;
				let last_request = *self.sealing_block_last_request.lock();
				let should_disable_sealing = !self.forced_sealing()
					&& !has_local_transactions
					&& current_no > last_request
					&& current_no - last_request > SEALING_TIMEOUT_IN_BLOCKS;

				trace!(target: "miner", "update_sealing: should_disable_sealing={}; current_no={}, last_request={}", should_disable_sealing, current_no, last_request);

				if should_disable_sealing {
					trace!(target: "miner", "Miner sleeping (current {}, last {})", current_no, last_request);
					sealing_work.enabled = false;
					sealing_work.queue.reset();
					false
				} else {
					// sealing enabled and we don't want to sleep.
					*self.next_allowed_reseal.lock() = Instant::now() + self.options.reseal_min_period;
					true
				}
			} else {
				// sealing is disabled.
				false
			}
		};

		if requires_reseal {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing_work locks.     |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			self.prepare_sealing(chain);
		}
	}

	fn is_sealing(&self) -> bool {
		self.sealing_work.lock().queue.is_in_use()
	}

	fn map_sealing_work<F, T>(&self, chain: &MiningBlockChainClient, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		trace!(target: "miner", "map_sealing_work: entering");
		self.enable_and_prepare_sealing(chain);
		trace!(target: "miner", "map_sealing_work: sealing prepared");
		let mut sealing_work = self.sealing_work.lock();
		let ret = sealing_work.queue.use_last_ref();
		trace!(target: "miner", "map_sealing_work: leaving use_last_ref={:?}", ret.as_ref().map(|b| b.block().fields().header.hash()));
		ret.map(f)
	}

	fn submit_seal(&self, chain: &MiningBlockChainClient, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		let result = if let Some(b) = self.sealing_work.lock().queue.get_used_if(if self.options.enable_resubmission { GetAction::Clone } else { GetAction::Take }, |b| &b.hash() == &pow_hash) {
			b.lock().try_seal(&*self.engine, seal).or_else(|_| {
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
			info!(target: "miner", "Mined block imported OK. #{}: {}", Colour::White.bold().paint(format!("{}", n)), Colour::White.bold().paint(h.hex()));
			Ok(())
		})
	}

	fn chain_new_blocks(&self, chain: &MiningBlockChainClient, _imported: &[H256], _invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		trace!(target: "miner", "chain_new_blocks");

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
				let mut transaction_queue = self.transaction_queue.lock();
				let _ = self.add_transactions_to_queue(
					chain, txs, TransactionOrigin::RetractedBlock, &mut transaction_queue
				);
			});
		}

		// ...and at the end remove old ones
		{
			let in_chain = enacted
				.par_iter()
				.map(|h: &H256| fetch_transactions(chain, h));

			in_chain.for_each(|mut txs| {
				let mut transaction_queue = self.transaction_queue.lock();

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

		if enacted.len() > 0 {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires transaction_queue and sealing_work locks.     |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			self.update_sealing(chain);
		}
	}
}

#[cfg(test)]
mod tests {

	use std::time::Duration;
	use super::super::{MinerService, PrioritizationStrategy};
	use super::*;
	use util::*;
	use client::{TestBlockChainClient, EachBlockWith};
	use client::{TransactionImportResult};
	use types::transaction::{Transaction, Action};
	use block::*;
	use spec::Spec;

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
				reseal_min_period: Duration::from_secs(5),
				tx_gas_limit: !U256::zero(),
				tx_queue_size: 1024,
				tx_queue_strategy: PrioritizationStrategy::GasFactorAndGasPrice,
				pending_set: PendingSet::AlwaysSealing,
				work_queue_size: 5,
				enable_resubmission: true,
			},
			GasPricer::new_fixed(0u64.into()),
			&Spec::new_test(),
			None, // accounts provider
		)).ok().expect("Miner was just created.")
	}

	#[test]
	fn should_make_pending_block_when_importing_own_transaction() {
		// given
		let client = TestBlockChainClient::default();
		let miner = miner();
		let transaction = {
			let keypair = KeyPair::create().unwrap();
			Transaction {
				action: Action::Create,
				value: U256::zero(),
				data: "3331600055".from_hex().unwrap(),
				gas: U256::from(100_000),
				gas_price: U256::zero(),
				nonce: U256::zero(),
			}.sign(keypair.secret())
		};
		let best_block = 0;
		// when
		let res = miner.import_own_transaction(&client, transaction);

		// then
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
		assert_eq!(miner.all_transactions().len(), 1);
		assert_eq!(miner.pending_transactions(best_block).len(), 1);
		assert_eq!(miner.pending_transactions_hashes(best_block).len(), 1);
		assert_eq!(miner.pending_receipts(best_block).len(), 1);
		// This method will let us know if pending block was created (before calling that method)
		assert_eq!(miner.enable_and_prepare_sealing(&client), false);
	}

	#[test]
	fn should_not_use_pending_block_if_best_block_is_higher() {
		// given
		let client = TestBlockChainClient::default();
		let miner = miner();
		let transaction = {
			let keypair = KeyPair::create().unwrap();
			Transaction {
				action: Action::Create,
				value: U256::zero(),
				data: "3331600055".from_hex().unwrap(),
				gas: U256::from(100_000),
				gas_price: U256::zero(),
				nonce: U256::zero(),
			}.sign(keypair.secret())
		};
		let best_block = 10;
		// when
		let res = miner.import_own_transaction(&client, transaction);

		// then
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
		assert_eq!(miner.all_transactions().len(), 1);
		assert_eq!(miner.pending_transactions(best_block).len(), 0);
		assert_eq!(miner.pending_transactions_hashes(best_block).len(), 0);
		assert_eq!(miner.pending_receipts(best_block).len(), 0);
	}

	#[test]
	fn should_import_external_transaction() {
		// given
		let client = TestBlockChainClient::default();
		let miner = miner();
		let transaction = {
			let keypair = KeyPair::create().unwrap();
			Transaction {
				action: Action::Create,
				value: U256::zero(),
				data: "3331600055".from_hex().unwrap(),
				gas: U256::from(100_000),
				gas_price: U256::zero(),
				nonce: U256::zero(),
			}.sign(keypair.secret())
		};
		let best_block = 0;
		// when
		let res = miner.import_external_transactions(&client, vec![transaction]).pop().unwrap();

		// then
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
		assert_eq!(miner.all_transactions().len(), 1);
		assert_eq!(miner.pending_transactions_hashes(best_block).len(), 0);
		assert_eq!(miner.pending_transactions(best_block).len(), 0);
		assert_eq!(miner.pending_receipts(best_block).len(), 0);
		// This method will let us know if pending block was created (before calling that method)
		assert_eq!(miner.enable_and_prepare_sealing(&client), true);
	}
}
