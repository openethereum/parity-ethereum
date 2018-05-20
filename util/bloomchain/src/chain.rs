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

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use number::Number;
use position::{Position, Manager as PositionManager};
use bloom::Bloom;
use filter::Filter;
use config::Config;
use database::BloomDatabase;

/// Prepares all bloom database operations.
pub struct BloomChain<'a> {
	positioner: PositionManager,
	db: &'a BloomDatabase,
}

impl<'a> BloomChain<'a> {
	/// Creates new bloom chain.
	pub fn new(config: Config, db: &'a BloomDatabase) -> Self {
		let positioner = PositionManager::new(config.elements_per_index, config.levels);

		BloomChain {
			positioner: positioner,
			db: db,
		}
	}

	/// Internal function which does bloom search recursively.
	fn blocks(&self, range: &Range<Number>, bloom: &Bloom, level: usize, offset: usize) -> Option<Vec<usize>> {
		let index = self.positioner.position(offset, level);

		match self.db.bloom_at(&index) {
			None => return None,
			Some(level_bloom) => match level {
				// if we are on the lowest level
				0 if level_bloom.contains_bloom(bloom) => return Some(vec![offset]),
				// return None if current level doesnt contain given bloom
				_ if !level_bloom.contains_bloom(bloom) => return None,
				// continue processing && go down
				_ => ()
			}
		};

		let level_size = self.positioner.level_size(level - 1);
		let from_position = self.positioner.position(range.start, level - 1);
		let to_position = self.positioner.position(range.end, level - 1);
		let res: Vec<usize> = self.positioner.lower_level_positions(&index).into_iter()
			// chose only blooms in range
			.filter(|li| li.index >= from_position.index && li.index <= to_position.index)
			// map them to offsets
			.map(|li| li.index * level_size)
			// get all blocks that may contain our bloom
			// filter existing ones
			.filter_map(|off| self.blocks(range, bloom, level - 1, off))
			// flatten nested structures
			.flat_map(|v| v)
			.collect();
		Some(res)
	}

	/// Inserts the bloom at all filter levels.
	pub fn insert(&self, number: Number, bloom: Bloom) -> HashMap<Position, Bloom> {
		let mut result: HashMap<Position, Bloom> = HashMap::new();

		for level in 0..self.positioner.levels() {
			let position = self.positioner.position(number, level);
			let new_bloom = match self.db.bloom_at(&position) {
				Some(mut old_bloom) => {
					old_bloom.accrue_bloom(&bloom);
					old_bloom
				},
				None => bloom.clone(),
			};

			result.insert(position, new_bloom);
		}

		result
	}

	/// Resets data in range.
	/// Inserts new data.
	/// Inserted data may exceed reseted range.
	pub fn replace(&self, range: &Range<Number>, blooms: Vec<Bloom>) -> HashMap<Position, Bloom> {
		let mut result: HashMap<Position, Bloom> = HashMap::new();

		// insert all new blooms at level 0
		for (i, bloom) in blooms.iter().enumerate() {
			result.insert(self.positioner.position(range.start + i, 0), bloom.clone());
		}

		// reset the rest of blooms
		for reset_number in range.start + blooms.len()..(range.end + 1) {
			result.insert(self.positioner.position(reset_number, 0), Bloom::default());
		}

		for level in 1..self.positioner.levels() {
			for i in 0..blooms.len() {

				let index = self.positioner.position(range.start + i, level);
				let new_bloom = {
					// use new blooms before db blooms where necessary
					let bloom_at = | index | { result.get(&index).cloned().or_else(|| self.db.bloom_at(&index)) };

					self.positioner.lower_level_positions(&index)
						.into_iter()
						// get blooms
						// filter existing ones
						.filter_map(bloom_at)
						// BitOr all of them
						.fold(Bloom::default(), |mut acc, bloom| {
							acc.accrue_bloom(&bloom);
							acc
						})
				};

				result.insert(index, new_bloom);
			}
		}

		result
	}

	/// Returns all numbers with given bloom.
	pub fn with_bloom(&self, range: &Range<Number>, bloom: &Bloom) -> Vec<Number> {
		let mut result = vec![];
		// lets start from highest level
		let max_level = self.positioner.max_level();
		let level_size = self.positioner.level_size(max_level);
		let from_position = self.positioner.position(range.start, max_level);
		let to_position = self.positioner.position(range.end, max_level);

		for index in from_position.index..to_position.index + 1 {
			// offset will be used to calculate where we are right now
			let offset = level_size * index;

			// go doooown!
			if let Some(blocks) = self.blocks(range, bloom, max_level, offset) {
				result.extend(blocks);
			}
		}

		result
	}

	/// Filter the chain returing all numbers matching the filter.
	pub fn filter(&self, filter: &Filter) -> Vec<Number> {
		let range = filter.range();
		let mut blocks = filter.bloom_possibilities()
			.into_iter()
			.flat_map(|ref bloom| self.with_bloom(&range, bloom))
			.collect::<HashSet<Number>>()
			.into_iter()
			.collect::<Vec<Number>>();

		blocks.sort();
		blocks
	}
}
