// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! `JournalDB` interface and implementation.

use std::{
	fmt, str, io,
	sync::Arc,
	collections::HashMap,
};

use ethereum_types::H256;
use hash_db::HashDB;
use keccak_hasher::KeccakHasher;
use kvdb::{self, DBTransaction, DBValue};
use parity_bytes::Bytes;

mod archivedb;
mod earlymergedb;
mod overlayrecentdb;
mod refcounteddb;
mod util;
mod as_hash_db_impls;
mod overlaydb;

/// A `HashDB` which can manage a short-term journal potentially containing many forks of mutually
/// exclusive actions.
pub trait JournalDB: HashDB<KeccakHasher, DBValue> {
	/// Return a copy of ourself, in a box.
	fn boxed_clone(&self) -> Box<dyn JournalDB>;

	/// Returns heap memory size used
	fn mem_used(&self) -> usize;

	/// Returns the size of journalled state in memory.
	/// This function has a considerable speed requirement --
	/// it must be fast enough to call several times per block imported.
	fn journal_size(&self) -> usize { 0 }

	/// Check if this database has any commits
	fn is_empty(&self) -> bool;

	/// Get the earliest era in the DB. None if there isn't yet any data in there.
	fn earliest_era(&self) -> Option<u64> { None }

	/// Get the latest era in the DB. None if there isn't yet any data in there.
	fn latest_era(&self) -> Option<u64>;

	/// Journal recent database operations as being associated with a given era and id.
	// TODO: give the overlay to this function so journaldbs don't manage the overlays themselves.
	fn journal_under(&mut self, batch: &mut DBTransaction, now: u64, id: &H256) -> io::Result<u32>;

	/// Mark a given block as canonical, indicating that competing blocks' states may be pruned out.
	fn mark_canonical(&mut self, batch: &mut DBTransaction, era: u64, id: &H256) -> io::Result<u32>;

	/// Commit all queued insert and delete operations without affecting any journalling -- this requires that all insertions
	/// and deletions are indeed canonical and will likely lead to an invalid database if that assumption is violated.
	///
	/// Any keys or values inserted or deleted must be completely independent of those affected
	/// by any previous `commit` operations. Essentially, this means that `inject` can be used
	/// either to restore a state to a fresh database, or to insert data which may only be journalled
	/// from this point onwards.
	fn inject(&mut self, batch: &mut DBTransaction) -> io::Result<u32>;

	/// State data query
	fn state(&self, _id: &H256) -> Option<Bytes>;

	/// Whether this database is pruned.
	fn is_prunable(&self) -> bool { true }

	/// Get backing database.
	fn backing(&self) -> &Arc<dyn kvdb::KeyValueDB>;

	/// Clear internal strucutres. This should called after changes have been written
	/// to the backing strage
	fn flush(&self) {}

	/// Consolidate all the insertions and deletions in the given memory overlay.
	fn consolidate(&mut self, overlay: MemoryDB);

	/// Primarily use for tests, highly inefficient.
	fn keys(&self) -> HashMap<H256, i32>;
}

/// Alias to ethereum MemoryDB
type MemoryDB = memory_db::MemoryDB<
	keccak_hasher::KeccakHasher,
	memory_db::HashKey<keccak_hasher::KeccakHasher>,
	kvdb::DBValue,
>;

/// Journal database operating strategy.
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

impl str::FromStr for Algorithm {
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

/// Create a new `JournalDB` trait object over a generic key-value database.
pub fn new(backing: Arc<dyn (::kvdb::KeyValueDB)>, algorithm: Algorithm, col: Option<u32>) -> Box<dyn JournalDB> {
	match algorithm {
		Algorithm::Archive => Box::new(archivedb::ArchiveDB::new(backing, col)),
		Algorithm::EarlyMerge => Box::new(earlymergedb::EarlyMergeDB::new(backing, col)),
		Algorithm::OverlayRecent => Box::new(overlayrecentdb::OverlayRecentDB::new(backing, col)),
		Algorithm::RefCounted => Box::new(refcounteddb::RefCountedDB::new(backing, col)),
	}
}

// all keys must be at least 12 bytes
const DB_PREFIX_LEN : usize = ::kvdb::PREFIX_LEN;
const LATEST_ERA_KEY : [u8; ::kvdb::PREFIX_LEN] = [ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];

fn error_key_already_exists(hash: &ethereum_types::H256) -> io::Error {
	io::Error::new(io::ErrorKind::AlreadyExists, hash.to_string())
}

fn error_negatively_reference_hash(hash: &ethereum_types::H256) -> io::Error {
	io::Error::new(io::ErrorKind::Other, format!("Entry {} removed from database more times than it was added.", hash))
}

pub fn new_memory_db() -> MemoryDB {
	MemoryDB::from_null_node(&rlp::NULL_RLP, rlp::NULL_RLP.as_ref().into())
}

#[cfg(test)]
/// Inject all changes in a single batch.
pub fn inject_batch(jdb: &mut dyn JournalDB) -> io::Result<u32> {
	let mut batch = jdb.backing().transaction();
	let res = jdb.inject(&mut batch)?;
	jdb.backing().write(batch).map(|_| res).map_err(Into::into)
}

/// Commit all changes in a single batch
#[cfg(test)]
fn commit_batch(jdb: &mut dyn JournalDB, now: u64, id: &H256, end: Option<(u64, H256)>) -> io::Result<u32> {
	let mut batch = jdb.backing().transaction();
	let mut ops = jdb.journal_under(&mut batch, now, id)?;

	if let Some((end_era, canon_id)) = end {
		ops += jdb.mark_canonical(&mut batch, end_era, &canon_id)?;
	}

	let result = jdb.backing().write(batch).map(|_| ops).map_err(Into::into);
	jdb.flush();
	result
}

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
