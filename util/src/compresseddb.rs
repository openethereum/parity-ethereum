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

//! Wrapper over `HashDB` which keeps the values compressed.

use std::collections::HashMap;
use hashdb::HashDB;
use hash::H256;
use rlp::*;
use bytes::Bytes;
use sha3::*;

/// `HashDB` wrapper which keeps the RLP values compressed.
pub trait CompressedDB: HashDB {
	/// Get the keys in the database together with number of underlying references.
	fn keys(&self) -> HashMap<H256, i32>;

	/// Look up a given hash into the bytes that hash to it, returning None if the
	/// hash is not known.
	/// 
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::compresseddb::*;
	/// use ethcore_util::memorydb::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let hello_bytes = "Hello world!".as_bytes();
	///   let hash = m.insert(hello_bytes);
	///   assert_eq!(m.get(&hash).unwrap(), hello_bytes);
	/// }
	/// ```
	fn get(&self, key: &H256) -> Option<ElasticArray1024<u8>> {
		self.as_hashdb()
			.get(key)
			.map(|compressed| UntrustedRlp::new(compressed).decompress())
	}

	/// Check for the existance of a hash-key.
	fn contains(&self, key: &H256) -> bool;

	/// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
	/// are counted and the equivalent number of `remove()`s must be performed before the data
	/// is considered dead.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::compresseddb::*;
	/// use ethcore_util::memorydb::*;
	/// use ethcore_util::hash::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let key = m.insert("Hello world!".as_bytes());
	///   assert!(m.contains(&key));
	/// }
	/// ```
	fn insert(&mut self, value: &[u8]) -> H256 {
		let key = value.sha3();
		self.as_hashdb_mut().emplace(key, UntrustedRlp::new(value).compress().to_vec());
		key
	}

	/// Like `insert()` , except you provide the key and the data is all moved.
	fn emplace(&mut self, key: H256, value: Bytes) {
		self.as_hashdb_mut().emplace(key, UntrustedRlp::new(&value).compress().to_vec())
	}	

	/// Remove a datum previously inserted. Insertions can be "owed" such that the same number of `insert()`s may
	/// happen without the data being eventually being inserted into the DB.
	fn remove(&mut self, key: &H256);

	/// Insert auxiliary data into hashdb.
	fn insert_aux(&mut self, _hash: Vec<u8>, _value: Vec<u8>) {
		unimplemented!();
	}

	/// Get auxiliary data from hashdb.
	fn get_aux(&self, _hash: &[u8]) -> Option<Vec<u8>> {
		unimplemented!();
	}

	/// Removes auxiliary data from hashdb.
	fn remove_aux(&mut self, _hash: &[u8]) {
		unimplemented!();
	}
}
