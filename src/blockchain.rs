use std::collections::HashMap;
use std::sync::RwLock;
use std::path::Path;
use std::hash::Hash;
use rocksdb::{DB, WriteBatch, Writable};
use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::sha3::*;
use util::bytes::*;
use blockheader::*;
use block::*;
use verifiedblock::*;
use importroute::*;
use account::*;
use genesis::*;
use extras::*;
use transaction::*;

pub struct BlockChain {
	// rlp list of 3
	genesis_block: Bytes,
	// genesis block header
	genesis_header: Bytes,
	genesis_hash: H256,
	genesis_state: HashMap<Address, Account>,

	last_block_number: U256,

	// block cache
	blocks: RwLock<HashMap<H256, Bytes>>,

	// extra caches
	block_details: 			Extras<H256, BlockDetails>,
	block_hashes: 			Extras<U256, H256>,
	transaction_addresses: 	Extras<H256, TransactionAddress>,
	block_logs: 			Extras<H256, BlockLogBlooms>,
	blocks_blooms: 			Extras<H256, BlocksBlooms>,

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
	/// 	assert_eq!(bc.genesis_hash(), &H256::from_str(genesis_hash).unwrap());
	/// 	assert!(bc.is_known(bc.genesis_hash()));
	/// 	assert_eq!(bc.genesis_hash(), &bc.number_hash(&U256::from(0u8)).unwrap());
	/// }
	/// ```
	pub fn new(genesis: Genesis, path: &Path) -> BlockChain {
		let (genesis_block, genesis_state) = genesis.drain();

		let genesis_header = BlockView::new(&genesis_block).header_view().rlp().raw().to_vec();
		let genesis_hash = HeaderView::new(&genesis_header).sha3();

		let genesis_details = BlockDetails {
			number: U256::from(0u64),
			total_difficulty: HeaderView::new(&genesis_header).difficulty(),
			parent: H256::new(),
			children: vec![]
		};

		let mut extras_path = path.to_path_buf();
		extras_path.push("extras");
		let extras_db = DB::open_default(extras_path.to_str().unwrap()).unwrap();

		let mut blocks_path = path.to_path_buf();
		blocks_path.push("blocks");
		let blocks_db = DB::open_default(blocks_path.to_str().unwrap()).unwrap();

		{
			let mut batch = WriteBatch::new();
			batch.put(&genesis_hash.to_extras_slice(ExtrasIndex::BlockDetails), &encode(&genesis_details));
			batch.put(&U256::from(0u8).to_extras_slice(ExtrasIndex::BlockHash), &encode(&genesis_hash));
			extras_db.write(batch);

			blocks_db.put(&genesis_hash, &genesis_block);
		}

		BlockChain {
			genesis_block: genesis_block,
			genesis_header: genesis_header,
			genesis_hash: genesis_hash,
			genesis_state: genesis_state,
			last_block_number: U256::from(0u8),
			blocks: RwLock::new(HashMap::new()),
			block_details: Extras::new(ExtrasIndex::BlockDetails),
			block_hashes: Extras::new(ExtrasIndex::BlockHash),
			transaction_addresses: Extras::new(ExtrasIndex::TransactionAddress),
			block_logs: Extras::new(ExtrasIndex::BlockLogBlooms),
			blocks_blooms: Extras::new(ExtrasIndex::BlocksBlooms),
			extras_db: extras_db,
			blocks_db: blocks_db
		}
	}

	pub fn genesis_block(&self, db: &OverlayDB) -> Block {
		let root = HeaderView::new(&self.genesis_block).state_root();

		if db.exists(&root) {
			return Block::new_existing(db.clone(), root)
		}

		let mut block = Block::new(db.clone());
		// TODO: commit
		//block.mutable_state().insert_accounts(&self.genesis_state);
		block.mutable_state().db().commit();
		// TODO: set previous block
		// TODO: reset current
		block
	}

	pub fn verify_block<'a>(&self, block: &'a [u8]) -> VerifiedBlock<'a> {
		//TODO: verify block 
		VerifiedBlock::new(block)
	}

	pub fn import_block(&self, block: &[u8], db: &OverlayDB) -> ImportRoute {
		let view = HeaderView::new(block);

		// check if we already know this block
		if self.is_known(&view.sha3()) {

		}

		// check if we already know parent of this block
		if !self.is_known(&view.parent_hash()) {
		}

		unimplemented!();
	}

	/// Returns true if the given block is known 
	/// (though not necessarily a part of the canon chain).
	pub fn is_known(&self, hash: &H256) -> bool {
		// TODO: consider taking into account current block
		// TODO: first do lookup in blocks_db for given hash
		// TODO: is comparing block numbers necessery?
		match self.query_extras(hash, &self.block_details) {
			None => false,
			Some(details) => details.number <= self.last_block_number
		}
	}

	/// Returns true if transaction is known.
	pub fn is_known_transaction(&self, hash: &H256) -> bool {
		self.query_extras_exist(hash, &self.transaction_addresses)
	}

	/// Returns reference to genesis hash
	pub fn genesis_hash(&self) -> &H256 {
		&self.genesis_hash
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
	pub fn number_hash(&self, hash: &U256) -> Option<H256> {
		self.query_extras(hash, &self.block_hashes)
	}

	/// Get the transactions' log blooms of a block
	pub fn log_blooms(&self, hash: &H256) -> Option<BlockLogBlooms> {
		self.query_extras(hash, &self.block_logs)
	}

	fn block(&self, hash: &H256) -> Option<Bytes> {
		{
			let read = self.blocks.read().unwrap();
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
				let mut write = self.blocks.write().unwrap();
				write.insert(hash.clone(), bytes.clone());
				Some(bytes)
			},
			None => None
		}
	}

	fn query_extras<K, T>(&self, hash: &K, cache: &Extras<K, T>) -> Option<T> where 
		T: Clone + Decodable, 
		K: ExtrasSliceConvertable + Eq + Hash + Clone {
		{
			let read = cache.read().unwrap();
			match read.get(hash) {
				Some(v) => return Some(v.clone()),
				None => ()
			}
		}

		let opt = self.extras_db.get(&hash.to_extras_slice(cache.index()))
			.expect("Low level database error. Some issue with disk?");

		match opt {
			Some(b) => {
				let t: T = decode(&b);
				let mut write = cache.write().unwrap();
				write.insert(hash.clone(), t.clone());
				Some(t)
			},
			None => None
		}
	}

	fn query_extras_exist<K, T>(&self, hash: &K, cache: &Extras<K, T>) -> bool where 
		K: ExtrasSliceConvertable + Eq + Hash + Clone {
		{
			let read = cache.read().unwrap();
			match read.get(hash) {
				Some(_) => return true,
				None => ()
			}
		}

		let opt = self.extras_db.get(&hash.to_extras_slice(cache.index()))
			.expect("Low level database error. Some issue with disk?");

		opt.is_some()
	}
}

