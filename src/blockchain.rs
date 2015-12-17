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
use genesis::*;
use extras::*;
use transaction::*;
use views::*;

/// blocks - a vector of hashes of all blocks, ordered from `from` block to `to` block.
/// ancestor - best common ancestor of these blocks
/// index - an index where best common ancestor would be
pub struct TreeRoute {
	pub blocks: Vec<H256>,
	pub ancestor: H256,
	pub index: usize
}

#[derive(Debug)]
pub struct CacheSize {
	pub blocks: usize,
	pub block_details: usize,
	pub transaction_addresses: usize,
	pub block_logs: usize,
	pub blocks_blooms: usize
}

pub struct BlockChain {
	best_block_hash: Cell<H256>,
	best_block_number: Cell<U256>,
	best_block_total_difficulty: Cell<U256>,

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
	/// 	let genesis = Genesis::new_frontier();
	///
	/// 	let mut dir = env::temp_dir();
	/// 	dir.push(H32::random().hex());
	///
	/// 	let bc = BlockChain::new(&genesis, &dir);
	///
	/// 	let genesis_hash = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
	/// 	assert_eq!(bc.genesis_hash(), H256::from_str(genesis_hash).unwrap());
	/// 	assert!(bc.is_known(&bc.genesis_hash()));
	/// 	assert_eq!(bc.genesis_hash(), bc.block_hash(&U256::from(0u8)).unwrap());
	/// }
	/// ```
	pub fn new(genesis: &Genesis, path: &Path) -> BlockChain {
		// open extras db
		let mut extras_path = path.to_path_buf();
		extras_path.push("extras");
		let extras_db = DB::open_default(extras_path.to_str().unwrap()).unwrap();

		// open blocks db
		let mut blocks_path = path.to_path_buf();
		blocks_path.push("blocks");
		let blocks_db = DB::open_default(blocks_path.to_str().unwrap()).unwrap();

		let bc = BlockChain {
			best_block_hash: Cell::new(H256::new()),
			best_block_number: Cell::new(U256::from(0u8)),
			best_block_total_difficulty: Cell::new(U256::from(0u8)),
			blocks: RefCell::new(HashMap::new()),
			block_details: RefCell::new(HashMap::new()),
			block_hashes: RefCell::new(HashMap::new()),
			transaction_addresses: RefCell::new(HashMap::new()),
			block_logs: RefCell::new(HashMap::new()),
			blocks_blooms: RefCell::new(HashMap::new()),
			extras_db: extras_db,
			blocks_db: blocks_db
		};

		// load best block
		let best_block_hash = match bc.extras_db.get(b"best").unwrap() {
			Some(best) => H256::from_slice(&best),
			None => {
				// best block does not exist
				// we need to insert genesis into the cache
				let bytes = genesis.block();
				let block = BlockView::new(bytes);
				let header = block.header_view();
				let hash = block.sha3();

				let details = BlockDetails {
					number: header.number(),
					total_difficulty: header.difficulty(),
					parent: header.parent_hash(),
					children: vec![]
				};

				bc.blocks_db.put(&hash, bytes).unwrap();

				let batch = WriteBatch::new();
				batch.put_extras(&hash, &details);
				batch.put_extras(&header.number(), &hash);
				batch.put(b"best", &hash).unwrap();
				bc.extras_db.write(batch).unwrap();
				
				hash
			}
		};

		bc.best_block_hash.set(best_block_hash);
		bc.best_block_number.set(bc.block_number(&best_block_hash).unwrap());
		bc.best_block_total_difficulty.set(bc.block_details(&best_block_hash).unwrap().total_difficulty);

		bc
	}

	/// Returns a tree route between `from` and `to`, which is a tuple of:
	/// - a vector of hashes of all blocks, ordered from `from` to `to`.
	/// - common ancestor of these blocks.
	/// - an index where best common ancestor would be
	/// 
	/// 1.) live blocks only:
	/// 
	/// bc: A1 -> A2 -> A3 -> A4 -> A5
	/// from: A5, to: A4
	/// route: { blocks: [A5, A4], ancestor: A3, index: 2 }
	/// 
	/// 2.) bad blocks only:
	/// 
	/// bc: A1 -> A2 -> A3 -> A4 -> A5
	///        -> B2 -> B3 -> B4
	/// from: B4, to: B3
	/// route: { blocks: [B4, B3], ancestor: B2, index: 2 }
	///
	/// 3.) fork:
	///
	/// bc: A1 -> A2 -> A3 -> A4
	///              -> B3 -> B4
	/// from: B4, to: A4
	/// route: { blocks: [B4, B3, A3, A4], ancestor: A2, index: 2 }
	pub fn tree_route(&self, from: &H256, to: &H256) -> TreeRoute {
		let mut from_branch = vec![];
		let mut to_branch = vec![];

		let mut from_details = self.block_details(from).expect("from hash is invalid!");
		let mut to_details = self.block_details(to).expect("to hash is invalid!");

		let mut current_from = from.clone();
		let mut current_to = to.clone();

		// reset from && to to the same level
		while from_details.number > to_details.number {
			from_branch.push(current_from);
			current_from = from_details.parent.clone();
			from_details = self.block_details(&from_details.parent).unwrap();
		}

		while to_details.number > from_details.number {
			to_branch.push(current_to);
			current_to = to_details.parent.clone();
			to_details = self.block_details(&to_details.parent).unwrap();
		}

		assert_eq!(from_details.number, to_details.number);

		// move to shared parent
		while from_details.parent != to_details.parent {
			from_branch.push(current_from);
			current_from = from_details.parent.clone();
			from_details = self.block_details(&from_details.parent).unwrap();

			to_branch.push(current_to);
			current_to = to_details.parent.clone();
			to_details = self.block_details(&to_details.parent).unwrap();
		}

		let index = from_branch.len();

		from_branch.extend(to_branch.iter().rev());

		TreeRoute {
			blocks: from_branch,
			ancestor: from_details.parent,
			index: index
		}
	}

	/// Inserts the block into backing cache database.
	/// Expects the block to be valid and already verified.
	/// If the block is already known, does nothing.
	pub fn insert_block(&self, bytes: &[u8]) {
		// create views onto rlp
		let block = BlockView::new(bytes);
		let header = block.header_view();

		if self.is_known(&header.sha3()) {
			return;
		}

		// prepare variables
		let hash = block.sha3();
		let mut parent_details = self.block_details(&header.parent_hash()).expect("Invalid parent hash.");
		let total_difficulty = parent_details.total_difficulty + header.difficulty();

		// create current block details
		let details = BlockDetails {
			number: header.number(),
			total_difficulty: total_difficulty,
			parent: header.parent_hash(),
			children: vec![]
		};
		
		// store block in db
		self.blocks_db.put(&hash, &bytes).unwrap();
		
		// update extra details
		{
			// insert new block details
			let batch = WriteBatch::new();
			batch.put_extras(&hash, &details);

			// update parent details
			parent_details.children.push(hash.clone());
			batch.put_extras(&header.parent_hash(), &parent_details);
			self.extras_db.write(batch).unwrap();
		}

		// check if we have new best block.
		// if yes, it means that we need to move it and its ancestors 
		// to "canon chain"
		if total_difficulty > self.best_block_total_difficulty() {

			// find the route between old best block and the new one
			let route = self.tree_route(&self.best_block_hash(), &hash);
			let ancestor_number = self.block_number(&route.ancestor).unwrap();
			let start_number = ancestor_number + U256::from(1u8);
			let extras_batch = route.blocks.iter()
				.skip(route.index)
				.enumerate()
				.fold(WriteBatch::new(), | acc, (index, hash) | {
					acc.put_extras(&(start_number + U256::from(index as u64)), hash);
					acc
				});

			// update extras database
			extras_batch.put(b"best", &hash).unwrap();
			self.extras_db.write(extras_batch).unwrap();

			// update local caches
			self.best_block_hash.set(hash);
			self.best_block_number.set(header.number());
			self.best_block_total_difficulty.set(total_difficulty);
		}
	}

	/// Returns true if the given block is known 
	/// (though not necessarily a part of the canon chain).
	pub fn is_known(&self, hash: &H256) -> bool {
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

	/// Get best block hash
	pub fn best_block_hash(&self) -> H256 {
		self.best_block_hash.get()
	}

	/// Get best block number
	pub fn best_block_number(&self) -> U256 {
		self.best_block_number.get()
	}

	/// Get best block total difficulty
	pub fn best_block_total_difficulty(&self) -> U256 {
		self.best_block_total_difficulty.get()
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

