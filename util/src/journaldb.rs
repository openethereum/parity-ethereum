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
use rocksdb::{DB, Writable, WriteBatch, IteratorMode};
#[cfg(test)]
use std::env;

/// Implementation of the HashDB trait for a disk-backed database with a memory overlay
/// and latent-removal semantics.
///
/// Like OverlayDB, there is a memory overlay; `commit()` must be called in order to 
/// write operations out to disk. Unlike OverlayDB, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
pub struct JournalDB {
	overlay: MemoryDB,
	backing: Arc<DB>,
	counters: Arc<RwLock<HashMap<H256, i32>>>,
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

const LAST_ERA_KEY : [u8; 4] = [ b'l', b'a', b's', b't' ]; 
const VERSION_KEY : [u8; 4] = [ b'j', b'v', b'e', b'r' ]; 

const DB_VERSION: u32 = 2;

impl JournalDB {
	/// Create a new instance given a `backing` database.
	pub fn new(backing: DB) -> JournalDB {
		let db = Arc::new(backing);
		JournalDB::new_with_arc(db)
	}

	/// Create a new instance given a shared `backing` database.
	pub fn new_with_arc(backing: Arc<DB>) -> JournalDB {
		if backing.iterator(IteratorMode::Start).next().is_some() {
			match backing.get(&VERSION_KEY).map(|d| d.map(|v| decode::<u32>(&v))) {
				Ok(Some(DB_VERSION)) => {},
				v => panic!("Incompatible DB version, expected {}, got {:?}", DB_VERSION, v)
			}
		} else {
			backing.put(&VERSION_KEY, &encode(&DB_VERSION)).expect("Error writing version to database");
		}
		let counters = JournalDB::read_counters(&backing);
		JournalDB {
			overlay: MemoryDB::new(),
			backing: backing,
			counters: Arc::new(RwLock::new(counters)),
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> JournalDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(DB::open_default(dir.to_str().unwrap()).unwrap())
	}

	/// Check if this database has any commits
	pub fn is_empty(&self) -> bool {
		self.backing.get(&LAST_ERA_KEY).expect("Low level database error").is_none()
	}

	/// Commit all recent insert operations and historical removals from the old era
	/// to the backing database.
	pub fn commit(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
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
		let batch = WriteBatch::new();
		let mut counters = self.counters.write().unwrap();
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
		}

		// apply old commits' details
		if let Some((end_era, canon_id)) = end {
			let mut index = 0usize;
			let mut last;
			let mut to_remove: Vec<H256> = Vec::new();
			let mut canon_inserts: Vec<H256> = Vec::new();
			while let Some(rlp_data) = try!(self.backing.get({
				let mut r = RlpStream::new_list(2);
				r.append(&end_era);
				r.append(&index);
				last = r.drain();
				&last
			})) {
				let rlp = Rlp::new(&rlp_data);
				let inserts: Vec<H256> = rlp.val_at(1);
				JournalDB::decrease_counters(&inserts, &mut counters);
				// Collect keys to be removed. These are removed keys for canonical block, inserted for non-canonical
				if canon_id == rlp.val_at(0) {
					to_remove.extend(rlp.at(2).iter().map(|r| r.as_val::<H256>()));
					canon_inserts = inserts;
				}
				else {
					to_remove.extend(inserts);
				}
				try!(batch.delete(&last));
				index += 1;
			}

			let canon_inserts = canon_inserts.drain(..).collect::<HashSet<_>>();
			// Purge removed keys if they are not referenced and not re-inserted in the canon commit
			let mut deletes = 0;
			for h in to_remove.iter().filter(|h| !counters.contains_key(h) && !canon_inserts.contains(h)) {
				try!(batch.delete(&h));
				deletes += 1;
			}
			try!(batch.put(&LAST_ERA_KEY, &encode(&end_era)));
			trace!("JournalDB: delete journal for time #{}.{}, (canon was {}): {} entries", end_era, index, canon_id, deletes);
		}

		// Commit overlay insertions
		let mut ret = 0u32;
		let mut deletes = 0usize;
		for i in self.overlay.drain().into_iter() {
			let (key, (value, rc)) = i;
			if rc > 0 {
				assert!(rc == 1);
				batch.put(&key.bytes(), &value).expect("Low-level database error. Some issue with your hard disk?");
				ret += 1;
			}
			if rc < 0 {
				assert!(rc == -1);
				ret += 1;
				deletes += 1;
			}
		}

		try!(self.backing.write(batch));
		trace!("JournalDB::commit() deleted {} nodes", deletes);
		Ok(ret)
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

	fn read_counters(db: &DB) -> HashMap<H256, i32> {
		let mut res = HashMap::new();
		if let Some(val) = db.get(&LAST_ERA_KEY).expect("Low-level database error.") {
			let mut era = decode::<u64>(&val) + 1;
			loop {
				let mut index = 0usize;
				while let Some(rlp_data) = db.get({
					let mut r = RlpStream::new_list(2);
					r.append(&era);
					r.append(&index);
					&r.drain()
				}).expect("Low-level database error.") {
					let rlp = Rlp::new(&rlp_data);
					let to_add: Vec<H256> = rlp.val_at(1);
					for h in to_add {
						*res.entry(h).or_insert(0) += 1;
					}
					index += 1;
				};
				if index == 0 {
					break;
				}
				era += 1;
			}
		}
		trace!("Recovered {} counters", res.len());
		res
	}
}

impl HashDB for JournalDB {
	fn keys(&self) -> HashMap<H256, i32> { 
		let mut ret: HashMap<H256, i32> = HashMap::new();
		for (key, _) in self.backing.iterator(IteratorMode::Start) {
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
}
