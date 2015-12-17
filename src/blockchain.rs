use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::hash::Hash;
use rocksdb::{DB, WriteBatch, Writable};
use heapsize::HeapSizeOf;
use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::hashdb::*;
use util::sha3::*;
use util::bytes::*;
use util::squeeze::*;
use blockheader::*;
use block::*;
use genesis::*;
use extras::*;
use transaction::*;

#[derive(Debug)]
pub struct CacheSize {
	pub blocks: usize,
	pub block_details: usize,
	pub transaction_addresses: usize,
	pub block_logs: usize,
	pub blocks_blooms: usize
}

pub struct BlockChain {
	last_block_number: Cell<U256>,

	// block cache
	blocks: RefCell<HashMap<H256, Bytes>>,

	// extra caches
	block_details: RefCell<HashMap<H256, BlockDetails>>,
	block_hashes: RefCell<HashMap<U256, H256>>,
	transaction_addresses: RefCell<HashMap<H256, TransactionAddress>>,
	block_logs: RefCell<HashMap<H256, BlockLogBlooms>>,
	blocks_blooms: RefCell<HashMap<H256, BlocksBlooms>>,

	extras_db: DB,
	blocks_db: DB
}

impl BlockChain {
	/// Create new instance of blockchain from given Genesis
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// extern crate ethcore;
	/// use std::env;
	/// use std::str::FromStr;
	/// use ethcore::genesis::*;
	/// use ethcore::blockchain::*;
	/// use util::hash::*;
	/// use util::uint::*;
	/// 
	/// fn main() {
	/// 	let mut dir = env::temp_dir();
	/// 	dir.push(H32::random().hex());
	///
	/// 	let genesis = Genesis::new_frontier();
	/// 	let bc = BlockChain::new(genesis, &dir);
	/// 	let genesis_hash = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
	/// 	assert_eq!(bc.genesis_hash(), H256::from_str(genesis_hash).unwrap());
	/// 	assert!(bc.is_known(&bc.genesis_hash()));
	/// 	assert_eq!(bc.genesis_hash(), bc.block_hash(&U256::from(0u8)).unwrap());
	/// }
	/// ```
	pub fn new(genesis: Genesis, path: &Path) -> BlockChain {
		let (genesis_block, _genesis_state) = genesis.drain();

		// open dbs
		let mut extras_path = path.to_path_buf();
		extras_path.push("extras");
		let extras_db = DB::open_default(extras_path.to_str().unwrap()).unwrap();

		let mut blocks_path = path.to_path_buf();
		blocks_path.push("blocks");
		let blocks_db = DB::open_default(blocks_path.to_str().unwrap()).unwrap();

		let bc = BlockChain {
			last_block_number: Cell::new(U256::from(0u8)),
			blocks: RefCell::new(HashMap::new()),
			block_details: RefCell::new(HashMap::new()),
			block_hashes: RefCell::new(HashMap::new()),
			transaction_addresses: RefCell::new(HashMap::new()),
			block_logs: RefCell::new(HashMap::new()),
			blocks_blooms: RefCell::new(HashMap::new()),
			extras_db: extras_db,
			blocks_db: blocks_db
		};

		bc.insert_block(&genesis_block);
		bc
	}

	/// Inserts the block into backing cache database.
	/// Expects the block to be valid and already verified.
	/// If the block is already known, does nothing.
	pub fn insert_block(&self, bytes: &[u8]) {
		let block = BlockView::new(bytes);
		let header = block.header_view();

		if self.is_known(&header.sha3()) {
			return;
		}

		let hash = block.sha3();
		
		self.blocks_db.put(&hash, &bytes).unwrap();
		
		let batch = WriteBatch::new();
		batch.put_extras(&hash, &block.block_details());
		batch.put_extras(&header.number(), &hash);
		self.extras_db.write(batch).unwrap();
	}

	/// Returns true if the given block is known 
	/// (though not necessarily a part of the canon chain).
	pub fn is_known(&self, hash: &H256) -> bool {
		// TODO: consider taking into account current block
		self.query_extras_exist(hash, &self.block_details)
	}

	/// Returns true if transaction is known.
	pub fn is_known_transaction(&self, hash: &H256) -> bool {
		self.query_extras_exist(hash, &self.transaction_addresses)
	}

	/// Returns reference to genesis hash
	pub fn genesis_hash(&self) -> H256 {
		self.block_hash(&U256::from(0u8)).expect("Genesis hash should always exist")
	}

	/// Get the partial-header of a block
	pub fn block_header(&self, hash: &H256) -> Option<Header> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).header())
	}

	/// Get a list of transactions for a given block.
	/// Returns None is block deos not exist.
	pub fn transactions(&self, hash: &H256) -> Option<Vec<Transaction>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).transactions())
	}

	/// Get a list of transaction hashes for a given block.
	/// Returns None if block does not exist.
	pub fn transaction_hashes(&self, hash: &H256) -> Option<Vec<H256>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).transaction_hashes())
	}

	/// Get a list of uncles for a given block.
	/// Returns None is block deos not exist.
	pub fn uncles(&self, hash: &H256) -> Option<Vec<Header>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).uncles())
	}

	/// Get a list of uncle hashes for a given block.
	/// Returns None if block does not exist.
	pub fn uncle_hashes(&self, hash: &H256) -> Option<Vec<H256>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).uncle_hashes())
	}

	/// Get the familial details concerning a block.
	pub fn block_details(&self, hash: &H256) -> Option<BlockDetails> {
		self.query_extras(hash, &self.block_details)
	}

	/// Get the hash of given block's number
	pub fn block_hash(&self, hash: &U256) -> Option<H256> {
		self.query_extras(hash, &self.block_hashes)
	}

	/// Get last block number
	pub fn last_block_number(&self) -> U256 {
		self.last_block_number.get()
	}

	/// Get the number of given block's hash
	pub fn block_number(&self, hash: &H256) -> Option<U256> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).header_view().number())
	}

	/// Get the transactions' log blooms of a block
	pub fn log_blooms(&self, hash: &H256) -> Option<BlockLogBlooms> {
		self.query_extras(hash, &self.block_logs)
	}

	fn block(&self, hash: &H256) -> Option<Bytes> {
		{
			let read = self.blocks.borrow();
			match read.get(hash) {
				Some(v) => return Some(v.clone()),
				None => ()
			}
		}

		let opt = self.blocks_db.get(hash)
			.expect("Low level database error. Some issue with disk?");

		match opt {
			Some(b) => {
				let bytes: Bytes = b.to_vec();
				let mut write = self.blocks.borrow_mut();
				write.insert(hash.clone(), bytes.clone());
				Some(bytes)
			},
			None => None
		}
	}

	fn query_extras<K, T>(&self, hash: &K, cache: &RefCell<HashMap<K, T>>) -> Option<T> where 
		T: Clone + Decodable + ExtrasIndexable, 
		K: ExtrasSliceConvertable + Eq + Hash + Clone {
		{
			let read = cache.borrow();
			match read.get(hash) {
				Some(v) => return Some(v.clone()),
				None => ()
			}
		}

		self.extras_db.get_extras(hash).map(| t: T | {
			let mut write = cache.borrow_mut();
			write.insert(hash.clone(), t.clone());
			t
		})
	}

	fn query_extras_exist<K, T>(&self, hash: &K, cache: &RefCell<HashMap<K, T>>) -> bool where 
		K: ExtrasSliceConvertable + Eq + Hash + Clone,
		T: ExtrasIndexable {
		{
			let read = cache.borrow();
			match read.get(hash) {
				Some(_) => return true,
				None => ()
			}
		}

		self.extras_db.extras_exists::<_, T>(hash)
	}

	/// Get current cache size
	pub fn cache_size(&self) -> CacheSize {
		CacheSize {
			blocks: self.blocks.heap_size_of_children(),
			block_details: self.block_details.heap_size_of_children(),
			transaction_addresses: self.transaction_addresses.heap_size_of_children(),
			block_logs: self.block_logs.heap_size_of_children(),
			blocks_blooms: self.blocks_blooms.heap_size_of_children()
		}
	}

	/// Squeeze the cache if its too big
	pub fn squeeze_to_fit(&self, size: CacheSize) {
		self.blocks.borrow_mut().squeeze(size.blocks);
		self.block_details.borrow_mut().squeeze(size.block_details);
		self.transaction_addresses.borrow_mut().squeeze(size.transaction_addresses);
		self.block_logs.borrow_mut().squeeze(size.block_logs);
		self.blocks_blooms.borrow_mut().squeeze(size.blocks_blooms);
	}
}

