use util::hash::H256;
use util::bytes::Bytes;
use util::uint::U256;

pub enum QueueStatus {
	/// Part of the known chain
	Known,
	/// Part of the unknown chain
	Unknown,
}

pub enum BlockStatus {
	InChain,
	Queued(QueueStatus),
	Bad,
	Unknown,
}

pub enum ImportResult {
	Queued(QueueStatus),
	AlreadyInChain,
	AlreadyQueued(QueueStatus),
	Bad,
}

pub struct BlockChainInfo {
	pub total_difficulty: U256,
	pub pending_total_difficulty: U256,
	pub genesis_hash: H256,
	pub last_block_hash: H256,
	pub last_block_number: BlockNumber
}

pub struct BlockQueueStatus {
	pub full: bool,
}

pub type TreeRoute = ::blockchain::TreeRoute;

pub type BlockNumber = u32;
pub type BlockHeader = ::header::Header;

pub trait BlockChainClient : Sync {
	fn block_header(&self, h: &H256) -> Option<Bytes>;
	fn block_body(&self, h: &H256) -> Option<Bytes>;
	fn block(&self, h: &H256) -> Option<Bytes>;
	fn block_status(&self, h: &H256) -> BlockStatus;
	fn block_header_at(&self, n: BlockNumber) -> Option<Bytes>;
	fn block_body_at(&self, n: BlockNumber) -> Option<Bytes>;
	fn block_at(&self, n: BlockNumber) -> Option<Bytes>;
	fn block_status_at(&self, n: BlockNumber) -> BlockStatus;
	fn tree_route(&self, from: &H256, to: &H256) -> TreeRoute;
	fn state_data(&self, h: &H256) -> Option<Bytes>;
	fn block_receipts(&self, h: &H256) -> Option<Bytes>;
	fn import_block(&mut self, b: &[u8]) -> ImportResult;
	fn queue_status(&self) -> BlockQueueStatus;
	fn clear_queue(&mut self);
	fn info(&self) -> BlockChainInfo;
}
