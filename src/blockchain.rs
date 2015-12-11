use util::hash::*;
use util::rlp::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::sha3::*;
use blockheader::*;
use block::*;
use verifiedblock::*;
use importroute::*;

pub struct BlockChain {
	genesis_block: Vec<u8>,
	genesis_hash: H256
}

impl BlockChain {
	pub fn new(genesis_block: Vec<u8>) -> BlockChain {
		// consider creating `GenesisView` for genesis block RLP
		let genesis_hash = BlockView::new(&genesis_block).parent_hash().sha3();

		BlockChain {
			genesis_block: genesis_block,
			genesis_hash: genesis_hash
		}
	}

	pub fn genesis_block(&self, db: &OverlayDB) -> Block {
		let root = BlockView::new(&self.genesis_block).state_root();

		if db.exists(&root) {
			return Block::new_existing(db.clone(), root)
		}

		let mut block = Block::new(db.clone());
		// TODO: commit genesis state (accounts) to block.state
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
		unimplemented!()
		// TODO: check is hash exist in hashes
	}
}
