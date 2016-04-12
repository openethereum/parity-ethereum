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

//! `JournalDB` over in-memory overlay

use common::*;
use rlp::*;
use hashdb::*;
use memorydb::*;
use kvdb::{Database, DBTransaction, DatabaseConfig};
#[cfg(test)]
use std::env;
use super::JournalDB;

/// Implementation of the `JournalDB` trait for a disk-backed database with a memory overlay
/// and, possibly, latent-removal semantics.
///
/// Like `OverlayDB`, there is a memory overlay; `commit()` must be called in order to
/// write operations out to disk. Unlike `OverlayDB`, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
///
/// There are two memory overlays:
/// - Transaction overlay contains current transaction data. It is merged with with history
/// overlay on each `commit()`
/// - History overlay contains all data inserted during the history period. When the node
/// in the overlay becomes ancient it is written to disk on `commit()`
///
/// There is also a journal maintained in memory and on the disk as well which lists insertions
/// and removals for each commit during the history period. This is used to track
/// data nodes that go out of history scope and must be written to disk.
///
/// Commit workflow:
/// 1. Create a new journal record from the transaction overlay.
/// 2. Inseart each node from the transaction overlay into the History overlay increasing reference
/// count if it is already there. Note that the reference counting is managed by `MemoryDB`
/// 3. Clear the transaction overlay.
/// 4. For a canonical journal record that becomes ancient inserts its insertions into the disk DB
/// 5. For each journal record that goes out of the history scope (becomes ancient) remove its
/// insertions from the history overlay, decreasing the reference counter and removing entry if
/// if reaches zero.
/// 6. For a canonical journal record that becomes ancient delete its removals from the disk only if
/// the removed key is not present in the history overlay.
/// 7. Delete ancient record from memory and disk.

pub struct OverlayRecentDB {
	transaction_overlay: MemoryDB,
	backing: Arc<Database>,
	journal_overlay: Arc<RwLock<JournalOverlay>>,
}

#[derive(PartialEq)]
struct JournalOverlay {
	backing_overlay: MemoryDB,
	journal: HashMap<u64, Vec<JournalEntry>>,
	latest_era: Option<u64>,
}

#[derive(PartialEq)]
struct JournalEntry {
	id: H256,
	insertions: Vec<H256>,
	deletions: Vec<H256>,
}

impl HeapSizeOf for JournalEntry {
	fn heap_size_of_children(&self) -> usize {
		self.insertions.heap_size_of_children() + self.deletions.heap_size_of_children()
	}
}

impl Clone for OverlayRecentDB {
	fn clone(&self) -> OverlayRecentDB {
		OverlayRecentDB {
			transaction_overlay: self.transaction_overlay.clone(),
			backing: self.backing.clone(),
			journal_overlay: self.journal_overlay.clone(),
		}
	}
}

// all keys must be at least 12 bytes
const LATEST_ERA_KEY : [u8; 12] = [ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];
const VERSION_KEY : [u8; 12] = [ b'j', b'v', b'e', b'r', 0, 0, 0, 0, 0, 0, 0, 0 ];
const DB_VERSION : u32 = 0x203;
const PADDING : [u8; 10] = [ 0u8; 10 ];

impl OverlayRecentDB {
	/// Create a new instance from file
	pub fn new(path: &str) -> OverlayRecentDB {
		Self::from_prefs(path)
	}

	/// Create a new instance from file
	pub fn from_prefs(path: &str) -> OverlayRecentDB {
		let opts = DatabaseConfig {
			prefix_size: Some(12) //use 12 bytes as prefix, this must match account_db prefix
		};
		let backing = Database::open(&opts, path).unwrap_or_else(|e| {
			panic!("Error opening state db: {}", e);
		});
		if !backing.is_empty() {
			match backing.get(&VERSION_KEY).map(|d| d.map(|v| decode::<u32>(&v))) {
				Ok(Some(DB_VERSION)) => {}
				v => panic!("Incompatible DB version, expected {}, got {:?}; to resolve, remove {} and restart.", DB_VERSION, v, path)
			}
		} else {
			backing.put(&VERSION_KEY, &encode(&DB_VERSION)).expect("Error writing version to database");
		}

		let journal_overlay = Arc::new(RwLock::new(OverlayRecentDB::read_overlay(&backing)));
		OverlayRecentDB {
			transaction_overlay: MemoryDB::new(),
			backing: Arc::new(backing),
			journal_overlay: journal_overlay,
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> OverlayRecentDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(dir.to_str().unwrap())
	}

	#[cfg(test)]
	fn can_reconstruct_refs(&self) -> bool {
		let reconstructed = Self::read_overlay(&self.backing);
		let journal_overlay = self.journal_overlay.read().unwrap();
		*journal_overlay == reconstructed
	}

	fn payload(&self, key: &H256) -> Option<Bytes> {
		self.backing.get(&key.bytes()).expect("Low-level database error. Some issue with your hard disk?").map(|v| v.to_vec())
	}

	fn read_overlay(db: &Database) -> JournalOverlay {
		let mut journal = HashMap::new();
		let mut overlay = MemoryDB::new();
		let mut count = 0;
		let mut latest_era = None;
		if let Some(val) = db.get(&LATEST_ERA_KEY).expect("Low-level database error.") {
			let mut era = decode::<u64>(&val);
			latest_era = Some(era);
			loop {
				let mut index = 0usize;
				while let Some(rlp_data) = db.get({
					let mut r = RlpStream::new_list(3);
					r.append(&era);
					r.append(&index);
					r.append(&&PADDING[..]);
					&r.drain()
				}).expect("Low-level database error.") {
					trace!("read_overlay: era={}, index={}", era, index);
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
					journal.entry(era).or_insert_with(Vec::new).push(JournalEntry {
						id: id,
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
		JournalOverlay { backing_overlay: overlay, journal: journal, latest_era: latest_era }
	}
}

impl JournalDB for OverlayRecentDB {
	fn boxed_clone(&self) -> Box<JournalDB> {
		Box::new(self.clone())
	}

	fn mem_used(&self) -> usize {
		let mut mem = self.transaction_overlay.mem_used();
		let overlay = self.journal_overlay.read().unwrap();
		mem += overlay.backing_overlay.mem_used();
		mem += overlay.journal.heap_size_of_children();
		mem
	}

	fn is_empty(&self) -> bool {
		self.backing.get(&LATEST_ERA_KEY).expect("Low level database error").is_none()
	}

	fn latest_era(&self) -> Option<u64> { self.journal_overlay.read().unwrap().latest_era }

	fn commit(&mut self, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		// record new commit's details.
		trace!("commit: #{} ({}), end era: {:?}", now, id, end);
		let mut journal_overlay = self.journal_overlay.write().unwrap();
		let batch = DBTransaction::new();
		{
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

			let mut k = RlpStream::new_list(3);
			let index = journal_overlay.journal.get(&now).map_or(0, |j| j.len());
			k.append(&now);
			k.append(&index);
			k.append(&&PADDING[..]);
			try!(batch.put(&k.drain(), r.as_raw()));
			if journal_overlay.latest_era.map_or(true, |e| now > e) {
				try!(batch.put(&LATEST_ERA_KEY, &encode(&now)));
				journal_overlay.latest_era = Some(now);
			}
			journal_overlay.journal.entry(now).or_insert_with(Vec::new).push(JournalEntry { id: id.clone(), insertions: inserted_keys, deletions: removed_keys });
		}

		let journal_overlay = journal_overlay.deref_mut();
		// apply old commits' details
		if let Some((end_era, canon_id)) = end {
			if let Some(ref mut records) = journal_overlay.journal.get_mut(&end_era) {
				let mut canon_insertions: Vec<(H256, Bytes)> = Vec::new();
				let mut canon_deletions: Vec<H256> = Vec::new();
				let mut overlay_deletions: Vec<H256> = Vec::new();
				let mut index = 0usize;
				for mut journal in records.drain(..) {
					//delete the record from the db
					let mut r = RlpStream::new_list(3);
					r.append(&end_era);
					r.append(&index);
					r.append(&&PADDING[..]);
					try!(batch.delete(&r.drain()));
					trace!("commit: Delete journal for time #{}.{}: {}, (canon was {}): +{} -{} entries", end_era, index, journal.id, canon_id, journal.insertions.len(), journal.deletions.len());
					{
						if canon_id == journal.id {
							for h in &journal.insertions {
								if let Some(&(ref d, rc)) = journal_overlay.backing_overlay.raw(h) {
									if rc > 0 {
										canon_insertions.push((h.clone(), d.clone())); //TODO: optimize this to avoid data copy
									}
								}
							}
							canon_deletions = journal.deletions;
						}
						overlay_deletions.append(&mut journal.insertions);
					}
					index += 1;
				}
				// apply canon inserts first
				for (k, v) in canon_insertions {
					try!(batch.put(&k, &v));
				}
				// update the overlay
				for k in overlay_deletions {
					journal_overlay.backing_overlay.kill(&k);
				}
				// apply canon deletions
				for k in canon_deletions {
					if !journal_overlay.backing_overlay.exists(&k) {
						try!(batch.delete(&k));
					}
				}
				journal_overlay.backing_overlay.purge();
			}
			journal_overlay.journal.remove(&end_era);
		}
		try!(self.backing.write(batch));
		Ok(0)
	}

}

impl HashDB for OverlayRecentDB {
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
				let v = self.journal_overlay.read().unwrap().backing_overlay.lookup(key).map(|v| v.to_vec());
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
	#![cfg_attr(feature="dev", allow(blacklisted_name))]
	#![cfg_attr(feature="dev", allow(similar_names))]

	use common::*;
	use super::*;
	use hashdb::*;
	use log::init_log;
	use journaldb::JournalDB;

	#[test]
	fn insert_same_in_fork() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let x = jdb.insert(b"X");
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(2, &b"2".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(3, &b"1002a".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(4, &b"1003a".sha3(), Some((2, b"2".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&x);
		jdb.commit(3, &b"1002b".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		let x = jdb.insert(b"X");
		jdb.commit(4, &b"1003b".sha3(), Some((2, b"2".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit(5, &b"1004a".sha3(), Some((3, b"1002a".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(6, &b"1005a".sha3(), Some((4, b"1003a".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.exists(&x));
	}

	#[test]
	fn long_history() {
		// history is 3
		let mut jdb = OverlayRecentDB::new_temp();
		let h = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&h));
		jdb.remove(&h);
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&h));
		jdb.commit(2, &b"2".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&h));
		jdb.commit(3, &b"3".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&h));
		jdb.commit(4, &b"4".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.exists(&h));
	}

	#[test]
	fn complex() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let foo = jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));

		jdb.remove(&foo);
		jdb.remove(&bar);
		let baz = jdb.insert(b"baz");
		jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));
		assert!(jdb.exists(&baz));

		let foo = jdb.insert(b"foo");
		jdb.remove(&baz);
		jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
		assert!(!jdb.exists(&bar));
		assert!(jdb.exists(&baz));

		jdb.remove(&foo);
		jdb.commit(3, &b"3".sha3(), Some((2, b"2".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
		assert!(!jdb.exists(&bar));
		assert!(!jdb.exists(&baz));

		jdb.commit(4, &b"4".sha3(), Some((3, b"3".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.exists(&foo));
		assert!(!jdb.exists(&bar));
		assert!(!jdb.exists(&baz));
	}

	#[test]
	fn fork() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let foo = jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));

		jdb.remove(&foo);
		let baz = jdb.insert(b"baz");
		jdb.commit(1, &b"1a".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&bar);
		jdb.commit(1, &b"1b".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));
		assert!(jdb.exists(&baz));

		jdb.commit(2, &b"2b".sha3(), Some((1, b"1b".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
		assert!(!jdb.exists(&baz));
		assert!(!jdb.exists(&bar));
	}

	#[test]
	fn overwrite() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let foo = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));

		jdb.remove(&foo);
		jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		assert!(jdb.exists(&foo));
		jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
		jdb.commit(3, &b"2".sha3(), Some((0, b"2".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
	}

	#[test]
	fn fork_same_key_one() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(b"foo");
		jdb.commit(1, &b"1a".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(1, &b"1b".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(1, &b"1c".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.exists(&foo));

		jdb.commit(2, &b"2a".sha3(), Some((1, b"1a".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
	}

	#[test]
	fn fork_same_key_other() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(b"foo");
		jdb.commit(1, &b"1a".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(1, &b"1b".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(1, &b"1c".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.exists(&foo));

		jdb.commit(2, &b"2b".sha3(), Some((1, b"1b".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));
	}

	#[test]
	fn fork_ins_del_ins() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(b"foo");
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit(2, &b"2a".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit(2, &b"2b".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(3, &b"3a".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(3, &b"3b".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit(4, &b"4a".sha3(), Some((2, b"2a".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit(5, &b"5a".sha3(), Some((3, b"3a".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn reopen() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let bar = H256::random();

		let foo = {
			let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			jdb.emplace(bar.clone(), b"bar".to_vec());
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			foo
		};

		{
			let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
			jdb.remove(&foo);
			jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
		}

		{
			let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
			assert!(jdb.exists(&foo));
			assert!(jdb.exists(&bar));
			jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(!jdb.exists(&foo));
		}
	}

	#[test]
	fn insert_delete_insert_delete_insert_expunge() {
		init_log();
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());

		// history is 4
		let foo = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo);
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		jdb.commit(2, &b"2".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo);
		jdb.commit(3, &b"3".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		jdb.commit(4, &b"4".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		// expunge foo
		jdb.commit(5, &b"5".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn forked_insert_delete_insert_delete_insert_expunge() {
		init_log();
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());

		// history is 4
		let foo = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit(1, &b"1a".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit(1, &b"1b".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(2, &b"2a".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(2, &b"2b".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit(3, &b"3a".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit(3, &b"3b".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(4, &b"4a".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(4, &b"4b".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// expunge foo
		jdb.commit(5, &b"5".sha3(), Some((1, b"1a".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn broken_assert() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
		// history is 1
		let foo = jdb.insert(b"foo");
		jdb.commit(1, &b"1".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// foo is ancient history.

		jdb.remove(&foo);
		jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit(3, &b"3".sha3(), Some((2, b"2".sha3()))).unwrap();	// BROKEN
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.exists(&foo));

		jdb.remove(&foo);
		jdb.commit(4, &b"4".sha3(), Some((3, b"3".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit(5, &b"5".sha3(), Some((4, b"4".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.exists(&foo));
	}

	#[test]
	fn reopen_test() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
		// history is 4
		let foo = jdb.insert(b"foo");
		jdb.commit(0, &b"0".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(1, &b"1".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(2, &b"2".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(3, &b"3".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(4, &b"4".sha3(), Some((0, b"0".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// foo is ancient history.

		jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit(5, &b"5".sha3(), Some((1, b"1".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo);
		jdb.remove(&bar);
		jdb.commit(6, &b"6".sha3(), Some((2, b"2".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		jdb.insert(b"bar");
		jdb.commit(7, &b"7".sha3(), Some((3, b"3".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn reopen_remove_three() {
		init_log();

		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let foo = b"foo".sha3();

		{
			let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
			// history is 1
			jdb.insert(b"foo");
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			jdb.commit(1, &b"1".sha3(), None).unwrap();
			assert!(jdb.can_reconstruct_refs());

			// foo is ancient history.

			jdb.remove(&foo);
			jdb.commit(2, &b"2".sha3(), Some((0, b"0".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.exists(&foo));

			jdb.insert(b"foo");
			jdb.commit(3, &b"3".sha3(), Some((1, b"1".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.exists(&foo));

		// incantation to reopen the db
		}; { let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());

			jdb.remove(&foo);
			jdb.commit(4, &b"4".sha3(), Some((2, b"2".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.exists(&foo));

		// incantation to reopen the db
		}; { let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());

			jdb.commit(5, &b"5".sha3(), Some((3, b"3".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.exists(&foo));

		// incantation to reopen the db
		}; { let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());

			jdb.commit(6, &b"6".sha3(), Some((4, b"4".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(!jdb.exists(&foo));
		}
	}

	#[test]
	fn reopen_fork() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let (foo, bar, baz) = {
			let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
			// history is 1
			let foo = jdb.insert(b"foo");
			let bar = jdb.insert(b"bar");
			jdb.commit(0, &b"0".sha3(), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			jdb.remove(&foo);
			let baz = jdb.insert(b"baz");
			jdb.commit(1, &b"1a".sha3(), Some((0, b"0".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());

			jdb.remove(&bar);
			jdb.commit(1, &b"1b".sha3(), Some((0, b"0".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			(foo, bar, baz)
		};

		{
			let mut jdb = OverlayRecentDB::new(dir.to_str().unwrap());
			jdb.commit(2, &b"2b".sha3(), Some((1, b"1b".sha3()))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.exists(&foo));
			assert!(!jdb.exists(&baz));
			assert!(!jdb.exists(&bar));
		}
	}

	#[test]
	fn insert_older_era() {
		let mut jdb = OverlayRecentDB::new_temp();
		let foo = jdb.insert(b"foo");
		jdb.commit(0, &b"0a".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let bar = jdb.insert(b"bar");
		jdb.commit(1, &b"1".sha3(), Some((0, b"0a".sha3()))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&bar);
		jdb.commit(0, &b"0b".sha3(), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit(2, &b"2".sha3(), Some((1, b"1".sha3()))).unwrap();

		assert!(jdb.exists(&foo));
		assert!(jdb.exists(&bar));
	}
}
