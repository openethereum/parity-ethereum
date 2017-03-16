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

//! Trie interface and implementation.

use std::fmt;
use hash::H256;
use hashdb::{HashDB, DBValue};

/// Export the standardmap module.
pub mod standardmap;
/// Export the node module.
pub mod node;
/// Export the triedb module.
pub mod triedb;
/// Export the triedbmut module.
pub mod triedbmut;
/// Export the sectriedb module.
pub mod sectriedb;
/// Export the sectriedbmut module.
pub mod sectriedbmut;
/// Trie query recording.
pub mod recorder;


mod fatdb;
mod fatdbmut;
mod lookup;

pub use self::standardmap::{Alphabet, StandardMap, ValueMode};
pub use self::triedbmut::TrieDBMut;
pub use self::triedb::{TrieDB, TrieDBIterator};
pub use self::sectriedbmut::SecTrieDBMut;
pub use self::sectriedb::SecTrieDB;
pub use self::fatdb::{FatDB, FatDBIterator};
pub use self::fatdbmut::FatDBMut;
pub use self::recorder::Recorder;

/// Trie Errors.
///
/// These borrow the data within them to avoid excessive copying on every
/// trie operation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TrieError {
	/// Attempted to create a trie with a state root not in the DB.
	InvalidStateRoot(H256),
	/// Trie item not found in the database,
	IncompleteDatabase(H256),
}

impl fmt::Display for TrieError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			TrieError::InvalidStateRoot(ref root) => write!(f, "Invalid state root: {}", root),
			TrieError::IncompleteDatabase(ref missing) =>
				write!(f, "Database missing expected key: {}", missing),
		}
	}
}

/// Trie result type. Boxed to avoid copying around extra space for `H256`s on successful queries.
pub type Result<T> = ::std::result::Result<T, Box<TrieError>>;

/// Trie-Item type.
pub type TrieItem<'a> = Result<(Vec<u8>, DBValue)>;

/// Description of what kind of query will be made to the trie.
///
/// This is implemented for any &mut recorder (where the query will return
/// a DBValue), any function taking raw bytes (where no recording will be made),
/// or any tuple of (&mut Recorder, FnOnce(&[u8]))
pub trait Query {
	/// Output item.
	type Item;

	/// Decode a byte-slice into the desired item.
	fn decode(self, &[u8]) -> Self::Item;

	/// Record that a node has been passed through.
	fn record(&mut self, &H256, &[u8], u32) { }
}

impl<'a> Query for &'a mut Recorder {
	type Item = DBValue;

	fn decode(self, value: &[u8]) -> DBValue { DBValue::from_slice(value) }
	fn record(&mut self, hash: &H256, data: &[u8], depth: u32) {
		(&mut **self).record(hash, data, depth);
	}
}

impl<F, T> Query for F where F: for<'a> FnOnce(&'a [u8]) -> T {
	type Item = T;

	fn decode(self, value: &[u8]) -> T { (self)(value) }
}

impl<'a, F, T> Query for (&'a mut Recorder, F) where F: FnOnce(&[u8]) -> T {
	type Item = T;

	fn decode(self, value: &[u8]) -> T { (self.1)(value) }
	fn record(&mut self, hash: &H256, data: &[u8], depth: u32) {
		self.0.record(hash, data, depth)
	}
}

/// A key-value datastore implemented as a database-backed modified Merkle tree.
pub trait Trie {
	/// Return the root of the trie.
	fn root(&self) -> &H256;

	/// Is the trie empty?
	fn is_empty(&self) -> bool { *self.root() == ::sha3::SHA3_NULL_RLP }

	/// Does the trie contain a given key?
	fn contains(&self, key: &[u8]) -> Result<bool> {
		self.get(key).map(|x| x.is_some())
	}

	/// What is the value of the given key in this trie?
	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Result<Option<DBValue>> where 'a: 'key {
		self.get_with(key, DBValue::from_slice)
	}

	/// Search for the key with the given query parameter. See the docs of the `Query`
	/// trait for more details.
	fn get_with<'a, 'key, Q: Query>(&'a self, key: &'key [u8], query: Q)
		-> Result<Option<Q::Item>> where 'a: 'key;

	/// Returns a depth-first iterator over the elements of trie.
	fn iter<'a>(&'a self) -> Result<Box<TrieIterator<Item = TrieItem> + 'a>>;
}

/// A key-value datastore implemented as a database-backed modified Merkle tree.
pub trait TrieMut {
	/// Return the root of the trie.
	fn root(&mut self) -> &H256;

	/// Is the trie empty?
	fn is_empty(&self) -> bool;

	/// Does the trie contain a given key?
	fn contains(&self, key: &[u8]) -> Result<bool> {
		self.get(key).map(|x| x.is_some())
	}

	/// What is the value of the given key in this trie?
	fn get<'a, 'key>(&'a self, key: &'key [u8]) -> Result<Option<DBValue>> where 'a: 'key;

	/// Insert a `key`/`value` pair into the trie. An empty value is equivalent to removing
	/// `key` from the trie. Returns the old value associated with this key, if it existed.
	fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<Option<DBValue>>;

	/// Remove a `key` from the trie. Equivalent to making it equal to the empty
	/// value. Returns the old value associated with this key, if it existed.
	fn remove(&mut self, key: &[u8]) -> Result<Option<DBValue>>;
}

/// A trie iterator that also supports random access.
pub trait TrieIterator : Iterator {
	/// Position the iterator on the first element with key > `key`
	fn seek(&mut self, key: &[u8]) -> Result<()>;
}

/// Trie types
#[derive(Debug, PartialEq, Clone)]
pub enum TrieSpec {
	/// Generic trie.
	Generic,
	/// Secure trie.
	Secure,
	///	Secure trie with fat database.
	Fat,
}

impl Default for TrieSpec {
	fn default() -> TrieSpec {
		TrieSpec::Secure
	}
}

/// Trie factory.
#[derive(Default, Clone)]
pub struct TrieFactory {
	spec: TrieSpec,
}

/// All different kinds of tries.
/// This is used to prevent a heap allocation for every created trie.
pub enum TrieKinds<'db> {
	/// A generic trie db.
	Generic(TrieDB<'db>),
	/// A secure trie db.
	Secure(SecTrieDB<'db>),
	/// A fat trie db.
	Fat(FatDB<'db>),
}

// wrapper macro for making the match easier to deal with.
macro_rules! wrapper {
	($me: ident, $f_name: ident, $($param: ident),*) => {
		match *$me {
			TrieKinds::Generic(ref t) => t.$f_name($($param),*),
			TrieKinds::Secure(ref t) => t.$f_name($($param),*),
			TrieKinds::Fat(ref t) => t.$f_name($($param),*),
		}
	}
}

impl<'db> Trie for TrieKinds<'db> {
	fn root(&self) -> &H256 {
		wrapper!(self, root,)
	}

	fn is_empty(&self) -> bool {
		wrapper!(self, is_empty,)
	}

	fn contains(&self, key: &[u8]) -> Result<bool> {
		wrapper!(self, contains, key)
	}

	fn get_with<'a, 'key, Q: Query>(&'a self, key: &'key [u8], query: Q) -> Result<Option<Q::Item>>
		where 'a: 'key
	{
		wrapper!(self, get_with, key, query)
	}

	fn iter<'a>(&'a self) -> Result<Box<TrieIterator<Item = TrieItem> + 'a>> {
		wrapper!(self, iter,)
	}
}

#[cfg_attr(feature="dev", allow(wrong_self_convention))]
impl TrieFactory {
	/// Creates new factory.
	pub fn new(spec: TrieSpec) -> Self {
		TrieFactory {
			spec: spec,
		}
	}

	/// Create new immutable instance of Trie.
	pub fn readonly<'db>(&self, db: &'db HashDB, root: &'db H256) -> Result<TrieKinds<'db>> {
		match self.spec {
			TrieSpec::Generic => Ok(TrieKinds::Generic(TrieDB::new(db, root)?)),
			TrieSpec::Secure => Ok(TrieKinds::Secure(SecTrieDB::new(db, root)?)),
			TrieSpec::Fat => Ok(TrieKinds::Fat(FatDB::new(db, root)?)),
		}
	}

	/// Create new mutable instance of Trie.
	pub fn create<'db>(&self, db: &'db mut HashDB, root: &'db mut H256) -> Box<TrieMut + 'db> {
		match self.spec {
			TrieSpec::Generic => Box::new(TrieDBMut::new(db, root)),
			TrieSpec::Secure => Box::new(SecTrieDBMut::new(db, root)),
			TrieSpec::Fat => Box::new(FatDBMut::new(db, root)),
		}
	}

	/// Create new mutable instance of trie and check for errors.
	pub fn from_existing<'db>(&self, db: &'db mut HashDB, root: &'db mut H256) -> Result<Box<TrieMut + 'db>> {
		match self.spec {
			TrieSpec::Generic => Ok(Box::new(TrieDBMut::from_existing(db, root)?)),
			TrieSpec::Secure => Ok(Box::new(SecTrieDBMut::from_existing(db, root)?)),
			TrieSpec::Fat => Ok(Box::new(FatDBMut::from_existing(db, root)?)),
		}
	}

	/// Returns true iff the trie DB is a fat DB (allows enumeration of keys).
	pub fn is_fat(&self) -> bool { self.spec == TrieSpec::Fat }
}
