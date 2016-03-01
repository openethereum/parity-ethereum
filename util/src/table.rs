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

//! A collection associating pair of keys (row and column) with a single value.

use std::hash::Hash;
use std::collections::HashMap;

/// Structure to hold double-indexed values
///
/// You can obviously use `HashMap<(Row,Col), Val>`, but this structure gives
/// you better access to all `Columns` in Specific `Row`. Namely you can get sub-hashmap
/// `HashMap<Col, Val>` for specific `Row`
pub struct Table<Row, Col, Val>
	where Row: Eq + Hash + Clone,
		  Col: Eq + Hash {
	map: HashMap<Row, HashMap<Col, Val>>,
}

impl<Row, Col, Val> Table<Row, Col, Val>
	where Row: Eq + Hash + Clone,
		  Col: Eq + Hash {
	/// Creates new Table
	pub fn new() -> Table<Row, Col, Val> {
		Table {
			map: HashMap::new(),
		}
	}

	/// Removes all elements from this Table
	pub fn clear(&mut self) {
		self.map.clear();
	}

	/// Returns length of the Table (number of (row, col, val) tuples)
	pub fn len(&self) -> usize {
		self.map.iter().fold(0, |acc, (_k, v)| acc + v.len())
	}

	/// Check if there is any element in this Table
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Get mutable reference for single Table row.
	pub fn get_row_mut(&mut self, row: &Row) -> Option<&mut HashMap<Col, Val>> {
		self.map.get_mut(row)
	}

	/// Checks if row is defined for that table (note that even if defined it might be empty)
	pub fn has_row(&self, row: &Row) -> bool {
		self.map.contains_key(row)
	}

	/// Get immutable reference for single row in this Table
	pub fn get_row(&self, row: &Row) -> Option<&HashMap<Col, Val>> {
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
			let mut row_map = row_map.unwrap();
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
	/// When using `#get_row_mut` it may happen that all values from some row are drained.
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
		if !self.map.contains_key(&row) {
			let m = HashMap::new();
			self.map.insert(row.clone(), m);
		}

		let mut columns = self.map.get_mut(&row).unwrap();
		columns.insert(col, val)
	}
}
