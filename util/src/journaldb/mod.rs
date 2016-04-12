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

//! `JournalDB` interface and implementation.

use common::*;

/// Export the journaldb module.
pub mod traits;
mod archivedb;
mod earlymergedb;
mod overlayrecentdb;
mod refcounteddb;

/// Export the `JournalDB` trait.
pub use self::traits::JournalDB;

/// A journal database algorithm.
#[derive(Debug, Clone, Copy)]
pub enum Algorithm {
	/// Keep all keys forever.
	Archive,

	/// Ancient and recent history maintained separately; recent history lasts for particular
	/// number of blocks.
	///
	/// Inserts go into backing database, journal retains knowledge of whether backing DB key is
	/// ancient or recent. Non-canon inserts get explicitly reverted and removed from backing DB.
	EarlyMerge,

	/// Ancient and recent history maintained separately; recent history lasts for particular
	/// number of blocks.
	///
	/// Inserts go into memory overlay, which is tried for key fetches. Memory overlay gets
	/// flushed in backing only at end of recent history.
	OverlayRecent,

	/// Ancient and recent history maintained separately; recent history lasts for particular
	/// number of blocks.
	///
	/// References are counted in disk-backed DB.
	RefCounted,
}

impl Default for Algorithm {
	fn default() -> Algorithm { Algorithm::Archive }
}

impl fmt::Display for Algorithm {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match self {
			&Algorithm::Archive => "archive",
			&Algorithm::EarlyMerge => "earlymerge",
			&Algorithm::OverlayRecent => "overlayrecent",
			&Algorithm::RefCounted => "refcounted",
		})
	}
}

/// Create a new `JournalDB` trait object.
pub fn new(path: &str, algorithm: Algorithm) -> Box<JournalDB> {
	match algorithm {
		Algorithm::Archive => Box::new(archivedb::ArchiveDB::new(path)),
		Algorithm::EarlyMerge => Box::new(earlymergedb::EarlyMergeDB::new(path)),
		Algorithm::OverlayRecent => Box::new(overlayrecentdb::OverlayRecentDB::new(path)),
		Algorithm::RefCounted => Box::new(refcounteddb::RefCountedDB::new(path)),
	}
}
