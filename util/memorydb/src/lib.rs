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

//! Reference-counted memory-based `HashDB` implementation.
extern crate heapsize;
extern crate ethcore_bigint as bigint;
extern crate rlp;
extern crate hash as keccak;
extern crate hashdb;

use std::mem;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use heapsize::HeapSizeOf;
use bigint::hash::{H256FastMap, H256};
use rlp::NULL_RLP;
use keccak::{KECCAK_NULL_RLP, keccak};
use hashdb::*;

/// Reference-counted memory-based `HashDB` implementation.
///
/// Use `new()` to create a new database. Insert items with `insert()`, remove items
/// with `remove()`, check for existence with `contains()` and lookup a hash to derive
/// the data with `get()`. Clear with `clear()` and purge the portions of the data
/// that have no references with `purge()`.
///
/// # Example
/// ```rust
/// extern crate hashdb;
/// extern crate memorydb;
/// use hashdb::*;
/// use memorydb::*;
/// fn main() {
///   let mut m = MemoryDB::new();
///   let d = "Hello world!".as_bytes();
///
///   let k = m.insert(d);
///   assert!(m.contains(&k));
///   assert_eq!(m.get(&k).unwrap(), d);
///
///   m.insert(d);
///   assert!(m.contains(&k));
///
///   m.remove(&k);
///   assert!(m.contains(&k));
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
///
///   m.insert(d);
///   assert!(!m.contains(&k));

///   m.insert(d);
///   assert!(m.contains(&k));
///   assert_eq!(m.get(&k).unwrap(), d);
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
/// }
/// ```
#[derive(Default, Clone, PartialEq)]
pub struct MemoryDB {
	data: H256FastMap<(DBValue, i32)>,
}

impl MemoryDB {
	/// Create a new instance of the memory DB.
	pub fn new() -> MemoryDB {
		MemoryDB {
			data: H256FastMap::default(),
		}
	}

	/// Clear all data from the database.
	///
	/// # Examples
	/// ```rust
	/// extern crate hashdb;
	/// extern crate memorydb;
	/// use hashdb::*;
	/// use memorydb::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let hello_bytes = "Hello world!".as_bytes();
	///   let hash = m.insert(hello_bytes);
	///   assert!(m.contains(&hash));
	///   m.clear();
	///   assert!(!m.contains(&hash));
	/// }
	/// ```
	pub fn clear(&mut self) {
		self.data.clear();
	}

	/// Purge all zero-referenced data from the database.
	pub fn purge(&mut self) {
		self.data.retain(|_, &mut (_, rc)| rc != 0);
	}

	/// Return the internal map of hashes to data, clearing the current state.
	pub fn drain(&mut self) -> H256FastMap<(DBValue, i32)> {
		mem::replace(&mut self.data, H256FastMap::default())
	}

	/// Grab the raw information associated with a key. Returns None if the key
	/// doesn't exist.
	///
	/// Even when Some is returned, the data is only guaranteed to be useful
	/// when the refs > 0.
	pub fn raw(&self, key: &H256) -> Option<(DBValue, i32)> {
		if key == &KECCAK_NULL_RLP {
			return Some((DBValue::from_slice(&NULL_RLP), 1));
		}
		self.data.get(key).cloned()
	}

	/// Returns the size of allocated heap memory
	pub fn mem_used(&self) -> usize {
		self.data.heap_size_of_children()
	}

	/// Remove an element and delete it from storage if reference count reaches zero.
	/// If the value was purged, return the old value.
	pub fn remove_and_purge(&mut self, key: &H256) -> Option<DBValue> {
		if key == &KECCAK_NULL_RLP {
			return None;
		}
		match self.data.entry(key.clone()) {
			Entry::Occupied(mut entry) =>
				if entry.get().1 == 1 {
					Some(entry.remove().0)
				} else {
					entry.get_mut().1 -= 1;
					None
				},
			Entry::Vacant(entry) => {
				entry.insert((DBValue::new(), -1));
				None
			}
		}
	}

	/// Consolidate all the entries of `other` into `self`.
	pub fn consolidate(&mut self, mut other: Self) {
		for (key, (value, rc)) in other.drain() {
			match self.data.entry(key) {
				Entry::Occupied(mut entry) => {
					if entry.get().1 < 0 {
						entry.get_mut().0 = value;
					}

					entry.get_mut().1 += rc;
				}
				Entry::Vacant(entry) => {
					entry.insert((value, rc));
				}
			}
		}
	}
}

impl HashDB for MemoryDB {
	fn get(&self, key: &H256) -> Option<DBValue> {
		if key == &KECCAK_NULL_RLP {
			return Some(DBValue::from_slice(&NULL_RLP));
		}

		match self.data.get(key) {
			Some(&(ref d, rc)) if rc > 0 => Some(d.clone()),
			_ => None
		}
	}

	fn keys(&self) -> HashMap<H256, i32> {
		self.data.iter()
			.filter_map(|(k, v)| if v.1 != 0 {
				Some((*k, v.1))
			} else {
				None
			})
			.collect()
	}

	fn contains(&self, key: &H256) -> bool {
		if key == &KECCAK_NULL_RLP {
			return true;
		}

		match self.data.get(key) {
			Some(&(_, x)) if x > 0 => true,
			_ => false
		}
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		if value == &NULL_RLP {
			return KECCAK_NULL_RLP.clone();
		}
		let key = keccak(value);
		match self.data.entry(key) {
			Entry::Occupied(mut entry) => {
				let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
				if *rc >= -0x80000000i32 && *rc <= 0 {
					*old_value = DBValue::from_slice(value);
				}
				*rc += 1;
			},
			Entry::Vacant(entry) => {
				entry.insert((DBValue::from_slice(value), 1));
			},
		}
		key
	}

	fn emplace(&mut self, key: H256, value: DBValue) {
		if &*value == &NULL_RLP {
			return;
		}

		match self.data.entry(key) {
			Entry::Occupied(mut entry) => {
				let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
				if *rc >= -0x80000000i32 && *rc <= 0 {
					*old_value = value;
				}
				*rc += 1;
			},
			Entry::Vacant(entry) => {
				entry.insert((value, 1));
			},
		}
	}

	fn remove(&mut self, key: &H256) {
		if key == &KECCAK_NULL_RLP {
			return;
		}

		match self.data.entry(*key) {
			Entry::Occupied(mut entry) => {
				let &mut (_, ref mut rc) = entry.get_mut();
				*rc -= 1;
			},
			Entry::Vacant(entry) => {
				entry.insert((DBValue::new(), -1));
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use keccak::keccak;
	use super::*;

	#[test]
	fn memorydb_remove_and_purge() {
		let hello_bytes = b"Hello world!";
		let hello_key = keccak(hello_bytes);

		let mut m = MemoryDB::new();
		m.remove(&hello_key);
		assert_eq!(m.raw(&hello_key).unwrap().1, -1);
		m.purge();
		assert_eq!(m.raw(&hello_key).unwrap().1, -1);
		m.insert(hello_bytes);
		assert_eq!(m.raw(&hello_key).unwrap().1, 0);
		m.purge();
		assert_eq!(m.raw(&hello_key), None);

		let mut m = MemoryDB::new();
		assert!(m.remove_and_purge(&hello_key).is_none());
		assert_eq!(m.raw(&hello_key).unwrap().1, -1);
		m.insert(hello_bytes);
		m.insert(hello_bytes);
		assert_eq!(m.raw(&hello_key).unwrap().1, 1);
		assert_eq!(&*m.remove_and_purge(&hello_key).unwrap(), hello_bytes);
		assert_eq!(m.raw(&hello_key), None);
		assert!(m.remove_and_purge(&hello_key).is_none());
	}

	#[test]
	fn consolidate() {
		let mut main = MemoryDB::new();
		let mut other = MemoryDB::new();
		let remove_key = other.insert(b"doggo");
		main.remove(&remove_key);

		let insert_key = other.insert(b"arf");
		main.emplace(insert_key, DBValue::from_slice(b"arf"));

		let negative_remove_key = other.insert(b"negative");
		other.remove(&negative_remove_key);	// ref cnt: 0
		other.remove(&negative_remove_key);	// ref cnt: -1
		main.remove(&negative_remove_key);	// ref cnt: -1

		main.consolidate(other);

		let overlay = main.drain();

		assert_eq!(overlay.get(&remove_key).unwrap(), &(DBValue::from_slice(b"doggo"), 0));
		assert_eq!(overlay.get(&insert_key).unwrap(), &(DBValue::from_slice(b"arf"), 2));
		assert_eq!(overlay.get(&negative_remove_key).unwrap(), &(DBValue::from_slice(b"negative"), -2));
	}
}
