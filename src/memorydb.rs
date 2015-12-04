//! Reference-counted memory-based HashDB implementation.

use hash::*;
use bytes::*;
use sha3::*;
use hashdb::*;
use std::mem;
use std::collections::HashMap;

#[derive(Debug,Clone)]
/// Reference-counted memory-based HashDB implementation.
///
/// Use `new()` to create a new database. Insert items with `insert()`, remove items
/// with `kill()`, check for existance with `exists()` and lookup a hash to derive
/// the data with `lookup()`. Clear with `clear()` and purge the portions of the data
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
///   assert!(m.exists(&k));
///   assert_eq!(m.lookup(&k).unwrap(), d);
///
///   m.insert(d);
///   assert!(m.exists(&k));
///
///   m.kill(&k);
///   assert!(m.exists(&k));
///
///   m.kill(&k);
///   assert!(!m.exists(&k));
///
///   m.kill(&k);
///   assert!(!m.exists(&k));
///
///   m.insert(d);
///   assert!(!m.exists(&k));

///   m.insert(d);
///   assert!(m.exists(&k));
///   assert_eq!(m.lookup(&k).unwrap(), d);
///
///   m.kill(&k);
///   assert!(!m.exists(&k));
/// }
/// ```
pub struct MemoryDB {
	data: HashMap<H256, (Bytes, i32)>,
}

impl MemoryDB {
	/// Create a new instance of the memory DB.
	pub fn new() -> MemoryDB {
		MemoryDB {
			data: HashMap::new()
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
	///   assert!(m.exists(&hash));
	///   m.clear();
	///   assert!(!m.exists(&hash));
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
		self.data.get(key)
	}

	pub fn drain(&mut self) -> HashMap<H256, (Bytes, i32)> {
		let mut data = HashMap::new();
		mem::swap(&mut self.data, &mut data);
		data
	}

	pub fn denote(&self, key: &H256, value: Bytes) -> &(Bytes, i32) {
		if self.data.get(&key) == None {
			unsafe {
				let p = &self.data as *const HashMap<H256, (Bytes, i32)> as *mut HashMap<H256, (Bytes, i32)>;
				(*p).insert(key.clone(), (value, 0));
			}
		}
		self.data.get(key).unwrap()
	}
}

impl HashDB for MemoryDB {
	fn lookup(&self, key: &H256) -> Option<&[u8]> {
		match self.data.get(key) {
			Some(&(ref d, rc)) if rc > 0 => Some(d),
			_ => None
		}
	}

	fn keys(&self) -> HashMap<H256, i32> {
		self.data.iter().filter_map(|(k, v)| if v.1 != 0 {Some((k.clone(), v.1))} else {None}).collect::<HashMap<H256, i32>>()
	}

	fn exists(&self, key: &H256) -> bool {
		match self.data.get(key) {
			Some(&(_, x)) if x > 0 => true,
			_ => false
		}
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
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
	assert_eq!(m.lookup(&hash).unwrap(), b"Hello world!");

	for _ in 0..1000 {
		let r = H256::random();
		let k = r.sha3();
		let &(ref v, ref rc) = m.denote(&k, r.bytes().to_vec());
		assert_eq!(v, &r.bytes());
		assert_eq!(*rc, 0);
	}

	assert_eq!(m.lookup(&hash).unwrap(), b"Hello world!");
}
