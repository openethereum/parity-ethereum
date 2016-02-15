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

use std::collections::HashMap;
use std::str::FromStr;
use util::hash::*;
use util::sha3::*;
use chainfilter::{BloomIndex, FilterDataSource, ChainFilter};

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
	fn bloom_at_index(&self, index: &BloomIndex) -> Option<H2048> {
		self.blooms.get(index).cloned()
	}
}

fn topic_to_bloom(topic: &H256) -> H2048 {
	let mut bloom = H2048::new();
	bloom.shift_bloomed(&topic.sha3());
	bloom
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
		filter.add_bloom(&topic_to_bloom(&topic), block_number)
	};

	// number of modified blooms should always be equal number of levels
	assert_eq!(modified_blooms.len(), bloom_levels as usize);
	cache.insert_blooms(modified_blooms);

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&topic_to_bloom(&topic), 0, 100);
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0], 23);
	}

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&topic_to_bloom(&topic), 0, 23);
		assert_eq!(blocks.len(), 0);
	}

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&topic_to_bloom(&topic), 23, 24);
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0], 23);
	}

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&topic_to_bloom(&topic), 24, 100);
		assert_eq!(blocks.len(), 0);
	}
}
