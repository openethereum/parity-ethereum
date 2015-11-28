//! basic implementation of multilevel bloom filter
use std::collections::{HashMap};
use hash::*;
use filter::*;
use sha3::*;
use num::pow;

/// in memory cache for blooms
pub struct MemoryCache {
	blooms: HashMap<BloomIndex, H2048>,
}

impl MemoryCache {
	pub fn new() -> MemoryCache {
		MemoryCache { blooms: HashMap::new() }
	}

	/// inserts all blooms into cache
	/// 
	/// if bloom at given index already exists, overwrites it
	pub fn insert_blooms(&mut self, blooms: HashMap<BloomIndex, H2048>) {
		self.blooms.extend(blooms);
	}
}

impl FilterDataSource for MemoryCache {
	fn bloom_at_index(&self, index: &BloomIndex) -> Option<&H2048> {
		self.blooms.get(index)
	}
}

/// Should be used to find blocks in FilterDataSource
pub struct ChainFilter<'a, D>
	where D: FilterDataSource + 'a
{
	data_source: &'a D,
	index_size: usize,
	level_sizes: Vec<usize>,
}

impl<'a, D> ChainFilter<'a, D> where D: FilterDataSource
{
	/// creates new filter instance
	pub fn new(data_source: &'a D, index_size: usize, levels: u8) -> Self {
		if levels == 0 {
			panic!("ChainFilter requires and least 1 level");
		}

		let mut filter = ChainFilter {
			data_source: data_source,
			index_size: index_size,
			level_sizes: vec![]
		};

		// cache level sizes, so we do not have to calculate them all the time
		for i in 0..levels {
			filter.level_sizes.push(pow(index_size, i as usize));
		}

		filter
	}

	/// unsafely get level size
	fn level_size(&self, level: u8) -> usize {
		self.level_sizes[level as usize]
	}

	/// converts block number and level to `BloomIndex`
	fn bloom_index(&self, block_number: usize, level: u8) -> BloomIndex {
		BloomIndex {
			level: level,
			index: block_number / self.level_size(level),
		}
	}

	/// return bloom which are dependencies for given index
	/// 
	/// bloom indexes are ordered from lowest to highest
	fn lower_level_bloom_indexes(&self, index: &BloomIndex) -> Vec<BloomIndex> {
		let mut indexes: Vec<BloomIndex> = vec![];

		// this is the lower level
		if index.level == 0 {
			return indexes;
		}

		let new_level = index.level - 1;
		let offset = self.index_size * index.index;

		for i in 0..self.index_size {
			indexes.push(BloomIndex {
				level: new_level,
				index: offset + i,
			});
		}

		indexes
	}

	/// return number of levels
	fn levels(&self) -> u8 {
		self.level_sizes.len() as u8
	}

	/// returns max filter level
	fn max_level(&self) -> u8 {
		self.level_sizes.len() as u8 - 1
	}

	/// internal function which actually does bloom search
	/// TODO: optimize it, maybe non-recursive version?
	/// TODO: clean up?
	fn blocks(&self, bloom: &H2048, from_block: usize, to_block: usize, level: u8, offset: usize) -> Vec<usize> {
		let index = self.bloom_index(offset, level);

		let contains = match self.data_source.bloom_at_index(&index) {
			None => false,
			Some(level_bloom) => match level {
				// if we are on the lowest level
				// take the value, exclude to_block
				0 if offset < to_block => return vec![offset],
				0 => false,
				_ => level_bloom.contains(bloom)
			}
		};

		if contains {
			let level_size = self.level_size(level - 1);
			let from_index = self.bloom_index(from_block, level - 1);
			let to_index = self.bloom_index(to_block, level - 1);
			let res: Vec<usize> = self.lower_level_bloom_indexes(&index).into_iter()
				// chose only blooms in range
				.filter(|li| li.index >= from_index.index && li.index <= to_index.index)
				// map them to offsets
				.map(|li| li.index * level_size)
				// get all blocks that may contain our bloom
				.map(|off| self.blocks(bloom, from_block, to_block, level - 1, off))
				// flatten nested structure
				.flat_map(|v| v)
				.collect();
			return res
		}

		return vec![];
	}
}

impl<'a, D> Filter for ChainFilter<'a, D> where D: FilterDataSource
{
	/// add new bloom to all levels 
	///
	/// BitOr new bloom with all levels of filter
	fn add_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048> {
		let mut result: HashMap<BloomIndex, H2048> = HashMap::new();

		for level in 0..self.levels() {
			let bloom_index = self.bloom_index(block_number, level);
			let new_bloom = match self.data_source.bloom_at_index(&bloom_index) {
				Some(old_bloom) => old_bloom | bloom,
				None => bloom.clone(),
			};

			result.insert(bloom_index, new_bloom);
		}

		result
	}

	/// add new blooms starting from block number
	/// 
	/// BitOr new blooms with all levels of filter
	fn add_blooms(&self, blooms: &[H2048], block_number: usize) -> HashMap<BloomIndex, H2048> {
		let mut result: HashMap<BloomIndex, H2048> = HashMap::new();

		for level in 0..self.levels() {
			for i in 0..blooms.len() {
				let bloom_index = self.bloom_index(block_number + i, level);
				let is_new_bloom = match result.get_mut(&bloom_index) {

					// it was already modified
					Some(to_shift) => {
						*to_shift = &blooms[i] | to_shift;
						false
					}
					None => true,
				};

				// it hasn't been modified yet
				if is_new_bloom {
					let new_bloom = match self.data_source.bloom_at_index(&bloom_index) {
						Some(old_bloom) => old_bloom | &blooms[i],
						None => blooms[i].clone(),
					};
					result.insert(bloom_index, new_bloom);
				}
			}
		}

		result
	}

	/// reset bloom at level 0 and forces rebuild on higher levels
	fn reset_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048> {
		let mut result: HashMap<BloomIndex, H2048> = HashMap::new();

		let mut reset_index = self.bloom_index(block_number, 0);
		result.insert(reset_index.clone(), bloom.clone());

		for level in 1..self.levels() {
			let index = self.bloom_index(block_number, level);
			// get all bloom indexes that were used to construct this bloom
			let lower_indexes = self.lower_level_bloom_indexes(&index);
			let new_bloom = lower_indexes.into_iter()
										 // skip reseted one
			                             .filter(|li| li != &reset_index)
										 // get blooms for these indexes
			                             .map(|li| self.data_source.bloom_at_index(&li))
										 // filter existing ones
			                             .filter_map(|b| b)
										 // BitOr all of them
			                             .fold(H2048::new(), |acc, bloom| &acc | bloom);

			reset_index = index.clone();
			result.insert(index, &new_bloom | bloom);
		}

		result
	}

	/// sets lowest level bloom to 0 and forces rebuild on higher levels
	fn clear_bloom(&self, block_number: usize) -> HashMap<BloomIndex, H2048> {
		self.reset_bloom(&H2048::new(), block_number)
	}

	/// returns numbers of blocks that may contain Address
	fn blocks_with_address(&self,
	                       address: &Address,
	                       from_block: usize,
	                       to_block: usize)
	                       -> Vec<usize> {
		let mut bloom = H2048::new();
		bloom.shift_bloom(&address.sha3());
		self.blocks_with_bloom(&bloom, from_block, to_block)
	}

	/// returns numbers of blocks that may contain Topic
	fn blocks_with_topics(&self, topic: &H256, from_block: usize, to_block: usize) -> Vec<usize> {
		let mut bloom = H2048::new();
		bloom.shift_bloom(&topic.sha3());
		self.blocks_with_bloom(&bloom, from_block, to_block)
	}

	/// returns numbers of blocks that may log bloom
	fn blocks_with_bloom(&self, bloom: &H2048, from_block: usize, to_block: usize) -> Vec<usize> {
		let mut result = vec![];
		// lets start from highest level
		let max_level = self.max_level();
		let level_size = self.level_size(max_level);
		let from_index = self.bloom_index(from_block, max_level);
		let to_index = self.bloom_index(to_block, max_level);

		for index in from_index.index..to_index.index + 1 {
			// offset will be used to calculate where we are right now
			let offset = level_size * index;

			// go doooown!
			result.extend(self.blocks(bloom, from_block, to_block, max_level, offset));
		}

		result
	}
}

#[cfg(test)]
mod tests {
	use hash::*;
	use filter::*;
	use chainfilter::*;
	use sha3::*;
	use std::str::FromStr;

	#[test]
	fn test_level_size() {
		let cache = MemoryCache::new();
		let filter = ChainFilter::new(&cache, 16, 3);
		assert_eq!(filter.level_size(0), 1);
		assert_eq!(filter.level_size(1), 16);
		assert_eq!(filter.level_size(2), 256);
	}

	#[test]
	fn test_bloom_index() {
		let cache = MemoryCache::new();
		let filter = ChainFilter::new(&cache, 16, 3);

		let bi0 = filter.bloom_index(0, 0);
		assert_eq!(bi0.level, 0);
		assert_eq!(bi0.index, 0);

		let bi1 = filter.bloom_index(1, 0);
		assert_eq!(bi1.level, 0);
		assert_eq!(bi1.index, 1);

		let bi2 = filter.bloom_index(2, 0);
		assert_eq!(bi2.level, 0);
		assert_eq!(bi2.index, 2);

		let bi3 = filter.bloom_index(3, 1);
		assert_eq!(bi3.level, 1);
		assert_eq!(bi3.index, 0);

		let bi4 = filter.bloom_index(15, 1);
		assert_eq!(bi4.level, 1);
		assert_eq!(bi4.index, 0);

		let bi5 = filter.bloom_index(16, 1);
		assert_eq!(bi5.level, 1);
		assert_eq!(bi5.index, 1);

		let bi6 = filter.bloom_index(255, 2);
		assert_eq!(bi6.level, 2);
		assert_eq!(bi6.index, 0);

		let bi7 = filter.bloom_index(256, 2);
		assert_eq!(bi7.level, 2);
		assert_eq!(bi7.index, 1);
	}

	#[test]
	fn test_lower_level_bloom_indexes() {
		let cache = MemoryCache::new();
		let filter = ChainFilter::new(&cache, 16, 3);

		let bi = filter.bloom_index(256, 2);
		assert_eq!(bi.level, 2);
		assert_eq!(bi.index, 1);

		let mut ebis = vec![];
		for i in 16..32 {
			ebis.push(BloomIndex::new(1, i));
		}

		let bis = filter.lower_level_bloom_indexes(&bi);
		assert_eq!(ebis, bis);
	}

	#[test]
	fn test_topic_basic_search() {
		let index_size = 16;
		let bloom_levels = 3;

		let mut cache = MemoryCache::new();
		let topic = H256::from_str("8d936b1bd3fc635710969ccfba471fb17d598d9d1971b538dd712e1e4b4f4dba").unwrap();

		let modified_blooms = {
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let block_number = 23;
			let mut bloom = H2048::new();
			bloom.shift_bloom(&topic.sha3());
			filter.add_bloom(&bloom, block_number)
		};

		// number of modified blooms should always be equal number of levels
		assert_eq!(modified_blooms.len(), bloom_levels as usize);
		cache.insert_blooms(modified_blooms);

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topics(&topic, 0, 100);
			assert_eq!(blocks.len(), 1);
			assert_eq!(blocks[0], 23);
		}

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topics(&topic, 0, 23);
			assert_eq!(blocks.len(), 0);
		}

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topics(&topic, 23, 24);
			assert_eq!(blocks.len(), 1);
			assert_eq!(blocks[0], 23);
		}

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topics(&topic, 24, 100);
			assert_eq!(blocks.len(), 0);
		}
	}
}
