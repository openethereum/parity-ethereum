

/// Represents blockchain's in-memory cache size in bytes.
#[derive(Debug)]
pub struct CacheSize {
	/// Blocks cache size.
	pub blocks: usize,
	/// BlockDetails cache size.
	pub block_details: usize,
	/// Transaction addresses cache size.
	pub transaction_addresses: usize,
	/// Logs cache size.
	pub block_logs: usize,
	/// Blooms cache size.
	pub blocks_blooms: usize,
	/// Block receipts size.
	pub block_receipts: usize,
}

impl CacheSize {
	/// Total amount used by the cache.
	pub fn total(&self) -> usize { self.blocks + self.block_details + self.transaction_addresses + self.block_logs + self.blocks_blooms }
}
