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

//! Trie interface and implementation.

use std::fmt;
use hash::H256;
use hashdb::{HashDB, DBValue};

/// Export the standardmap module.
pub mod standardmap;
/// Export the journal module.
pub mod journal;
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
		self.get_recorded(key, &mut recorder::NoOp)
	}

	/// Query the value of the given key in this trie while recording visited nodes
	/// to the given recorder. If the query fails, the nodes passed to the recorder are unspecified.
	fn get_recorded<'a, 'b, R: 'b>(&'a self, key: &'b [u8], rec: &'b mut R) -> Result<Option<DBValue>>
		where 'a: 'b, R: Recorder;

	/// Returns an iterator over elements of trie.
	fn iter<'a>(&'a self) -> Result<Box<Iterator<Item = TrieItem> + 'a>>;
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

	/// Insert a `key`/`value` pair into the trie. An `empty` value is equivalent to removing
	/// `key` from the trie.
	fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<()>;

	/// Remove a `key` from the trie. Equivalent to making it equal to the empty
	/// value.
	fn remove(&mut self, key: &[u8]) -> Result<()>;
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

	fn get_recorded<'a, 'b, R: 'b>(&'a self, key: &'b [u8], r: &'b mut R) -> Result<Option<DBValue>>
		where 'a: 'b, R: Recorder {
		wrapper!(self, get_recorded, key, r)
	}

	fn iter<'a>(&'a self) -> Result<Box<Iterator<Item = TrieItem> + 'a>> {
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
			TrieSpec::Generic => Ok(TrieKinds::Generic(try!(TrieDB::new(db, root)))),
			TrieSpec::Secure => Ok(TrieKinds::Secure(try!(SecTrieDB::new(db, root)))),
			TrieSpec::Fat => Ok(TrieKinds::Fat(try!(FatDB::new(db, root)))),
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
			TrieSpec::Generic => Ok(Box::new(try!(TrieDBMut::from_existing(db, root)))),
			TrieSpec::Secure => Ok(Box::new(try!(SecTrieDBMut::from_existing(db, root)))),
			TrieSpec::Fat => Ok(Box::new(try!(FatDBMut::from_existing(db, root)))),
		}
	}

	/// Returns true iff the trie DB is a fat DB (allows enumeration of keys).
	pub fn is_fat(&self) -> bool { self.spec == TrieSpec::Fat } 
}
