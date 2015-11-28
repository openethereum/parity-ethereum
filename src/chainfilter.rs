//! basic implementation of multilevel bloom filter
use std::collections::{HashMap, HashSet};
use hash::*;
use filter::*;
use sha3::*;
use num::pow;

pub struct MemoryCache {
	blooms: HashMap<BloomIndex, H2048>,
}

impl MemoryCache {
	pub fn new() -> MemoryCache {
		MemoryCache { blooms: HashMap::new() }
	}

	pub fn insert_blooms(&mut self, blooms: HashMap<BloomIndex, H2048>) {
		self.blooms.extend(blooms);
	}
}

impl FilterDataSource for MemoryCache {
	fn bloom_at_index(&self, index: &BloomIndex) -> Option<&H2048> {
		self.blooms.get(index)
	}
}

pub struct ChainFilter<'a, D>
	where D: FilterDataSource + 'a
{
	data_source: &'a D,
	index_size: usize,
	levels: u8,
	level_sizes: HashMap<u8, usize>,
}

impl<'a, D> ChainFilter<'a, D> where D: FilterDataSource
{
	/// creates new filter instance
	pub fn new(data_source: &'a D, index_size: usize, levels: u8) -> Self {
		let mut filter = ChainFilter {
			data_source: data_source,
			index_size: index_size,
			levels: levels,
			level_sizes: HashMap::new(),
		};

		// cache level sizes, so we do not have to calculate them all the time
		for i in 0..levels {
			filter.level_sizes.insert(i, pow(index_size, i as usize));
		}

		filter
	}

	/// unsafely get level size
	fn level_size(&self, level: u8) -> usize {
		*self.level_sizes.get(&level).unwrap()
	}

	/// converts block number and level to `BloomIndex`
	fn bloom_index(&self, block_number: usize, level: u8) -> BloomIndex {
		BloomIndex {
			level: level,
			index: block_number / self.level_size(level),
		}
	}

	/// return bloom which are dependencies for given index
	fn lower_level_bloom_indexes(&self, index: &BloomIndex) -> HashSet<BloomIndex> {
		let mut indexes: HashSet<BloomIndex> = HashSet::with_capacity(self.index_size);

		// this is the lower level
		if index.level == 0 {
			return indexes;
		}

		let new_level = index.level - 1;
		let offset = self.index_size * index.index;

		for i in 0..self.index_size {
			indexes.insert(BloomIndex {
				level: new_level,
				index: offset + i,
			});
		}

		indexes
	}
}

impl<'a, D> Filter for ChainFilter<'a, D> where D: FilterDataSource
{
	/// add new bloom to all levels 
	///
	/// BitOr new bloom with all levels of filter
	fn add_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048> {
		let mut result: HashMap<BloomIndex, H2048> = HashMap::new();

		for level in 0..self.levels {
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

		for level in 0..self.levels {
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

		for level in 1..self.levels {
			let index = self.bloom_index(block_number, level);
			let lower_indexes = self.lower_level_bloom_indexes(&index);
			let new_bloom = lower_indexes.into_iter()
			                             .filter(|li| li != &reset_index)
			                             .map(|li| self.data_source.bloom_at_index(&li))
			                             .filter_map(|b| b)
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
		panic!();
	}
}

#[cfg(test)]
mod tests {
	use std::collections::{HashMap, HashSet};
	use hash::*;
	use filter::*;
	use chainfilter::*;

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

		let mut ebis = HashSet::with_capacity(16);
		for i in 16..32 {
			ebis.insert(BloomIndex::new(1, i));
		}

		let bis = filter.lower_level_bloom_indexes(&bi);
		assert_eq!(ebis, bis);
	}
}
