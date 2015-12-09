use util::hash::*;
use util::rlp::*;
use util::hashdb::*;
use util::overlaydb::*;
use blockheader::*;
use block::*;

pub struct BlockChain {
	genesis_hash: H256,
	genesis_block: Vec<u8>
}

impl BlockChain {
	pub fn genesis_block(&self, db: &OverlayDB) -> Block {
		let root = BlockView::new(&self.genesis_block).state_root();
		if db.exists(&root) {
			return Block::new(db.clone())
		}
		unimplemented!()
	}
}
