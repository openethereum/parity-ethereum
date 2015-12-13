use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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

	// extras
	// TODO: is arc really needed here, since blockchain itself will be wrapped
	// into `Arc`?
	block_details: Arc<Mutex<HashMap<H256, BlockDetails>>>
}

impl BlockChain {
	/// Create new instance of blockchain from given Genesis
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// extern crate ethcore;
	/// use std::str::FromStr;
	/// use ethcore::genesis::*;
	/// use ethcore::blockchain::*;
	/// use util::hash::*;
	/// 
	/// fn main() {
	/// 	let genesis = Genesis::new_frontier();
	/// 	let bc = BlockChain::new(genesis);
	/// 	let genesis_hash = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
	/// 	assert_eq!(bc.genesis_hash(), &H256::from_str(genesis_hash).unwrap());
	/// }
	/// ```
	pub fn new(genesis: Genesis) -> BlockChain {
		let (genesis_block, genesis_state) = genesis.drain();

		let genesis_header = Rlp::new(&genesis_block).at(0).raw().to_vec();
		let genesis_hash = BlockView::new(&genesis_header).sha3();

		let genesis_details = BlockDetails {
			number: U256::from(0u64),
			total_difficulty: BlockView::new(&genesis_header).difficulty(),
			parent: H256::new(),
			children: vec![]
		};

		// TODO: also insert into backing db
		let mut block_details = HashMap::new();
		block_details.insert(genesis_hash.clone(), genesis_details);

		let bc = BlockChain {
			genesis_block: genesis_block,
			genesis_header: genesis_header,
			genesis_hash: genesis_hash,
			genesis_state: genesis_state,
			block_details: Arc::new(Mutex::new(block_details))
		};

		bc
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
		block.mutable_state().insert_accounts(&self.genesis_state);
		block.mutable_state().commit_db();
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

	/// Returns true if the given block is known 
	/// (though not necessarily a part of the canon chain).
	pub fn is_known(&self, hash: &H256) -> bool {
		return false;
		//unimplemented!()
		// TODO: check is hash exist in hashes
	}
}
