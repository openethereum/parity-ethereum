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
use super::{TrieDB, Trie, TrieDBIterator, TrieItem, TrieIterator, Query};
use rlp::{Decodable, Encodable};
use node_codec::NodeCodec;

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
/// Additionaly it stores inserted hash-key mappings for later retrieval.
///
/// Use it as a `Trie` or `TrieMut` trait object.
pub struct FatDB<'db, H, C>
	where H: Hasher + 'db, C: NodeCodec<H>
{
	raw: TrieDB<'db, H, C>,
}

impl<'db, H, C> FatDB<'db, H, C>
	where H: Hasher, H::Out: Decodable, C: NodeCodec<H>
{
	/// Create a new trie with the backing database `db` and empty `root`
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	pub fn new(db: &'db HashDB<H>, root: &'db H::Out) -> super::Result<Self, H::Out> {
		Ok(FatDB { raw: TrieDB::new(db, root)? })
	}

	/// Get the backing database.
	pub fn db(&self) -> &HashDB<H> { self.raw.db() }
}

impl<'db, H, C> Trie for FatDB<'db, H, C>
	where H: Hasher, H::Out: Decodable + Encodable, C: NodeCodec<H>
{
	type H = H;

	fn root(&self) -> &<Self::H as Hasher>::Out { self.raw.root() }

	fn contains(&self, key: &[u8]) -> super::Result<bool, <Self::H as Hasher>::Out> {
		self.raw.contains(Self::H::hash(key).as_ref())
	}

	fn get_with<'a, 'key, Q: Query<Self::H>>(&'a self, key: &'key [u8], query: Q) -> super::Result<Option<Q::Item>, <Self::H as Hasher>::Out>
		where 'a: 'key
	{
		self.raw.get_with(Self::H::hash(key).as_ref(), query)
	}

	fn iter<'a>(&'a self) -> super::Result<Box<TrieIterator<Self::H, Item = TrieItem<Self::H>> + 'a>, <Self::H as Hasher>::Out> {
		FatDBIterator::<Self::H, C>::new(&self.raw).map(|iter| Box::new(iter) as Box<_>)
	}
}

/// Itarator over inserted pairs of key values.
pub struct FatDBIterator<'db, H, C>
	where H: Hasher + 'db, C: NodeCodec<H> + 'db
{
	trie_iterator: TrieDBIterator<'db, H, C>,
	trie: &'db TrieDB<'db, H, C>,
}

impl<'db, H, C> FatDBIterator<'db, H, C>
	where H: Hasher, H::Out: Decodable, C: NodeCodec<H>
{
	/// Creates new iterator.
	pub fn new(trie: &'db TrieDB<H, C>) -> super::Result<Self, H::Out> {
		Ok(FatDBIterator {
			trie_iterator: TrieDBIterator::new(trie)?,
			trie: trie,
		})
	}
}

impl<'db, H, C> TrieIterator<H> for FatDBIterator<'db, H, C>
	where H: Hasher, H::Out: Decodable, C: NodeCodec<H>
{
	fn seek(&mut self, key: &[u8]) -> super::Result<(), H::Out> {
		let hashed_key = H::hash(key);
		self.trie_iterator.seek(hashed_key.as_ref())
	}
}

impl<'db, H, C> Iterator for FatDBIterator<'db, H, C>
	where H: Hasher, H::Out: Decodable, C: NodeCodec<H>
{
	type Item = TrieItem<'db, H>;

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

#[test]
fn fatdb_to_trie() {
	use memorydb::MemoryDB;
	use hashdb::DBValue;
	use super::fatdbmut::FatDBMut;
	use super::TrieMut;
	use hashdb::KeccakHasher;
	use node_codec::RlpNodeCodec;

	let mut memdb = MemoryDB::<KeccakHasher>::new();
	let mut root = <KeccakHasher as Hasher>::Out::default();
	{
		let mut t = FatDBMut::<_, RlpNodeCodec<_>>::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
	}
	let t = FatDB::<_, RlpNodeCodec<_>>::new(&memdb, &root).unwrap();
	assert_eq!(t.get(&[0x01u8, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
	assert_eq!(
		t.iter().unwrap().map(Result::unwrap).collect::<Vec<_>>(),
		vec![(vec![0x01u8, 0x23], DBValue::from_slice(&[0x01u8, 0x23] as &[u8]))]);
}
