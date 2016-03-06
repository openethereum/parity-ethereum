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
	transaction_overlay: MemoryDB,
	backing: Arc<Database>,
	journal_overlay: Option<Arc<RwLock<JournalOverlay>>>,
}

struct JournalOverlay {
	backing_overlay: MemoryDB,
	journal: VecDeque<JournalEntry>
}

struct JournalEntry {
	id: H256,
	index: usize,
	era: u64,
	insertions: Vec<H256>,
	deletions: Vec<H256>,
}

impl HeapSizeOf for JournalEntry {
	fn heap_size_of_children(&self) -> usize {
		self.insertions.heap_size_of_children() + self.deletions.heap_size_of_children()
	}
}

impl Clone for JournalDB {
	fn clone(&self) -> JournalDB {
		JournalDB {
			transaction_overlay: MemoryDB::new(),
			backing: self.backing.clone(),
			journal_overlay: self.journal_overlay.clone(),
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


		let journal_overlay = if with_journal {
			Some(Arc::new(RwLock::new(JournalDB::read_overlay(&backing))))
		} else {
			None
		};
		JournalDB {
			transaction_overlay: MemoryDB::new(),
			backing: Arc::new(backing),
			journal_overlay: journal_overlay,
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
		let have_journal_overlay = self.journal_overlay.is_some();
		if have_journal_overlay {
			self.commit_with_overlay(now, id, end)
		} else {
			self.commit_without_overlay()
		}
	}

	/// Drain the overlay and place it into a batch for the DB.
	fn batch_overlay_insertions(overlay: &mut MemoryDB, batch: &DBTransaction) -> usize {
		let mut insertions = 0usize;
		let mut deletions = 0usize;
		for i in overlay.drain().into_iter() {
			let (key, (value, rc)) = i;
			if rc > 0 {
				assert!(rc == 1);
				batch.put(&key.bytes(), &value).expect("Low-level database error. Some issue with your hard disk?");
				insertions += 1;
			}
			if rc < 0 {
				assert!(rc == -1);
				deletions += 1;
			}
		}
		trace!("commit: Inserted {}, Deleted {} nodes", insertions, deletions);
		insertions + deletions
	}

	/// Just commit the transaction overlay into the backing DB.
	fn commit_without_overlay(&mut self) -> Result<u32, UtilError> {
		let batch = DBTransaction::new();
		let ret = Self::batch_overlay_insertions(&mut self.transaction_overlay, &batch);
		try!(self.backing.write(batch));
		Ok(ret as u32)
	}

	/// Commit all recent insert operations and historical removals from the old era
	/// to the backing database.
	fn commit_with_overlay(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		// record new commit's details.
		trace!("commit: #{} ({}), end era: {:?}", now, id, end);
		let mut journal_overlay = self.journal_overlay.as_mut().unwrap().write().unwrap();
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
			let mut tx = self.transaction_overlay.drain();
			let inserted_keys: Vec<_> = tx.iter().filter_map(|(k, &(_, c))| if c > 0 { Some(k.clone()) } else { None }).collect();
			let removed_keys: Vec<_> = tx.iter().filter_map(|(k, &(_, c))| if c < 0 { Some(k.clone()) } else { None }).collect();
			// Increase counter for each inserted key no matter if the block is canonical or not. 
			let insertions = tx.drain().filter_map(|(k, (v, c))| if c > 0 { Some((k, v)) } else { None });
			r.append(id);
			r.begin_list(inserted_keys.len());
			for (k, v) in insertions {
				r.begin_list(2);
				r.append(&k);
				r.append(&v);
				journal_overlay.backing_overlay.emplace(k, v);
			}
			r.append(&removed_keys);
			try!(batch.put(&last, r.as_raw()));
			try!(batch.put(&LATEST_ERA_KEY, &encode(&now)));
			journal_overlay.journal.push_back(JournalEntry { id: id.clone(), index: index, era: now, insertions: inserted_keys, deletions: removed_keys });
		}

		// apply old commits' details
		
		if let Some((end_era, canon_id)) = end {
			let mut canon_insertions: Vec<(H256, Bytes)> = Vec::new();
			let mut canon_deletions: Vec<H256> = Vec::new();
			let mut overlay_deletions: Vec<H256> = Vec::new();
			while journal_overlay.journal.front().map_or(false, |e| e.era <= end_era) {
				let mut journal = journal_overlay.journal.pop_front().unwrap();
				//delete the record from the db
				let mut r = RlpStream::new_list(3);
				r.append(&journal.era);
				r.append(&journal.index);
				r.append(&&PADDING[..]);
				try!(batch.delete(&r.drain()));
				trace!("commit: Delete journal for time #{}.{}: {}, (canon was {}): +{} -{} entries", end_era, journal.index, journal.id, canon_id, journal.insertions.len(), journal.deletions.len());
				{
					if canon_id == journal.id {
						for h in &journal.insertions {
							match journal_overlay.backing_overlay.raw(&h) {
								Some(&(ref d, rc)) if rc > 0 => canon_insertions.push((h.clone(), d.clone())), //TODO: optimizie this to avoid data copy
								_ => ()
							}
						}
						canon_deletions = journal.deletions;
					}
					overlay_deletions.append(&mut journal.insertions);
				}
				if canon_id == journal.id {
				}
			}
			// apply canon inserts first
			for (k, v) in canon_insertions {
				try!(batch.put(&k, &v));
			}
			// clean the overlay
			for k in overlay_deletions {
				journal_overlay.backing_overlay.kill(&k);
			}
			// apply removes
			for k in canon_deletions {
				if !journal_overlay.backing_overlay.exists(&k) {
					try!(batch.delete(&k));
				}
			}
			journal_overlay.backing_overlay.purge();
		}
		try!(self.backing.write(batch));
		Ok(0 as u32)
	}

	fn payload(&self, key: &H256) -> Option<Bytes> {
		self.backing.get(&key.bytes()).expect("Low-level database error. Some issue with your hard disk?").map(|v| v.to_vec())
	}

	fn read_overlay(db: &Database) -> JournalOverlay {
		let mut journal = VecDeque::new();
		let mut overlay = MemoryDB::new();
		let mut count = 0;
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
					let id: H256 = rlp.val_at(0);
					let insertions = rlp.at(1);
					let deletions: Vec<H256> = rlp.val_at(2);
					let mut inserted_keys = Vec::new();
					for r in insertions.iter() {
						let k: H256 = r.val_at(0);
						let v: Bytes = r.val_at(1);
						overlay.emplace(k.clone(), v);
						inserted_keys.push(k);
						count += 1;
					}
					journal.push_front(JournalEntry {
						id: id,
						index: index,
						era: era,
						insertions: inserted_keys,
						deletions: deletions,
					});
					index += 1;
				};
				if index == 0 || era == 0 {
					break;
				}
				era -= 1;
			}
		}
		trace!("Recovered {} overlay entries, {} journal entries", count, journal.len());
		JournalOverlay { backing_overlay: overlay, journal: journal }
	}

	/// Returns heap memory size used
	pub fn mem_used(&self) -> usize {
		let mut mem = self.transaction_overlay.mem_used();
		if let Some(ref overlay) = self.journal_overlay.as_ref() {
			let overlay = overlay.read().unwrap();
			mem += overlay.backing_overlay.mem_used();
			mem += overlay.journal.heap_size_of_children();
		}
		mem
	}
}

impl HashDB for JournalDB {
	fn keys(&self) -> HashMap<H256, i32> { 
		let mut ret: HashMap<H256, i32> = HashMap::new();
		for (key, _) in self.backing.iter() {
			let h = H256::from_slice(key.deref());
			ret.insert(h, 1);
		}

		for (key, refs) in self.transaction_overlay.keys().into_iter() {
			let refs = *ret.get(&key).unwrap_or(&0) + refs;
			ret.insert(key, refs);
		}
		ret
	}

	fn lookup(&self, key: &H256) -> Option<&[u8]> { 
		let k = self.transaction_overlay.raw(key);
		match k {
			Some(&(ref d, rc)) if rc > 0 => Some(d),
			_ => {
				let v = self.journal_overlay.as_ref().map_or(None, |ref j| j.read().unwrap().backing_overlay.lookup(key).map(|v| v.to_vec()));
				match v {
					Some(x) => {
						Some(&self.transaction_overlay.denote(key, x).0)
					}
					_ => {
						if let Some(x) = self.payload(key) {
							Some(&self.transaction_overlay.denote(key, x).0)
						}
						else {
							None
						}
					}
				}
			}
		}
	}

	fn exists(&self, key: &H256) -> bool { 
		self.lookup(key).is_some()
	}

	fn insert(&mut self, value: &[u8]) -> H256 { 
		self.transaction_overlay.insert(value)
	}
	fn emplace(&mut self, key: H256, value: Bytes) {
		self.transaction_overlay.emplace(key, value); 
	}
	fn kill(&mut self, key: &H256) { 
		self.transaction_overlay.kill(key); 
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
		let bar = H256::random();

		let foo = {
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			jdb.emplace(bar.clone(), b"bar".to_vec());
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
			assert!(jdb.exists(&bar));
			jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
			assert!(!jdb.exists(&foo));
		}
	}

	#[test]
	fn reopen_remove() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let bar = H256::random();

		let foo = {
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			jdb.insert(b"foo");
			jdb.commit(1, &b"1".sha3(), None).unwrap();
			foo
		};

		{
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
			jdb.remove(&foo);
			jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
			assert!(jdb.exists(&foo));
			jdb.commit(3, &b"3".sha3(), Some((2, b"2".sha3()))).unwrap();
			assert!(!jdb.exists(&foo));
		}
	}
	#[test]
	fn reopen_fork() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let (foo, bar, baz) = {
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
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
			let mut jdb = JournalDB::new(dir.to_str().unwrap());
			jdb.commit(2, &b"2b".sha3(), Some((1, b"1b".sha3()))).unwrap();
			assert!(jdb.exists(&foo));
			assert!(!jdb.exists(&baz));
			assert!(!jdb.exists(&bar));
		}
	}
}
