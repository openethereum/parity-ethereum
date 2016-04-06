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

use hash::*;
use sha3::*;
use hashdb::*;
use rlp::*;
use super::triedb::*;
use super::trietraits::*;

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
///
/// Use it as a `Trie` trait object. You can use `raw()` to get the backing `TrieDB` object.
pub struct SecTrieDB<'db> {
	raw: TrieDB<'db>
}

impl<'db> SecTrieDB<'db> {
	/// Create a new trie with the backing database `db` and empty `root`
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	pub fn new(db: &'db HashDB, root: &'db H256) -> Self {
		SecTrieDB { raw: TrieDB::new(db, root) }
	}

	/// Get a reference to the underlying raw `TrieDB` struct.
	pub fn raw(&self) -> &TrieDB {
		&self.raw
	}

	/// Get a mutable reference to the underlying raw `TrieDB` struct.
	pub fn raw_mut(&mut self) -> &TrieDB {
		&mut self.raw
	}
}

impl<'db> Trie for SecTrieDB<'db> {
	fn root(&self) -> &H256 { self.raw.root() }

	fn contains(&self, key: &[u8]) -> bool {
		self.raw.contains(&key.sha3())
	}

	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Option<&'a [u8]> where 'a: 'key {
		self.raw.get(&key.sha3())
	}
}

#[test]
fn trie_to_sectrie() {
	use memorydb::*;
	use super::triedbmut::*;

	let mut memdb = MemoryDB::new();
	let mut root = H256::new();
	{
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&(&[0x01u8, 0x23]).sha3(), &[0x01u8, 0x23]);
	}
	let t = SecTrieDB::new(&memdb, &root);
	assert_eq!(t.get(&[0x01u8, 0x23]).unwrap(), &[0x01u8, 0x23]);
}
