use std::path::Path;
use util::uint::U256;
use util::hash::*;
use util::sha3::*;
use util::rlp::*;
use util::bytes::Bytes;
use blockchain::BlockChain;
use views::BlockView;

/// Status for a block in a queue.
pub enum QueueStatus {
	/// Part of the known chain.
	Known,
	/// Part of the unknown chain.
	Unknown,
}

/// General block status
pub enum BlockStatus {
	/// Part of the blockchain.
	InChain,
	/// Queued for import.
	Queued(QueueStatus),
	/// Known as bad.
	Bad,
	/// Unknown.
	Unknown,
}

/// Result of import block operation.
pub enum ImportResult {
	/// Added to import queue.
	Queued(QueueStatus),
	/// Already in the chain.
	AlreadyInChain,
	/// Already queued for import.
	AlreadyQueued(QueueStatus),
	/// Bad or already known as bad.
	Bad,
}

/// Information about the blockchain gthered together.
pub struct BlockChainInfo {
	/// Blockchain difficulty.
	pub total_difficulty: U256,
	/// Block queue difficulty.
	pub pending_total_difficulty: U256,
	/// Genesis block hash.
	pub genesis_hash: H256,
	/// Best blockchain block hash.
	pub best_block_hash: H256,
	/// Best blockchain block number.
	pub best_block_number: BlockNumber
}

/// Block queue status
pub struct BlockQueueStatus {
	pub full: bool,
}

pub type TreeRoute = ::blockchain::TreeRoute;

pub type BlockNumber = u64;

/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient : Sync {
	/// Get raw block header data by block header hash.
	fn block_header(&self, hash: &H256) -> Option<Bytes>;

	/// Get raw block body data by block header hash.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body(&self, hash: &H256) -> Option<Bytes>;

	/// Get raw block data by block header hash.
	fn block(&self, hash: &H256) -> Option<Bytes>;

	/// Get block status by block header hash.
	fn block_status(&self, hash: &H256) -> BlockStatus;

	/// Get raw block header data by block number.
	fn block_header_at(&self, n: BlockNumber) -> Option<Bytes>;

	/// Get raw block body data by block number.
	/// Block body is an RLP list of two items: uncles and transactions.
	fn block_body_at(&self, n: BlockNumber) -> Option<Bytes>;

	/// Get raw block data by block number.
	fn block_at(&self, n: BlockNumber) -> Option<Bytes>;

	/// Get block status by block number.
	fn block_status_at(&self, n: BlockNumber) -> BlockStatus;

	/// Get a tree route between `from` and `to`.
	/// See `BlockChain::tree_route`.
	fn tree_route(&self, from: &H256, to: &H256) -> TreeRoute;

	/// Get latest state node
	fn state_data(&self, hash: &H256) -> Option<Bytes>;

	/// Get raw block receipts data by block header hash.
	fn block_receipts(&self, hash: &H256) -> Option<Bytes>;

	/// Import a block into the blockchain.
	fn import_block(&mut self, byte: &[u8]) -> ImportResult;

	/// Get block queue information.
	fn queue_status(&self) -> BlockQueueStatus;

	/// Clear block qeueu and abort all import activity.
	fn clear_queue(&mut self);

	/// Get blockchain information.
	fn chain_info(&self) -> BlockChainInfo;
}

/// Blockchain database client backed by a persistent database. Owns and manages a blockchain and a block queue.
pub struct Client {
	chain: BlockChain
}

impl Client {
	pub fn new(genesis: &[u8], path: &Path) -> Client {
		Client {
			chain: BlockChain::new(genesis, path)
		}
	}
}

impl BlockChainClient for Client {
	fn block_header(&self, hash: &H256) -> Option<Bytes> {
		self.chain.block(hash).map(|bytes| BlockView::new(&bytes).rlp().at(0).raw().to_vec())
	}

	fn block_body(&self, hash: &H256) -> Option<Bytes> {
		self.chain.block(hash).map(|bytes| {
			let rlp = Rlp::new(&bytes);
			let mut body = RlpStream::new();
			body.append_raw(rlp.at(1).raw(), 1);
			body.append_raw(rlp.at(2).raw(), 1);
			body.out()
		})
	}

	fn block(&self, hash: &H256) -> Option<Bytes> {
		self.chain.block(hash)
	}

	fn block_status(&self, hash: &H256) -> BlockStatus {
		if self.chain.is_known(&hash) { BlockStatus::InChain } else { BlockStatus::Unknown }
	}

	fn block_header_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.chain.block_hash(&From::from(n)).and_then(|h| self.block_header(&h))
	}

	fn block_body_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.chain.block_hash(&From::from(n)).and_then(|h| self.block_body(&h))
	}

	fn block_at(&self, n: BlockNumber) -> Option<Bytes> {
		self.chain.block_hash(&From::from(n)).and_then(|h| self.block(&h))
	}

	fn block_status_at(&self, n: BlockNumber) -> BlockStatus {
		match self.chain.block_hash(&From::from(n)) {
			Some(h) => self.block_status(&h),
			None => BlockStatus::Unknown
		}
	}

	fn tree_route(&self, from: &H256, to: &H256) -> TreeRoute {
		self.chain.tree_route(from.clone(), to.clone())
	}

	fn state_data(&self, _hash: &H256) -> Option<Bytes> {
		unimplemented!();
	}

	fn block_receipts(&self, _hash: &H256) -> Option<Bytes> {
		unimplemented!();
	}

	fn import_block(&mut self, bytes: &[u8]) -> ImportResult {
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

	fn queue_status(&self) -> BlockQueueStatus {
		BlockQueueStatus {
			full: false
		}
	}

	fn clear_queue(&mut self) {
	}

	fn chain_info(&self) -> BlockChainInfo {
		BlockChainInfo {
			total_difficulty: self.chain.best_block_total_difficulty(),
			pending_total_difficulty: self.chain.best_block_total_difficulty(),
			genesis_hash: self.chain.genesis_hash(),
			best_block_hash: self.chain.best_block_hash(),
			best_block_number: From::from(self.chain.best_block_number())
		}
	}
}
