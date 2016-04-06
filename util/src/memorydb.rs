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

//! Reference-counted memory-based `HashDB` implementation.

use hash::*;
use bytes::*;
use rlp::*;
use sha3::*;
use hashdb::*;
use heapsize::*;
use std::mem;
use std::collections::HashMap;
use std::default::Default;

#[derive(Debug,Clone)]
/// Reference-counted memory-based `HashDB` implementation.
///
/// Use `new()` to create a new database. Insert items with `insert()`, remove items
/// with `remove()`, check for existence with `containce()` and lookup a hash to derive
/// the data with `get()`. Clear with `clear()` and purge the portions of the data
/// that have no references with `purge()`.
///
/// # Example
/// ```rust
/// extern crate ethcore_util;
/// use ethcore_util::hashdb::*;
/// use ethcore_util::memorydb::*;
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
#[derive(PartialEq)]
pub struct MemoryDB {
	data: HashMap<H256, (Bytes, i32)>,
	static_null_rlp: (Bytes, i32),
}

impl Default for MemoryDB {
	fn default() -> Self {
		MemoryDB::new()
	}
}

impl MemoryDB {
	/// Create a new instance of the memory DB.
	pub fn new() -> MemoryDB {
		MemoryDB {
			data: HashMap::new(),
 			static_null_rlp: (vec![0x80u8; 1], 1),
		}
	}

	/// Clear all data from the database.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
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
		let empties: Vec<_> = self.data.iter()
			.filter(|&(_, &(_, rc))| rc == 0)
			.map(|(k, _)| k.clone())
			.collect();
		for empty in empties { self.data.remove(&empty); }
	}

	/// Grab the raw information associated with a key. Returns None if the key
	/// doesn't exist.
	///
	/// Even when Some is returned, the data is only guaranteed to be useful
	/// when the refs > 0.
	pub fn raw(&self, key: &H256) -> Option<&(Bytes, i32)> {
		if key == &SHA3_NULL_RLP {
			return Some(&self.static_null_rlp);
		}
		self.data.get(key)
	}

	/// Return the internal map of hashes to data, clearing the current state.
	pub fn drain(&mut self) -> HashMap<H256, (Bytes, i32)> {
		let mut data = HashMap::new();
		mem::swap(&mut self.data, &mut data);
		data
	}

	/// Denote than an existing value has the given key. Used when a key gets removed without
	/// a prior insert and thus has a negative reference with no value.
	///
	/// May safely be called even if the key's value is known, in which case it will be a no-op.
	pub fn denote(&self, key: &H256, value: Bytes) -> &(Bytes, i32) {
		if self.raw(key) == None {
			unsafe {
				let p = &self.data as *const HashMap<H256, (Bytes, i32)> as *mut HashMap<H256, (Bytes, i32)>;
				(*p).insert(key.clone(), (value, 0));
			}
		}
		self.raw(key).unwrap()
	}

	/// Returns the size of allocated heap memory
	pub fn mem_used(&self) -> usize {
		self.data.heap_size_of_children()
	}
}

static NULL_RLP_STATIC: [u8; 1] = [0x80; 1];

impl HashDB for MemoryDB {
	fn lookup(&self, key: &H256) -> Option<&[u8]> {
		if key == &SHA3_NULL_RLP {
			return Some(&NULL_RLP_STATIC);
		}
		match self.data.get(key) {
			Some(&(ref d, rc)) if rc > 0 => Some(d),
			_ => None
		}
	}

	fn keys(&self) -> HashMap<H256, i32> {
		self.data.iter().filter_map(|(k, v)| if v.1 != 0 {Some((k.clone(), v.1))} else {None}).collect()
	}

	fn exists(&self, key: &H256) -> bool {
		if key == &SHA3_NULL_RLP {
			return true;
		}
		match self.data.get(key) {
			Some(&(_, x)) if x > 0 => true,
			_ => false
		}
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		if value == &NULL_RLP {
			return SHA3_NULL_RLP.clone();
		}
		let key = value.sha3();
		if match self.data.get_mut(&key) {
			Some(&mut (ref mut old_value, ref mut rc @ -0x80000000i32 ... 0)) => {
				*old_value = From::from(value.bytes());
				*rc += 1;
				false
			},
			Some(&mut (_, ref mut x)) => { *x += 1; false } ,
			None => true,
		}{	// ... None falls through into...
			self.data.insert(key.clone(), (From::from(value.bytes()), 1));
		}
		key
	}

	fn emplace(&mut self, key: H256, value: Bytes) {
		if value == &NULL_RLP {
			return;
		}
		match self.data.get_mut(&key) {
			Some(&mut (ref mut old_value, ref mut rc @ -0x80000000i32 ... 0)) => {
				*old_value = value;
				*rc += 1;
				return;
			},
			Some(&mut (_, ref mut x)) => { *x += 1; return; } ,
			None => {},
		}
		// ... None falls through into...
		self.data.insert(key, (value, 1));
	}

	fn kill(&mut self, key: &H256) {
		if key == &SHA3_NULL_RLP {
			return;
		}
		if match self.data.get_mut(key) {
			Some(&mut (_, ref mut x)) => { *x -= 1; false }
			None => true
		}{	// ... None falls through into...
			self.data.insert(key.clone(), (Bytes::new(), -1));
		}
	}
}

#[test]
fn memorydb_denote() {
	let mut m = MemoryDB::new();
	let hello_bytes = b"Hello world!";
	let hash = m.insert(hello_bytes);
	assert_eq!(m.get(&hash).unwrap(), b"Hello world!");

	for _ in 0..1000 {
		let r = H256::random();
		let k = r.sha3();
		let &(ref v, ref rc) = m.denote(&k, r.bytes().to_vec());
		assert_eq!(v, &r.bytes());
		assert_eq!(*rc, 0);
	}

	assert_eq!(m.get(&hash).unwrap(), b"Hello world!");
}
