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

//! Database of byte-slices keyed to their Keccak hash.
use hash::*;
use bytes::*;
use std::collections::HashMap;

/// Trait modelling datastore keyed by a 32-byte Keccak hash.
pub trait HashDB : AsHashDB {
	/// Get the keys in the database together with number of underlying references.
	fn keys(&self) -> HashMap<H256, i32>;

	/// Deprecated. use `get`.
	fn lookup(&self, key: &H256) -> Option<&[u8]>; // TODO: rename to get.
	/// Look up a given hash into the bytes that hash to it, returning None if the
	/// hash is not known.
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
	///   assert_eq!(m.get(&hash).unwrap(), hello_bytes);
	/// }
	/// ```
	fn get(&self, key: &H256) -> Option<&[u8]> { self.lookup(key) }

	/// Deprecated. Use `contains`.
	fn exists(&self, key: &H256) -> bool; // TODO: rename to contains.
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
	///   assert!(!m.contains(&hello_bytes.sha3()));
	///   let key = m.insert(hello_bytes);
	///   assert!(m.contains(&key));
	///   m.remove(&key);
	///   assert!(!m.contains(&key));
	/// }
	/// ```
	fn contains(&self, key: &H256) -> bool { self.exists(key) }

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
	///   assert!(m.contains(&key));
	/// }
	/// ```
	fn insert(&mut self, value: &[u8]) -> H256;

	/// Like `insert()` , except you provide the key and the data is all moved.
	fn emplace(&mut self, key: H256, value: Bytes);

	/// Deprecated - use `remove`.
	fn kill(&mut self, key: &H256); // TODO: rename to remove.
	/// Remove a datum previously inserted. Insertions can be "owed" such that the same number of `insert()`s may
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
	///   m.remove(key);	// OK - we now owe an insertion.
	///   assert!(!m.contains(key));
	///   m.insert(d);	// OK - now it's "empty" again.
	///   assert!(!m.contains(key));
	///   m.insert(d);	// OK - now we've
	///   assert_eq!(m.get(key).unwrap(), d);
	/// }
	/// ```
	fn remove(&mut self, key: &H256) { self.kill(key) }
}

/// Upcast trait.
pub trait AsHashDB {
	/// Perform upcast to HashDB for anything that derives from HashDB.
	fn as_hashdb(&self) -> &HashDB;
	/// Perform mutable upcast to HashDB for anything that derives from HashDB.
	fn as_hashdb_mut(&mut self) -> &mut HashDB;
}

impl<T: HashDB> AsHashDB for T {
	fn as_hashdb(&self) -> &HashDB { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB { self }
}
