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
use std::collections::{HashSet, HashMap, BTreeMap, VecDeque};
use std::sync::{Arc, Weak};
use std::path::{Path};
use std::fmt;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering as AtomicOrdering};
use std::time::{Instant};
use time::precise_time_ns;

// util
use util::{Bytes, PerfTimer, Itertools, Mutex, RwLock};
use util::journaldb::{self, JournalDB};
use util::{U256, H256, Address, H2048, Uint};
use util::sha3::*;
use util::TrieFactory;
use util::kvdb::*;

// other
use io::*;
use views::{BlockView, HeaderView, BodyView};
use error::{ImportError, ExecutionError, CallError, BlockError, ImportResult, Error as EthcoreError};
use header::BlockNumber;
use state::State;
use spec::Spec;
use basic_types::Seal;
use engines::Engine;
use service::ClientIoMessage;
use env_info::LastHashes;
use verification;
use verification::{PreverifiedBlock, Verifier};
use block::*;
use transaction::{LocalizedTransaction, SignedTransaction, Action};
use blockchain::extras::TransactionAddress;
use types::filter::Filter;
use log_entry::LocalizedLogEntry;
use block_queue::{BlockQueue, BlockQueueInfo};
use blockchain::{BlockChain, BlockProvider, TreeRoute, ImportRoute};
use client::{
	BlockID, TransactionID, UncleID, TraceId, ClientConfig, BlockChainClient,
	MiningBlockChainClient, TraceFilter, CallAnalytics, BlockImportError, Mode,
	ChainNotify
};
use client::Error as ClientError;
use env_info::EnvInfo;
use executive::{Executive, Executed, TransactOptions, contract_address};
use receipt::LocalizedReceipt;
use trace::{TraceDB, ImportRequest as TraceImportRequest, LocalizedTrace, Database as TraceDatabase};
use trace;
use trace::FlatTransactionTraces;
use evm::Factory as EvmFactory;
use miner::{Miner, MinerService};
use snapshot::{self, io as snapshot_io};
use factory::Factories;
use rlp::{View, UntrustedRlp};


// re-export
pub use types::blockchain_info::BlockChainInfo;
pub use types::block_status::BlockStatus;
pub use blockchain::CacheSize as BlockChainCacheSize;

const MAX_TX_QUEUE_SIZE: usize = 4096;
const MAX_QUEUE_SIZE_TO_SLEEP_ON: usize = 2;

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
	mode: Mode,
	chain: RwLock<Arc<BlockChain>>,
	tracedb: RwLock<TraceDB<BlockChain>>,
	engine: Arc<Engine>,
	config: ClientConfig,
	db: RwLock<Arc<Database>>,
	pruning: journaldb::Algorithm,
	state_db: RwLock<Box<JournalDB>>,
	block_queue: BlockQueue,
	report: RwLock<ClientReport>,
	import_lock: Mutex<()>,
	panic_handler: Arc<PanicHandler>,
	verifier: Box<Verifier>,
	miner: Arc<Miner>,
	sleep_state: Mutex<SleepState>,
	liveness: AtomicBool,
	io_channel: IoChannel<ClientIoMessage>,
	notify: RwLock<Vec<Weak<ChainNotify>>>,
	queue_transactions: AtomicUsize,
	last_hashes: RwLock<VecDeque<H256>>,
	factories: Factories,
}

const HISTORY: u64 = 1200;

/// Append a path element to the given path and return the string.
pub fn append_path<P>(path: P, item: &str) -> String where P: AsRef<Path> {
	let mut p = path.as_ref().to_path_buf();
	p.push(item);
	p.to_str().unwrap().to_owned()
}

impl Client {
	/// Create a new client with given spec and DB path and custom verifier.
	pub fn new(
		config: ClientConfig,
		spec: &Spec,
		path: &Path,
		miner: Arc<Miner>,
		message_channel: IoChannel<ClientIoMessage>,
		db_config: &DatabaseConfig,
	) -> Result<Arc<Client>, ClientError> {
		let path = path.to_path_buf();
		let gb = spec.genesis_block();

		let db = Arc::new(try!(Database::open(&db_config, &path.to_str().unwrap()).map_err(ClientError::Database)));
		let chain = Arc::new(BlockChain::new(config.blockchain.clone(), &gb, db.clone()));
		let tracedb = RwLock::new(try!(TraceDB::new(config.tracing.clone(), db.clone(), chain.clone())));

		let mut state_db = journaldb::new(db.clone(), config.pruning, ::db::COL_STATE);
		if state_db.is_empty() && try!(spec.ensure_db_good(state_db.as_hashdb_mut())) {
			let mut batch = DBTransaction::new(&db);
			try!(state_db.commit(&mut batch, 0, &spec.genesis_header().hash(), None));
			try!(db.write(batch).map_err(ClientError::Database));
		}

		if !chain.block_header(&chain.best_block_hash()).map_or(true, |h| state_db.contains(h.state_root())) {
			warn!("State root not found for block #{} ({})", chain.best_block_number(), chain.best_block_hash().hex());
		}

		let engine = spec.engine.clone();

		let block_queue = BlockQueue::new(config.queue.clone(), engine.clone(), message_channel.clone());
		let panic_handler = PanicHandler::new_in_arc();
		panic_handler.forward_from(&block_queue);

		let awake = match config.mode { Mode::Dark(..) => false, _ => true };

		let factories = Factories {
			vm: EvmFactory::new(config.vm_type.clone()),
			trie: TrieFactory::new(config.trie_spec.clone()),
			accountdb: Default::default(),
		};

		let client = Client {
			sleep_state: Mutex::new(SleepState::new(awake)),
			liveness: AtomicBool::new(awake),
			mode: config.mode.clone(),
			chain: RwLock::new(chain),
			tracedb: tracedb,
			engine: engine,
			pruning: config.pruning.clone(),
			verifier: verification::new(config.verifier_type.clone()),
			config: config,
			db: RwLock::new(db),
			state_db: RwLock::new(state_db),
			block_queue: block_queue,
			report: RwLock::new(Default::default()),
			import_lock: Mutex::new(()),
			panic_handler: panic_handler,
			miner: miner,
			io_channel: message_channel,
			notify: RwLock::new(Vec::new()),
			queue_transactions: AtomicUsize::new(0),
			last_hashes: RwLock::new(VecDeque::new()),
			factories: factories,
		};
		Ok(Arc::new(client))
	}

	/// Adds an actor to be notified on certain events
	pub fn add_notify(&self, target: Arc<ChainNotify>) {
		self.notify.write().push(Arc::downgrade(&target));
	}

	fn notify<F>(&self, f: F) where F: Fn(&ChainNotify) {
		for np in self.notify.read().iter() {
			if let Some(n) = np.upgrade() {
				f(&*n);
			}
		}
	}

	/// Flush the block import queue.
	pub fn flush_queue(&self) {
		self.block_queue.flush();
		while !self.block_queue.queue_info().is_empty() {
			self.import_verified_blocks();
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
		if best_block_number >= HISTORY && header.number() <= best_block_number - HISTORY {
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
		if let None = chain_has_parent {
			warn!(target: "client", "Block import failed for #{} ({}): Parent not found ({}) ", header.number(), header.hash(), header.parent_hash());
			return Err(());
		};

		// Enact Verified Block
		let parent = chain_has_parent.unwrap();
		let last_hashes = self.build_last_hashes(header.parent_hash().clone());
		let db = self.state_db.read().boxed_clone();

		let enact_result = enact_verified(block, engine, self.tracedb.read().tracing_enabled(), db, &parent, last_hashes, self.factories.clone());
		if let Err(e) = enact_result {
			warn!(target: "client", "Block import failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
			return Err(());
		};

		// Final Verification
		let locked_block = enact_result.unwrap();
		if let Err(e) = self.verifier.verify_block_final(header, locked_block.block().header()) {
			warn!(target: "client", "Stage 4 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
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
		let max_blocks_to_import = 64;
		let (imported_blocks, import_results, invalid_blocks, imported, duration) = {
			let mut imported_blocks = Vec::with_capacity(max_blocks_to_import);
			let mut invalid_blocks = HashSet::new();
			let mut import_results = Vec::with_capacity(max_blocks_to_import);

			let _import_lock = self.import_lock.lock();
			let _timer = PerfTimer::new("import_verified_blocks");
			let start = precise_time_ns();
			let blocks = self.block_queue.drain(max_blocks_to_import);

			for block in blocks {
				let header = &block.header;
				if invalid_blocks.contains(header.parent_hash()) {
					invalid_blocks.insert(header.hash());
					continue;
				}
				let closed_block = self.check_and_close_block(&block);
				if let Err(_) = closed_block {
					invalid_blocks.insert(header.hash());
					continue;
				}

				let closed_block = closed_block.unwrap();
				imported_blocks.push(header.hash());

				let route = self.commit_block(closed_block, &header.hash(), &block.bytes);
				import_results.push(route);

				self.report.write().accrue_block(&block);
			}

			let imported = imported_blocks.len();
			let invalid_blocks = invalid_blocks.into_iter().collect::<Vec<H256>>();

			{
				if !invalid_blocks.is_empty() {
					self.block_queue.mark_as_bad(&invalid_blocks);
				}
				if !imported_blocks.is_empty() {
					self.block_queue.mark_as_good(&imported_blocks);
				}
			}
			let duration_ns = precise_time_ns() - start;
			(imported_blocks, import_results, invalid_blocks, imported, duration_ns)
		};

		{
			if !imported_blocks.is_empty() && self.block_queue.queue_info().is_empty() {
				let (enacted, retracted) = self.calculate_enacted_retracted(&import_results);

				if self.queue_info().is_empty() {
					self.miner.chain_new_blocks(self, &imported_blocks, &invalid_blocks, &enacted, &retracted);
				}

				self.notify(|notify| {
					notify.new_blocks(
						imported_blocks.clone(),
						invalid_blocks.clone(),
						enacted.clone(),
						retracted.clone(),
						Vec::new(),
						duration,
					);
				});
			}
		}

		self.db.read().flush().expect("DB flush failed.");
		imported
	}

	fn commit_block<B>(&self, block: B, hash: &H256, block_data: &[u8]) -> ImportRoute where B: IsBlock + Drain {
		let number = block.header().number();
		let parent = block.header().parent_hash().clone();
		let chain = self.chain.read();
		// Are we committing an era?
		let ancient = if number >= HISTORY {
			let n = number - HISTORY;
			Some((n, chain.block_hash(n).unwrap()))
		} else {
			None
		};

		// Commit results
		let receipts = block.receipts().to_owned();
		let traces = block.traces().clone().unwrap_or_else(Vec::new);
		let traces: Vec<FlatTransactionTraces> = traces.into_iter()
			.map(Into::into)
			.collect();

		//let traces = From::from(block.traces().clone().unwrap_or_else(Vec::new));

		let mut batch = DBTransaction::new(&self.db.read());
		// CHECK! I *think* this is fine, even if the state_root is equal to another
		// already-imported block of the same number.
		// TODO: Prove it with a test.
		block.drain().commit(&mut batch, number, hash, ancient).expect("DB commit failed.");

		let route = chain.insert_block(&mut batch, block_data, receipts);
		self.tracedb.read().import(&mut batch, TraceImportRequest {
			traces: traces.into(),
			block_hash: hash.clone(),
			block_number: number,
			enacted: route.enacted.clone(),
			retracted: route.retracted.len()
		});
		// Final commit to the DB
		self.db.read().write_buffered(batch);
		chain.commit();

		self.update_last_hashes(&parent, hash);
		route
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
	pub fn import_queued_transactions(&self, transactions: &[Bytes]) -> usize {
		let _timer = PerfTimer::new("import_queued_transactions");
		self.queue_transactions.fetch_sub(transactions.len(), AtomicOrdering::SeqCst);
		let txs = transactions.iter().filter_map(|bytes| UntrustedRlp::new(bytes).as_val().ok()).collect();
		let results = self.miner.import_external_transactions(self, txs);
		results.len()
	}

	/// Attempt to get a copy of a specific block's final state.
	///
	/// This will not fail if given BlockID::Latest.
	/// Otherwise, this can fail (but may not) if the DB prunes state.
	pub fn state_at(&self, id: BlockID) -> Option<State> {
		// fast path for latest state.
		match id.clone() {
			BlockID::Pending => return self.miner.pending_state().or_else(|| Some(self.state())),
			BlockID::Latest => return Some(self.state()),
			_ => {},
		}

		let block_number = match self.block_number(id.clone()) {
			Some(num) => num,
			None => return None,
		};

		self.block_header(id).and_then(|header| {
			let db = self.state_db.read().boxed_clone();

			// early exit for pruned blocks
			if db.is_pruned() && self.chain.read().best_block_number() >= block_number + HISTORY {
				return None;
			}

			let root = HeaderView::new(&header).state_root();

			State::from_existing(db, root, self.engine.account_start_nonce(), self.factories.clone()).ok()
		})
	}

	/// Attempt to get a copy of a specific block's beginning state.
	///
	/// This will not fail if given BlockID::Latest.
	/// Otherwise, this can fail (but may not) if the DB prunes state.
	pub fn state_at_beginning(&self, id: BlockID) -> Option<State> {
		// fast path for latest state.
		match id {
			BlockID::Pending => self.state_at(BlockID::Latest),
			id => match self.block_number(id) {
				None | Some(0) => None,
				Some(n) => self.state_at(BlockID::Number(n - 1)),
			}
		}
	}

	/// Get a copy of the best block's state.
	pub fn state(&self) -> State {
		State::from_existing(
			self.state_db.read().boxed_clone(),
			HeaderView::new(&self.best_block_header()).state_root(),
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
		report.state_db_mem = self.state_db.read().mem_used();
		report
	}

	/// Tick the client.
	// TODO: manage by real events.
	pub fn tick(&self) {
		self.chain.read().collect_garbage();
		self.block_queue.collect_garbage();
		self.tracedb.read().collect_garbage();

		match self.mode {
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

	/// Look up the block number for the given block ID.
	pub fn block_number(&self, id: BlockID) -> Option<BlockNumber> {
		match id {
			BlockID::Number(number) => Some(number),
			BlockID::Hash(ref hash) => self.chain.read().block_number(hash),
			BlockID::Earliest => Some(0),
			BlockID::Latest | BlockID::Pending => Some(self.chain.read().best_block_number()),
		}
	}

	/// Take a snapshot at the given block.
	/// If the ID given is "latest", this will default to 1000 blocks behind.
	pub fn take_snapshot<W: snapshot_io::SnapshotWriter + Send>(&self, writer: W, at: BlockID, p: &snapshot::Progress) -> Result<(), EthcoreError> {
		let db = self.state_db.read().boxed_clone();
		let best_block_number = self.chain_info().best_block_number;
		let block_number = try!(self.block_number(at).ok_or(snapshot::Error::InvalidStartingBlock(at)));

		if best_block_number > HISTORY + block_number && db.is_pruned() {
			return Err(snapshot::Error::OldBlockPrunedDB.into());
		}

		let start_hash = match at {
			BlockID::Latest => {
				let start_num = if best_block_number > 1000 {
					best_block_number - 1000
				} else {
					0
				};

				self.block_hash(BlockID::Number(start_num))
					.expect("blocks within HISTORY are always stored.")
			}
			_ => match self.block_hash(at) {
				Some(hash) => hash,
				None => return Err(snapshot::Error::InvalidStartingBlock(at).into()),
			},
		};

		try!(snapshot::take_snapshot(&self.chain.read(), start_hash, db.as_hashdb(), writer, p));

		Ok(())
	}

	fn block_hash(chain: &BlockChain, id: BlockID) -> Option<H256> {
		match id {
			BlockID::Hash(hash) => Some(hash),
			BlockID::Number(number) => chain.block_hash(number),
			BlockID::Earliest => chain.block_hash(0),
			BlockID::Latest | BlockID::Pending => Some(chain.best_block_hash()),
		}
	}

	fn transaction_address(&self, id: TransactionID) -> Option<TransactionAddress> {
		match id {
			TransactionID::Hash(ref hash) => self.chain.read().transaction_address(hash),
			TransactionID::Location(id, index) => Self::block_hash(&self.chain.read(), id).map(|hash| TransactionAddress {
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
		let mut state_db = self.state_db.write();
		let mut chain = self.chain.write();
		let mut tracedb = self.tracedb.write();
		self.miner.clear();
		let db = self.db.write();
		try!(db.restore(new_db));

		*state_db = journaldb::new(db.clone(), self.pruning, ::db::COL_STATE);
		*chain = Arc::new(BlockChain::new(self.config.blockchain.clone(), &[], db.clone()));
		*tracedb = try!(TraceDB::new(self.config.tracing.clone(), db.clone(), chain.clone()).map_err(ClientError::from));
		Ok(())
	}
}


impl BlockChainClient for Client {
	fn call(&self, t: &SignedTransaction, block: BlockID, analytics: CallAnalytics) -> Result<Executed, CallError> {
		let header = try!(self.block_header(block).ok_or(CallError::StatePruned));
		let view = HeaderView::new(&header);
		let last_hashes = self.build_last_hashes(view.hash());
		let env_info = EnvInfo {
			number: view.number(),
			author: view.author(),
			timestamp: view.timestamp(),
			difficulty: view.difficulty(),
			last_hashes: last_hashes,
			gas_used: U256::zero(),
			gas_limit: U256::max_value(),
		};
		// that's just a copy of the state.
		let mut state = try!(self.state_at(block).ok_or(CallError::StatePruned));
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
		let mut ret = try!(Executive::new(&mut state, &env_info, &*self.engine, &self.factories.vm).transact(t, options));

		// TODO gav move this into Executive.
		ret.state_diff = original_state.map(|original| state.diff_from(original));

		Ok(ret)
	}

	fn replay(&self, id: TransactionID, analytics: CallAnalytics) -> Result<Executed, CallError> {
		let address = try!(self.transaction_address(id).ok_or(CallError::TransactionNotFound));
		let header_data = try!(self.block_header(BlockID::Hash(address.block_hash)).ok_or(CallError::StatePruned));
		let body_data = try!(self.block_body(BlockID::Hash(address.block_hash)).ok_or(CallError::StatePruned));
		let mut state = try!(self.state_at_beginning(BlockID::Hash(address.block_hash)).ok_or(CallError::StatePruned));
		let txs = BodyView::new(&body_data).transactions();

		if address.index >= txs.len() {
			return Err(CallError::TransactionNotFound);
		}

		let options = TransactOptions { tracing: analytics.transaction_tracing, vm_tracing: analytics.vm_tracing, check_nonce: false };
		let view = HeaderView::new(&header_data);
		let last_hashes = self.build_last_hashes(view.hash());
		let mut env_info = EnvInfo {
			number: view.number(),
			author: view.author(),
			timestamp: view.timestamp(),
			difficulty: view.difficulty(),
			last_hashes: last_hashes,
			gas_used: U256::default(),
			gas_limit: view.gas_limit(),
		};
		for t in txs.iter().take(address.index) {
			match Executive::new(&mut state, &env_info, &*self.engine, &self.factories.vm).transact(t, Default::default()) {
				Ok(x) => { env_info.gas_used = env_info.gas_used + x.gas_used; }
				Err(ee) => { return Err(CallError::Execution(ee)) }
			}
		}
		let t = &txs[address.index];

		let original_state = if analytics.state_diffing { Some(state.clone()) } else { None };
		let mut ret = try!(Executive::new(&mut state, &env_info, &*self.engine, &self.factories.vm).transact(t, options));
		ret.state_diff = original_state.map(|original| state.diff_from(original));

		Ok(ret)
	}

	fn keep_alive(&self) {
		if self.mode != Mode::Active {
			self.wake_up();
			(*self.sleep_state.lock()).last_activity = Some(Instant::now());
		}
	}

	fn best_block_header(&self) -> Bytes {
		self.chain.read().best_block_header()
	}

	fn block_header(&self, id: BlockID) -> Option<Bytes> {
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| chain.block_header_data(&hash))
	}

	fn block_body(&self, id: BlockID) -> Option<Bytes> {
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| chain.block_body(&hash))
	}

	fn block(&self, id: BlockID) -> Option<Bytes> {
		if let BlockID::Pending = id {
			if let Some(block) = self.miner.pending_block() {
				return Some(block.rlp_bytes(Seal::Without));
			}
		}
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| {
			chain.block(&hash)
		})
	}

	fn block_status(&self, id: BlockID) -> BlockStatus {
		let chain = self.chain.read();
		match Self::block_hash(&chain, id) {
			Some(ref hash) if chain.is_known(hash) => BlockStatus::InChain,
			Some(hash) => self.block_queue.block_status(&hash),
			None => BlockStatus::Unknown
		}
	}

	fn block_total_difficulty(&self, id: BlockID) -> Option<U256> {
		if let BlockID::Pending = id {
			if let Some(block) = self.miner.pending_block() {
				return Some(*block.header.difficulty() + self.block_total_difficulty(BlockID::Latest).expect("blocks in chain have details; qed"));
			}
		}
		let chain = self.chain.read();
		Self::block_hash(&chain, id).and_then(|hash| chain.block_details(&hash)).map(|d| d.total_difficulty)
	}

	fn nonce(&self, address: &Address, id: BlockID) -> Option<U256> {
		self.state_at(id).map(|s| s.nonce(address))
	}

	fn block_hash(&self, id: BlockID) -> Option<H256> {
		let chain = self.chain.read();
		Self::block_hash(&chain, id)
	}

	fn code(&self, address: &Address, id: BlockID) -> Option<Option<Bytes>> {
		self.state_at(id).map(|s| s.code(address))
	}

	fn balance(&self, address: &Address, id: BlockID) -> Option<U256> {
		self.state_at(id).map(|s| s.balance(address))
	}

	fn storage_at(&self, address: &Address, position: &H256, id: BlockID) -> Option<H256> {
		self.state_at(id).map(|s| s.storage_at(address, position))
	}

	fn transaction(&self, id: TransactionID) -> Option<LocalizedTransaction> {
		self.transaction_address(id).and_then(|address| self.chain.read().transaction(&address))
	}

	fn uncle(&self, id: UncleID) -> Option<Bytes> {
		let index = id.position;
		self.block_body(id.block).and_then(|body| BodyView::new(&body).uncle_rlp_at(index))
	}

	fn transaction_receipt(&self, id: TransactionID) -> Option<LocalizedReceipt> {
		let chain = self.chain.read();
		self.transaction_address(id).and_then(|address| chain.block_number(&address.block_hash).and_then(|block_number| {
			let t = chain.block_body(&address.block_hash)
				.and_then(|block| BodyView::new(&block).localized_transaction_at(&address.block_hash, block_number, address.index));

			match (t, chain.transaction_receipt(&address)) {
				(Some(tx), Some(receipt)) => {
					let block_hash = tx.block_hash.clone();
					let block_number = tx.block_number.clone();
					let transaction_hash = tx.hash();
					let transaction_index = tx.transaction_index;
					let prior_gas_used = match tx.transaction_index {
						0 => U256::zero(),
						i => {
							let prior_address = TransactionAddress { block_hash: address.block_hash, index: i - 1 };
							let prior_receipt = chain.transaction_receipt(&prior_address).expect("Transaction receipt at `address` exists; `prior_address` has lower index in same block; qed");
							prior_receipt.gas_used
						}
					};
					Some(LocalizedReceipt {
						transaction_hash: tx.hash(),
						transaction_index: tx.transaction_index,
						block_hash: tx.block_hash,
						block_number: tx.block_number,
						cumulative_gas_used: receipt.gas_used,
						gas_used: receipt.gas_used - prior_gas_used,
						contract_address: match tx.action {
							Action::Call(_) => None,
							Action::Create => Some(contract_address(&tx.sender().unwrap(), &tx.nonce))
						},
						logs: receipt.logs.into_iter().enumerate().map(|(i, log)| LocalizedLogEntry {
							entry: log,
							block_hash: block_hash.clone(),
							block_number: block_number,
							transaction_hash: transaction_hash.clone(),
							transaction_index: transaction_index,
							log_index: i
						}).collect()
					})
				},
				_ => None
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
		self.state_db.read().state(hash)
	}

	fn block_receipts(&self, hash: &H256) -> Option<Bytes> {
		self.chain.read().block_receipts(hash).map(|receipts| ::rlp::encode(&receipts).to_vec())
	}

	fn import_block(&self, bytes: Bytes) -> Result<H256, BlockImportError> {
		{
			let header = BlockView::new(&bytes).header_view();
			if self.chain.read().is_known(&header.sha3()) {
				return Err(BlockImportError::Import(ImportError::AlreadyInChain));
			}
			if self.block_status(BlockID::Hash(header.parent_hash())) == BlockStatus::Unknown {
				return Err(BlockImportError::Block(BlockError::UnknownParent(header.parent_hash())));
			}
		}
		Ok(try!(self.block_queue.import_block(bytes)))
	}

	fn queue_info(&self) -> BlockQueueInfo {
		self.block_queue.queue_info()
	}

	fn clear_queue(&self) {
		self.block_queue.clear();
	}

	fn chain_info(&self) -> BlockChainInfo {
		let chain = self.chain.read();
		BlockChainInfo {
			total_difficulty: chain.best_block_total_difficulty(),
			pending_total_difficulty: chain.best_block_total_difficulty(),
			genesis_hash: chain.genesis_hash(),
			best_block_hash: chain.best_block_hash(),
			best_block_number: From::from(chain.best_block_number())
		}
	}

	fn additional_params(&self) -> BTreeMap<String, String> {
		self.engine.additional_params().into_iter().collect()
	}

	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockID, to_block: BlockID) -> Option<Vec<BlockNumber>> {
		match (self.block_number(from_block), self.block_number(to_block)) {
			(Some(from), Some(to)) => Some(self.chain.read().blocks_with_bloom(bloom, from, to)),
			_ => None
		}
	}

	fn logs(&self, filter: Filter, limit: Option<usize>) -> Vec<LocalizedLogEntry> {
		let blocks = filter.bloom_possibilities().iter()
			.filter_map(|bloom| self.blocks_with_bloom(bloom, filter.from_block.clone(), filter.to_block.clone()))
			.flat_map(|m| m)
			// remove duplicate elements
			.collect::<HashSet<u64>>()
			.into_iter()
			.collect::<Vec<u64>>();

		self.chain.read().logs(blocks, |entry| filter.matches(entry), limit)
	}

	fn filter_traces(&self, filter: TraceFilter) -> Option<Vec<LocalizedTrace>> {
		let start = self.block_number(filter.range.start);
		let end = self.block_number(filter.range.end);

		if start.is_some() && end.is_some() {
			let filter = trace::Filter {
				range: start.unwrap() as usize..end.unwrap() as usize,
				from_address: From::from(filter.from_address),
				to_address: From::from(filter.to_address),
			};

			let traces = self.tracedb.read().filter(&filter);
			Some(traces)
		} else {
			None
		}
	}

	fn trace(&self, trace: TraceId) -> Option<LocalizedTrace> {
		let trace_address = trace.address;
		self.transaction_address(trace.transaction)
			.and_then(|tx_address| {
				self.block_number(BlockID::Hash(tx_address.block_hash))
					.and_then(|number| self.tracedb.read().trace(number, tx_address.index, trace_address))
			})
	}

	fn transaction_traces(&self, transaction: TransactionID) -> Option<Vec<LocalizedTrace>> {
		self.transaction_address(transaction)
			.and_then(|tx_address| {
				self.block_number(BlockID::Hash(tx_address.block_hash))
					.and_then(|number| self.tracedb.read().transaction_traces(number, tx_address.index))
			})
	}

	fn block_traces(&self, block: BlockID) -> Option<Vec<LocalizedTrace>> {
		self.block_number(block)
			.and_then(|number| self.tracedb.read().block_traces(number))
	}

	fn last_hashes(&self) -> LastHashes {
		(*self.build_last_hashes(self.chain.read().best_block_hash())).clone()
	}

	fn queue_transactions(&self, transactions: Vec<Bytes>) {
		if self.queue_transactions.load(AtomicOrdering::Relaxed) > MAX_TX_QUEUE_SIZE {
			debug!("Ignoring {} transactions: queue is full", transactions.len());
		} else {
			let len = transactions.len();
			match self.io_channel.send(ClientIoMessage::NewTransactions(transactions)) {
				Ok(_) => {
					self.queue_transactions.fetch_add(len, AtomicOrdering::SeqCst);
				}
				Err(e) => {
					debug!("Ignoring {} transactions: error queueing: {}", len, e);
				}
			}
		}
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		self.miner.pending_transactions()
	}
}

impl MiningBlockChainClient for Client {
	fn prepare_open_block(&self, author: Address, gas_range_target: (U256, U256), extra_data: Bytes) -> OpenBlock {
		let engine = &*self.engine;
		let chain = self.chain.read();
		let h = chain.best_block_hash();

		let mut open_block = OpenBlock::new(
			engine,
			self.factories.clone(),
			false,	// TODO: this will need to be parameterised once we want to do immediate mining insertion.
			self.state_db.read().boxed_clone(),
			&chain.block_header(&h).expect("h is best block hash: so its header must exist: qed"),
			self.build_last_hashes(h.clone()),
			author,
			gas_range_target,
			extra_data,
		).expect("OpenBlock::new only fails if parent state root invalid; state root of best block's header is never invalid; qed");

		// Add uncles
		chain
			.find_uncle_headers(&h, engine.maximum_uncle_age())
			.unwrap()
			.into_iter()
			.take(engine.maximum_uncle_count())
			.foreach(|h| {
				open_block.push_uncle(h).unwrap();
			});

		open_block
	}

	fn vm_factory(&self) -> &EvmFactory {
		&self.factories.vm
	}

	fn import_sealed_block(&self, block: SealedBlock) -> ImportResult {
		let _import_lock = self.import_lock.lock();
		let _timer = PerfTimer::new("import_sealed_block");
		let start = precise_time_ns();

		let h = block.header().hash();
		let number = block.header().number();

		let block_data = block.rlp_bytes();
		let route = self.commit_block(block, &h, &block_data);
		trace!(target: "client", "Imported sealed block #{} ({})", number, h);

		let (enacted, retracted) = self.calculate_enacted_retracted(&[route]);
		self.miner.chain_new_blocks(self, &[h.clone()], &[], &enacted, &retracted);

		self.notify(|notify| {
			notify.new_blocks(
				vec![h.clone()],
				vec![],
				enacted.clone(),
				retracted.clone(),
				vec![h.clone()],
				precise_time_ns() - start,
			);
		});
		self.db.read().flush().expect("DB flush failed.");
		Ok(h)
	}
}

impl MayPanic for Client {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}
