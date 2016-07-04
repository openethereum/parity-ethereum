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

use hashdb::HashDB;
use hash::H256;
use rlp::*;
use bytes::Bytes;
use sha3::*;

/// `HashDB` wrapper which keeps the RLP values compressed.
pub trait CompressedDB: HashDB {
	/// Look up a given hash into the bytes that hash to it, returning None if the
	/// hash is not known.
	fn get(&self, key: &H256) -> Option<ElasticArray1024<u8>> {
		self.as_hashdb()
			.get(key)
			.map(|compressed| UntrustedRlp::new(compressed).decompress())
	}
	/// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
	/// are counted and the equivalent number of `remove()`s must be performed before the data
	/// is considered dead.
	fn insert(&mut self, value: &[u8]) -> H256 {
		let key = value.sha3();
		self.as_hashdb_mut().emplace(key, UntrustedRlp::new(value).compress().to_vec());
		key
	}
	/// Like `insert()` , except you provide the key and the data is all moved.
	fn emplace(&mut self, key: H256, value: Bytes) {
		self.as_hashdb_mut().emplace(key, UntrustedRlp::new(&value).compress().to_vec())
	}	
}

#[test]
fn compressed_db() {
