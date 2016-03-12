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

//! Disk-backed HashDB implementation.

use common::*;
use rlp::*;
use hashdb::*;
use memorydb::*;
use super::traits::JournalDB;
use kvdb::{Database, DBTransaction, DatabaseConfig};
#[cfg(test)]
use std::env;

/// Implementation of the HashDB trait for a disk-backed database with a memory overlay
/// and latent-removal semantics.
///
/// Like OverlayDB, there is a memory overlay; `commit()` must be called in order to
/// write operations out to disk. Unlike OverlayDB, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
pub struct EarlyMergeDB {
	overlay: MemoryDB,
	backing: Arc<Database>,
	counters: Option<Arc<RwLock<HashMap<H256, i32>>>>,
}

// all keys must be at least 12 bytes
const LATEST_ERA_KEY : [u8; 12] = [ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];
const VERSION_KEY : [u8; 12] = [ b'j', b'v', b'e', b'r', 0, 0, 0, 0, 0, 0, 0, 0 ];
const DB_VERSION : u32 = 3;
const PADDING : [u8; 10] = [ 0u8; 10 ];

impl EarlyMergeDB {
	/// Create a new instance from file
	pub fn new(path: &str) -> EarlyMergeDB {
		let opts = DatabaseConfig {
			prefix_size: Some(12) //use 12 bytes as prefix, this must match account_db prefix
		};
		let backing = Database::open(&opts, path).unwrap_or_else(|e| {
			panic!("Error opening state db: {}", e);
		});
		if !backing.is_empty() {
			match backing.get(&VERSION_KEY).map(|d| d.map(|v| decode::<u32>(&v))) {
				Ok(Some(DB_VERSION)) => {},
				v => panic!("Incompatible DB version, expected {}, got {:?}", DB_VERSION, v)
			}
		} else {
			backing.put(&VERSION_KEY, &encode(&DB_VERSION)).expect("Error writing version to database");
		}

		let counters = Some(Arc::new(RwLock::new(EarlyMergeDB::read_counters(&backing))));
		EarlyMergeDB {
			overlay: MemoryDB::new(),
			backing: Arc::new(backing),
			counters: counters,
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	fn new_temp() -> EarlyMergeDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(dir.to_str().unwrap())
	}

	fn morph_key(key: &H256, index: u8) -> Bytes {
		let mut ret = key.bytes().to_owned();
		ret.push(index);
		ret
	}

	// The next three are valid only as long as there is an insert operation of `key` in the journal.
	fn set_already_in(batch: &DBTransaction, key: &H256) { batch.put(&Self::morph_key(key, 0), &[1u8]).expect("Low-level database error. Some issue with your hard disk?"); }
	fn reset_already_in(batch: &DBTransaction, key: &H256) { batch.delete(&Self::morph_key(key, 0)).expect("Low-level database error. Some issue with your hard disk?"); }
	fn is_already_in(backing: &Database, key: &H256) -> bool {
		backing.get(&Self::morph_key(key, 0)).expect("Low-level database error. Some issue with your hard disk?").is_some()
	}

	fn insert_keys(inserts: &[(H256, Bytes)], backing: &Database, counters: &mut HashMap<H256, i32>, batch: &DBTransaction) {
		for &(ref h, ref d) in inserts {
			if let Some(c) = counters.get_mut(h) {
				// already counting. increment.
				*c += 1;
				continue;
			}

			// this is the first entry for this node in the journal.
			if backing.get(&h.bytes()).expect("Low-level database error. Some issue with your hard disk?").is_some() {
				// already in the backing DB. start counting, and remember it was already in.
				Self::set_already_in(batch, &h);
				counters.insert(h.clone(), 1);
				continue;
			}

			// Gets removed when a key leaves the journal, so should never be set when we're placing a new key.
			//Self::reset_already_in(&h);
			assert!(!Self::is_already_in(backing, &h));
			batch.put(&h.bytes(), d).expect("Low-level database error. Some issue with your hard disk?");
		}
	}

	fn replay_keys(inserts: &[H256], backing: &Database, counters: &mut HashMap<H256, i32>) {
		trace!("replay_keys: inserts={:?}, counters={:?}", inserts, counters);
		for h in inserts {
			if let Some(c) = counters.get_mut(h) {
				// already counting. increment.
				*c += 1;
				continue;
			}

			// this is the first entry for this node in the journal.
			// it is initialised to 1 if it was already in.
			if Self::is_already_in(backing, h) {
				trace!("replace_keys: Key {} was already in!", h);
				counters.insert(h.clone(), 1);
			}
		}
		trace!("replay_keys: (end) counters={:?}", counters);
	}

	fn kill_keys(deletes: Vec<H256>, counters: &mut HashMap<H256, i32>, batch: &DBTransaction) {
		for h in deletes.into_iter() {
			let mut n: Option<i32> = None;
			if let Some(c) = counters.get_mut(&h) {
				if *c > 1 {
					*c -= 1;
					continue;
				} else {
					n = Some(*c);
				}
			}
			match n {
				Some(i) if i == 1 => {
					counters.remove(&h);
					Self::reset_already_in(batch, &h);
				}
				None => {
					// Gets removed when moving from 1 to 0 additional refs. Should never be here at 0 additional refs.
					//assert!(!Self::is_already_in(db, &h));
					batch.delete(&h.bytes()).expect("Low-level database error. Some issue with your hard disk?");
				}
				_ => panic!("Invalid value in counters: {:?}", n),
			}
		}
	}

	fn payload(&self, key: &H256) -> Option<Bytes> {
		self.backing.get(&key.bytes()).expect("Low-level database error. Some issue with your hard disk?").map(|v| v.to_vec())
	}

	fn read_counters(db: &Database) -> HashMap<H256, i32> {
		let mut counters = HashMap::new();
		if let Some(val) = db.get(&LATEST_ERA_KEY).expect("Low-level database error.") {
			let mut era = decode::<u64>(&val);
			loop {
				let mut index = 0usize;
				while let Some(rlp_data) = db.get({
					let mut r = RlpStream::new_list(3);
					r.append(&era);
					r.append(&index);
					r.append(&&PADDING[..]);
					&r.drain()
				}).expect("Low-level database error.") {
					trace!("read_counters: era={}, index={}", era, index);
					let rlp = Rlp::new(&rlp_data);
					let inserts: Vec<H256> = rlp.val_at(1);
					Self::replay_keys(&inserts, db, &mut counters);
					index += 1;
				};
				if index == 0 || era == 0 {
					break;
				}
				era -= 1;
			}
		}
		trace!("Recovered {} counters", counters.len());
		counters
	}
}

impl HashDB for EarlyMergeDB {
	fn keys(&self) -> HashMap<H256, i32> {
		let mut ret: HashMap<H256, i32> = HashMap::new();
		for (key, _) in self.backing.iter() {
			let h = H256::from_slice(key.deref());
			ret.insert(h, 1);
		}

		for (key, refs) in self.overlay.keys().into_iter() {
			let refs = *ret.get(&key).unwrap_or(&0) + refs;
			ret.insert(key, refs);
		}
		ret
	}

	fn lookup(&self, key: &H256) -> Option<&[u8]> {
		let k = self.overlay.raw(key);
		match k {
			Some(&(ref d, rc)) if rc > 0 => Some(d),
			_ => {
				if let Some(x) = self.payload(key) {
					Some(&self.overlay.denote(key, x).0)
				}
				else {
					None
				}
			}
		}
	}

	fn exists(&self, key: &H256) -> bool {
		self.lookup(key).is_some()
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		self.overlay.insert(value)
	}
	fn emplace(&mut self, key: H256, value: Bytes) {
		self.overlay.emplace(key, value);
	}
	fn kill(&mut self, key: &H256) {
		self.overlay.kill(key);
	}
}

impl JournalDB for EarlyMergeDB {
	fn spawn(&self) -> Box<JournalDB> {
		Box::new(EarlyMergeDB {
			overlay: MemoryDB::new(),
			backing: self.backing.clone(),
			counters: self.counters.clone(),
		})
	}

	fn mem_used(&self) -> usize {
		self.overlay.mem_used() + match self.counters {
			Some(ref c) => c.read().unwrap().heap_size_of_children(),
			None => 0
		}
 	}

	fn is_empty(&self) -> bool {
		self.backing.get(&LATEST_ERA_KEY).expect("Low level database error").is_none()
	}

	fn commit(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		// journal format:
		// [era, 0] => [ id, [insert_0, ...], [remove_0, ...] ]
		// [era, 1] => [ id, [insert_0, ...], [remove_0, ...] ]
		// [era, n] => [ ... ]

		// TODO: store reclaim_period.

		// When we make a new commit, we make a journal of all blocks in the recent history and record
		// all keys that were inserted and deleted. The journal is ordered by era; multiple commits can
		// share the same era. This forms a data structure similar to a queue but whose items are tuples.
		// By the time comes to remove a tuple from the queue (i.e. then the era passes from recent history
		// into ancient history) then only one commit from the tuple is considered canonical. This commit
		// is kept in the main backing database, whereas any others from the same era are reverted.
		//
		// It is possible that a key, properly available in the backing database be deleted and re-inserted
		// in the recent history queue, yet have both operations in commits that are eventually non-canonical.
		// To avoid the original, and still required, key from being deleted, we maintain a reference count
		// which includes an original key, if any.
		//
		// The semantics of the `counter` are:
		// insert key k:
		//   counter already contains k: count += 1
		//   counter doesn't contain k:
		//     backing db contains k: count = 1
		//     backing db doesn't contain k: insert into backing db, count = 0
		// delete key k:
		//   counter contains k (count is asserted to be non-zero):
		//     count > 1: counter -= 1
		//     count == 1: remove counter
		//     count == 0: remove key from backing db
		//   counter doesn't contain k: remove key from backing db
		//
		// Practically, this means that for each commit block turning from recent to ancient we do the
		// following:
		// is_canonical:
		//   inserts: Ignored (left alone in the backing database).
		//   deletes: Enacted; however, recent history queue is checked for ongoing references. This is
		//            reduced as a preference to deletion from the backing database.
		// !is_canonical:
		//   inserts: Reverted; however, recent history queue is checked for ongoing references. This is
		//            reduced as a preference to deletion from the backing database.
		//   deletes: Ignored (they were never inserted).
		//

		// record new commit's details.
		trace!("commit: #{} ({}), end era: {:?}", now, id, end);
		let mut counters = self.counters.as_ref().unwrap().write().unwrap();
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

			let drained = self.overlay.drain();
			let removes: Vec<H256> = drained
				.iter()
				.filter_map(|(k, &(_, c))| if c < 0 {Some(k.clone())} else {None})
				.collect();
			let inserts: Vec<(H256, Bytes)> = drained
				.into_iter()
				.filter_map(|(k, (v, r))| if r > 0 { assert!(r == 1); Some((k, v)) } else { assert!(r >= -1); None })
				.collect();

			let mut r = RlpStream::new_list(3);
			r.append(id);

			// Process the new inserts.
			// We use the inserts for three things. For each:
			// - we place into the backing DB or increment the counter if already in;
			// - we note in the backing db that it was already in;
			// - we write the key into our journal for this block;

			r.begin_list(inserts.len());
			inserts.iter().foreach(|&(k, _)| {r.append(&k);});
			r.append(&removes);
			Self::insert_keys(&inserts, &self.backing, &mut counters, &batch);
			try!(batch.put(&last, r.as_raw()));
			try!(batch.put(&LATEST_ERA_KEY, &encode(&now)));
		}

		// apply old commits' details
		if let Some((end_era, canon_id)) = end {
			let mut index = 0usize;
			let mut last;
			while let Some(rlp_data) = try!(self.backing.get({
				let mut r = RlpStream::new_list(3);
				r.append(&end_era);
				r.append(&index);
				r.append(&&PADDING[..]);
				last = r.drain();
				&last
			})) {
				let rlp = Rlp::new(&rlp_data);
				let inserts: Vec<H256> = rlp.val_at(1);
				let deletes: Vec<H256> = rlp.val_at(2);
				// Collect keys to be removed. These are removed keys for canonical block, inserted for non-canonical
				Self::kill_keys(if canon_id == rlp.val_at(0) {deletes} else {inserts}, &mut counters, &batch);
				try!(batch.delete(&last));
				index += 1;
			}
			trace!("EarlyMergeDB: delete journal for time #{}.{}, (canon was {})", end_era, index, canon_id);
		}

		try!(self.backing.write(batch));
//		trace!("EarlyMergeDB::commit() deleted {} nodes", deletes);
		Ok(0)
	}
}

#[cfg(test)]
mod tests {
	use common::*;
	use super::*;
	use hashdb::*;
	use journaldb::traits::JournalDB;

	#[test]
	fn insert_same_in_fork() {
		// history is 1
		let mut jdb = EarlyMergeDB::new_temp();

		let x = jdb.insert(b"X");
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		jdb.commit(2, &b"2".sha3(), None).unwrap();
		jdb.commit(3, &b"1002a".sha3(), Some((1, b"1".sha3()))).unwrap();
		jdb.commit(4, &b"1003a".sha3(), Some((2, b"2".sha3()))).unwrap();

		jdb.remove(&x);
		jdb.commit(3, &b"1002b".sha3(), Some((1, b"1".sha3()))).unwrap();
		let x = jdb.insert(b"X");
		jdb.commit(4, &b"1003b".sha3(), Some((2, b"2".sha3()))).unwrap();

		jdb.commit(5, &b"1004a".sha3(), Some((3, b"1002a".sha3()))).unwrap();
		jdb.commit(6, &b"1005a".sha3(), Some((4, b"1003a".sha3()))).unwrap();

		assert!(jdb.exists(&x));
	}

	#[test]
	fn long_history() {
		// history is 3
		let mut jdb = EarlyMergeDB::new_temp();
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
		let mut jdb = EarlyMergeDB::new_temp();

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
		let mut jdb = EarlyMergeDB::new_temp();

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

	#[test]
	fn overwrite() {
		// history is 1
		let mut jdb = EarlyMergeDB::new_temp();

		let foo = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.exists(&foo));

		jdb.remove(&foo);
		jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
		jdb.insert(b"foo");
		assert!(jdb.exists(&foo));
		jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.exists(&foo));
		jdb.commit(3, &b"2".sha3(), Some((0, b"2".sha3()))).unwrap();
		assert!(jdb.exists(&foo));
	}

	#[test]
	fn fork_same_key() {
		// history is 1
		let mut jdb = EarlyMergeDB::new_temp();
		jdb.commit(0, &b"0".sha3(), None).unwrap();

		let foo = jdb.insert(b"foo");
		jdb.commit(1, &b"1a".sha3(), Some((0, b"0".sha3()))).unwrap();

		jdb.insert(b"foo");
		jdb.commit(1, &b"1b".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.exists(&foo));

		jdb.commit(2, &b"2a".sha3(), Some((1, b"1a".sha3()))).unwrap();
		assert!(jdb.exists(&foo));
	}


	#[test]
	fn reopen() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let bar = H256::random();

		let foo = {
			let mut jdb = EarlyMergeDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			jdb.emplace(bar.clone(), b"bar".to_vec());
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			foo
		};

		{
			let mut jdb = EarlyMergeDB::new(dir.to_str().unwrap());
			jdb.remove(&foo);
			jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
		}

		{
			let mut jdb = EarlyMergeDB::new(dir.to_str().unwrap());
			assert!(jdb.exists(&foo));
			assert!(jdb.exists(&bar));
			jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
			assert!(!jdb.exists(&foo));
		}
	}

	#[test]
	fn reopen_remove() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let foo = {
			let mut jdb = EarlyMergeDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();

			// foo is ancient history.

			jdb.insert(b"foo");
			jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
			foo
		};

		{
			let mut jdb = EarlyMergeDB::new(dir.to_str().unwrap());
			jdb.remove(&foo);
			jdb.commit(3, &b"3".sha3(), Some((2, b"2".sha3()))).unwrap();
			assert!(jdb.exists(&foo));
			jdb.remove(&foo);
			jdb.commit(4, &b"4".sha3(), Some((3, b"3".sha3()))).unwrap();
			jdb.commit(5, &b"5".sha3(), Some((4, b"4".sha3()))).unwrap();
			assert!(!jdb.exists(&foo));
		}
	}
	#[test]
	fn reopen_fork() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let (foo, bar, baz) = {
			let mut jdb = EarlyMergeDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			let bar = jdb.insert(b"bar");
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			jdb.remove(&foo);
			let baz = jdb.insert(b"baz");
			jdb.commit(1, &b"1a".sha3(), Some((0, b"0".sha3()))).unwrap();

			jdb.remove(&bar);
			jdb.commit(1, &b"1b".sha3(), Some((0, b"0".sha3()))).unwrap();
			(foo, bar, baz)
		};

		{
			let mut jdb = EarlyMergeDB::new(dir.to_str().unwrap());
			jdb.commit(2, &b"2b".sha3(), Some((1, b"1b".sha3()))).unwrap();
			assert!(jdb.exists(&foo));
			assert!(!jdb.exists(&baz));
			assert!(!jdb.exists(&bar));
		}
	}
}
