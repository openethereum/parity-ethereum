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

use std::collections::{HashSet, HashMap, BTreeMap, VecDeque};
use std::str::FromStr;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering as AtomicOrdering};
use std::time::{Instant};
use time::precise_time_ns;
use itertools::Itertools;

// util
use hash::keccak;
use timer::PerfTimer;
use bytes::Bytes;
use util::{Address, DBValue};
use journaldb;
use util_error::UtilError;
use trie::{TrieSpec, TrieFactory, Trie};
use kvdb::{KeyValueDB, DBTransaction};

// other
use bigint::prelude::U256;
use bigint::hash::H256;
use basic_types::Seal;
use block::*;
use blockchain::{BlockChain, BlockProvider,  TreeRoute, ImportRoute};
use blockchain::extras::TransactionAddress;
use client::ancient_import::AncientVerifier;
use client::Error as ClientError;
use client::{
	BlockId, TransactionId, UncleId, TraceId, ClientConfig, BlockChainClient,
	MiningBlockChainClient, TraceFilter, CallAnalytics, BlockImportError, Mode,
	ChainNotify, PruningInfo, ProvingBlockChainClient,
};
use encoded;
use engines::{EthEngine, EpochTransition};
use error::{ImportError, ExecutionError, CallError, BlockError, ImportResult, Error as EthcoreError};
use vm::{EnvInfo, LastHashes};
use evm::{Factory as EvmFactory, Schedule};
use executive::{Executive, Executed, TransactOptions, contract_address};
use factory::Factories;
use futures::{future, Future};
use header::{BlockNumber, Header};
use io::*;
use log_entry::LocalizedLogEntry;
use miner::{Miner, MinerService, TransactionImportResult};
use native_contracts::Registry;
use parking_lot::{Mutex, RwLock, MutexGuard};
use rand::OsRng;
use receipt::{Receipt, LocalizedReceipt};
use rlp::UntrustedRlp;
use service::ClientIoMessage;
use snapshot::{self, io as snapshot_io};
use spec::Spec;
use state_db::StateDB;
use state::{self, State};
use trace;
use trace::{TraceDB, ImportRequest as TraceImportRequest, LocalizedTrace, Database as TraceDatabase};
use trace::FlatTransactionTraces;
use transaction::{LocalizedTransaction, UnverifiedTransaction, SignedTransaction, Transaction, PendingTransaction, Action};
use types::filter::Filter;
use types::mode::Mode as IpcMode;
use verification;
use verification::{PreverifiedBlock, Verifier};
use verification::queue::BlockQueue;
use views::BlockView;

// re-export
pub use types::blockchain_info::BlockChainInfo;
pub use types::block_status::BlockStatus;
pub use blockchain::CacheSize as BlockChainCacheSize;
pub use verification::queue::QueueInfo as BlockQueueInfo;

const MAX_TX_QUEUE_SIZE: usize = 4096;
const MAX_QUEUE_SIZE_TO_SLEEP_ON: usize = 2;
const MIN_HISTORY_SIZE: u64 = 8;

/// Report on the status of a client.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct ClientReport {
	/// How many blocks have been imported so far.
	pub blocks_imported: usize,
	/// How many transactions have been applied so far.
	pub transactions_applied: usize,
	/// How much gas has been processed so far.
	pub gas_processed: U256,
	/// Memory used by state DB
	pub state_db_mem: usize,
}

impl ClientReport {
	/// Alter internal reporting to reflect the additional `block` has been processed.
	pub fn accrue_block(&mut self, block: &PreverifiedBlock) {
		self.blocks_imported += 1;
		self.transactions_applied += block.transactions.len();
		self.gas_processed = self.gas_processed + block.header.gas_used().clone();
	}
}

impl<'a> ::std::ops::Sub<&'a ClientReport> for ClientReport {
	type Output = Self;

	fn sub(mut self, other: &'a ClientReport) -> Self {
		let higher_mem = ::std::cmp::max(self.state_db_mem, other.state_db_mem);
		let lower_mem = ::std::cmp::min(self.state_db_mem, other.state_db_mem);

		self.blocks_imported -= other.blocks_imported;
		self.transactions_applied -= other.transactions_applied;
		self.gas_processed = self.gas_processed - other.gas_processed;
		self.state_db_mem  = higher_mem - lower_mem;

		self
	}
}

struct SleepState {
	last_activity: Option<Instant>,
	last_autosleep: Option<Instant>,
}

impl SleepState {
	fn new(awake: bool) -> Self {
		SleepState {
			last_activity: match awake { false => None, true => Some(Instant::now()) },
			last_autosleep: match awake { false => Some(Instant::now()), true => None },
		}
	}
}

/// Blockchain database client backed by a persistent database. Owns and manages a blockchain and a block queue.
/// Call `import_block()` to import a block asynchronously; `flush_queue()` flushes the queue.
pub struct Client {
	enabled: AtomicBool,
	mode: Mutex<Mode>,
	chain: RwLock<Arc<BlockChain>>,
	tracedb: RwLock<TraceDB<BlockChain>>,
	engine: Arc<EthEngine>,
	config: ClientConfig,
	pruning: journaldb::Algorithm,
	db: RwLock<Arc<KeyValueDB>>,
	state_db: Mutex<StateDB>,
	block_queue: BlockQueue,
	report: RwLock<ClientReport>,
	import_lock: Mutex<()>,
	verifier: Box<Verifier>,
	miner: Arc<Miner>,
	sleep_state: Mutex<SleepState>,
	liveness: AtomicBool,
	io_channel: Mutex<IoChannel<ClientIoMessage>>,
	notify: RwLock<Vec<Weak<ChainNotify>>>,
	queue_transactions: AtomicUsize,
	last_hashes: RwLock<VecDeque<H256>>,
	factories: Factories,
	history: u64,
	rng: Mutex<OsRng>,
	ancient_verifier: Mutex<Option<AncientVerifier>>,
	on_user_defaults_change: Mutex<Option<Box<FnMut(Option<Mode>) + 'static + Send>>>,
	registrar: Mutex<Option<Registry>>,
	exit_handler: Mutex<Option<Box<Fn(bool, Option<String>) + 'static + Send>>>,
}

impl Client {
	/// Create a new client with given parameters.
	/// The database is assumed to have been initialized with the correct columns.
	pub fn new(
		config: ClientConfig,
		spec: &Spec,
		db: Arc<KeyValueDB>,
		miner: Arc<Miner>,
		message_channel: IoChannel<ClientIoMessage>,
	) -> Result<Arc<Client>, ::error::Error> {
		let trie_spec = match config.fat_db {
			true => TrieSpec::Fat,
			false => TrieSpec::Secure,
		};

		let trie_factory = TrieFactory::new(trie_spec);
		let factories = Factories {
			vm: EvmFactory::new(config.vm_type.clone(), config.jump_table_size),
			trie: trie_factory,
			accountdb: Default::default(),
		};

		let journal_db = journaldb::new(db.clone(), config.pruning, ::db::COL_STATE);
		let mut state_db = StateDB::new(journal_db, config.state_cache_size);
		if state_db.journal_db().is_empty() {
			// Sets the correct state root.
			state_db = spec.ensure_db_good(state_db, &factories)?;
			let mut batch = DBTransaction::new();
			state_db.journal_under(&mut batch, 0, &spec.genesis_header().hash())?;
			db.write(batch).map_err(ClientError::Database)?;
		}

		let gb = spec.genesis_block();
		let chain = Arc::new(BlockChain::new(config.blockchain.clone(), &gb, db.clone()));
		let tracedb = RwLock::new(TraceDB::new(config.tracing.clone(), db.clone(), chain.clone()));

		trace!("Cleanup journal: DB Earliest = {:?}, Latest = {:?}", state_db.journal_db().earliest_era(), state_db.journal_db().latest_era());

		let history = if config.history < MIN_HISTORY_SIZE {
			info!(target: "client", "Ignoring pruning history parameter of {}\
				, falling back to minimum of {}",
				config.history, MIN_HISTORY_SIZE);
			MIN_HISTORY_SIZE
		} else {
			config.history
		};

		if !chain.block_header(&chain.best_block_hash()).map_or(true, |h| state_db.journal_db().contains(h.state_root())) {
			warn!("State root not found for block #{} ({})", chain.best_block_number(), chain.best_block_hash().hex());
		}

		let engine = spec.engine.clone();

		let block_queue = BlockQueue::new(config.queue.clone(), engine.clone(), message_channel.clone(), config.verifier_type.verifying_seal());

		let awake = match config.mode { Mode::Dark(..) | Mode::Off => false, _ => true };

		let client = Arc::new(Client {
			enabled: AtomicBool::new(true),
			sleep_state: Mutex::new(SleepState::new(awake)),
			liveness: AtomicBool::new(awake),
			mode: Mutex::new(config.mode.clone()),
			chain: RwLock::new(chain),
			tracedb: tracedb,
			engine: engine,
			pruning: config.pruning.clone(),
			verifier: verification::new(config.verifier_type.clone()),
			config: config,
			db: RwLock::new(db),
			state_db: Mutex::new(state_db),
			block_queue: block_queue,
			report: RwLock::new(Default::default()),
			import_lock: Mutex::new(()),
			miner: miner,
			io_channel: Mutex::new(message_channel),
			notify: RwLock::new(Vec::new()),
			queue_transactions: AtomicUsize::new(0),
			last_hashes: RwLock::new(VecDeque::new()),
			factories: factories,
			history: history,
			rng: Mutex::new(OsRng::new().map_err(UtilError::from)?),
			ancient_verifier: Mutex::new(None),
			on_user_defaults_change: Mutex::new(None),
			registrar: Mutex::new(None),
			exit_handler: Mutex::new(None),
		});

		// prune old states.
		{
			let state_db = client.state_db.lock().boxed_clone();
			let chain = client.chain.read();
			client.prune_ancient(state_db, &chain)?;
		}

		// ensure genesis epoch proof in the DB.
		{
			let chain = client.chain.read();
			let gh = spec.genesis_header();
			if chain.epoch_transition(0, gh.hash()).is_none() {
				trace!(target: "client", "No genesis transition found.");

				let proof = client.with_proving_caller(
					BlockId::Number(0),
					|call| client.engine.genesis_epoch_data(&gh, call)
				);
				let proof = match proof {
					Ok(proof) => proof,
					Err(e) => {
						warn!(target: "client", "Error generating genesis epoch data: {}. Snapshots generated may not be complete.", e);
						Vec::new()
					}
				};

				debug!(target: "client", "Obtained genesis transition proof: {:?}", proof);

				let mut batch = DBTransaction::new();
				chain.insert_epoch_transition(&mut batch, 0, EpochTransition {
					block_hash: gh.hash(),
					block_number: 0,
					proof: proof,
				});

				client.db.read().write_buffered(batch);
			}
		}

		if let Some(reg_addr) = client.additional_params().get("registrar").and_then(|s| Address::from_str(s).ok()) {
			trace!(target: "client", "Found registrar at {}", reg_addr);
			let registrar = Registry::new(reg_addr);
			*client.registrar.lock() = Some(registrar);
		}

		// ensure buffered changes are flushed.
		client.db.read().flush().map_err(ClientError::Database)?;
		Ok(client)
	}

	/// Wakes up client if it's a sleep.
	pub fn keep_alive(&self) {
		let should_wake = match *self.mode.lock() {
			Mode::Dark(..) | Mode::Passive(..) => true,
			_ => false,
		};
		if should_wake {
			self.wake_up();
			(*self.sleep_state.lock()).last_activity = Some(Instant::now());
		}
	}

	/// Adds an actor to be notified on certain events
	pub fn add_notify(&self, target: Arc<ChainNotify>) {
		self.notify.write().push(Arc::downgrade(&target));
	}

	/// Set a closure to call when we want to restart the client
	pub fn set_exit_handler<F>(&self, f: F) where F: Fn(bool, Option<String>) + 'static + Send {
		*self.exit_handler.lock() = Some(Box::new(f));
	}

	/// Returns engine reference.
	pub fn engine(&self) -> &EthEngine {
		&*self.engine
	}

	fn notify<F>(&self, f: F) where F: Fn(&ChainNotify) {
		for np in self.notify.read().iter() {
			if let Some(n) = np.upgrade() {
				f(&*n);
			}
		}
	}

	/// Get the Registry object - useful for looking up names.
	pub fn registrar(&self) -> MutexGuard<Option<Registry>> {
		self.registrar.lock()
	}

	/// Register an action to be done if a mode/spec_name change happens.
	pub fn on_user_defaults_change<F>(&self, f: F) where F: 'static + FnMut(Option<Mode>) + Send {
		*self.on_user_defaults_change.lock() = Some(Box::new(f));
	}

	/// Flush the block import queue.
	pub fn flush_queue(&self) {
		self.block_queue.flush();
		while !self.block_queue.queue_info().is_empty() {
			self.import_verified_blocks();
		}
	}

	/// The env info as of the best block.
	pub fn latest_env_info(&self) -> EnvInfo {
		self.env_info(BlockId::Latest).expect("Best block header always stored; qed")
	}

	/// The env info as of a given block.
	/// returns `None` if the block unknown.
	pub fn env_info(&self, id: BlockId) -> Option<EnvInfo> {
		self.block_header(id).map(|header| {
			EnvInfo {
				number: header.number(),
				author: header.author(),
				timestamp: header.timestamp(),
				difficulty: header.difficulty(),
				last_hashes: self.build_last_hashes(header.parent_hash()),
				gas_used: U256::default(),
				gas_limit: header.gas_limit(),
			}
		})
	}

	fn build_last_hashes(&self, parent_hash: H256) -> Arc<LastHashes> {
		{
			let hashes = self.last_hashes.read();
			if hashes.front().map_or(false, |h| h == &parent_hash) {
				let mut res = Vec::from(hashes.clone());
				res.resize(256, H256::default());
				return Arc::new(res);
			}
		}
		let mut last_hashes = LastHashes::new();
		last_hashes.resize(256, H256::default());
		last_hashes[0] = parent_hash;
		let chain = self.chain.read();
		for i in 0..255 {
			match chain.block_details(&last_hashes[i]) {
				Some(details) => {
					last_hashes[i + 1] = details.parent.clone();
				},
				None => break,
			}
		}
		let mut cached_hashes = self.last_hashes.write();
		*cached_hashes = VecDeque::from(last_hashes.clone());
		Arc::new(last_hashes)
	}

	fn check_and_close_block(&self, block: &PreverifiedBlock) -> Result<LockedBlock, ()> {
		let engine = &*self.engine;
		let header = &block.header;

		let chain = self.chain.read();
		// Check the block isn't so old we won't be able to enact it.
		let best_block_number = chain.best_block_number();
		if self.pruning_info().earliest_state > header.number() {
			warn!(target: "client", "Block import failed for #{} ({})\nBlock is ancient (current best block: #{}).", header.number(), header.hash(), best_block_number);
			return Err(());
		}

		// Check if parent is in chain
		let parent = match chain.block_header(header.parent_hash()) {
			Some(h) => h,
			None => {
				warn!(target: "client", "Block import failed for #{} ({}): Parent not found ({}) ", header.number(), header.hash(), header.parent_hash());
				return Err(());
			}
		};

		// Verify Block Family
		let verify_family_result = self.verifier.verify_block_family(
			header,
			&parent,
			engine,
			Some((&block.bytes, &block.transactions, &**chain, self)),
		);

		if let Err(e) = verify_family_result {
			warn!(target: "client", "Stage 3 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
			return Err(());
		};

		let verify_external_result = self.verifier.verify_block_external(header, engine);
		if let Err(e) = verify_external_result {
			warn!(target: "client", "Stage 4 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
			return Err(());
		};

		// Enact Verified Block
		let last_hashes = self.build_last_hashes(header.parent_hash().clone());
		let db = self.state_db.lock().boxed_clone_canon(header.parent_hash());

		let is_epoch_begin = chain.epoch_transition(parent.number(), *header.parent_hash()).is_some();
		let enact_result = enact_verified(block,
			engine,
			self.tracedb.read().tracing_enabled(),
			db,
			&parent,
			last_hashes,
			self.factories.clone(),
			is_epoch_begin,
		);
		let mut locked_block = enact_result.map_err(|e| {
			warn!(target: "client", "Block import failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
		})?;

		if header.number() < self.engine().params().validate_receipts_transition && header.receipts_root() != locked_block.block().header().receipts_root() {
			locked_block = locked_block.strip_receipts();
		}

		// Final Verification
		if let Err(e) = self.verifier.verify_block_final(header, locked_block.block().header()) {
			warn!(target: "client", "Stage 5 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
			return Err(());
		}

		Ok(locked_block)
	}

	fn calculate_enacted_retracted(&self, import_results: &[ImportRoute]) -> (Vec<H256>, Vec<H256>) {
		fn map_to_vec(map: Vec<(H256, bool)>) -> Vec<H256> {
			map.into_iter().map(|(k, _v)| k).collect()
		}

		// In ImportRoute we get all the blocks that have been enacted and retracted by single insert.
		// Because we are doing multiple inserts some of the blocks that were enacted in import `k`
		// could be retracted in import `k+1`. This is why to understand if after all inserts
		// the block is enacted or retracted we iterate over all routes and at the end final state
		// will be in the hashmap
		let map = import_results.iter().fold(HashMap::new(), |mut map, route| {
			for hash in &route.enacted {
				map.insert(hash.clone(), true);
			}
			for hash in &route.retracted {
				map.insert(hash.clone(), false);
			}
			map
		});

		// Split to enacted retracted (using hashmap value)
		let (enacted, retracted) = map.into_iter().partition(|&(_k, v)| v);
		// And convert tuples to keys
		(map_to_vec(enacted), map_to_vec(retracted))
	}

	/// This is triggered by a message coming from a block queue when the block is ready for insertion
	pub fn import_verified_blocks(&self) -> usize {

		// Shortcut out if we know we're incapable of syncing the chain.
		if !self.enabled.load(AtomicOrdering::Relaxed) {
			return 0;
		}

		let max_blocks_to_import = 4;
		let (imported_blocks, import_results, invalid_blocks, imported, proposed_blocks, duration, is_empty) = {
			let mut imported_blocks = Vec::with_capacity(max_blocks_to_import);
			let mut invalid_blocks = HashSet::new();
			let mut proposed_blocks = Vec::with_capacity(max_blocks_to_import);
			let mut import_results = Vec::with_capacity(max_blocks_to_import);

			let _import_lock = self.import_lock.lock();
			let blocks = self.block_queue.drain(max_blocks_to_import);
			if blocks.is_empty() {
				return 0;
			}
			let _timer = PerfTimer::new("import_verified_blocks");
			let start = precise_time_ns();

			for block in blocks {
				let header = &block.header;
				let is_invalid = invalid_blocks.contains(header.parent_hash());
				if is_invalid {
					invalid_blocks.insert(header.hash());
					continue;
				}
				if let Ok(closed_block) = self.check_and_close_block(&block) {
					if self.engine.is_proposal(&block.header) {
						self.block_queue.mark_as_good(&[header.hash()]);
						proposed_blocks.push(block.bytes);
					} else {
						imported_blocks.push(header.hash());

						let route = self.commit_block(closed_block, &header, &block.bytes);
						import_results.push(route);

						self.report.write().accrue_block(&block);
					}
				} else {
					invalid_blocks.insert(header.hash());
				}
			}

			let imported = imported_blocks.len();
			let invalid_blocks = invalid_blocks.into_iter().collect::<Vec<H256>>();

			if !invalid_blocks.is_empty() {
				self.block_queue.mark_as_bad(&invalid_blocks);
			}
			let is_empty = self.block_queue.mark_as_good(&imported_blocks);
			let duration_ns = precise_time_ns() - start;
			(imported_blocks, import_results, invalid_blocks, imported, proposed_blocks, duration_ns, is_empty)
		};

		{
			if !imported_blocks.is_empty() && is_empty {
				let (enacted, retracted) = self.calculate_enacted_retracted(&import_results);

				if is_empty {
					self.miner.chain_new_blocks(self, &imported_blocks, &invalid_blocks, &enacted, &retracted);
				}

				self.notify(|notify| {
					notify.new_blocks(
						imported_blocks.clone(),
						invalid_blocks.clone(),
						enacted.clone(),
						retracted.clone(),
						Vec::new(),
						proposed_blocks.clone(),
						duration,
					);
				});
			}
		}

		self.db.read().flush().expect("DB flush failed.");
		imported
	}

	/// Import a block with transaction receipts.
	/// The block is guaranteed to be the next best blocks in the first block sequence.
	/// Does no sealing or transaction validation.
	fn import_old_block(&self, block_bytes: Bytes, receipts_bytes: Bytes) -> Result<H256, ::error::Error> {
		let block = BlockView::new(&block_bytes);
		let header = block.header();
		let receipts = ::rlp::decode_list(&receipts_bytes);
		let hash = header.hash();
		let _import_lock = self.import_lock.lock();

		{
			let _timer = PerfTimer::new("import_old_block");
			let chain = self.chain.read();
			let mut ancient_verifier = self.ancient_verifier.lock();

			{
				// closure for verifying a block.
				let verify_with = |verifier: &AncientVerifier| -> Result<(), ::error::Error> {
					// verify the block, passing the chain for updating the epoch
					// verifier.
					verifier.verify(&mut *self.rng.lock(), &header, &chain)
				};

				// initialize the ancient block verifier if we don't have one already.
				match &mut *ancient_verifier {
					&mut Some(ref verifier) => {
						verify_with(verifier)?
					}
					x @ &mut None => {
						// load most recent epoch.
						trace!(target: "client", "Initializing ancient block restoration.");
						let current_epoch_data = chain.epoch_transitions()
							.take_while(|&(_, ref t)| t.block_number < header.number())
							.last()
							.map(|(_, t)| t.proof)
							.expect("At least one epoch entry (genesis) always stored; qed");

						let current_verifier = self.engine.epoch_verifier(&header, &current_epoch_data)
							.known_confirmed()?;
						let current_verifier = AncientVerifier::new(self.engine.clone(), current_verifier);

						verify_with(&current_verifier)?;
						*x = Some(current_verifier);
					}
				}
			}

			// Commit results
			let mut batch = DBTransaction::new();
			chain.insert_unordered_block(&mut batch, &block_bytes, receipts, None, false, true);
			// Final commit to the DB
			self.db.read().write_buffered(batch);
			chain.commit();
		}
		self.db.read().flush().expect("DB flush failed.");
		Ok(hash)
	}

	// NOTE: the header of the block passed here is not necessarily sealed, as
	// it is for reconstructing the state transition.
	//
	// The header passed is from the original block data and is sealed.
	fn commit_block<B>(&self, block: B, header: &Header, block_data: &[u8]) -> ImportRoute where B: IsBlock + Drain {
		let hash = &header.hash();
		let number = header.number();
		let parent = header.parent_hash();
		let chain = self.chain.read();

		// Commit results
		let receipts = block.receipts().to_owned();
		let traces = block.traces().clone().unwrap_or_else(Vec::new);
		let traces: Vec<FlatTransactionTraces> = traces.into_iter()
			.map(Into::into)
			.collect();

		assert_eq!(header.hash(), BlockView::new(block_data).header_view().hash());

		//let traces = From::from(block.traces().clone().unwrap_or_else(Vec::new));

		let mut batch = DBTransaction::new();

		// CHECK! I *think* this is fine, even if the state_root is equal to another
		// already-imported block of the same number.
		// TODO: Prove it with a test.
		let mut state = block.drain();

		// check epoch end signal, potentially generating a proof on the current
		// state.
		self.check_epoch_end_signal(
			&header,
			block_data,
			&receipts,
			&state,
			&chain,
			&mut batch,
		);

		state.journal_under(&mut batch, number, hash).expect("DB commit failed");
		let route = chain.insert_block(&mut batch, block_data, receipts.clone());

		self.tracedb.read().import(&mut batch, TraceImportRequest {
			traces: traces.into(),
			block_hash: hash.clone(),
			block_number: number,
			enacted: route.enacted.clone(),
			retracted: route.retracted.len()
		});

		let is_canon = route.enacted.last().map_or(false, |h| h == hash);
		state.sync_cache(&route.enacted, &route.retracted, is_canon);
		// Final commit to the DB
		self.db.read().write_buffered(batch);
		chain.commit();

		self.check_epoch_end(&header, &chain);

		self.update_last_hashes(&parent, hash);

		if let Err(e) = self.prune_ancient(state, &chain) {
			warn!("Failed to prune ancient state data: {}", e);
		}

		route
	}

	// check for epoch end signal and write pending transition if it occurs.
	// state for the given block must be available.
	fn check_epoch_end_signal(
		&self,
		header: &Header,
		block_bytes: &[u8],
		receipts: &[Receipt],
		state_db: &StateDB,
		chain: &BlockChain,
		batch: &mut DBTransaction,
	) {
		use engines::EpochChange;

		let hash = header.hash();
		let auxiliary = ::machine::AuxiliaryData {
			bytes: Some(block_bytes),
			receipts: Some(&receipts),
		};

		match self.engine.signals_epoch_end(header, auxiliary) {
			EpochChange::Yes(proof) => {
				use engines::epoch::PendingTransition;
				use engines::Proof;

				let proof = match proof {
					Proof::Known(proof) => proof,
					Proof::WithState(with_state) => {
						let env_info = EnvInfo {
							number: header.number(),
							author: header.author().clone(),
							timestamp: header.timestamp(),
							difficulty: header.difficulty().clone(),
							last_hashes: self.build_last_hashes(header.parent_hash().clone()),
							gas_used: U256::default(),
							gas_limit: u64::max_value().into(),
						};

						let call = move |addr, data| {
							let mut state_db = state_db.boxed_clone();
							let backend = ::state::backend::Proving::new(state_db.as_hashdb_mut());

							let transaction =
								self.contract_call_tx(BlockId::Hash(*header.parent_hash()), addr, data);

							let mut state = State::from_existing(
								backend,
								header.state_root().clone(),
								self.engine.account_start_nonce(header.number()),
								self.factories.clone(),
							).expect("state known to be available for just-imported block; qed");

							let options = TransactOptions::with_no_tracing().dont_check_nonce();
							let res = Executive::new(&mut state, &env_info, self.engine.machine())
								.transact(&transaction, options);

							let res = match res {
								Err(ExecutionError::Internal(e)) =>
									Err(format!("Internal error: {}", e)),
								Err(e) => {
									trace!(target: "client", "Proved call failed: {}", e);
									Ok((Vec::new(), state.drop().1.extract_proof()))
								}
								Ok(res) => Ok((res.output, state.drop().1.extract_proof())),
							};

							res.map(|(output, proof)| (output, proof.into_iter().map(|x| x.into_vec()).collect()))
						};

						match with_state.generate_proof(&call) {
							Ok(proof) => proof,
							Err(e) => {
								warn!(target: "client", "Failed to generate transition proof for block {}: {}", hash, e);
								warn!(target: "client", "Snapshots produced by this client may be incomplete");
								Vec::new()
							}
						}
					}
				};

				debug!(target: "client", "Block {} signals epoch end.", hash);

				let pending = PendingTransition { proof: proof };
				chain.insert_pending_transition(batch, hash, pending);
			},
			EpochChange::No => {},
			EpochChange::Unsure(_) => {
				warn!(target: "client", "Detected invalid engine implementation.");
				warn!(target: "client", "Engine claims to require more block data, but everything provided.");
			}
		}
	}

	// check for ending of epoch and write transition if it occurs.
	fn check_epoch_end<'a>(&self, header: &'a Header, chain: &BlockChain) {
		let is_epoch_end = self.engine.is_epoch_end(
			header,
			&(|hash| chain.block_header(&hash)),
			&(|hash| chain.get_pending_transition(hash)), // TODO: limit to current epoch.
		);

		if let Some(proof) = is_epoch_end {
			debug!(target: "client", "Epoch transition at block {}", header.hash());

			let mut batch = DBTransaction::new();
			chain.insert_epoch_transition(&mut batch, header.number(), EpochTransition {
				block_hash: header.hash(),
				block_number: header.number(),
				proof: proof,
			});

			// always write the batch directly since epoch transition proofs are
			// fetched from a DB iterator and DB iterators are only available on
			// flushed data.
			self.db.read().write(batch).expect("DB flush failed");
		}
	}

	// use a state-proving closure for the given block.
	fn with_proving_caller<F, T>(&self, id: BlockId, with_call: F) -> T
		where F: FnOnce(&::machine::Call) -> T
	{
		let call = |a, d| {
			let tx = self.contract_call_tx(id, a, d);
			let (result, items) = self.prove_transaction(tx, id)
				.ok_or_else(|| format!("Unable to make call. State unavailable?"))?;

			let items = items.into_iter().map(|x| x.to_vec()).collect();
			Ok((result, items))
		};

		with_call(&call)
	}

	// prune ancient states until below the memory limit or only the minimum amount remain.
	fn prune_ancient(&self, mut state_db: StateDB, chain: &BlockChain) -> Result<(), ClientError> {
		let number = match state_db.journal_db().latest_era() {
			Some(n) => n,
			None => return Ok(()),
		};

		// prune all ancient eras until we're below the memory target,
		// but have at least the minimum number of states.
		loop {
			let needs_pruning = state_db.journal_db().is_pruned() &&
				state_db.journal_db().journal_size() >= self.config.history_mem;

			if !needs_pruning { break }
			match state_db.journal_db().earliest_era() {
				Some(era) if era + self.history <= number => {
					trace!(target: "client", "Pruning state for ancient era {}", era);
					match chain.block_hash(era) {
						Some(ancient_hash) => {
							let mut batch = DBTransaction::new();
							state_db.mark_canonical(&mut batch, era, &ancient_hash)?;
							self.db.read().write_buffered(batch);
							state_db.journal_db().flush();
						}
						None =>
							debug!(target: "client", "Missing expected hash for block {}", era),
					}
				}
				_ => break, // means that every era is kept, no pruning necessary.
			}
		}

		Ok(())
	}

	fn update_last_hashes(&self, parent: &H256, hash: &H256) {
		let mut hashes = self.last_hashes.write();
		if hashes.front().map_or(false, |h| h == parent) {
			if hashes.len() > 255 {
				hashes.pop_back();
			}
			hashes.push_front(hash.clone());
		}
	}

	/// Import transactions from the IO queue
	pub fn import_queued_transactions(&self, transactions: &[Bytes], peer_id: usize) -> usize {
		trace!(target: "external_tx", "Importing queued");
		let _timer = PerfTimer::new("import_queued_transactions");
		self.queue_transactions.fetch_sub(transactions.len(), AtomicOrdering::SeqCst);
		let txs: Vec<UnverifiedTransaction> = transactions.iter().filter_map(|bytes| UntrustedRlp::new(bytes).as_val().ok()).collect();
		let hashes: Vec<_> = txs.iter().map(|tx| tx.hash()).collect();
		self.notify(|notify| {
			notify.transactions_received(hashes.clone(), peer_id);
		});
		let results = self.miner.import_external_transactions(self, txs);
		results.len()
	}

	/// Get shared miner reference.
	pub fn miner(&self) -> Arc<Miner> {
		self.miner.clone()
	}

	/// Replace io channel. Useful for testing.
	pub fn set_io_channel(&self, io_channel: IoChannel<ClientIoMessage>) {
		*self.io_channel.lock() = io_channel;
	}

	/// Attempt to get a copy of a specific block's final state.
	///
	/// This will not fail if given BlockId::Latest.
	/// Otherwise, this can fail (but may not) if the DB prunes state or the block
	/// is unknown.
	pub fn state_at(&self, id: BlockId) -> Option<State<StateDB>> {
		// fast path for latest state.
		match id.clone() {
			BlockId::Pending => return self.miner.pending_state(self.chain.read().best_block_number()).or_else(|| Some(self.state())),
			BlockId::Latest => return Some(self.state()),
			_ => {},
		}

		let block_number = match self.block_number(id) {
			Some(num) => num,
			None => return None,
		};

		self.block_header(id).and_then(|header| {
			let db = self.state_db.lock().boxed_clone();

			// early exit for pruned blocks
			if db.is_pruned() && self.pruning_info().earliest_state > block_number {
				return None;
			}

			let root = header.state_root();
			State::from_existing(db, root, self.engine.account_start_nonce(block_number), self.factories.clone()).ok()
		})
	}

	/// Attempt to get a copy of a specific block's beginning state.
	///
	/// This will not fail if given BlockId::Latest.
	/// Otherwise, this can fail (but may not) if the DB prunes state.
	pub fn state_at_beginning(&self, id: BlockId) -> Option<State<StateDB>> {
		// fast path for latest state.
		match id {
			BlockId::Pending => self.state_at(BlockId::Latest),
			id => match self.block_number(id) {
				None | Some(0) => None,
				Some(n) => self.state_at(BlockId::Number(n - 1)),
			}
		}
	}

	/// Get a copy of the best block's state.
	pub fn state(&self) -> State<StateDB> {
		let header = self.best_block_header();
		State::from_existing(
			self.state_db.lock().boxed_clone_canon(&header.hash()),
			header.state_root(),
			self.engine.account_start_nonce(header.number()),
			self.factories.clone())
		.expect("State root of best block header always valid.")
	}

	/// Get info on the cache.
	pub fn blockchain_cache_info(&self) -> BlockChainCacheSize {
		self.chain.read().cache_size()
	}

	/// Get the report.
	pub fn report(&self) -> ClientReport {
		let mut report = self.report.read().clone();
		report.state_db_mem = self.state_db.lock().mem_used();
		report
	}

	/// Tick the client.
	// TODO: manage by real events.
	pub fn tick(&self, prevent_sleep: bool) {
		self.check_garbage();
		if !prevent_sleep {
			self.check_snooze();
		}
	}

	fn check_garbage(&self) {
		self.chain.read().collect_garbage();
		self.block_queue.collect_garbage();
		self.tracedb.read().collect_garbage();
	}

	fn check_snooze(&self) {
		let mode = self.mode.lock().clone();
		match mode {
			Mode::Dark(timeout) => {
				let mut ss = self.sleep_state.lock();
				if let Some(t) = ss.last_activity {
					if Instant::now() > t + timeout {
						self.sleep();
						ss.last_activity = None;
					}
				}
			}
			Mode::Passive(timeout, wakeup_after) => {
				let mut ss = self.sleep_state.lock();
				let now = Instant::now();
				if let Some(t) = ss.last_activity {
					if now > t + timeout {
						self.sleep();
						ss.last_activity = None;
						ss.last_autosleep = Some(now);
					}
				}
				if let Some(t) = ss.last_autosleep {
					if now > t + wakeup_after {
						self.wake_up();
						ss.last_activity = Some(now);
						ss.last_autosleep = None;
					}
				}
			}
			_ => {}
		}
	}

	/// Take a snapshot at the given block.
	/// If the ID given is "latest", this will default to 1000 blocks behind.
	pub fn take_snapshot<W: snapshot_io::SnapshotWriter + Send>(&self, writer: W, at: BlockId, p: &snapshot::Progress) -> Result<(), EthcoreError> {
		let db = self.state_db.lock().journal_db().boxed_clone();
		let best_block_number = self.chain_info().best_block_number;
		let block_number = self.block_number(at).ok_or(snapshot::Error::InvalidStartingBlock(at))?;

		if db.is_pruned() && self.pruning_info().earliest_state > block_number {
			return Err(snapshot::Error::OldBlockPrunedDB.into());
		}

		let history = ::std::cmp::min(self.history, 1000);

		let start_hash = match at {
			BlockId::Latest => {
				let start_num = match db.earliest_era() {
					Some(era) => ::std::cmp::max(era, best_block_number.saturating_sub(history)),
					None => best_block_number.saturating_sub(history),
				};

				match self.block_hash(BlockId::Number(start_num)) {
					Some(h) => h,
					None => return Err(snapshot::Error::InvalidStartingBlock(at).into()),
				}
			}
			_ => match self.block_hash(at) {
				Some(hash) => hash,
				None => return Err(snapshot::Error::InvalidStartingBlock(at).into()),
			},
		};

		snapshot::take_snapshot(&*self.engine, &self.chain.read(), start_hash, db.as_hashdb(), writer, p)?;

		Ok(())
	}

	/// Ask the client what the history parameter is.
	pub fn pruning_history(&self) -> u64 {
		self.history
	}

	fn block_hash(chain: &BlockChain, miner: &Miner, id: BlockId) -> Option<H256> {
		match id {
			BlockId::Hash(hash) => Some(hash),
			BlockId::Number(number) => chain.block_hash(number),
			BlockId::Earliest => chain.block_hash(0),
			BlockId::Latest => Some(chain.best_block_hash()),
			BlockId::Pending => miner.pending_block_header(chain.best_block_number()).map(|header| header.hash())
		}
	}

	fn transaction_address(&self, id: TransactionId) -> Option<TransactionAddress> {
		match id {
			TransactionId::Hash(ref hash) => self.chain.read().transaction_address(hash),
			TransactionId::Location(id, index) => Self::block_hash(&self.chain.read(), &self.miner, id).map(|hash| TransactionAddress {
				block_hash: hash,
				index: index,
			})
		}
	}

	fn wake_up(&self) {
		if !self.liveness.load(AtomicOrdering::Relaxed) {
			self.liveness.store(true, AtomicOrdering::Relaxed);
			self.notify(|n| n.start());
			info!(target: "mode", "wake_up: Waking.");
		}
	}

	fn sleep(&self) {
		if self.liveness.load(AtomicOrdering::Relaxed) {
			// only sleep if the import queue is mostly empty.
			if self.queue_info().total_queue_size() <= MAX_QUEUE_SIZE_TO_SLEEP_ON {
				self.liveness.store(false, AtomicOrdering::Relaxed);
				self.notify(|n| n.stop());
				info!(target: "mode", "sleep: Sleeping.");
			} else {
				info!(target: "mode", "sleep: Cannot sleep - syncing ongoing.");
				// TODO: Consider uncommenting.
				//(*self.sleep_state.lock()).last_activity = Some(Instant::now());
			}
		}
	}

	// transaction for calling contracts from services like engine.
	// from the null sender, with 50M gas.
	fn contract_call_tx(&self, block_id: BlockId, address: Address, data: Bytes) -> SignedTransaction {
		let from = Address::default();
		Transaction {
			nonce: self.nonce(&from, block_id).unwrap_or_else(|| self.engine.account_start_nonce(0)),
			action: Action::Call(address),
			gas: U256::from(50_000_000),
			gas_price: U256::default(),
			value: U256::default(),
			data: data,
		}.fake_sign(from)
	}

	fn do_virtual_call(&self, env_info: &EnvInfo, state: &mut State<StateDB>, t: &SignedTransaction, analytics: CallAnalytics) -> Result<Executed, CallError> {
		fn call<V, T>(
			state: &mut State<StateDB>,
			env_info: &EnvInfo,
			machine: &::machine::EthereumMachine,
			state_diff: bool,
			transaction: &SignedTransaction,
			options: TransactOptions<T, V>,
		) -> Result<Executed<T::Output, V::Output>, CallError> where
			T: trace::Tracer,
			V: trace::VMTracer,
		{
			let options = options
				.dont_check_nonce()
				.save_output_from_contract();
			let original_state = if state_diff { Some(state.clone()) } else { None };

			let mut ret = Executive::new(state, env_info, machine).transact_virtual(transaction, options)?;

			if let Some(original) = original_state {
				ret.state_diff = Some(state.diff_from(original).map_err(ExecutionError::from)?);
			}
			Ok(ret)
		}

		let state_diff = analytics.state_diffing;
		let machine = self.engine.machine();

		match (analytics.transaction_tracing, analytics.vm_tracing) {
			(true, true) => call(state, env_info, machine, state_diff, t, TransactOptions::with_tracing_and_vm_tracing()),
			(true, false) => call(state, env_info, machine, state_diff, t, TransactOptions::with_tracing()),
			(false, true) => call(state, env_info, machine, state_diff, t, TransactOptions::with_vm_tracing()),
			(false, false) => call(state, env_info, machine, state_diff, t, TransactOptions::with_no_tracing()),
		}
	}

	fn block_number_ref(&self, id: &BlockId) -> Option<BlockNumber> {
		match *id {
			BlockId::Number(number) => Some(number),
			BlockId::Hash(ref hash) => self.chain.read().block_number(hash),
			BlockId::Earliest => Some(0),
			BlockId::Latest => Some(self.chain.read().best_block_number()),
			BlockId::Pending => Some(self.chain.read().best_block_number() + 1),
		}
	}
}

impl snapshot::DatabaseRestore for Client {
	/// Restart the client with a new backend
	fn restore_db(&self, new_db: &str) -> Result<(), EthcoreError> {
		trace!(target: "snapshot", "Replacing client database with {:?}", new_db);

		let _import_lock = self.import_lock.lock();
		let mut state_db = self.state_db.lock();
		let mut chain = self.chain.write();
		let mut tracedb = self.tracedb.write();
		self.miner.clear();
		let db = self.db.write();
		db.restore(new_db)?;

		let cache_size = state_db.cache_size();
		*state_db = StateDB::new(journaldb::new(db.clone(), self.pruning, ::db::COL_STATE), cache_size);
		*chain = Arc::new(BlockChain::new(self.config.blockchain.clone(), &[], db.clone()));
		*tracedb = TraceDB::new(self.config.tracing.clone(), db.clone(), chain.clone());
		Ok(())
	}
}

impl BlockChainClient for Client {
	fn call(&self, transaction: &SignedTransaction, analytics: CallAnalytics, block: BlockId) -> Result<Executed, CallError> {
		let mut env_info = self.env_info(block).ok_or(CallError::StatePruned)?;
		env_info.gas_limit = U256::max_value();

		// that's just a copy of the state.
		let mut state = self.state_at(block).ok_or(CallError::StatePruned)?;

		self.do_virtual_call(&env_info, &mut state, transaction, analytics)
	}

	fn call_many(&self, transactions: &[(SignedTransaction, CallAnalytics)], block: BlockId) -> Result<Vec<Executed>, CallError> {
		let mut env_info = self.env_info(block).ok_or(CallError::StatePruned)?;
		env_info.gas_limit = U256::max_value();

		// that's just a copy of the state.
		let mut state = self.state_at(block).ok_or(CallError::StatePruned)?;
		let mut results = Vec::with_capacity(transactions.len());

		for &(ref t, analytics) in transactions {
			let ret = self.do_virtual_call(&env_info, &mut state, t, analytics)?;
			env_info.gas_used = ret.cumulative_gas_used;
			results.push(ret);
		}

		Ok(results)
	}

	fn estimate_gas(&self, t: &SignedTransaction, block: BlockId) -> Result<U256, CallError> {
		let (mut upper, max_upper, env_info)  = {
			let mut env_info = self.env_info(block).ok_or(CallError::StatePruned)?;
			let init = env_info.gas_limit;
			let max = init * U256::from(10);
			env_info.gas_limit = max;
			(init, max, env_info)
		};

		// that's just a copy of the state.
		let original_state = self.state_at(block).ok_or(CallError::StatePruned)?;
		let sender = t.sender();
		let options = || TransactOptions::with_tracing().dont_check_nonce();

		let cond = |gas| {
			let mut tx = t.as_unsigned().clone();
			tx.gas = gas;
			let tx = tx.fake_sign(sender);

			let mut state = original_state.clone();
			Ok(Executive::new(&mut state, &env_info, self.engine.machine())
				.transact_virtual(&tx, options())
				.map(|r| r.exception.is_none())
				.unwrap_or(false))
		};

		if !cond(upper)? {
			upper = max_upper;
			if !cond(upper)? {
				trace!(target: "estimate_gas", "estimate_gas failed with {}", upper);
				let err = ExecutionError::Internal(format!("Requires higher than upper limit of {}", upper));
				return Err(err.into())
			}
		}
		let lower = t.gas_required(&self.engine.schedule(env_info.number)).into();
		if cond(lower)? {
			trace!(target: "estimate_gas", "estimate_gas succeeded with {}", lower);
			return Ok(lower)
		}

		/// Find transition point between `lower` and `upper` where `cond` changes from `false` to `true`.
		/// Returns the lowest value between `lower` and `upper` for which `cond` returns true.
		/// We assert: `cond(lower) = false`, `cond(upper) = true`
		fn binary_chop<F, E>(mut lower: U256, mut upper: U256, mut cond: F) -> Result<U256, E>
			where F: FnMut(U256) -> Result<bool, E>
		{
			while upper - lower > 1.into() {
				let mid = (lower + upper) / 2.into();
				trace!(target: "estimate_gas", "{} .. {} .. {}", lower, mid, upper);
				let c = cond(mid)?;
				match c {
					true => upper = mid,
					false => lower = mid,
				};
				trace!(target: "estimate_gas", "{} => {} .. {}", c, lower, upper);
			}
			Ok(upper)
		}

		// binary chop to non-excepting call with gas somewhere between 21000 and block gas limit
		trace!(target: "estimate_gas", "estimate_gas chopping {} .. {}", lower, upper);
		binary_chop(lower, upper, cond)
	}

	fn replay(&self, id: TransactionId, analytics: CallAnalytics) -> Result<Executed, CallError> {
		let address = self.transaction_address(id).ok_or(CallError::TransactionNotFound)?;
		let mut env_info = self.env_info(BlockId::Hash(address.block_hash)).ok_or(CallError::StatePruned)?;
		let body = self.block_body(BlockId::Hash(address.block_hash)).ok_or(CallError::StatePruned)?;
		let mut state = self.state_at_beginning(BlockId::Hash(address.block_hash)).ok_or(CallError::StatePruned)?;
		let mut txs = body.transactions();

		if address.index >= txs.len() {
			return Err(CallError::TransactionNotFound);
		}

		const PROOF: &'static str = "Transactions fetched from blockchain; blockchain transactions are valid; qed";
		let rest = txs.split_off(address.index);
		for t in txs {
			let t = SignedTransaction::new(t).expect(PROOF);
			let x = Executive::new(&mut state, &env_info, self.engine.machine()).transact(&t, TransactOptions::with_no_tracing())?;
			env_info.gas_used = env_info.gas_used + x.gas_used;
		}
		let first = rest.into_iter().next().expect("We split off < `address.index`; Length is checked earlier; qed");
		let t = SignedTransaction::new(first).expect(PROOF);

		self.do_virtual_call(&env_info, &mut state, &t, analytics)
	}

	fn mode(&self) -> IpcMode {
		let r = self.mode.lock().clone().into();
		trace!(target: "mode", "Asked for mode = {:?}. returning {:?}", &*self.mode.lock(), r);
		r
	}

	fn disable(&self) {
		self.set_mode(IpcMode::Off);
		self.enabled.store(false, AtomicOrdering::Relaxed);
		self.clear_queue();
	}

	fn set_mode(&self, new_mode: IpcMode) {
		trace!(target: "mode", "Client::set_mode({:?})", new_mode);
		if !self.enabled.load(AtomicOrdering::Relaxed) {
			return;
		}
		{
			let mut mode = self.mode.lock();
			*mode = new_mode.clone().into();
			trace!(target: "mode", "Mode now {:?}", &*mode);
			if let Some(ref mut f) = *self.on_user_defaults_change.lock() {
				trace!(target: "mode", "Making callback...");
				f(Some((&*mode).clone()))
			}
		}
		match new_mode {
			IpcMode::Active => self.wake_up(),
			IpcMode::Off => self.sleep(),
			_ => {(*self.sleep_state.lock()).last_activity = Some(Instant::now()); }
		}
	}

	fn spec_name(&self) -> String {
		self.config.spec_name.clone()
	}

	fn set_spec_name(&self, new_spec_name: String) {
		trace!(target: "mode", "Client::set_spec_name({:?})", new_spec_name);
		if !self.enabled.load(AtomicOrdering::Relaxed) {
			return;
		}
		if let Some(ref h) = *self.exit_handler.lock() {
			(*h)(true, Some(new_spec_name));
		} else {
			warn!("Not hypervised; cannot change chain.");
		}
	}

	fn best_block_header(&self) -> encoded::Header {
		self.chain.read().best_block_header()
	}

	fn block_header(&self, id: BlockId) -> Option<::encoded::Header> {
		let chain = self.chain.read();

		if let BlockId::Pending = id {
			if let Some(block) = self.miner.pending_block(chain.best_block_number()) {
				return Some(encoded::Header::new(block.header.rlp(Seal::Without)));
			}
			// fall back to latest
			return self.block_header(BlockId::Latest);
		}

		Self::block_hash(&chain, &self.miner, id).and_then(|hash| chain.block_header_data(&hash))
	}

	fn block_number(&self, id: BlockId) -> Option<BlockNumber> {
		self.block_number_ref(&id)
	}

	fn block_body(&self, id: BlockId) -> Option<encoded::Body> {
		let chain = self.chain.read();

		if let BlockId::Pending = id {
			if let Some(block) = self.miner.pending_block(chain.best_block_number()) {
				return Some(encoded::Body::new(BlockChain::block_to_body(&block.rlp_bytes(Seal::Without))));
			}
			// fall back to latest
			return self.block_body(BlockId::Latest);
		}

		Self::block_hash(&chain, &self.miner, id).and_then(|hash| chain.block_body(&hash))
	}

	fn block(&self, id: BlockId) -> Option<encoded::Block> {
		let chain = self.chain.read();

		if let BlockId::Pending = id {
			if let Some(block) = self.miner.pending_block(chain.best_block_number()) {
				return Some(encoded::Block::new(block.rlp_bytes(Seal::Without)));
			}
			// fall back to latest
			return self.block(BlockId::Latest);
		}

		Self::block_hash(&chain, &self.miner, id).and_then(|hash| {
			chain.block(&hash)
		})
	}

	fn block_status(&self, id: BlockId) -> BlockStatus {
		if let BlockId::Pending = id {
			return BlockStatus::Pending;
		}

		let chain = self.chain.read();
		match Self::block_hash(&chain, &self.miner, id) {
			Some(ref hash) if chain.is_known(hash) => BlockStatus::InChain,
			Some(hash) => self.block_queue.status(&hash).into(),
			None => BlockStatus::Unknown
		}
	}

	fn block_total_difficulty(&self, id: BlockId) -> Option<U256> {
		let chain = self.chain.read();
		if let BlockId::Pending = id {
			let latest_difficulty = self.block_total_difficulty(BlockId::Latest).expect("blocks in chain have details; qed");
			let pending_difficulty = self.miner.pending_block_header(chain.best_block_number()).map(|header| *header.difficulty());
			if let Some(difficulty) = pending_difficulty {
				return Some(difficulty + latest_difficulty);
			}
			// fall back to latest
			return Some(latest_difficulty);
		}

		Self::block_hash(&chain, &self.miner, id).and_then(|hash| chain.block_details(&hash)).map(|d| d.total_difficulty)
	}

	fn nonce(&self, address: &Address, id: BlockId) -> Option<U256> {
		self.state_at(id).and_then(|s| s.nonce(address).ok())
	}

	fn storage_root(&self, address: &Address, id: BlockId) -> Option<H256> {
		self.state_at(id).and_then(|s| s.storage_root(address).ok()).and_then(|x| x)
	}

	fn block_hash(&self, id: BlockId) -> Option<H256> {
		let chain = self.chain.read();
		Self::block_hash(&chain, &self.miner, id)
	}

	fn code(&self, address: &Address, id: BlockId) -> Option<Option<Bytes>> {
		self.state_at(id).and_then(|s| s.code(address).ok()).map(|c| c.map(|c| (&*c).clone()))
	}

	fn code_hash(&self, address: &Address, id: BlockId) -> Option<H256> {
		self.state_at(id).and_then(|s| s.code_hash(address).ok())
	}

	fn balance(&self, address: &Address, id: BlockId) -> Option<U256> {
		self.state_at(id).and_then(|s| s.balance(address).ok())
	}

	fn storage_at(&self, address: &Address, position: &H256, id: BlockId) -> Option<H256> {
		self.state_at(id).and_then(|s| s.storage_at(address, position).ok())
	}

	fn list_accounts(&self, id: BlockId, after: Option<&Address>, count: u64) -> Option<Vec<Address>> {
		if !self.factories.trie.is_fat() {
			trace!(target: "fatdb", "list_accounts: Not a fat DB");
			return None;
		}

		let state = match self.state_at(id) {
			Some(state) => state,
			_ => return None,
		};

		let (root, db) = state.drop();
		let trie = match self.factories.trie.readonly(db.as_hashdb(), &root) {
			Ok(trie) => trie,
			_ => {
				trace!(target: "fatdb", "list_accounts: Couldn't open the DB");
				return None;
			}
		};

		let mut iter = match trie.iter() {
			Ok(iter) => iter,
			_ => return None,
		};

		if let Some(after) = after {
			if let Err(e) = iter.seek(after) {
				trace!(target: "fatdb", "list_accounts: Couldn't seek the DB: {:?}", e);
			}
		}

		let accounts = iter.filter_map(|item| {
			item.ok().map(|(addr, _)| Address::from_slice(&addr))
		}).take(count as usize).collect();

		Some(accounts)
	}

	fn list_storage(&self, id: BlockId, account: &Address, after: Option<&H256>, count: u64) -> Option<Vec<H256>> {
		if !self.factories.trie.is_fat() {
			trace!(target: "fatdb", "list_stroage: Not a fat DB");
			return None;
		}

		let state = match self.state_at(id) {
			Some(state) => state,
			_ => return None,
		};

		let root = match state.storage_root(account) {
			Ok(Some(root)) => root,
			_ => return None,
		};

		let (_, db) = state.drop();
		let account_db = self.factories.accountdb.readonly(db.as_hashdb(), keccak(account));
		let trie = match self.factories.trie.readonly(account_db.as_hashdb(), &root) {
			Ok(trie) => trie,
			_ => {
				trace!(target: "fatdb", "list_storage: Couldn't open the DB");
				return None;
			}
		};

		let mut iter = match trie.iter() {
			Ok(iter) => iter,
			_ => return None,
		};

		if let Some(after) = after {
			if let Err(e) = iter.seek(after) {
				trace!(target: "fatdb", "list_accounts: Couldn't seek the DB: {:?}", e);
			}
		}

		let keys = iter.filter_map(|item| {
			item.ok().map(|(key, _)| H256::from_slice(&key))
		}).take(count as usize).collect();

		Some(keys)
	}

	fn transaction(&self, id: TransactionId) -> Option<LocalizedTransaction> {
		self.transaction_address(id).and_then(|address| self.chain.read().transaction(&address))
	}

	fn transaction_block(&self, id: TransactionId) -> Option<H256> {
		self.transaction_address(id).map(|addr| addr.block_hash)
	}

	fn uncle(&self, id: UncleId) -> Option<encoded::Header> {
		let index = id.position;
		self.block_body(id.block).and_then(|body| body.view().uncle_rlp_at(index))
			.map(encoded::Header::new)
	}

	fn transaction_receipt(&self, id: TransactionId) -> Option<LocalizedReceipt> {
		let chain = self.chain.read();
		self.transaction_address(id)
			.and_then(|address| chain.block_number(&address.block_hash).and_then(|block_number| {
				let transaction = chain.block_body(&address.block_hash)
					.and_then(|body| body.view().localized_transaction_at(&address.block_hash, block_number, address.index));

				let previous_receipts = (0..address.index + 1)
					.map(|index| {
						let mut address = address.clone();
						address.index = index;
						chain.transaction_receipt(&address)
					})
					.collect();
				match (transaction, previous_receipts) {
					(Some(transaction), Some(previous_receipts)) => {
						Some(transaction_receipt(self.engine().machine(), transaction, previous_receipts))
					},
					_ => None,
				}
			}))
	}

	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute> {
		let chain = self.chain.read();
		match chain.is_known(from) && chain.is_known(to) {
			true => chain.tree_route(from.clone(), to.clone()),
			false => None
		}
	}

	fn find_uncles(&self, hash: &H256) -> Option<Vec<H256>> {
		self.chain.read().find_uncle_hashes(hash, self.engine.maximum_uncle_age())
	}

	fn state_data(&self, hash: &H256) -> Option<Bytes> {
		self.state_db.lock().journal_db().state(hash)
	}

	fn block_receipts(&self, hash: &H256) -> Option<Bytes> {
		self.chain.read().block_receipts(hash).map(|receipts| ::rlp::encode(&receipts).into_vec())
	}

	fn import_block(&self, bytes: Bytes) -> Result<H256, BlockImportError> {
		use verification::queue::kind::BlockLike;
		use verification::queue::kind::blocks::Unverified;

		// create unverified block here so the `keccak` calculation can be cached.
		let unverified = Unverified::new(bytes);

		{
			if self.chain.read().is_known(&unverified.hash()) {
				return Err(BlockImportError::Import(ImportError::AlreadyInChain));
			}
			let status = self.block_status(BlockId::Hash(unverified.parent_hash()));
			if status == BlockStatus::Unknown || status == BlockStatus::Pending {
				return Err(BlockImportError::Block(BlockError::UnknownParent(unverified.parent_hash())));
			}
		}
		Ok(self.block_queue.import(unverified)?)
	}

	fn import_block_with_receipts(&self, block_bytes: Bytes, receipts_bytes: Bytes) -> Result<H256, BlockImportError> {
		{
			// check block order
			let header = BlockView::new(&block_bytes).header_view();
			if self.chain.read().is_known(&header.hash()) {
				return Err(BlockImportError::Import(ImportError::AlreadyInChain));
			}
			let status = self.block_status(BlockId::Hash(header.parent_hash()));
			if  status == BlockStatus::Unknown || status == BlockStatus::Pending {
				return Err(BlockImportError::Block(BlockError::UnknownParent(header.parent_hash())));
			}
		}
		self.import_old_block(block_bytes, receipts_bytes).map_err(Into::into)
	}

	fn queue_info(&self) -> BlockQueueInfo {
		self.block_queue.queue_info()
	}

	fn clear_queue(&self) {
		self.block_queue.clear();
	}

	fn chain_info(&self) -> BlockChainInfo {
		let mut chain_info = self.chain.read().chain_info();
		chain_info.pending_total_difficulty = chain_info.total_difficulty + self.block_queue.total_difficulty();
		chain_info
	}

	fn additional_params(&self) -> BTreeMap<String, String> {
		self.engine.additional_params().into_iter().collect()
	}

	fn logs(&self, filter: Filter) -> Vec<LocalizedLogEntry> {
		let (from, to) = match (self.block_number_ref(&filter.from_block), self.block_number_ref(&filter.to_block)) {
			(Some(from), Some(to)) => (from, to),
			_ => return Vec::new(),
		};

		let chain = self.chain.read();
		let blocks = filter.bloom_possibilities().iter()
			.map(move |bloom| {
				chain.blocks_with_bloom(bloom, from, to)
			})
			.flat_map(|m| m)
			// remove duplicate elements
			.collect::<HashSet<u64>>()
			.into_iter()
			.collect::<Vec<u64>>();

		self.chain.read().logs(blocks, |entry| filter.matches(entry), filter.limit)
	}

	fn filter_traces(&self, filter: TraceFilter) -> Option<Vec<LocalizedTrace>> {
		let start = self.block_number(filter.range.start);
		let end = self.block_number(filter.range.end);

		match (start, end) {
			(Some(s), Some(e)) => {
				let db_filter = trace::Filter {
					range: s as usize..e as usize,
					from_address: From::from(filter.from_address),
					to_address: From::from(filter.to_address),
				};

				let traces = self.tracedb.read().filter(&db_filter);
				if traces.is_empty() {
					return Some(vec![]);
				}

				let traces_iter = traces.into_iter().skip(filter.after.unwrap_or(0));
				Some(match filter.count {
					Some(count) => traces_iter.take(count).collect(),
					None => traces_iter.collect(),
				})
			},
			_ => None,
		}
	}

	fn trace(&self, trace: TraceId) -> Option<LocalizedTrace> {
		let trace_address = trace.address;
		self.transaction_address(trace.transaction)
			.and_then(|tx_address| {
				self.block_number(BlockId::Hash(tx_address.block_hash))
					.and_then(|number| self.tracedb.read().trace(number, tx_address.index, trace_address))
			})
	}

	fn transaction_traces(&self, transaction: TransactionId) -> Option<Vec<LocalizedTrace>> {
		self.transaction_address(transaction)
			.and_then(|tx_address| {
				self.block_number(BlockId::Hash(tx_address.block_hash))
					.and_then(|number| self.tracedb.read().transaction_traces(number, tx_address.index))
			})
	}

	fn block_traces(&self, block: BlockId) -> Option<Vec<LocalizedTrace>> {
		self.block_number(block)
			.and_then(|number| self.tracedb.read().block_traces(number))
	}

	fn last_hashes(&self) -> LastHashes {
		(*self.build_last_hashes(self.chain.read().best_block_hash())).clone()
	}

	fn queue_transactions(&self, transactions: Vec<Bytes>, peer_id: usize) {
		let queue_size = self.queue_transactions.load(AtomicOrdering::Relaxed);
		trace!(target: "external_tx", "Queue size: {}", queue_size);
		if queue_size > MAX_TX_QUEUE_SIZE {
			debug!("Ignoring {} transactions: queue is full", transactions.len());
		} else {
			let len = transactions.len();
			match self.io_channel.lock().send(ClientIoMessage::NewTransactions(transactions, peer_id)) {
				Ok(_) => {
					self.queue_transactions.fetch_add(len, AtomicOrdering::SeqCst);
				}
				Err(e) => {
					debug!("Ignoring {} transactions: error queueing: {}", len, e);
				}
			}
		}
	}

	fn ready_transactions(&self) -> Vec<PendingTransaction> {
		let (number, timestamp) = {
			let chain = self.chain.read();
			(chain.best_block_number(), chain.best_block_timestamp())
		};
		self.miner.ready_transactions(number, timestamp)
	}

	fn queue_consensus_message(&self, message: Bytes) {
		let channel = self.io_channel.lock().clone();
		if let Err(e) = channel.send(ClientIoMessage::NewMessage(message)) {
			debug!("Ignoring the message, error queueing: {}", e);
		}
	}

	fn signing_chain_id(&self) -> Option<u64> {
		self.engine.signing_chain_id(&self.latest_env_info())
	}

	fn block_extra_info(&self, id: BlockId) -> Option<BTreeMap<String, String>> {
		self.block_header(id)
			.map(|header| self.engine.extra_info(&header.decode()))
	}

	fn uncle_extra_info(&self, id: UncleId) -> Option<BTreeMap<String, String>> {
		self.uncle(id)
			.map(|header| self.engine.extra_info(&header.decode()))
	}

	fn pruning_info(&self) -> PruningInfo {
		PruningInfo {
			earliest_chain: self.chain.read().first_block_number().unwrap_or(1),
			earliest_state: self.state_db.lock().journal_db().earliest_era().unwrap_or(0),
		}
	}

	fn call_contract(&self, block_id: BlockId, address: Address, data: Bytes) -> Result<Bytes, String> {
		let transaction = self.contract_call_tx(block_id, address, data);

		self.call(&transaction, Default::default(), block_id)
			.map_err(|e| format!("{:?}", e))
			.map(|executed| {
				executed.output
			})
	}

	fn transact_contract(&self, address: Address, data: Bytes) -> Result<TransactionImportResult, EthcoreError> {
		let transaction = Transaction {
			nonce: self.latest_nonce(&self.miner.author()),
			action: Action::Call(address),
			gas: self.miner.gas_floor_target(),
			gas_price: self.miner.sensible_gas_price(),
			value: U256::zero(),
			data: data,
		};
		let chain_id = self.engine.signing_chain_id(&self.latest_env_info());
		let signature = self.engine.sign(transaction.hash(chain_id))?;
		let signed = SignedTransaction::new(transaction.with_signature(signature, chain_id))?;
		self.miner.import_own_transaction(self, signed.into())
	}

	fn registrar_address(&self) -> Option<Address> {
		self.registrar.lock().as_ref().map(|r| r.address)
	}

	fn registry_address(&self, name: String) -> Option<Address> {
		self.registrar.lock().as_ref()
			.and_then(|r| {
				let dispatch = move |reg_addr, data| {
					future::done(self.call_contract(BlockId::Latest, reg_addr, data))
				};
				r.get_address(dispatch, keccak(name.as_bytes()), "A".to_string()).wait().ok()
			})
			.and_then(|a| if a.is_zero() { None } else { Some(a) })
	}

	fn eip86_transition(&self) -> u64 {
		self.engine().params().eip86_transition
	}
}

impl MiningBlockChainClient for Client {
	fn latest_schedule(&self) -> Schedule {
		self.engine.schedule(self.latest_env_info().number)
	}

	fn prepare_open_block(&self, author: Address, gas_range_target: (U256, U256), extra_data: Bytes) -> OpenBlock {
		let engine = &*self.engine;
		let chain = self.chain.read();
		let h = chain.best_block_hash();
		let best_header = &chain.block_header(&h)
			.expect("h is best block hash: so its header must exist: qed");

		let is_epoch_begin = chain.epoch_transition(best_header.number(), h).is_some();
		let mut open_block = OpenBlock::new(
			engine,
			self.factories.clone(),
			self.tracedb.read().tracing_enabled(),
			self.state_db.lock().boxed_clone_canon(&h),
			best_header,
			self.build_last_hashes(h.clone()),
			author,
			gas_range_target,
			extra_data,
			is_epoch_begin,
		).expect("OpenBlock::new only fails if parent state root invalid; state root of best block's header is never invalid; qed");

		// Add uncles
		chain
			.find_uncle_headers(&h, engine.maximum_uncle_age())
			.unwrap_or_else(Vec::new)
			.into_iter()
			.take(engine.maximum_uncle_count(open_block.header().number()))
			.foreach(|h| {
				open_block.push_uncle(h).expect("pushing maximum_uncle_count;
												open_block was just created;
												push_uncle is not ok only if more than maximum_uncle_count is pushed;
												so all push_uncle are Ok;
												qed");
			});

		open_block
	}

	fn reopen_block(&self, block: ClosedBlock) -> OpenBlock {
		let engine = &*self.engine;
		let mut block = block.reopen(engine);
		let max_uncles = engine.maximum_uncle_count(block.header().number());
		if block.uncles().len() < max_uncles {
			let chain = self.chain.read();
			let h = chain.best_block_hash();
			// Add new uncles
			let uncles = chain
				.find_uncle_hashes(&h, engine.maximum_uncle_age())
				.unwrap_or_else(Vec::new);

			for h in uncles {
				if !block.uncles().iter().any(|header| header.hash() == h) {
					let uncle = chain.block_header(&h).expect("find_uncle_hashes only returns hashes for existing headers; qed");
					block.push_uncle(uncle).expect("pushing up to maximum_uncle_count;
												push_uncle is not ok only if more than maximum_uncle_count is pushed;
												so all push_uncle are Ok;
												qed");
					if block.uncles().len() >= max_uncles { break }
				}
			}

		}
		block
	}

	fn vm_factory(&self) -> &EvmFactory {
		&self.factories.vm
	}

	fn broadcast_proposal_block(&self, block: SealedBlock) {
		self.notify(|notify| {
			notify.new_blocks(
				vec![],
				vec![],
				vec![],
				vec![],
				vec![],
				vec![block.rlp_bytes()],
				0,
			);
		});
	}

	fn import_sealed_block(&self, block: SealedBlock) -> ImportResult {
		let h = block.header().hash();
		let start = precise_time_ns();
		let route = {
			// scope for self.import_lock
			let _import_lock = self.import_lock.lock();
			let _timer = PerfTimer::new("import_sealed_block");

			let number = block.header().number();
			let block_data = block.rlp_bytes();
			let header = block.header().clone();

			let route = self.commit_block(block, &header, &block_data);
			trace!(target: "client", "Imported sealed block #{} ({})", number, h);
			self.state_db.lock().sync_cache(&route.enacted, &route.retracted, false);
			route
		};
		let (enacted, retracted) = self.calculate_enacted_retracted(&[route]);
		self.miner.chain_new_blocks(self, &[h.clone()], &[], &enacted, &retracted);
		self.notify(|notify| {
			notify.new_blocks(
				vec![h.clone()],
				vec![],
				enacted.clone(),
				retracted.clone(),
				vec![h.clone()],
				vec![],
				precise_time_ns() - start,
			);
		});
		self.db.read().flush().expect("DB flush failed.");
		Ok(h)
	}
}

impl super::traits::EngineClient for Client {
	fn update_sealing(&self) {
		self.miner.update_sealing(self)
	}

	fn submit_seal(&self, block_hash: H256, seal: Vec<Bytes>) {
		if self.miner.submit_seal(self, block_hash, seal).is_err() {
			warn!(target: "poa", "Wrong internal seal submission!")
		}
	}

	fn broadcast_consensus_message(&self, message: Bytes) {
		self.notify(|notify| notify.broadcast(message.clone()));
	}

	fn epoch_transition_for(&self, parent_hash: H256) -> Option<::engines::EpochTransition> {
		self.chain.read().epoch_transition_for(parent_hash)
	}

	fn chain_info(&self) -> BlockChainInfo {
		BlockChainClient::chain_info(self)
	}

	fn as_full_client(&self) -> Option<&BlockChainClient> { Some(self) }

	fn block_number(&self, id: BlockId) -> Option<BlockNumber> {
		BlockChainClient::block_number(self, id)
	}
}

impl ProvingBlockChainClient for Client {
	fn prove_storage(&self, key1: H256, key2: H256, id: BlockId) -> Option<(Vec<Bytes>, H256)> {
		self.state_at(id)
			.and_then(move |state| state.prove_storage(key1, key2).ok())
	}

	fn prove_account(&self, key1: H256, id: BlockId) -> Option<(Vec<Bytes>, ::types::basic_account::BasicAccount)> {
		self.state_at(id)
			.and_then(move |state| state.prove_account(key1).ok())
	}

	fn prove_transaction(&self, transaction: SignedTransaction, id: BlockId) -> Option<(Bytes, Vec<DBValue>)> {
		let (header, mut env_info) = match (self.block_header(id), self.env_info(id)) {
			(Some(s), Some(e)) => (s, e),
			_ => return None,
		};

		env_info.gas_limit = transaction.gas.clone();
		let mut jdb = self.state_db.lock().journal_db().boxed_clone();

		state::prove_transaction(
			jdb.as_hashdb_mut(),
			header.state_root().clone(),
			&transaction,
			self.engine.machine(),
			&env_info,
			self.factories.clone(),
			false,
		)
	}


	fn epoch_signal(&self, hash: H256) -> Option<Vec<u8>> {
		// pending transitions are never deleted, and do not contain
		// finality proofs by definition.
		self.chain.read().get_pending_transition(hash).map(|pending| pending.proof)
	}
}

impl Drop for Client {
	fn drop(&mut self) {
		self.engine.stop();
	}
}

/// Returns `LocalizedReceipt` given `LocalizedTransaction`
/// and a vector of receipts from given block up to transaction index.
fn transaction_receipt(machine: &::machine::EthereumMachine, mut tx: LocalizedTransaction, mut receipts: Vec<Receipt>) -> LocalizedReceipt {
	assert_eq!(receipts.len(), tx.transaction_index + 1, "All previous receipts are provided.");

	let sender = tx.sender();
	let receipt = receipts.pop().expect("Current receipt is provided; qed");
	let prior_gas_used = match tx.transaction_index {
		0 => 0.into(),
		i => receipts.get(i - 1).expect("All previous receipts are provided; qed").gas_used,
	};
	let no_of_logs = receipts.into_iter().map(|receipt| receipt.logs.len()).sum::<usize>();
	let transaction_hash = tx.hash();
	let block_hash = tx.block_hash;
	let block_number = tx.block_number;
	let transaction_index = tx.transaction_index;

	LocalizedReceipt {
		transaction_hash: transaction_hash,
		transaction_index: transaction_index,
		block_hash: block_hash,
		block_number: block_number,
		cumulative_gas_used: receipt.gas_used,
		gas_used: receipt.gas_used - prior_gas_used,
		contract_address: match tx.action {
			Action::Call(_) => None,
			Action::Create => Some(contract_address(machine.create_address_scheme(block_number), &sender, &tx.nonce, &tx.data).0)
		},
		logs: receipt.logs.into_iter().enumerate().map(|(i, log)| LocalizedLogEntry {
			entry: log,
			block_hash: block_hash,
			block_number: block_number,
			transaction_hash: transaction_hash,
			transaction_index: transaction_index,
			transaction_log_index: i,
			log_index: no_of_logs + i,
		}).collect(),
		log_bloom: receipt.log_bloom,
		outcome: receipt.outcome,
	}
}

#[cfg(test)]
mod tests {

	#[test]
	fn should_not_cache_details_before_commit() {
		use client::BlockChainClient;
		use tests::helpers::*;

		use std::thread;
		use std::time::Duration;
		use std::sync::Arc;
		use std::sync::atomic::{AtomicBool, Ordering};
		use kvdb::DBTransaction;

		let client = generate_dummy_client(0);
		let genesis = client.chain_info().best_block_hash;
		let (new_hash, new_block) = get_good_dummy_block_hash();

		let go = {
			// Separate thread uncommited transaction
			let go = Arc::new(AtomicBool::new(false));
			let go_thread = go.clone();
			let another_client = client.clone();
			thread::spawn(move || {
				let mut batch = DBTransaction::new();
				another_client.chain.read().insert_block(&mut batch, &new_block, Vec::new());
				go_thread.store(true, Ordering::SeqCst);
			});
			go
		};

		while !go.load(Ordering::SeqCst) { thread::park_timeout(Duration::from_millis(5)); }

		assert!(client.tree_route(&genesis, &new_hash).is_none());
	}

	#[test]
	fn should_return_correct_log_index() {
		use hash::keccak;
		use super::transaction_receipt;
		use ethkey::KeyPair;
		use log_entry::{LogEntry, LocalizedLogEntry};
		use receipt::{Receipt, LocalizedReceipt, TransactionOutcome};
		use transaction::{Transaction, LocalizedTransaction, Action};

		// given
		let key = KeyPair::from_secret_slice(&keccak("test")).unwrap();
		let secret = key.secret();
		let machine = ::ethereum::new_frontier_test_machine();

		let block_number = 1;
		let block_hash = 5.into();
		let state_root = 99.into();
		let gas_used = 10.into();
		let raw_tx = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 21000.into(),
			action: Action::Call(10.into()),
			value: 0.into(),
			data: vec![],
		};
		let tx1 = raw_tx.clone().sign(secret, None);
		let transaction = LocalizedTransaction {
			signed: tx1.clone().into(),
			block_number: block_number,
			block_hash: block_hash,
			transaction_index: 1,
			cached_sender: Some(tx1.sender()),
		};
		let logs = vec![LogEntry {
			address: 5.into(),
			topics: vec![],
			data: vec![],
		}, LogEntry {
			address: 15.into(),
			topics: vec![],
			data: vec![],
		}];
		let receipts = vec![Receipt {
			outcome: TransactionOutcome::StateRoot(state_root),
			gas_used: 5.into(),
			log_bloom: Default::default(),
			logs: vec![logs[0].clone()],
		}, Receipt {
			outcome: TransactionOutcome::StateRoot(state_root),
			gas_used: gas_used,
			log_bloom: Default::default(),
			logs: logs.clone(),
		}];

		// when
		let receipt = transaction_receipt(&machine, transaction, receipts);

		// then
		assert_eq!(receipt, LocalizedReceipt {
			transaction_hash: tx1.hash(),
			transaction_index: 1,
			block_hash: block_hash,
			block_number: block_number,
			cumulative_gas_used: gas_used,
			gas_used: gas_used - 5.into(),
			contract_address: None,
			logs: vec![LocalizedLogEntry {
				entry: logs[0].clone(),
				block_hash: block_hash,
				block_number: block_number,
				transaction_hash: tx1.hash(),
				transaction_index: 1,
				transaction_log_index: 0,
				log_index: 1,
			}, LocalizedLogEntry {
				entry: logs[1].clone(),
				block_hash: block_hash,
				block_number: block_number,
				transaction_hash: tx1.hash(),
				transaction_index: 1,
				transaction_log_index: 1,
				log_index: 2,
			}],
			log_bloom: Default::default(),
			outcome: TransactionOutcome::StateRoot(state_root),
		});
	}
}
