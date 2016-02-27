use std::collections::HashMap;
use util::hash::H256;
use header::BlockNumber;
use blockchain::block_info::BlockInfo;
use extras::{BlockDetails, BlockReceipts, TransactionAddress, BlocksBlooms};

/// Block extras update info.
pub struct ExtrasUpdate {
	/// Block info.
	pub info: BlockInfo,
	/// Numbers of blocks to update in block hashes cache.
	pub block_hashes: HashMap<BlockNumber, H256>,
	/// Hashes of blocks to update in block details cache.
	pub block_details: HashMap<H256, BlockDetails>,
	/// Hashes of receipts to update in block receipts cache.
	pub block_receipts: HashMap<H256, BlockReceipts>,
	/// Hashes of transactions to update in transactions addresses cache.
	pub transactions_addresses: HashMap<H256, TransactionAddress>,
	/// Changed blocks bloom location hashes.
	pub blocks_blooms: HashMap<H256, BlocksBlooms>,
}
