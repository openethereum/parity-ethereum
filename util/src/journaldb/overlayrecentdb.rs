// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use parking_lot::RwLock;
use heapsize::HeapSizeOf;
use rlp::*;
use hashdb::*;
use memorydb::*;
use super::{DB_PREFIX_LEN, LATEST_ERA_KEY};
use kvdb::{KeyValueDB, DBTransaction};
use super::JournalDB;
use bigint::hash::{H256, H256FastMap};
use error::{BaseDataError, UtilError};
use bytes::Bytes;

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
/// 2. Insert each node from the transaction overlay into the History overlay increasing reference
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
	backing: Arc<KeyValueDB>,
	journal_overlay: Arc<RwLock<JournalOverlay>>,
	column: Option<u32>,
}

#[derive(PartialEq)]
struct JournalOverlay {
	backing_overlay: MemoryDB, // Nodes added in the history period
	pending_overlay: H256FastMap<DBValue>, // Nodes being transfered from backing_overlay to backing db
	journal: HashMap<u64, Vec<JournalEntry>>,
	latest_era: Option<u64>,
	earliest_era: Option<u64>,
	cumulative_size: usize, // cumulative size of all entries.
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
			column: self.column.clone(),
		}
	}
}

const PADDING : [u8; 10] = [ 0u8; 10 ];

impl OverlayRecentDB {
	/// Create a new instance.
	pub fn new(backing: Arc<KeyValueDB>, col: Option<u32>) -> OverlayRecentDB {
		let journal_overlay = Arc::new(RwLock::new(OverlayRecentDB::read_overlay(&*backing, col)));
		OverlayRecentDB {
			transaction_overlay: MemoryDB::new(),
			backing: backing,
			journal_overlay: journal_overlay,
			column: col,
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> OverlayRecentDB {
		let backing = Arc::new(::kvdb::in_memory(0));
		Self::new(backing, None)
	}

	#[cfg(test)]
	fn can_reconstruct_refs(&self) -> bool {
		let reconstructed = Self::read_overlay(&*self.backing, self.column);
		let journal_overlay = self.journal_overlay.read();
		journal_overlay.backing_overlay == reconstructed.backing_overlay &&
		journal_overlay.pending_overlay == reconstructed.pending_overlay &&
		journal_overlay.journal == reconstructed.journal &&
		journal_overlay.latest_era == reconstructed.latest_era &&
		journal_overlay.cumulative_size == reconstructed.cumulative_size
	}

	fn payload(&self, key: &H256) -> Option<DBValue> {
		self.backing.get(self.column, key).expect("Low-level database error. Some issue with your hard disk?")
	}

	fn read_overlay(db: &KeyValueDB, col: Option<u32>) -> JournalOverlay {
		let mut journal = HashMap::new();
		let mut overlay = MemoryDB::new();
		let mut count = 0;
		let mut latest_era = None;
		let mut earliest_era = None;
		let mut cumulative_size = 0;
		if let Some(val) = db.get(col, &LATEST_ERA_KEY).expect("Low-level database error.") {
			let mut era = decode::<u64>(&val);
			latest_era = Some(era);
			loop {
				let mut index = 0usize;
				while let Some(rlp_data) = db.get(col, {
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
					let deletions: Vec<H256> = rlp.list_at(2);
					let mut inserted_keys = Vec::new();
					for r in insertions.iter() {
						let k: H256 = r.val_at(0);
						let v = r.at(1).data();

						let short_key = to_short_key(&k);

						if !overlay.contains(&short_key) {
							cumulative_size += v.len();
						}

						overlay.emplace(short_key, DBValue::from_slice(v));
						inserted_keys.push(k);
						count += 1;
					}
					journal.entry(era).or_insert_with(Vec::new).push(JournalEntry {
						id: id,
						insertions: inserted_keys,
						deletions: deletions,
					});
					index += 1;
					earliest_era = Some(era);
				};
				if index == 0 || era == 0 {
					break;
				}
				era -= 1;
			}
		}
		trace!("Recovered {} overlay entries, {} journal entries", count, journal.len());
		JournalOverlay {
			backing_overlay: overlay,
			pending_overlay: HashMap::default(),
			journal: journal,
			latest_era: latest_era,
			earliest_era: earliest_era,
			cumulative_size: cumulative_size,
		}
	}


}

#[inline]
fn to_short_key(key: &H256) -> H256 {
	let mut k = H256::new();
	k[0..DB_PREFIX_LEN].copy_from_slice(&key[0..DB_PREFIX_LEN]);
	k
}

impl JournalDB for OverlayRecentDB {
	fn boxed_clone(&self) -> Box<JournalDB> {
		Box::new(self.clone())
	}

	fn mem_used(&self) -> usize {
		let mut mem = self.transaction_overlay.mem_used();
		let overlay = self.journal_overlay.read();

		mem += overlay.backing_overlay.mem_used();
		mem += overlay.pending_overlay.heap_size_of_children();
		mem += overlay.journal.heap_size_of_children();

		mem
	}

	fn journal_size(&self) -> usize {
		self.journal_overlay.read().cumulative_size

	}

	fn is_empty(&self) -> bool {
		self.backing.get(self.column, &LATEST_ERA_KEY).expect("Low level database error").is_none()
	}

	fn backing(&self) -> &Arc<KeyValueDB> {
		&self.backing
	}

	fn latest_era(&self) -> Option<u64> { self.journal_overlay.read().latest_era }

	fn earliest_era(&self) -> Option<u64> { self.journal_overlay.read().earliest_era }

	fn state(&self, key: &H256) -> Option<Bytes> {
		let journal_overlay = self.journal_overlay.read();
		let key = to_short_key(key);
		journal_overlay.backing_overlay.get(&key).map(|v| v.into_vec())
		.or_else(|| journal_overlay.pending_overlay.get(&key).map(|d| d.clone().into_vec()))
		.or_else(|| self.backing.get_by_prefix(self.column, &key[0..DB_PREFIX_LEN]).map(|b| b.into_vec()))
	}

	fn journal_under(&mut self, batch: &mut DBTransaction, now: u64, id: &H256) -> Result<u32, UtilError> {
		trace!(target: "journaldb", "entry: #{} ({})", now, id);

		let mut journal_overlay = self.journal_overlay.write();

		// flush previous changes
		journal_overlay.pending_overlay.clear();

		let mut r = RlpStream::new_list(3);
		let mut tx = self.transaction_overlay.drain();
		let inserted_keys: Vec<_> = tx.iter().filter_map(|(k, &(_, c))| if c > 0 { Some(k.clone()) } else { None }).collect();
		let removed_keys: Vec<_> = tx.iter().filter_map(|(k, &(_, c))| if c < 0 { Some(k.clone()) } else { None }).collect();
		let ops = inserted_keys.len() + removed_keys.len();

		// Increase counter for each inserted key no matter if the block is canonical or not.
		let insertions = tx.drain().filter_map(|(k, (v, c))| if c > 0 { Some((k, v)) } else { None });

		r.append(id);
		r.begin_list(inserted_keys.len());
		for (k, v) in insertions {
			r.begin_list(2);
			r.append(&k);
			r.append(&&*v);

			let short_key = to_short_key(&k);
			if !journal_overlay.backing_overlay.contains(&short_key) {
				journal_overlay.cumulative_size += v.len();
			}

			journal_overlay.backing_overlay.emplace(short_key, v);
		}
		r.append_list(&removed_keys);

		let mut k = RlpStream::new_list(3);
		let index = journal_overlay.journal.get(&now).map_or(0, |j| j.len());
		k.append(&now);
		k.append(&index);
		k.append(&&PADDING[..]);
		batch.put_vec(self.column, &k.drain(), r.out());
		if journal_overlay.latest_era.map_or(true, |e| now > e) {
			trace!(target: "journaldb", "Set latest era to {}", now);
			batch.put_vec(self.column, &LATEST_ERA_KEY, encode(&now).into_vec());
			journal_overlay.latest_era = Some(now);
		}

		if journal_overlay.earliest_era.map_or(true, |e| e > now) {
			trace!(target: "journaldb", "Set earliest era to {}", now);
			journal_overlay.earliest_era = Some(now);
		}

		journal_overlay.journal.entry(now).or_insert_with(Vec::new).push(JournalEntry { id: id.clone(), insertions: inserted_keys, deletions: removed_keys });
		Ok(ops as u32)
	}

	fn mark_canonical(&mut self, batch: &mut DBTransaction, end_era: u64, canon_id: &H256) -> Result<u32, UtilError> {
		trace!(target: "journaldb", "canonical: #{} ({})", end_era, canon_id);

		let mut journal_overlay = self.journal_overlay.write();
		let journal_overlay = &mut *journal_overlay;

		let mut ops = 0;
		// apply old commits' details
		if let Some(ref mut records) = journal_overlay.journal.get_mut(&end_era) {
			let mut canon_insertions: Vec<(H256, DBValue)> = Vec::new();
			let mut canon_deletions: Vec<H256> = Vec::new();
			let mut overlay_deletions: Vec<H256> = Vec::new();
			let mut index = 0usize;
			for mut journal in records.drain(..) {
				//delete the record from the db
				let mut r = RlpStream::new_list(3);
				r.append(&end_era);
				r.append(&index);
				r.append(&&PADDING[..]);
				batch.delete(self.column, &r.drain());
				trace!(target: "journaldb", "Delete journal for time #{}.{}: {}, (canon was {}): +{} -{} entries", end_era, index, journal.id, canon_id, journal.insertions.len(), journal.deletions.len());
				{
					if *canon_id == journal.id {
						for h in &journal.insertions {
							if let Some((d, rc)) = journal_overlay.backing_overlay.raw(&to_short_key(h)) {
								if rc > 0 {
									canon_insertions.push((h.clone(), d)); //TODO: optimize this to avoid data copy
								}
							}
						}
						canon_deletions = journal.deletions;
					}
					overlay_deletions.append(&mut journal.insertions);
				}
				index += 1;
			}

			ops += canon_insertions.len();
			ops += canon_deletions.len();

			// apply canon inserts first
			for (k, v) in canon_insertions {
				batch.put(self.column, &k, &v);
				journal_overlay.pending_overlay.insert(to_short_key(&k), v);
			}
			// update the overlay
			for k in overlay_deletions {
				if let Some(val) = journal_overlay.backing_overlay.remove_and_purge(&to_short_key(&k)) {
					journal_overlay.cumulative_size -= val.len();
				}
			}
			// apply canon deletions
			for k in canon_deletions {
				if !journal_overlay.backing_overlay.contains(&to_short_key(&k)) {
					batch.delete(self.column, &k);
				}
			}
		}
		journal_overlay.journal.remove(&end_era);

		if !journal_overlay.journal.is_empty() {
			trace!(target: "journaldb", "Set earliest_era to {}", end_era + 1);
			journal_overlay.earliest_era = Some(end_era + 1);
		}

		Ok(ops as u32)
	}

	fn flush(&self) {
		self.journal_overlay.write().pending_overlay.clear();
	}

	fn inject(&mut self, batch: &mut DBTransaction) -> Result<u32, UtilError> {
		let mut ops = 0;
		for (key, (value, rc)) in self.transaction_overlay.drain() {
			if rc != 0 { ops += 1 }

			match rc {
				0 => {}
				_ if rc > 0 => {
					batch.put(self.column, &key, &value)
				}
				-1 => {
					if cfg!(debug_assertions) && self.backing.get(self.column, &key)?.is_none() {
						return Err(BaseDataError::NegativelyReferencedHash(key).into());
					}
					batch.delete(self.column, &key)
				}
				_ => panic!("Attempted to inject invalid state ({})", rc),
			}
		}

		Ok(ops)
	}

	fn consolidate(&mut self, with: MemoryDB) {
		self.transaction_overlay.consolidate(with);
	}
}

impl HashDB for OverlayRecentDB {
	fn keys(&self) -> HashMap<H256, i32> {
		let mut ret: HashMap<H256, i32> = self.backing.iter(self.column)
			.map(|(key, _)| (H256::from_slice(&*key), 1))
			.collect();

		for (key, refs) in self.transaction_overlay.keys() {
			match ret.entry(key) {
				Entry::Occupied(mut entry) => {
					*entry.get_mut() += refs;
				},
				Entry::Vacant(entry) => {
					entry.insert(refs);
				}
			}
		}
		ret
	}

	fn get(&self, key: &H256) -> Option<DBValue> {
		if let Some((d, rc)) = self.transaction_overlay.raw(key) {
			if rc > 0 {
				return Some(d)
			}
		}
		let v = {
			let journal_overlay = self.journal_overlay.read();
			let key = to_short_key(key);
			journal_overlay.backing_overlay.get(&key)
				.or_else(|| journal_overlay.pending_overlay.get(&key).cloned())
		};
		v.or_else(|| self.payload(key))
	}

	fn contains(&self, key: &H256) -> bool {
		self.get(key).is_some()
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		self.transaction_overlay.insert(value)
	}
	fn emplace(&mut self, key: H256, value: DBValue) {
		self.transaction_overlay.emplace(key, value);
	}
	fn remove(&mut self, key: &H256) {
		self.transaction_overlay.remove(key);
	}
}

#[cfg(test)]
mod tests {
	#![cfg_attr(feature="dev", allow(blacklisted_name))]
	#![cfg_attr(feature="dev", allow(similar_names))]

	use std::path::Path;
	use keccak::keccak;
	use super::*;
	use hashdb::{HashDB, DBValue};
	use ethcore_logger::init_log;
	use journaldb::JournalDB;
	use kvdb::Database;
	use bigint::hash::H32;

	fn new_db(path: &Path) -> OverlayRecentDB {
		let backing = Arc::new(Database::open_default(path.to_str().unwrap()).unwrap());
		OverlayRecentDB::new(backing, None)
	}

	#[test]
	fn insert_same_in_fork() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let x = jdb.insert(b"X");
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(3, &keccak(b"1002a"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(4, &keccak(b"1003a"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&x);
		jdb.commit_batch(3, &keccak(b"1002b"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		let x = jdb.insert(b"X");
		jdb.commit_batch(4, &keccak(b"1003b"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(5, &keccak(b"1004a"), Some((3, keccak(b"1002a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(6, &keccak(b"1005a"), Some((4, keccak(b"1003a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&x));
	}

	#[test]
	fn long_history() {
		// history is 3
		let mut jdb = OverlayRecentDB::new_temp();
		let h = jdb.insert(b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h));
		jdb.remove(&h);
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h));
		jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h));
		jdb.commit_batch(3, &keccak(b"3"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h));
		jdb.commit_batch(4, &keccak(b"4"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.contains(&h));
	}

	#[test]
	fn complex() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let foo = jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
		assert!(jdb.contains(&bar));

		jdb.remove(&foo);
		jdb.remove(&bar);
		let baz = jdb.insert(b"baz");
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
		assert!(jdb.contains(&bar));
		assert!(jdb.contains(&baz));

		let foo = jdb.insert(b"foo");
		jdb.remove(&baz);
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
		assert!(!jdb.contains(&bar));
		assert!(jdb.contains(&baz));

		jdb.remove(&foo);
		jdb.commit_batch(3, &keccak(b"3"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
		assert!(!jdb.contains(&bar));
		assert!(!jdb.contains(&baz));

		jdb.commit_batch(4, &keccak(b"4"), Some((3, keccak(b"3")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.contains(&foo));
		assert!(!jdb.contains(&bar));
		assert!(!jdb.contains(&baz));
	}

	#[test]
	fn fork() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let foo = jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
		assert!(jdb.contains(&bar));

		jdb.remove(&foo);
		let baz = jdb.insert(b"baz");
		jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&bar);
		jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&foo));
		assert!(jdb.contains(&bar));
		assert!(jdb.contains(&baz));

		jdb.commit_batch(2, &keccak(b"2b"), Some((1, keccak(b"1b")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
		assert!(!jdb.contains(&baz));
		assert!(!jdb.contains(&bar));
	}

	#[test]
	fn overwrite() {
		// history is 1
		let mut jdb = OverlayRecentDB::new_temp();

		let foo = jdb.insert(b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));

		jdb.remove(&foo);
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		assert!(jdb.contains(&foo));
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
		jdb.commit_batch(3, &keccak(b"2"), Some((0, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
	}

	#[test]
	fn fork_same_key_one() {
		let mut jdb = OverlayRecentDB::new_temp();
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1c"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&foo));

		jdb.commit_batch(2, &keccak(b"2a"), Some((1, keccak(b"1a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
	}

	#[test]
	fn fork_same_key_other() {
		let mut jdb = OverlayRecentDB::new_temp();

		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1c"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&foo));

		jdb.commit_batch(2, &keccak(b"2b"), Some((1, keccak(b"1b")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));
	}

	#[test]
	fn fork_ins_del_ins() {
		let mut jdb = OverlayRecentDB::new_temp();

		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit_batch(2, &keccak(b"2a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit_batch(2, &keccak(b"2b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(3, &keccak(b"3a"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(3, &keccak(b"3b"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(4, &keccak(b"4a"), Some((2, keccak(b"2a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(5, &keccak(b"5a"), Some((3, keccak(b"3a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn reopen() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let bar = H256::random();

		let foo = {
			let mut jdb = new_db(&dir);
			// history is 1
			let foo = jdb.insert(b"foo");
			jdb.emplace(bar.clone(), DBValue::from_slice(b"bar"));
			jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			foo
		};

		{
			let mut jdb = new_db(&dir);
			jdb.remove(&foo);
			jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
		}

		{
			let mut jdb = new_db(&dir);
			assert!(jdb.contains(&foo));
			assert!(jdb.contains(&bar));
			jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(!jdb.contains(&foo));
		}
	}

	#[test]
	fn insert_delete_insert_delete_insert_expunge() {
		init_log();
		let mut jdb = OverlayRecentDB::new_temp();

		// history is 4
		let foo = jdb.insert(b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo);
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo);
		jdb.commit_batch(3, &keccak(b"3"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		jdb.commit_batch(4, &keccak(b"4"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		// expunge foo
		jdb.commit_batch(5, &keccak(b"5"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn forked_insert_delete_insert_delete_insert_expunge() {
		init_log();
		let mut jdb = OverlayRecentDB::new_temp();

		// history is 4
		let foo = jdb.insert(b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit_batch(1, &keccak(b"1a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit_batch(1, &keccak(b"1b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(2, &keccak(b"2a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(2, &keccak(b"2b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit_batch(3, &keccak(b"3a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo);
		jdb.commit_batch(3, &keccak(b"3b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(4, &keccak(b"4a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(4, &keccak(b"4b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// expunge foo
		jdb.commit_batch(5, &keccak(b"5"), Some((1, keccak(b"1a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn broken_assert() {
		let mut jdb = OverlayRecentDB::new_temp();

		let foo = jdb.insert(b"foo");
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// foo is ancient history.

		jdb.remove(&foo);
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(b"foo");
		jdb.commit_batch(3, &keccak(b"3"), Some((2, keccak(b"2")))).unwrap();	// BROKEN
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo));

		jdb.remove(&foo);
		jdb.commit_batch(4, &keccak(b"4"), Some((3, keccak(b"3")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(5, &keccak(b"5"), Some((4, keccak(b"4")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.contains(&foo));
	}

	#[test]
	fn reopen_test() {
		let mut jdb = OverlayRecentDB::new_temp();
		// history is 4
		let foo = jdb.insert(b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(3, &keccak(b"3"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(4, &keccak(b"4"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// foo is ancient history.

		jdb.insert(b"foo");
		let bar = jdb.insert(b"bar");
		jdb.commit_batch(5, &keccak(b"5"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo);
		jdb.remove(&bar);
		jdb.commit_batch(6, &keccak(b"6"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(b"foo");
		jdb.insert(b"bar");
		jdb.commit_batch(7, &keccak(b"7"), Some((3, keccak(b"3")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn reopen_remove_three() {
		init_log();

		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());

		let foo = keccak(b"foo");

		{
			let mut jdb = new_db(&dir);
			// history is 1
			jdb.insert(b"foo");
			jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());

			// foo is ancient history.

			jdb.remove(&foo);
			jdb.commit_batch(2, &keccak(b"2"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo));

			jdb.insert(b"foo");
			jdb.commit_batch(3, &keccak(b"3"), Some((1, keccak(b"1")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo));

		// incantation to reopen the db
		}; {
			let mut jdb = new_db(&dir);

			jdb.remove(&foo);
			jdb.commit_batch(4, &keccak(b"4"), Some((2, keccak(b"2")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo));

		// incantation to reopen the db
		}; {
			let mut jdb = new_db(&dir);

			jdb.commit_batch(5, &keccak(b"5"), Some((3, keccak(b"3")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo));

		// incantation to reopen the db
		}; {
			let mut jdb = new_db(&dir);

			jdb.commit_batch(6, &keccak(b"6"), Some((4, keccak(b"4")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(!jdb.contains(&foo));
		}
	}

	#[test]
	fn reopen_fork() {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		let (foo, bar, baz) = {
			let mut jdb = new_db(&dir);
			// history is 1
			let foo = jdb.insert(b"foo");
			let bar = jdb.insert(b"bar");
			jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			jdb.remove(&foo);
			let baz = jdb.insert(b"baz");
			jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());

			jdb.remove(&bar);
			jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			(foo, bar, baz)
		};

		{
			let mut jdb = new_db(&dir);
			jdb.commit_batch(2, &keccak(b"2b"), Some((1, keccak(b"1b")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo));
			assert!(!jdb.contains(&baz));
			assert!(!jdb.contains(&bar));
		}
	}

	#[test]
	fn insert_older_era() {
		let mut jdb = OverlayRecentDB::new_temp();
		let foo = jdb.insert(b"foo");
		jdb.commit_batch(0, &keccak(b"0a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let bar = jdb.insert(b"bar");
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&bar);
		jdb.commit_batch(0, &keccak(b"0b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();

		assert!(jdb.contains(&foo));
		assert!(jdb.contains(&bar));
	}

	#[test]
	fn inject() {
		let temp = ::devtools::RandomTempPath::new();

		let mut jdb = new_db(temp.as_path().as_path());
		let key = jdb.insert(b"dog");
		jdb.inject_batch().unwrap();

		assert_eq!(jdb.get(&key).unwrap(), DBValue::from_slice(b"dog"));
		jdb.remove(&key);
		jdb.inject_batch().unwrap();

		assert!(jdb.get(&key).is_none());
	}

	#[test]
	fn earliest_era() {
		let temp = ::devtools::RandomTempPath::new();

		// empty DB
		let mut jdb = new_db(temp.as_path().as_path());
		assert!(jdb.earliest_era().is_none());

		// single journalled era.
		let _key = jdb.insert(b"hello!");
		let mut batch = jdb.backing().transaction();
		jdb.journal_under(&mut batch, 0, &keccak(b"0")).unwrap();
		jdb.backing().write_buffered(batch);

		assert_eq!(jdb.earliest_era(), Some(0));

		// second journalled era.
		let mut batch = jdb.backing().transaction();
		jdb.journal_under(&mut batch, 1, &keccak(b"1")).unwrap();
		jdb.backing().write_buffered(batch);

		assert_eq!(jdb.earliest_era(), Some(0));

		// single journalled era.
		let mut batch = jdb.backing().transaction();
		jdb.mark_canonical(&mut batch, 0, &keccak(b"0")).unwrap();
		jdb.backing().write_buffered(batch);

		assert_eq!(jdb.earliest_era(), Some(1));

		// no journalled eras.
		let mut batch = jdb.backing().transaction();
		jdb.mark_canonical(&mut batch, 1, &keccak(b"1")).unwrap();
		jdb.backing().write_buffered(batch);

		assert_eq!(jdb.earliest_era(), Some(1));

		// reconstructed: no journal entries.
		drop(jdb);
		let jdb = new_db(temp.as_path().as_path());
		assert_eq!(jdb.earliest_era(), None);
	}
}
