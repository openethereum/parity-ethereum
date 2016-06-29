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
pub trait TrieMut {
	/// Return the root of the trie.
	fn root(&mut self) -> &H256;

	/// Is the trie empty?
	fn is_empty(&self) -> bool;

	/// Does the trie contain a given key?
	fn contains(&self, key: &[u8]) -> bool;

	/// What is the value of the given key in this trie?
	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Option<&'a [u8]> where 'a: 'key;

	/// Insert a `key`/`value` pair into the trie. An `empty` value is equivalent to removing
	/// `key` from the trie.
	fn insert(&mut self, key: &[u8], value: &[u8]);

	/// Remove a `key` from the trie. Equivalent to making it equal to the empty
	/// value.
	fn remove(&mut self, key: &[u8]);
}

