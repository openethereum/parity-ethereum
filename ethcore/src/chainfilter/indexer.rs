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

//! Simplifies working with bloom indexes.

use chainfilter::BloomIndex;

/// Simplifies working with bloom indexes.
pub struct Indexer {
	index_size: usize,
	level_sizes: Vec<usize>,
}

impl Indexer {
	/// Creates new indexer.
	pub fn new(index_size: usize, levels: u8) -> Self {
		if levels == 0 {
			panic!("Indexer requires at least 1 level.");
		}

		let mut level_sizes = vec![1];
		level_sizes.extend_from_slice(&(1..).into_iter()
			.scan(1, |acc, _| {
				*acc = *acc * index_size;
				Some(*acc)
			})
			.take(levels as usize - 1)
			.collect::<Vec<usize>>());

		Indexer {
			index_size: index_size,
			level_sizes: level_sizes,
		}
	}

	/// unsafely get level size.
	pub fn level_size(&self, level: u8) -> usize {
		self.level_sizes[level as usize]
	}

	/// Converts block number and level to `BloomIndex`.
	pub fn bloom_index(&self, block_number: usize, level: u8) -> BloomIndex {
		BloomIndex {
			level: level,
			index: block_number / self.level_size(level),
		}
	}

	/// Return bloom which are dependencies for given index.
	///
	/// Bloom indexes are ordered from lowest to highest.
	pub fn lower_level_bloom_indexes(&self, index: &BloomIndex) -> Vec<BloomIndex> {
		// this is the lowest level
		if index.level == 0 {
			return vec![];
		}

		let new_level = index.level - 1;
		let offset = self.index_size * index.index;

		(0..self.index_size).map(|i| BloomIndex::new(new_level, offset + i)).collect()
	}

	/// Return number of levels.
	pub fn levels(&self) -> u8 {
		self.level_sizes.len() as u8
	}

	/// Returns max indexer level.
	pub fn max_level(&self) -> u8 {
		self.level_sizes.len() as u8 - 1
	}
}

#[cfg(test)]
mod tests {
	#![cfg_attr(feature="dev", allow(similar_names))]
	use chainfilter::BloomIndex;
	use chainfilter::indexer::Indexer;

	#[test]
	fn test_level_size() {
		let indexer = Indexer::new(16, 3);
		assert_eq!(indexer.level_size(0), 1);
		assert_eq!(indexer.level_size(1), 16);
		assert_eq!(indexer.level_size(2), 256);
	}

	#[test]
	fn test_bloom_index() {
		let indexer = Indexer::new(16, 3);

		let bi0 = indexer.bloom_index(0, 0);
		assert_eq!(bi0.level, 0);
		assert_eq!(bi0.index, 0);

		let bi1 = indexer.bloom_index(1, 0);
		assert_eq!(bi1.level, 0);
		assert_eq!(bi1.index, 1);

		let bi2 = indexer.bloom_index(2, 0);
		assert_eq!(bi2.level, 0);
		assert_eq!(bi2.index, 2);

		let bi3 = indexer.bloom_index(3, 1);
		assert_eq!(bi3.level, 1);
		assert_eq!(bi3.index, 0);

		let bi4 = indexer.bloom_index(15, 1);
		assert_eq!(bi4.level, 1);
		assert_eq!(bi4.index, 0);

		let bi5 = indexer.bloom_index(16, 1);
		assert_eq!(bi5.level, 1);
		assert_eq!(bi5.index, 1);

		let bi6 = indexer.bloom_index(255, 2);
		assert_eq!(bi6.level, 2);
		assert_eq!(bi6.index, 0);

		let bi7 = indexer.bloom_index(256, 2);
		assert_eq!(bi7.level, 2);
		assert_eq!(bi7.index, 1);
	}

	#[test]
	fn test_lower_level_bloom_indexes() {
		let indexer = Indexer::new(16, 3);

		let bi = indexer.bloom_index(256, 2);
		assert_eq!(bi.level, 2);
		assert_eq!(bi.index, 1);

		let mut ebis = vec![];
		for i in 16..32 {
			ebis.push(BloomIndex::new(1, i));
		}

		let bis = indexer.lower_level_bloom_indexes(&bi);
		assert_eq!(ebis, bis);
	}
}
