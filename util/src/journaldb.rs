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
use kvdb::{Database, DBTransaction, DatabaseConfig};
#[cfg(test)]
use std::env;

/// Implementation of the HashDB trait for a disk-backed database with a memory overlay
/// and, possibly, latent-removal semantics.
///
/// If `counters` is `None`, then it behaves exactly like OverlayDB. If not it behaves
/// differently:
///
/// Like OverlayDB, there is a memory overlay; `commit()` must be called in order to 
/// write operations out to disk. Unlike OverlayDB, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
pub struct JournalDB {
	overlay: MemoryDB,
	backing: Arc<Database>,
	counters: Option<Arc<RwLock<HashMap<H256, i32>>>>,
}

impl Clone for JournalDB {
	fn clone(&self) -> JournalDB {
		JournalDB {
			overlay: MemoryDB::new(),
			backing: self.backing.clone(),
			counters: self.counters.clone(),
		}
	}
}

// all keys must be at least 12 bytes
const LATEST_ERA_KEY : [u8; 12] = [ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];
const VERSION_KEY : [u8; 12] = [ b'j', b'v', b'e', b'r', 0, 0, 0, 0, 0, 0, 0, 0 ];

const DB_VERSION : u32 = 3;
const DB_VERSION_NO_JOURNAL : u32 = 3 + 256;

const PADDING : [u8; 10] = [ 0u8; 10 ];

impl JournalDB {

	/// Create a new instance from file
	pub fn new(path: &str) -> JournalDB {
		Self::from_prefs(path, true)
	}

	/// Create a new instance from file
	pub fn from_prefs(path: &str, prefer_journal: bool) -> JournalDB {
		let opts = DatabaseConfig {
			prefix_size: Some(12) //use 12 bytes as prefix, this must match account_db prefix
		};
		let backing = Database::open(&opts, path).unwrap_or_else(|e| {
			panic!("Error opening state db: {}", e);
		});
		let with_journal;
		if !backing.is_empty() {
			match backing.get(&VERSION_KEY).map(|d| d.map(|v| decode::<u32>(&v))) {
				Ok(Some(DB_VERSION)) => { with_journal = true; },
				Ok(Some(DB_VERSION_NO_JOURNAL)) => { with_journal = false; },
				v => panic!("Incompatible DB version, expected {}, got {:?}", DB_VERSION, v)
			}
		} else {
			backing.put(&VERSION_KEY, &encode(&(if prefer_journal { DB_VERSION } else { DB_VERSION_NO_JOURNAL }))).expect("Error writing version to database");
			with_journal = prefer_journal;
		}

		let counters = if with_journal {
			Some(Arc::new(RwLock::new(JournalDB::read_counters(&backing))))
		} else {
			None
		};
		JournalDB {
			overlay: MemoryDB::new(),
			backing: Arc::new(backing),
			counters: counters,
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> JournalDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(dir.to_str().unwrap())
	}

	/// Check if this database has any commits
	pub fn is_empty(&self) -> bool {
		self.backing.get(&LATEST_ERA_KEY).expect("Low level database error").is_none()
	}

	/// Commit all recent insert operations.
	pub fn commit(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		let have_counters = self.counters.is_some();
		if have_counters {
			self.commit_with_counters(now, id, end)
		} else {
			self.commit_without_counters()
		}
	}

	/// Drain the overlay and place it into a batch for the DB.
	fn batch_overlay_insertions(overlay: &mut MemoryDB, batch: &DBTransaction) -> usize {
		let mut inserts = 0usize;
		let mut deletes = 0usize;
		for i in overlay.drain().into_iter() {
			let (key, (value, rc)) = i;
			if rc > 0 {
				assert!(rc == 1);
				batch.put(&key.bytes(), &value).expect("Low-level database error. Some issue with your hard disk?");
				inserts += 1;
			}
			if rc < 0 {
				assert!(rc == -1);
				deletes += 1;
			}
		}
		trace!("commit: Inserted {}, Deleted {} nodes", inserts, deletes);
		inserts + deletes
	}

	/// Just commit the overlay into the backing DB.
	fn commit_without_counters(&mut self) -> Result<u32, UtilError> {
		let batch = DBTransaction::new();
		let ret = Self::batch_overlay_insertions(&mut self.overlay, &batch);
		try!(self.backing.write(batch));
		Ok(ret as u32)
	}

	/// Commit all recent insert operations and historical removals from the old era
	/// to the backing database.
	fn commit_with_counters(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		// journal format: 
		// [era, 0] => [ id, [insert_0, ...], [remove_0, ...] ]
		// [era, 1] => [ id, [insert_0, ...], [remove_0, ...] ]
		// [era, n] => [ ... ]

		// TODO: store reclaim_period.

		// when we make a new commit, we journal the inserts and removes.
		// for each end_era that we journaled that we are no passing by, 
		// we remove all of its removes assuming it is canonical and all
		// of its inserts otherwise.
		//
		// We also keep reference counters for each key inserted in the journal to handle 
		// the following cases where key K must not be deleted from the DB when processing removals :
		// Given H is the journal size in eras, 0 <= C <= H.
		// Key K is removed in era A(N) and re-inserted in canonical era B(N + C).
		// Key K is removed in era A(N) and re-inserted in non-canonical era B`(N + C).
		// Key K is added in non-canonical era A'(N) canonical B(N + C).
		//
		// The counter is encreased each time a key is inserted in the journal in the commit. The list of insertions
		// is saved with the era record. When the era becomes end_era and goes out of journal the counter is decreased
		// and the key is safe to delete.

		// record new commit's details.
		trace!("commit: #{} ({}), end era: {:?}", now, id, end);
		let mut counters = self.counters.as_ref().unwrap().write().unwrap();
		let batch = DBTransaction::new();
		{
			let mut index = 0usize;
			let mut last;

			while {
				let record = try!(self.backing.get({
					let mut r = RlpStream::new_list(3);
					r.append(&now);
					r.append(&index);
					r.append(&&PADDING[..]);
					last = r.drain();
					&last
				}));
				match record {
					Some(r) => {
						assert!(&Rlp::new(&r).val_at::<H256>(0) != id);
						true
					},
					None => false,
				}
			} {
				index += 1;
			}

			let mut r = RlpStream::new_list(3);
			let inserts: Vec<H256> = self.overlay.keys().iter().filter(|&(_, &c)| c > 0).map(|(key, _)| key.clone()).collect();
			// Increase counter for each inserted key no matter if the block is canonical or not. 
			for i in &inserts {
				*counters.entry(i.clone()).or_insert(0) += 1;
			}
			let removes: Vec<H256> = self.overlay.keys().iter().filter(|&(_, &c)| c < 0).map(|(key, _)| key.clone()).collect();
			r.append(id);
			r.append(&inserts);
			r.append(&removes);
			try!(batch.put(&last, r.as_raw()));
			try!(batch.put(&LATEST_ERA_KEY, &encode(&now)));
		}

		// apply old commits' details
		if let Some((end_era, canon_id)) = end {
			let mut index = 0usize;
			let mut last;
			let mut to_remove: Vec<H256> = Vec::new();
			let mut canon_inserts: Vec<H256> = Vec::new();
			while let Some(rlp_data) = try!(self.backing.get({
				let mut r = RlpStream::new_list(3);
				r.append(&end_era);
				r.append(&index);
				r.append(&&PADDING[..]);
				last = r.drain();
				&last
			})) {
				let rlp = Rlp::new(&rlp_data);
				let mut inserts: Vec<H256> = rlp.val_at(1);
				JournalDB::decrease_counters(&inserts, &mut counters);
				// Collect keys to be removed. These are removed keys for canonical block, inserted for non-canonical
				if canon_id == rlp.val_at(0) {
					let mut canon_deletes: Vec<H256> = rlp.val_at(2);
					trace!("Purging nodes deleted from canon: {:?}", canon_deletes);
					to_remove.append(&mut canon_deletes);
					canon_inserts = inserts;
				}
				else {
					trace!("Purging nodes inserted in non-canon: {:?}", inserts);
					to_remove.append(&mut inserts);
				}
				trace!("commit: Delete journal for time #{}.{}: {}, (canon was {}): {} entries", end_era, index, rlp.val_at::<H256>(0), canon_id, to_remove.len());
				try!(batch.delete(&last));
				index += 1;
			}

			let canon_inserts = canon_inserts.drain(..).collect::<HashSet<_>>();
			// Purge removed keys if they are not referenced and not re-inserted in the canon commit
			let mut deletes = 0;
			trace!("Purging filtered notes: {:?}", to_remove.iter().filter(|h| !counters.contains_key(h) && !canon_inserts.contains(h)).collect::<Vec<_>>());
			for h in to_remove.iter().filter(|h| !counters.contains_key(h) && !canon_inserts.contains(h)) {
				try!(batch.delete(&h));
				deletes += 1;
			}
			trace!("Total nodes purged: {}", deletes);
		}

		// Commit overlay insertions
		let ret = Self::batch_overlay_insertions(&mut self.overlay, &batch);
		try!(self.backing.write(batch));
		Ok(ret as u32)
	}


	// Decrease counters for given keys. Deletes obsolete counters
	fn decrease_counters(keys: &[H256], counters: &mut HashMap<H256, i32>) {
		for i in keys.iter() {
			let delete_counter = {
				let cnt = counters.get_mut(i).expect("Missing key counter");
				*cnt -= 1;
				*cnt == 0
			};
			if delete_counter {
				counters.remove(i);
			}
		}
	}

	fn payload(&self, key: &H256) -> Option<Bytes> {
		self.backing.get(&key.bytes()).expect("Low-level database error. Some issue with your hard disk?").map(|v| v.to_vec())
	}

	fn read_counters(db: &Database) -> HashMap<H256, i32> {
		let mut res = HashMap::new();
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
					let rlp = Rlp::new(&rlp_data);
					let to_add: Vec<H256> = rlp.val_at(1);
					for h in to_add {
						*res.entry(h).or_insert(0) += 1;
					}
					index += 1;
				};
				if index == 0 || era == 0 {
					break;
				}
				era -= 1;
			}
		}
		trace!("Recovered {} counters", res.len());
		res
	}
}

impl HashDB for JournalDB {
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

	#[test]
	fn overwrite() {
		// history is 1
		let mut jdb = JournalDB::new_temp();

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
		let mut jdb = JournalDB::new_temp();
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

		let foo = {
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			foo
		};

		{
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
			jdb.remove(&foo);
			jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
		}

		{
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
			assert!(jdb.exists(&foo));
			jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
			assert!(!jdb.exists(&foo));
		}
	}
}
