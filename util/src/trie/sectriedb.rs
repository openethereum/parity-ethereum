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

use hash::H256;
use sha3::Hashable;
use hashdb::{HashDB, DBValue};
use super::triedb::TrieDB;
use super::{Trie, TrieItem, Recorder};

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
///
/// Use it as a `Trie` trait object. You can use `raw()` to get the backing `TrieDB` object.
pub struct SecTrieDB<'db> {
	raw: TrieDB<'db>
}

impl<'db> SecTrieDB<'db> {
	/// Create a new trie with the backing database `db` and empty `root`
	///
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	/// Returns an error if root does not exist.
	pub fn new(db: &'db HashDB, root: &'db H256) -> super::Result<Self> {
		Ok(SecTrieDB { raw: try!(TrieDB::new(db, root)) })
	}

	/// Get a reference to the underlying raw `TrieDB` struct.
	pub fn raw(&self) -> &TrieDB {
		&self.raw
	}

	/// Get a mutable reference to the underlying raw `TrieDB` struct.
	pub fn raw_mut(&mut self) -> &mut TrieDB<'db> {
		&mut self.raw
	}
}

impl<'db> Trie for SecTrieDB<'db> {
	fn iter<'a>(&'a self) -> super::Result<Box<Iterator<Item = TrieItem> + 'a>> {
		TrieDB::iter(&self.raw)
	}

	fn root(&self) -> &H256 { self.raw.root() }

	fn contains(&self, key: &[u8]) -> super::Result<bool> {
		self.raw.contains(&key.sha3())
	}

	fn get_recorded<'a, 'b, R: 'b>(&'a self, key: &'b [u8], rec: &'b mut R) -> super::Result<Option<DBValue>>
		where 'a: 'b, R: Recorder
	{
		self.raw.get_recorded(&key.sha3(), rec)
	}
}

#[test]
fn trie_to_sectrie() {
	use memorydb::MemoryDB;
	use super::triedbmut::TrieDBMut;
	use super::super::TrieMut;

	let mut memdb = MemoryDB::new();
	let mut root = H256::default();
	{
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&(&[0x01u8, 0x23]).sha3(), &[0x01u8, 0x23]).unwrap();
	}
	let t = SecTrieDB::new(&memdb, &root).unwrap();
	assert_eq!(t.get(&[0x01u8, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
}
