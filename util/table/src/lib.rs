// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! A collection associating pair of keys (row and column) with a single value.

use std::hash::Hash;
use std::collections::HashMap;
use std::collections::hash_map::Keys;

/// Structure to hold double-indexed values
///
/// You can obviously use `HashMap<(Row,Col), Val>`, but this structure gives
/// you better access to all `Columns` in Specific `Row`. Namely you can get sub-hashmap
/// `HashMap<Col, Val>` for specific `Row`
#[derive(Default, Debug, PartialEq)]
pub struct Table<Row, Col, Val>
	where Row: Eq + Hash + Clone,
		  Col: Eq + Hash {
	map: HashMap<Row, HashMap<Col, Val>>,
}

impl<Row, Col, Val> Table<Row, Col, Val>
	where Row: Eq + Hash + Clone,
		  Col: Eq + Hash {
	/// Creates new Table
	pub fn new() -> Self {
		Table {
			map: HashMap::new(),
		}
	}

	/// Returns keys iterator for this Table.
	pub fn keys(&self) -> Keys<Row, HashMap<Col, Val>> {
		self.map.keys()
	}

	/// Removes all elements from this Table
	pub fn clear(&mut self) {
		self.map.clear();
	}

	/// Returns length of the Table (number of (row, col, val) tuples)
	pub fn len(&self) -> usize {
		self.map.values().fold(0, |acc, v| acc + v.len())
	}

	/// Check if there is any element in this Table
	pub fn is_empty(&self) -> bool {
		self.map.is_empty() || self.map.values().all(|v| v.is_empty())
	}

	/// Get mutable reference for single Table row.
	pub fn row_mut(&mut self, row: &Row) -> Option<&mut HashMap<Col, Val>> {
		self.map.get_mut(row)
	}

	/// Checks if row is defined for that table (note that even if defined it might be empty)
	pub fn has_row(&self, row: &Row) -> bool {
		self.map.contains_key(row)
	}

	/// Get immutable reference for single row in this Table
	pub fn row(&self, row: &Row) -> Option<&HashMap<Col, Val>> {
		self.map.get(row)
	}

	/// Get element in cell described by `(row, col)`
	pub fn get(&self, row: &Row, col: &Col) -> Option<&Val> {
		self.map.get(row).and_then(|r| r.get(col))
	}

	/// Remove value from specific cell
	///
	/// It will remove the row if it's the last value in it
	pub fn remove(&mut self, row: &Row, col: &Col) -> Option<Val> {
		let (val, is_empty) = {
			let row_map = self.map.get_mut(row);
			if let None = row_map {
				return None;
			}
			let row_map = row_map.unwrap();
			let val = row_map.remove(col);
			(val, row_map.is_empty())
		};
		// Clean row
		if is_empty {
			self.map.remove(row);
		}
		val
	}

	/// Remove given row from Table if there are no values defined in it
	///
	/// When using `#row_mut` it may happen that all values from some row are drained.
	/// Table however will not be aware that row is empty.
	/// You can use this method to explicitly remove row entry from the Table.
	pub fn clear_if_empty(&mut self, row: &Row) {
		let is_empty = self.map.get(row).map_or(false, |m| m.is_empty());
		if is_empty {
			self.map.remove(row);
		}
	}

	/// Inserts new value to specified cell
	///
	/// Returns previous value (if any)
	pub fn insert(&mut self, row: Row, col: Col, val: Val) -> Option<Val> {
		self.map.entry(row).or_insert_with(HashMap::new).insert(col, val)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn should_create_empty_table() {
		// when
		let table : Table<usize, usize, bool> = Table::new();

		// then
		assert!(table.is_empty());
		assert_eq!(table.len(), 0);
	}

	#[test]
	fn should_insert_elements_and_return_previous_if_any() {
		// given
		let mut table = Table::new();

		// when
		let r1 = table.insert(5, 4, true);
		let r2 = table.insert(10, 4, true);
		let r3 = table.insert(10, 10, true);
		let r4 = table.insert(10, 10, false);

		// then
		assert!(r1.is_none());
		assert!(r2.is_none());
		assert!(r3.is_none());
		assert!(r4.is_some());
		assert!(!table.is_empty());
		assert_eq!(r4.unwrap(), true);
		assert_eq!(table.len(), 3);
	}

	#[test]
	fn should_remove_element() {
		// given
		let mut table = Table::new();
		table.insert(5, 4, true);
		assert!(!table.is_empty());
		assert_eq!(table.len(), 1);

		// when
		let r = table.remove(&5, &4);

		// then
		assert!(table.is_empty());
		assert_eq!(table.len() ,0);
		assert_eq!(r.unwrap(), true);
	}

	#[test]
	fn should_return_none_if_trying_to_remove_non_existing_element() {
				// given
		let mut table : Table<usize, usize, usize> = Table::new();
		assert!(table.is_empty());

		// when
		let r = table.remove(&5, &4);

		// then
		assert!(r.is_none());
	}

	#[test]
	fn should_clear_row_if_removing_last_element() {
		// given
		let mut table = Table::new();
		table.insert(5, 4, true);
		assert!(table.has_row(&5));

		// when
		let r = table.remove(&5, &4);

		// then
		assert!(r.is_some());
		assert!(!table.has_row(&5));
	}

	#[test]
	fn should_return_element_given_row_and_col() {
		// given
		let mut table = Table::new();
		table.insert(1551, 1234, 123);

		// when
		let r1 = table.get(&1551, &1234);
		let r2 = table.get(&5, &4);

		// then
		assert!(r1.is_some());
		assert!(r2.is_none());
		assert_eq!(r1.unwrap(), &123);
	}

	#[test]
	fn should_clear_table() {
		// given
		let mut table = Table::new();
		table.insert(1, 1, true);
		table.insert(1, 2, false);
		table.insert(2, 2, false);
		assert_eq!(table.len(), 3);

		// when
		table.clear();

		// then
		assert!(table.is_empty());
		assert_eq!(table.len(), 0);
		assert_eq!(table.has_row(&1), false);
		assert_eq!(table.has_row(&2), false);
	}

	#[test]
	fn should_return_mutable_row() {
		// given
		let mut table = Table::new();
		table.insert(1, 1, true);
		table.insert(1, 2, false);
		table.insert(2, 2, false);

		// when
		{
			let mut row = table.row_mut(&1).unwrap();
			row.remove(&1);
			row.remove(&2);
		}
		assert!(table.has_row(&1));
		table.clear_if_empty(&1);

		// then
		assert!(!table.has_row(&1));
		assert_eq!(table.len(), 1);
	}
}
