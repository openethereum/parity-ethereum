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
//! ```not_run
//! extern crate ethcore_util as util;
//! extern crate ethcore;
//! use std::str::FromStr;
//! use util::sha3::*;
//! use util::hash::*;
//! use ethcore::chainfilter::*;
//!
//! fn main() {
//! 		let (index_size, bloom_levels) = (16, 3);
//! 		let mut cache = MemoryCache::new();
//!
//! 		let address = Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap();
//!
//! 		// borrow cache for reading inside the scope
//! 		let modified_blooms = {
//! 			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
//! 			let block_number = 39;
//! 			let mut bloom = H2048::new();
//! 			bloom.shift_bloomed(&address.sha3());
//! 			filter.add_bloom(&bloom, block_number)
//! 		};
//!
//! 		// number of updated blooms is equal number of levels
//! 		assert_eq!(modified_blooms.len(), bloom_levels as usize);
//!
//! 		// lets inserts modified blooms into the cache
//! 		cache.insert_blooms(modified_blooms);
//!
//! 		// borrow cache for another reading operations
//! 		{
//! 			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
//! 			let blocks = filter.blocks_with_address(&address, 10, 40);
//! 			assert_eq!(blocks.len(), 1);
//! 			assert_eq!(blocks[0], 39);
//! 		}
//! }
//! ```
//!
use std::collections::HashMap;
use util::hash::*;
use chainfilter::{BloomIndex, FilterDataSource};
use chainfilter::indexer::Indexer;

/// Should be used for search operations on blockchain.
pub struct ChainFilter<'a, D>
	where D: FilterDataSource + 'a,
{
	data_source: &'a D,
	indexer: Indexer,
}

impl<'a, D> ChainFilter<'a, D>
    where D: FilterDataSource,
{
	/// Creates new filter instance.
	///
	/// Borrows `FilterDataSource` for reading.
	pub fn new(data_source: &'a D, index_size: usize, levels: u8) -> Self {
		ChainFilter {
			data_source: data_source,
			indexer: Indexer::new(index_size, levels),
		}
	}

	/// internal function which does bloom search recursively
	fn blocks(&self, bloom: &H2048, from_block: usize, to_block: usize, level: u8, offset: usize) -> Option<Vec<usize>> {
		let index = self.indexer.bloom_index(offset, level);

		match self.data_source.bloom_at_index(&index) {
			None => return None,
			Some(level_bloom) => match level {
				// if we are on the lowest level
				0 => return match offset <= to_block {
					// take the value if its smaller than to_block
					true if level_bloom.contains(bloom) => Some(vec![offset]),
					// return None if it is is equal to to_block
					_ => None,
				},
				// return None if current level doesnt contain given bloom
				_ if !level_bloom.contains(bloom) => return None,
				// continue processing && go down
				_ => (),
			},
		};

		let level_size = self.indexer.level_size(level - 1);
		let from_index = self.indexer.bloom_index(from_block, level - 1);
		let to_index = self.indexer.bloom_index(to_block, level - 1);
		let res: Vec<usize> = self.indexer.lower_level_bloom_indexes(&index).into_iter()
			// chose only blooms in range
			.filter(|li| li.index >= from_index.index && li.index <= to_index.index)
			// map them to offsets
			.map(|li| li.index * level_size)
			// get all blocks that may contain our bloom
			// filter existing ones
			.filter_map(|off| self.blocks(bloom, from_block, to_block, level - 1, off))
			// flatten nested structures
			.flat_map(|v| v)
			.collect();
		Some(res)
	}

	/// Adds new bloom to all filter levels
	pub fn add_bloom(&self, bloom: &H2048, block_number: usize) -> HashMap<BloomIndex, H2048> {
		let mut result: HashMap<BloomIndex, H2048> = HashMap::new();

		for level in 0..self.indexer.levels() {
			let bloom_index = self.indexer.bloom_index(block_number, level);
			let new_bloom = match self.data_source.bloom_at_index(&bloom_index) {
				Some(old_bloom) => old_bloom | bloom.clone(),
				None => bloom.clone(),
			};

			result.insert(bloom_index, new_bloom);
		}

		result
	}

	/// Resets blooms at level 0 and forces rebuild on higher levels.
	pub fn reset_chain_head(&self, blooms: &[H2048], block_number: usize, old_highest_block: usize) -> HashMap<BloomIndex, H2048> {
		let mut result: HashMap<BloomIndex, H2048> = HashMap::new();

		// insert all new blooms at level 0
		for (i, bloom) in blooms.iter().enumerate() {
			result.insert(self.indexer.bloom_index(block_number + i, 0), bloom.clone());
		}

		// reset the rest of blooms
		for reset_number in block_number + blooms.len()..(old_highest_block + 1) {
			result.insert(self.indexer.bloom_index(reset_number, 0), H2048::new());
		}

		for level in 1..self.indexer.levels() {
			for i in 0..blooms.len() {

				let index = self.indexer.bloom_index(block_number + i, level);
				let new_bloom = {
					// use new blooms before db blooms where necessary
					let bloom_at = |index| result.get(&index).cloned().or_else(|| self.data_source.bloom_at_index(&index));

					self.indexer.lower_level_bloom_indexes(&index)
						.into_iter()
						// get blooms
						// filter existing ones
						.filter_map(bloom_at)
						// BitOr all of them
						.fold(H2048::new(), |acc, bloom| acc | bloom)
				};

				result.insert(index, new_bloom);
			}
		}

		result
	}

	/// Returns numbers of blocks that may log bloom.
	pub fn blocks_with_bloom(&self, bloom: &H2048, from_block: usize, to_block: usize) -> Vec<usize> {
		let mut result = vec![];
		// lets start from highest level
		let max_level = self.indexer.max_level();
		let level_size = self.indexer.level_size(max_level);
		let from_index = self.indexer.bloom_index(from_block, max_level);
		let to_index = self.indexer.bloom_index(to_block, max_level);

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
