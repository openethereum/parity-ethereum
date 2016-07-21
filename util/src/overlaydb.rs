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

//! Disk-backed, `HashDB` implementation.

use error::{BaseDataError, UtilError};
use kvdb::{Database, DBTransaction};
use memorydb::MemoryDB;
use hash::{FixedHash, H32, H256};
use hashdb::HashDB;
use Bytes;

use std::collections::HashMap;
use std::sync::Arc;

/// How to treat entries which can be deleted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletionMode {
	/// Ignore them.
	Ignore,
	/// Delete them.
	Delete,
}

/// Implementation of the `HashDB` trait for a disk-backed database with a memory overlay.
///
/// The operations `insert()` and `remove()` take place on the memory overlay; batches of
/// such operations may be flushed to the disk-backed DB with `commit()` or discarded with
/// `revert()`.
///
/// `lookup()` and `contains()` maintain normal behaviour - all `insert()` and `remove()`
/// queries have an immediate effect in terms of these functions.
#[derive(Clone)]
pub struct OverlayDB {
	overlay: MemoryDB,
	backing: Arc<Database>,
	mode: DeletionMode,
}

impl OverlayDB {
	/// Create a new instance of OverlayDB given a `backing` database and deletion mode.
	pub fn new(backing: Database, mode: DeletionMode) -> Self {
		OverlayDB::new_with_arc(Arc::new(backing), mode)
	}

	/// Create a new instance of OverlayDB given a shared `backing` database.
	pub fn new_with_arc(backing: Arc<Database>, mode: DeletionMode) -> Self {
		OverlayDB {
			overlay: MemoryDB::new(),
			backing: backing,
			mode: mode
		}
	}

	/// Create a new instance of OverlayDB with an anonymous temporary database.
	pub fn new_temp(mode: DeletionMode) -> Self {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(Database::open_default(dir.to_str().unwrap()).unwrap(), mode)
	}

	/// Commit all operations to given batch. Returns the number of insertions
	/// and deletions.
	pub fn commit_to_batch(&mut self, batch: &DBTransaction) -> Result<u32, UtilError> {
		let mut ret = 0u32;
		let mut deletes = 0usize;
		for (key, (value, rc)) in self.overlay.drain() {
			match rc {
				0 => continue,
				1 => try!(batch.put(&key, &value)),
				-1 => {
					deletes += 1;
					if self.mode == DeletionMode::Delete {
						try!(batch.delete(&key));
					}
				}
				rc => return Err(BaseDataError::InvalidReferenceCount(key, rc).into()),
			}

			ret += 1;
		}

		trace!(target: "overlaydb", "OverlayDB::commit() deleted {} nodes", deletes);
		Ok(ret)
	}

	/// Commit all operations to the backing database. Returns the number of insertions and deletions.
	/// This does not commit auxiliary data.
	pub fn commit(&mut self) -> Result<u32, UtilError> {
		let batch = DBTransaction::new();
		let ops = try!(self.commit_to_batch(&batch));
		try!(self.backing.write(batch));

		Ok(ops)
	}

	/// Revert all operations on this object since last commit.
	pub fn revert(&mut self) { self.overlay.clear() }

	/// Drain auxiliary entries from the overlay.
	pub fn drain_aux(&mut self) -> HashMap<Bytes, Bytes> {
		self.overlay.drain_aux()
	}

	/// Returns the size of allocated heap memory.
	pub fn mem_used(&self) -> usize {
		self.overlay.mem_used()
	}

	/// Get the value of the given key.
	fn payload(&self, key: &H256) -> Option<Bytes> {
		// TODO [rob] have this and HashDB functions all return Results.
		self.backing.get(key).expect("Low level database error.").map(|v| v.to_vec())
	}
}

impl HashDB for OverlayDB {
	fn keys(&self) -> HashMap<H256, i32> {
		let mut ret = HashMap::new();

		for (key, _) in self.backing.iter() {
			ret.insert(H256::from_slice(&*key), 1);
		}

		for (key, refs) in self.overlay.keys() {
			*ret.entry(key).or_insert(0) += refs;
		}

		ret
	}

	fn get(&self, key: &H256) -> Option<&[u8]> {
		match self.overlay.raw(key) {
			Some(&(ref d, rc)) if rc > 0 => { Some(d) }
			_ => if let Some(x) = self.payload(key) {
				Some(&self.overlay.denote(key, x).0)
			} else {
				None
			},
		}
	}

	fn contains(&self, key: &H256) -> bool {
		self.get(key).is_some()
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		self.overlay.insert(value)
	}

	fn emplace(&mut self, key: H256, value: Bytes) {
		self.overlay.emplace(key, value);
	}

	fn remove(&mut self, key: &H256) {
		self.overlay.remove(key);
	}

	fn insert_aux(&mut self, hash: Bytes, value: Bytes) {
		self.overlay.insert_aux(hash, value);
	}

	fn get_aux(&self, hash: &[u8]) -> Option<Bytes> {
		self.overlay.get_aux(hash)
	}

	fn remove_aux(&mut self, hash: &[u8]) {
		self.overlay.remove_aux(hash);
	}
}