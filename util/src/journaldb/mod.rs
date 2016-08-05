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
use kvdb::Database;

/// Export the journaldb module.
pub mod traits;
mod archivedb;
mod earlymergedb;
mod overlayrecentdb;
mod refcounteddb;

/// Export the `JournalDB` trait.
pub use self::traits::JournalDB;

/// A journal database algorithm.
#[derive(Debug, PartialEq, Clone, Copy)]
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
	fn default() -> Algorithm { Algorithm::OverlayRecent }
}

impl FromStr for Algorithm {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"archive" => Ok(Algorithm::Archive),
			"light" => Ok(Algorithm::EarlyMerge),
			"fast" => Ok(Algorithm::OverlayRecent),
			"basic" => Ok(Algorithm::RefCounted),
			e => Err(format!("Invalid algorithm: {}", e)),
		}
	}
}

impl Algorithm {
	/// Returns static str describing journal database algorithm.
	pub fn as_str(&self) -> &'static str {
		match *self {
			Algorithm::Archive => "archive",
			Algorithm::EarlyMerge => "light",
			Algorithm::OverlayRecent => "fast",
			Algorithm::RefCounted => "basic",
		}
	}

	/// Returns static str describing journal database algorithm.
	pub fn as_internal_name_str(&self) -> &'static str {
		match *self {
			Algorithm::Archive => "archive",
			Algorithm::EarlyMerge => "earlymerge",
			Algorithm::OverlayRecent => "overlayrecent",
			Algorithm::RefCounted => "refcounted",
		}
	}

	/// Returns true if pruning strategy is stable
	pub fn is_stable(&self) -> bool {
		match *self {
			Algorithm::Archive | Algorithm::OverlayRecent => true,
			_ => false,
		}
	}

	/// Returns all algorithm types.
	pub fn all_types() -> Vec<Algorithm> {
		vec![Algorithm::Archive, Algorithm::EarlyMerge, Algorithm::OverlayRecent, Algorithm::RefCounted]
	}
}

impl fmt::Display for Algorithm {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

/// Create a new `JournalDB` trait object.
pub fn new(backing: Arc<Database>, algorithm: Algorithm, col: Option<u32>) -> Box<JournalDB> {
	match algorithm {
		Algorithm::Archive => Box::new(archivedb::ArchiveDB::new(backing, col)),
		Algorithm::EarlyMerge => Box::new(earlymergedb::EarlyMergeDB::new(backing, col)),
		Algorithm::OverlayRecent => Box::new(overlayrecentdb::OverlayRecentDB::new(backing, col)),
		Algorithm::RefCounted => Box::new(refcounteddb::RefCountedDB::new(backing, col)),
	}
}

// all keys must be at least 12 bytes
const DB_PREFIX_LEN : usize = 12;
const LATEST_ERA_KEY : [u8; DB_PREFIX_LEN] = [ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];

#[cfg(test)]
mod tests {
	use super::Algorithm;

	#[test]
	fn test_journal_algorithm_parsing() {
		assert_eq!(Algorithm::Archive, "archive".parse().unwrap());
		assert_eq!(Algorithm::EarlyMerge, "light".parse().unwrap());
		assert_eq!(Algorithm::OverlayRecent, "fast".parse().unwrap());
		assert_eq!(Algorithm::RefCounted, "basic".parse().unwrap());
	}

	#[test]
	fn test_journal_algorithm_printing() {
		assert_eq!(Algorithm::Archive.to_string(), "archive".to_owned());
		assert_eq!(Algorithm::EarlyMerge.to_string(), "light".to_owned());
		assert_eq!(Algorithm::OverlayRecent.to_string(), "fast".to_owned());
		assert_eq!(Algorithm::RefCounted.to_string(), "basic".to_owned());
	}

	#[test]
	fn test_journal_algorithm_is_stable() {
		assert!(Algorithm::Archive.is_stable());
		assert!(Algorithm::OverlayRecent.is_stable());
		assert!(!Algorithm::EarlyMerge.is_stable());
		assert!(!Algorithm::RefCounted.is_stable());
	}

	#[test]
	fn test_journal_algorithm_default() {
		assert_eq!(Algorithm::default(), Algorithm::OverlayRecent);
	}

	#[test]
	fn test_journal_algorithm_all_types() {
		// compiling should fail if some cases are not covered
		let mut archive = 0;
		let mut earlymerge = 0;
		let mut overlayrecent = 0;
		let mut refcounted = 0;

		for a in &Algorithm::all_types() {
			match *a {
				Algorithm::Archive => archive += 1,
				Algorithm::EarlyMerge => earlymerge += 1,
				Algorithm::OverlayRecent => overlayrecent += 1,
				Algorithm::RefCounted => refcounted += 1,
			}
		}

		assert_eq!(archive, 1);
		assert_eq!(earlymerge, 1);
		assert_eq!(overlayrecent, 1);
		assert_eq!(refcounted, 1);
	}
}