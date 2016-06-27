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
use hashdb::HashDB;

/// Export the trietraits module.
pub mod trietraits;
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

mod fatdb;

mod fatdbmut;

pub use self::trietraits::{Trie, TrieMut};
pub use self::standardmap::{Alphabet, StandardMap, ValueMode};
pub use self::triedbmut::TrieDBMut;
pub use self::triedb::{TrieDB, TrieDBIterator};
pub use self::sectriedbmut::SecTrieDBMut;
pub use self::sectriedb::SecTrieDB;
pub use self::fatdb::{FatDB, FatDBIterator};
pub use self::fatdbmut::FatDBMut;

/// Trie Errors
#[derive(Debug)]
pub enum TrieError {
	/// Attempted to create a trie with a state root not in the DB.
	InvalidStateRoot,
}

impl fmt::Display for TrieError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Trie Error: Invalid state root.")
	}
}

/// Trie types
#[derive(Debug, Clone)]
pub enum TrieSpec {
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

impl TrieFactory {
	/// Creates new factory.
	pub fn new(spec: TrieSpec) -> Self {
		TrieFactory {
			spec: spec,
		}
	}

	/// Create new immutable instance of Trie.
	pub fn create<'db>(&self, db: &'db HashDB, root: &'db H256) -> Result<Box<Trie + 'db>, TrieError> {
		match self.spec {
			TrieSpec::Secure => Ok(Box::new(try!(SecTrieDB::new(db, root)))),
			TrieSpec::Fat => Ok(Box::new(try!(FatDB::new(db, root)))),
		}
	}

	/// Create new mutable instance of Trie.
	pub fn create_mut<'db>(&self, db: &'db mut HashDB, root: &'db mut H256) -> Result<Box<Trie + 'db>, TrieError> {
		match self.spec {
			TrieSpec::Secure => Ok(Box::new(SecTrieDBMut::new(db, root))),
			TrieSpec::Fat => Ok(Box::new(FatDBMut::new(db, root))),
		}
	}

	/// Create new mutable instance of trie and check for errors.
	pub fn from_existing<'db>(&self, db: &'db mut HashDB, root: &'db mut H256) -> Result<Box<Trie + 'db>, TrieError> {
		match self.spec {
			TrieSpec::Secure => Ok(Box::new(try!(SecTrieDBMut::from_existing(db, root)))),
			TrieSpec::Fat => Ok(Box::new(try!(FatDBMut::from_existing(db, root)))),
		}
	}
}
