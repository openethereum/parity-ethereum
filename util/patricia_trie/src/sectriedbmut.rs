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

use ethereum_types::H256;
use keccak::keccak;
use hashdb::{HashDB, DBValue, Hasher};
use super::triedbmut::TrieDBMut;
use super::TrieMut;

/// A mutable `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
///
/// Use it as a `Trie` or `TrieMut` trait object. You can use `raw()` to get the backing `TrieDBMut` object.
pub struct SecTrieDBMut<'db, H: Hasher + 'db> {
	raw: TrieDBMut<'db, H>
}

impl<'db, H: Hasher> SecTrieDBMut<'db, H> {
	/// Create a new trie with the backing database `db` and empty `root`
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	pub fn new(db: &'db mut HashDB<H=H>, root: &'db mut H256) -> Self {
		SecTrieDBMut { raw: TrieDBMut::new(db, root) }
	}

	/// Create a new trie with the backing database `db` and `root`.
	///
	/// Returns an error if root does not exist.
	pub fn from_existing(db: &'db mut HashDB<H=H>, root: &'db mut H256) -> super::Result<Self> {
		Ok(SecTrieDBMut { raw: TrieDBMut::from_existing(db, root)? })
	}

	/// Get the backing database.
	pub fn db(&self) -> &HashDB<H=H> { self.raw.db() }

	/// Get the backing database.
	pub fn db_mut(&mut self) -> &mut HashDB<H=H> { self.raw.db_mut() }
}

impl<'db, H: Hasher> TrieMut for SecTrieDBMut<'db, H> {
	type H = H;
	fn root(&mut self) -> &<Self::H as Hasher>::Out {
		self.raw.root()
	}

	fn is_empty(&self) -> bool {
		self.raw.is_empty()
	}

	fn contains(&self, key: &[u8]) -> super::Result<bool> {
		self.raw.contains(&keccak(key))
	}

	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> super::Result<Option<DBValue>>
		where 'a: 'key
	{
		self.raw.get(&keccak(key)) // TODO
	}

	fn insert(&mut self, key: &[u8], value: &[u8]) -> super::Result<Option<DBValue>> {
		self.raw.insert(&keccak(key), value) // TODO
	}

	fn remove(&mut self, key: &[u8]) -> super::Result<Option<DBValue>> {
		self.raw.remove(&keccak(key)) // TODO
	}
}

#[test]
fn sectrie_to_trie() {
	use memorydb::*;
	use super::triedb::*;
	use super::Trie;

	let mut memdb = MemoryDB::new();
	let mut root = H256::default();
	{
		let mut t = SecTrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
	}
	let t = TrieDB::new(&memdb, &root).unwrap();
	assert_eq!(t.get(&keccak(&[0x01u8, 0x23])).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
}
