//! Reference-counted memory-based HashDB implementation.
//! 
//! # Example
//! ```rust
//! extern crate ethcore_util;
//! use ethcore_util::hashdb::*;
//! use ethcore_util::memorydb::*;
//! fn main() {
//!   let mut m = MemoryDB::new();
//!   let d = "Hello world!".as_bytes();
//!
//!   let k = m.insert(d);
//!   assert!(m.exists(&k));
//!   assert_eq!(m.lookup(&k).unwrap(), &d);
//!
//!   m.insert(d);
//!   assert!(m.exists(&k));
//!
//!   m.kill(&k);
//!   assert!(m.exists(&k));
//!
//!   m.kill(&k);
//!   assert!(!m.exists(&k));
//!
//!   m.insert(d);
//!   assert!(m.exists(&k));
//!   assert_eq!(m.lookup(&k).unwrap(), &d);
//!
//!   m.kill(&k);
//!   assert!(!m.exists(&k));
//! }

//! ```
use hash::*;
use bytes::*;
use sha3::*;
use hashdb::*;
use std::collections::HashMap;

#[derive(Debug,Clone)]
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
}

impl HashDB for MemoryDB {
	/// Do a hash dereference and look up a given hash into the bytes that make it up,
	/// returning None if nothing is found (or if the only entries have 0 or fewer
	/// references).
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
	///   assert_eq!(m.lookup(&hash).unwrap(), &hello_bytes);
	/// }
	/// ```
	fn lookup(&self, key: &H256) -> Option<&Bytes> {
		match self.data.get(key) {
			Some(&(ref d, rc)) if rc > 0 => Some(d),
			_ => None
		}
	}

	/// Check for the existance of a hash-key.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
	/// use ethcore_util::sha3::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let hello_bytes = "Hello world!".as_bytes();
	///   assert!(!m.exists(&hello_bytes.sha3()));
	///   let key = m.insert(hello_bytes);
	///   assert!(m.exists(&key));
	///   m.kill(&key);
	///   assert!(!m.exists(&key));
	/// }
	/// ```
	fn exists(&self, key: &H256) -> bool {
		match self.data.get(key) {
			Some(&(_, x)) if x > 0 => true,
			_ => false
		}
	}

	/// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
	/// are counted and the equivalent number of `kill()`s must be performed before the data
	/// is considered dead.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
	/// use ethcore_util::hash::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let key = m.insert("Hello world!".as_bytes());
	///   assert!(m.exists(&key));
	/// }
	/// ```
	fn insert(&mut self, value: &[u8]) -> H256 {
		let key = value.sha3();
		if match self.data.get_mut(&key) {
			Some(&mut (ref mut old_value, ref mut rc @ 0)) => { *old_value = From::from(value.bytes()); *rc = 1; false },
			Some(&mut (_, ref mut x)) => { *x += 1; false } ,
			None => true,
		}{	// ... None falls through into...
			self.data.insert(key, (From::from(value.bytes()), 1));
		}
		key
	}
	/// Remove a datum previously inserted. Insertions can be "owed" such that the same number of inserts may
	/// happen without the data being eventually being inserted into the DB.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
	/// use ethcore_util::sha3::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let d = "Hello world!".as_bytes();
	///   let key = &d.sha3();
	///   m.kill(key);	// OK - we now owe an insertion.
	///   assert!(!m.exists(key));
	///   m.insert(d);	// OK - now it's "empty" again.
	///   assert!(!m.exists(key));
	///   m.insert(d);	// OK - now we've
	///   assert_eq!(m.lookup(key).unwrap(), &d);
	/// }
	/// ```
	fn kill(&mut self, key: &H256) {
		if match self.data.get_mut(key) {
			Some(&mut (_, ref mut x)) => { *x -= 1; false }
			None => true
		}{	// ... None falls through into...
			self.data.insert(*key, (Bytes::new(), -1));
		}
	}
}

