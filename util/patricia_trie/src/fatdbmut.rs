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

use hashdb::{HashDB, DBValue, Hasher};
use super::{Result, TrieDBMut, TrieMut};
use node_codec::NodeCodec;

/// A mutable `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
/// Additionaly it stores inserted hash-key mappings for later retrieval.
///
/// Use it as a `Trie` or `TrieMut` trait object.
pub struct FatDBMut<'db, H, C>
where 
	H: Hasher + 'db, 
	C: NodeCodec<H>
{
	raw: TrieDBMut<'db, H, C>,
}

impl<'db, H, C> FatDBMut<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	/// Create a new trie with the backing database `db` and empty `root`
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	pub fn new(db: &'db mut HashDB<H>, root: &'db mut H::Out) -> Self {
		FatDBMut { raw: TrieDBMut::new(db, root) }
	}

	/// Create a new trie with the backing database `db` and `root`.
	///
	/// Returns an error if root does not exist.
	pub fn from_existing(db: &'db mut HashDB<H>, root: &'db mut H::Out) -> Result<Self, H::Out, C::Error> {
		Ok(FatDBMut { raw: TrieDBMut::from_existing(db, root)? })
	}

	/// Get the backing database.
	pub fn db(&self) -> &HashDB<H> {
		self.raw.db()
	}

	/// Get the backing database.
	pub fn db_mut(&mut self) -> &mut HashDB<H> {
		self.raw.db_mut()
	}
}

impl<'db, H, C> TrieMut<H, C> for FatDBMut<'db, H, C>
where 
	H: Hasher, 
	C: NodeCodec<H>
{
	fn root(&mut self) -> &H::Out { self.raw.root() }

	fn is_empty(&self) -> bool { self.raw.is_empty() }

	fn contains(&self, key: &[u8]) -> Result<bool, H::Out, C::Error> {
		self.raw.contains(H::hash(key).as_ref())
	}

	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Result<Option<DBValue>, H::Out, C::Error>
		where 'a: 'key
	{
		self.raw.get(H::hash(key).as_ref())
	}

	fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<Option<DBValue>, H::Out, C::Error> {
		let hash = H::hash(key);
		let out = self.raw.insert(hash.as_ref(), value)?;
		let db = self.raw.db_mut();

		// don't insert if it doesn't exist.
		if out.is_none() {
			let aux_hash = H::hash(hash.as_ref());
			db.emplace(aux_hash, DBValue::from_slice(key));
		}
		Ok(out)
	}

	fn remove(&mut self, key: &[u8]) -> Result<Option<DBValue>, H::Out, C::Error> {
		let hash = H::hash(key);
		let out = self.raw.remove(hash.as_ref())?;

		// don't remove if it already exists.
		if out.is_some() {
			self.raw.db_mut().remove(&hash);
		}

		Ok(out)
	}
}

#[cfg(test)]
mod test {
	use hashdb::DBValue;
	use memorydb::MemoryDB;
	use ethtrie::trie::{Trie, TrieMut};
	use ethtrie::{TrieDB, FatDBMut};
	use keccak_hasher::KeccakHasher;
	use keccak;
	use ethereum_types::H256;

	#[test]
	fn fatdbmut_to_trie() {
		let mut memdb = MemoryDB::<KeccakHasher>::new();
		let mut root = H256::new();
		{
			let mut t = FatDBMut::new(&mut memdb, &mut root);
			t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		}
		let t = TrieDB::new(&memdb, &root).unwrap();
		assert_eq!(t.get(&keccak::keccak(&[0x01u8, 0x23])).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
	}
}