//! multilevel bloom filter interface
use hash::*;
use std::collections::HashMap;

/// Represents bloom index in cache
/// 
/// On bloom level 0, all positions represent different blooms. 
/// On higher levels multiple positions represent one bloom
/// and should be transformed to `BlockIndex` to get index of this bloom
#[derive(Eq, PartialEq, Hash)]
pub struct BloomIndex {
	level: u8,
	level_index: usize,
	index: usize,
}

pub trait FilterDataSource {
	/// returns reference to log at given position if it exists
	fn bloom_at_index(&self, index: &BloomIndex) -> Option<&H2048>;
}

pub trait Filter: Sized {
	/// creates new filter instance
	fn new<T>(data_source: &T, index_size: usize, levels: u8) -> Self where T: FilterDataSource;

	/// converts block number and level to `BloomIndex`
	fn bloom_index(&self, block_number: usize, level: u8) -> BloomIndex;

	/// add new bloom to all levels 
	fn add_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048>;

	/// add new blooms starting from block number
	fn add_blooms(&self, blooms: &[H2048], block_number: usize) -> HashMap<BloomIndex, H2048>;

	/// reset bloom at level 0 and forces rebuild on higher levels
	fn reset_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048>;

	/// sets lowest level bloom to 0 and forces rebuild on higher levels
	fn clear_bloom(&self, block_number: usize) -> HashMap<BloomIndex, H2048>;

	/// returns numbers of blocks that may contain Address
	fn blocks_with_address(&self, address: &Address) -> Vec<usize>;

	/// returns numbers of blocks that may contain Topic
	fn blocks_with_topics(&self, topic: &H256) -> Vec<usize>;

	/// returns numbers of blocks that may log bloom
	fn blocks_with_bloom(&self, bloom: &H2048) -> Vec<usize>;
}
