// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Disk-backed `HashDB` implementation.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io;
use std::sync::Arc;

use bytes::Bytes;
use ethereum_types::H256;
use hash_db::{HashDB, Prefix};
use parity_util_mem::{MallocSizeOf, allocators::new_malloc_size_ops};
use keccak_hasher::KeccakHasher;
use kvdb::{KeyValueDB, DBTransaction, DBValue};
use parking_lot::RwLock;
use rlp::{encode, decode};
use super::{DB_PREFIX_LEN, LATEST_ERA_KEY, error_negatively_reference_hash, error_key_already_exists};
use super::traits::JournalDB;
use util::{DatabaseKey, DatabaseValueView, DatabaseValueRef};

#[derive(Debug, Clone, PartialEq, Eq, MallocSizeOf)]
struct RefInfo {
	queue_refs: usize,
	in_archive: bool,
}

#[derive(Clone, PartialEq, Eq)]
enum RemoveFrom {
	Queue,
	Archive,
}

/// Implementation of the `HashDB` trait for a disk-backed database with a memory overlay
/// and latent-removal semantics.
///
/// Like `OverlayDB`, there is a memory overlay; `commit()` must be called in order to
/// write operations out to disk. Unlike `OverlayDB`, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
///
/// journal format:
/// ```text
/// [era, 0] => [ id, [insert_0, ...], [remove_0, ...] ]
/// [era, 1] => [ id, [insert_0, ...], [remove_0, ...] ]
/// [era, n] => [ ... ]
/// ```
///
/// When we make a new commit, we make a journal of all blocks in the recent history and record
/// all keys that were inserted and deleted. The journal is ordered by era; multiple commits can
/// share the same era. This forms a data structure similar to a queue but whose items are tuples.
/// By the time comes to remove a tuple from the queue (i.e. then the era passes from recent history
/// into ancient history) then only one commit from the tuple is considered canonical. This commit
/// is kept in the main backing database, whereas any others from the same era are reverted.
///
/// It is possible that a key, properly available in the backing database be deleted and re-inserted
/// in the recent history queue, yet have both operations in commits that are eventually non-canonical.
/// To avoid the original, and still required, key from being deleted, we maintain a reference count
/// which includes an original key, if any.
///
/// The semantics of the `counter` are:
/// ```text
/// insert key k:
///   counter already contains k: count += 1
///   counter doesn't contain k:
///     backing db contains k: count = 1
///     backing db doesn't contain k: insert into backing db, count = 0
/// delete key k:
///   counter contains k (count is asserted to be non-zero):
///     count > 1: counter -= 1
///     count == 1: remove counter
///     count == 0: remove key from backing db
///   counter doesn't contain k: remove key from backing db
/// ```
///
/// Practically, this means that for each commit block turning from recent to ancient we do the
/// following:
/// ```text
/// is_canonical:
///   inserts: Ignored (left alone in the backing database).
///   deletes: Enacted; however, recent history queue is checked for ongoing references. This is
///            reduced as a preference to deletion from the backing database.
/// !is_canonical:
///   inserts: Reverted; however, recent history queue is checked for ongoing references. This is
///            reduced as a preference to deletion from the backing database.
///   deletes: Ignored (they were never inserted).
/// ```
///
/// TODO: `store_reclaim_period`
pub struct EarlyMergeDB {
	overlay: super::MemoryDB,
	backing: Arc<dyn KeyValueDB>,
	refs: Option<Arc<RwLock<HashMap<H256, RefInfo>>>>,
	latest_era: Option<u64>,
	column: Option<u32>,
}

impl EarlyMergeDB {
	/// Create a new instance from file
	pub fn new(backing: Arc<dyn KeyValueDB>, col: Option<u32>) -> EarlyMergeDB {
		let (latest_era, refs) = EarlyMergeDB::read_refs(&*backing, col);
		let refs = Some(Arc::new(RwLock::new(refs)));
		EarlyMergeDB {
			overlay: ::new_memory_db(),
			backing: backing,
			refs: refs,
			latest_era: latest_era,
			column: col,
		}
	}

	fn morph_key(key: &H256, index: u8) -> Bytes {
		let mut ret = key.as_bytes().to_owned();
		ret.push(index);
		ret
	}

	// The next three are valid only as long as there is an insert operation of `key` in the journal.
	fn set_already_in(batch: &mut DBTransaction, col: Option<u32>, key: &H256) { batch.put(col, &Self::morph_key(key, 0), &[1u8]); }
	fn reset_already_in(batch: &mut DBTransaction, col: Option<u32>, key: &H256) { batch.delete(col, &Self::morph_key(key, 0)); }
	fn is_already_in(backing: &dyn KeyValueDB, col: Option<u32>, key: &H256) -> bool {
		backing.get(col, &Self::morph_key(key, 0)).expect("Low-level database error. Some issue with your hard disk?").is_some()
	}

	fn insert_keys(inserts: &[(H256, DBValue)], backing: &dyn KeyValueDB, col: Option<u32>, refs: &mut HashMap<H256, RefInfo>, batch: &mut DBTransaction) {
		for &(ref h, ref d) in inserts {
			match refs.entry(*h) {
				Entry::Occupied(mut entry) => {
					let info = entry.get_mut();
					// already counting. increment.
					info.queue_refs += 1;
					trace!(target: "jdb.fine", "    insert({}): In queue: Incrementing refs to {}", h, info.queue_refs);
				},
				Entry::Vacant(entry) => {
					// this is the first entry for this node in the journal.
					let in_archive = backing.get(col, h.as_bytes())
						.expect("Low-level database error. Some issue with your hard disk?").is_some();
					if in_archive {
						// already in the backing DB. start counting, and remember it was already in.
						Self::set_already_in(batch, col, h);
						trace!(target: "jdb.fine", "    insert({}): New to queue, in DB: Recording and inserting into queue", h);
					} else {
						// Gets removed when a key leaves the journal, so should never be set when we're placing a new key.
						//Self::reset_already_in(&h);
						assert!(!Self::is_already_in(backing, col, h));
						trace!(target: "jdb.fine", "    insert({}): New to queue, not in DB: Inserting into queue and DB", h);
						batch.put(col, h.as_bytes(), d);
					}
					entry.insert(RefInfo {
						queue_refs: 1,
						in_archive: in_archive,
					});
				},
			}
		}
	}

	fn replay_keys(inserts: &[H256], backing: &dyn KeyValueDB, col: Option<u32>, refs: &mut HashMap<H256, RefInfo>) {
		trace!(target: "jdb.fine", "replay_keys: inserts={:?}, refs={:?}", inserts, refs);
		for h in inserts {
			match refs.entry(*h) {
				// already counting. increment.
				Entry::Occupied(mut entry) => {
					entry.get_mut().queue_refs += 1;
				},
				// this is the first entry for this node in the journal.
				// it is initialised to 1 if it was already in.
				Entry::Vacant(entry) => {
					entry.insert(RefInfo {
						queue_refs: 1,
						in_archive: Self::is_already_in(backing, col, h),
					});
				},
			}
		}
		trace!(target: "jdb.fine", "replay_keys: (end) refs={:?}", refs);
	}

	fn remove_keys(deletes: &[H256], refs: &mut HashMap<H256, RefInfo>, batch: &mut DBTransaction, col: Option<u32>, from: RemoveFrom) {
		// with a remove on {queue_refs: 1, in_archive: true}, we have two options:
		// - convert to {queue_refs: 1, in_archive: false} (i.e. remove it from the conceptual archive)
		// - convert to {queue_refs: 0, in_archive: true} (i.e. remove it from the conceptual queue)
		// (the latter option would then mean removing the RefInfo, since it would no longer be counted in the queue.)
		// both are valid, but we switch between them depending on context.
		//     All inserts in queue (i.e. those which may yet be reverted) have an entry in refs.
		for h in deletes {
			match refs.entry(*h) {
				Entry::Occupied(mut entry) => {
					if entry.get().in_archive && from == RemoveFrom::Archive {
						entry.get_mut().in_archive = false;
						Self::reset_already_in(batch, col, h);
						trace!(target: "jdb.fine", "    remove({}): In archive, 1 in queue: Reducing to queue only and recording", h);
						continue;
					}
					if entry.get().queue_refs > 1 {
						entry.get_mut().queue_refs -= 1;
						trace!(target: "jdb.fine", "    remove({}): In queue > 1 refs: Decrementing ref count to {}", h, entry.get().queue_refs);
						continue;
					}

					let queue_refs = entry.get().queue_refs;
					let in_archive = entry.get().in_archive;

					match (queue_refs, in_archive) {
						(1, true) => {
							entry.remove();
							Self::reset_already_in(batch, col, h);
							trace!(target: "jdb.fine", "    remove({}): In archive, 1 in queue: Removing from queue and leaving in archive", h);
						},
						(1, false) => {
							entry.remove();
							batch.delete(col, h.as_bytes());
							trace!(target: "jdb.fine", "    remove({}): Not in archive, only 1 ref in queue: Removing from queue and DB", h);
						},
						_ => panic!("Invalid value in refs: {:?}", entry.get()),
					}
				},
				Entry::Vacant(_entry) => {
					// Gets removed when moving from 1 to 0 additional refs. Should never be here at 0 additional refs.
					//assert!(!Self::is_already_in(db, &h));
					batch.delete(col, h.as_bytes());
					trace!(target: "jdb.fine", "    remove({}): Not in queue - MUST BE IN ARCHIVE: Removing from DB", h);
				},
			}
		}
	}

	#[cfg(test)]
	fn can_reconstruct_refs(&self) -> bool {
		let (latest_era, reconstructed) = Self::read_refs(&*self.backing, self.column);
		let refs = self.refs.as_ref().unwrap().write();
		if *refs != reconstructed || latest_era != self.latest_era {
			let clean_refs = refs.iter().filter_map(|(k, v)| if reconstructed.get(k) == Some(v) {None} else {Some((k.clone(), v.clone()))}).collect::<HashMap<_, _>>();
			let clean_recon = reconstructed.into_iter().filter_map(|(k, v)| if refs.get(&k) == Some(&v) {None} else {Some((k.clone(), v.clone()))}).collect::<HashMap<_, _>>();
			warn!(target: "jdb", "mem: {:?}  !=  log: {:?}", clean_refs, clean_recon);
			false
		} else {
			true
		}
	}

	fn payload(&self, key: &H256) -> Option<DBValue> {
		self.backing
			.get(self.column, key.as_bytes())
			.expect("Low-level database error. Some issue with your hard disk?")
	}

	fn read_refs(db: &dyn KeyValueDB, col: Option<u32>) -> (Option<u64>, HashMap<H256, RefInfo>) {
		let mut refs = HashMap::new();
		let mut latest_era = None;
		if let Some(val) = db.get(col, &LATEST_ERA_KEY).expect("Low-level database error.") {
			let mut era = decode::<u64>(&val).expect("decoding db value failed");
			latest_era = Some(era);
			loop {
				let mut db_key = DatabaseKey {
					era,
					index: 0usize,
				};
				while let Some(rlp_data) = db.get(col, &encode(&db_key)).expect("Low-level database error.") {
					let inserts = DatabaseValueView::from_rlp(&rlp_data).inserts().expect("rlp read from db; qed");
					Self::replay_keys(&inserts, db, col, &mut refs);
					db_key.index += 1;
				};
				if db_key.index == 0 || era == 0 {
					break;
				}
				era -= 1;
			}
		}
		(latest_era, refs)
	}

}

impl HashDB<KeccakHasher, DBValue> for EarlyMergeDB {
	fn get(&self, key: &H256, prefix: Prefix) -> Option<DBValue> {
		if let Some((d, rc)) = self.overlay.raw(key, prefix) {
			if rc > 0 {
				return Some(d.clone())
			}
		}
		self.payload(key)
	}

	fn contains(&self, key: &H256, prefix: Prefix) -> bool {
		self.get(key, prefix).is_some()
	}

	fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H256 {
		self.overlay.insert(prefix, value)
	}
	fn emplace(&mut self, key: H256, prefix: Prefix, value: DBValue) {
		self.overlay.emplace(key, prefix, value);
	}
	fn remove(&mut self, key: &H256, prefix: Prefix) {
		self.overlay.remove(key, prefix);
	}
}

impl ::traits::KeyedHashDB for EarlyMergeDB {
	fn keys(&self) -> HashMap<H256, i32> {
		let mut ret: HashMap<H256, i32> = self.backing.iter(self.column)
			.map(|(key, _)| (H256::from_slice(&*key), 1))
			.collect();

		for (key, refs) in self.overlay.keys() {
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
}

impl JournalDB for EarlyMergeDB {
	fn boxed_clone(&self) -> Box<dyn JournalDB> {
		Box::new(EarlyMergeDB {
			overlay: self.overlay.clone(),
			backing: self.backing.clone(),
			refs: self.refs.clone(),
			latest_era: self.latest_era.clone(),
			column: self.column.clone(),
		})
	}

	fn is_empty(&self) -> bool {
		self.backing.get(self.column, &LATEST_ERA_KEY).expect("Low level database error").is_none()
	}

	fn backing(&self) -> &Arc<dyn KeyValueDB> {
		&self.backing
	}

	fn latest_era(&self) -> Option<u64> { self.latest_era }

	fn mem_used(&self) -> usize {
		let mut ops = new_malloc_size_ops();
		self.overlay.size_of(&mut ops) + match self.refs {
			Some(ref c) => c.read().size_of(&mut ops),
			None => 0
		}
 	}

	fn state(&self, id: &H256) -> Option<Bytes> {
		self.backing.get_by_prefix(self.column, &id[0..DB_PREFIX_LEN]).map(|b| b.into_vec())
	}

	fn journal_under(&mut self, batch: &mut DBTransaction, now: u64, id: &H256) -> io::Result<u32> {
		// record new commit's details.
		let mut refs = match self.refs.as_ref() {
			Some(refs) => refs.write(),
			None => return Ok(0),
		};

		{
			let mut db_key = DatabaseKey {
				era: now,
				index: 0usize,
			};
			let mut last;

			while self.backing.get(self.column, {
				last = encode(&db_key);
				&last
			})?.is_some() {
				db_key.index += 1;
			}

			let drained = self.overlay.drain();

			trace!(target: "jdb", "commit: #{} ({})", now, id);

			let removes: Vec<H256> = drained
				.iter()
				.filter_map(|(k, &(_, c))| if c < 0 {Some(k.clone())} else {None})
				.collect();
			let inserts: Vec<(H256, _)> = drained
				.into_iter()
				.filter_map(|(k, (v, r))| if r > 0 { assert!(r == 1); Some((k, v)) } else { assert!(r >= -1); None })
				.collect();

			// TODO: check all removes are in the db.

			// Process the new inserts.
			// We use the inserts for three things. For each:
			// - we place into the backing DB or increment the counter if already in;
			// - we note in the backing db that it was already in;
			// - we write the key into our journal for this block;

			Self::insert_keys(&inserts, &*self.backing, self.column, &mut refs, batch);

			let ins = inserts.iter().map(|&(k, _)| k).collect::<Vec<_>>();
			let value_ref = DatabaseValueRef {
				id,
				inserts: &ins,
				deletes: &removes,
			};

			trace!(target: "jdb.ops", "  Deletes: {:?}", removes);
			trace!(target: "jdb.ops", "  Inserts: {:?}", ins);

			batch.put(self.column, &last, &encode(&value_ref));
			if self.latest_era.map_or(true, |e| now > e) {
				batch.put(self.column, &LATEST_ERA_KEY, &encode(&now));
				self.latest_era = Some(now);
			}

			Ok((ins.len() + removes.len()) as u32)
		}
	}

	fn mark_canonical(&mut self, batch: &mut DBTransaction, end_era: u64, canon_id: &H256) -> io::Result<u32> {
		let mut refs = self.refs.as_ref().unwrap().write();

		// apply old commits' details
		let mut db_key = DatabaseKey {
			era: end_era,
			index: 0usize,
		};
		let mut last;

		while let Some(rlp_data) = {
			last = encode(&db_key);
			self.backing.get(self.column, &last)
		}? {
			let view = DatabaseValueView::from_rlp(&rlp_data);
			let inserts = view.inserts().expect("rlp read from db; qed");

			if canon_id == &view.id().expect("rlp read from db; qed") {
				// Collect keys to be removed. Canon block - remove the (enacted) deletes.
				let deletes = view.deletes().expect("rlp read from db; qed");
				trace!(target: "jdb.ops", "  Expunging: {:?}", deletes);
				Self::remove_keys(&deletes, &mut refs, batch, self.column, RemoveFrom::Archive);

				trace!(target: "jdb.ops", "  Finalising: {:?}", inserts);
				for k in &inserts {
					match refs.get(k).cloned() {
						None => {
							// [in archive] -> SHIFT remove -> SHIFT insert None->Some{queue_refs: 1, in_archive: true} -> TAKE remove Some{queue_refs: 1, in_archive: true}->None -> TAKE insert
							// already expunged from the queue (which is allowed since the key is in the archive).
							// leave well alone.
						}
						Some( RefInfo{queue_refs: 1, in_archive: false} ) => {
							// just delete the refs entry.
							refs.remove(k);
						}
						Some( RefInfo{queue_refs: x, in_archive: false} ) => {
							// must set already in; ,
							Self::set_already_in(batch, self.column, k);
							refs.insert(k.clone(), RefInfo{ queue_refs: x - 1, in_archive: true });
						}
						Some( RefInfo{in_archive: true, ..} ) => {
							// Invalid! Reinserted the same key twice.
							warn!("Key {} inserted twice into same fork.", k);
						}
					}
				}
			} else {
				// Collect keys to be removed. Non-canon block - remove the (reverted) inserts.
				trace!(target: "jdb.ops", "  Reverting: {:?}", inserts);
				Self::remove_keys(&inserts, &mut refs, batch, self.column, RemoveFrom::Queue);
			}

			batch.delete(self.column, &last);
			db_key.index += 1;
		}

		trace!(target: "jdb", "EarlyMergeDB: delete journal for time #{}.{}, (canon was {})", end_era, db_key.index, canon_id);
		trace!(target: "jdb", "OK: {:?}", &*refs);

		Ok(0)
	}

	fn inject(&mut self, batch: &mut DBTransaction) -> io::Result<u32> {
		let mut ops = 0;
		for (key, (value, rc)) in self.overlay.drain() {
			if rc != 0 { ops += 1 }

			match rc {
				0 => {}
				1 => {
					if self.backing.get(self.column, key.as_bytes())?.is_some() {
						return Err(error_key_already_exists(&key));
					}
					batch.put(self.column, key.as_bytes(), &value)
				}
				-1 => {
					if self.backing.get(self.column, key.as_bytes())?.is_none() {
						return Err(error_negatively_reference_hash(&key));
					}
					batch.delete(self.column, key.as_bytes())
				}
				_ => panic!("Attempted to inject invalid state."),
			}
		}

		Ok(ops)
	}

	fn consolidate(&mut self, with: super::MemoryDB) {
		self.overlay.consolidate(with);
	}
}

#[cfg(test)]
mod tests {

	use keccak::keccak;
	use hash_db::{HashDB, EMPTY_PREFIX};
	use super::*;
	use super::super::traits::JournalDB;
	use kvdb_memorydb;

	#[test]
	fn insert_same_in_fork() {
		// history is 1
		let mut jdb = new_db();

		let x = jdb.insert(EMPTY_PREFIX, b"X");
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(3, &keccak(b"1002a"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(4, &keccak(b"1003a"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&x, EMPTY_PREFIX);
		jdb.commit_batch(3, &keccak(b"1002b"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		let x = jdb.insert(EMPTY_PREFIX, b"X");
		jdb.commit_batch(4, &keccak(b"1003b"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(5, &keccak(b"1004a"), Some((3, keccak(b"1002a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(6, &keccak(b"1005a"), Some((4, keccak(b"1003a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&x, EMPTY_PREFIX));
	}

	#[test]
	fn insert_older_era() {
		let mut jdb = new_db();
		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(0, &keccak(b"0a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let bar = jdb.insert(EMPTY_PREFIX, b"bar");
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&bar, EMPTY_PREFIX);
		jdb.commit_batch(0, &keccak(b"0b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();

		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(jdb.contains(&bar, EMPTY_PREFIX));
	}

	#[test]
	fn long_history() {
		// history is 3
		let mut jdb = new_db();
		let h = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h, EMPTY_PREFIX));
		jdb.remove(&h, EMPTY_PREFIX);
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h, EMPTY_PREFIX));
		jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h, EMPTY_PREFIX));
		jdb.commit_batch(3, &keccak(b"3"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&h, EMPTY_PREFIX));
		jdb.commit_batch(4, &keccak(b"4"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.contains(&h, EMPTY_PREFIX));
	}

	#[test]
	fn complex() {
		// history is 1
		let mut jdb = new_db();

		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		let bar = jdb.insert(EMPTY_PREFIX, b"bar");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(jdb.contains(&bar, EMPTY_PREFIX));

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.remove(&bar, EMPTY_PREFIX);
		let baz = jdb.insert(EMPTY_PREFIX, b"baz");
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(jdb.contains(&bar, EMPTY_PREFIX));
		assert!(jdb.contains(&baz, EMPTY_PREFIX));

		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.remove(&baz, EMPTY_PREFIX);
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(!jdb.contains(&bar, EMPTY_PREFIX));
		assert!(jdb.contains(&baz, EMPTY_PREFIX));

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(3, &keccak(b"3"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(!jdb.contains(&bar, EMPTY_PREFIX));
		assert!(!jdb.contains(&baz, EMPTY_PREFIX));

		jdb.commit_batch(4, &keccak(b"4"), Some((3, keccak(b"3")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.contains(&foo, EMPTY_PREFIX));
		assert!(!jdb.contains(&bar, EMPTY_PREFIX));
		assert!(!jdb.contains(&baz, EMPTY_PREFIX));
	}

	#[test]
	fn fork() {
		// history is 1
		let mut jdb = new_db();

		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		let bar = jdb.insert(EMPTY_PREFIX, b"bar");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(jdb.contains(&bar, EMPTY_PREFIX));

		jdb.remove(&foo, EMPTY_PREFIX);
		let baz = jdb.insert(EMPTY_PREFIX, b"baz");
		jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&bar, EMPTY_PREFIX);
		jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(jdb.contains(&bar, EMPTY_PREFIX));
		assert!(jdb.contains(&baz, EMPTY_PREFIX));

		jdb.commit_batch(2, &keccak(b"2b"), Some((1, keccak(b"1b")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		assert!(!jdb.contains(&baz, EMPTY_PREFIX));
		assert!(!jdb.contains(&bar, EMPTY_PREFIX));
	}

	#[test]
	fn overwrite() {
		// history is 1
		let mut jdb = new_db();

		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(EMPTY_PREFIX, b"foo");
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
		jdb.commit_batch(3, &keccak(b"2"), Some((0, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
	}

	#[test]
	fn fork_same_key_one() {

		let mut jdb = new_db();
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1c"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&foo, EMPTY_PREFIX));

		jdb.commit_batch(2, &keccak(b"2a"), Some((1, keccak(b"1a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
	}

	#[test]
	fn fork_same_key_other() {
		let mut jdb = new_db();
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1c"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		assert!(jdb.contains(&foo, EMPTY_PREFIX));

		jdb.commit_batch(2, &keccak(b"2b"), Some((1, keccak(b"1b")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));
	}

	#[test]
	fn fork_ins_del_ins() {
		let mut jdb = new_db();
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(2, &keccak(b"2a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(2, &keccak(b"2b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(3, &keccak(b"3a"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(3, &keccak(b"3b"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(4, &keccak(b"4a"), Some((2, keccak(b"2a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(5, &keccak(b"5a"), Some((3, keccak(b"3a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	fn new_db() -> EarlyMergeDB {
		let backing = Arc::new(kvdb_memorydb::create(0));
		EarlyMergeDB::new(backing, None)
	}

	#[test]
	fn reopen() {
		let shared_db = Arc::new(kvdb_memorydb::create(0));
		let bar = H256::random();

		let foo = {
			let mut jdb = EarlyMergeDB::new(shared_db.clone(), None);
			// history is 1
			let foo = jdb.insert(EMPTY_PREFIX, b"foo");
			jdb.emplace(bar.clone(), EMPTY_PREFIX, DBValue::from_slice(b"bar"));
			jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			foo
		};

		{
			let mut jdb = EarlyMergeDB::new(shared_db.clone(), None);
			jdb.remove(&foo, EMPTY_PREFIX);
			jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
		}

		{
			let mut jdb = EarlyMergeDB::new(shared_db, None);
			assert!(jdb.contains(&foo, EMPTY_PREFIX));
			assert!(jdb.contains(&bar, EMPTY_PREFIX));
			jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(!jdb.contains(&foo, EMPTY_PREFIX));
		}
	}

	#[test]
	fn insert_delete_insert_delete_insert_expunge() {
		let _ = ::env_logger::try_init();

		let mut jdb = new_db();

		// history is 4
		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(3, &keccak(b"3"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(4, &keccak(b"4"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		// expunge foo
		jdb.commit_batch(5, &keccak(b"5"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn forked_insert_delete_insert_delete_insert_expunge() {
		let _ = ::env_logger::try_init();
		let mut jdb = new_db();

		// history is 4
		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(1, &keccak(b"1a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(1, &keccak(b"1b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(2, &keccak(b"2a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(2, &keccak(b"2b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(3, &keccak(b"3a"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(3, &keccak(b"3b"), None).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(4, &keccak(b"4a"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(4, &keccak(b"4b"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// expunge foo
		jdb.commit_batch(5, &keccak(b"5"), Some((1, keccak(b"1a")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn broken_assert() {
		let mut jdb = new_db();

		// history is 1
		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		// foo is ancient history.

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.commit_batch(3, &keccak(b"3"), Some((2, keccak(b"2")))).unwrap();	// BROKEN
		assert!(jdb.can_reconstruct_refs());
		assert!(jdb.contains(&foo, EMPTY_PREFIX));

		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.commit_batch(4, &keccak(b"4"), Some((3, keccak(b"3")))).unwrap();
		assert!(jdb.can_reconstruct_refs());

		jdb.commit_batch(5, &keccak(b"5"), Some((4, keccak(b"4")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		assert!(!jdb.contains(&foo, EMPTY_PREFIX));
	}

	#[test]
	fn reopen_test() {
		let mut jdb = new_db();

		// history is 4
		let foo = jdb.insert(EMPTY_PREFIX, b"foo");
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

		jdb.insert(EMPTY_PREFIX, b"foo");
		let bar = jdb.insert(EMPTY_PREFIX, b"bar");
		jdb.commit_batch(5, &keccak(b"5"), Some((1, keccak(b"1")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.remove(&foo, EMPTY_PREFIX);
		jdb.remove(&bar, EMPTY_PREFIX);
		jdb.commit_batch(6, &keccak(b"6"), Some((2, keccak(b"2")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
		jdb.insert(EMPTY_PREFIX, b"foo");
		jdb.insert(EMPTY_PREFIX, b"bar");
		jdb.commit_batch(7, &keccak(b"7"), Some((3, keccak(b"3")))).unwrap();
		assert!(jdb.can_reconstruct_refs());
	}

	#[test]
	fn reopen_remove_three() {
		let _ = ::env_logger::try_init();

		let shared_db = Arc::new(kvdb_memorydb::create(0));
		let foo = keccak(b"foo");

		{
			let mut jdb = EarlyMergeDB::new(shared_db.clone(), None);
			// history is 1
			jdb.insert(EMPTY_PREFIX, b"foo");
			jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());

			// foo is ancient history.

			jdb.remove(&foo, EMPTY_PREFIX);
			jdb.commit_batch(2, &keccak(b"2"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo, EMPTY_PREFIX));

			jdb.insert(EMPTY_PREFIX, b"foo");
			jdb.commit_batch(3, &keccak(b"3"), Some((1, keccak(b"1")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo, EMPTY_PREFIX));

		// incantation to reopen the db
		}; {
			let mut jdb = EarlyMergeDB::new(shared_db.clone(), None);

			jdb.remove(&foo, EMPTY_PREFIX);
			jdb.commit_batch(4, &keccak(b"4"), Some((2, keccak(b"2")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo, EMPTY_PREFIX));

		// incantation to reopen the db
		}; {
			let mut jdb = EarlyMergeDB::new(shared_db.clone(), None);

			jdb.commit_batch(5, &keccak(b"5"), Some((3, keccak(b"3")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo, EMPTY_PREFIX));

		// incantation to reopen the db
		}; {
			let mut jdb = EarlyMergeDB::new(shared_db, None);

			jdb.commit_batch(6, &keccak(b"6"), Some((4, keccak(b"4")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(!jdb.contains(&foo, EMPTY_PREFIX));
		}
	}

	#[test]
	fn reopen_fork() {
		let shared_db = Arc::new(kvdb_memorydb::create(0));

		let (foo, bar, baz) = {
			let mut jdb = EarlyMergeDB::new(shared_db.clone(), None);
			// history is 1
			let foo = jdb.insert(EMPTY_PREFIX, b"foo");
			let bar = jdb.insert(EMPTY_PREFIX, b"bar");
			jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
			assert!(jdb.can_reconstruct_refs());
			jdb.remove(&foo, EMPTY_PREFIX);
			let baz = jdb.insert(EMPTY_PREFIX, b"baz");
			jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());

			jdb.remove(&bar, EMPTY_PREFIX);
			jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			(foo, bar, baz)
		};

		{
			let mut jdb = EarlyMergeDB::new(shared_db, None);
			jdb.commit_batch(2, &keccak(b"2b"), Some((1, keccak(b"1b")))).unwrap();
			assert!(jdb.can_reconstruct_refs());
			assert!(jdb.contains(&foo, EMPTY_PREFIX));
			assert!(!jdb.contains(&baz, EMPTY_PREFIX));
			assert!(!jdb.contains(&bar, EMPTY_PREFIX));
		}
	}

	#[test]
	fn inject() {
		let mut jdb = new_db();
		let key = jdb.insert(EMPTY_PREFIX, b"dog");
		jdb.inject_batch().unwrap();

		assert_eq!(jdb.get(&key, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"dog"));
		jdb.remove(&key, EMPTY_PREFIX);
		jdb.inject_batch().unwrap();

		assert!(jdb.get(&key, EMPTY_PREFIX).is_none());
	}
}
