use std::sync::Arc;
use util::*;
use blockchain::BlockChain;
use client::{QueueStatus, ImportResult};
use views::{BlockView};

/// A queue of blocks. Sits between network or other I/O and the BlockChain.
/// Sorts them ready for blockchain insertion.
pub struct BlockQueue;

impl BlockQueue {
	/// Creates a new queue instance.
	pub fn new() -> BlockQueue {
	}

	/// Clear the queue and stop verification activity.
	pub fn clear(&mut self) {
	}

	/// Add a block to the queue.
	pub fn import_block(&mut self, bytes: &[u8], bc: &mut BlockChain) -> ImportResult {
		//TODO: verify block
		{
			let block = BlockView::new(bytes);
			let header = block.header_view();
			let hash = header.sha3();
			if self.chain.is_known(&hash) {
				return ImportResult::Bad;
			}
		}
		bc.insert_block(bytes);
		ImportResult::Queued(QueueStatus::Known)
	}
}

