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

//! Disk-backed `HashDB` implementation.

use common::*;
use hashdb::*;
use kvdb::{Database, DBTransaction};

/// A `HashDB` which can manage a short-term journal potentially containing many forks of mutually
/// exclusive actions.
pub trait JournalDB: HashDB {
	/// Return a copy of ourself, in a box.
	fn boxed_clone(&self) -> Box<JournalDB>;

	/// Returns heap memory size used
	fn mem_used(&self) -> usize;

	/// Check if this database has any commits
	fn is_empty(&self) -> bool;

	/// Get the latest era in the DB. None if there isn't yet any data in there.
	fn latest_era(&self) -> Option<u64>;

	/// Get the earliest era in the DB. None if there isn't yet any data in there.
	fn earliest_era(&self) -> Option<u64> { None }

	/// Commit all recent insert operations and canonical historical commits' removals from the
	/// old era to the backing database, reverting any non-canonical historical commit's inserts.
	fn commit(&mut self, batch: &DBTransaction, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError>;

	/// Commit canonical historical commits' removals from the
	/// old era to the backing database, reverting any non-canonical historical commit's inserts.
	fn commit_old(&mut self, _batch: &DBTransaction, _end_era: u64, _end_id: &H256) -> Result<(), UtilError> {
		Ok(())
	}

	/// Commit all queued insert and delete operations without affecting any journalling -- this requires that all insertions
	/// and deletions are indeed canonical and will likely lead to an invalid database if that assumption is violated.
	///
	/// Any keys or values inserted or deleted must be completely independent of those affected
	/// by any previous `commit` operations. Essentially, this means that `inject` can be used
	/// either to restore a state to a fresh database, or to insert data which may only be journalled
	/// from this point onwards.
	fn inject(&mut self, batch: &DBTransaction) -> Result<u32, UtilError>;

	/// State data query
	fn state(&self, _id: &H256) -> Option<Bytes>;

	/// Whether this database is pruned.
	fn is_pruned(&self) -> bool { true }

	/// Get backing database.
	fn backing(&self) -> &Arc<Database>;

	/// Clear internal strucutres. This should called after changes have been written
	/// to the backing strage
	fn flush(&self) {}

	/// Commit all changes in a single batch
	#[cfg(test)]
	fn commit_batch(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		let batch = self.backing().transaction();
		let res = try!(self.commit(&batch, now, id, end));
		let result = self.backing().write(batch).map(|_| res).map_err(Into::into);
		self.flush();
		result
	}

	/// Inject all changes in a single batch.
	#[cfg(test)]
	fn inject_batch(&mut self) -> Result<u32, UtilError> {
		let batch = self.backing().transaction();
		let res = try!(self.inject(&batch));
		self.backing().write(batch).map(|_| res).map_err(Into::into)
	}
}
