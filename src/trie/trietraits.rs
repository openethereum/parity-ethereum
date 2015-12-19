use hash::H256;
use rlp::SHA3_NULL_RLP;

/// A key-value datastore implemented as a database-backed modified Merkle tree.
pub trait Trie {
	/// Return the root of the trie.
	fn root(&self) -> &H256;

	/// Is the trie empty?
	fn is_empty(&self) -> bool { *self.root() == SHA3_NULL_RLP }

	/// Does the trie contain a given key?
	fn contains(&self, key: &[u8]) -> bool;

	/// What is the value of the given key in this trie?
	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Option<&'a [u8]> where 'a: 'key;
}

/// A key-value datastore implemented as a database-backed modified Merkle tree.
pub trait TrieMut: Trie {
	/// Insert a `key`/`value` pair into the trie. An `empty` value is equivalent to removing
	/// `key` from the trie.
	fn insert(&mut self, key: &[u8], value: &[u8]);

	/// Remove a `key` from the trie. Equivalent to making it equal to the empty
	/// value.
	fn remove(&mut self, key: &[u8]);
}

