use util::*;
use rocksdb::{Options, DB};
use blockchain::{BlockChain, BlockProvider, CacheSize};
use views::BlockView;
use error::*;
use header::BlockNumber;
use state::State;
use spec::Spec;
use engine::Engine;
use views::HeaderView;
use block_queue::{BlockQueue, BlockQueueInfo};
use service::NetSyncMessage;
use env_info::LastHashes;
use verification::*;
use block::*;

/// General block status
#[derive(Debug)]
pub enum BlockStatus {
	/// Part of the blockchain.
	InChain,
	/// Queued for import.
	Queued,
	/// Known as bad.
	Bad,
	/// Unknown.
	Unknown,
}

/// Information about the blockchain gthered together.
#[derive(Debug)]
pub struct BlockChainInfo {
	/// Blockchain difficulty.
	pub total_difficulty: U256,
	/// Block queue difficulty.
	pub pending_total_difficulty: U256,
	/// Genesis block hash.
	pub genesis_hash: H256,
	/// Best blockchain block hash.
	pub best_block_hash: H256,
	/// Best blockchain block number.
	pub best_block_number: BlockNumber
}

impl fmt::Display for BlockChainInfo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "#{}.{}", self.best_block_number, self.best_block_hash)
	}
}

/// TODO [arkpar] Please document me
pub type TreeRoute = ::blockchain::TreeRoute;

/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient : Sync + Send {
	/// Get raw block header data by block header hash.
	fn block_header(&self, hash: &H256) -> Option<Bytes>;

	/// Get raw block body data by block header hash.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body(&self, hash: &H256) -> Option<Bytes>;

	/// Get raw block data by block header hash.
	fn block(&self, hash: &H256) -> Option<Bytes>;

	/// Get block status by block header hash.
	fn block_status(&self, hash: &H256) -> BlockStatus;

	/// Get raw block header data by block number.
	fn block_header_at(&self, n: BlockNumber) -> Option<Bytes>;

	/// Get raw block body data by block number.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body_at(&self, n: BlockNumber) -> Option<Bytes>;

	/// Get raw block data by block number.
	fn block_at(&self, n: BlockNumber) -> Option<Bytes>;

	/// Get block status by block number.
	fn block_status_at(&self, n: BlockNumber) -> BlockStatus;

	/// Get a tree route between `from` and `to`.
	/// See `BlockChain::tree_route`.
	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute>;

	/// Get latest state node
	fn state_data(&self, hash: &H256) -> Option<Bytes>;

	/// Get raw block receipts data by block header hash.
	fn block_receipts(&self, hash: &H256) -> Option<Bytes>;

	/// Import a block into the blockchain.
	fn import_block(&self, bytes: Bytes) -> ImportResult;

	/// Get block queue information.
	fn queue_info(&self) -> BlockQueueInfo;

	/// Clear block queue and abort all import activity.
	fn clear_queue(&self);

	/// Get blockchain information.
	fn chain_info(&self) -> BlockChainInfo;

	/// Get the best block header.
	fn best_block_header(&self) -> Bytes {
		self.block_header(&self.chain_info().best_block_hash).unwrap()
	}
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
/// TODO [Gav Wood] Please document me
pub struct ClientReport {
	/// TODO [Gav Wood] Please document me
	pub blocks_imported: usize,
	/// TODO [Gav Wood] Please document me
	pub transactions_applied: usize,
	/// TODO [Gav Wood] Please document me
	pub gas_processed: U256,
}

impl ClientReport {
	/// TODO [Gav Wood] Please document me
	pub fn accrue_block(&mut self, block: &PreVerifiedBlock) {
		self.blocks_imported += 1;
		self.transactions_applied += block.transactions.len();
		self.gas_processed += block.header.gas_used;
	}
}

/// Blockchain database client backed by a persistent database. Owns and manages a blockchain and a block queue.
/// Call `import_block()` to import a block asynchronously; `flush_queue()` flushes the queue.
pub struct Client {
	chain: Arc<RwLock<BlockChain>>,
	engine: Arc<Box<Engine>>,
	state_db: JournalDB,
	block_queue: RwLock<BlockQueue>,
	report: RwLock<ClientReport>,
	uncommited_states: RwLock<HashMap<H256, JournalDB>>,
	import_lock: Mutex<()>
}

const HISTORY: u64 = 1000;

impl Client {
	/// Create a new client with given spec and DB path.
	pub fn new(spec: Spec, path: &Path, message_channel: IoChannel<NetSyncMessage> ) -> Result<Arc<Client>, Error> {
		let gb = spec.genesis_block();
		let chain = Arc::new(RwLock::new(BlockChain::new(&gb, path)));
		let mut opts = Options::new();
		opts.set_max_open_files(256);
		opts.create_if_missing(true);
		opts.set_use_fsync(false);
		/*
		opts.set_bytes_per_sync(8388608);
		opts.set_disable_data_sync(false);
		opts.set_block_cache_size_mb(1024);
		opts.set_table_cache_num_shard_bits(6);
		opts.set_max_write_buffer_number(32);
		opts.set_write_buffer_size(536870912);
		opts.set_target_file_size_base(1073741824);
		opts.set_min_write_buffer_number_to_merge(4);
		opts.set_level_zero_stop_writes_trigger(2000);
		opts.set_level_zero_slowdown_writes_trigger(0);
		opts.set_compaction_style(DBUniversalCompaction);
		opts.set_max_background_compactions(4);
		opts.set_max_background_flushes(4);
		opts.set_filter_deletes(false);
		opts.set_disable_auto_compactions(false);*/

		let mut state_path = path.to_path_buf();
		state_path.push("state");
		let db = Arc::new(DB::open(&opts, state_path.to_str().unwrap()).unwrap());
		
		let engine = Arc::new(try!(spec.to_engine()));
		let mut state_db = JournalDB::new_with_arc(db.clone());
		if engine.spec().ensure_db_good(&mut state_db) {
			state_db.commit(0, &engine.spec().genesis_header().hash(), None).expect("Error commiting genesis state to state DB");
		}
		Ok(Arc::new(Client {
			chain: chain,
			engine: engine.clone(),
			state_db: state_db,
			block_queue: RwLock::new(BlockQueue::new(engine, message_channel)),
			report: RwLock::new(Default::default()),
			uncommited_states: RwLock::new(HashMap::new()),
			import_lock: Mutex::new(()),
		}))
	}

	/// Flush the block import queue.
	pub fn flush_queue(&self) {
		self.block_queue.write().unwrap().flush();
	}

	/// This is triggered by a message coming from a block queue when the block is ready for insertion
	pub fn import_verified_blocks(&self, _io: &IoChannel<NetSyncMessage>) -> usize {
		let mut ret = 0;
		let mut bad = HashSet::new();
		let _import_lock = self.import_lock.lock();
		let blocks = self.block_queue.write().unwrap().drain(128);
		for block in blocks {
//			flushln!("Importing {}...", block.header.hash());
			if bad.contains(&block.header.parent_hash) {
				self.block_queue.write().unwrap().mark_as_bad(&block.header.hash());
				bad.insert(block.header.hash());
				continue;
			}

			let header = &block.header;
			if let Err(e) = verify_block_family(&header, &block.bytes, self.engine.deref().deref(), self.chain.read().unwrap().deref()) {
				flushln!("Stage 3 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
				warn!(target: "client", "Stage 3 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
				self.block_queue.write().unwrap().mark_as_bad(&header.hash());
				bad.insert(block.header.hash());
				break;
			};
			let parent = match self.chain.read().unwrap().block_header(&header.parent_hash) {
				Some(p) => p,
				None => {
					flushln!("Block import failed for #{} ({}): Parent not found ({}) ", header.number(), header.hash(), header.parent_hash);
					warn!(target: "client", "Block import failed for #{} ({}): Parent not found ({}) ", header.number(), header.hash(), header.parent_hash);
					self.block_queue.write().unwrap().mark_as_bad(&header.hash());
					bad.insert(block.header.hash());
					break;
				},
			};
			// build last hashes
			let mut last_hashes = LastHashes::new();
			last_hashes.resize(256, H256::new());
			last_hashes[0] = header.parent_hash.clone();
			for i in 0..255 {
				match self.chain.read().unwrap().block_details(&last_hashes[i]) {
					Some(details) => {
						last_hashes[i + 1] = details.parent.clone();
					},
					None => break,
				}
			}

			let db = self.state_db.clone();
			let result = match enact_verified(&block, self.engine.deref().deref(), db, &parent, &last_hashes) {
				Ok(b) => b,
				Err(e) => {
					flushln!("Block import failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
					warn!(target: "client", "Block import failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
					bad.insert(block.header.hash());
					self.block_queue.write().unwrap().mark_as_bad(&header.hash());
					break;
				}
			};
			if let Err(e) = verify_block_final(&header, result.block().header()) {
				flushln!("Stage 4 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
				warn!(target: "client", "Stage 4 block verification failed for #{} ({})\nError: {:?}", header.number(), header.hash(), e);
				self.block_queue.write().unwrap().mark_as_bad(&header.hash());
				break;
			}

			self.chain.write().unwrap().insert_block(&block.bytes); //TODO: err here?
			let ancient = if header.number() >= HISTORY { Some(header.number() - HISTORY) } else { None };
			match result.drain().commit(header.number(), &header.hash(), ancient.map(|n|(n, self.chain.read().unwrap().block_hash(n).unwrap()))) {
				Ok(_) => (),
				Err(e) => {
					warn!(target: "client", "State DB commit failed: {:?}", e);
					break;
				}
			}
			self.report.write().unwrap().accrue_block(&block);
			trace!(target: "client", "Imported #{} ({})", header.number(), header.hash());
			ret += 1;
		}
		ret
	}

	/// Clear cached state overlay 
	pub fn clear_state(&self, hash: &H256) {
		self.uncommited_states.write().unwrap().remove(hash);
	}

	/// Get a copy of the best block's state.
	pub fn state(&self) -> State {
		State::from_existing(self.state_db.clone(), HeaderView::new(&self.best_block_header()).state_root(), self.engine.account_start_nonce())
	}

	/// Get info on the cache.
	pub fn cache_info(&self) -> CacheSize {
		self.chain.read().unwrap().cache_size()
	}

	/// Get the report.
	pub fn report(&self) -> ClientReport {
		self.report.read().unwrap().clone()
	}

	/// Tick the client.
	pub fn tick(&self) {
		self.chain.read().unwrap().collect_garbage(false);
	}
}

impl BlockChainClient for Client {
	fn block_header(&self, hash: &H256) -> Option<Bytes> {
		self.chain.read().unwrap().block(hash).map(|bytes| BlockView::new(&bytes).rlp().at(0).as_raw().to_vec())
	}

	fn block_body(&self, hash: &H256) -> Option<Bytes> {
		self.chain.read().unwrap().block(hash).map(|bytes| {
			let rlp = Rlp::new(&bytes);
			let mut body = RlpStream::new();
			body.append_raw(rlp.at(1).as_raw(), 1);
			body.append_raw(rlp.at(2).as_raw(), 1);
			body.out()
		})
	}

	fn block(&self, hash: &H256) -> Option<Bytes> {
		self.chain.read().unwrap().block(hash)
	}

	fn block_status(&self, hash: &H256) -> BlockStatus {
		if self.chain.read().unwrap().is_known(&hash) { BlockStatus::InChain } else { BlockStatus::Unknown }
	}

	fn block_header_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.chain.read().unwrap().block_hash(n).and_then(|h| self.block_header(&h))
	}

	fn block_body_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.chain.read().unwrap().block_hash(n).and_then(|h| self.block_body(&h))
	}

	fn block_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.chain.read().unwrap().block_hash(n).and_then(|h| self.block(&h))
	}

	fn block_status_at(&self, n: BlockNumber) -> BlockStatus {
		match self.chain.read().unwrap().block_hash(n) {
			Some(h) => self.block_status(&h),
			None => BlockStatus::Unknown
		}
	}

	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute> {
		self.chain.read().unwrap().tree_route(from.clone(), to.clone())
	}

	fn state_data(&self, _hash: &H256) -> Option<Bytes> {
		unimplemented!();
	}

	fn block_receipts(&self, _hash: &H256) -> Option<Bytes> {
		unimplemented!();
	}

	fn import_block(&self, bytes: Bytes) -> ImportResult {
		let header = BlockView::new(&bytes).header();
		if self.chain.read().unwrap().is_known(&header.hash()) {
			return Err(ImportError::AlreadyInChain);
		}
		self.block_queue.write().unwrap().import_block(bytes)
	}

	fn queue_info(&self) -> BlockQueueInfo {
		self.block_queue.read().unwrap().queue_info()
	}

	fn clear_queue(&self) {
		self.block_queue.write().unwrap().clear();
	}

	fn chain_info(&self) -> BlockChainInfo {
		let chain = self.chain.read().unwrap();
		BlockChainInfo {
			total_difficulty: chain.best_block_total_difficulty(),
			pending_total_difficulty: chain.best_block_total_difficulty(),
			genesis_hash: chain.genesis_hash(),
			best_block_hash: chain.best_block_hash(),
			best_block_number: From::from(chain.best_block_number())
		}
	}
}
