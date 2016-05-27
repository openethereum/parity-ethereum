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

//! Disk-backed, ref-counted `JournalDB` implementation.

use common::*;
use rlp::*;
use hashdb::*;
use overlaydb::*;
use super::traits::JournalDB;
use kvdb::{Database, DBTransaction, DatabaseConfig};
#[cfg(test)]
use std::env;

/// Implementation of the `HashDB` trait for a disk-backed database with a memory overlay
/// and latent-removal semantics.
///
/// Like `OverlayDB`, there is a memory overlay; `commit()` must be called in order to
/// write operations out to disk. Unlike `OverlayDB`, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
pub struct RefCountedDB {
	forward: OverlayDB,
	backing: Arc<Database>,
	latest_era: Option<u64>,
	inserts: Vec<H256>,
	removes: Vec<H256>,
}

const LATEST_ERA_KEY : [u8; 12] = [ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];
const VERSION_KEY : [u8; 12] = [ b'j', b'v', b'e', b'r', 0, 0, 0, 0, 0, 0, 0, 0 ];
const DB_VERSION : u32 = 0x200;
const PADDING : [u8; 10] = [ 0u8; 10 ];

impl RefCountedDB {
	/// Create a new instance given a `backing` database.
	pub fn new(path: &str) -> RefCountedDB {
		let opts = DatabaseConfig {
			//use 12 bytes as prefix, this must match account_db prefix
			prefix_size: Some(12),
			max_open_files: 256,
		};
		let backing = Database::open(&opts, path).unwrap_or_else(|e| {
			panic!("Error opening state db: {}", e);
		});
		if !backing.is_empty() {
			match backing.get(&VERSION_KEY).map(|d| d.map(|v| decode::<u32>(&v))) {
				Ok(Some(DB_VERSION)) => {},
				v => panic!("Incompatible DB version, expected {}, got {:?}; to resolve, remove {} and restart.", DB_VERSION, v, path)
			}
		} else {
			backing.put(&VERSION_KEY, &encode(&DB_VERSION)).expect("Error writing version to database");
		}

		let backing = Arc::new(backing);
		let latest_era = backing.get(&LATEST_ERA_KEY).expect("Low-level database error.").map(|val| decode::<u64>(&val));

		RefCountedDB {
			forward: OverlayDB::new_with_arc(backing.clone()),
			backing: backing,
			inserts: vec![],
			removes: vec![],
			latest_era: latest_era,
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	fn new_temp() -> RefCountedDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(dir.to_str().unwrap())
	}
}

impl HashDB for RefCountedDB {
	fn keys(&self) -> HashMap<H256, i32> { self.forward.keys() }
	fn lookup(&self, key: &H256) -> Option<&[u8]> { self.forward.lookup(key) }
	fn exists(&self, key: &H256) -> bool { self.forward.exists(key) }
	fn insert(&mut self, value: &[u8]) -> H256 { let r = self.forward.insert(value); self.inserts.push(r.clone()); r }
	fn emplace(&mut self, key: H256, value: Bytes) { self.inserts.push(key.clone()); self.forward.emplace(key, value); }
	fn kill(&mut self, key: &H256) { self.removes.push(key.clone()); }
}

impl JournalDB for RefCountedDB {
	fn boxed_clone(&self) -> Box<JournalDB> {
		Box::new(RefCountedDB {
			forward: self.forward.clone(),
			backing: self.backing.clone(),
			latest_era: self.latest_era,
			inserts: self.inserts.clone(),
			removes: self.removes.clone(),
		})
	}

	fn mem_used(&self) -> usize {
		self.inserts.heap_size_of_children() + self.removes.heap_size_of_children()
 	}

	fn is_empty(&self) -> bool {
		self.latest_era.is_none()
	}

	fn latest_era(&self) -> Option<u64> { self.latest_era }

	fn commit(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		// journal format:
		// [era, 0] => [ id, [insert_0, ...], [remove_0, ...] ]
		// [era, 1] => [ id, [insert_0, ...], [remove_0, ...] ]
		// [era, n] => [ ... ]

		// TODO: store last_era, reclaim_period.

		// when we make a new commit, we journal the inserts and removes.
		// for each end_era that we journaled that we are no passing by,
		// we remove all of its removes assuming it is canonical and all
		// of its inserts otherwise.

		// record new commit's details.
		let batch = DBTransaction::new();
		{
			let mut index = 0usize;
			let mut last;

			while try!(self.backing.get({
				let mut r = RlpStream::new_list(3);
				r.append(&now);
				r.append(&index);
				r.append(&&PADDING[..]);
				last = r.drain();
				&last
			})).is_some() {
				index += 1;
			}

			let mut r = RlpStream::new_list(3);
			r.append(id);
			r.append(&self.inserts);
			r.append(&self.removes);
			try!(batch.put(&last, r.as_raw()));

			trace!(target: "rcdb", "new journal for time #{}.{} => {}: inserts={:?}, removes={:?}", now, index, id, self.inserts, self.removes);

			self.inserts.clear();
			self.removes.clear();

			if self.latest_era.map_or(true, |e| now > e) {
				try!(batch.put(&LATEST_ERA_KEY, &encode(&now)));
				self.latest_era = Some(now);
			}
		}

		// apply old commits' details
		if let Some((end_era, canon_id)) = end {
			let mut index = 0usize;
			let mut last;
			while let Some(rlp_data) = {
//				trace!(target: "rcdb", "checking for journal #{}.{}", end_era, index);
				try!(self.backing.get({
					let mut r = RlpStream::new_list(3);
					r.append(&end_era);
					r.append(&index);
					r.append(&&PADDING[..]);
					last = r.drain();
					&last
				}))
			} {
				let rlp = Rlp::new(&rlp_data);
				let our_id: H256 = rlp.val_at(0);
				let to_remove: Vec<H256> = rlp.val_at(if canon_id == our_id {2} else {1});
				trace!(target: "rcdb", "delete journal for time #{}.{}=>{}, (canon was {}): deleting {:?}", end_era, index, our_id, canon_id, to_remove);
				for i in &to_remove {
					self.forward.remove(i);
				}
				try!(batch.delete(&last));
				index += 1;
			}
		}

		let r = try!(self.forward.commit_to_batch(&batch));
		try!(self.backing.write(batch));
		Ok(r)
	}
}

#[cfg(test)]
mod tests {
	#![cfg_attr(feature="dev", allow(blacklisted_name))]
	#![cfg_attr(feature="dev", allow(similar_names))]

	use common::*;
	use super::*;
	use super::super::traits::JournalDB;
	use hashdb::*;

	#[test]
	fn long_history() {
		// history is 3
		let mut jdb = RefCountedDB::new_temp();
		let h = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.exists(&h));
		jdb.remove(&h);
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		assert!(jdb.exists(&h));
		jdb.commit(2, &b"2".sha3(), None).unwrap();
		assert!(jdb.exists(&h));
		jdb.commit(3, &b"3".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.exists(&h));
		jdb.commit(4, &b"4".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(!jdb.exists(&h));
	}

	#[test]
	fn latest_era_should_work() {
		// history is 3
		let mut jdb = RefCountedDB::new_temp();
		assert_eq!(jdb.latest_era(), None);
		let h = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert_eq!(jdb.latest_era(), Some(0));
		jdb.remove(&h);
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		assert_eq!(jdb.latest_era(), Some(1));
		jdb.commit(2, &b"2".sha3(), None).unwrap();
		assert_eq!(jdb.latest_era(), Some(2));
		jdb.commit(3, &b"3".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert_eq!(jdb.latest_era(), Some(3));
		jdb.commit(4, &b"4".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert_eq!(jdb.latest_era(), Some(4));
	}

	#[test]
	fn complex() {
		// history is 1
		let mut jdb = RefCountedDB::new_temp();

		let foo = jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));

		jdb.remove(&foo);
		jdb.remove(&bar);
		let baz = jdb.insert(b"baz");
		jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));
		assert!(jdb.exists(&baz));

		let foo = jdb.insert(b"foo");
		jdb.remove(&baz);
		jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.exists(&foo));
		assert!(!jdb.exists(&bar));
		assert!(jdb.exists(&baz));

		jdb.remove(&foo);
		jdb.commit(3, &b"3".sha3(), Some((2, b"2".sha3()))).unwrap();
		assert!(jdb.exists(&foo));
		assert!(!jdb.exists(&bar));
		assert!(!jdb.exists(&baz));

		jdb.commit(4, &b"4".sha3(), Some((3, b"3".sha3()))).unwrap();
		assert!(!jdb.exists(&foo));
		assert!(!jdb.exists(&bar));
		assert!(!jdb.exists(&baz));
	}

	#[test]
	fn fork() {
		// history is 1
		let mut jdb = RefCountedDB::new_temp();

		let foo = jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));

		jdb.remove(&foo);
		let baz = jdb.insert(b"baz");
		jdb.commit(1, &b"1a".sha3(), Some((0, b"0".sha3()))).unwrap();

		jdb.remove(&bar);
		jdb.commit(1, &b"1b".sha3(), Some((0, b"0".sha3()))).unwrap();

		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));
		assert!(jdb.exists(&baz));

		jdb.commit(2, &b"2b".sha3(), Some((1, b"1b".sha3()))).unwrap();
		assert!(jdb.exists(&foo));
		assert!(!jdb.exists(&baz));
		assert!(!jdb.exists(&bar));
	}
}
