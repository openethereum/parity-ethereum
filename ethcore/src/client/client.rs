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
use std::fmt;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering as AtomicOrdering};
use std::time::{Instant};
use time::precise_time_ns;

// util
use util::{Bytes, PerfTimer, Itertools, Mutex, RwLock, MutexGuard, Hashable};
use util::{journaldb, TrieFactory, Trie};
use util::{U256, H256, Address, H2048, Uint, FixedHash};
use util::trie::TrieSpec;
use util::kvdb::*;

// other
use io::*;
use views::BlockView;
use error::{ImportError, ExecutionError, CallError, BlockError, ImportResult, Error as EthcoreError};
use header::BlockNumber;
use state::{State, CleanupMode};
use spec::Spec;
use basic_types::Seal;
use engines::Engine;
use service::ClientIoMessage;
use env_info::LastHashes;
use verification;
use verification::{PreverifiedBlock, Verifier};
use block::*;
use transaction::{LocalizedTransaction, UnverifiedTransaction, SignedTransaction, Transaction, PendingTransaction, Action};
use blockchain::extras::TransactionAddress;
use types::filter::Filter;
use types::mode::Mode as IpcMode;
use log_entry::LocalizedLogEntry;
use verification::queue::BlockQueue;
use blockchain::{BlockChain, BlockProvider, TreeRoute, ImportRoute};
use client::{
	BlockId, TransactionId, UncleId, TraceId, ClientConfig, BlockChainClient,
	MiningBlockChainClient, EngineClient, TraceFilter, CallAnalytics, BlockImportError, Mode,
	ChainNotify, PruningInfo,
};
use client::Error as ClientError;
use env_info::EnvInfo;
use executive::{Executive, Executed, TransactOptions, contract_address};
use receipt::{Receipt, LocalizedReceipt};
use trace::{TraceDB, ImportRequest as TraceImportRequest, LocalizedTrace, Database as TraceDatabase};
use trace;
use trace::FlatTransactionTraces;
use evm::{Factory as EvmFactory, Schedule};
use miner::{Miner, MinerService, TransactionImportResult};
use snapshot::{self, io as snapshot_io};
use factory::Factories;
use rlp::{View, UntrustedRlp};
use state_db::StateDB;
use rand::OsRng;
use client::registry::Registry;
use encoded;

// re-export
pub use types::blockchain_info::BlockChainInfo;
pub use types::block_status::BlockStatus;
pub use blockchain::CacheSize as BlockChainCacheSize;
pub use verification::queue::QueueInfo as BlockQueueInfo;

const MAX_TX_QUEUE_SIZE: usize = 4096;
const MAX_QUEUE_SIZE_TO_SLEEP_ON: usize = 2;
const MIN_HISTORY_SIZE: u64 = 8;

impl fmt::Display for BlockChainInfo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "#{}.{}", self.best_block_number, self.best_block_hash)
	}
}

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
	engine: Arc<Engine>,
	config: ClientConfig,
	pruning: journaldb::Algorithm,
	db: RwLock<Arc<KeyValueDB>>,
	state_db: Mutex<StateDB>,
	block_queue: BlockQueue,
	report: RwLock<ClientReport>,
	import_lock: Mutex<()>,
	panic_handler: Arc<PanicHandler>,
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
	on_mode_change: Mutex<Option<Box<FnMut(&Mode) + 'static + Send>>>,
	registrar: Mutex<Option<Registry>>,
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
	) -> Result<Arc<Client>, ClientError> {

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
		let panic_handler = PanicHandler::new_in_arc();
		panic_handler.forward_from(&block_queue);

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
			panic_handler: panic_handler,
			miner: miner,
			io_channel: Mutex::new(message_channel),
			notify: RwLock::new(Vec::new()),
			queue_transactions: AtomicUsize::new(0),
			last_hashes: RwLock::new(VecDeque::new()),
			factories: factories,
			history: history,
			rng: Mutex::new(OsRng::new().map_err(::util::UtilError::StdIo)?),
			on_mode_change: Mutex::new(None),
			registrar: Mutex::new(None),
		});

		{
			let state_db = client.state_db.lock().boxed_clone();
			let chain = client.chain.read();
			client.prune_ancient(state_db, &chain)?;
		}

		if let Some(reg_addr) = client.additional_params().get("registrar").and_then(|s| Address::from_str(s).ok()) {
			trace!(target: "client", "Found registrar at {}", reg_addr);
			let weak = Arc::downgrade(&client);
			let registrar = Registry::new(reg_addr, move |a, d| weak.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d)));
			*client.registrar.lock() = Some(registrar);
		}
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

	/// Returns engine reference.
	pub fn engine(&self) -> &Engine {
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

	/// Register an action to be done if a mode change happens.
	pub fn on_mode_change<F>(&self, f: F) where F: 'static + FnMut(&Mode) + Send {
		*self.on_mode_change.lock() = Some(Box::new(f));
	}

	/// Flush the block import queue.
	pub fn flush_queue(&self) {
		self.block_queue.flush();
		while !self.block_queue.queue_info().is_empty() {
			self.import_verified_blocks();
		}
	}

	/// The env info as of the best block.
	fn latest_env_info(&self) -> EnvInfo {
		let header = self.best_block_header();

		EnvInfo {
			number: header.number(),
			author: header.author(),
			timestamp: header.timestamp(),
			difficulty: header.difficulty(),
			last_hashes: self.build_last_hashes(header.hash()),
			gas_used: U256::default(),
			gas_limit: header.gas_limit(),
		}
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
		if best_block_number >= self.history && header.number() <= best_block_number - self.history {
			warn!(target: "client", "Block import failed for #{} ({})\nBlock is ancient (current best block: #{}).", header.number(), header.hash(), best_block_number);
			return Err(());
		}

		// Verify Block Family
		let verify_family_result = self.verifier.verify_block_family(header, &block.bytes, engine, &**chain);
		if let Err(e) = verify_family_result {
			warn!(target: "client", "Stage 3 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
			return Err(());
		};

		// Check if Parent is in chain
		let chain_has_parent = chain.block_header(header.parent_hash());
		if let Some(parent) = chain_has_parent {
			// Enact Verified Block
			let last_hashes = self.build_last_hashes(header.parent_hash().clone());
			let db = self.state_db.lock().boxed_clone_canon(header.parent_hash());

			let enact_result = enact_verified(block, engine, self.tracedb.read().tracing_enabled(), db, &parent, last_hashes, self.factories.clone());
			let locked_block = enact_result.map_err(|e| {
				warn!(target: "client", "Block import failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
			})?;

			// Final Verification
			if let Err(e) = self.verifier.verify_block_final(header, locked_block.block().header()) {
				warn!(target: "client", "Stage 4 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
				return Err(());
			}

			Ok(locked_block)
		} else {
			warn!(target: "client", "Block import failed for #{} ({}): Parent not found ({}) ", header.number(), header.hash(), header.parent_hash());
			Err(())
		}
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

						let route = self.commit_block(closed_block, &header.hash(), &block.bytes);
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
		let hash = header.hash();
		let _import_lock = self.import_lock.lock();
		{
			let _timer = PerfTimer::new("import_old_block");
			let mut rng = self.rng.lock();
			let chain = self.chain.read();

			// verify block.
			::snapshot::verify_old_block(
				&mut *rng,
				&header,
				&*self.engine,
				&*chain,
				Some(&block_bytes),
				false,
			)?;

			// Commit results
			let receipts = ::rlp::decode(&receipts_bytes);
			let mut batch = DBTransaction::new();
			chain.insert_unordered_block(&mut batch, &block_bytes, receipts, None, false, true);
			// Final commit to the DB
			self.db.read().write_buffered(batch);
			chain.commit();
		}
		self.db.read().flush().expect("DB flush failed.");
		Ok(hash)
	}

	fn commit_block<B>(&self, block: B, hash: &H256, block_data: &[u8]) -> ImportRoute where B: IsBlock + Drain {
		let number = block.header().number();
		let parent = block.header().parent_hash().clone();
		let chain = self.chain.read();

		// Commit results
		let receipts = block.receipts().to_owned();
		let traces = block.traces().clone().unwrap_or_else(Vec::new);
		let traces: Vec<FlatTransactionTraces> = traces.into_iter()
			.map(Into::into)
			.collect();

		//let traces = From::from(block.traces().clone().unwrap_or_else(Vec::new));

		let mut batch = DBTransaction::new();
		// CHECK! I *think* this is fine, even if the state_root is equal to another
		// already-imported block of the same number.
		// TODO: Prove it with a test.
		let mut state = block.drain();

		state.journal_under(&mut batch, number, hash).expect("DB commit failed");
		let route = chain.insert_block(&mut batch, block_data, receipts);
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
		self.update_last_hashes(&parent, hash);

		if let Err(e) = self.prune_ancient(state, &chain) {
			warn!("Failed to prune ancient state data: {}", e);
		}

		route
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
			BlockId::Pending => return self.miner.pending_state().or_else(|| Some(self.state())),
			BlockId::Latest => return Some(self.state()),
			_ => {},
		}

		let block_number = match self.block_number(id.clone()) {
			Some(num) => num,
			None => return None,
		};

		self.block_header(id).and_then(|header| {
			let db = self.state_db.lock().boxed_clone();

			// early exit for pruned blocks
			if db.is_pruned() && self.chain.read().best_block_number() >= block_number + self.history {
				return None;
			}

			let root = header.state_root();
			State::from_existing(db, root, self.engine.account_start_nonce(), self.factories.clone()).ok()
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
			self.engine.account_start_nonce(),
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
	pub fn tick(&self) {
		self.check_garbage();
		self.check_snooze();
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

		if best_block_number > self.history + block_number && db.is_pruned() {
			return Err(snapshot::Error::OldBlockPrunedDB.into());
		}

		let history = ::std::cmp::min(self.history, 1000);

		let start_hash = match at {
			BlockId::Latest => {
				let start_num = match db.earliest_era() {
					Some(era) => ::std::cmp::max(era, best_block_number - history),
					None => best_block_number - history,
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

		snapshot::take_snapshot(&self.chain.read(), start_hash, db.as_hashdb(), writer, p)?;

		Ok(())
	}

	/// Ask the client what the history parameter is.
	pub fn pruning_history(&self) -> u64 {
		self.history
	}

	fn block_hash(chain: &BlockChain, id: BlockId) -> Option<H256> {
		match id {
			BlockId::Hash(hash) => Some(hash),
			BlockId::Number(number) => chain.block_hash(number),
			BlockId::Earliest => chain.block_hash(0),
			BlockId::Latest | BlockId::Pending => Some(chain.best_block_hash()),
		}
	}

	fn transaction_address(&self, id: TransactionId) -> Option<TransactionAddress> {
		match id {
			TransactionId::Hash(ref hash) => self.chain.read().transaction_address(hash),
			TransactionId::Location(id, index) => Self::block_hash(&self.chain.read(), id).map(|hash| TransactionAddress {
				block_hash: hash,
				index: index,
			})
		}
	}

	fn wake_up(&self) {
		if !self.liveness.load(AtomicOrdering::Relaxed) {
			self.liveness.store(true, AtomicOrdering::Relaxed);
			self.notify(|n| n.start());
			trace!(target: "mode", "wake_up: Waking.");
		}
	}

	fn sleep(&self) {
		if self.liveness.load(AtomicOrdering::Relaxed) {
			// only sleep if the import queue is mostly empty.
			if self.queue_info().total_queue_size() <= MAX_QUEUE_SIZE_TO_SLEEP_ON {
				self.liveness.store(false, AtomicOrdering::Relaxed);
				self.notify(|n| n.stop());
				trace!(target: "mode", "sleep: Sleeping.");
			} else {
				trace!(target: "mode", "sleep: Cannot sleep - syncing ongoing.");
				// TODO: Consider uncommenting.
				//*self.last_activity.lock() = Some(Instant::now());
			}
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
	fn call(&self, t: &SignedTransaction, block: BlockId, analytics: CallAnalytics) -> Result<Executed, CallError> {
		let header = self.block_header(block).ok_or(CallError::StatePruned)?;
		let last_hashes = self.build_last_hashes(header.parent_hash());
		let env_info = EnvInfo {
			number: header.number(),
			author: header.author(),
			timestamp: header.timestamp(),
			difficulty: header.difficulty(),
			last_hashes: last_hashes,
			gas_used: U256::zero(),
			gas_limit: U256::max_value(),
		};
		// that's just a copy of the state.
		let mut state = self.state_at(block).ok_or(CallError::StatePruned)?;
		let original_state = if analytics.state_diffing { Some(state.clone()) } else { None };

		let sender = t.sender();
		let balance = state.balance(&sender).map_err(|_| CallError::StateCorrupt)?;
		let needed_balance = t.value + t.gas * t.gas_price;
		if balance < needed_balance {
			// give the sender a sufficient balance
			state.add_balance(&sender, &(needed_balance - balance), CleanupMode::NoEmpty)
				.map_err(|_| CallError::StateCorrupt)?;
		}
		let options = TransactOptions { tracing: analytics.transaction_tracing, vm_tracing: analytics.vm_tracing, check_nonce: false };
		let mut ret = Executive::new(&mut state, &env_info, &*self.engine, &self.factories.vm).transact(t, options)?;

		// TODO gav move this into Executive.
		if let Some(original) = original_state {
			ret.state_diff = Some(state.diff_from(original).map_err(ExecutionError::from)?);
		}

		Ok(ret)
	}

	fn estimate_gas(&self, t: &SignedTransaction, block: BlockId) -> Result<U256, CallError> {
		const UPPER_CEILING: u64 = 1_000_000_000_000u64;
		let header = self.block_header(block).ok_or(CallError::StatePruned)?;
		let last_hashes = self.build_last_hashes(header.parent_hash());
		let env_info = EnvInfo {
			number: header.number(),
			author: header.author(),
			timestamp: header.timestamp(),
			difficulty: header.difficulty(),
			last_hashes: last_hashes,
			gas_used: U256::zero(),
			gas_limit: UPPER_CEILING.into(),
		};
		// that's just a copy of the state.
		let original_state = self.state_at(block).ok_or(CallError::StatePruned)?;
		let sender = t.sender();
		let balance = original_state.balance(&sender).map_err(ExecutionError::from)?;
		let options = TransactOptions { tracing: true, vm_tracing: false, check_nonce: false };

		let cond = |gas| {
			let mut tx = t.as_unsigned().clone();
			tx.gas = gas;
			let tx = tx.fake_sign(sender);

			let mut state = original_state.clone();
			let needed_balance = tx.value + tx.gas * tx.gas_price;
			if balance < needed_balance {
				// give the sender a sufficient balance
				state.add_balance(&sender, &(needed_balance - balance), CleanupMode::NoEmpty)
					.map_err(ExecutionError::from)?;
			}

			Ok(Executive::new(&mut state, &env_info, &*self.engine, &self.factories.vm)
				.transact(&tx, options.clone())
				.map(|r| r.exception.is_none())
				.unwrap_or(false))
		};

		let mut upper = header.gas_limit();
		if !cond(upper)? {
			// impossible at block gas limit - try `UPPER_CEILING` instead.
			// TODO: consider raising limit by powers of two.
			upper = UPPER_CEILING.into();
			if !cond(upper)? {
				trace!(target: "estimate_gas", "estimate_gas failed with {}", upper);
				let err = ExecutionError::Internal(format!("Requires higher than upper limit of {}", upper));
				return Err(err.into())
			}
		}
		let lower = t.gas_required(&self.engine.schedule(&env_info)).into();
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
		let header = self.block_header(BlockId::Hash(address.block_hash)).ok_or(CallError::StatePruned)?;
		let body = self.block_body(BlockId::Hash(address.block_hash)).ok_or(CallError::StatePruned)?;
		let mut state = self.state_at_beginning(BlockId::Hash(address.block_hash)).ok_or(CallError::StatePruned)?;
		let mut txs = body.transactions();

		if address.index >= txs.len() {
			return Err(CallError::TransactionNotFound);
		}

		let options = TransactOptions { tracing: analytics.transaction_tracing, vm_tracing: analytics.vm_tracing, check_nonce: false };
		let last_hashes = self.build_last_hashes(header.hash());
		let mut env_info = EnvInfo {
			number: header.number(),
			author: header.author(),
			timestamp: header.timestamp(),
			difficulty: header.difficulty(),
			last_hashes: last_hashes,
			gas_used: U256::default(),
			gas_limit: header.gas_limit(),
		};
		const PROOF: &'static str = "Transactions fetched from blockchain; blockchain transactions are valid; qed";
		let rest = txs.split_off(address.index);
		for t in txs {
			let t = SignedTransaction::new(t).expect(PROOF);
			let x = Executive::new(&mut state, &env_info, &*self.engine, &self.factories.vm).transact(&t, Default::default())?;
			env_info.gas_used = env_info.gas_used + x.gas_used;
		}
		let first = rest.into_iter().next().expect("We split off < `address.index`; Length is checked earlier; qed");
		let t = SignedTransaction::new(first).expect(PROOF);
		let original_state = if analytics.state_diffing { Some(state.clone()) } else { None };
		let mut ret = Executive::new(&mut state, &env_info, &*self.engine, &self.factories.vm).transact(&t, options)?;
		if let Some(original) = original_state {
			ret.state_diff = Some(state.diff_from(original).map_err(ExecutionError::from)?)
		}
		Ok(ret)
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
			if let Some(ref mut f) = *self.on_mode_change.lock() {
				trace!(target: "mode", "Making callback...");
				f(&*mode)
			}
		}
		match new_mode {
			IpcMode::Active => self.wake_up(),
			IpcMode::Off => self.sleep(),
			_ => {(*self.sleep_state.lock()).last_activity = Some(Instant::now()); }
		}
	}

	fn best_block_header(&self) -> encoded::Header {
		self.chain.read().best_block_header()
	}

	fn block_header(&self, id: BlockId) -> Option<::encoded::Header> {
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| chain.block_header_data(&hash))
	}

	fn block_number(&self, id: BlockId) -> Option<BlockNumber> {
		match id {
			BlockId::Number(number) => Some(number),
			BlockId::Hash(ref hash) => self.chain.read().block_number(hash),
			BlockId::Earliest => Some(0),
			BlockId::Latest | BlockId::Pending => Some(self.chain.read().best_block_number()),
		}
	}

	fn block_body(&self, id: BlockId) -> Option<encoded::Body> {
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| chain.block_body(&hash))
	}

	fn block(&self, id: BlockId) -> Option<encoded::Block> {
		if let BlockId::Pending = id {
			if let Some(block) = self.miner.pending_block() {
				return Some(encoded::Block::new(block.rlp_bytes(Seal::Without)));
			}
		}
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| {
			chain.block(&hash)
		})
	}

	fn block_status(&self, id: BlockId) -> BlockStatus {
		let chain = self.chain.read();
		match Self::block_hash(&chain, id) {
			Some(ref hash) if chain.is_known(hash) => BlockStatus::InChain,
			Some(hash) => self.block_queue.status(&hash).into(),
			None => BlockStatus::Unknown
		}
	}

	fn block_total_difficulty(&self, id: BlockId) -> Option<U256> {
		if let BlockId::Pending = id {
			if let Some(block) = self.miner.pending_block() {
				return Some(*block.header.difficulty() + self.block_total_difficulty(BlockId::Latest).expect("blocks in chain have details; qed"));
			}
		}
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| chain.block_details(&hash)).map(|d| d.total_difficulty)
	}

	fn nonce(&self, address: &Address, id: BlockId) -> Option<U256> {
		self.state_at(id).and_then(|s| s.nonce(address).ok())
	}

	fn storage_root(&self, address: &Address, id: BlockId) -> Option<H256> {
		self.state_at(id).and_then(|s| s.storage_root(address).ok()).and_then(|x| x)
	}

	fn block_hash(&self, id: BlockId) -> Option<H256> {
		let chain = self.chain.read();
		Self::block_hash(&chain, id)
	}

	fn code(&self, address: &Address, id: BlockId) -> Option<Option<Bytes>> {
		self.state_at(id).and_then(|s| s.code(address).ok()).map(|c| c.map(|c| (&*c).clone()))
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
		let account_db = self.factories.accountdb.readonly(db.as_hashdb(), account.sha3());
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
						Some(transaction_receipt(transaction, previous_receipts))
					},
					_ => None,
				}
			}))
	}

	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute> {
		let chain = self.chain.read();
		match chain.is_known(from) && chain.is_known(to) {
			true => Some(chain.tree_route(from.clone(), to.clone())),
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
		self.chain.read().block_receipts(hash).map(|receipts| ::rlp::encode(&receipts).to_vec())
	}

	fn import_block(&self, bytes: Bytes) -> Result<H256, BlockImportError> {
		use verification::queue::kind::BlockLike;
		use verification::queue::kind::blocks::Unverified;

		// create unverified block here so the `sha3` calculation can be cached.
		let unverified = Unverified::new(bytes);

		{
			if self.chain.read().is_known(&unverified.hash()) {
				return Err(BlockImportError::Import(ImportError::AlreadyInChain));
			}
			if self.block_status(BlockId::Hash(unverified.parent_hash())) == BlockStatus::Unknown {
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
			if self.block_status(BlockId::Hash(header.parent_hash())) == BlockStatus::Unknown {
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

	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockId, to_block: BlockId) -> Option<Vec<BlockNumber>> {
		match (self.block_number(from_block), self.block_number(to_block)) {
			(Some(from), Some(to)) => Some(self.chain.read().blocks_with_bloom(bloom, from, to)),
			_ => None
		}
	}

	fn logs(&self, filter: Filter) -> Vec<LocalizedLogEntry> {
		let blocks = filter.bloom_possibilities().iter()
			.filter_map(|bloom| self.blocks_with_bloom(bloom, filter.from_block.clone(), filter.to_block.clone()))
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
				let filter = trace::Filter {
					range: s as usize..e as usize,
					from_address: From::from(filter.from_address),
					to_address: From::from(filter.to_address),
				};

				let traces = self.tracedb.read().filter(&filter);
				Some(traces)
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

	fn signing_network_id(&self) -> Option<u64> {
		self.engine.signing_network_id(&self.latest_env_info())
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

	fn call_contract(&self, address: Address, data: Bytes) -> Result<Bytes, String> {
		let from = Address::default();
		let transaction = Transaction {
			nonce: self.latest_nonce(&from),
			action: Action::Call(address),
			gas: U256::from(50_000_000),
			gas_price: U256::default(),
			value: U256::default(),
			data: data,
		}.fake_sign(from);

		self.call(&transaction, BlockId::Latest, Default::default())
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
		let network_id = self.engine.signing_network_id(&self.latest_env_info());
		let signature = self.engine.sign(transaction.hash(network_id))?;
		let signed = SignedTransaction::new(transaction.with_signature(signature, network_id))?;
		self.miner.import_own_transaction(self, signed.into())
	}

	fn registrar_address(&self) -> Option<Address> {
		self.registrar.lock().as_ref().map(|r| r.address.clone())
	}

	fn registry_address(&self, name: String) -> Option<Address> {
		self.registrar.lock().as_ref()
			.and_then(|r| r.get_address(&(name.as_bytes().sha3()), "A").ok())
			.and_then(|a| if a.is_zero() { None } else { Some(a) })
	}
}

impl MiningBlockChainClient for Client {
	fn latest_schedule(&self) -> Schedule {
		self.engine.schedule(&self.latest_env_info())
	}

	fn prepare_open_block(&self, author: Address, gas_range_target: (U256, U256), extra_data: Bytes) -> OpenBlock {
		let engine = &*self.engine;
		let chain = self.chain.read();
		let h = chain.best_block_hash();

		let mut open_block = OpenBlock::new(
			engine,
			self.factories.clone(),
			false,	// TODO: this will need to be parameterised once we want to do immediate mining insertion.
			self.state_db.lock().boxed_clone_canon(&h),
			&chain.block_header(&h).expect("h is best block hash: so its header must exist: qed"),
			self.build_last_hashes(h.clone()),
			author,
			gas_range_target,
			extra_data,
		).expect("OpenBlock::new only fails if parent state root invalid; state root of best block's header is never invalid; qed");

		// Add uncles
		chain
			.find_uncle_headers(&h, engine.maximum_uncle_age())
			.unwrap_or_else(Vec::new)
			.into_iter()
			.take(engine.maximum_uncle_count())
			.foreach(|h| {
				open_block.push_uncle(h).expect("pushing maximum_uncle_count;
												open_block was just created;
												push_uncle is not ok only if more than maximum_uncle_count is pushed;
												so all push_uncle are Ok;
												qed");
			});

		open_block
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
			let route = self.commit_block(block, &h, &block_data);
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

impl EngineClient for Client {
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
}

impl MayPanic for Client {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

impl ::client::ProvingBlockChainClient for Client {
	fn prove_storage(&self, key1: H256, key2: H256, from_level: u32, id: BlockId) -> Vec<Bytes> {
		self.state_at(id)
			.and_then(move |state| state.prove_storage(key1, key2, from_level).ok())
			.unwrap_or_else(Vec::new)
	}

	fn prove_account(&self, key1: H256, from_level: u32, id: BlockId) -> Vec<Bytes> {
		self.state_at(id)
			.and_then(move |state| state.prove_account(key1, from_level).ok())
			.unwrap_or_else(Vec::new)
	}

	fn code_by_hash(&self, account_key: H256, id: BlockId) -> Bytes {
		self.state_at(id)
			.and_then(move |state| state.code_by_address_hash(account_key).ok())
			.and_then(|x| x)
			.unwrap_or_else(Vec::new)
	}
}

impl Drop for Client {
	fn drop(&mut self) {
		self.engine.stop();
	}
}

/// Returns `LocalizedReceipt` given `LocalizedTransaction`
/// and a vector of receipts from given block up to transaction index.
fn transaction_receipt(mut tx: LocalizedTransaction, mut receipts: Vec<Receipt>) -> LocalizedReceipt {
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
		block_number:block_number,
		cumulative_gas_used: receipt.gas_used,
		gas_used: receipt.gas_used - prior_gas_used,
		contract_address: match tx.action {
			Action::Call(_) => None,
			Action::Create => Some(contract_address(&sender, &tx.nonce))
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
		state_root: receipt.state_root,
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
		use util::kvdb::DBTransaction;

		let client = generate_dummy_client(0);
		let genesis = client.chain_info().best_block_hash;
		let (new_hash, new_block) = get_good_dummy_block_hash();

		let go = {
			// Separate thread uncommited transaction
			let go = Arc::new(AtomicBool::new(false));
			let go_thread = go.clone();
			let another_client = client.reference().clone();
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
		use super::transaction_receipt;
		use ethkey::KeyPair;
		use log_entry::{LogEntry, LocalizedLogEntry};
		use receipt::{Receipt, LocalizedReceipt};
		use transaction::{Transaction, LocalizedTransaction, Action};
		use util::Hashable;

		// given
		let key = KeyPair::from_secret_slice(&"test".sha3()).unwrap();
		let secret = key.secret();

		let block_number = 1;
		let block_hash = 5.into();
		let state_root = Some(99.into());
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
			state_root: state_root,
			gas_used: 5.into(),
			log_bloom: Default::default(),
			logs: vec![logs[0].clone()],
		}, Receipt {
			state_root: state_root,
			gas_used: gas_used,
			log_bloom: Default::default(),
			logs: logs.clone(),
		}];

		// when
		let receipt = transaction_receipt(transaction, receipts);

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
			state_root: state_root,
		});
	}
}
