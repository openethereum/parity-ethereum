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
use MemoryDB;

/// Backing compressed `HashDB` with decompressed `MemoryDB` overlay.
pub struct CompressedDB<'a, T: 'a + HashDB> {
	overlay: MemoryDB,
	backing: &'a mut T,
}

impl<'a, T: 'a + HashDB> CompressedDB<'a, T> {
	/// Create a compressing wrapper for `backing` db.
	pub fn new(backing: &'a mut T) -> CompressedDB<'a, T> {
		CompressedDB { overlay: MemoryDB::new(), backing: backing }
	}
}

/// `HashDB` wrapper which keeps the RLP values compressed.
impl<'a, T> HashDB for CompressedDB<'a, T> where T: HashDB {
	fn keys(&self) -> HashMap<H256, i32> { self.backing.keys() }

	fn get(&self, key: &H256) -> Option<&[u8]> {
		self.overlay.get(key).or(self.backing.get(key).and_then(|v| {
			let decompressed = UntrustedRlp::new(v).decompress().to_vec();
			let raw = self.overlay.denote(key, decompressed);
			if raw.1 > 0 { Some(raw.0.as_slice()) } else { None }
		}))
	}

	fn contains(&self, key: &H256) -> bool { self.backing.contains(key) }

	fn insert(&mut self, value: &[u8]) -> H256 {
		let key = value.sha3();
		self.overlay.emplace(key, value.to_vec());
		self.backing.emplace(key, UntrustedRlp::new(value).compress().to_vec());
		key
	}

	fn emplace(&mut self, key: H256, value: Bytes) {
		self.backing.emplace(key, UntrustedRlp::new(&value).compress().to_vec());
		self.overlay.emplace(key, value)
	}	

	fn remove(&mut self, key: &H256) { self.backing.remove(key) }

	fn insert_aux(&mut self, hash: Vec<u8>, value: Vec<u8>) {
		self.backing.insert_aux(hash, value)
	}

	fn get_aux(&self, hash: &[u8]) -> Option<Vec<u8>> {
		self.backing.get_aux(hash)
	}

	fn remove_aux(&mut self, hash: &[u8]) {
		self.backing.remove_aux(hash)
	}
}

#[test]
fn compressed_db() {
	let mut backing = MemoryDB::new();
	let common_rlp = vec![160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33];
	{
		let mut db: CompressedDB<MemoryDB> = CompressedDB::new(&mut backing);
		let key = db.insert(&common_rlp);
		assert_eq!(db.get(&key).unwrap(), common_rlp.as_slice());
	}
	let compressed_rlp = backing.get(backing.keys().keys().next().unwrap()).unwrap();
	assert_eq!(compressed_rlp.len(), 2);
}
