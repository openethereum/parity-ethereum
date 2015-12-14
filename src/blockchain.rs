use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::hash::Hash;
use rocksdb::{DB, WriteBatch, Writable};
use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::sha3::*;
use blockheader::*;
use block::*;
use verifiedblock::*;
use importroute::*;
use account::*;
use genesis::*;
use extras::*;

pub struct BlockChain {
	// TODO: consider wrapping `genesis_info` into Arc<GenesisInfo>
	// rlp list of 3
	genesis_block: Vec<u8>,
	// genesis block header
	genesis_header: Vec<u8>,
	genesis_hash: H256,
	genesis_state: HashMap<Address, Account>,

	last_block_number: U256,

	// extras
	blocks_details: Extras<H256, BlockDetails>,
	blocks_hashes: Extras<U256, H256>,
	extras_db: DB
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

		let genesis_header = Rlp::new(&genesis_block).at(0).raw().to_vec();
		let genesis_hash = BlockView::new(&genesis_header).sha3();

		let genesis_details = BlockDetails {
			number: U256::from(0u64),
			total_difficulty: BlockView::new(&genesis_header).difficulty(),
			parent: H256::new(),
			children: vec![]
		};

		let db = DB::open_default(path.to_str().unwrap()).unwrap();

		{
			let mut batch = WriteBatch::new();
			batch.put(&genesis_hash.to_extras_slice(ExtrasIndex::BlockDetails), &encode(&genesis_details));
			batch.put(&U256::from(0u8).to_extras_slice(ExtrasIndex::BlockHash), &encode(&genesis_hash));
			db.write(batch);
		}

		BlockChain {
			genesis_block: genesis_block,
			genesis_header: genesis_header,
			genesis_hash: genesis_hash,
			genesis_state: genesis_state,
			last_block_number: U256::from(0u8),
			blocks_details: Extras::new(ExtrasIndex::BlockDetails),
			blocks_hashes: Extras::new(ExtrasIndex::BlockHash),
			extras_db: db
		}
	}

	pub fn genesis_hash(&self) -> &H256 {
		&self.genesis_hash
	}

	pub fn genesis_block(&self, db: &OverlayDB) -> Block {
		let root = BlockView::new(&self.genesis_block).state_root();

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
		let view = BlockView::new(block);

		// check if we already know this block
		if self.is_known(&view.sha3()) {

		}

		// check if we already know parent of this block
		if !self.is_known(&view.parent_hash()) {
		}

		unimplemented!();
	}

	/// Get the hash of given block's number
	pub fn number_hash(&self, hash: &U256) -> Option<H256> {
		self.query_extras(hash, &self.blocks_hashes)
	}

	/// Returns true if the given block is known 
	/// (though not necessarily a part of the canon chain).
	pub fn is_known(&self, hash: &H256) -> bool {
		// TODO: first do lookup in blocks_db for given hash

		// TODO: consider taking into account current block
		match self.query_extras(hash, &self.blocks_details) {
			None => false,
			Some(details) => details.number <= self.last_block_number
		}
	}

	pub fn query_extras<K, T>(&self, hash: &K, cache: &Extras<K, T>) -> Option<T> where 
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

	pub fn query_extras_exist<K, T>(&self, hash: &K, cache: &Extras<K, T>) -> bool where 
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

