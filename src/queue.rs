use std::sync::Arc;
//use util::bytes::*;
use util::sha3::*;
use blockchain::BlockChain;
use client::{QueueStatus, ImportResult};
use views::{BlockView};


pub struct BlockQueue {
	chain: Arc<BlockChain>
}

impl BlockQueue {
	pub fn new(chain: Arc<BlockChain>) -> BlockQueue {
		BlockQueue {
			chain: chain
		}
	}

	pub fn clear(&mut self) {
	}

	pub fn import_block(&mut self, bytes: &[u8]) -> ImportResult {
		//TODO: verify block
		{
			let block = BlockView::new(bytes);
			let header = block.header_view();
			let hash = header.sha3();
			if self.chain.is_known(&hash) {
				return ImportResult::Bad;
			}
		}
		self.chain.insert_block(bytes);
		ImportResult::Queued(QueueStatus::Known)
	}
}

