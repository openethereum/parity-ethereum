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
use hash::{FixedHash, H256};
use hashdb::HashDB;
use Bytes;

use std::collections::HashMap;
use std::sync::Arc;

/// Implementation of the `HashDB` trait for a disk-backed database with a memory overlay.
///
/// Alterations to the database may be made through `insert`, `delete` and associated
/// `emplace` function.
///
/// Keys may be `insert()`ed and `remove()`d. Persistent checkpoints may be made with
/// `commit()`. At the point on `commit`, the total historical insertions (when
/// after reduction by corresponding deletes) must sum to either ONE or ZERO. Anything
/// else will return an `Error`.
///
/// `revert()` (or the cessation of the program) will anull any alterations (using
/// `insert()` and `delete()`) from the historical record up until the `commit`
/// immediately prior.
///
/// The inspection functions `get()`, `contains()` and `keys()` maintain normal behaviour
/// - all `insert()` and `remove()` queries have an immediate effect and `commit` has no
/// effect on their behaviour. The keys considered in the database at any time are all
/// of those with a NET POSITIVE number of insertions in the historical record after all
/// deletions have been accounted for.
///
/// An auxilliary datum is also available for each key (functions with the `_aux` suffix)
/// which may be used to store additional non-persistent data. Auxilliary data is entirely
/// independent from the main database.
#[derive(Clone)]
pub struct OverlayDB {
	overlay: MemoryDB,
	backing: Arc<Database>,
	column: Option<u32>,
}

impl OverlayDB {
	/// Create a new instance of OverlayDB given a `backing` database and deletion mode.
	pub fn new(backing: Arc<Database>, col: Option<u32>) -> Self {
		OverlayDB {
			overlay: MemoryDB::new(),
			backing: backing,
			column: col,
		}
	}

	/// Create a new instance of OverlayDB with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> OverlayDB {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(Arc::new(Database::open_default(dir.to_str().unwrap()).unwrap()), None)
	}

	/// Commit all operations in a single batch.
	///
	/// As mentioned in the struct documentation, the total historical insertions and removals of
	/// any given key in the database must amount to either ZERO or ONE at the time that `commit` is called.
	/// Anything else will lead to an error.
	pub fn commit(&mut self) -> Result<u32, UtilError> {
		let batch = self.backing.transaction();
		let res = try!(self.commit_to_batch(&batch));
		self.backing.write(batch).map(|_| res).map_err(|e| e.into())
	}

	/// Commit all operations to given batch. Returns the number of insertions
	/// and deletions.
	fn commit_to_batch(&mut self, batch: &DBTransaction) -> Result<u32, UtilError> {
		let mut ret = 0u32;
		let mut deletes = 0usize;
		for (ref key, &(ref value, ref rc)) in self.overlay.peek().iter() {
			match *rc {
				0 => continue,
				1 => {
					if try!(self.backing.get(self.column, key)).is_none() {
						try!(batch.put(self.column, key, value))
					} else {
						// error to insert something more than once.
						return Err(BaseDataError::InsertionInvalid(*key.clone()).into());
					}
				}
				-1 => {
					deletes += 1;
					if try!(self.backing.get(self.column, key)).is_none() {
						return Err(BaseDataError::DeletionInvalid(*key.clone()).into());
					}

					try!(batch.delete(self.column, key));
				}
				rc => return Err(BaseDataError::InvalidReferenceCount(*key.clone(), rc).into()),
			}

			ret += 1;
		}

		trace!(target: "overlaydb", "OverlayDB::commit() deleted {} nodes", deletes);
		self.overlay.drain();
		Ok(ret)
	}

	/// Get the value of the given key in the backing database, or `None` if it's not there.
	fn payload(&self, key: &H256) -> Option<Bytes> {
		// TODO [rob] have this and HashDB functions all return Results.
		self.backing.get(self.column, key).expect("Low level database error.").map(|v| v.to_vec())
	}

	/// Revert all operations on this object (i.e. `insert()`s and `remove()`s) since the
	/// last `commit()`.
	pub fn revert(&mut self) { self.overlay.clear(); }

}

impl HashDB for OverlayDB {
	fn keys(&self) -> HashMap<H256, i32> {
		let mut ret = HashMap::new();

		for (key, _) in self.backing.iter(self.column) {
			ret.insert(H256::from_slice(&*key), 1);
		}

		for (key, refs) in self.overlay.keys() {
			*ret.entry(key).or_insert(0) += refs;
		}

		ret.into_iter().filter(|&(_, rc)| rc > 0).collect()
	}

	fn get(&self, key: &H256) -> Option<&[u8]> {
		match self.overlay.raw(key) {
			// +ve + {0,1} = {1..} : must be in!
			Some(&(ref d, rc)) if rc > 0 => Some(d),
			// -ve + {0,1} != {..0} : must be out!
			Some(&(_, rc)) if rc < 0 => None,
			// no refs - wholly dependent on the backing DB.
			_ => if let Some(value) = self.payload(key) {
				Some(&self.overlay.denote(key, value).0)
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

#[cfg(test)]
mod tests {
	use kvdb::Database;
	use hashdb::HashDB;
	use super::OverlayDB;
	use devtools::RandomTempPath;
	use sha3::Hashable;

	#[test]
	fn should_give_empty_keys_when_all_deleted() {
		let path = RandomTempPath::create_dir();
		let backing = Database::open_default(path.as_str()).unwrap();
		let mut db = OverlayDB::new(backing);

		let hash = db.insert(b"dog");
		db.commit().unwrap();

		db.remove(&hash);
		assert!(db.keys().is_empty());

		db.commit().unwrap();
		assert!(db.keys().is_empty());
	}

	#[test]
	fn should_not_return_negative_reffed_keys() {
		let path = RandomTempPath::create_dir();
		let backing = Database::open_default(path.as_str()).unwrap();
		let mut db = OverlayDB::new(backing);

		let hash = db.insert(b"dog");
		db.commit().unwrap();

		db.remove(&hash);
		assert!(db.keys().is_empty());
		assert!(!db.contains(&hash));
		assert!(db.get(&hash).is_none());

		db.remove(&hash);
		assert!(db.keys().is_empty());
		assert!(!db.contains(&hash));
		assert!(db.get(&hash).is_none());

		assert!(db.commit().is_err());

		db.insert(b"dog");
		assert!(db.keys().is_empty());
		assert!(!db.contains(&hash));
		assert!(db.get(&hash).is_none());

		db.commit().unwrap();
		assert!(db.keys().is_empty());
		assert!(!db.contains(&hash));
		assert!(db.get(&hash).is_none());
	}

	#[test]
	fn delete_remove() {
		let path = RandomTempPath::create_dir();
		let backing = Database::open_default(path.as_str()).unwrap();
		let mut db = OverlayDB::new(backing);

		let hash = db.insert(b"dog");
		db.commit().unwrap();

		db.remove(&hash);
		db.commit().unwrap();

		assert!(db.get(&hash).is_none())
	}

	#[test]
	#[should_panic]
	fn double_remove() {
		let path = RandomTempPath::create_dir();
		let backing = Database::open_default(path.as_str()).unwrap();
		let mut db = OverlayDB::new(backing);

		let hash = db.insert(b"cat");
		assert!(db.commit().is_ok());

		db.remove(&hash);
		db.remove(&hash);

		db.commit().unwrap();
	}

	#[test]
	#[should_panic]
	fn deletion_invalid() {
		let path = RandomTempPath::create_dir();
		let backing = Database::open_default(path.as_str()).unwrap();
		let mut db = OverlayDB::new(backing);

		let hash = b"hello".sha3();
		db.remove(&hash);
		db.commit().unwrap();
	}

	#[test]
	#[should_panic]
	fn insertion_invalid() {
		let path = RandomTempPath::create_dir();
		let backing = Database::open_default(path.as_str()).unwrap();
		let mut db = OverlayDB::new(backing);

		db.insert(b"bad juju");
		assert!(db.commit().is_ok());

		db.insert(b"bad juju");
		db.commit().unwrap();
	}
}