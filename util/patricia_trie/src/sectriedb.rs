// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use hashdb::{HashDB, Hasher};
use super::triedb::TrieDB;
use super::{Result, Trie, TrieItem, TrieIterator, Query};
use node_codec::NodeCodec;

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
///
/// Use it as a `Trie` trait object. You can use `raw()` to get the backing `TrieDB` object.
pub struct SecTrieDB<'db, H, C>
where 
	H: Hasher + 'db, 
	C: NodeCodec<H>
{
	raw: TrieDB<'db, H, C>
}

impl<'db, H, C> SecTrieDB<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	/// Create a new trie with the backing database `db` and empty `root`
	///
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	/// Returns an error if root does not exist.
	pub fn new(db: &'db HashDB<H>, root: &'db H::Out) -> Result<Self, H::Out> {
		Ok(SecTrieDB { raw: TrieDB::new(db, root)? })
	}

	/// Get a reference to the underlying raw `TrieDB` struct.
	pub fn raw(&self) -> &TrieDB<H, C> {
		&self.raw
	}

	/// Get a mutable reference to the underlying raw `TrieDB` struct.
	pub fn raw_mut(&mut self) -> &mut TrieDB<'db, H, C> {
		&mut self.raw
	}
}

impl<'db, H, C> Trie for SecTrieDB<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	type H = H;

	fn root(&self) -> &<Self::H as Hasher>::Out { self.raw.root() }

	fn contains(&self, key: &[u8]) -> Result<bool, <Self::H as Hasher>::Out> {
		self.raw.contains(Self::H::hash(key).as_ref())
	}

	fn get_with<'a, 'key, Q: Query<Self::H>>(&'a self, key: &'key [u8], query: Q) -> Result<Option<Q::Item>, <Self::H as Hasher>::Out>
		where 'a: 'key
	{
		self.raw.get_with(Self::H::hash(key).as_ref(), query)
	}

	fn iter<'a>(&'a self) -> Result<Box<TrieIterator<Self::H, Item = TrieItem<Self::H>> + 'a>, <Self::H as Hasher>::Out> {
		TrieDB::iter(&self.raw)
	}
}

#[cfg(test)]
mod test {
	use memorydb::MemoryDB;
	use hashdb::{Hasher, DBValue};
	use keccak;
	use keccak_hasher::KeccakHasher;
	use ethtrie::RlpCodec;
	use ethtrie::trie::{Trie, TrieMut, TrieDBMut, SecTrieDB};

	#[test]
	fn trie_to_sectrie() {
		let mut db = MemoryDB::<KeccakHasher>::new();
		let mut root = <KeccakHasher as Hasher>::Out::default();
		{
			let mut t = TrieDBMut::<_, RlpCodec>::new(&mut db, &mut root);
			t.insert(&keccak::keccak(&[0x01u8, 0x23]), &[0x01u8, 0x23]).unwrap();
		}
		let t = SecTrieDB::<_, RlpCodec>::new(&db, &root).unwrap();
		assert_eq!(t.get(&[0x01u8, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
	}
}