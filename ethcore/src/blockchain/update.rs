use std::collections::HashMap;
use util::numbers::H256;
use header::BlockNumber;
use blockchain::block_info::BlockInfo;
use blooms::BloomGroup;
use super::extras::{BlockDetails, BlockReceipts, TransactionAddress, LogGroupPosition};

/// Block extras update info.
pub struct ExtrasUpdate {
	/// Block info.
	pub info: BlockInfo,
	/// Modified block hashes.
	pub block_hashes: HashMap<BlockNumber, H256>,
	/// Modified block details.
	pub block_details: HashMap<H256, BlockDetails>,
	/// Modified block receipts.
	pub block_receipts: HashMap<H256, BlockReceipts>,
	/// Modified transaction addresses.
	pub transactions_addresses: HashMap<H256, TransactionAddress>,
	/// Modified blocks blooms.
	pub blocks_blooms: HashMap<LogGroupPosition, BloomGroup>,
}
