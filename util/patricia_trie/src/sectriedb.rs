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

//use ethereum_types::H256;
//use keccak::keccak;
use hashdb::{HashDB, Hasher};
use super::triedb::TrieDB;
use super::{Trie, TrieItem, TrieIterator, Query};
use rlp::{Decodable, Encodable};

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
///
/// Use it as a `Trie` trait object. You can use `raw()` to get the backing `TrieDB` object.
pub struct SecTrieDB<'db, H: Hasher + 'db> {
	raw: TrieDB<'db, H>
}

impl<'db, H: Hasher> SecTrieDB<'db, H> where H::Out: Decodable {
	/// Create a new trie with the backing database `db` and empty `root`
	///
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	/// Returns an error if root does not exist.
	pub fn new(db: &'db HashDB<H=H>, root: &'db H::Out) -> super::Result<Self, H::Out> {
		Ok(SecTrieDB { raw: TrieDB::new(db, root)? })
	}

	/// Get a reference to the underlying raw `TrieDB` struct.
	pub fn raw(&self) -> &TrieDB<H> {
		&self.raw
	}

	/// Get a mutable reference to the underlying raw `TrieDB` struct.
	pub fn raw_mut(&mut self) -> &mut TrieDB<'db, H> {
		&mut self.raw
	}
}

impl<'db, H: Hasher> Trie for SecTrieDB<'db, H> where H::Out: Decodable + Encodable {
	type H = H;

	fn root(&self) -> &<Self::H as Hasher>::Out { self.raw.root() }

	fn contains(&self, key: &[u8]) -> super::Result<bool, <Self::H as Hasher>::Out> {
		self.raw.contains(Self::H::hash(key).as_ref())
	}

	fn get_with<'a, 'key, Q: Query<Self::H>>(&'a self, key: &'key [u8], query: Q) -> super::Result<Option<Q::Item>,  <Self::H as Hasher>::Out>
		where 'a: 'key
	{
		self.raw.get_with(Self::H::hash(key).as_ref(), query)
	}

	fn iter<'a>(&'a self) -> super::Result<Box<TrieIterator<Self::H, Item = TrieItem<Self::H>> + 'a>, <Self::H as Hasher>::Out> {
		TrieDB::iter(&self.raw)
	}
}

#[test]
fn trie_to_sectrie() {
	use memorydb::MemoryDB;
	use hashdb::DBValue;
	use super::triedbmut::TrieDBMut;
	use super::TrieMut;
	use hashdb::KeccakHasher;
	use keccak;

	let mut memdb = MemoryDB::<KeccakHasher>::new();
	let mut root = <KeccakHasher as Hasher>::Out::default();
	{
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&keccak::keccak(&[0x01u8, 0x23]), &[0x01u8, 0x23]).unwrap();
	}
	let t = SecTrieDB::new(&memdb, &root).unwrap();
	assert_eq!(t.get(&[0x01u8, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
}
