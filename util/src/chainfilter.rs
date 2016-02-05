// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Multilevel blockchain bloom filter.
//! 
//! ```
//! extern crate ethcore_util as util;
//! use std::str::FromStr;
//! use util::chainfilter::*;
//! use util::sha3::*;
//! use util::hash::*;
//! 
//! fn main() {
//!		let (index_size, bloom_levels) = (16, 3);
//!		let mut cache = MemoryCache::new();
//!		
//!		let address = Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap();
//!		
//!		// borrow cache for reading inside the scope
//!		let modified_blooms = {
//!			let filter = ChainFilter::new(&cache, index_size, bloom_levels);	
//!			let block_number = 39;
//!			let mut bloom = H2048::new();
//!			bloom.shift_bloomed(&address.sha3());
//!			filter.add_bloom(&bloom, block_number)
//!		};
//!		
//!		// number of updated blooms is equal number of levels
//!		assert_eq!(modified_blooms.len(), bloom_levels as usize);
//!
//!		// lets inserts modified blooms into the cache
//!		cache.insert_blooms(modified_blooms);
//!		
//!		// borrow cache for another reading operations
//!		{
//!			let filter = ChainFilter::new(&cache, index_size, bloom_levels);	
//!			let blocks = filter.blocks_with_address(&address, 10, 40);
//!			assert_eq!(blocks.len(), 1);	
//!			assert_eq!(blocks[0], 39);
//!		}
//! }
//! ```
//!
use std::collections::{HashMap};
use hash::*;
use sha3::*;

/// Represents bloom index in cache
/// 
/// On cache level 0, every block bloom is represented by different index.
/// On higher cache levels, multiple block blooms are represented by one
/// index. Their `BloomIndex` can be created from block number and given level.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct BloomIndex {
	/// Bloom level
	pub level: u8,
	///  Filter Index
	pub index: usize,
}

impl BloomIndex {
	/// Default constructor for `BloomIndex`
	pub fn new(level: u8, index: usize) -> BloomIndex {
		BloomIndex {
			level: level,
			index: index,
		}
	}
}

/// Types implementing this trait should provide read access for bloom filters database.
pub trait FilterDataSource {
	/// returns reference to log at given position if it exists
	fn bloom_at_index(&self, index: &BloomIndex) -> Option<&H2048>;
}

/// In memory cache for blooms.
/// 
/// Stores all blooms in HashMap, which indexes them by `BloomIndex`.
pub struct MemoryCache {
	blooms: HashMap<BloomIndex, H2048>,
}

impl MemoryCache {
	/// Default constructor for MemoryCache
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

/// Should be used for search operations on blockchain.
pub struct ChainFilter<'a, D>
	where D: FilterDataSource + 'a
{
	data_source: &'a D,
	index_size: usize,
	level_sizes: Vec<usize>,
}

impl<'a, D> ChainFilter<'a, D> where D: FilterDataSource
{
	/// Creates new filter instance.
	/// 
	/// Borrows `FilterDataSource` for reading.
	pub fn new(data_source: &'a D, index_size: usize, levels: u8) -> Self {
		if levels == 0 {
			panic!("ChainFilter requires at least 1 level");
		}

		let mut filter = ChainFilter {
			data_source: data_source,
			index_size: index_size,
			// 0 level has always a size of 1
			level_sizes: vec![1]
		};

		// cache level sizes, so we do not have to calculate them all the time
		// eg. if levels == 3, index_size = 16
		// level_sizes = [1, 16, 256]
		let additional: Vec<usize> = (1..).into_iter()
			.scan(1, |acc, _| {
				*acc = *acc * index_size; 
				Some(*acc)
			})
			.take(levels as usize - 1)
			.collect();
		filter.level_sizes.extend(additional);

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
		// this is the lowest level
		if index.level == 0 {
			return vec![];
		}

		let new_level = index.level - 1;
		let offset = self.index_size * index.index;

		(0..self.index_size).map(|i| BloomIndex::new(new_level, offset + i)).collect()
	}

	/// return number of levels
	fn levels(&self) -> u8 {
		self.level_sizes.len() as u8
	}

	/// returns max filter level
	fn max_level(&self) -> u8 {
		self.level_sizes.len() as u8 - 1
	}

	/// internal function which does bloom search recursively
	fn blocks(&self, bloom: &H2048, from_block: usize, to_block: usize, level: u8, offset: usize) -> Option<Vec<usize>> {
		let index = self.bloom_index(offset, level);

		match self.data_source.bloom_at_index(&index) {
			None => return None,
			Some(level_bloom) => match level {
				// if we are on the lowest level
				// take the value, exclude to_block
				0 if offset < to_block => return Some(vec![offset]),
				// return None if it is is equal to to_block
				0 => return None,
				// return None if current level doesnt contain given bloom
				_ if !level_bloom.contains(bloom) => return None,
				// continue processing && go down
				_ => ()
			}
		};

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
			// filter existing ones
			.filter_map(|x| x)
			// flatten nested structures
			.flat_map(|v| v)
			.collect();
		Some(res)
	}

	/// Adds new bloom to all filter levels
	pub fn add_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048> {
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

	/// Adds new blooms starting from block number.
	pub fn add_blooms(&self, blooms: &[H2048], block_number: usize) -> HashMap<BloomIndex, H2048> {
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

	/// Resets bloom at level 0 and forces rebuild on higher levels.
	pub fn reset_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048> {
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

	/// Sets lowest level bloom to 0 and forces rebuild on higher levels.
	pub fn clear_bloom(&self, block_number: usize) -> HashMap<BloomIndex, H2048> {
		self.reset_bloom(&H2048::new(), block_number)
	}

	/// Returns numbers of blocks that may contain Address.
	pub fn blocks_with_address(&self, address: &Address, from_block: usize, to_block: usize) -> Vec<usize> {
		let mut bloom = H2048::new();
		bloom.shift_bloomed(&address.sha3());
		self.blocks_with_bloom(&bloom, from_block, to_block)
	}

	/// Returns numbers of blocks that may contain Topic.
	pub fn blocks_with_topic(&self, topic: &H256, from_block: usize, to_block: usize) -> Vec<usize> {
		let mut bloom = H2048::new();
		bloom.shift_bloomed(&topic.sha3());
		self.blocks_with_bloom(&bloom, from_block, to_block)
	}

	/// Returns numbers of blocks that may log bloom.
	pub fn blocks_with_bloom(&self, bloom: &H2048, from_block: usize, to_block: usize) -> Vec<usize> {
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
			if let Some(blocks) = self.blocks(bloom, from_block, to_block, max_level, offset) {
				result.extend(blocks);
			}
		}

		result
	}
}

#[cfg(test)]
mod tests {
	use hash::*;
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
			bloom.shift_bloomed(&topic.sha3());
			filter.add_bloom(&bloom, block_number)
		};

		// number of modified blooms should always be equal number of levels
		assert_eq!(modified_blooms.len(), bloom_levels as usize);
		cache.insert_blooms(modified_blooms);

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topic(&topic, 0, 100);
			assert_eq!(blocks.len(), 1);
			assert_eq!(blocks[0], 23);
		}

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topic(&topic, 0, 23);
			assert_eq!(blocks.len(), 0);
		}

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topic(&topic, 23, 24);
			assert_eq!(blocks.len(), 1);
			assert_eq!(blocks[0], 23);
		}

		{
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			let blocks = filter.blocks_with_topic(&topic, 24, 100);
			assert_eq!(blocks.len(), 0);
		}
	}
}
