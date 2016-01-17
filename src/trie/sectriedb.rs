use hash::*;
use sha3::*;
use hashdb::*;
use rlp::*;
use super::triedb::*;
use super::trietraits::*;

/// A `Trie` implementation which hashes keys and uses a generic `HashDB` backing database.
/// 
/// Use it as a `Trie` trait object. You can use `raw()` to get the backing TrieDB object.
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

	/// Get a reference to the underlying raw TrieDB struct.
	pub fn raw(&self) -> &TrieDB {
		&self.raw
	}

	/// Get a mutable reference to the underlying raw TrieDB struct.
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
