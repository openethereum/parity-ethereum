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
use super::{TrieDB, Trie, TrieDBIterator, TrieItem, Recorder};

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
/// Additionaly it stores inserted hash-key mappings for later retrieval.
///
/// Use it as a `Trie` or `TrieMut` trait object.
pub struct FatDB<'db> {
	raw: TrieDB<'db>,
}

impl<'db> FatDB<'db> {
	/// Create a new trie with the backing database `db` and empty `root`
	/// Initialise to the state entailed by the genesis block.
	/// This guarantees the trie is built correctly.
	pub fn new(db: &'db HashDB, root: &'db H256) -> super::Result<Self> {
		let fatdb = FatDB {
			raw: try!(TrieDB::new(db, root))
		};

		Ok(fatdb)
	}

	/// Get the backing database.
	pub fn db(&self) -> &HashDB {
		self.raw.db()
	}
}

impl<'db> Trie for FatDB<'db> {
	fn iter<'a>(&'a self) -> super::Result<Box<Iterator<Item = TrieItem> + 'a>> {
		FatDBIterator::new(&self.raw).map(|iter| Box::new(iter) as Box<_>)
	}

	fn root(&self) -> &H256 {
		self.raw.root()
	}

	fn contains(&self, key: &[u8]) -> super::Result<bool> {
		self.raw.contains(&key.sha3())
	}

	fn get_recorded<'a, 'b, R: 'b>(&'a self, key: &'b [u8], rec: &'b mut R) -> super::Result<Option<DBValue>>
		where 'a: 'b, R: Recorder
	{
		self.raw.get_recorded(&key.sha3(), rec)
	}
}

/// Itarator over inserted pairs of key values.
pub struct FatDBIterator<'db> {
	trie_iterator: TrieDBIterator<'db>,
	trie: &'db TrieDB<'db>,
}

impl<'db> FatDBIterator<'db> {
	/// Creates new iterator.
	pub fn new(trie: &'db TrieDB) -> super::Result<Self> {
		Ok(FatDBIterator {
			trie_iterator: try!(TrieDBIterator::new(trie)),
			trie: trie,
		})
	}
}

impl<'db> Iterator for FatDBIterator<'db> {
	type Item = TrieItem<'db>;

	fn next(&mut self) -> Option<Self::Item> {
		self.trie_iterator.next()
			.map(|res|
				res.map(|(hash, value)| {
					(self.trie.db().get_aux(&hash).expect("Missing fatdb hash").to_vec(), value)
				})
			)
	}
}

#[test]
fn fatdb_to_trie() {
	use memorydb::MemoryDB;
	use trie::{FatDBMut, TrieMut};

	let mut memdb = MemoryDB::new();
	let mut root = H256::default();
	{
		let mut t = FatDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
	}
	let t = FatDB::new(&memdb, &root).unwrap();
	assert_eq!(t.get(&[0x01u8, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
	assert_eq!(t.iter().unwrap().map(Result::unwrap).collect::<Vec<_>>(), vec![(vec![0x01u8, 0x23], DBValue::from_slice(&[0x01u8, 0x23] as &[u8]))]);
}
