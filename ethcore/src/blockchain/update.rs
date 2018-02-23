use std::collections::HashMap;
use ethereum_types::H256;
use header::BlockNumber;
use blockchain::block_info::BlockInfo;
use blockchain::extras::{BlockDetails, BlockReceipts, TransactionAddress};

/// Block extras update info.
pub struct ExtrasUpdate<'a> {
	/// Block info.
	pub info: BlockInfo,
	/// Block timestamp.
	pub timestamp: u64,
	/// Current block uncompressed rlp bytes
	pub block: &'a [u8],
	/// Modified block hashes.
	pub block_hashes: HashMap<BlockNumber, H256>,
	/// Modified block details.
	pub block_details: HashMap<H256, BlockDetails>,
	/// Modified block receipts.
	pub block_receipts: HashMap<H256, BlockReceipts>,
	/// Modified transaction addresses (None signifies removed transactions).
	pub transactions_addresses: HashMap<H256, Option<TransactionAddress>>,
}
