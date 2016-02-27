use std::collections::HashSet;
use util::hash::H256;
use util::kvdb::DBTransaction;
use header::BlockNumber;
use blockchain::block_info::BlockInfo;

/// Block extras update info.
pub struct ExtrasUpdate {
	/// Block info.
	pub info: BlockInfo,
	/// DB update batch.
	pub batch: DBTransaction,
	/// Numbers of blocks to update in block hashes cache.
	pub block_numbers: HashSet<BlockNumber>,
	/// Hashes of blocks to update in block details cache.
	pub block_details_hashes: HashSet<H256>,
	/// Hashes of receipts to update in block receipts cache.
	pub block_receipts_hashes: HashSet<H256>,
	/// Hashes of transactions to update in transactions addresses cache.
	pub transactions_addresses_hashes: HashSet<H256>,
	/// Changed blocks bloom location hashes.
	pub bloom_hashes: HashSet<H256>,
}
