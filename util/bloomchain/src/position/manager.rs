// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use super::Position;

/// Simplifies working with bloom indexes.
pub struct Manager {
	index_size: usize,
	level_sizes: Vec<usize>,
}

impl Manager {
	/// Creates new indexer.
	pub fn new(index_size: usize, levels: usize) -> Self {
		if levels == 0 {
			panic!("Manager requires at least 1 level.");
		}

		let mut level_sizes = vec![1];
		level_sizes.extend_from_slice(&(1..).into_iter()
			.scan(1, |acc, _| {
				*acc = *acc * index_size;
				Some(*acc)
			})
			.take(levels - 1)
			.collect::<Vec<usize>>());

		Manager {
			index_size: index_size,
			level_sizes: level_sizes,
		}
	}

	/// Unsafely get level size.
	pub fn level_size(&self, level: usize) -> usize {
		self.level_sizes[level as usize]
	}

	/// Converts block number and level to `Position`.
	pub fn position(&self, block_number: usize, level: usize) -> Position {
		Position {
			level: level,
			index: block_number / self.level_size(level),
		}
	}

	/// Return bloom which are dependencies for given index.
	///
	/// Bloom indexes are ordered from lowest to highest.
	pub fn lower_level_positions(&self, index: &Position) -> Vec<Position> {
		// this is the lowest level
		if index.level == 0 {
			return vec![];
		}

		let new_level = index.level - 1;
		let offset = self.index_size * index.index;

		(0..self.index_size)
			.map(|i| Position {
				level: new_level, 
				index: offset + i
			})
			.collect()
	}

	/// Return number of levels.
	pub fn levels(&self) -> usize {
		self.level_sizes.len()
	}

	/// Returns max indexer level.
	pub fn max_level(&self) -> usize {
		self.level_sizes.len() - 1
	}
}

#[cfg(test)]
mod tests {
	use position::Position;
	use super::*;
	#[test]
	fn test_level_size() {
		let indexer = Manager::new(16, 3);
		assert_eq!(indexer.level_size(0), 1);
		assert_eq!(indexer.level_size(1), 16);
		assert_eq!(indexer.level_size(2), 256);
	}

	#[test]
	fn test_position() {
		let indexer = Manager::new(16, 3);

		let bi0 = indexer.position(0, 0);
		assert_eq!(bi0.level, 0);
		assert_eq!(bi0.index, 0);

		let bi1 = indexer.position(1, 0);
		assert_eq!(bi1.level, 0);
		assert_eq!(bi1.index, 1);

		let bi2 = indexer.position(2, 0);
		assert_eq!(bi2.level, 0);
		assert_eq!(bi2.index, 2);

		let bi3 = indexer.position(3, 1);
		assert_eq!(bi3.level, 1);
		assert_eq!(bi3.index, 0);

		let bi4 = indexer.position(15, 1);
		assert_eq!(bi4.level, 1);
		assert_eq!(bi4.index, 0);

		let bi5 = indexer.position(16, 1);
		assert_eq!(bi5.level, 1);
		assert_eq!(bi5.index, 1);

		let bi6 = indexer.position(255, 2);
		assert_eq!(bi6.level, 2);
		assert_eq!(bi6.index, 0);

		let bi7 = indexer.position(256, 2);
		assert_eq!(bi7.level, 2);
		assert_eq!(bi7.index, 1);
	}

	#[test]
	fn test_lower_level_positions() {
		let indexer = Manager::new(16, 3);

		let bi = indexer.position(256, 2);
		assert_eq!(bi.level, 2);
		assert_eq!(bi.index, 1);

		let mut ebis = vec![];
		for i in 16..32 {
			ebis.push(Position { level: 1, index: i});
		}

		let bis = indexer.lower_level_positions(&bi);
		assert_eq!(ebis, bis);
	}
}
