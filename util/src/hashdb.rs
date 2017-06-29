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

//! Database of byte-slices keyed to their Keccak hash.

use elastic_array::ElasticArray128;
use hash::*;
use std::collections::HashMap;

/// `HashDB` value type.
pub type DBValue = ElasticArray128<u8>;

/// Supplemental trait to `HashDB` supplying generic methods for trait objects, internally
/// calling monomorphic forms of these methods.
pub trait HashDBExt {
	/// Get a reference, transform it, and return the result (prevents cloning).
	fn get_with<Out, F: FnOnce(&[u8]) -> Out>(&self, _: &H256, _: F) -> Option<Out>;
}

macro_rules! get_with_fn_def {
	() => {
		fn get_with<Out, F: for<'a> FnOnce(&'a [u8]) -> Out>(
			&self,
			key: &H256,
			f: F,
		) -> Option<Out> {
			let mut o_func: Option<F>   = Some(f);
			let mut output: Option<Out> = None;

			{
				let mut wrapper = |key: &[u8]| {
					output = Some(
						(
							o_func.take().expect(
								"The implementation of `get_exec` called its argument twice - this \
								 is a bug!"
							)
						)(key));
				};

				self.get_exec(key, &mut wrapper);
			}

			output
		}
	}
}

impl<T: HashDB> HashDBExt for T {
	get_with_fn_def!{}
}

impl HashDBExt for Box<HashDB> {
	get_with_fn_def!{}
}

impl<'any> HashDBExt for &'any HashDB {
	get_with_fn_def!{}
}

impl<'any> HashDBExt for &'any mut HashDB {
	get_with_fn_def!{}
}

impl HashDBExt for ::std::rc::Rc<HashDB> {
	get_with_fn_def!{}
}

impl HashDBExt for ::std::sync::Arc<HashDB> {
	get_with_fn_def!{}
}

/// Trait modelling datastore keyed by a 32-byte Keccak hash.
pub trait HashDB: AsHashDB + Send + Sync {
	/// Get the keys in the database together with number of underlying references.
	fn keys(&self) -> HashMap<H256, i32>;

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
	fn get(&self, key: &H256) -> Option<DBValue> {
		let mut output = None;

		{
			let mut wrapper = |key: &[u8]| { output = Some(DBValue::from_slice(key)); };

			self.get_exec(key, &mut wrapper);
		}

		output
	}

	/// Any implementation of this function should call `f` zero times if a value
	/// for `key` is not found, and precisely once if a value is found. The
	/// default implementation of `get_with` will panic if `f` is called more than
	/// once and in general there is no sensible behavior if `f` is called more
	/// than once.
	fn get_exec(
		&self,
		key: &H256,
		f: &mut FnMut(&[u8])
	);

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
	fn contains(&self, key: &H256) -> bool;

	/// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
	/// are counted and the equivalent number of `remove()`s must be performed before the data
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
	fn emplace(&mut self, key: H256, value: DBValue);

	/// Remove a datum previously inserted. Insertions can be "owed" such that the same number of `insert()`s may
	/// happen without the data being eventually being inserted into the DB. It can be "owed" more than once.
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
	///   m.remove(key);	// OK - we now owe two insertions.
	///   assert!(!m.contains(key));
	///   m.insert(d);	// OK - still owed.
	///   assert!(!m.contains(key));
	///   m.insert(d);	// OK - now it's "empty" again.
	///   assert!(!m.contains(key));
	///   m.insert(d);	// OK - now we've
	///   assert_eq!(m.get(key).unwrap(), d);
	/// }
	/// ```
	fn remove(&mut self, key: &H256);
}

/// Upcast trait.
pub trait AsHashDB {
	/// Perform upcast to HashDB for anything that derives from HashDB.
	fn as_hashdb(&self) -> &HashDB;
	/// Perform mutable upcast to HashDB for anything that derives from HashDB.
	fn as_hashdb_mut(&mut self) -> &mut HashDB;
}

impl<T: HashDB> AsHashDB for T {
	fn as_hashdb(&self) -> &HashDB {
		self
	}
	fn as_hashdb_mut(&mut self) -> &mut HashDB {
		self
	}
}

impl<'a> AsHashDB for &'a mut HashDB {
	fn as_hashdb(&self) -> &HashDB {
		&**self
	}

	fn as_hashdb_mut(&mut self) -> &mut HashDB {
		&mut **self
	}
}
