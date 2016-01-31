//! Disk-backed HashDB implementation.

use common::*;
use rlp::*;
use hashdb::*;
use overlaydb::*;
use rocksdb::{DB, Writable};
#[cfg(test)]
use std::env;

#[derive(Clone)]
/// Implementation of the HashDB trait for a disk-backed database with a memory overlay
/// and latent-removal semantics.
///
/// Like OverlayDB, there is a memory overlay; `commit()` must be called in order to 
/// write operations out to disk. Unlike OverlayDB, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
pub struct JournalDB {
	forward: OverlayDB,
	backing: Arc<DB>,
	inserts: Vec<H256>,
	removes: Vec<H256>
}

impl JournalDB {
	/// Create a new instance given a `backing` database.
	pub fn new(backing: DB) -> JournalDB {
		let db = Arc::new(backing);
		JournalDB {
			forward: OverlayDB::new_with_arc(db.clone()),
			backing: db,
			inserts: vec![],
			removes: vec![],
		}
	}

	/// Create a new instance given a shared `backing` database.
	pub fn new_with_arc(backing: Arc<DB>) -> JournalDB {
		JournalDB {
			forward: OverlayDB::new_with_arc(backing.clone()),
			backing: backing,
			inserts: vec![],
			removes: vec![],
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> JournalDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(DB::open_default(dir.to_str().unwrap()).unwrap())
	}

	/// Get a clone of the overlay db portion of this.
	pub fn to_overlaydb(&self) -> OverlayDB { self.forward.clone() }

	/// Commit all recent insert operations and historical removals from the old era
	/// to the backing database.
	pub fn commit(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
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
		{
			let mut index = 0usize;
			let mut last;

			while try!(self.backing.get({
				let mut r = RlpStream::new_list(2);
				r.append(&now);
				r.append(&index);
				last = r.drain();
				&last
			})).is_some() {
				index += 1;
			}

			let mut r = RlpStream::new_list(3);
			r.append(id);
			r.append(&self.inserts);
			r.append(&self.removes);
			try!(self.backing.put(&last, r.as_raw()));
			self.inserts.clear();
			self.removes.clear();
		}

		// apply old commits' details
		if let Some((end_era, canon_id)) = end {
			let mut index = 0usize;
			let mut last;
			while let Some(rlp_data) = try!(self.backing.get({
				let mut r = RlpStream::new_list(2);
				r.append(&end_era);
				r.append(&index);
				last = r.drain();
				&last
			})) {
				let rlp = Rlp::new(&rlp_data);
				let to_remove: Vec<H256> = rlp.val_at(if canon_id == rlp.val_at(0) {2} else {1});
				for i in &to_remove {
					self.forward.remove(i);
				}
				try!(self.backing.delete(&last));
				trace!("JournalDB: delete journal for time #{}.{}, (canon was {}): {} entries", end_era, index, canon_id, to_remove.len());
				index += 1;
			}
		}

		self.forward.commit()
	}

	/// Revert all operations on this object (i.e. `insert()`s and `removes()`s) since the
	/// last `commit()`.
	pub fn revert(&mut self) { self.forward.revert(); self.removes.clear(); }
}

impl HashDB for JournalDB {
	fn keys(&self) -> HashMap<H256, i32> { self.forward.keys() }
	fn lookup(&self, key: &H256) -> Option<&[u8]> { self.forward.lookup(key) }
	fn exists(&self, key: &H256) -> bool { self.forward.exists(key) }
	fn insert(&mut self, value: &[u8]) -> H256 { let r = self.forward.insert(value); self.inserts.push(r.clone()); r }
	fn emplace(&mut self, key: H256, value: Bytes) { self.inserts.push(key.clone()); self.forward.emplace(key, value); }
	fn kill(&mut self, key: &H256) { self.removes.push(key.clone()); }
}

#[cfg(test)]
mod tests {
	use common::*;
	use super::*;
	use hashdb::*;

	#[test]
	fn long_history() {
		// history is 3
		let mut jdb = JournalDB::new_temp();
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
	fn complex() {
		// history is 1
		let mut jdb = JournalDB::new_temp();

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
		let mut jdb = JournalDB::new_temp();

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
