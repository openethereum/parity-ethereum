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

use bloomchain as bc;
use util::*;
use rlp::*;
use header::*;
use super::extras::*;
use transaction::*;
use views::*;
use log_entry::{LogEntry, LocalizedLogEntry};
use receipt::Receipt;
use blooms::{Bloom, BloomGroup};
use blockchain::block_info::{BlockInfo, BlockLocation, BranchBecomingCanonChainData};
use blockchain::best_block::BestBlock;
use types::tree_route::TreeRoute;
use blockchain::update::ExtrasUpdate;
use blockchain::{CacheSize, ImportRoute, Config};
use db::{self, Writable, Readable, CacheUpdatePolicy};
use cache_manager::CacheManager;

const LOG_BLOOMS_LEVELS: usize = 3;
const LOG_BLOOMS_ELEMENTS_PER_INDEX: usize = 16;

/// Interface for querying blocks by hash and by number.
pub trait BlockProvider {
	/// Returns true if the given block is known
	/// (though not necessarily a part of the canon chain).
	fn is_known(&self, hash: &H256) -> bool;

	/// Get the first block which this chain holds.
	/// Any queries of blocks which precede this one are not guaranteed to
	/// succeed.
	fn first_block(&self) -> H256;

	/// Get the number of the first block.
	fn first_block_number(&self) -> BlockNumber {
		self.block_number(&self.first_block()).expect("First block always stored; qed")
	}

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
		self.block_header_data(hash).map(|header| decode(&header))
	}

	/// Get the header RLP of a block.
	fn block_header_data(&self, hash: &H256) -> Option<Bytes>;

	/// Get the block body (uncles and transactions).
	fn block_body(&self, hash: &H256) -> Option<Bytes>;

	/// Get a list of uncles for a given block.
	/// Returns None if block does not exist.
	fn uncles(&self, hash: &H256) -> Option<Vec<Header>> {
		self.block_body(hash).map(|bytes| BodyView::new(&bytes).uncles())
	}

	/// Get a list of uncle hashes for a given block.
	/// Returns None if block does not exist.
	fn uncle_hashes(&self, hash: &H256) -> Option<Vec<H256>> {
		self.block_body(hash).map(|bytes| BodyView::new(&bytes).uncle_hashes())
	}

	/// Get the number of given block's hash.
	fn block_number(&self, hash: &H256) -> Option<BlockNumber> {
		self.block_details(hash).map(|details| details.number)
	}

	/// Get transaction with given transaction hash.
	fn transaction(&self, address: &TransactionAddress) -> Option<LocalizedTransaction> {
		self.block_body(&address.block_hash)
			.and_then(|bytes| self.block_number(&address.block_hash)
			.and_then(|n| BodyView::new(&bytes).localized_transaction_at(&address.block_hash, n, address.index)))
	}

	/// Get transaction receipt.
	fn transaction_receipt(&self, address: &TransactionAddress) -> Option<Receipt> {
		self.block_receipts(&address.block_hash).and_then(|br| br.receipts.into_iter().nth(address.index))
	}

	/// Get a list of transactions for a given block.
	/// Returns None if block does not exist.
	fn transactions(&self, hash: &H256) -> Option<Vec<LocalizedTransaction>> {
		self.block_body(hash)
			.and_then(|bytes| self.block_number(hash)
			.map(|n| BodyView::new(&bytes).localized_transactions(hash, n)))
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

	/// Returns logs matching given filter.
	fn logs<F>(&self, mut blocks: Vec<BlockNumber>, matches: F, limit: Option<usize>) -> Vec<LocalizedLogEntry>
		where F: Fn(&LogEntry) -> bool, Self: Sized;
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum CacheID {
	BlockHeader(H256),
	BlockBody(H256),
	BlockDetails(H256),
	BlockHashes(BlockNumber),
	TransactionAddresses(H256),
	BlocksBlooms(LogGroupPosition),
	BlockReceipts(H256),
}

impl bc::group::BloomGroupDatabase for BlockChain {
	fn blooms_at(&self, position: &bc::group::GroupPosition) -> Option<bc::group::BloomGroup> {
		let position = LogGroupPosition::from(position.clone());
		let result = self.db.read_with_cache(db::COL_EXTRA, &self.blocks_blooms, &position).map(Into::into);
		self.cache_man.lock().note_used(CacheID::BlocksBlooms(position));
		result
	}
}

/// Structure providing fast access to blockchain data.
///
/// **Does not do input data verification.**
pub struct BlockChain {
	// All locks must be captured in the order declared here.
	blooms_config: bc::Config,
	first_block: H256,

	best_block: RwLock<BestBlock>,

	// block cache
	block_headers: RwLock<HashMap<H256, Bytes>>,
	block_bodies: RwLock<HashMap<H256, Bytes>>,

	// extra caches
	block_details: RwLock<HashMap<H256, BlockDetails>>,
	block_hashes: RwLock<HashMap<BlockNumber, H256>>,
	transaction_addresses: RwLock<HashMap<H256, TransactionAddress>>,
	blocks_blooms: RwLock<HashMap<LogGroupPosition, BloomGroup>>,
	block_receipts: RwLock<HashMap<H256, BlockReceipts>>,

	db: Arc<Database>,

	cache_man: Mutex<CacheManager<CacheID>>,

	pending_best_block: RwLock<Option<BestBlock>>,
	pending_block_hashes: RwLock<HashMap<BlockNumber, H256>>,
	pending_transaction_addresses: RwLock<HashMap<H256, Option<TransactionAddress>>>,
}

impl BlockProvider for BlockChain {
	/// Returns true if the given block is known
	/// (though not necessarily a part of the canon chain).
	fn is_known(&self, hash: &H256) -> bool {
		self.db.exists_with_cache(db::COL_EXTRA, &self.block_details, hash)
	}

	fn first_block(&self) -> H256 {
		self.first_block
	}

	/// Get raw block data
	fn block(&self, hash: &H256) -> Option<Bytes> {
		match (self.block_header_data(hash), self.block_body(hash)) {
			(Some(header), Some(body)) => {
				let mut block = RlpStream::new_list(3);
				let body_rlp = Rlp::new(&body);
				block.append_raw(&header, 1);
				block.append_raw(body_rlp.at(0).as_raw(), 1);
				block.append_raw(body_rlp.at(1).as_raw(), 1);
				Some(block.out())
			},
			_ => None,
		}
	}

	/// Get block header data
	fn block_header_data(&self, hash: &H256) -> Option<Bytes> {
		// Check cache first
		{
			let read = self.block_headers.read();
			if let Some(v) = read.get(hash) {
				return Some(v.clone());
			}
		}

		// Check if it's the best block
		{
			let best_block = self.best_block.read();
			if &best_block.hash == hash {
				return Some(Rlp::new(&best_block.block).at(0).as_raw().to_vec());
			}
		}

		// Read from DB and populate cache
		let opt = self.db.get(db::COL_HEADERS, hash)
			.expect("Low level database error. Some issue with disk?");

		let result = match opt {
			Some(b) => {
				let bytes: Bytes = UntrustedRlp::new(&b).decompress(RlpType::Blocks).to_vec();
				let mut write = self.block_headers.write();
				write.insert(hash.clone(), bytes.clone());
				Some(bytes)
			},
			None => None
		};

		self.cache_man.lock().note_used(CacheID::BlockHeader(hash.clone()));
		result
	}

	/// Get block body data
	fn block_body(&self, hash: &H256) -> Option<Bytes> {
		// Check cache first
		{
			let read = self.block_bodies.read();
			if let Some(v) = read.get(hash) {
				return Some(v.clone());
			}
		}

		// Check if it's the best block
		{
			let best_block = self.best_block.read();
			if &best_block.hash == hash {
				return Some(Self::block_to_body(&best_block.block));
			}
		}

		// Read from DB and populate cache
		let opt = self.db.get(db::COL_BODIES, hash)
			.expect("Low level database error. Some issue with disk?");

		let result = match opt {
			Some(b) => {
				let bytes: Bytes = UntrustedRlp::new(&b).decompress(RlpType::Blocks).to_vec();
				let mut write = self.block_bodies.write();
				write.insert(hash.clone(), bytes.clone());
				Some(bytes)
			},
			None => None
		};

		self.cache_man.lock().note_used(CacheID::BlockBody(hash.clone()));

		result
	}

	/// Get the familial details concerning a block.
	fn block_details(&self, hash: &H256) -> Option<BlockDetails> {
		let result = self.db.read_with_cache(db::COL_EXTRA, &self.block_details, hash);
		self.cache_man.lock().note_used(CacheID::BlockDetails(hash.clone()));
		result
	}

	/// Get the hash of given block's number.
	fn block_hash(&self, index: BlockNumber) -> Option<H256> {
		let result = self.db.read_with_cache(db::COL_EXTRA, &self.block_hashes, &index);
		self.cache_man.lock().note_used(CacheID::BlockHashes(index));
		result
	}

	/// Get the address of transaction with given hash.
	fn transaction_address(&self, hash: &H256) -> Option<TransactionAddress> {
		let result = self.db.read_with_cache(db::COL_EXTRA, &self.transaction_addresses, hash);
		self.cache_man.lock().note_used(CacheID::TransactionAddresses(hash.clone()));
		result
	}

	/// Get receipts of block with given hash.
	fn block_receipts(&self, hash: &H256) -> Option<BlockReceipts> {
		let result = self.db.read_with_cache(db::COL_EXTRA, &self.block_receipts, hash);
		self.cache_man.lock().note_used(CacheID::BlockReceipts(hash.clone()));
		result
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

	fn logs<F>(&self, mut blocks: Vec<BlockNumber>, matches: F, limit: Option<usize>) -> Vec<LocalizedLogEntry>
		where F: Fn(&LogEntry) -> bool, Self: Sized {
		// sort in reverse order
		blocks.sort_by(|a, b| b.cmp(a));

		let mut log_index = 0;
		let mut logs = blocks.into_iter()
			.filter_map(|number| self.block_hash(number).map(|hash| (number, hash)))
			.filter_map(|(number, hash)| self.block_receipts(&hash).map(|r| (number, hash, r.receipts)))
			.filter_map(|(number, hash, receipts)| self.block_body(&hash).map(|ref b| (number, hash, receipts, BodyView::new(b).transaction_hashes())))
			.flat_map(|(number, hash, mut receipts, mut hashes)| {
				if receipts.len() != hashes.len() {
					warn!("Block {} ({}) has different number of receipts ({}) to transactions ({}). Database corrupt?", number, hash, receipts.len(), hashes.len());
					assert!(false);
				}
				log_index = receipts.iter().fold(0, |sum, receipt| sum + receipt.logs.len());

				let receipts_len = receipts.len();
				hashes.reverse();
				receipts.reverse();
				receipts.into_iter()
					.map(|receipt| receipt.logs)
					.zip(hashes)
					.enumerate()
					.flat_map(move |(index, (mut logs, tx_hash))| {
						let current_log_index = log_index;
						log_index -= logs.len();

						logs.reverse();
						logs.into_iter()
							.enumerate()
							.map(move |(i, log)| LocalizedLogEntry {
								entry: log,
								block_hash: hash,
								block_number: number,
								transaction_hash: tx_hash,
								// iterating in reverse order
								transaction_index: receipts_len - index - 1,
								log_index: current_log_index - i - 1,
							})
					})
			})
			.filter(|log_entry| matches(&log_entry.entry))
			.take(limit.unwrap_or(::std::usize::MAX))
			.collect::<Vec<LocalizedLogEntry>>();
		logs.reverse();
		logs
	}
}

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
	pub fn new(config: Config, genesis: &[u8], db: Arc<Database>) -> BlockChain {
		// 400 is the avarage size of the key
		let cache_man = CacheManager::new(config.pref_cache_size, config.max_cache_size, 400);

		let mut bc = BlockChain {
			blooms_config: bc::Config {
				levels: LOG_BLOOMS_LEVELS,
				elements_per_index: LOG_BLOOMS_ELEMENTS_PER_INDEX,
			},
			first_block: H256::zero(),
			best_block: RwLock::new(BestBlock::default()),
			block_headers: RwLock::new(HashMap::new()),
			block_bodies: RwLock::new(HashMap::new()),
			block_details: RwLock::new(HashMap::new()),
			block_hashes: RwLock::new(HashMap::new()),
			transaction_addresses: RwLock::new(HashMap::new()),
			blocks_blooms: RwLock::new(HashMap::new()),
			block_receipts: RwLock::new(HashMap::new()),
			db: db.clone(),
			cache_man: Mutex::new(cache_man),
			pending_best_block: RwLock::new(None),
			pending_block_hashes: RwLock::new(HashMap::new()),
			pending_transaction_addresses: RwLock::new(HashMap::new()),
		};

		// load best block
		let best_block_hash = match bc.db.get(db::COL_EXTRA, b"best").unwrap() {
			Some(best) => {
				H256::from_slice(&best)
			}
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

				let mut batch = DBTransaction::new(&db);
				batch.put(db::COL_HEADERS, &hash, block.header_rlp().as_raw());
				batch.put(db::COL_BODIES, &hash, &Self::block_to_body(genesis));

				batch.write(db::COL_EXTRA, &hash, &details);
				batch.write(db::COL_EXTRA, &header.number(), &hash);

				batch.put(db::COL_EXTRA, b"best", &hash);
				batch.put(db::COL_EXTRA, b"first", &hash);
				bc.db.write(batch).expect("Low level database error. Some issue with disk?");
				hash
			}
		};

		{
			// Fetch best block details
			let best_block_number = bc.block_number(&best_block_hash).unwrap();
			let best_block_total_difficulty = bc.block_details(&best_block_hash).unwrap().total_difficulty;
			let best_block_rlp = bc.block(&best_block_hash).unwrap();

			let raw_first = bc.db.get(db::COL_EXTRA, b"first").unwrap().map_or(Vec::new(), |v| v.to_vec());

			// binary search for the first block.
			if raw_first.is_empty() {
				let (mut f, mut hash) = (best_block_number, best_block_hash);
				let mut l = 0;

				loop {
					if l >= f { break; }

					let step = (f - l) >> 1;
					let m = l + step;

					match bc.block_hash(m) {
						Some(h) => { f = m; hash = h },
						None => { l = m + 1 },
					}
				}

				let mut batch = db.transaction();
				batch.put(db::COL_EXTRA, b"first", &hash);
				db.write(batch).expect("Low level database error.");

				bc.first_block = hash;
			} else {
				bc.first_block = H256::from_slice(&raw_first);
			}

			// and write them
			let mut best_block = bc.best_block.write();
			*best_block = BestBlock {
				number: best_block_number,
				total_difficulty: best_block_total_difficulty,
				hash: best_block_hash,
				block: best_block_rlp,
			};
		}

		bc
	}

	/// Returns true if the given parent block has given child
	/// (though not necessarily a part of the canon chain).
	fn is_known_child(&self, parent: &H256, hash: &H256) -> bool {
		self.db.read_with_cache(db::COL_EXTRA, &self.block_details, parent).map_or(false, |d| d.children.contains(hash))
	}

	/// Rewind to a previous block
	#[cfg(test)]
	fn rewind(&self) -> Option<H256> {
		use db::Key;
		let mut batch =self.db.transaction();
		// track back to the best block we have in the blocks database
		if let Some(best_block_hash) = self.db.get(db::COL_EXTRA, b"best").unwrap() {
			let best_block_hash = H256::from_slice(&best_block_hash);
			if best_block_hash == self.genesis_hash() {
				return None;
			}
			if let Some(extras) = self.db.read(db::COL_EXTRA, &best_block_hash) as Option<BlockDetails> {
				type DetailsKey = Key<BlockDetails, Target=H264>;
				batch.delete(db::COL_EXTRA, &(DetailsKey::key(&best_block_hash)));
				let hash = extras.parent;
				let range = extras.number as bc::Number .. extras.number as bc::Number;
				let chain = bc::group::BloomGroupChain::new(self.blooms_config, self);
				let changes = chain.replace(&range, vec![]);
				for (k, v) in changes.into_iter() {
					batch.write(db::COL_EXTRA, &LogGroupPosition::from(k), &BloomGroup::from(v));
				}
				batch.put(db::COL_EXTRA, b"best", &hash);

				let best_block_total_difficulty = self.block_details(&hash).unwrap().total_difficulty;
				let best_block_rlp = self.block(&hash).unwrap();

				let mut best_block = self.best_block.write();
				*best_block = BestBlock {
					number: extras.number - 1,
					total_difficulty: best_block_total_difficulty,
					hash: hash,
					block: best_block_rlp,
				};
				// update parent extras
				if let Some(mut details) = self.db.read(db::COL_EXTRA, &hash) as Option<BlockDetails> {
					details.children.clear();
					batch.write(db::COL_EXTRA, &hash, &details);
				}
				self.db.write(batch).expect("Writing to db failed");
				self.block_details.write().clear();
				self.block_hashes.write().clear();
				self.block_headers.write().clear();
				self.block_bodies.write().clear();
				self.block_receipts.write().clear();
				return Some(hash);
			}
		}

		None
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

		let mut from_details = self.block_details(&from).unwrap_or_else(|| panic!("0. Expected to find details for block {:?}", from));
		let mut to_details = self.block_details(&to).unwrap_or_else(|| panic!("1. Expected to find details for block {:?}", to));
		let mut current_from = from;
		let mut current_to = to;

		// reset from && to to the same level
		while from_details.number > to_details.number {
			from_branch.push(current_from);
			current_from = from_details.parent.clone();
			from_details = self.block_details(&from_details.parent).unwrap_or_else(|| panic!("2. Expected to find details for block {:?}", from_details.parent));
		}

		while to_details.number > from_details.number {
			to_branch.push(current_to);
			current_to = to_details.parent.clone();
			to_details = self.block_details(&to_details.parent).unwrap_or_else(|| panic!("3. Expected to find details for block {:?}", to_details.parent));
		}

		assert_eq!(from_details.number, to_details.number);

		// move to shared parent
		while current_from != current_to {
			from_branch.push(current_from);
			current_from = from_details.parent.clone();
			from_details = self.block_details(&from_details.parent).unwrap_or_else(|| panic!("4. Expected to find details for block {:?}", from_details.parent));

			to_branch.push(current_to);
			current_to = to_details.parent.clone();
			to_details = self.block_details(&to_details.parent).unwrap_or_else(|| panic!("5. Expected to find details for block {:?}", from_details.parent));
		}

		let index = from_branch.len();

		from_branch.extend(to_branch.into_iter().rev());

		TreeRoute {
			blocks: from_branch,
			ancestor: current_from,
			index: index
		}
	}

	/// Inserts a verified, known block from the canonical chain.
	///
	/// Can be performed out-of-order, but care must be taken that the final chain is in a correct state.
	/// This is used by snapshot restoration.
	///
	/// Supply a dummy parent total difficulty when the parent block may not be in the chain.
	/// Returns true if the block is disconnected.
	pub fn insert_snapshot_block(&self, bytes: &[u8], receipts: Vec<Receipt>, parent_td: Option<U256>, is_best: bool) -> bool {
		let block = BlockView::new(bytes);
		let header = block.header_view();
		let hash = header.sha3();

		if self.is_known(&hash) {
			return false;
		}

		assert!(self.pending_best_block.read().is_none());

		let mut batch = self.db.transaction();

		let block_rlp = UntrustedRlp::new(bytes);
		let compressed_header = block_rlp.at(0).unwrap().compress(RlpType::Blocks);
		let compressed_body = UntrustedRlp::new(&Self::block_to_body(bytes)).compress(RlpType::Blocks);

		// store block in db
		batch.put(db::COL_HEADERS, &hash, &compressed_header);
		batch.put(db::COL_BODIES, &hash, &compressed_body);

		let maybe_parent = self.block_details(&header.parent_hash());

		if let Some(parent_details) = maybe_parent {
			// parent known to be in chain.
			let info = BlockInfo {
				hash: hash,
				number: header.number(),
				total_difficulty: parent_details.total_difficulty + header.difficulty(),
				location: BlockLocation::CanonChain,
			};

			self.prepare_update(&mut batch, ExtrasUpdate {
				block_hashes: self.prepare_block_hashes_update(bytes, &info),
				block_details: self.prepare_block_details_update(bytes, &info),
				block_receipts: self.prepare_block_receipts_update(receipts, &info),
				blocks_blooms: self.prepare_block_blooms_update(bytes, &info),
				transactions_addresses: self.prepare_transaction_addresses_update(bytes, &info),
				info: info,
				block: bytes
			}, is_best);
			self.db.write(batch).unwrap();

			false
		} else {
			// parent not in the chain yet. we need the parent difficulty to proceed.
			let d = parent_td
				.expect("parent total difficulty always supplied for first block in chunk. only first block can have missing parent; qed");

			let info = BlockInfo {
				hash: hash,
				number: header.number(),
				total_difficulty: d + header.difficulty(),
				location: BlockLocation::CanonChain,
			};

			let block_details = BlockDetails {
				number: header.number(),
				total_difficulty: info.total_difficulty,
				parent: header.parent_hash(),
				children: Vec::new(),
			};

			let mut update = HashMap::new();
			update.insert(hash, block_details);

			self.prepare_update(&mut batch, ExtrasUpdate {
				block_hashes: self.prepare_block_hashes_update(bytes, &info),
				block_details: update,
				block_receipts: self.prepare_block_receipts_update(receipts, &info),
				blocks_blooms: self.prepare_block_blooms_update(bytes, &info),
				transactions_addresses: self.prepare_transaction_addresses_update(bytes, &info),
				info: info,
				block: bytes,
			}, is_best);
			self.db.write(batch).unwrap();

			true
		}
	}

	/// Add a child to a given block. Assumes that the block hash is in
	/// the chain and the child's parent is this block.
	///
	/// Used in snapshots to glue the chunks together at the end.
	pub fn add_child(&self, block_hash: H256, child_hash: H256) {
		let mut parent_details = self.block_details(&block_hash)
			.unwrap_or_else(|| panic!("Invalid block hash: {:?}", block_hash));

		let mut batch = self.db.transaction();
		parent_details.children.push(child_hash);

		let mut update = HashMap::new();
		update.insert(block_hash, parent_details);


		let mut write_details = self.block_details.write();
		batch.extend_with_cache(db::COL_EXTRA, &mut *write_details, update, CacheUpdatePolicy::Overwrite);

		self.cache_man.lock().note_used(CacheID::BlockDetails(block_hash));

		self.db.write(batch).unwrap();
	}

	#[cfg_attr(feature="dev", allow(similar_names))]
	/// Inserts the block into backing cache database.
	/// Expects the block to be valid and already verified.
	/// If the block is already known, does nothing.
	pub fn insert_block(&self, batch: &mut DBTransaction, bytes: &[u8], receipts: Vec<Receipt>) -> ImportRoute {
		// create views onto rlp
		let block = BlockView::new(bytes);
		let header = block.header_view();
		let hash = header.sha3();

		if self.is_known_child(&header.parent_hash(), &hash) {
			return ImportRoute::none();
		}

		assert!(self.pending_best_block.read().is_none());

		// store block in db
		batch.put_compressed(db::COL_HEADERS, &hash, block.header_rlp().as_raw().to_vec());
		batch.put_compressed(db::COL_BODIES, &hash, Self::block_to_body(bytes));

		let info = self.block_info(&header);

		if let BlockLocation::BranchBecomingCanonChain(ref d) = info.location {
			info!(target: "reorg", "Reorg to {} ({} {} {})",
				Colour::Yellow.bold().paint(format!("#{} {}", info.number, info.hash)),
				Colour::Red.paint(d.retracted.iter().join(" ")),
				Colour::White.paint(format!("#{} {}", self.block_details(&d.ancestor).expect("`ancestor` is in the route; qed").number, d.ancestor)),
				Colour::Green.paint(d.enacted.iter().join(" "))
			);
		}

		self.prepare_update(batch, ExtrasUpdate {
			block_hashes: self.prepare_block_hashes_update(bytes, &info),
			block_details: self.prepare_block_details_update(bytes, &info),
			block_receipts: self.prepare_block_receipts_update(receipts, &info),
			blocks_blooms: self.prepare_block_blooms_update(bytes, &info),
			transactions_addresses: self.prepare_transaction_addresses_update(bytes, &info),
			info: info.clone(),
			block: bytes,
		}, true);

		ImportRoute::from(info)
	}

	/// Get inserted block info which is critical to prepare extras updates.
	fn block_info(&self, header: &HeaderView) -> BlockInfo {
		let hash = header.sha3();
		let number = header.number();
		let parent_hash = header.parent_hash();
		let parent_details = self.block_details(&parent_hash).unwrap_or_else(|| panic!("Invalid parent hash: {:?}", parent_hash));
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
						let retracted = route.blocks.iter().take(route.index).cloned().collect::<Vec<_>>().into_iter().collect::<Vec<_>>();
						let enacted = route.blocks.into_iter().skip(route.index).collect::<Vec<_>>();
						BlockLocation::BranchBecomingCanonChain(BranchBecomingCanonChainData {
							ancestor: route.ancestor,
							enacted: enacted,
							retracted: retracted,
						})
					}
				}
			} else {
				BlockLocation::Branch
			}
		}
	}

	/// Prepares extras update.
	fn prepare_update(&self, batch: &mut DBTransaction, update: ExtrasUpdate, is_best: bool) {
		{
			let block_hashes: Vec<_> = update.block_details.keys().cloned().collect();

			let mut write_details = self.block_details.write();
			batch.extend_with_cache(db::COL_EXTRA, &mut *write_details, update.block_details, CacheUpdatePolicy::Overwrite);

			let mut cache_man = self.cache_man.lock();
			for hash in block_hashes {
				cache_man.note_used(CacheID::BlockDetails(hash));
			}
		}

		{
			let mut write_receipts = self.block_receipts.write();
			batch.extend_with_cache(db::COL_EXTRA, &mut *write_receipts, update.block_receipts, CacheUpdatePolicy::Remove);
		}

		{
			let mut write_blocks_blooms = self.blocks_blooms.write();
			batch.extend_with_cache(db::COL_EXTRA, &mut *write_blocks_blooms, update.blocks_blooms, CacheUpdatePolicy::Remove);
		}

		// These cached values must be updated last with all three locks taken to avoid
		// cache decoherence
		{
			let mut best_block = self.pending_best_block.write();
			// update best block
			match update.info.location {
				BlockLocation::Branch => (),
				_ => if is_best {
					batch.put(db::COL_EXTRA, b"best", &update.info.hash);
					*best_block = Some(BestBlock {
						hash: update.info.hash,
						number: update.info.number,
						total_difficulty: update.info.total_difficulty,
						block: update.block.to_vec(),
					});
				},
			}
			let mut write_hashes = self.pending_block_hashes.write();
			let mut write_txs = self.pending_transaction_addresses.write();

			batch.extend_with_cache(db::COL_EXTRA, &mut *write_hashes, update.block_hashes, CacheUpdatePolicy::Overwrite);
			batch.extend_with_option_cache(db::COL_EXTRA, &mut *write_txs, update.transactions_addresses, CacheUpdatePolicy::Overwrite);
		}
	}

	/// Apply pending insertion updates
	pub fn commit(&self) {
		let mut pending_best_block = self.pending_best_block.write();
		let mut pending_write_hashes = self.pending_block_hashes.write();
		let mut pending_write_txs = self.pending_transaction_addresses.write();

		let mut best_block = self.best_block.write();
		let mut write_hashes = self.block_hashes.write();
		let mut write_txs = self.transaction_addresses.write();
		// update best block
		if let Some(block) = pending_best_block.take() {
			*best_block = block;
		}

		let pending_txs = mem::replace(&mut *pending_write_txs, HashMap::new());
		let (retracted_txs, enacted_txs) = pending_txs.into_iter().partition::<HashMap<_, _>, _>(|&(_, ref value)| value.is_none());

		let pending_hashes_keys: Vec<_> = pending_write_hashes.keys().cloned().collect();
		let enacted_txs_keys: Vec<_> = enacted_txs.keys().cloned().collect();

		write_hashes.extend(mem::replace(&mut *pending_write_hashes, HashMap::new()));
		write_txs.extend(enacted_txs.into_iter().map(|(k, v)| (k, v.expect("Transactions were partitioned; qed"))));

		for hash in retracted_txs.keys() {
			write_txs.remove(hash);
		}

		let mut cache_man = self.cache_man.lock();
		for n in pending_hashes_keys {
			cache_man.note_used(CacheID::BlockHashes(n));
		}

		for hash in enacted_txs_keys {
			cache_man.note_used(CacheID::TransactionAddresses(hash));
		}
	}

	/// Iterator that lists `first` and then all of `first`'s ancestors, by hash.
	pub fn ancestry_iter(&self, first: H256) -> Option<AncestryIter> {
		if self.is_known(&first) {
			Some(AncestryIter {
				current: first,
				chain: self,
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
				let ancestor_number = self.block_number(&data.ancestor).expect("Block number of ancestor is always in DB");
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
	/// Uses the given parent details or attempts to load them from the database.
	fn prepare_block_details_update(&self, block_bytes: &[u8], info: &BlockInfo) -> HashMap<H256, BlockDetails> {
		let block = BlockView::new(block_bytes);
		let header = block.header_view();
		let parent_hash = header.parent_hash();

		// update parent
		let mut parent_details = self.block_details(&parent_hash).unwrap_or_else(|| panic!("Invalid parent hash: {:?}", parent_hash));
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
	fn prepare_transaction_addresses_update(&self, block_bytes: &[u8], info: &BlockInfo) -> HashMap<H256, Option<TransactionAddress>> {
		let block = BlockView::new(block_bytes);
		let transaction_hashes = block.transaction_hashes();

		match info.location {
			BlockLocation::CanonChain => {
				transaction_hashes.into_iter()
					.enumerate()
					.map(|(i ,tx_hash)| {
						(tx_hash, Some(TransactionAddress {
							block_hash: info.hash.clone(),
							index: i
						}))
					})
					.collect()
			},
			BlockLocation::BranchBecomingCanonChain(ref data) => {
				let addresses = data.enacted.iter()
					.flat_map(|hash| {
						let bytes = self.block_body(hash).expect("Enacted block must be in database.");
						let hashes = BodyView::new(&bytes).transaction_hashes();
						hashes.into_iter()
							.enumerate()
							.map(|(i, tx_hash)| (tx_hash, Some(TransactionAddress {
								block_hash: hash.clone(),
								index: i,
							})))
							.collect::<HashMap<H256, Option<TransactionAddress>>>()
					});

				let current_addresses = transaction_hashes.into_iter()
					.enumerate()
					.map(|(i ,tx_hash)| {
						(tx_hash, Some(TransactionAddress {
							block_hash: info.hash.clone(),
							index: i
						}))
					});

				let retracted = data.retracted.iter().flat_map(|hash| {
					let bytes = self.block_body(hash).expect("Retracted block must be in database.");
					let hashes = BodyView::new(&bytes).transaction_hashes();
					hashes.into_iter().map(|hash| (hash, None)).collect::<HashMap<H256, Option<TransactionAddress>>>()
				});

				// The order here is important! Don't remove transaction if it was part of enacted blocks as well.
				retracted.chain(addresses).chain(current_addresses).collect()
			},
			BlockLocation::Branch => HashMap::new(),
		}
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
				let log_bloom = header.log_bloom();
				if log_bloom.is_zero() {
					HashMap::new()
				} else {
					let chain = bc::group::BloomGroupChain::new(self.blooms_config, self);
					chain.insert(info.number as bc::Number, Bloom::from(log_bloom).into())
				}
			},
			BlockLocation::BranchBecomingCanonChain(ref data) => {
				let ancestor_number = self.block_number(&data.ancestor).unwrap();
				let start_number = ancestor_number + 1;
				let range = start_number as bc::Number..self.best_block_number() as bc::Number;

				let mut blooms: Vec<bc::Bloom> = data.enacted.iter()
					.map(|hash| self.block_header_data(hash).unwrap())
					.map(|bytes| HeaderView::new(&bytes).log_bloom())
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
		self.best_block.read().hash.clone()
	}

	/// Get best block number.
	pub fn best_block_number(&self) -> BlockNumber {
		self.best_block.read().number
	}

	/// Get best block total difficulty.
	pub fn best_block_total_difficulty(&self) -> U256 {
		self.best_block.read().total_difficulty
	}

	/// Get best block header
	pub fn best_block_header(&self) -> Bytes {
		let block = self.best_block.read();
		BlockView::new(&block.block).header_view().rlp().as_raw().to_vec()
	}

	/// Get current cache size.
	pub fn cache_size(&self) -> CacheSize {
		CacheSize {
			blocks: self.block_headers.read().heap_size_of_children() + self.block_bodies.read().heap_size_of_children(),
			block_details: self.block_details.read().heap_size_of_children(),
			transaction_addresses: self.transaction_addresses.read().heap_size_of_children(),
			blocks_blooms: self.blocks_blooms.read().heap_size_of_children(),
			block_receipts: self.block_receipts.read().heap_size_of_children(),
		}
	}

	/// Ticks our cache system and throws out any old data.
	pub fn collect_garbage(&self) {
		let current_size = self.cache_size().total();

		let mut block_headers = self.block_headers.write();
		let mut block_bodies = self.block_bodies.write();
		let mut block_details = self.block_details.write();
		let mut block_hashes = self.block_hashes.write();
		let mut transaction_addresses = self.transaction_addresses.write();
		let mut blocks_blooms = self.blocks_blooms.write();
		let mut block_receipts = self.block_receipts.write();

		let mut cache_man = self.cache_man.lock();
		cache_man.collect_garbage(current_size, | ids | {
			for id in &ids {
				match *id {
					CacheID::BlockHeader(ref h) => { block_headers.remove(h); },
					CacheID::BlockBody(ref h) => { block_bodies.remove(h); },
					CacheID::BlockDetails(ref h) => { block_details.remove(h); }
					CacheID::BlockHashes(ref h) => { block_hashes.remove(h); }
					CacheID::TransactionAddresses(ref h) => { transaction_addresses.remove(h); }
					CacheID::BlocksBlooms(ref h) => { blocks_blooms.remove(h); }
					CacheID::BlockReceipts(ref h) => { block_receipts.remove(h); }
				}
			}

			block_headers.shrink_to_fit();
			block_bodies.shrink_to_fit();
			block_details.shrink_to_fit();
			block_hashes.shrink_to_fit();
			transaction_addresses.shrink_to_fit();
			blocks_blooms.shrink_to_fit();
			block_receipts.shrink_to_fit();

			block_headers.heap_size_of_children() +
			block_bodies.heap_size_of_children() +
			block_details.heap_size_of_children() +
			block_hashes.heap_size_of_children() +
			transaction_addresses.heap_size_of_children() +
			blocks_blooms.heap_size_of_children() +
			block_receipts.heap_size_of_children()
		});
	}

	/// Create a block body from a block.
	pub fn block_to_body(block: &[u8]) -> Bytes {
		let mut body = RlpStream::new_list(2);
		let block_rlp = Rlp::new(block);
		body.append_raw(block_rlp.at(1).as_raw(), 1);
		body.append_raw(block_rlp.at(2).as_raw(), 1);
		body.out()
	}
}

#[cfg(test)]
mod tests {
	#![cfg_attr(feature="dev", allow(similar_names))]
	use std::sync::Arc;
	use rustc_serialize::hex::FromHex;
	use util::{Database, DatabaseConfig};
	use util::hash::*;
	use util::sha3::Hashable;
	use receipt::Receipt;
	use blockchain::{BlockProvider, BlockChain, Config, ImportRoute};
	use tests::helpers::*;
	use devtools::*;
	use blockchain::generator::{ChainGenerator, ChainIterator, BlockFinalizer};
	use blockchain::extras::TransactionAddress;
	use views::BlockView;
	use transaction::{Transaction, Action};
	use log_entry::{LogEntry, LocalizedLogEntry};

	fn new_db(path: &str) -> Arc<Database> {
		Arc::new(Database::open(&DatabaseConfig::with_columns(::db::NUM_COLUMNS), path).unwrap())
	}

	#[test]
	fn should_cache_best_block() {
		// given
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let first = canon_chain.generate(&mut finalizer).unwrap();

		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());
		assert_eq!(bc.best_block_number(), 0);

		// when
		let mut batch =db.transaction();
		bc.insert_block(&mut batch, &first, vec![]);
		assert_eq!(bc.best_block_number(), 0);
		bc.commit();
		// NOTE no db.write here (we want to check if best block is cached)

		// then
		assert_eq!(bc.best_block_number(), 1);
		assert!(bc.block(&bc.best_block_hash()).is_some(), "Best block should be queryable even without DB write.");
	}

	#[test]
	fn basic_blockchain_insert() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let first = canon_chain.generate(&mut finalizer).unwrap();
		let genesis_hash = BlockView::new(&genesis).header_view().sha3();
		let first_hash = BlockView::new(&first).header_view().sha3();

		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		assert_eq!(bc.genesis_hash(), genesis_hash.clone());
		assert_eq!(bc.best_block_hash(), genesis_hash.clone());
		assert_eq!(bc.block_hash(0), Some(genesis_hash.clone()));
		assert_eq!(bc.block_hash(1), None);
		assert_eq!(bc.block_details(&genesis_hash).unwrap().children, vec![]);

		let mut batch =db.transaction();
		bc.insert_block(&mut batch, &first, vec![]);
		db.write(batch).unwrap();
		bc.commit();

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
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		let mut block_hashes = vec![genesis_hash.clone()];
		let mut batch = db.transaction();
		for _ in 0..10 {
			let block = canon_chain.generate(&mut finalizer).unwrap();
			block_hashes.push(BlockView::new(&block).header_view().sha3());
			bc.insert_block(&mut batch, &block, vec![]);
			bc.commit();
		}
		db.write(batch).unwrap();

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
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		let mut batch =db.transaction();
		for b in &[&b1a, &b1b, &b2a, &b2b, &b3a, &b3b, &b4a, &b4b, &b5a, &b5b] {
			bc.insert_block(&mut batch, b, vec![]);
			bc.commit();
		}
		bc.insert_block(&mut batch, &b1b, vec![]);
		bc.insert_block(&mut batch, &b2a, vec![]);
		bc.insert_block(&mut batch, &b2b, vec![]);
		bc.insert_block(&mut batch, &b3a, vec![]);
		bc.insert_block(&mut batch, &b3b, vec![]);
		bc.insert_block(&mut batch, &b4a, vec![]);
		bc.insert_block(&mut batch, &b4b, vec![]);
		bc.insert_block(&mut batch, &b5a, vec![]);
		bc.insert_block(&mut batch, &b5b, vec![]);
		db.write(batch).unwrap();

		assert_eq!(
			[&b4b, &b3b, &b2b].iter().map(|b| BlockView::new(b).header()).collect::<Vec<_>>(),
			bc.find_uncle_headers(&BlockView::new(&b4a).header_view().sha3(), 3).unwrap()
		);

		// TODO: insert block that already includes one of them as an uncle to check it's not allowed.
	}

	#[test]
	fn test_fork_transaction_addresses() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let mut fork_chain = canon_chain.fork(1);
		let mut fork_finalizer = finalizer.fork();

		let t1 = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 100.into(),
			data: "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap(),
		}.sign(&"".sha3());


		let b1a = canon_chain
			.with_transaction(t1.clone())
			.generate(&mut finalizer).unwrap();

		// Empty block
		let b1b = fork_chain
			.generate(&mut fork_finalizer).unwrap();

		let b2 = fork_chain
			.generate(&mut fork_finalizer).unwrap();

		let b1a_hash = BlockView::new(&b1a).header_view().sha3();
		let b2_hash = BlockView::new(&b2).header_view().sha3();

		let t1_hash = t1.hash();

		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		let mut batch = db.transaction();
		let _ = bc.insert_block(&mut batch, &b1a, vec![]);
		bc.commit();
		let _ = bc.insert_block(&mut batch, &b1b, vec![]);
		bc.commit();
		db.write(batch).unwrap();

		assert_eq!(bc.best_block_hash(), b1a_hash);
		assert_eq!(bc.transaction_address(&t1_hash), Some(TransactionAddress {
			block_hash: b1a_hash.clone(),
			index: 0,
		}));

		// now let's make forked chain the canon chain
		let mut batch = db.transaction();
		let _ = bc.insert_block(&mut batch, &b2, vec![]);
		bc.commit();
		db.write(batch).unwrap();

		// Transaction should be retracted
		assert_eq!(bc.best_block_hash(), b2_hash);
		assert_eq!(bc.transaction_address(&t1_hash), None);
	}



	#[test]
	fn test_overwriting_transaction_addresses() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let mut fork_chain = canon_chain.fork(1);
		let mut fork_finalizer = finalizer.fork();

		let t1 = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 100.into(),
			data: "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap(),
		}.sign(&"".sha3());

		let t2 = Transaction {
			nonce: 1.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 100.into(),
			data: "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap(),
		}.sign(&"".sha3());

		let t3 = Transaction {
			nonce: 2.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 100.into(),
			data: "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap(),
		}.sign(&"".sha3());

		let b1a = canon_chain
			.with_transaction(t1.clone())
			.with_transaction(t2.clone())
			.generate(&mut finalizer).unwrap();

		// insert transactions in different order
		let b1b = fork_chain
			.with_transaction(t2.clone())
			.with_transaction(t1.clone())
			.generate(&mut fork_finalizer).unwrap();

		let b2 = fork_chain
			.with_transaction(t3.clone())
			.generate(&mut fork_finalizer).unwrap();

		let b1a_hash = BlockView::new(&b1a).header_view().sha3();
		let b1b_hash = BlockView::new(&b1b).header_view().sha3();
		let b2_hash = BlockView::new(&b2).header_view().sha3();

		let t1_hash = t1.hash();
		let t2_hash = t2.hash();
		let t3_hash = t3.hash();

		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		let mut batch = db.transaction();
		let _ = bc.insert_block(&mut batch, &b1a, vec![]);
		bc.commit();
		let _ = bc.insert_block(&mut batch, &b1b, vec![]);
		bc.commit();
		db.write(batch).unwrap();

		assert_eq!(bc.best_block_hash(), b1a_hash);
		assert_eq!(bc.transaction_address(&t1_hash), Some(TransactionAddress {
			block_hash: b1a_hash.clone(),
			index: 0,
		}));
		assert_eq!(bc.transaction_address(&t2_hash), Some(TransactionAddress {
			block_hash: b1a_hash.clone(),
			index: 1,
		}));

		// now let's make forked chain the canon chain
		let mut batch = db.transaction();
		let _ = bc.insert_block(&mut batch, &b2, vec![]);
		bc.commit();
		db.write(batch).unwrap();

		assert_eq!(bc.best_block_hash(), b2_hash);
		assert_eq!(bc.transaction_address(&t1_hash), Some(TransactionAddress {
			block_hash: b1b_hash.clone(),
			index: 1,
		}));
		assert_eq!(bc.transaction_address(&t2_hash), Some(TransactionAddress {
			block_hash: b1b_hash.clone(),
			index: 0,
		}));
		assert_eq!(bc.transaction_address(&t3_hash), Some(TransactionAddress {
			block_hash: b2_hash.clone(),
			index: 0,
		}));
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
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		let mut batch = db.transaction();
		let ir1 = bc.insert_block(&mut batch, &b1, vec![]);
		bc.commit();
		let ir2 = bc.insert_block(&mut batch, &b2, vec![]);
		bc.commit();
		let ir3b = bc.insert_block(&mut batch, &b3b, vec![]);
		bc.commit();
		db.write(batch).unwrap();
		assert_eq!(bc.block_hash(3).unwrap(), b3b_hash);
		let mut batch =db.transaction();
		let ir3a = bc.insert_block(&mut batch, &b3a, vec![]);
		bc.commit();
		db.write(batch).unwrap();

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
			let db = new_db(temp.as_str());
			let bc = BlockChain::new(Config::default(), &genesis, db.clone());
			assert_eq!(bc.best_block_hash(), genesis_hash);
			let mut batch =db.transaction();
			bc.insert_block(&mut batch, &first, vec![]);
			db.write(batch).unwrap();
			bc.commit();
			assert_eq!(bc.best_block_hash(), first_hash);
		}

		{
			let db = new_db(temp.as_str());
			let bc = BlockChain::new(Config::default(), &genesis, db.clone());

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
			block_header = bc.block_header(block_header.unwrap().parent_hash());
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
		let b1_hash: H256 = "f53f268d23a71e85c7d6d83a9504298712b84c1a2ba220441c86eeda0bf0b6e3".into();

		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());
		let mut batch =db.transaction();
		bc.insert_block(&mut batch, &b1, vec![]);
		db.write(batch).unwrap();
		bc.commit();

		let transactions = bc.transactions(&b1_hash).unwrap();
		assert_eq!(transactions.len(), 7);
		for t in transactions {
			assert_eq!(bc.transaction(&bc.transaction_address(&t.hash()).unwrap()).unwrap(), t);
		}
	}

	fn insert_block(db: &Arc<Database>, bc: &BlockChain, bytes: &[u8], receipts: Vec<Receipt>) -> ImportRoute {
		let mut batch = db.transaction();
		let res = bc.insert_block(&mut batch, bytes, receipts);
		db.write(batch).unwrap();
		bc.commit();
		res
	}

	#[test]
	fn test_logs() {
		// given
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		// just insert dummy transaction so that #transactions=#receipts
		let t1 = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 101.into(),
			data: "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap(),
		}.sign(&"".sha3());
		let t2 = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 102.into(),
			data: "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap(),
		}.sign(&"".sha3());
		let t3 = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 100_000.into(),
			action: Action::Create,
			value: 103.into(),
			data: "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap(),
		}.sign(&"".sha3());
		let tx_hash1 = t1.hash();
		let tx_hash2 = t2.hash();
		let tx_hash3 = t3.hash();
		let b1 = canon_chain.with_transaction(t1).with_transaction(t2).generate(&mut finalizer).unwrap();
		let b2 = canon_chain.with_transaction(t3).generate(&mut finalizer).unwrap();

		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());
		insert_block(&db, &bc, &b1, vec![Receipt {
			state_root: H256::default(),
			gas_used: 10_000.into(),
			log_bloom: Default::default(),
			logs: vec![
				LogEntry { address: Default::default(), topics: vec![], data: vec![1], },
				LogEntry { address: Default::default(), topics: vec![], data: vec![2], },
			],
		},
		Receipt {
			state_root: H256::default(),
			gas_used: 10_000.into(),
			log_bloom: Default::default(),
			logs: vec![
				LogEntry { address: Default::default(), topics: vec![], data: vec![3], },
			],
		}]);
		insert_block(&db, &bc, &b2, vec![
			Receipt {
				state_root: H256::default(),
				gas_used: 10_000.into(),
				log_bloom: Default::default(),
				logs: vec![
					LogEntry { address: Default::default(), topics: vec![], data: vec![4], },
				],
			}
		]);

		// when
		let block1 = BlockView::new(&b1);
		let block2 = BlockView::new(&b2);
		let logs1 = bc.logs(vec![1, 2], |_| true, None);
		let logs2 = bc.logs(vec![1, 2], |_| true, Some(1));

		// then
		assert_eq!(logs1, vec![
			LocalizedLogEntry {
				entry: LogEntry { address: Default::default(), topics: vec![], data: vec![1] },
				block_hash: block1.hash(),
				block_number: block1.header().number(),
				transaction_hash: tx_hash1.clone(),
				transaction_index: 0,
				log_index: 0,
			},
			LocalizedLogEntry {
				entry: LogEntry { address: Default::default(), topics: vec![], data: vec![2] },
				block_hash: block1.hash(),
				block_number: block1.header().number(),
				transaction_hash: tx_hash1.clone(),
				transaction_index: 0,
				log_index: 1,
			},
			LocalizedLogEntry {
				entry: LogEntry { address: Default::default(), topics: vec![], data: vec![3] },
				block_hash: block1.hash(),
				block_number: block1.header().number(),
				transaction_hash: tx_hash2.clone(),
				transaction_index: 1,
				log_index: 2,
			},
			LocalizedLogEntry {
				entry: LogEntry { address: Default::default(), topics: vec![], data: vec![4] },
				block_hash: block2.hash(),
				block_number: block2.header().number(),
				transaction_hash: tx_hash3.clone(),
				transaction_index: 0,
				log_index: 0,
			}
		]);
		assert_eq!(logs2, vec![
			LocalizedLogEntry {
				entry: LogEntry { address: Default::default(), topics: vec![], data: vec![4] },
				block_hash: block2.hash(),
				block_number: block2.header().number(),
				transaction_hash: tx_hash3.clone(),
				transaction_index: 0,
				log_index: 0,
			}
		]);
	}

	#[test]
	fn test_bloom_filter_simple() {
		// TODO: From here
		let bloom_b1: H2048 = "00000020000000000000000000000000000000000000000002000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000400000000000000000000002000".into();

		let bloom_b2: H2048 = "00000000000000000000000000000000000000000000020000001000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000008000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".into();

		let bloom_ba: H2048 = "00000000000000000000000000000000000000000000020000000800000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000008000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".into();

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
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		assert_eq!(blocks_b1, vec![]);
		assert_eq!(blocks_b2, vec![]);

		insert_block(&db, &bc, &b1, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![]);

		insert_block(&db, &bc, &b2, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![2]);

		// hasn't been forked yet
		insert_block(&db, &bc, &b1a, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		let blocks_ba = bc.blocks_with_bloom(&bloom_ba, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![2]);
		assert_eq!(blocks_ba, vec![]);

		// fork has happend
		insert_block(&db, &bc, &b2a, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		let blocks_ba = bc.blocks_with_bloom(&bloom_ba, 0, 5);
		assert_eq!(blocks_b1, vec![]);
		assert_eq!(blocks_b2, vec![]);
		assert_eq!(blocks_ba, vec![1, 2]);

		// fork back
		insert_block(&db, &bc, &b3, vec![]);
		let blocks_b1 = bc.blocks_with_bloom(&bloom_b1, 0, 5);
		let blocks_b2 = bc.blocks_with_bloom(&bloom_b2, 0, 5);
		let blocks_ba = bc.blocks_with_bloom(&bloom_ba, 0, 5);
		assert_eq!(blocks_b1, vec![1]);
		assert_eq!(blocks_b2, vec![2]);
		assert_eq!(blocks_ba, vec![3]);
	}

	#[test]
	fn test_best_block_update() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();

		let temp = RandomTempPath::new();

		{
			let db = new_db(temp.as_str());
			let bc = BlockChain::new(Config::default(), &genesis, db.clone());
			let uncle = canon_chain.fork(1).generate(&mut finalizer.fork()).unwrap();

			let mut batch =db.transaction();
			// create a longer fork
			for _ in 0..5 {
				let canon_block = canon_chain.generate(&mut finalizer).unwrap();
				bc.insert_block(&mut batch, &canon_block, vec![]);
				bc.commit();
			}

			assert_eq!(bc.best_block_number(), 5);
			bc.insert_block(&mut batch, &uncle, vec![]);
			db.write(batch).unwrap();
			bc.commit();
		}

		// re-loading the blockchain should load the correct best block.
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());
		assert_eq!(bc.best_block_number(), 5);
	}

	#[test]
	fn test_rewind() {
		let mut canon_chain = ChainGenerator::default();
		let mut finalizer = BlockFinalizer::default();
		let genesis = canon_chain.generate(&mut finalizer).unwrap();
		let first = canon_chain.generate(&mut finalizer).unwrap();
		let second = canon_chain.generate(&mut finalizer).unwrap();
		let genesis_hash = BlockView::new(&genesis).header_view().sha3();
		let first_hash = BlockView::new(&first).header_view().sha3();
		let second_hash = BlockView::new(&second).header_view().sha3();

		let temp = RandomTempPath::new();
		let db = new_db(temp.as_str());
		let bc = BlockChain::new(Config::default(), &genesis, db.clone());

		let mut batch =db.transaction();
		bc.insert_block(&mut batch, &first, vec![]);
		bc.commit();
		bc.insert_block(&mut batch, &second, vec![]);
		bc.commit();
		db.write(batch).unwrap();

		assert_eq!(bc.rewind(), Some(first_hash.clone()));
		assert!(!bc.is_known(&second_hash));
		assert_eq!(bc.best_block_number(), 1);
		assert_eq!(bc.best_block_hash(), first_hash.clone());

		assert_eq!(bc.rewind(), Some(genesis_hash.clone()));
		assert_eq!(bc.rewind(), None);
	}
}
