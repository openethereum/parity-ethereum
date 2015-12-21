//! Fast access to blockchain data.

use std::collections::HashMap;
use std::cell::RefCell;
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
use header::*;
use extras::*;
use transaction::*;
use views::*;

/// Represents a tree route between `from` block and `to` block:
/// 
/// - `blocks` - a vector of hashes of all blocks, ordered from `from` to `to`.
///
/// - `ancestor` - best common ancestor of these blocks.
///
/// - `index` - an index where best common ancestor would be.
pub struct TreeRoute {
	pub blocks: Vec<H256>,
	pub ancestor: H256,
	pub index: usize
}

/// Represents blockchain's in-memory cache size in bytes.
#[derive(Debug)]
pub struct CacheSize {
	pub blocks: usize,
	pub block_details: usize,
	pub transaction_addresses: usize,
	pub block_logs: usize,
	pub blocks_blooms: usize
}

/// Grouped information about best block
struct BestBlock {
	pub hash: H256,
	pub number: U256,
	pub total_difficulty: U256
}

impl BestBlock {
	fn new() -> BestBlock {
		BestBlock {
			hash: H256::new(),
			number: U256::from(0),
			total_difficulty: U256::from(0)
		}
	}
}

/// Structure providing fast access to blockchain data.
/// 
/// **Does not do input data verifycation.**
pub struct BlockChain {
	best_block: RefCell<BestBlock>,

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
	/// 	let bc = BlockChain::new(genesis.block(), &dir);
	///
	/// 	let genesis_hash = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
	/// 	assert_eq!(bc.genesis_hash(), H256::from_str(genesis_hash).unwrap());
	/// 	assert!(bc.is_known(&bc.genesis_hash()));
	/// 	assert_eq!(bc.genesis_hash(), bc.block_hash(&U256::from(0u8)).unwrap());
	/// }
	/// ```
	pub fn new(genesis: &[u8], path: &Path) -> BlockChain {
		// open extras db
		let mut extras_path = path.to_path_buf();
		extras_path.push("extras");
		let extras_db = DB::open_default(extras_path.to_str().unwrap()).unwrap();

		// open blocks db
		let mut blocks_path = path.to_path_buf();
		blocks_path.push("blocks");
		let blocks_db = DB::open_default(blocks_path.to_str().unwrap()).unwrap();

		let bc = BlockChain {
			best_block: RefCell::new(BestBlock::new()),
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
				let block = BlockView::new(genesis);
				let header = block.header_view();
				let hash = block.sha3();

				let details = BlockDetails {
					number: header.number(),
					total_difficulty: header.difficulty(),
					parent: header.parent_hash(),
					children: vec![]
				};

				bc.blocks_db.put(&hash, genesis).unwrap();

				let batch = WriteBatch::new();
				batch.put_extras(&hash, &details);
				batch.put_extras(&header.number(), &hash);
				batch.put(b"best", &hash).unwrap();
				bc.extras_db.write(batch).unwrap();
				
				hash
			}
		};

		{
			let mut best_block = bc.best_block.borrow_mut();
			best_block.number = bc.block_number(&best_block_hash).unwrap();
			best_block.total_difficulty = bc.block_details(&best_block_hash).unwrap().total_difficulty;
			best_block.hash = best_block_hash;
		}

		bc
	}

	/// Returns a tree route between `from` and `to`, which is a tuple of:
	/// 
	/// - a vector of hashes of all blocks, ordered from `from` to `to`.
	///
	/// - common ancestor of these blocks.
	///
	/// - an index where best common ancestor would be
	/// 
	/// 1.) from newer to older
	/// 
	/// - bc: `A1 -> A2 -> A3 -> A4 -> A5`
	/// - from: A5, to: A4
	/// - route: 
	///
	///   ```json
	///   { blocks: [A5], ancestor: A4, index: 1 }
	///   ```
	/// 
	/// 2.) from older to newer
	/// 
	/// - bc: `A1 -> A2 -> A3 -> A4 -> A5`
	/// - from: A3, to: A4
	/// - route: 
	/// 
	///   ```json
	///   { blocks: [A4], ancestor: A3, index: 0 }
	///   ```
	///
	/// 3.) fork:
	///
	/// - bc: 
	///
	///   ```text
	///   A1 -> A2 -> A3 -> A4
	///              -> B3 -> B4
	///   ``` 
	/// - from: B4, to: A4
	/// - route: 
	/// 
	///   ```json
	///   { blocks: [B4, B3, A3, A4], ancestor: A2, index: 2 }
	///   ```
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
		while current_from != current_to {
			from_branch.push(current_from);
			current_from = from_details.parent.clone();
			from_details = self.block_details(&from_details.parent).unwrap();

			to_branch.push(current_to);
			current_to = to_details.parent.clone();
			to_details = self.block_details(&to_details.parent).unwrap();
		}

		let index = from_branch.len();

		from_branch.extend(to_branch.into_iter().rev());

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
		let parent_hash = header.parent_hash();

		// create current block details
		let details = BlockDetails {
			number: header.number(),
			total_difficulty: total_difficulty,
			parent: parent_hash.clone(),
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
			batch.put_extras(&parent_hash, &parent_details);
			self.extras_db.write(batch).unwrap();
			
			// also in cache if it's there...
			let mut write = self.block_details.borrow_mut();
			match write.get_mut(&parent_hash) {
				Some(parent_details) => parent_details.children.push(hash.clone()),
				None => ()
			}
		}

		// check if we have new best block.
		// if yes, it means that we need to move it and its ancestors 
		// to "canon chain"
		if total_difficulty > self.best_block_total_difficulty() {

			// find the route between old best block and the new one
			let route = self.tree_route(&self.best_block_hash(), &hash);

			let extras_batch = match route.blocks.len() {
				// its our parent
				1 => {
					let extras = WriteBatch::new();
					extras.put_extras(&header.number(), &hash);
					extras
				},
				// it is a fork
				i if i > 1 => {
					let ancestor_number = self.block_number(&route.ancestor).unwrap();
					let start_number = ancestor_number + U256::from(1u8);
					route.blocks.iter()
						.skip(route.index)
						.enumerate()
						.fold(WriteBatch::new(), | acc, (index, hash) | {
							acc.put_extras(&(start_number + U256::from(index as u64)), hash);
							acc
						})
				},
				// route.len() could be 0 only if inserted block is best block,
				// and this is not possible at this stage
				_ => { unreachable!(); }
			};

			// update extras database
			extras_batch.put(b"best", &hash).unwrap();
			self.extras_db.write(extras_batch).unwrap();

			// update local caches
			let mut best_block = self.best_block.borrow_mut();
			best_block.hash = hash;
			best_block.number = header.number();
			best_block.total_difficulty = total_difficulty;
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

	/// Returns reference to genesis hash.
	pub fn genesis_hash(&self) -> H256 {
		self.block_hash(&U256::from(0u8)).expect("Genesis hash should always exist")
	}

	/// Get the partial-header of a block.
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

	/// Get the hash of given block's number.
	pub fn block_hash(&self, hash: &U256) -> Option<H256> {
		self.query_extras(hash, &self.block_hashes)
	}

	/// Get best block hash.
	pub fn best_block_hash(&self) -> H256 {
		self.best_block.borrow().hash.clone()
	}

	/// Get best block number.
	pub fn best_block_number(&self) -> U256 {
		self.best_block.borrow().number
	}

	/// Get best block total difficulty.
	pub fn best_block_total_difficulty(&self) -> U256 {
		self.best_block.borrow().total_difficulty
	}

	/// Get the number of given block's hash.
	pub fn block_number(&self, hash: &H256) -> Option<U256> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).header_view().number())
	}

	/// Get the transactions' log blooms of a block.
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

	/// Get current cache size.
	pub fn cache_size(&self) -> CacheSize {
		CacheSize {
			blocks: self.blocks.heap_size_of_children(),
			block_details: self.block_details.heap_size_of_children(),
			transaction_addresses: self.transaction_addresses.heap_size_of_children(),
			block_logs: self.block_logs.heap_size_of_children(),
			blocks_blooms: self.blocks_blooms.heap_size_of_children()
		}
	}

	/// Tries to squeeze the cache if its too big.
	pub fn squeeze_to_fit(&self, size: CacheSize) {
		self.blocks.borrow_mut().squeeze(size.blocks);
		self.block_details.borrow_mut().squeeze(size.block_details);
		self.transaction_addresses.borrow_mut().squeeze(size.transaction_addresses);
		self.block_logs.borrow_mut().squeeze(size.block_logs);
		self.blocks_blooms.borrow_mut().squeeze(size.blocks_blooms);
	}
}

#[cfg(test)]
mod tests {
	use std::env;
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;
	use util::hash::*;
	use util::uint::*;
	use blockchain::*;

	#[test]
	fn valid_tests_extra32() {
		let genesis = "f901fcf901f7a00000000000000000000000000000000000000000000000000000000000000000a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0925002c3260b44e44c3edebad1cc442142b03020209df1ab8bb86752edbd2cd7a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000080832fefd8808454c98c8142a0363659b251bf8b819179874c8cce7b9b983d7f3704cbb58a3b334431f7032871889032d09c281e1236c0c0".from_hex().unwrap();

		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());

		let bc = BlockChain::new(&genesis, &dir);
		
		let genesis_hash = H256::from_str("3caa2203f3d7c136c0295ed128a7d31cea520b1ca5e27afe17d0853331798942").unwrap();

		assert_eq!(bc.genesis_hash(), genesis_hash.clone());
		assert_eq!(bc.best_block_number(), U256::from(0u8));
		assert_eq!(bc.best_block_hash(), genesis_hash.clone());
		assert_eq!(bc.block_hash(&U256::from(0u8)), Some(genesis_hash.clone()));
		assert_eq!(bc.block_hash(&U256::from(1u8)), None);
		

		let first = "f90285f90219a03caa2203f3d7c136c0295ed128a7d31cea520b1ca5e27afe17d0853331798942a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0bac6177a79e910c98d86ec31a09ae37ac2de15b754fd7bed1ba52362c49416bfa0d45893a296c1490a978e0bd321b5f2635d8280365c1fe9f693d65f233e791344a0c7778a7376099ee2e5c455791c1885b5c361b95713fddcbe32d97fd01334d296b90100000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000400000000000000000000000000000000000000000000000000000008302000001832fefd882560b845627cb99a00102030405060708091011121314151617181920212223242526272829303132a08ccb2837fb2923bd97e8f2d08ea32012d6e34be018c73e49a0f98843e8f47d5d88e53be49fec01012ef866f864800a82c35094095e7baea6a6c7c4c2dfeb977efac326af552d8785012a05f200801ba0cb088b8d2ff76a7b2c6616c9d02fb6b7a501afbf8b69d7180b09928a1b80b5e4a06448fe7476c606582039bb72a9f6f4b4fad18507b8dfbd00eebbe151cc573cd2c0".from_hex().unwrap();

		bc.insert_block(&first);

		let first_hash = H256::from_str("a940e5af7d146b3b917c953a82e1966b906dace3a4e355b5b0a4560190357ea1").unwrap();

		assert_eq!(bc.block_hash(&U256::from(0u8)), Some(genesis_hash.clone()));
		assert_eq!(bc.best_block_number(), U256::from(1u8));
		assert_eq!(bc.best_block_hash(), first_hash.clone());
		assert_eq!(bc.block_hash(&U256::from(1u8)), Some(first_hash.clone()));
		assert_eq!(bc.block_details(&first_hash).unwrap().parent, genesis_hash.clone());
		assert_eq!(bc.block_details(&genesis_hash).unwrap().children, vec![first_hash.clone()]);
		assert_eq!(bc.block_hash(&U256::from(2u8)), None);
	}
}
