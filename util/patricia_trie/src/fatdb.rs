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
use super::{Result, TrieDB, Trie, TrieDBIterator, TrieItem, TrieIterator, Query};
use node_codec::NodeCodec;

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
/// Additionaly it stores inserted hash-key mappings for later retrieval.
///
/// Use it as a `Trie` or `TrieMut` trait object.
pub struct FatDB<'db, H, C>
where 
	H: Hasher + 'db, 
	C: NodeCodec<H>
{
	raw: TrieDB<'db, H, C>,
}

impl<'db, H, C> FatDB<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	/// Create a new trie with the backing database `db` and empty `root`
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	pub fn new(db: &'db HashDB<H>, root: &'db H::Out) -> Result<Self, H::Out, C::Error> {
		Ok(FatDB { raw: TrieDB::new(db, root)? })
	}

	/// Get the backing database.
	pub fn db(&self) -> &HashDB<H> { self.raw.db() }
}

impl<'db, H, C> Trie<H, C> for FatDB<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	fn root(&self) -> &H::Out { self.raw.root() }

	fn contains(&self, key: &[u8]) -> Result<bool, H::Out, C::Error> {
		self.raw.contains(H::hash(key).as_ref())
	}

	fn get_with<'a, 'key, Q: Query<H>>(&'a self, key: &'key [u8], query: Q) -> Result<Option<Q::Item>, H::Out, C::Error>
		where 'a: 'key
	{
		self.raw.get_with(H::hash(key).as_ref(), query)
	}

	fn iter<'a>(&'a self) -> Result<Box<TrieIterator<H, C, Item = TrieItem<H::Out, C::Error>> + 'a>, <H as Hasher>::Out, C::Error> {
		FatDBIterator::<H, C>::new(&self.raw).map(|iter| Box::new(iter) as Box<_>)
	}
}

/// Itarator over inserted pairs of key values.
pub struct FatDBIterator<'db, H, C>
where 
	H: Hasher + 'db, 
	C: NodeCodec<H> + 'db
{
	trie_iterator: TrieDBIterator<'db, H, C>,
	trie: &'db TrieDB<'db, H, C>,
}

impl<'db, H, C> FatDBIterator<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	/// Creates new iterator.
	pub fn new(trie: &'db TrieDB<H, C>) -> Result<Self, H::Out, C::Error> {
		Ok(FatDBIterator {
			trie_iterator: TrieDBIterator::new(trie)?,
			trie: trie,
		})
	}
}

impl<'db, H, C> TrieIterator<H, C> for FatDBIterator<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	fn seek(&mut self, key: &[u8]) -> Result<(), H::Out, C::Error> {
		let hashed_key = H::hash(key);
		self.trie_iterator.seek(hashed_key.as_ref())
	}
}

impl<'db, H, C> Iterator for FatDBIterator<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	type Item = TrieItem<'db, H::Out, C::Error>;

	fn next(&mut self) -> Option<Self::Item> {
		self.trie_iterator.next()
			.map(|res| {
				res.map(|(hash, value)| {
					let aux_hash = H::hash(&hash);
					(self.trie.db().get(&aux_hash).expect("Missing fatdb hash").into_vec(), value)
				})
			})
	}
}

#[cfg(test)]
mod test {
	use memorydb::MemoryDB;
	use hashdb::DBValue;
	use keccak_hasher::KeccakHasher;
	use ethtrie::trie::{Trie, TrieMut};
	use ethtrie::{FatDB, FatDBMut};
	use ethereum_types::H256;

	#[test]
	fn fatdb_to_trie() {
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		{
			let mut t = FatDBMut::new(&mut memdb, &mut root);
			t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		}
		let t = FatDB::new(&memdb, &root).unwrap();
		assert_eq!(t.get(&[0x01u8, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
		assert_eq!(
			t.iter().unwrap().map(Result::unwrap).collect::<Vec<_>>(),
			vec![(vec![0x01u8, 0x23], DBValue::from_slice(&[0x01u8, 0x23] as &[u8]))]);
	}
}
