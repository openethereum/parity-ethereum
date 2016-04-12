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
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;
use util::hash::*;
use util::sha3::*;
use chainfilter::{BloomIndex, FilterDataSource, ChainFilter};

/// In memory cache for blooms.
///
/// Stores all blooms in `HashMap`, which indexes them by `BloomIndex`.
pub struct MemoryCache {
	blooms: HashMap<BloomIndex, H2048>,
}

impl Default for MemoryCache {
	fn default() -> Self {
		MemoryCache::new()
	}
}

impl MemoryCache {
	/// Default constructor for MemoryCache
	pub fn new() -> Self {
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

fn to_bloom<T>(hashable: &T) -> H2048 where T: Hashable {
	let mut bloom = H2048::new();
	bloom.shift_bloomed(&hashable.sha3());
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
		filter.add_bloom(&to_bloom(&topic), block_number)
	};

	// number of modified blooms should always be equal number of levels
	assert_eq!(modified_blooms.len(), bloom_levels as usize);
	cache.insert_blooms(modified_blooms);

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&to_bloom(&topic), 0, 100);
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0], 23);
	}

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&to_bloom(&topic), 0, 22);
		assert_eq!(blocks.len(), 0);
	}

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&to_bloom(&topic), 23, 23);
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0], 23);
	}

	{
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&to_bloom(&topic), 24, 100);
		assert_eq!(blocks.len(), 0);
	}
}

#[test]
fn test_reset_chain_head_simple() {
	let index_size = 16;
	let bloom_levels = 3;

	let mut cache = MemoryCache::new();
	let topic_0 = H256::from_str("8d936b1bd3fc635710969ccfba471fb17d598d9d1971b538dd712e1e4b4f4dba").unwrap();
	let topic_1 = H256::from_str("8d936b1bd3fc635710969ccfba471fb17d598d9d1971b538dd712e1e4b4f4dbb").unwrap();
	let topic_2 = H256::from_str("8d936b1bd3fc635710969ccfba471fb17d598d9d1971b538dd712e1e4b4f4dbc").unwrap();
	let topic_3 = H256::from_str("8d936b1bd3fc635710969ccfba471fb17d598d9d1971b538dd712e1e4b4f4dbd").unwrap();
	let topic_4 = H256::from_str("8d936b1bd3fc635710969ccfba471fb17d598d9d1971b538dd712e1e4b4f4dbe").unwrap();
	let topic_5 = H256::from_str("8d936b1bd3fc635710969ccfba471fb17d598d9d1971b538dd712e1e4b4f4dbf").unwrap();

	let modified_blooms_0 = {
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let block_number = 14;
		filter.add_bloom(&to_bloom(&topic_0), block_number)
	};

	cache.insert_blooms(modified_blooms_0);

	let modified_blooms_1 = {
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let block_number = 15;
		filter.add_bloom(&to_bloom(&topic_1), block_number)
	};

	cache.insert_blooms(modified_blooms_1);

	let modified_blooms_2 = {
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let block_number = 16;
		filter.add_bloom(&to_bloom(&topic_2), block_number)
	};

	cache.insert_blooms(modified_blooms_2);

	let modified_blooms_3 = {
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let block_number = 17;
		filter.add_bloom(&to_bloom(&topic_3), block_number)
	};

	cache.insert_blooms(modified_blooms_3);


	let reset_modified_blooms = {
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		filter.reset_chain_head(&[to_bloom(&topic_4), to_bloom(&topic_5)], 15, 17)
	};

	cache.insert_blooms(reset_modified_blooms);

	let filter = ChainFilter::new(&cache, index_size, bloom_levels);
	assert_eq!(filter.blocks_with_bloom(&to_bloom(&topic_0), 0, 100), vec![14]);
	assert_eq!(filter.blocks_with_bloom(&to_bloom(&topic_1), 0, 100), vec![]);
	assert_eq!(filter.blocks_with_bloom(&to_bloom(&topic_2), 0, 100), vec![]);
	assert_eq!(filter.blocks_with_bloom(&to_bloom(&topic_3), 0, 100), vec![]);
	assert_eq!(filter.blocks_with_bloom(&to_bloom(&topic_4), 0, 100), vec![15]);
	assert_eq!(filter.blocks_with_bloom(&to_bloom(&topic_5), 0, 100), vec![16]);
}

fn for_each_bloom<F>(bytes: &[u8], mut f: F) where F: FnMut(usize, &H2048) {
	let mut reader = BufReader::new(bytes);
	let mut line = String::new();
	while reader.read_line(&mut line).unwrap() > 0 {
		{
			let mut number_bytes = vec![];
			let mut bloom_bytes = [0; 512];

			let mut line_reader = BufReader::new(line.as_ref() as &[u8]);
			line_reader.read_until(b' ', &mut number_bytes).unwrap();
			line_reader.consume(2);
			line_reader.read_exact(&mut bloom_bytes).unwrap();

			let number = String::from_utf8(number_bytes).map(|s| s[..s.len() -1].to_owned()).unwrap().parse::<usize>().unwrap();
			let bloom = H2048::from_str(&String::from_utf8(bloom_bytes.to_vec()).unwrap()).unwrap();
			f(number, &bloom);
		}
		line.clear();
	}
}

fn for_each_log<F>(bytes: &[u8], mut f: F) where F: FnMut(usize, &Address, &[H256]) {
	let mut reader = BufReader::new(bytes);
	let mut line = String::new();
	while reader.read_line(&mut line).unwrap() > 0 {
		{
			let mut number_bytes = vec![];
			let mut address_bytes = [0;42];
			let mut topic = [0;66];
			let mut topics_bytes = vec![];

			let mut line_reader = BufReader::new(line.as_ref() as &[u8]);
			line_reader.read_until(b' ', &mut number_bytes).unwrap();
			line_reader.read_exact(&mut address_bytes).unwrap();
			line_reader.consume(1);
			while let Ok(_) = line_reader.read_exact(&mut topic) {
				line_reader.consume(1);
				topics_bytes.push(topic.to_vec());
			}

			let number = String::from_utf8(number_bytes).map(|s| s[..s.len() -1].to_owned()).unwrap().parse::<usize>().unwrap();
			let address = Address::from_str(&String::from_utf8(address_bytes.to_vec()).map(|a| a[2..].to_owned()).unwrap()).unwrap();
			let topics: Vec<H256> = topics_bytes
				.into_iter()
				.map(|t| H256::from_str(&String::from_utf8(t).map(|t| t[2..].to_owned()).unwrap()).unwrap())
				.collect();
			f(number, &address, &topics);
		}
		line.clear();
	}
}

// tests chain filter on real data between blocks 300_000 and 400_000
#[test]
fn test_chainfilter_real_data_short_searches() {
	let index_size = 16;
	let bloom_levels = 3;

	let mut cache = MemoryCache::new();

	for_each_bloom(include_bytes!("blooms.txt"), | block_number, bloom | {
		let modified_blooms = {
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			filter.add_bloom(bloom, block_number)
		};

		// number of modified blooms should always be equal number of levels
		assert_eq!(modified_blooms.len(), bloom_levels as usize);
		cache.insert_blooms(modified_blooms);
	});

	for_each_log(include_bytes!("logs.txt"), | block_number, address, topics | {
		println!("block_number: {:?}", block_number);
		let filter = ChainFilter::new(&cache, index_size, bloom_levels);
		let blocks = filter.blocks_with_bloom(&to_bloom(address), block_number, block_number);
		assert_eq!(blocks.len(), 1);
		for (i, topic) in topics.iter().enumerate() {
			println!("topic: {:?}", i);
			let blocks = filter.blocks_with_bloom(&to_bloom(topic), block_number, block_number);
			assert_eq!(blocks.len(), 1);
		}
	});
}

// tests chain filter on real data between blocks 300_000 and 400_000
#[test]
fn test_chainfilter_real_data_single_search() {
	let index_size = 16;
	let bloom_levels = 3;

	let mut cache = MemoryCache::new();

	for_each_bloom(include_bytes!("blooms.txt"), | block_number, bloom | {
		let modified_blooms = {
			let filter = ChainFilter::new(&cache, index_size, bloom_levels);
			filter.add_bloom(bloom, block_number)
		};

		// number of modified blooms should always be equal number of levels
		assert_eq!(modified_blooms.len(), bloom_levels as usize);
		cache.insert_blooms(modified_blooms);
	});

	let address = Address::from_str("c4395759e26469baa0e6421bdc1d0232c6f4b6c3").unwrap();
	let filter = ChainFilter::new(&cache, index_size, bloom_levels);
	let blocks = filter.blocks_with_bloom(&to_bloom(&address), 300_000, 400_000);
	// bloom may return more blocks, but our log density is low, so it should be fine
	assert_eq!(blocks.len(), 3);
	assert_eq!(blocks[0], 392697);
	assert_eq!(blocks[1], 396348);
	assert_eq!(blocks[2], 399804);
}


