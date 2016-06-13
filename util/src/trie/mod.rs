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

pub use self::trietraits::{Trie, TrieMut};
pub use self::standardmap::{Alphabet, StandardMap, ValueMode};
pub use self::triedbmut::TrieDBMut;
pub use self::triedb::TrieDB;
pub use self::sectriedbmut::SecTrieDBMut;
pub use self::sectriedb::SecTrieDB;

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