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

//! Blockchain database.

use util::*;
use rocksdb::{DB, WriteBatch, Writable};
use header::*;
use extras::*;
use transaction::*;
use views::*;

/// Represents a tree route between `from` block and `to` block:
pub struct TreeRoute {
	/// A vector of hashes of all blocks, ordered from `from` to `to`.
	pub blocks: Vec<H256>,
	/// Best common ancestor of these blocks.
	pub ancestor: H256,
	/// An index where best common ancestor would be.
	pub index: usize
}

/// Represents blockchain's in-memory cache size in bytes.
#[derive(Debug)]
pub struct CacheSize {
	/// Blocks cache size.
	pub blocks: usize,
	/// BlockDetails cache size.
	pub block_details: usize,
	/// Transaction addresses cache size.
	pub transaction_addresses: usize,
	/// Logs cache size.
	pub block_logs: usize,
	/// Blooms cache size.
	pub blocks_blooms: usize
}

impl CacheSize {
	/// Total amount used by the cache.
	fn total(&self) -> usize { self.blocks + self.block_details + self.transaction_addresses + self.block_logs + self.blocks_blooms }
}

/// Information about best block gathered together
struct BestBlock {
	pub hash: H256,
	pub number: BlockNumber,
	pub total_difficulty: U256
}

impl BestBlock {
	fn new() -> BestBlock {
		BestBlock {
			hash: H256::new(),
			number: 0,
			total_difficulty: U256::from(0)
		}
	}
}

/// Interface for querying blocks by hash and by number.
pub trait BlockProvider {
	/// Returns true if the given block is known
	/// (though not necessarily a part of the canon chain).
	fn is_known(&self, hash: &H256) -> bool;

	/// Get raw block data
	fn block(&self, hash: &H256) -> Option<Bytes>;

	/// Get the familial details concerning a block.
	fn block_details(&self, hash: &H256) -> Option<BlockDetails>;

	/// Get the hash of given block's number.
	fn block_hash(&self, index: BlockNumber) -> Option<H256>;

	/// Get the partial-header of a block.
	fn block_header(&self, hash: &H256) -> Option<Header> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).header())
	}

	/// Get a list of uncles for a given block.
	/// Returns None if block deos not exist.
	fn uncles(&self, hash: &H256) -> Option<Vec<Header>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).uncles())
	}

	/// Get a list of uncle hashes for a given block.
	/// Returns None if block does not exist.
	fn uncle_hashes(&self, hash: &H256) -> Option<Vec<H256>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).uncle_hashes())
	}

	/// Get the number of given block's hash.
	fn block_number(&self, hash: &H256) -> Option<BlockNumber> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).header_view().number())
	}

	/// Get a list of transactions for a given block.
	/// Returns None if block deos not exist.
	fn transactions(&self, hash: &H256) -> Option<Vec<SignedTransaction>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).transactions())
	}

	/// Returns reference to genesis hash.
	fn genesis_hash(&self) -> H256 {
		self.block_hash(0).expect("Genesis hash should always exist")
	}

	/// Returns the header of the genesis block.
	fn genesis_header(&self) -> Header {
		self.block_header(&self.genesis_hash()).unwrap()
	}
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum CacheID {
	Block(H256),
	Extras(ExtrasIndex, H256),
}

struct CacheManager {
	cache_usage: VecDeque<HashSet<CacheID>>,
	in_use: HashSet<CacheID>,
}

/// Structure providing fast access to blockchain data.
///
/// **Does not do input data verification.**
pub struct BlockChain {
	pref_cache_size: usize,
	max_cache_size: usize,

	best_block: RwLock<BestBlock>,

	// block cache
	blocks: RwLock<HashMap<H256, Bytes>>,

	// extra caches
	block_details: RwLock<HashMap<H256, BlockDetails>>,
	block_hashes: RwLock<HashMap<BlockNumber, H256>>,
	transaction_addresses: RwLock<HashMap<H256, TransactionAddress>>,
	block_logs: RwLock<HashMap<H256, BlockLogBlooms>>,
	blocks_blooms: RwLock<HashMap<H256, BlocksBlooms>>,

	extras_db: DB,
	blocks_db: DB,

	cache_man: RwLock<CacheManager>,
}

impl BlockProvider for BlockChain {
	/// Returns true if the given block is known
	/// (though not necessarily a part of the canon chain).
	fn is_known(&self, hash: &H256) -> bool {
		self.query_extras_exist(hash, &self.block_details)
	}

	/// Get raw block data
	fn block(&self, hash: &H256) -> Option<Bytes> {
		{
			let read = self.blocks.read().unwrap();
			if let Some(v) = read.get(hash) {
				return Some(v.clone());
			}
		}

		let opt = self.blocks_db.get(hash)
			.expect("Low level database error. Some issue with disk?");

		self.note_used(CacheID::Block(hash.clone()));

		match opt {
			Some(b) => {
				let bytes: Bytes = b.to_vec();
				let mut write = self.blocks.write().unwrap();
				write.insert(hash.clone(), bytes.clone());
				Some(bytes)
			},
			None => None
		}
	}

	/// Get the familial details concerning a block.
	fn block_details(&self, hash: &H256) -> Option<BlockDetails> {
		self.query_extras(hash, &self.block_details)
	}

	/// Get the hash of given block's number.
	fn block_hash(&self, index: BlockNumber) -> Option<H256> {
		self.query_extras(&index, &self.block_hashes)
	}
}

const COLLECTION_QUEUE_SIZE: usize = 8;

impl BlockChain {
	/// Create new instance of blockchain from given Genesis
	///
	/// ```rust
	/// extern crate ethcore_util as util;
	/// extern crate ethcore;
	/// use std::env;
	/// use std::str::FromStr;
	/// use ethcore::spec::*;
	/// use ethcore::blockchain::*;
	/// use ethcore::ethereum;
	/// use util::hash::*;
	/// use util::uint::*;
	///
	/// fn main() {
	/// 	let spec = ethereum::new_frontier();
	///
	/// 	let mut dir = env::temp_dir();
	/// 	dir.push(H32::random().hex());
	///
	/// 	let bc = BlockChain::new(&spec.genesis_block(), &dir);
	///
	/// 	let genesis_hash = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
	/// 	assert_eq!(bc.genesis_hash(), H256::from_str(genesis_hash).unwrap());
	/// 	assert!(bc.is_known(&bc.genesis_hash()));
	/// 	assert_eq!(bc.genesis_hash(), bc.block_hash(0).unwrap());
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

		let mut cache_man = CacheManager{cache_usage: VecDeque::new(), in_use: HashSet::new()};
		(0..COLLECTION_QUEUE_SIZE).foreach(|_| cache_man.cache_usage.push_back(HashSet::new()));

		let bc = BlockChain {
			pref_cache_size: 1 << 14,
			max_cache_size: 1 << 20,
			best_block: RwLock::new(BestBlock::new()),
			blocks: RwLock::new(HashMap::new()),
			block_details: RwLock::new(HashMap::new()),
			block_hashes: RwLock::new(HashMap::new()),
			transaction_addresses: RwLock::new(HashMap::new()),
			block_logs: RwLock::new(HashMap::new()),
			blocks_blooms: RwLock::new(HashMap::new()),
			extras_db: extras_db,
			blocks_db: blocks_db,
			cache_man: RwLock::new(cache_man),
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
			let mut best_block = bc.best_block.write().unwrap();
			best_block.number = bc.block_number(&best_block_hash).unwrap();
			best_block.total_difficulty = bc.block_details(&best_block_hash).unwrap().total_difficulty;
			best_block.hash = best_block_hash;
		}

		bc
	}

	/// Set the cache configuration.
	pub fn configure_cache(&mut self, pref_cache_size: usize, max_cache_size: usize) {
		self.pref_cache_size = pref_cache_size;
		self.max_cache_size = max_cache_size;
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
	pub fn tree_route(&self, from: H256, to: H256) -> Option<TreeRoute> {
		let from_details = match self.block_details(&from) {
			Some(h) => h,
			None => return None,
		};
		let to_details = match self.block_details(&to) {
			Some(h) => h,
			None => return None,
		};
		Some(self.tree_route_aux((&from_details, &from), (&to_details, &to)))
	}

	/// Similar to `tree_route` function, but can be used to return a route
	/// between blocks which may not be in database yet.
	fn tree_route_aux(&self, from: (&BlockDetails, &H256), to: (&BlockDetails, &H256)) -> TreeRoute {
		let mut from_branch = vec![];
		let mut to_branch = vec![];

		let mut from_details = from.0.clone();
		let mut to_details = to.0.clone();
		let mut current_from = from.1.clone();
		let mut current_to = to.1.clone();

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
			ancestor: current_from,
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
		let hash = header.sha3();

		if self.is_known(&hash) {
			return;
		}

		// store block in db
		self.blocks_db.put(&hash, &bytes).unwrap();
		let (batch, new_best, details) = self.block_to_extras_insert_batch(bytes);

		// update best block
		let mut best_block = self.best_block.write().unwrap();
		if let Some(b) = new_best {
			*best_block = b;
		}

		// update caches
		let mut write = self.block_details.write().unwrap();
		write.remove(&header.parent_hash());
		write.insert(hash.clone(), details);
		self.note_used(CacheID::Block(hash));

		// update extras database
		self.extras_db.write(batch).unwrap();
	}

	/// Transforms block into WriteBatch that may be written into database
	/// Additionally, if it's new best block it returns new best block object.
	fn block_to_extras_insert_batch(&self, bytes: &[u8]) -> (WriteBatch, Option<BestBlock>, BlockDetails) {
		// create views onto rlp
		let block = BlockView::new(bytes);
		let header = block.header_view();

		// prepare variables
		let hash = block.sha3();
		let mut parent_details = self.block_details(&header.parent_hash()).expect("Invalid parent hash.");
		let total_difficulty = parent_details.total_difficulty + header.difficulty();
		let is_new_best = total_difficulty > self.best_block_total_difficulty();
		let parent_hash = header.parent_hash();

		// create current block details
		let details = BlockDetails {
			number: header.number(),
			total_difficulty: total_difficulty,
			parent: parent_hash.clone(),
			children: vec![]
		};

		// prepare the batch
		let batch = WriteBatch::new();

		// insert new block details
		batch.put_extras(&hash, &details);

		// update parent details
		parent_details.children.push(hash.clone());
		batch.put_extras(&parent_hash, &parent_details);

		// if it's not new best block, just return
		if !is_new_best {
			return (batch, None, details);
		}

		// if its new best block we need to make sure that all ancestors
		// are moved to "canon chain"
		// find the route between old best block and the new one
		let best_hash = self.best_block_hash();
		let best_details = self.block_details(&best_hash).expect("best block hash is invalid!");
		let route = self.tree_route_aux((&best_details, &best_hash), (&details, &hash));

		match route.blocks.len() {
			// its our parent
			1 => batch.put_extras(&header.number(), &hash),
			// it is a fork
			i if i > 1 => {
				let ancestor_number = self.block_number(&route.ancestor).unwrap();
				let start_number = ancestor_number + 1;
				for (index, hash) in route.blocks.iter().skip(route.index).enumerate() {
					batch.put_extras(&(start_number + index as BlockNumber), hash);
				}
			},
			// route.blocks.len() could be 0 only if inserted block is best block,
			// and this is not possible at this stage
			_ => { unreachable!(); }
		};

		// this is new best block
		batch.put(b"best", &hash).unwrap();

		let best_block = BestBlock {
			hash: hash,
			number: header.number(),
			total_difficulty: total_difficulty
		};

		(batch, Some(best_block), details)
	}

	/// Returns true if transaction is known.
	pub fn is_known_transaction(&self, hash: &H256) -> bool {
		self.query_extras_exist(hash, &self.transaction_addresses)
	}

	/// Get best block hash.
	pub fn best_block_hash(&self) -> H256 {
		self.best_block.read().unwrap().hash.clone()
	}

	/// Get best block number.
	pub fn best_block_number(&self) -> BlockNumber {
		self.best_block.read().unwrap().number
	}

	/// Get best block total difficulty.
	pub fn best_block_total_difficulty(&self) -> U256 {
		self.best_block.read().unwrap().total_difficulty
	}

	/// Get the transactions' log blooms of a block.
	pub fn log_blooms(&self, hash: &H256) -> Option<BlockLogBlooms> {
		self.query_extras(hash, &self.block_logs)
	}

	fn query_extras<K, T>(&self, hash: &K, cache: &RwLock<HashMap<K, T>>) -> Option<T> where
		T: Clone + Decodable + ExtrasIndexable,
		K: ExtrasSliceConvertable + Eq + Hash + Clone {
		{
			let read = cache.read().unwrap();
			if let Some(v) = read.get(hash) {
				return Some(v.clone());
			}
		}

		if let Some(h) = hash.as_h256() {
			self.note_used(CacheID::Extras(T::extras_index(), h.clone()));
		}

		self.extras_db.get_extras(hash).map(| t: T | {
			let mut write = cache.write().unwrap();
			write.insert(hash.clone(), t.clone());
			t
		})
	}

	fn query_extras_exist<K, T>(&self, hash: &K, cache: &RwLock<HashMap<K, T>>) -> bool where
		K: ExtrasSliceConvertable + Eq + Hash + Clone,
		T: ExtrasIndexable {
		{
			let read = cache.read().unwrap();
			if let Some(_) = read.get(hash) {
				return true;
			}
		}

		self.extras_db.extras_exists::<_, T>(hash)
	}

	/// Get current cache size.
	pub fn cache_size(&self) -> CacheSize {
		CacheSize {
			blocks: self.blocks.read().unwrap().heap_size_of_children(),
			block_details: self.block_details.read().unwrap().heap_size_of_children(),
			transaction_addresses: self.transaction_addresses.read().unwrap().heap_size_of_children(),
			block_logs: self.block_logs.read().unwrap().heap_size_of_children(),
			blocks_blooms: self.blocks_blooms.read().unwrap().heap_size_of_children()
		}
	}

	/// Let the cache system know that a cacheable item has been used.
	fn note_used(&self, id: CacheID) {
		let mut cache_man = self.cache_man.write().unwrap();
		if !cache_man.cache_usage[0].contains(&id) {
			cache_man.cache_usage[0].insert(id.clone());
			if cache_man.in_use.contains(&id) {
				if let Some(c) = cache_man.cache_usage.iter_mut().skip(1).find(|e|e.contains(&id)) {
					c.remove(&id);
				}
			} else {
				cache_man.in_use.insert(id);
			}
		}
	}

	/// Ticks our cache system and throws out any old data.
	pub fn collect_garbage(&self) {
		if self.cache_size().total() < self.pref_cache_size { return; }

		for _ in 0..COLLECTION_QUEUE_SIZE {
			{
				let mut cache_man = self.cache_man.write().unwrap();
				let mut blocks = self.blocks.write().unwrap();
				let mut block_details = self.block_details.write().unwrap();
				let mut block_hashes = self.block_hashes.write().unwrap();
				let mut transaction_addresses = self.transaction_addresses.write().unwrap();
				let mut block_logs = self.block_logs.write().unwrap();
				let mut blocks_blooms = self.blocks_blooms.write().unwrap();

				for id in cache_man.cache_usage.pop_back().unwrap().into_iter() {
					cache_man.in_use.remove(&id);
					match id {
						CacheID::Block(h) => { blocks.remove(&h); },
						CacheID::Extras(ExtrasIndex::BlockDetails, h) => { block_details.remove(&h); },
						CacheID::Extras(ExtrasIndex::TransactionAddress, h) => { transaction_addresses.remove(&h); },
						CacheID::Extras(ExtrasIndex::BlockLogBlooms, h) => { block_logs.remove(&h); },
						CacheID::Extras(ExtrasIndex::BlocksBlooms, h) => { blocks_blooms.remove(&h); },
						_ => panic!(),
					}
				}
				cache_man.cache_usage.push_front(HashSet::new());

				// TODO: handle block_hashes properly.
				block_hashes.clear();
			}
			if self.cache_size().total() < self.max_cache_size { break; }
		}

		// TODO: m_lastCollection = chrono::system_clock::now();
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;
	use util::hash::*;
	use blockchain::*;
	use tests::helpers::*;

	#[test]
	fn valid_tests_extra32() {
		let genesis = "f901fcf901f7a00000000000000000000000000000000000000000000000000000000000000000a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0925002c3260b44e44c3edebad1cc442142b03020209df1ab8bb86752edbd2cd7a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000080832fefd8808454c98c8142a0363659b251bf8b819179874c8cce7b9b983d7f3704cbb58a3b334431f7032871889032d09c281e1236c0c0".from_hex().unwrap();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(&genesis, temp.as_path());

		let genesis_hash = H256::from_str("3caa2203f3d7c136c0295ed128a7d31cea520b1ca5e27afe17d0853331798942").unwrap();

		assert_eq!(bc.genesis_hash(), genesis_hash.clone());
		assert_eq!(bc.best_block_number(), 0);
		assert_eq!(bc.best_block_hash(), genesis_hash.clone());
		assert_eq!(bc.block_hash(0), Some(genesis_hash.clone()));
		assert_eq!(bc.block_hash(1), None);
		
		let first = "f90285f90219a03caa2203f3d7c136c0295ed128a7d31cea520b1ca5e27afe17d0853331798942a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0bac6177a79e910c98d86ec31a09ae37ac2de15b754fd7bed1ba52362c49416bfa0d45893a296c1490a978e0bd321b5f2635d8280365c1fe9f693d65f233e791344a0c7778a7376099ee2e5c455791c1885b5c361b95713fddcbe32d97fd01334d296b90100000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000400000000000000000000000000000000000000000000000000000008302000001832fefd882560b845627cb99a00102030405060708091011121314151617181920212223242526272829303132a08ccb2837fb2923bd97e8f2d08ea32012d6e34be018c73e49a0f98843e8f47d5d88e53be49fec01012ef866f864800a82c35094095e7baea6a6c7c4c2dfeb977efac326af552d8785012a05f200801ba0cb088b8d2ff76a7b2c6616c9d02fb6b7a501afbf8b69d7180b09928a1b80b5e4a06448fe7476c606582039bb72a9f6f4b4fad18507b8dfbd00eebbe151cc573cd2c0".from_hex().unwrap();

		bc.insert_block(&first);

		let first_hash = H256::from_str("a940e5af7d146b3b917c953a82e1966b906dace3a4e355b5b0a4560190357ea1").unwrap();

		assert_eq!(bc.block_hash(0), Some(genesis_hash.clone()));
		assert_eq!(bc.best_block_number(), 1);
		assert_eq!(bc.best_block_hash(), first_hash.clone());
		assert_eq!(bc.block_hash(1), Some(first_hash.clone()));
		assert_eq!(bc.block_details(&first_hash).unwrap().parent, genesis_hash.clone());
		assert_eq!(bc.block_details(&genesis_hash).unwrap().children, vec![first_hash.clone()]);
		assert_eq!(bc.block_hash(2), None);
	}

	#[test]
	#[allow(cyclomatic_complexity)]
	fn test_small_fork() {
		let genesis = "f901fcf901f7a00000000000000000000000000000000000000000000000000000000000000000a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a07dba07d6b448a186e9612e5f737d1c909dce473e53199901a302c00646d523c1a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000080832fefd8808454c98c8142a059262c330941f3fe2a34d16d6e3c7b30d2ceb37c6a0e9a994c494ee1a61d2410885aa4c8bf8e56e264c0c0".from_hex().unwrap();
		let b1 = "f90261f901f9a05716670833ec874362d65fea27a7cd35af5897d275b31a44944113111e4e96d2a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0cb52de543653d86ccd13ba3ddf8b052525b04231c6884a4db3188a184681d878a0e78628dd45a1f8dc495594d83b76c588a3ee67463260f8b7d4a42f574aeab29aa0e9244cf7503b79c03d3a099e07a80d2dbc77bb0b502d8a89d51ac0d68dd31313b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000001832fefd882520884562791e580a051b3ecba4e3f2b49c11d42dd0851ec514b1be3138080f72a2b6e83868275d98f8877671f479c414b47f862f86080018304cb2f94095e7baea6a6c7c4c2dfeb977efac326af552d870a801ca09e2709d7ec9bbe6b1bbbf0b2088828d14cd5e8642a1fee22dc74bfa89761a7f9a04bd8813dee4be989accdb708b1c2e325a7e9c695a8024e30e89d6c644e424747c0".from_hex().unwrap();
		let b2 = "f902ccf901f9a0437e51676ff10756fcfee5edd9159fa41dbcb1b2c592850450371cbecd54ee4fa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0c70a5dc56146e5ef025e4e5726a6373c6f12fd2f6784093a19ead0a7d17fb292a040645cbce4fd399e7bb9160b4c30c40d7ee616a030d4e18ef0ed3b02bdb65911a086e608555f63628417032a011d107b36427af37d153f0da02ce3f90fdd5e8c08b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302004002832fefd882c0e384562791e880a0e3cc39ff775cc0a32f175995b92e84b729e5c9a3563ff899e3555b908bc21d75887c3cde283f4846a6f8cdf8cb01018304cb2f8080b87e6060604052606e8060106000396000f360606040526000357c010000000000000000000000000000000000000000000000000000000090048063c0406226146037576035565b005b60406004506056565b6040518082815260200191505060405180910390f35b6000600560006000508190555060059050606b565b90561ba05258615c63503c0a600d6994b12ea5750d45b3c69668e2a371b4fbfb9eeff6b8a0a11be762bc90491231274a2945be35a43f23c27775b1ff24dd521702fe15f73ec0".from_hex().unwrap();
		let b3a = "f90261f901f9a036fde1253128666fcb95a5956da14a73489e988bb72738717ec1d31e1cee781aa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a05fb2b4bfdef7b314451cb138a534d225c922fc0e5fbe25e451142732c3e25c25a09dc4b1357c0b7b8108f8a098f4f9a1a274957bc9ebc22a9ae67ae81739e5b19ca007c6fdfa8eea7e86b81f5b0fc0f78f90cc19f4aa60d323151e0cac660199e9a1b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302008003832fefd882524d84562791eb80a074861666bd346c025889745c793b91ab9cd1e2ca19b5cf3c50d04d135b0a4d2b8809fe9587ea4cdc04f862f86002018304cb2f94ec0e71ad0a90ffe1909d27dac207f7680abba42d01801ba06fd84874d36d5de9e8e48978c03619b53a96b7ae0a4cd1ac118f103098b44801a00572596974dd7df4f9f69bd7456585618c568d8434ef6453391b89281ce12ae1c0".from_hex().unwrap();
		let b3b = "f90265f901f9a036fde1253128666fcb95a5956da14a73489e988bb72738717ec1d31e1cee781aa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0ab87dc338bfd6f662b1cd90bc0c9e40a1b2146a095312393c9e13ce3a5008b09a0e609b7a7d4b8a2403ec1268627ecd98783627246e8f1b26addb3ff504f76a054a0592fabf92476512952db3a69a2481a42912e668a1ee28c4c322e703bb665f8beb90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302008003832fefd882a1f084562791ee80a0fe7098fa7e4ac5d637eea81fb23f8f78346826dbab430068dd9a249d0afa99818853e1a6b201ae3545f866f86402018304cb2f94ec0e71ad0a90ffe1909d27dac207f7680abba42d0284c04062261ca06edc9ce8e7da4cc34067beb325dcad59e5655a164a5100a50bc3eb681b12c716a0abf9053d5de65b1be81fe50d327b84de685efbeecea34e7b747180a6c6023e44c0".from_hex().unwrap();

		let genesis_hash = H256::from_str("5716670833ec874362d65fea27a7cd35af5897d275b31a44944113111e4e96d2").unwrap();
		let b1_hash = H256::from_str("437e51676ff10756fcfee5edd9159fa41dbcb1b2c592850450371cbecd54ee4f").unwrap();
		let b2_hash = H256::from_str("36fde1253128666fcb95a5956da14a73489e988bb72738717ec1d31e1cee781a").unwrap();
		let b3a_hash = H256::from_str("c208f88c9f5bf7e00840439742c12e5226d9752981f3ec0521bdcb6dd08af277").unwrap();
		let b3b_hash = H256::from_str("bf72270ae0d95c9ea39a6adab994793fddb8c10fba7391e26279474124605d54").unwrap();

		// b3a is a part of canon chain, whereas b3b is part of sidechain
		let best_block_hash = H256::from_str("c208f88c9f5bf7e00840439742c12e5226d9752981f3ec0521bdcb6dd08af277").unwrap();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(&genesis, temp.as_path());
		bc.insert_block(&b1);
		bc.insert_block(&b2);
		bc.insert_block(&b3a);
		bc.insert_block(&b3b);

		assert_eq!(bc.best_block_hash(), best_block_hash);
		assert_eq!(bc.block_number(&genesis_hash).unwrap(), 0);
		assert_eq!(bc.block_number(&b1_hash).unwrap(), 1);
		assert_eq!(bc.block_number(&b2_hash).unwrap(), 2);
		assert_eq!(bc.block_number(&b3a_hash).unwrap(), 3);
		assert_eq!(bc.block_number(&b3b_hash).unwrap(), 3);

		assert_eq!(bc.block_hash(0).unwrap(), genesis_hash);
		assert_eq!(bc.block_hash(1).unwrap(), b1_hash);
		assert_eq!(bc.block_hash(2).unwrap(), b2_hash);
		assert_eq!(bc.block_hash(3).unwrap(), b3a_hash);

		// test trie route
		let r0_1 = bc.tree_route(genesis_hash.clone(), b1_hash.clone()).unwrap();
		assert_eq!(r0_1.ancestor, genesis_hash);
		assert_eq!(r0_1.blocks, [b1_hash.clone()]);
		assert_eq!(r0_1.index, 0);

		let r0_2 = bc.tree_route(genesis_hash.clone(), b2_hash.clone()).unwrap();
		assert_eq!(r0_2.ancestor, genesis_hash);
		assert_eq!(r0_2.blocks, [b1_hash.clone(), b2_hash.clone()]);
		assert_eq!(r0_2.index, 0);

		let r1_3a = bc.tree_route(b1_hash.clone(), b3a_hash.clone()).unwrap();
		assert_eq!(r1_3a.ancestor, b1_hash);
		assert_eq!(r1_3a.blocks, [b2_hash.clone(), b3a_hash.clone()]);
		assert_eq!(r1_3a.index, 0);

		let r1_3b = bc.tree_route(b1_hash.clone(), b3b_hash.clone()).unwrap();
		assert_eq!(r1_3b.ancestor, b1_hash);
		assert_eq!(r1_3b.blocks, [b2_hash.clone(), b3b_hash.clone()]);
		assert_eq!(r1_3b.index, 0);

		let r3a_3b = bc.tree_route(b3a_hash.clone(), b3b_hash.clone()).unwrap();
		assert_eq!(r3a_3b.ancestor, b2_hash);
		assert_eq!(r3a_3b.blocks, [b3a_hash.clone(), b3b_hash.clone()]);
		assert_eq!(r3a_3b.index, 1);

		let r1_0 = bc.tree_route(b1_hash.clone(), genesis_hash.clone()).unwrap();
		assert_eq!(r1_0.ancestor, genesis_hash);
		assert_eq!(r1_0.blocks, [b1_hash.clone()]);
		assert_eq!(r1_0.index, 1);

		let r2_0 = bc.tree_route(b2_hash.clone(), genesis_hash.clone()).unwrap();
		assert_eq!(r2_0.ancestor, genesis_hash);
		assert_eq!(r2_0.blocks, [b2_hash.clone(), b1_hash.clone()]);
		assert_eq!(r2_0.index, 2);

		let r3a_1 = bc.tree_route(b3a_hash.clone(), b1_hash.clone()).unwrap();
		assert_eq!(r3a_1.ancestor, b1_hash);
		assert_eq!(r3a_1.blocks, [b3a_hash.clone(), b2_hash.clone()]);
		assert_eq!(r3a_1.index, 2);

		let r3b_1 = bc.tree_route(b3b_hash.clone(), b1_hash.clone()).unwrap();
		assert_eq!(r3b_1.ancestor, b1_hash);
		assert_eq!(r3b_1.blocks, [b3b_hash.clone(), b2_hash.clone()]);
		assert_eq!(r3b_1.index, 2);

		let r3b_3a = bc.tree_route(b3b_hash.clone(), b3a_hash.clone()).unwrap();
		assert_eq!(r3b_3a.ancestor, b2_hash);
		assert_eq!(r3b_3a.blocks, [b3b_hash.clone(), b3a_hash.clone()]);
		assert_eq!(r3b_3a.index, 1);
	}

	#[test]
	fn test_reopen_blockchain_db() {
		let genesis = "f901fcf901f7a00000000000000000000000000000000000000000000000000000000000000000a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a07dba07d6b448a186e9612e5f737d1c909dce473e53199901a302c00646d523c1a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000080832fefd8808454c98c8142a059262c330941f3fe2a34d16d6e3c7b30d2ceb37c6a0e9a994c494ee1a61d2410885aa4c8bf8e56e264c0c0".from_hex().unwrap();
		let b1 = "f90261f901f9a05716670833ec874362d65fea27a7cd35af5897d275b31a44944113111e4e96d2a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0cb52de543653d86ccd13ba3ddf8b052525b04231c6884a4db3188a184681d878a0e78628dd45a1f8dc495594d83b76c588a3ee67463260f8b7d4a42f574aeab29aa0e9244cf7503b79c03d3a099e07a80d2dbc77bb0b502d8a89d51ac0d68dd31313b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000001832fefd882520884562791e580a051b3ecba4e3f2b49c11d42dd0851ec514b1be3138080f72a2b6e83868275d98f8877671f479c414b47f862f86080018304cb2f94095e7baea6a6c7c4c2dfeb977efac326af552d870a801ca09e2709d7ec9bbe6b1bbbf0b2088828d14cd5e8642a1fee22dc74bfa89761a7f9a04bd8813dee4be989accdb708b1c2e325a7e9c695a8024e30e89d6c644e424747c0".from_hex().unwrap();
		let genesis_hash = H256::from_str("5716670833ec874362d65fea27a7cd35af5897d275b31a44944113111e4e96d2").unwrap();
		let b1_hash = H256::from_str("437e51676ff10756fcfee5edd9159fa41dbcb1b2c592850450371cbecd54ee4f").unwrap();

		let temp = RandomTempPath::new();
		{
			let bc = BlockChain::new(&genesis, temp.as_path());
			assert_eq!(bc.best_block_hash(), genesis_hash);
			bc.insert_block(&b1);
			assert_eq!(bc.best_block_hash(), b1_hash);
		}

		{
			let bc = BlockChain::new(&genesis, temp.as_path());
			assert_eq!(bc.best_block_hash(), b1_hash);
		}
	}

	#[test]
	fn can_contain_arbitrary_block_sequence() {
		let bc_result = generate_dummy_blockchain(50);
		let bc = bc_result.reference();
		assert_eq!(bc.best_block_number(), 49);
	}

	#[test]
	fn can_collect_garbage() {
		let bc_result = generate_dummy_blockchain(3000);
		let bc = bc_result.reference();

		assert_eq!(bc.best_block_number(), 2999);
		let best_hash = bc.best_block_hash();
		let mut block_header = bc.block_header(&best_hash);

		while !block_header.is_none() {
			block_header = bc.block_header(&block_header.unwrap().parent_hash);
		}
		assert!(bc.cache_size().blocks > 1024 * 1024);

		for _ in 0..2 {
			bc.collect_garbage();
		}
		assert!(bc.cache_size().blocks < 1024 * 1024);
	}

	#[test]
	fn can_contain_arbitrary_block_sequence_with_extra() {
		let bc_result = generate_dummy_blockchain_with_extra(25);
		let bc = bc_result.reference();
		assert_eq!(bc.best_block_number(), 24);
	}

	#[test]
	fn can_contain_only_genesis_block() {
		let bc_result = generate_dummy_empty_blockchain();
		let bc = bc_result.reference();
		assert_eq!(bc.best_block_number(), 0);
	}
}
