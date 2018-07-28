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

//! Traits for accessing contract storage.

use ethereum_types::H256;
use ethtrie;
use std::fmt;

/// Errors from accessing storage.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
	/// Trie lookup error.
	TrieError {
		/// Underlying error.
		error: Box<ethtrie::TrieError>,
	},
}

impl fmt::Display for StorageError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::StorageError::*;

		match *self {
			TrieError { ref error } => write!(f, "trie error: {}", error),
		}
	}
}

impl From<Box<ethtrie::TrieError>> for StorageError {
	fn from(error: Box<ethtrie::TrieError>) -> Self {
		StorageError::TrieError { error }
	}
}

/// Storage access for the current VM.
pub trait StorageAccess {
	/// Access storage at the given location.
	fn storage_at(&self, key: &H256) -> Result<H256, StorageError>;
}
