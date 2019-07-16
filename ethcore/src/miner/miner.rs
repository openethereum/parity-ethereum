// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

use std::cmp;
use std::time::{Instant, Duration};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

use ansi_term::Colour;
use bytes::Bytes;
use call_contract::CallContract;
use ethcore_miner::gas_pricer::GasPricer;
use ethcore_miner::local_accounts::LocalAccounts;
use ethcore_miner::pool::{self, TransactionQueue, VerifiedTransaction, QueueStatus, PrioritizationStrategy, TxStatus};
use ethcore_miner::service_transaction_checker::ServiceTransactionChecker;
#[cfg(feature = "work-notify")]
use ethcore_miner::work_notify::NotifyWork;
use ethereum_types::{H256, U256, Address};
use futures::sync::mpsc;
use io::IoChannel;
use miner::filter_options::{FilterOptions, FilterOperator};
use miner::pool_client::{PoolClient, CachedNonceClient, NonceCache};
use miner;
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use types::transaction::{
	self,
	Action,
	UnverifiedTransaction,
	SignedTransaction,
	PendingTransaction,
};
use types::BlockNumber;
use types::block::Block;
use types::header::Header;
use types::receipt::RichReceipt;
use using_queue::{UsingQueue, GetAction};

use block::{ClosedBlock, SealedBlock};
use client::{
	BlockChain, ChainInfo, BlockProducer, SealedBlockImporter, Nonce, TransactionInfo, TransactionId
};
use client::{BlockId, ClientIoMessage};
use engines::{Engine, Seal, SealingState, EngineSigner};
use error::Error;
use executed::ExecutionError;
use executive::contract_address;
use spec::Spec;
use account_state::State;
use vm::CreateContractAddress;

/// Different possible definitions for pending transaction set.
#[derive(Debug, PartialEq)]
pub enum PendingSet {
	/// Always just the transactions in the queue. These have had only cheap checks.
	AlwaysQueue,
	/// Always just the transactions in the sealing block. These have had full checks but
	/// may be empty if the node is not actively mining or has no force_sealing enabled.
	AlwaysSealing,
	/// Takes from sealing if mining, from queue otherwise.
	SealingOrElseQueue,
}

/// Transaction queue penalization settings.
///
/// Senders of long-running transactions (above defined threshold)
/// will get lower priority.
#[derive(Debug, PartialEq, Clone)]
pub enum Penalization {
	/// Penalization in transaction queue is disabled
	Disabled,
	/// Penalization in transaction queue is enabled
	Enabled {
		/// Upper limit of transaction processing time before penalizing.
		offend_threshold: Duration,
	},
}

/// Pending block preparation status.
#[derive(Debug, PartialEq)]
pub enum BlockPreparationStatus {
	/// We had to prepare new pending block and the preparation succeeded.
	Succeeded,
	/// We had to prepare new pending block but the preparation failed.
	Failed,
	/// We didn't have to prepare a new block.
	NotPrepared,
}

/// Initial minimal gas price.
///
/// Gas price should be later overwritten externally
/// for instance by a dynamic gas price mechanism or CLI parameter.
/// This constant controls the initial value.
const DEFAULT_MINIMAL_GAS_PRICE: u64 = 20_000_000_000;

/// Allowed number of skipped transactions when constructing pending block.
///
/// When we push transactions to pending block, some of the transactions might
/// get skipped because of block gas limit being reached.
/// This constant controls how many transactions we can skip because of that
/// before stopping attempts to push more transactions to the block.
/// This is an optimization that prevents traversing the entire pool
/// in case we have only a fraction of available block gas limit left.
const MAX_SKIPPED_TRANSACTIONS: usize = 128;

/// Configures the behaviour of the miner.
#[derive(Debug, PartialEq)]
pub struct MinerOptions {
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
	/// Whether we should fallback to providing all the queue's transactions or just pending.
	pub pending_set: PendingSet,
	/// How many historical work packages can we store before running out?
	pub work_queue_size: usize,
	/// Can we submit two different solutions for the same block and expect both to result in an import?
	pub enable_resubmission: bool,
	/// Create a pending block with maximal possible gas limit.
	/// NOTE: Such block will contain all pending transactions but
	/// will be invalid if mined.
	pub infinite_pending_block: bool,

	/// Strategy to use for prioritizing transactions in the queue.
	pub tx_queue_strategy: PrioritizationStrategy,
	/// Simple senders penalization.
	pub tx_queue_penalization: Penalization,
	/// Do we want to mark transactions received locally (e.g. RPC) as local if we don't have the sending account?
	pub tx_queue_no_unfamiliar_locals: bool,
	/// Do we refuse to accept service transactions even if sender is certified.
	pub refuse_service_transactions: bool,
	/// Transaction pool limits.
	pub pool_limits: pool::Options,
	/// Initial transaction verification options.
	pub pool_verification_options: pool::verifier::Options,
}

impl Default for MinerOptions {
	fn default() -> Self {
		MinerOptions {
			force_sealing: false,
			reseal_on_external_tx: false,
			reseal_on_own_tx: true,
			reseal_on_uncle: false,
			reseal_min_period: Duration::from_secs(2),
			reseal_max_period: Duration::from_secs(120),
			pending_set: PendingSet::AlwaysQueue,
			work_queue_size: 20,
			enable_resubmission: true,
			infinite_pending_block: false,
			tx_queue_strategy: PrioritizationStrategy::GasPriceOnly,
			tx_queue_penalization: Penalization::Disabled,
			tx_queue_no_unfamiliar_locals: false,
			refuse_service_transactions: false,
			pool_limits: pool::Options {
				max_count: 8_192,
				max_per_sender: 81,
				max_mem_usage: 4 * 1024 * 1024,
			},
			pool_verification_options: pool::verifier::Options {
				minimal_gas_price: DEFAULT_MINIMAL_GAS_PRICE.into(),
				block_gas_limit: U256::max_value(),
				tx_gas_limit: U256::max_value(),
				no_early_reject: false,
			},
		}
	}
}

/// Configurable parameters of block authoring.
#[derive(Debug, Default, Clone)]
pub struct AuthoringParams {
	/// Lower and upper bound of block gas limit that we are targeting
	pub gas_range_target: (U256, U256),
	/// Block author
	pub author: Address,
	/// Block extra data
	pub extra_data: Bytes,
}

/// Block sealing mechanism
pub enum Author {
	/// Sealing block is external and we only need a reward beneficiary (i.e. PoW)
	External(Address),
	/// Sealing is done internally, we need a way to create signatures to seal block (i.e. PoA)
	Sealer(Box<dyn EngineSigner>),
}

impl Author {
	/// Get author's address.
	pub fn address(&self) -> Address {
		match *self {
			Author::External(address) => address,
			Author::Sealer(ref sealer) => sealer.address(),
		}
	}
}

struct SealingWork {
	queue: UsingQueue<ClosedBlock>,
	enabled: bool,
	next_allowed_reseal: Instant,
	next_mandatory_reseal: Instant,
	// block number when sealing work was last requested
	last_request: Option<u64>,
}

impl SealingWork {
	/// Are we allowed to do a non-mandatory reseal?
	fn reseal_allowed(&self) -> bool {
		Instant::now() > self.next_allowed_reseal
	}

	fn work_available(&self) -> bool {
		self.queue.peek_last_ref().is_some()
	}
}

/// Keeps track of transactions using priority queue and holds currently mined block.
/// Handles preparing work for "work sealing" or seals "internally" if Engine does not require work.
pub struct Miner {
	// NOTE [ToDr]  When locking always lock in this order!
	sealing: Mutex<SealingWork>,
	params: RwLock<AuthoringParams>,
	#[cfg(feature = "work-notify")]
	listeners: RwLock<Vec<Box<NotifyWork>>>,
	nonce_cache: NonceCache,
	gas_pricer: Mutex<GasPricer>,
	options: MinerOptions,
	// TODO [ToDr] Arc is only required because of price updater
	transaction_queue: Arc<TransactionQueue>,
	engine: Arc<dyn Engine>,
	accounts: Arc<dyn LocalAccounts>,
	io_channel: RwLock<Option<IoChannel<ClientIoMessage>>>,
	service_transaction_checker: Option<ServiceTransactionChecker>,
}

impl Miner {
	/// Push listener that will handle new jobs
	#[cfg(feature = "work-notify")]
	pub fn add_work_listener(&self, notifier: Box<NotifyWork>) {
		self.listeners.write().push(notifier);
		self.sealing.lock().enabled = true;
	}

	/// Set a callback to be notified about imported transactions' hashes.
	pub fn pending_transactions_receiver(&self) -> mpsc::UnboundedReceiver<Arc<Vec<H256>>> {
		let (sender, receiver) = mpsc::unbounded();
		self.transaction_queue.add_pending_listener(sender);
		receiver
	}

	/// Set a callback to be notified about imported transactions' hashes.
	pub fn full_transactions_receiver(&self) -> mpsc::UnboundedReceiver<Arc<Vec<(H256, TxStatus)>>> {
		let (sender, receiver) = mpsc::unbounded();
		self.transaction_queue.add_full_listener(sender);
		receiver
	}

	/// Creates new instance of miner Arc.
	pub fn new<A: LocalAccounts + 'static>(
		options: MinerOptions,
		gas_pricer: GasPricer,
		spec: &Spec,
		accounts: A,
	) -> Self {
		let limits = options.pool_limits.clone();
		let verifier_options = options.pool_verification_options.clone();
		let tx_queue_strategy = options.tx_queue_strategy;
		let nonce_cache_size = cmp::max(4096, limits.max_count / 4);
		let refuse_service_transactions = options.refuse_service_transactions;

		Miner {
			sealing: Mutex::new(SealingWork {
				queue: UsingQueue::new(options.work_queue_size),
				enabled: options.force_sealing
					|| spec.engine.sealing_state() != SealingState::External,
				next_allowed_reseal: Instant::now(),
				next_mandatory_reseal: Instant::now() + options.reseal_max_period,
				last_request: None,
			}),
			params: RwLock::new(AuthoringParams::default()),
			#[cfg(feature = "work-notify")]
			listeners: RwLock::new(vec![]),
			gas_pricer: Mutex::new(gas_pricer),
			nonce_cache: NonceCache::new(nonce_cache_size),
			options,
			transaction_queue: Arc::new(TransactionQueue::new(limits, verifier_options, tx_queue_strategy)),
			accounts: Arc::new(accounts),
			engine: spec.engine.clone(),
			io_channel: RwLock::new(None),
			service_transaction_checker: if refuse_service_transactions {
				None
			} else {
				Some(ServiceTransactionChecker::default())
			},
		}
	}

	/// Creates new instance of miner with given spec and accounts.
	///
	/// NOTE This should be only used for tests.
	pub fn new_for_tests(spec: &Spec, accounts: Option<HashSet<Address>>) -> Miner {
		let minimal_gas_price = 0.into();
		Miner::new(MinerOptions {
			pool_verification_options: pool::verifier::Options {
				minimal_gas_price,
				block_gas_limit: U256::max_value(),
				tx_gas_limit: U256::max_value(),
				no_early_reject: false,
			},
			reseal_min_period: Duration::from_secs(0),
			..Default::default()
		}, GasPricer::new_fixed(minimal_gas_price), spec, accounts.unwrap_or_default())
	}

	/// Sets `IoChannel`
	pub fn set_io_channel(&self, io_channel: IoChannel<ClientIoMessage>) {
		*self.io_channel.write() = Some(io_channel);
	}

	/// Sets in-blockchain checker for transactions.
	pub fn set_in_chain_checker<C>(&self, chain: &Arc<C>) where
		C: TransactionInfo + Send + Sync + 'static,
	{
		let client = Arc::downgrade(chain);
		self.transaction_queue.set_in_chain_checker(move |hash| {
			match client.upgrade() {
				Some(info) => info.transaction_block(TransactionId::Hash(*hash)).is_some(),
				None => false,
			}
		});
	}

	/// Clear all pending block states
	pub fn clear(&self) {
		self.sealing.lock().queue.reset();
	}

	/// Updates transaction queue verification limits.
	///
	/// Limits consist of current block gas limit and minimal gas price.
	pub fn update_transaction_queue_limits(&self, block_gas_limit: U256) {
		trace!(target: "miner", "minimal_gas_price: recalibrating...");
		let txq = self.transaction_queue.clone();
		let mut options = self.options.pool_verification_options.clone();
		self.gas_pricer.lock().recalibrate(move |gas_price| {
			debug!(target: "miner", "minimal_gas_price: Got gas price! {}", gas_price);
			options.minimal_gas_price = gas_price;
			options.block_gas_limit = block_gas_limit;
			txq.set_verifier_options(options);
		});
	}

	/// Returns ServiceTransactionChecker
	pub fn service_transaction_checker(&self) -> Option<ServiceTransactionChecker> {
		self.service_transaction_checker.clone()
	}

	/// Retrieves an existing pending block iff it's not older than given block number.
	///
	/// NOTE: This will not prepare a new pending block if it's not existing.
	fn map_existing_pending_block<F, T>(&self, f: F, latest_block_number: BlockNumber) -> Option<T> where
		F: FnOnce(&ClosedBlock) -> T,
	{
		self.sealing.lock().queue
			.peek_last_ref()
			.and_then(|b| {
				// to prevent a data race between block import and updating pending block
				// we allow the number to be equal.
				if b.header.number() >= latest_block_number {
					Some(f(b))
				} else {
					None
				}
			})
	}

	fn pool_client<'a, C: 'a>(&'a self, chain: &'a C) -> PoolClient<'a, C> where
		C: BlockChain + CallContract,
	{
		PoolClient::new(
			chain,
			&self.nonce_cache,
			&*self.engine,
			&*self.accounts,
			self.service_transaction_checker.as_ref(),
		)
	}

	/// Prepares new block for sealing including top transactions from queue.
	fn prepare_block<C>(&self, chain: &C) -> Option<(ClosedBlock, Option<H256>)> where
		C: BlockChain + CallContract + BlockProducer + Nonce + Sync,
	{
		trace_time!("prepare_block");
		let chain_info = chain.chain_info();

		// Open block
		let (mut open_block, original_work_hash) = {
			let mut sealing = self.sealing.lock();
			let last_work_hash = sealing.queue.peek_last_ref().map(|pb| pb.header.hash());
			let best_hash = chain_info.best_block_hash;

			// check to see if last ClosedBlock in would_seals is actually same parent block.
			// if so
			//   duplicate, re-open and push any new transactions.
			//   if at least one was pushed successfully, close and enqueue new ClosedBlock;
			//   otherwise, leave everything alone.
			// otherwise, author a fresh block.
			let mut open_block = match sealing.queue.get_pending_if(|b| b.header.parent_hash() == &best_hash) {
				Some(old_block) => {
					trace!(target: "miner", "prepare_block: Already have previous work; updating and returning");
					// add transactions to old_block
					chain.reopen_block(old_block)
				}
				None => {
					// block not found - create it.
					trace!(target: "miner", "prepare_block: No existing work - making new block");
					let params = self.params.read().clone();

					match chain.prepare_open_block(
						params.author,
						params.gas_range_target,
						params.extra_data,
					) {
						Ok(block) => block,
						Err(err) => {
							warn!(target: "miner", "Open new block failed with error {:?}. This is likely an error in chain specificiations or on-chain consensus smart contracts.", err);
							return None;
						}
					}
				}
			};

			if self.options.infinite_pending_block {
				open_block.remove_gas_limit();
			}

			(open_block, last_work_hash)
		};

		let mut invalid_transactions = HashSet::new();
		let mut not_allowed_transactions = HashSet::new();
		let mut senders_to_penalize = HashSet::new();
		let block_number = open_block.header.number();

		let mut tx_count = 0usize;
		let mut skipped_transactions = 0usize;

		let client = self.pool_client(chain);
		let engine_params = self.engine.params();
		let min_tx_gas: U256 = self.engine.schedule(chain_info.best_block_number).tx_gas.into();
		let nonce_cap: Option<U256> = if chain_info.best_block_number + 1 >= engine_params.dust_protection_transition {
			Some((engine_params.nonce_cap_increment * (chain_info.best_block_number + 1)).into())
		} else {
			None
		};
		// we will never need more transactions than limit divided by min gas
		let max_transactions = if min_tx_gas.is_zero() {
			usize::max_value()
		} else {
			MAX_SKIPPED_TRANSACTIONS.saturating_add(cmp::min(*open_block.header.gas_limit() / min_tx_gas, u64::max_value().into()).as_u64() as usize)
		};

		let pending: Vec<Arc<_>> = self.transaction_queue.pending(
			client.clone(),
			pool::PendingSettings {
				block_number: chain_info.best_block_number,
				current_timestamp: chain_info.best_block_timestamp,
				nonce_cap,
				max_len: max_transactions,
				ordering: miner::PendingOrdering::Priority,
			}
		);

		let took_ms = |elapsed: &Duration| {
			elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1_000_000
		};

		let block_start = Instant::now();
		debug!(target: "miner", "Attempting to push {} transactions.", pending.len());

		for tx in pending {
			let start = Instant::now();

			let transaction = tx.signed().clone();
			let hash = transaction.hash();
			let sender = transaction.sender();

			// Re-verify transaction again vs current state.
			let result = client.verify_signed(&transaction)
				.map_err(|e| e.into())
				.and_then(|_| {
					open_block.push_transaction(transaction, None)
				});

			let took = start.elapsed();

			// Check for heavy transactions
			match self.options.tx_queue_penalization {
				Penalization::Enabled { ref offend_threshold } if &took > offend_threshold => {
					senders_to_penalize.insert(sender);
					debug!(target: "miner", "Detected heavy transaction ({} ms). Penalizing sender.", took_ms(&took));
				},
				_ => {},
			}

			debug!(target: "miner", "Adding tx {:?} took {} ms", hash, took_ms(&took));
			match result {
				Err(Error::Execution(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, gas })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of gas limit: {:?} (limit: {:?}, used: {:?}, gas: {:?})", hash, gas_limit, gas_used, gas);

					// Penalize transaction if it's above current gas limit
					if gas > gas_limit {
						debug!(target: "txqueue", "[{:?}] Transaction above block gas limit.", hash);
						invalid_transactions.insert(hash);
					}

					// Exit early if gas left is smaller then min_tx_gas
					let gas_left = gas_limit - gas_used;
					if gas_left < min_tx_gas {
						debug!(target: "miner", "Remaining gas is lower than minimal gas for a transaction. Block is full.");
						break;
					}

					// Avoid iterating over the entire queue in case block is almost full.
					skipped_transactions += 1;
					if skipped_transactions > MAX_SKIPPED_TRANSACTIONS {
						debug!(target: "miner", "Reached skipped transactions threshold. Assuming block is full.");
						break;
					}
				},
				// Invalid nonce error can happen only if previous transaction is skipped because of gas limit.
				// If there is erroneous state of transaction queue it will be fixed when next block is imported.
				Err(Error::Execution(ExecutionError::InvalidNonce { expected, got })) => {
					debug!(target: "miner", "Skipping adding transaction to block because of invalid nonce: {:?} (expected: {:?}, got: {:?})", hash, expected, got);
				},
				// already have transaction - ignore
				Err(Error::Transaction(transaction::Error::AlreadyImported)) => {},
				Err(Error::Transaction(transaction::Error::NotAllowed)) => {
					not_allowed_transactions.insert(hash);
					debug!(target: "miner", "Skipping non-allowed transaction for sender {:?}", hash);
				},
				Err(e) => {
					debug!(target: "txqueue", "[{:?}] Marking as invalid: {:?}.", hash, e);
					debug!(
						target: "miner", "Error adding transaction to block: number={}. transaction_hash={:?}, Error: {:?}", block_number, hash, e
					);
					invalid_transactions.insert(hash);
				},
				// imported ok
				_ => tx_count += 1,
			}
		}
		let elapsed = block_start.elapsed();
		debug!(target: "miner", "Pushed {} transactions in {} ms", tx_count, took_ms(&elapsed));

		let block = match open_block.close() {
			Ok(block) => block,
			Err(err) => {
				warn!(target: "miner", "Closing the block failed with error {:?}. This is likely an error in chain specificiations or on-chain consensus smart contracts.", err);
				return None;
			}
		};

		{
			self.transaction_queue.remove(invalid_transactions.iter(), true);
			self.transaction_queue.remove(not_allowed_transactions.iter(), false);
			self.transaction_queue.penalize(senders_to_penalize.iter());
		}

		Some((block, original_work_hash))
	}

	/// Returns `true` if we should create pending block even if some other conditions are not met.
	///
	/// In general we always seal iff:
	/// 1. --force-sealing CLI parameter is provided
	/// 2. There are listeners awaiting new work packages (e.g. remote work notifications or stratum).
	fn forced_sealing(&self) -> bool {
		let listeners_empty = {
			#[cfg(feature = "work-notify")]
			{ self.listeners.read().is_empty() }
			#[cfg(not(feature = "work-notify"))]
			{ true }
		};

		self.options.force_sealing || !listeners_empty
	}

	/// Check is reseal is allowed and necessary.
	fn requires_reseal(&self, best_block: BlockNumber) -> bool {
		let mut sealing = self.sealing.lock();
		if !sealing.enabled {
			trace!(target: "miner", "requires_reseal: sealing is disabled");
			return false
		}

		if !sealing.reseal_allowed() {
			trace!(target: "miner", "requires_reseal: reseal too early");
			return false
		}

		trace!(target: "miner", "requires_reseal: sealing enabled");

		const SEALING_TIMEOUT_IN_BLOCKS : u64 = 5;
		// Disable sealing if there were no requests for SEALING_TIMEOUT_IN_BLOCKS
		let had_requests = sealing.last_request.map(|last_request|
			best_block.saturating_sub(last_request) <= SEALING_TIMEOUT_IN_BLOCKS
		).unwrap_or(false);

		// keep sealing enabled if any of the conditions is met
		let sealing_enabled = self.forced_sealing()
			|| self.transaction_queue.has_local_pending_transactions()
			|| self.engine.sealing_state() == SealingState::Ready
			|| had_requests;

		let should_disable_sealing = !sealing_enabled;

		trace!(target: "miner", "requires_reseal: should_disable_sealing={}; forced={:?}, has_local={:?}, internal={:?}, had_requests={:?}",
			should_disable_sealing,
			self.forced_sealing(),
			self.transaction_queue.has_local_pending_transactions(),
			self.engine.sealing_state(),
			had_requests,
		);

		if should_disable_sealing {
			trace!(target: "miner", "Miner sleeping (current {}, last {})", best_block, sealing.last_request.unwrap_or(0));
			sealing.enabled = false;
			sealing.queue.reset();
			false
		} else {
			// sealing enabled and we don't want to sleep.
			sealing.next_allowed_reseal = Instant::now() + self.options.reseal_min_period;
			true
		}
	}

	// TODO: (https://github.com/paritytech/parity-ethereum/issues/10407)
	// This is only used in authority_round path, and should be refactored to merge with the other seal() path.
	// Attempts to perform internal sealing (one that does not require work: e.g. Clique
	// and Aura) and handles the result depending on the type of Seal.
	fn seal_and_import_block_internally<C>(&self, chain: &C, block: ClosedBlock) -> bool
		where C: BlockChain + SealedBlockImporter,
	{
		{
			let sealing = self.sealing.lock();
			if block.transactions.is_empty()
				&& !self.forced_sealing()
				&& Instant::now() <= sealing.next_mandatory_reseal
			{
				return false
			}
		}
		let block_number = block.header.number();
		trace!(target: "miner", "seal_block_internally: attempting internal seal for block #{}", block_number);

		let parent_header = match chain.block_header(BlockId::Hash(*block.header.parent_hash())) {
			Some(h) => {
				match h.decode() {
					Ok(decoded_hdr) => decoded_hdr,
					Err(e) => {
						error!(target: "miner", "seal_block_internally: Block #{}, Could not decode header from parent block (hash={}): {:?}", block_number, block.header.parent_hash(), e);
						return false
					}
				}
			},
			None => {
				trace!(target: "miner", "Block #{}: Parent with hash={} does not exist in our DB", block_number, block.header.parent_hash());
				return false
			},
		};

		let sealing_result =
			match self.engine.generate_seal(&block, &parent_header) {
				// Directly import a regular sealed block.
				Seal::Regular(seal) => {
					trace!(target: "miner", "Block #{}: Received a Regular seal.", block_number);
					{
						let mut sealing = self.sealing.lock();
						sealing.next_mandatory_reseal = Instant::now() + self.options.reseal_max_period;
					}

					block
						.lock()
						.seal(&*self.engine, seal)
						.map(|sealed| {
							match chain.import_sealed_block(sealed) {
								Ok(_) => true,
								Err(e) => {
									error!(target: "miner", "Block #{}: seal_and_import_block_internally: import_sealed_block returned {:?}", block_number, e);
									false
								}
							}
						})
						.unwrap_or_else(|e| {
							warn!("ERROR: Block #{}, importing sealed block failed when given internally generated seal: {}", block_number, e);
							false
						})
				},
				Seal::None => false,
			};
		sealing_result
	}

	/// Prepares work which has to be done to seal.
	fn prepare_work(&self, block: ClosedBlock, original_work_hash: Option<H256>) {
		let (work, is_new) = {
			let block_header = block.header.clone();
			let block_hash = block_header.hash();

			let mut sealing = self.sealing.lock();
			let last_work_hash = sealing.queue.peek_last_ref().map(|pb| pb.header.hash());

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

				sealing.queue.set_pending(block);

				#[cfg(feature = "work-notify")]
				{
					// If push notifications are enabled we assume all work items are used.
					if is_new && !self.listeners.read().is_empty() {
						sealing.queue.use_last_ref();
					}
				}

				(Some((block_hash, *block_header.difficulty(), block_header.number())), is_new)
			} else {
				(None, false)
			};
			trace!(
				target: "miner",
				"prepare_work: leaving (last={:?})",
				sealing.queue.peek_last_ref().map(|b| b.header.hash())
			);
			(work, is_new)
		};

		#[cfg(feature = "work-notify")]
		{
			if is_new {
				work.map(|(pow_hash, difficulty, number)| {
					for notifier in self.listeners.read().iter() {
						notifier.notify(pow_hash, difficulty, number)
					}
				});
			}
		}

		// NB: hack to use variables to avoid warning.
		#[cfg(not(feature = "work-notify"))]
		{
			let _work = work;
			let _is_new = is_new;
		}
	}

	/// Prepare a pending block. Returns the preparation status.
	/// Only used by externally sealing engines.
	fn prepare_pending_block<C>(&self, client: &C) -> BlockPreparationStatus
		where
			C: BlockChain + CallContract + BlockProducer + SealedBlockImporter + Nonce + Sync,
	{
		trace!(target: "miner", "prepare_pending_block: entering");
		// Unless we are `--force-sealing` we create pending blocks if
		//  1. we have local pending transactions
		//  2. or someone is requesting `eth_getWork`
		// When either condition is true, `sealing.enabled` is flipped to true (and if you
		// have no more local transactions or stop calling `eth_getWork`, it is set to `false`).

		// Here we check if there are pending blocks already (if so, we don't need to create
		// a new one); if there are none, we set `sealing.enabled` to true because the
		// calling code expects it to be on (or they wouldn't have called this method).
		// Yes, it's a bit convoluted.
		let prepare_new_block = self.maybe_enable_sealing();

		if self.engine.sealing_state() != SealingState::External {
			trace!(target: "miner", "prepare_pending_block: engine not sealing externally; not preparing");
			return BlockPreparationStatus::NotPrepared;
		}

		let preparation_status = if prepare_new_block {
			// --------------------------------------------------------------------------
			// | NOTE Code below requires sealing locks.                                |
			// | Make sure to release the locks before calling that method.             |
			// --------------------------------------------------------------------------
			match self.prepare_block(client) {
				Some((block, original_work_hash)) => {
					self.prepare_work(block, original_work_hash);
					BlockPreparationStatus::Succeeded
				},
				None => BlockPreparationStatus::Failed,
			}
		} else {
			BlockPreparationStatus::NotPrepared
		};

		let best_number = client.chain_info().best_block_number;
		let mut sealing = self.sealing.lock();
		if sealing.last_request != Some(best_number) {
			trace!(
				target: "miner",
				"prepare_pending_block: Miner received request (was {}, now {}) - waking up.",
				sealing.last_request.unwrap_or(0), best_number
			);
			sealing.last_request = Some(best_number);
		}

		preparation_status
	}


	/// Set `sealing.enabled` to true if there is available work to do (pending or in the queue).
	fn maybe_enable_sealing(&self) -> bool {
		let mut sealing = self.sealing.lock();
		if !sealing.work_available() {
			trace!(target: "miner", "maybe_enable_sealing: we have work to do so enabling sealing");
			sealing.enabled = true;
			true
		} else {
			false
		}
	}
	/// Prepare pending block, check whether sealing is needed, and then update sealing.
	fn prepare_and_update_sealing<C: miner::BlockChainClient>(&self, chain: &C) {
		use miner::MinerService;
		match self.engine.sealing_state() {
			SealingState::Ready => self.update_sealing(chain),
			SealingState::External => {
				// this calls `maybe_enable_sealing()`
				if self.prepare_pending_block(chain) == BlockPreparationStatus::NotPrepared {
					self.update_sealing(chain)
				}
			}
			SealingState::NotReady => { self.maybe_enable_sealing(); },
		}
	}
}

impl miner::MinerService for Miner {
	type State = State<::state_db::StateDB>;

	fn authoring_params(&self) -> AuthoringParams {
		self.params.read().clone()
	}

	fn set_gas_range_target(&self, gas_range_target: (U256, U256)) {
		self.params.write().gas_range_target = gas_range_target;
	}

	fn set_extra_data(&self, extra_data: Bytes) {
		self.params.write().extra_data = extra_data;
	}

	fn set_author(&self, author: Author) {
		self.params.write().author = author.address();

		if let Author::Sealer(signer) = author {
			if self.engine.sealing_state() != SealingState::External {
				// Enable sealing
				self.sealing.lock().enabled = true;
				// --------------------------------------------------------------------------
				// | NOTE Code below may require author and sealing locks                   |
				// | (some `Engine`s call `EngineClient.update_sealing()`)                  |
				// | Make sure to release the locks before calling that method.             |
				// --------------------------------------------------------------------------
				self.engine.set_signer(signer);
			} else {
				warn!("Setting an EngineSigner while Engine does not require one.");
			}
		}
	}

	fn sensible_gas_price(&self) -> U256 {
		// 10% above our minimum.
		self.transaction_queue.current_worst_gas_price() * 110u32 / 100
	}

	fn sensible_gas_limit(&self) -> U256 {
		self.params.read().gas_range_target.0 / 5
	}

	fn set_minimal_gas_price(&self, new_price: U256) -> Result<bool, &str> {
		match *self.gas_pricer.lock() {
			// Binding the gas pricer to `gp` here to prevent
			// a deadlock when calling recalibrate()
			ref mut gp @ GasPricer::Fixed(_) => {
				trace!(target: "miner", "minimal_gas_price: recalibrating fixed...");
				*gp = GasPricer::new_fixed(new_price);

				let txq = self.transaction_queue.clone();
				let mut options = self.options.pool_verification_options.clone();
				gp.recalibrate(move |gas_price| {
					debug!(target: "miner", "minimal_gas_price: Got gas price! {}", gas_price);
					options.minimal_gas_price = gas_price;
					txq.set_verifier_options(options);
				});

				Ok(true)
			},
			#[cfg(feature = "price-info")]
			GasPricer::Calibrated(_) => {
				let error_msg = "Can't update fixed gas price while automatic gas calibration is enabled.";
				return Err(error_msg);
			},
		}
	}

	fn import_external_transactions<C: miner::BlockChainClient>(
		&self,
		chain: &C,
		transactions: Vec<UnverifiedTransaction>
	) -> Vec<Result<(), transaction::Error>> {
		trace!(target: "external_tx", "Importing external transactions");
		let client = self.pool_client(chain);
		let results = self.transaction_queue.import(
			client,
			transactions.into_iter().map(pool::verifier::Transaction::Unverified),
		);

		// --------------------------------------------------------------------------
		// | NOTE Code below requires sealing locks.                                |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
		if !results.is_empty() && self.options.reseal_on_external_tx &&	self.sealing.lock().reseal_allowed() {
			self.prepare_and_update_sealing(chain);
		}

		results
	}

	fn import_own_transaction<C: miner::BlockChainClient>(
		&self,
		chain: &C,
		pending: PendingTransaction
	) -> Result<(), transaction::Error> {
		// note: you may want to use `import_claimed_local_transaction` instead of this one.

		trace!(target: "own_tx", "Importing transaction: {:?}", pending);

		let client = self.pool_client(chain);
		let imported = self.transaction_queue.import(
			client,
			Some(pool::verifier::Transaction::Local(pending))
		).pop().expect("one result returned per added transaction; one added => one result; qed");

		// --------------------------------------------------------------------------
		// | NOTE Code below requires sealing locks.                                |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
		if imported.is_ok() && self.options.reseal_on_own_tx && self.sealing.lock().reseal_allowed() {
			self.prepare_and_update_sealing(chain);
		}

		imported
	}

	fn import_claimed_local_transaction<C: miner::BlockChainClient>(
		&self,
		chain: &C,
		pending: PendingTransaction,
		trusted: bool
	) -> Result<(), transaction::Error> {
		// treat the tx as local if the option is enabled, if we have the account, or if
		// the account is specified as a Prioritized Local Addresses
		let sender = pending.sender();
		let treat_as_local = trusted
			|| !self.options.tx_queue_no_unfamiliar_locals
			|| self.accounts.is_local(&sender);

		if treat_as_local {
			self.import_own_transaction(chain, pending)
		} else {
			// We want to replicate behaviour for external transactions if we're not going to treat
			// this as local. This is important with regards to sealing blocks
			self.import_external_transactions(chain, vec![pending.transaction.into()])
				.pop().expect("one result per tx, as in `import_own_transaction`")
		}
	}

	fn local_transactions(&self) -> BTreeMap<H256, pool::local_transactions::Status> {
		self.transaction_queue.local_transactions()
	}

	fn queued_transactions(&self) -> Vec<Arc<VerifiedTransaction>> {
		self.transaction_queue.all_transactions()
	}

	fn queued_transaction_hashes(&self) -> Vec<H256> {
		self.transaction_queue.all_transaction_hashes()
	}

	fn pending_transaction_hashes<C>(&self, chain: &C) -> BTreeSet<H256> where
		C: ChainInfo + Sync,
	{
		let chain_info = chain.chain_info();

		let from_queue = || self.transaction_queue.pending_hashes(
			|sender| self.nonce_cache.get(sender),
		);

		let from_pending = || {
			self.map_existing_pending_block(|sealing| {
				sealing.transactions
					.iter()
					.map(|signed| signed.hash())
					.collect()
			}, chain_info.best_block_number)
		};

		match self.options.pending_set {
			PendingSet::AlwaysQueue => {
				from_queue()
			},
			PendingSet::AlwaysSealing => {
				from_pending().unwrap_or_default()
			},
			PendingSet::SealingOrElseQueue => {
				from_pending().unwrap_or_else(from_queue)
			},
		}
	}

	fn ready_transactions<C>(&self, chain: &C, max_len: usize, ordering: miner::PendingOrdering)
		-> Vec<Arc<VerifiedTransaction>>
	where
		C: ChainInfo + Nonce + Sync,
	{
		// No special filtering options applied (neither tx_hash, receiver or sender)
		self.ready_transactions_filtered(chain, max_len, None, ordering)
	}

	fn ready_transactions_filtered<C>(
		&self,
		chain: &C,
		max_len: usize,
		filter: Option<FilterOptions>,
		ordering: miner::PendingOrdering,
	) -> Vec<Arc<VerifiedTransaction>> where
		C: ChainInfo + Nonce + Sync,
	{
		let chain_info = chain.chain_info();

		let from_queue = || {
			// We propagate transactions over the nonce cap.
			// The mechanism is only to limit number of transactions in pending block
			// those transactions are valid and will just be ready to be included in next block.
			let nonce_cap = None;

			self.transaction_queue.pending(
				CachedNonceClient::new(chain, &self.nonce_cache),
				pool::PendingSettings {
					block_number: chain_info.best_block_number,
					current_timestamp: chain_info.best_block_timestamp,
					nonce_cap,
					max_len,
					ordering,
				},
			)
		};

		use miner::filter_options::FilterOperator::*;
		let from_pending = || {
			self.map_existing_pending_block(|sealing| {
				// This filter is used for gas, gas price, value and nonce.
				// Sender and receiver have their custom matches, since those
				// allow/disallow different operators.
				fn match_common_filter(operator: &FilterOperator<U256>, tx_value: &U256) -> bool {
					match operator {
						Eq(value) => tx_value == value,
						GreaterThan(value) => tx_value > value,
						LessThan(value) => tx_value < value,
						// Will always occure on `Any`, other operators
						// get handled during deserialization
						_ => true,
					}
				}

				sealing.transactions
					.iter()
					.map(|signed| pool::VerifiedTransaction::from_pending_block_transaction(signed.clone()))
					.map(Arc::new)
					// Filter by sender
					.filter(|tx| {
						filter.as_ref().map_or(true, |filter| {
							let sender = tx.signed().sender();
							match filter.from {
								Eq(value) => sender == value,
								// Will always occure on `Any`, other operators
								// get handled during deserialization
								_ => true,
							}
						})
					})
					// Filter by receiver
					.filter(|tx| {
						filter.as_ref().map_or(true, |filter| {
							let receiver = (*tx.signed()).receiver();
							match filter.to {
								// Could apply to `Some(Address)` or `None` (for contract creation)
								Eq(value) => receiver == value,
								// Will always occure on `Any`, other operators
								// get handled during deserialization
								_ => true,
							}
						})
					})
					// Filter by gas
					.filter(|tx| {
						filter.as_ref().map_or(true, |filter| {
							match_common_filter(&filter.gas, &(*tx.signed()).gas)
						})
					})
					// Filter by gas price
					.filter(|tx| {
						filter.as_ref().map_or(true, |filter| {
							match_common_filter(&filter.gas_price, &(*tx.signed()).gas_price)
						})
					})
					// Filter by tx value
					.filter(|tx| {
						filter.as_ref().map_or(true, |filter| {
							match_common_filter(&filter.value, &(*tx.signed()).value)
						})
					})
					// Filter by nonce
					.filter(|tx| {
						filter.as_ref().map_or(true, |filter| {
							match_common_filter(&filter.nonce, &(*tx.signed()).nonce)
						})
					})
					.take(max_len)
					.collect()
			}, chain_info.best_block_number)
		};

		match self.options.pending_set {
			PendingSet::AlwaysQueue => {
				from_queue()
			},
			PendingSet::AlwaysSealing => {
				from_pending().unwrap_or_default()
			},
			PendingSet::SealingOrElseQueue => {
				from_pending().unwrap_or_else(from_queue)
			},
		}
	}

	fn next_nonce<C>(&self, chain: &C, address: &Address) -> U256 where
		C: Nonce + Sync,
	{
		self.transaction_queue.next_nonce(CachedNonceClient::new(chain, &self.nonce_cache), address)
			.unwrap_or_else(|| chain.latest_nonce(address))
	}

	fn transaction(&self, hash: &H256) -> Option<Arc<VerifiedTransaction>> {
		self.transaction_queue.find(hash)
	}

	fn remove_transaction(&self, hash: &H256) -> Option<Arc<VerifiedTransaction>> {
		self.transaction_queue.remove(::std::iter::once(hash), false)
			.pop()
			.expect("remove() returns one result per hash; one hash passed; qed")
	}

	fn queue_status(&self) -> QueueStatus {
		self.transaction_queue.status()
	}

	fn pending_receipts(&self, best_block: BlockNumber) -> Option<Vec<RichReceipt>> {
		self.map_existing_pending_block(|pending| {
			let receipts = &pending.receipts;
			pending.transactions
				.iter()
				.enumerate()
				.map(|(index, tx)| {
					let prev_gas = if index == 0 { Default::default() } else { receipts[index - 1].gas_used };
					let receipt = &receipts[index];
					RichReceipt {
						transaction_hash: tx.hash(),
						transaction_index: index,
						cumulative_gas_used: receipt.gas_used,
						gas_used: receipt.gas_used - prev_gas,
						contract_address: match tx.action {
							Action::Call(_) => None,
							Action::Create => {
								let sender = tx.sender();
								Some(contract_address(CreateContractAddress::FromSenderAndNonce, &sender, &tx.nonce, &tx.data).0)
							}
						},
						logs: receipt.logs.clone(),
						log_bloom: receipt.log_bloom,
						outcome: receipt.outcome.clone(),
					}
				})
				.collect()
		}, best_block)
	}

	/// Update sealing if required.
	/// Prepare the block and work if the Engine does not seal internally.
	fn update_sealing<C>(&self, chain: &C) where
		C: BlockChain + CallContract + BlockProducer + SealedBlockImporter + Nonce + Sync,
	{
		trace!(target: "miner", "update_sealing");

		// Do nothing if reseal is not required,
		// but note that `requires_reseal` updates internal state.
		if !self.requires_reseal(chain.chain_info().best_block_number) {
			return;
		}

		let sealing_state = self.engine.sealing_state();
		if sealing_state == SealingState::NotReady {
			return;
		}

		// --------------------------------------------------------------------------
		// | NOTE Code below requires sealing locks.                                |
		// | Make sure to release the locks before calling that method.             |
		// --------------------------------------------------------------------------
		trace!(target: "miner", "update_sealing: preparing a block");
		let (block, original_work_hash) = match self.prepare_block(chain) {
			Some((block, original_work_hash)) => (block, original_work_hash),
			None => return,
		};

		// refuse to seal the first block of the chain if it contains hard forks
		// which should be on by default.
		if block.header.number() == 1 {
			if let Some(name) = self.engine.params().nonzero_bugfix_hard_fork() {
				warn!("Your chain specification contains one or more hard forks which are required to be \
						on by default. Please remove these forks and start your chain again: {}.", name);
				return;
			}
		}

		match sealing_state {
			SealingState::Ready => {
				trace!(target: "miner", "update_sealing: engine indicates internal sealing");
				if self.seal_and_import_block_internally(chain, block) {
					trace!(target: "miner", "update_sealing: imported internally sealed block");
				}
			},
			SealingState::NotReady => unreachable!("We returned right after sealing_state was computed. qed."),
			SealingState::External => {
				trace!(target: "miner", "update_sealing: engine does not seal internally, preparing work");
				self.prepare_work(block, original_work_hash)
			},
		}
	}

	fn is_currently_sealing(&self) -> bool {
		self.sealing.lock().enabled
	}

	fn work_package<C>(&self, chain: &C) -> Option<(H256, BlockNumber, u64, U256)> where
		C: BlockChain + CallContract + BlockProducer + SealedBlockImporter + Nonce + Sync,
	{
		if self.engine.sealing_state() != SealingState::External {
			return None;
		}

		self.prepare_pending_block(chain);

		self.sealing.lock().queue.use_last_ref().map(|b| {
			let header = &b.header;
			(header.hash(), header.number(), header.timestamp(), *header.difficulty())
		})
	}

	// Note used for external submission (PoW) and internally by sealing engines.
	fn submit_seal(&self, block_hash: H256, seal: Vec<Bytes>) -> Result<SealedBlock, Error> {
		let action = if self.options.enable_resubmission {
			GetAction::Clone
		} else {
			GetAction::Take
		};

		let block = self.sealing.lock().queue
			.get_used_if(action, |b| &b.header.bare_hash() == &block_hash)
			.ok_or_else(|| {
				warn!(target: "miner", "Submitted solution rejected: Block unknown or out of date.");
				Error::PowHashInvalid
			})?;

		trace!(
			target: "miner", "Submitted block {hash}={bare_hash} with seal {seal:?}",
			hash = block_hash,
			bare_hash = block.header.bare_hash(),
			seal = seal
		);

		let sealed = block.lock()
			.try_seal(&*self.engine, seal)
			.map_err(|e| {
				warn!(target: "miner", "Mined solution rejected: {}", e);
				Error::PowInvalid
			})?;

		let n = sealed.header.number();
		let h = sealed.header.hash();

		info!(
			target: "miner", "Submitted block imported OK. #{number}: {hash}",
			number = Colour::White.bold().paint(n.to_string()),
			hash = Colour::White.bold().paint(format!("{:x}", h))
		);

		Ok(sealed)
	}

	fn chain_new_blocks<C>(&self, chain: &C, imported: &[H256], _invalid: &[H256], enacted: &[H256], retracted: &[H256], is_internal_import: bool)
		where C: miner::BlockChainClient,
	{
		trace!(target: "miner", "chain_new_blocks");

		// 1. We ignore blocks that were `imported` unless resealing on new uncles is enabled.
		// 2. We ignore blocks that are `invalid` because it doesn't have any meaning in terms of the transactions that
		//    are in those blocks

		let has_new_best_block = enacted.len() > 0;

		if has_new_best_block {
			// Clear nonce cache
			self.nonce_cache.clear();
		}

		// First update gas limit in transaction queue and minimal gas price.
		let gas_limit = *chain.best_block_header().gas_limit();
		self.update_transaction_queue_limits(gas_limit);

		// Then import all transactions from retracted blocks.
		let client = self.pool_client(chain);
		{
			retracted
				.par_iter()
				.for_each(|hash| {
					let block = chain.block(BlockId::Hash(*hash))
						.expect("Client is sending message after commit to db and inserting to chain; the block is available; qed");
					let txs = block.transactions()
						.into_iter()
						.map(pool::verifier::Transaction::Retracted);
					let _ = self.transaction_queue.import(
						client.clone(),
						txs,
					);
				});
		}

		if has_new_best_block || (imported.len() > 0 && self.options.reseal_on_uncle) {
			// Reset `next_allowed_reseal` in case a block is imported.
			// Even if min_period is high, we will always attempt to create
			// new pending block.
			self.sealing.lock().next_allowed_reseal = Instant::now();

			if !is_internal_import {
				// --------------------------------------------------------------------------
				// | NOTE Code below requires sealing locks.                                |
				// | Make sure to release the locks before calling that method.             |
				// --------------------------------------------------------------------------
				self.update_sealing(chain);
			}
		}

		if has_new_best_block {
			// Make sure to cull transactions after we update sealing.
			// Not culling won't lead to old transactions being added to the block
			// (thanks to Ready), but culling can take significant amount of time,
			// so best to leave it after we create some work for miners to prevent increased
			// uncle rate.
			// If the io_channel is available attempt to offload culling to a separate task
			// to avoid blocking chain_new_blocks
			if let Some(ref channel) = *self.io_channel.read() {
				let queue = self.transaction_queue.clone();
				let nonce_cache = self.nonce_cache.clone();
				let engine = self.engine.clone();
				let accounts = self.accounts.clone();
				let service_transaction_checker = self.service_transaction_checker.clone();

				let cull = move |chain: &::client::Client| {
					let client = PoolClient::new(
						chain,
						&nonce_cache,
						&*engine,
						&*accounts,
						service_transaction_checker.as_ref(),
					);
					queue.cull(client);
				};

				if let Err(e) = channel.send(ClientIoMessage::execute(cull)) {
					warn!(target: "miner", "Error queueing cull: {:?}", e);
				}
			} else {
				self.transaction_queue.cull(client);
			}
		}
		if let Some(ref service_transaction_checker) = self.service_transaction_checker {
			match service_transaction_checker.refresh_cache(chain) {
				Ok(true) => {
					trace!(target: "client", "Service transaction cache was refreshed successfully");
				},
				Ok(false) => {
					trace!(target: "client", "Registrar or/and service transactions contract does not exist");
				},
				Err(e) => error!(target: "client", "Error occurred while refreshing service transaction cache: {}", e)
			};
		};
	}

	fn pending_state(&self, latest_block_number: BlockNumber) -> Option<Self::State> {
		self.map_existing_pending_block(|b| b.state.clone(), latest_block_number)
	}

	fn pending_block_header(&self, latest_block_number: BlockNumber) -> Option<Header> {
		self.map_existing_pending_block(|b| b.header.clone(), latest_block_number)
	}

	fn pending_block(&self, latest_block_number: BlockNumber) -> Option<Block> {
		self.map_existing_pending_block(|b| {
			Block {
				header: b.header.clone(),
				transactions: b.transactions.iter().cloned().map(Into::into).collect(),
				uncles: b.uncles.to_vec(),
			}
		}, latest_block_number)
	}

	fn pending_transactions(&self, latest_block_number: BlockNumber) -> Option<Vec<SignedTransaction>> {
		self.map_existing_pending_block(|b| b.transactions.iter().cloned().collect(), latest_block_number)
	}
}

#[cfg(test)]
mod tests {
	use std::iter::FromIterator;

	use super::*;
	use accounts::AccountProvider;
	use ethkey::{Generator, Random};
	use hash::keccak;
	use rustc_hex::FromHex;
	use types::BlockNumber;

	use client::{TestBlockChainClient, EachBlockWith, ChainInfo, ImportSealedBlock};
	use miner::{MinerService, PendingOrdering};
	use test_helpers::{generate_dummy_client, generate_dummy_client_with_spec};
	use types::transaction::{Transaction};

	#[test]
	fn should_prepare_block_to_seal() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::new_for_tests(&Spec::new_test(), None);

		// when
		let sealing_work = miner.work_package(&client);
		assert!(sealing_work.is_some(), "Expected closed block");
	}

	#[test]
	fn should_still_work_after_a_couple_of_blocks() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::new_for_tests(&Spec::new_test(), None);

		let res = miner.work_package(&client);
		let hash = res.unwrap().0;
		let block = miner.submit_seal(hash, vec![]).unwrap();
		client.import_sealed_block(block).unwrap();

		// two more blocks mined, work requested.
		client.add_blocks(1, EachBlockWith::Uncle);
		miner.work_package(&client);

		client.add_blocks(1, EachBlockWith::Uncle);
		miner.work_package(&client);

		// solution to original work submitted.
		assert!(miner.submit_seal(hash, vec![]).is_ok());
	}

	fn miner() -> Miner {
		Miner::new(
			MinerOptions {
				force_sealing: false,
				reseal_on_external_tx: false,
				reseal_on_own_tx: true,
				reseal_on_uncle: false,
				reseal_min_period: Duration::from_secs(5),
				reseal_max_period: Duration::from_secs(120),
				pending_set: PendingSet::AlwaysSealing,
				work_queue_size: 5,
				enable_resubmission: true,
				infinite_pending_block: false,
				tx_queue_penalization: Penalization::Disabled,
				tx_queue_strategy: PrioritizationStrategy::GasPriceOnly,
				tx_queue_no_unfamiliar_locals: false,
				refuse_service_transactions: false,
				pool_limits: Default::default(),
				pool_verification_options: pool::verifier::Options {
					minimal_gas_price: 0.into(),
					block_gas_limit: U256::max_value(),
					tx_gas_limit: U256::max_value(),
					no_early_reject: false,
				},
			},
			GasPricer::new_fixed(0u64.into()),
			&Spec::new_test(),
			::std::collections::HashSet::new(), // local accounts
		)
	}

	const TEST_CHAIN_ID: u64 = 2;

	fn transaction() -> SignedTransaction {
		transaction_with_chain_id(TEST_CHAIN_ID)
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
		assert_eq!(res.unwrap(), ());
		assert_eq!(miner.pending_transactions(best_block).unwrap().len(), 1);
		assert_eq!(miner.pending_receipts(best_block).unwrap().len(), 1);
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 1);
		// This method will let us know if pending block was created (before calling that method)
		assert_eq!(miner.prepare_pending_block(&client), BlockPreparationStatus::NotPrepared);
	}

	#[test]
	fn should_not_return_stale_work_packages() {
		// given
		let client = TestBlockChainClient::default();
		let miner = miner();

		// initial work package should create the pending block
		let res = miner.work_package(&client);
		assert_eq!(res.unwrap().1, 1);
		// This should be true, since there were some requests.
		assert_eq!(miner.requires_reseal(0), true);

		// when new block is imported
		let client = generate_dummy_client(2);
		let imported = [H256::zero()];
		let empty = &[];
		miner.chain_new_blocks(&*client, &imported, empty, &imported, empty, false);

		// then
		// This should be false, because it's too early.
		assert_eq!(miner.requires_reseal(2), false);
		// but still work package should be ready
		let res = miner.work_package(&*client);
		assert_eq!(res.unwrap().1, 3);
		assert_eq!(miner.prepare_pending_block(&*client), BlockPreparationStatus::NotPrepared);
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
		assert_eq!(res.unwrap(), ());
		assert_eq!(miner.pending_transactions(best_block), None);
		assert_eq!(miner.pending_receipts(best_block), None);
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 1);
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
		assert_eq!(res.unwrap(), ());
		// By default we don't reseal on external transactions
		assert_eq!(miner.pending_transactions(best_block), None);
		assert_eq!(miner.pending_receipts(best_block), None);
		// By default we use PendingSet::AlwaysSealing, so no transactions yet.
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 0);
		// This method will let us know if pending block was created (before calling that method)
		assert_eq!(miner.prepare_pending_block(&client), BlockPreparationStatus::Succeeded);
		// After pending block is created we should see a transaction.
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 1);
	}

	#[test]
	fn should_treat_unfamiliar_locals_selectively() {
		// given
		let keypair = Random.generate().unwrap();
		let client = TestBlockChainClient::default();
		let mut local_accounts = ::std::collections::HashSet::new();
		local_accounts.insert(keypair.address());

		let miner = Miner::new(
			MinerOptions {
				tx_queue_no_unfamiliar_locals: true,
				..miner().options
			},
			GasPricer::new_fixed(0u64.into()),
			&Spec::new_test(),
			local_accounts,
		);
		let transaction = transaction();
		let best_block = 0;
		// when
		// This transaction should not be marked as local because our account_provider doesn't have the sender
		let res = miner.import_claimed_local_transaction(&client, PendingTransaction::new(transaction.clone(), None), false);

		// then
		// Check the same conditions as `should_import_external_transaction` first. Behaviour should be identical.
		// That is: it's treated as though we added it through `import_external_transactions`
		assert_eq!(res.unwrap(), ());
		assert_eq!(miner.pending_transactions(best_block), None);
		assert_eq!(miner.pending_receipts(best_block), None);
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 0);
		assert_eq!(miner.prepare_pending_block(&client), BlockPreparationStatus::Succeeded);
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 1);

		// when - 2nd part: create a local transaction from account_provider.
		// Borrow the transaction used before & sign with our generated keypair.
		let local_transaction = transaction.deconstruct().0.as_unsigned().clone().sign(keypair.secret(), Some(TEST_CHAIN_ID));
		let res2 = miner.import_claimed_local_transaction(&client, PendingTransaction::new(local_transaction, None), false);

		// then - 2nd part: we add on the results from the last pending block.
		// This is borrowed from `should_make_pending_block_when_importing_own_transaction` and slightly modified.
		assert_eq!(res2.unwrap(), ());
		assert_eq!(miner.pending_transactions(best_block).unwrap().len(), 2);
		assert_eq!(miner.pending_receipts(best_block).unwrap().len(), 2);
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 2);
		assert_eq!(miner.prepare_pending_block(&client), BlockPreparationStatus::NotPrepared);
	}

	#[test]
	fn should_prioritize_locals() {
		let client = TestBlockChainClient::default();
		let transaction = transaction();
		let miner = Miner::new(
			MinerOptions {
				tx_queue_no_unfamiliar_locals: true, // should work even with this enabled
				..miner().options
			},
			GasPricer::new_fixed(0u64.into()),
			&Spec::new_test(),
			HashSet::from_iter(vec![transaction.sender()].into_iter()),
		);
		let best_block = 0;

		// Miner with sender as a known local address should prioritize transactions from that address
		let res2 = miner.import_claimed_local_transaction(&client, PendingTransaction::new(transaction, None), false);

		// check to make sure the prioritized transaction is pending
		assert_eq!(res2.unwrap(), ());
		assert_eq!(miner.pending_transactions(best_block).unwrap().len(), 1);
		assert_eq!(miner.pending_receipts(best_block).unwrap().len(), 1);
		assert_eq!(miner.ready_transactions(&client, 10, PendingOrdering::Priority).len(), 1);
		assert_eq!(miner.prepare_pending_block(&client), BlockPreparationStatus::NotPrepared);
	}

	#[test]
	fn should_not_seal_unless_enabled() {
		let miner = miner();
		let client = TestBlockChainClient::default();
		// By default resealing is not required.
		assert!(!miner.requires_reseal(1u8.into()));

		miner.import_external_transactions(&client, vec![transaction().into()]).pop().unwrap().unwrap();
		assert_eq!(miner.prepare_pending_block(&client), BlockPreparationStatus::Succeeded);
		// Unless asked to prepare work.
		assert!(miner.requires_reseal(1u8.into()));
	}

	#[test]
	fn internal_seals_without_work() {
		let _ = env_logger::try_init();
		let spec = Spec::new_instant();
		let miner = Miner::new_for_tests(&spec, None);

		let client = generate_dummy_client(2);

		let import = miner.import_external_transactions(
			&*client,
			vec![transaction_with_chain_id(spec.chain_id()).into()]
		).pop().unwrap();
		assert_eq!(import.unwrap(), ());

		miner.update_sealing(&*client);
		client.flush_queue();
		assert!(miner.pending_block(0).is_none());
		assert_eq!(client.chain_info().best_block_number, 3 as BlockNumber);

		assert!(miner.import_own_transaction(
			&*client,
			PendingTransaction::new(transaction_with_chain_id(spec.chain_id()).into(), None)
		).is_ok());

		miner.update_sealing(&*client);
		client.flush_queue();
		assert!(miner.pending_block(0).is_none());
		assert_eq!(client.chain_info().best_block_number, 4 as BlockNumber);
	}

	#[test]
	fn should_not_fail_setting_engine_signer_without_account_provider() {
		let spec = Spec::new_test_round;
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr = tap.insert_account(keccak("1").into(), &"".into()).unwrap();
		let client = generate_dummy_client_with_spec(spec);
		let engine_signer = Box::new((tap.clone(), addr, "".into()));
		let msg = Default::default();
		assert!(client.engine().sign(msg).is_err());

		// should set engine signer and miner author
		client.miner().set_author(Author::Sealer(engine_signer));
		assert_eq!(client.miner().authoring_params().author, addr);
		assert!(client.engine().sign(msg).is_ok());
	}

	#[test]
	fn should_mine_if_internal_sealing_is_enabled() {
		let spec = Spec::new_instant();
		let miner = Miner::new_for_tests(&spec, None);

		let client = generate_dummy_client(2);
		miner.update_sealing(&*client);

		assert!(miner.is_currently_sealing());
	}

	#[test]
	fn should_not_mine_if_internal_sealing_is_disabled() {
		let spec = Spec::new_test_round();
		let miner = Miner::new_for_tests(&spec, None);

		let client = generate_dummy_client(2);
		miner.update_sealing(&*client);

		assert!(!miner.is_currently_sealing());
	}

	#[test]
	fn should_not_mine_if_no_fetch_work_request() {
		let spec = Spec::new_test();
		let miner = Miner::new_for_tests(&spec, None);

		let client = generate_dummy_client(2);
		miner.update_sealing(&*client);

		assert!(!miner.is_currently_sealing());
	}

	#[cfg(feature = "work-notify")]
	#[test]
	fn should_mine_if_fetch_work_request() {
		struct DummyNotifyWork;

		impl NotifyWork for DummyNotifyWork {
			fn notify(&self, _pow_hash: H256, _difficulty: U256, _number: u64) { }
		}

		let spec = Spec::new_test();
		let miner = Miner::new_for_tests(&spec, None);
		miner.add_work_listener(Box::new(DummyNotifyWork));

		let client = generate_dummy_client(2);
		miner.update_sealing(&*client);

		assert!(miner.is_currently_sealing());
	}

	#[test]
	fn should_set_new_minimum_gas_price() {
		// Creates a new GasPricer::Fixed behind the scenes
		let miner = Miner::new_for_tests(&Spec::new_test(), None);

		let expected_minimum_gas_price: U256 = 0x1337.into();
		miner.set_minimal_gas_price(expected_minimum_gas_price).unwrap();

		let txq_options = miner.transaction_queue.status().options;
		let current_minimum_gas_price = txq_options.minimal_gas_price;

		assert!(current_minimum_gas_price == expected_minimum_gas_price);
	}

	#[cfg(feature = "price-info")]
	fn dynamic_gas_pricer() -> GasPricer {
		use parity_runtime::Executor;
		use fetch::Client as FetchClient;
		use ethcore_miner::gas_price_calibrator::{GasPriceCalibrator, GasPriceCalibratorOptions};

		// Don't really care about any of these settings since
		// the gas pricer is never actually going to be used
		let fetch = FetchClient::new(1).unwrap();
		let p = Executor::new_sync();

		GasPricer::new_calibrated(
			GasPriceCalibrator::new(
				GasPriceCalibratorOptions {
					usd_per_tx: 0.0,
					recalibration_period: Duration::from_secs(0),
				},
				fetch,
				p,
			)
		)
	}

	#[test]
	#[cfg(feature = "price-info")]
	fn should_fail_to_set_new_minimum_gas_price() {
		// We get a fixed gas pricer by default, need to change that
		let miner = Miner::new_for_tests(&Spec::new_test(), None);
		let calibrated_gas_pricer = dynamic_gas_pricer();
		*miner.gas_pricer.lock() = calibrated_gas_pricer;

		let expected_minimum_gas_price: U256 = 0x1337.into();
		let result = miner.set_minimal_gas_price(expected_minimum_gas_price);
		assert!(result.is_err());

		let received_error_msg = result.unwrap_err();
		let expected_error_msg = "Can't update fixed gas price while automatic gas calibration is enabled.";

		assert!(received_error_msg == expected_error_msg);
	}
}
