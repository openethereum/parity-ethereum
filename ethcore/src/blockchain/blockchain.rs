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

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrder};
use bloomchain as bc;
use util::*;
use header::*;
use super::extras::*;
use transaction::*;
use views::*;
use receipt::Receipt;
use blooms::{Bloom, BloomGroup};
use blockchain::block_info::{BlockInfo, BlockLocation, BranchBecomingCanonChainData};
use blockchain::best_block::BestBlock;
use types::tree_route::TreeRoute;
use blockchain::update::ExtrasUpdate;
use blockchain::{CacheSize, ImportRoute, Config};
use db::{Writable, Readable, CacheUpdatePolicy};

const LOG_BLOOMS_LEVELS: usize = 3;
const LOG_BLOOMS_ELEMENTS_PER_INDEX: usize = 16;

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

	/// Get the address of transaction with given hash.
	fn transaction_address(&self, hash: &H256) -> Option<TransactionAddress>;

	/// Get receipts of block with given hash.
	fn block_receipts(&self, hash: &H256) -> Option<BlockReceipts>;

	/// Get the partial-header of a block.
	fn block_header(&self, hash: &H256) -> Option<Header> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).header())
	}

	/// Get a list of uncles for a given block.
	/// Returns None if block does not exist.
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

	/// Get transaction with given transaction hash.
	fn transaction(&self, address: &TransactionAddress) -> Option<LocalizedTransaction> {
		self.block(&address.block_hash).and_then(|bytes| BlockView::new(&bytes).localized_transaction_at(address.index))
	}

	/// Get transaction receipt.
	fn transaction_receipt(&self, address: &TransactionAddress) -> Option<Receipt> {
		self.block_receipts(&address.block_hash).and_then(|br| br.receipts.into_iter().nth(address.index))
	}

	/// Get a list of transactions for a given block.
	/// Returns None if block does not exist.
	fn transactions(&self, hash: &H256) -> Option<Vec<LocalizedTransaction>> {
		self.block(hash).map(|bytes| BlockView::new(&bytes).localized_transactions())
	}

	/// Returns reference to genesis hash.
	fn genesis_hash(&self) -> H256 {
		self.block_hash(0).expect("Genesis hash should always exist")
	}

	/// Returns the header of the genesis block.
	fn genesis_header(&self) -> Header {
		self.block_header(&self.genesis_hash()).unwrap()
	}

	/// Returns numbers of blocks containing given bloom.
	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockNumber, to_block: BlockNumber) -> Vec<BlockNumber>;
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum CacheID {
	Block(H256),
	BlockDetails(H256),
	BlockHashes(BlockNumber),
	TransactionAddresses(H256),
	BlocksBlooms(LogGroupPosition),
	BlockReceipts(H256),
}

struct CacheManager {
	cache_usage: VecDeque<HashSet<CacheID>>,
	in_use: HashSet<CacheID>,
}

impl bc::group::BloomGroupDatabase for BlockChain {
	fn blooms_at(&self, position: &bc::group::GroupPosition) -> Option<bc::group::BloomGroup> {
		let position = LogGroupPosition::from(position.clone());
		self.note_used(CacheID::BlocksBlooms(position.clone()));
		self.extras_db.read_with_cache(&self.blocks_blooms, &position).map(Into::into)
	}
}

/// Structure providing fast access to blockchain data.
///
/// **Does not do input data verification.**
pub struct BlockChain {
	// All locks must be captured in the order declared here.
	pref_cache_size: AtomicUsize,
	max_cache_size: AtomicUsize,
	blooms_config: bc::Config,

	best_block: RwLock<BestBlock>,

	// block cache
	blocks: RwLock<HashMap<H256, Bytes>>,

	// extra caches
	block_details: RwLock<HashMap<H256, BlockDetails>>,
	block_hashes: RwLock<HashMap<BlockNumber, H256>>,
	transaction_addresses: RwLock<HashMap<H256, TransactionAddress>>,
	blocks_blooms: RwLock<HashMap<LogGroupPosition, BloomGroup>>,
	block_receipts: RwLock<HashMap<H256, BlockReceipts>>,

	extras_db: Database,
	blocks_db: Database,

	cache_man: RwLock<CacheManager>,

	insert_lock: Mutex<()>
}

impl BlockProvider for BlockChain {
	/// Returns true if the given block is known
	/// (though not necessarily a part of the canon chain).
	fn is_known(&self, hash: &H256) -> bool {
		self.extras_db.exists_with_cache(&self.block_details, hash)
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
		self.note_used(CacheID::BlockDetails(hash.clone()));
		self.extras_db.read_with_cache(&self.block_details, hash)
	}

	/// Get the hash of given block's number.
	fn block_hash(&self, index: BlockNumber) -> Option<H256> {
		self.note_used(CacheID::BlockHashes(index));
		self.extras_db.read_with_cache(&self.block_hashes, &index)
	}

	/// Get the address of transaction with given hash.
	fn transaction_address(&self, hash: &H256) -> Option<TransactionAddress> {
		self.note_used(CacheID::TransactionAddresses(hash.clone()));
		self.extras_db.read_with_cache(&self.transaction_addresses, hash)
	}

	/// Get receipts of block with given hash.
	fn block_receipts(&self, hash: &H256) -> Option<BlockReceipts> {
		self.note_used(CacheID::BlockReceipts(hash.clone()));
		self.extras_db.read_with_cache(&self.block_receipts, hash)
	}

	/// Returns numbers of blocks containing given bloom.
	fn blocks_with_bloom(&self, bloom: &H2048, from_block: BlockNumber, to_block: BlockNumber) -> Vec<BlockNumber> {
		let range = from_block as bc::Number..to_block as bc::Number;
		let chain = bc::group::BloomGroupChain::new(self.blooms_config, self);
		chain.with_bloom(&range, &Bloom::from(bloom.clone()).into())
			.into_iter()
			.map(|b| b as BlockNumber)
			.collect()
	}
}

const COLLECTION_QUEUE_SIZE: usize = 8;

pub struct AncestryIter<'a> {
	current: H256,
	chain: &'a BlockChain,
}

impl<'a> Iterator for AncestryIter<'a> {
	type Item = H256;
	fn next(&mut self) -> Option<H256> {
		if self.current.is_zero() {
			Option::None
		} else {
			let mut n = self.chain.block_details(&self.current).unwrap().parent;
			mem::swap(&mut self.current, &mut n);
			Some(n)
		}
	}
}

impl BlockChain {
	/// Create new instance of blockchain from given Genesis
	pub fn new(config: Config, genesis: &[u8], path: &Path) -> BlockChain {
		// open extras db
		let mut extras_path = path.to_path_buf();
		extras_path.push("extras");
		let extras_db = Database::open_default(extras_path.to_str().unwrap()).unwrap();

		// open blocks db
		let mut blocks_path = path.to_path_buf();
		blocks_path.push("blocks");
		let blocks_db = Database::open_default(blocks_path.to_str().unwrap()).unwrap();

		let mut cache_man = CacheManager{cache_usage: VecDeque::new(), in_use: HashSet::new()};
		(0..COLLECTION_QUEUE_SIZE).foreach(|_| cache_man.cache_usage.push_back(HashSet::new()));

		let bc = BlockChain {
			pref_cache_size: AtomicUsize::new(config.pref_cache_size),
			max_cache_size: AtomicUsize::new(config.max_cache_size),
			blooms_config: bc::Config {
				levels: LOG_BLOOMS_LEVELS,
				elements_per_index: LOG_BLOOMS_ELEMENTS_PER_INDEX,
			},
			best_block: RwLock::new(BestBlock::default()),
			blocks: RwLock::new(HashMap::new()),
			block_details: RwLock::new(HashMap::new()),
			block_hashes: RwLock::new(HashMap::new()),
			transaction_addresses: RwLock::new(HashMap::new()),
			blocks_blooms: RwLock::new(HashMap::new()),
			block_receipts: RwLock::new(HashMap::new()),
			extras_db: extras_db,
			blocks_db: blocks_db,
			cache_man: RwLock::new(cache_man),
			insert_lock: Mutex::new(()),
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

				let batch = DBTransaction::new();
				batch.write(&hash, &details);
				batch.write(&header.number(), &hash);
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
	pub fn configure_cache(&self, pref_cache_size: usize, max_cache_size: usize) {
		self.pref_cache_size.store(pref_cache_size, AtomicOrder::Relaxed);
		self.max_cache_size.store(max_cache_size, AtomicOrder::Relaxed);
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
	pub fn tree_route(&self, from: H256, to: H256) -> TreeRoute {
		let mut from_branch = vec![];
		let mut to_branch = vec![];

		let mut from_details = self.block_details(&from).expect(&format!("0. Expected to find details for block {:?}", from));
		let mut to_details = self.block_details(&to).expect(&format!("1. Expected to find details for block {:?}", to));
		let mut current_from = from;
		let mut current_to = to;

		// reset from && to to the same level
		while from_details.number > to_details.number {
			from_branch.push(current_from);
			current_from = from_details.parent.clone();
			from_details = self.block_details(&from_details.parent).expect(&format!("2. Expected to find details for block {:?}", from_details.parent));
		}

		while to_details.number > from_details.number {
			to_branch.push(current_to);
			current_to = to_details.parent.clone();
			to_details = self.block_details(&to_details.parent).expect(&format!("3. Expected to find details for block {:?}", to_details.parent));
		}

		assert_eq!(from_details.number, to_details.number);

		// move to shared parent
		while current_from != current_to {
			from_branch.push(current_from);
			current_from = from_details.parent.clone();
			from_details = self.block_details(&from_details.parent).expect(&format!("4. Expected to find details for block {:?}", from_details.parent));

			to_branch.push(current_to);
			current_to = to_details.parent.clone();
			to_details = self.block_details(&to_details.parent).expect(&format!("5. Expected to find details for block {:?}", from_details.parent));
		}

		let index = from_branch.len();

		from_branch.extend(to_branch.into_iter().rev());

		TreeRoute {
			blocks: from_branch,
			ancestor: current_from,
			index: index
		}
	}

	#[cfg_attr(feature="dev", allow(similar_names))]
	/// Inserts the block into backing cache database.
	/// Expects the block to be valid and already verified.
	/// If the block is already known, does nothing.
	pub fn insert_block(&self, bytes: &[u8], receipts: Vec<Receipt>) -> ImportRoute {
		// create views onto rlp
		let block = BlockView::new(bytes);
		let header = block.header_view();
		let hash = header.sha3();

		if self.is_known(&hash) {
			return ImportRoute::none();
		}

		let _lock = self.insert_lock.lock();
		// store block in db
		self.blocks_db.put(&hash, &bytes).unwrap();

		let info = self.block_info(bytes);

		self.apply_update(ExtrasUpdate {
			block_hashes: self.prepare_block_hashes_update(bytes, &info),
			block_details: self.prepare_block_details_update(bytes, &info),
			block_receipts: self.prepare_block_receipts_update(receipts, &info),
			transactions_addresses: self.prepare_transaction_addresses_update(bytes, &info),
			blocks_blooms: self.prepare_block_blooms_update(bytes, &info),
			info: info.clone(),
		});

		ImportRoute::from(info)
	}

	/// Applies extras update.
	fn apply_update(&self, update: ExtrasUpdate) {
		let batch = DBTransaction::new();
		batch.put(b"best", &update.info.hash).unwrap();

		{
			for hash in update.block_details.keys().cloned() {
				self.note_used(CacheID::BlockDetails(hash));
			}

			let mut write_details = self.block_details.write().unwrap();
			batch.extend_with_cache(write_details.deref_mut(), update.block_details, CacheUpdatePolicy::Overwrite);
		}

		{
			let mut write_receipts = self.block_receipts.write().unwrap();
			batch.extend_with_cache(write_receipts.deref_mut(), update.block_receipts, CacheUpdatePolicy::Remove);
		}

		{
			let mut write_blocks_blooms = self.blocks_blooms.write().unwrap();
			batch.extend_with_cache(write_blocks_blooms.deref_mut(), update.blocks_blooms, CacheUpdatePolicy::Remove);
		}

		// These cached values must be updated last and togeterh
		{
			let mut best_block = self.best_block.write().unwrap();
			let mut write_hashes = self.block_hashes.write().unwrap();
			let mut write_txs = self.transaction_addresses.write().unwrap();

			// update best block
			match update.info.location {
				BlockLocation::Branch => (),
				_ => {
					*best_block = BestBlock {
						hash: update.info.hash,
						number: update.info.number,
						total_difficulty: update.info.total_difficulty
					};
				}
			}

			batch.extend_with_cache(write_hashes.deref_mut(), update.block_hashes, CacheUpdatePolicy::Remove);
			batch.extend_with_cache(write_txs.deref_mut(), update.transactions_addresses, CacheUpdatePolicy::Remove);

			// update extras database
			self.extras_db.write(batch).unwrap();
		}
	}

	/// Iterator that lists `first` and then all of `first`'s ancestors, by hash.
	pub fn ancestry_iter(&self, first: H256) -> Option<AncestryIter> {
		if self.is_known(&first) {
			Some(AncestryIter {
				current: first,
				chain: &self,
			})
		} else {
			None
		}
	}

	/// Given a block's `parent`, find every block header which represents a valid possible uncle.
	pub fn find_uncle_headers(&self, parent: &H256, uncle_generations: usize) -> Option<Vec<Header>> {
		self.find_uncle_hashes(parent, uncle_generations).map(|v| v.into_iter().filter_map(|h| self.block_header(&h)).collect())
	}

	/// Given a block's `parent`, find every block hash which represents a valid possible uncle.
	pub fn find_uncle_hashes(&self, parent: &H256, uncle_generations: usize) -> Option<Vec<H256>> {
		if !self.is_known(parent) { return None; }

		let mut excluded = HashSet::new();
		for a in self.ancestry_iter(parent.clone()).unwrap().take(uncle_generations) {
			excluded.extend(self.uncle_hashes(&a).unwrap().into_iter());
			excluded.insert(a);
		}

		let mut ret = Vec::new();
		for a in self.ancestry_iter(parent.clone()).unwrap().skip(1).take(uncle_generations) {
			ret.extend(self.block_details(&a).unwrap().children.iter()
				.filter(|h| !excluded.contains(h))
			);
		}
		Some(ret)
	}

	/// Get inserted block info which is critical to prepare extras updates.
	fn block_info(&self, block_bytes: &[u8]) -> BlockInfo {
		let block = BlockView::new(block_bytes);
		let header = block.header_view();
		let hash = block.sha3();
		let number = header.number();
		let parent_hash = header.parent_hash();
		let parent_details = self.block_details(&parent_hash).expect(format!("Invalid parent hash: {:?}", parent_hash).as_ref());
		let total_difficulty = parent_details.total_difficulty + header.difficulty();
		let is_new_best = total_difficulty > self.best_block_total_difficulty();

		BlockInfo {
			hash: hash,
			number: number,
			total_difficulty: total_difficulty,
			location: if is_new_best {
				// on new best block we need to make sure that all ancestors
				// are moved to "canon chain"
				// find the route between old best block and the new one
				let best_hash = self.best_block_hash();
				let route = self.tree_route(best_hash, parent_hash);

				assert_eq!(number, parent_details.number + 1);

				match route.blocks.len() {
					0 => BlockLocation::CanonChain,
					_ => {
						let retracted = route.blocks.iter().take(route.index).cloned().collect::<Vec<H256>>();

						BlockLocation::BranchBecomingCanonChain(BranchBecomingCanonChainData {
							ancestor: route.ancestor,
							enacted: route.blocks.into_iter().skip(route.index).collect(),
							retracted: retracted.into_iter().rev().collect(),
						})
					}
				}
			} else {
				BlockLocation::Branch
			}
		}
	}

	/// This function returns modified block hashes.
	fn prepare_block_hashes_update(&self, block_bytes: &[u8], info: &BlockInfo) -> HashMap<BlockNumber, H256> {
		let mut block_hashes = HashMap::new();
		let block = BlockView::new(block_bytes);
		let header = block.header_view();
		let number = header.number();

		match info.location {
			BlockLocation::Branch => (),
			BlockLocation::CanonChain => {
				block_hashes.insert(number, info.hash.clone());
			},
			BlockLocation::BranchBecomingCanonChain(ref data) => {
				let ancestor_number = self.block_number(&data.ancestor).unwrap();
				let start_number = ancestor_number + 1;

				for (index, hash) in data.enacted.iter().cloned().enumerate() {
					block_hashes.insert(start_number + index as BlockNumber, hash);
				}

				block_hashes.insert(number, info.hash.clone());
			}
		}

		block_hashes
	}

	/// This function returns modified block details.
	fn prepare_block_details_update(&self, block_bytes: &[u8], info: &BlockInfo) -> HashMap<H256, BlockDetails> {
		let block = BlockView::new(block_bytes);
		let header = block.header_view();
		let parent_hash = header.parent_hash();

		// update parent
		let mut parent_details = self.block_details(&parent_hash).expect(format!("Invalid parent hash: {:?}", parent_hash).as_ref());
		parent_details.children.push(info.hash.clone());

		// create current block details
		let details = BlockDetails {
			number: header.number(),
			total_difficulty: info.total_difficulty,
			parent: parent_hash.clone(),
			children: vec![]
		};

		// write to batch
		let mut block_details = HashMap::new();
		block_details.insert(parent_hash, parent_details);
		block_details.insert(info.hash.clone(), details);
		block_details
	}

	/// This function returns modified block receipts.
	fn prepare_block_receipts_update(&self, receipts: Vec<Receipt>, info: &BlockInfo) -> HashMap<H256, BlockReceipts> {
		let mut block_receipts = HashMap::new();
		block_receipts.insert(info.hash.clone(), BlockReceipts::new(receipts));
		block_receipts
	}

	/// This function returns modified transaction addresses.
	fn prepare_transaction_addresses_update(&self, block_bytes: &[u8], info: &BlockInfo) -> HashMap<H256, TransactionAddress> {
		let block = BlockView::new(block_bytes);
		let transaction_hashes = block.transaction_hashes();

		transaction_hashes.into_iter()
			.enumerate()
			.fold(HashMap::new(), |mut acc, (i ,tx_hash)| {
				acc.insert(tx_hash, TransactionAddress {
					block_hash: info.hash.clone(),
					index: i
				});
				acc
			})
	}

	/// This functions returns modified blocks blooms.
	///
	/// To accelerate blooms lookups, blomms are stored in multiple
	/// layers (BLOOM_LEVELS, currently 3).
	/// ChainFilter is responsible for building and rebuilding these layers.
	/// It returns them in HashMap, where values are Blooms and
	/// keys are BloomIndexes. BloomIndex represents bloom location on one
	/// of these layers.
	///
	/// To reduce number of queries to databse, block blooms are stored
	/// in BlocksBlooms structure which contains info about several
	/// (BLOOM_INDEX_SIZE, currently 16) consecutive blocks blooms.
	///
	/// Later, BloomIndexer is used to map bloom location on filter layer (BloomIndex)
	/// to bloom location in database (BlocksBloomLocation).
	///
	fn prepare_block_blooms_update(&self, block_bytes: &[u8], info: &BlockInfo) -> HashMap<LogGroupPosition, BloomGroup> {
		let block = BlockView::new(block_bytes);
		let header = block.header_view();

		let log_blooms = match info.location {
			BlockLocation::Branch => HashMap::new(),
			BlockLocation::CanonChain => {
				let chain = bc::group::BloomGroupChain::new(self.blooms_config, self);
				chain.insert(info.number as bc::Number, Bloom::from(header.log_bloom()).into())
			},
			BlockLocation::BranchBecomingCanonChain(ref data) => {
				let ancestor_number = self.block_number(&data.ancestor).unwrap();
				let start_number = ancestor_number + 1;
				let range = start_number as bc::Number..self.best_block_number() as bc::Number;

				let mut blooms: Vec<bc::Bloom> = data.enacted.iter()
					.map(|hash| self.block(hash).unwrap())
					.map(|bytes| BlockView::new(&bytes).header_view().log_bloom())
					.map(Bloom::from)
					.map(Into::into)
					.collect();

				blooms.push(Bloom::from(header.log_bloom()).into());

				let chain = bc::group::BloomGroupChain::new(self.blooms_config, self);
				chain.replace(&range, blooms)
			}
		};

		log_blooms.into_iter()
			.map(|p| (From::from(p.0), From::from(p.1)))
			.collect()
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

	/// Get current cache size.
	pub fn cache_size(&self) -> CacheSize {
		CacheSize {
			blocks: self.blocks.read().unwrap().heap_size_of_children(),
			block_details: self.block_details.read().unwrap().heap_size_of_children(),
			transaction_addresses: self.transaction_addresses.read().unwrap().heap_size_of_children(),
			blocks_blooms: self.blocks_blooms.read().unwrap().heap_size_of_children(),
			block_receipts: self.block_receipts.read().unwrap().heap_size_of_children(),
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
		if self.cache_size().total() < self.pref_cache_size.load(AtomicOrder::Relaxed) { return; }

		for _ in 0..COLLECTION_QUEUE_SIZE {
			{
				let mut blocks = self.blocks.write().unwrap();
				let mut block_details = self.block_details.write().unwrap();
				let mut block_hashes = self.block_hashes.write().unwrap();
				let mut transaction_addresses = self.transaction_addresses.write().unwrap();
				let mut blocks_blooms = self.blocks_blooms.write().unwrap();
				let mut block_receipts = self.block_receipts.write().unwrap();
				let mut cache_man = self.cache_man.write().unwrap();

				for id in cache_man.cache_usage.pop_back().unwrap().into_iter() {
					cache_man.in_use.remove(&id);
					match id {
						CacheID::Block(h) => { blocks.remove(&h); },
						CacheID::BlockDetails(h) => { block_details.remove(&h); }
						CacheID::BlockHashes(h) => { block_hashes.remove(&h); }
						CacheID::TransactionAddresses(h) => { transaction_addresses.remove(&h); }
						CacheID::BlocksBlooms(h) => { blocks_blooms.remove(&h); }
						CacheID::BlockReceipts(h) => { block_receipts.remove(&h); }
					}
				}
				cache_man.cache_usage.push_front(HashSet::new());

				// TODO: handle block_hashes properly.
				block_hashes.clear();

				blocks.shrink_to_fit();
				block_details.shrink_to_fit();
 				block_hashes.shrink_to_fit();
 				transaction_addresses.shrink_to_fit();
 				blocks_blooms.shrink_to_fit();
 				block_receipts.shrink_to_fit();
			}
			if self.cache_size().total() < self.max_cache_size.load(AtomicOrder::Relaxed) { break; }
		}

		// TODO: m_lastCollection = chrono::system_clock::now();
	}
}

#[cfg(test)]
mod tests {
	#![cfg_attr(feature="dev", allow(similar_names))]
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;
	use util::hash::*;
	use util::sha3::Hashable;
	use blockchain::{BlockProvider, BlockChain, Config, ImportRoute};
	use tests::helpers::*;
	use devtools::*;
	use blockchain::generator::{ChainGenerator, ChainIterator, BlockFinalizer};
	use views::BlockView;

	#[test]
	fn basic_blockchain_insert() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let first = canon_chain.generate(&mut finalizer).unwrap();
		let genesis_hash = BlockView::new(&genesis).header_view().sha3();
		let first_hash = BlockView::new(&first).header_view().sha3();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());

		assert_eq!(bc.genesis_hash(), genesis_hash.clone());
		assert_eq!(bc.best_block_number(), 0);
		assert_eq!(bc.best_block_hash(), genesis_hash.clone());
		assert_eq!(bc.block_hash(0), Some(genesis_hash.clone()));
		assert_eq!(bc.block_hash(1), None);
		assert_eq!(bc.block_details(&genesis_hash).unwrap().children, vec![]);

		bc.insert_block(&first, vec![]);

		assert_eq!(bc.block_hash(0), Some(genesis_hash.clone()));
		assert_eq!(bc.best_block_number(), 1);
		assert_eq!(bc.best_block_hash(), first_hash.clone());
		assert_eq!(bc.block_hash(1), Some(first_hash.clone()));
		assert_eq!(bc.block_details(&first_hash).unwrap().parent, genesis_hash.clone());
		assert_eq!(bc.block_details(&genesis_hash).unwrap().children, vec![first_hash.clone()]);
		assert_eq!(bc.block_hash(2), None);
	}

	#[test]
	fn check_ancestry_iter() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let genesis_hash = BlockView::new(&genesis).header_view().sha3();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());

		let mut block_hashes = vec![genesis_hash.clone()];
		for _ in 0..10 {
			let block = canon_chain.generate(&mut finalizer).unwrap();
			block_hashes.push(BlockView::new(&block).header_view().sha3());
			bc.insert_block(&block, vec![]);
		}

		block_hashes.reverse();

		assert_eq!(bc.ancestry_iter(block_hashes[0].clone()).unwrap().collect::<Vec<_>>(), block_hashes)
	}

	#[test]
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn test_find_uncles() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let b1b = canon_chain.fork(1).generate(&mut finalizer.fork()).unwrap();
		let b1a = canon_chain.generate(&mut finalizer).unwrap();
		let b2b = canon_chain.fork(1).generate(&mut finalizer.fork()).unwrap();
		let b2a = canon_chain.generate(&mut finalizer).unwrap();
		let b3b = canon_chain.fork(1).generate(&mut finalizer.fork()).unwrap();
		let b3a = canon_chain.generate(&mut finalizer).unwrap();
		let b4b = canon_chain.fork(1).generate(&mut finalizer.fork()).unwrap();
		let b4a = canon_chain.generate(&mut finalizer).unwrap();
		let b5b = canon_chain.fork(1).generate(&mut finalizer.fork()).unwrap();
		let b5a = canon_chain.generate(&mut finalizer).unwrap();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());
		bc.insert_block(&b1a, vec![]);
		bc.insert_block(&b1b, vec![]);
		bc.insert_block(&b2a, vec![]);
		bc.insert_block(&b2b, vec![]);
		bc.insert_block(&b3a, vec![]);
		bc.insert_block(&b3b, vec![]);
		bc.insert_block(&b4a, vec![]);
		bc.insert_block(&b4b, vec![]);
		bc.insert_block(&b5a, vec![]);
		bc.insert_block(&b5b, vec![]);

		assert_eq!(
			[&b4b, &b3b, &b2b].iter().map(|b| BlockView::new(b).header()).collect::<Vec<_>>(),
			bc.find_uncle_headers(&BlockView::new(&b4a).header_view().sha3(), 3).unwrap()
		);

		// TODO: insert block that already includes one of them as an uncle to check it's not allowed.
	}

	#[test]
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn test_small_fork() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let b1 = canon_chain.generate(&mut finalizer).unwrap();
		let b2 = canon_chain.generate(&mut finalizer).unwrap();
		let b3b = canon_chain.fork(1).generate(&mut finalizer.fork()).unwrap();
		let b3a = canon_chain.generate(&mut finalizer).unwrap();

		let genesis_hash = BlockView::new(&genesis).header_view().sha3();
		let b1_hash= BlockView::new(&b1).header_view().sha3();
		let b2_hash= BlockView::new(&b2).header_view().sha3();
		let b3a_hash= BlockView::new(&b3a).header_view().sha3();
		let b3b_hash= BlockView::new(&b3b).header_view().sha3();

		// b3a is a part of canon chain, whereas b3b is part of sidechain
		let best_block_hash = b3a_hash.clone();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());
		let ir1 = bc.insert_block(&b1, vec![]);
		let ir2 = bc.insert_block(&b2, vec![]);
		let ir3b = bc.insert_block(&b3b, vec![]);
		let ir3a = bc.insert_block(&b3a, vec![]);

		assert_eq!(ir1, ImportRoute {
			enacted: vec![b1_hash],
			retracted: vec![],
			omitted: vec![],
		});

		assert_eq!(ir2, ImportRoute {
			enacted: vec![b2_hash],
			retracted: vec![],
			omitted: vec![],
		});

		assert_eq!(ir3b, ImportRoute {
			enacted: vec![b3b_hash],
			retracted: vec![],
			omitted: vec![],
		});

		assert_eq!(ir3a, ImportRoute {
			enacted: vec![b3a_hash],
			retracted: vec![b3b_hash],
			omitted: vec![],
		});

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
		let r0_1 = bc.tree_route(genesis_hash.clone(), b1_hash.clone());
		assert_eq!(r0_1.ancestor, genesis_hash);
		assert_eq!(r0_1.blocks, [b1_hash.clone()]);
		assert_eq!(r0_1.index, 0);

		let r0_2 = bc.tree_route(genesis_hash.clone(), b2_hash.clone());
		assert_eq!(r0_2.ancestor, genesis_hash);
		assert_eq!(r0_2.blocks, [b1_hash.clone(), b2_hash.clone()]);
		assert_eq!(r0_2.index, 0);

		let r1_3a = bc.tree_route(b1_hash.clone(), b3a_hash.clone());
		assert_eq!(r1_3a.ancestor, b1_hash);
		assert_eq!(r1_3a.blocks, [b2_hash.clone(), b3a_hash.clone()]);
		assert_eq!(r1_3a.index, 0);

		let r1_3b = bc.tree_route(b1_hash.clone(), b3b_hash.clone());
		assert_eq!(r1_3b.ancestor, b1_hash);
		assert_eq!(r1_3b.blocks, [b2_hash.clone(), b3b_hash.clone()]);
		assert_eq!(r1_3b.index, 0);

		let r3a_3b = bc.tree_route(b3a_hash.clone(), b3b_hash.clone());
		assert_eq!(r3a_3b.ancestor, b2_hash);
		assert_eq!(r3a_3b.blocks, [b3a_hash.clone(), b3b_hash.clone()]);
		assert_eq!(r3a_3b.index, 1);

		let r1_0 = bc.tree_route(b1_hash.clone(), genesis_hash.clone());
		assert_eq!(r1_0.ancestor, genesis_hash);
		assert_eq!(r1_0.blocks, [b1_hash.clone()]);
		assert_eq!(r1_0.index, 1);

		let r2_0 = bc.tree_route(b2_hash.clone(), genesis_hash.clone());
		assert_eq!(r2_0.ancestor, genesis_hash);
		assert_eq!(r2_0.blocks, [b2_hash.clone(), b1_hash.clone()]);
		assert_eq!(r2_0.index, 2);

		let r3a_1 = bc.tree_route(b3a_hash.clone(), b1_hash.clone());
		assert_eq!(r3a_1.ancestor, b1_hash);
		assert_eq!(r3a_1.blocks, [b3a_hash.clone(), b2_hash.clone()]);
		assert_eq!(r3a_1.index, 2);

		let r3b_1 = bc.tree_route(b3b_hash.clone(), b1_hash.clone());
		assert_eq!(r3b_1.ancestor, b1_hash);
		assert_eq!(r3b_1.blocks, [b3b_hash.clone(), b2_hash.clone()]);
		assert_eq!(r3b_1.index, 2);

		let r3b_3a = bc.tree_route(b3b_hash.clone(), b3a_hash.clone());
		assert_eq!(r3b_3a.ancestor, b2_hash);
		assert_eq!(r3b_3a.blocks, [b3b_hash.clone(), b3a_hash.clone()]);
		assert_eq!(r3b_3a.index, 1);
	}

	#[test]
	fn test_reopen_blockchain_db() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let first = canon_chain.generate(&mut finalizer).unwrap();
		let genesis_hash = BlockView::new(&genesis).header_view().sha3();
		let first_hash = BlockView::new(&first).header_view().sha3();

		let temp = RandomTempPath::new();
		{
			let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());
			assert_eq!(bc.best_block_hash(), genesis_hash);
			bc.insert_block(&first, vec![]);
			assert_eq!(bc.best_block_hash(), first_hash);
		}

		{
			let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());
			assert_eq!(bc.best_block_hash(), first_hash);
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

	#[test]
	fn find_transaction_by_hash() {
		let genesis = "f901fcf901f7a00000000000000000000000000000000000000000000000000000000000000000a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0af81e09f8c46ca322193edfda764fa7e88e81923f802f1d325ec0b0308ac2cd0a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000830200008083023e38808454c98c8142a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421880102030405060708c0c0".from_hex().unwrap();
		let b1 = "f904a8f901faa0ce1f26f798dd03c8782d63b3e42e79a64eaea5694ea686ac5d7ce3df5171d1aea01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0a65c2364cd0f1542d761823dc0109c6b072f14c20459598c5455c274601438f4a070616ebd7ad2ed6fb7860cf7e9df00163842351c38a87cac2c1cb193895035a2a05c5b4fc43c2d45787f54e1ae7d27afdb4ad16dfc567c5692070d5c4556e0b1d7b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000830200000183023ec683021536845685109780a029f07836e4e59229b3a065913afc27702642c683bba689910b2b2fd45db310d3888957e6d004a31802f902a7f85f800a8255f094aaaf5374fce5edbc8e2a8697c15331677e6ebf0b0a801ca0575da4e21b66fa764be5f74da9389e67693d066fb0d1312e19e17e501da00ecda06baf5a5327595f6619dfc2fcb3f2e6fb410b5810af3cb52d0e7508038e91a188f85f010a82520894bbbf5374fce5edbc8e2a8697c15331677e6ebf0b0a801ba04fa966bf34b93abc1bcd665554b7f316b50f928477b50be0f3285ead29d18c5ba017bba0eeec1625ab433746955e125d46d80b7fdc97386c51266f842d8e02192ef85f020a82520894bbbf5374fce5edbc8e2a8697c15331677e6ebf0b0a801ca004377418ae981cc32b1312b4a427a1d69a821b28db8584f5f2bd8c6d42458adaa053a1dba1af177fac92f3b6af0a9fa46a22adf56e686c93794b6a012bf254abf5f85f030a82520894bbbf5374fce5edbc8e2a8697c15331677e6ebf0b0a801ca04fe13febd28a05f4fcb2f451d7ddc2dda56486d9f8c79a62b0ba4da775122615a0651b2382dd402df9ebc27f8cb4b2e0f3cea68dda2dca0ee9603608f0b6f51668f85f040a82520894bbbf5374fce5edbc8e2a8697c15331677e6ebf0b0a801ba078e6a0ba086a08f8450e208a399bb2f2d2a0d984acd2517c7c7df66ccfab567da013254002cd45a97fac049ae00afbc43ed0d9961d0c56a3b2382c80ce41c198ddf85f050a82520894bbbf5374fce5edbc8e2a8697c15331677e6ebf0b0a801ba0a7174d8f43ea71c8e3ca9477691add8d80ac8e0ed89d8d8b572041eef81f4a54a0534ea2e28ec4da3b5b944b18c51ec84a5cf35f5b3343c5fb86521fd2d388f506f85f060a82520894bbbf5374fce5edbc8e2a8697c15331677e6ebf0b0a801ba034bd04065833536a10c77ee2a43a5371bc6d34837088b861dd9d4b7f44074b59a078807715786a13876d3455716a6b9cb2186b7a4887a5c31160fc877454958616c0".from_hex().unwrap();
		let b1_hash = H256::from_str("f53f268d23a71e85c7d6d83a9504298712b84c1a2ba220441c86eeda0bf0b6e3").unwrap();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());
		bc.insert_block(&b1, vec![]);

		let transactions = bc.transactions(&b1_hash).unwrap();
		assert_eq!(transactions.len(), 7);
		for t in transactions {
			assert_eq!(bc.transaction(&bc.transaction_address(&t.hash()).unwrap()).unwrap(), t);
		}
	}

	#[test]
	fn test_bloom_filter_simple() {
		// TODO: From here
		let bloom_b1 = H2048::from_str("00000020000000000000000000000000000000000000000002000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000400000000000000000000002000").unwrap();

		let bloom_b2 = H2048::from_str("00000000000000000000000000000000000000000000020000001000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000008000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

		let bloom_ba = H2048::from_str("00000000000000000000000000000000000000000000020000000800000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000008000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let mut fork = canon_chain.fork(1);
		let mut fork_finalizer = finalizer.fork();
		let b1 = fork.with_bloom(bloom_b1.clone()).generate(&mut fork_finalizer).unwrap();
		let b2 = fork.with_bloom(bloom_b2.clone()).generate(&mut fork_finalizer).unwrap();
		let b3 = fork.with_bloom(bloom_ba.clone()).generate(&mut fork_finalizer).unwrap();
		let b1a = canon_chain.with_bloom(bloom_ba.clone()).generate(&mut finalizer).unwrap();
		let b2a = canon_chain.with_bloom(bloom_ba.clone()).generate(&mut finalizer).unwrap();

		let temp = RandomTempPath::new();
		let bc = BlockChain::new(Config::default(), &genesis, temp.as_path());

		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		assert_eq!(blocks_b1, vec![]);
		assert_eq!(blocks_b2, vec![]);

		bc.insert_block(&b1, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![]);

		bc.insert_block(&b2, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![2]);

		// hasn't been forked yet
		bc.insert_block(&b1a, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		let blocks_ba = bc.blocks_with_bloom(&bloom_ba, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![2]);
		assert_eq!(blocks_ba, vec![]);

		// fork has happend
		bc.insert_block(&b2a, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		let blocks_ba = bc.blocks_with_bloom(&bloom_ba, 0, 5);
		assert_eq!(blocks_b1, vec![]);
		assert_eq!(blocks_b2, vec![]);
		assert_eq!(blocks_ba, vec![1, 2]);

		// fork back
		bc.insert_block(&b3, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		let blocks_ba = bc.blocks_with_bloom(&bloom_ba, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![2]);
		assert_eq!(blocks_ba, vec![3]);
	}
}
