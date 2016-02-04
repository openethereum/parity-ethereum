//! Disk-backed HashDB implementation.

use common::*;
use rlp::*;
use hashdb::*;
use memorydb::*;
use rocksdb::{DB, Writable, IteratorMode};
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
}

impl JournalDB {
	/// Create a new instance given a `backing` database.
	pub fn new(backing: DB) -> JournalDB {
		let db = Arc::new(backing);
		JournalDB {
			overlay: MemoryDB::new(),
			backing: db,
		}
	}

	/// Create a new instance given a shared `backing` database.
	pub fn new_with_arc(backing: Arc<DB>) -> JournalDB {
		JournalDB {
			overlay: MemoryDB::new(),
			backing: backing,
		}
	}

	/// Create a new instance with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> JournalDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(DB::open_default(dir.to_str().unwrap()).unwrap())
	}


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
			let inserts: Vec<H256> = self.overlay.keys().iter().filter(|&(_, &c)| c > 0).map(|(key, _)| key.clone()).collect();
			let removes: Vec<H256> = self.overlay.keys().iter().filter(|&(_, &c)| c < 0).map(|(key, _)| key.clone()).collect();
			r.append(id);
			r.append(&inserts);
			r.append(&removes);
			try!(self.backing.put(&last, r.as_raw()));
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
					self.backing.delete(&i).expect("Low-level database error. Some issue with your hard disk?");
				}
				try!(self.backing.delete(&last));
				trace!("JournalDB: delete journal for time #{}.{}, (canon was {}): {} entries", end_era, index, canon_id, to_remove.len());
				index += 1;
			}
		}

		let mut ret = 0u32;
		let mut deletes = 0usize;
		for i in self.overlay.drain().into_iter() {
			let (key, (value, rc)) = i;
			if rc > 0 {
				assert!(rc == 1);
				if !self.backing.get(&key.bytes()).unwrap().is_none() {
					info!("Exist: {:?}", key);
					key.clone();
				}
				self.backing.put(&key.bytes(), &value).expect("Low-level database error. Some issue with your hard disk?");
				ret += 1;
			}
			if rc < 0 {
				assert!(rc == -1);
				ret += 1;
				deletes += 1;
			}
		}
		trace!("JournalDB::commit() deleted {} nodes", deletes);
		Ok(ret)
	}

	fn payload(&self, key: &H256) -> Option<Bytes> {
		self.backing.get(&key.bytes()).expect("Low-level database error. Some issue with your hard disk?").map(|v| v.to_vec())
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
		if value.sha3() == h256_from_hex("3567da57862169b0dc409933ec10da8113ef3810fd225ad81d4fc23c36ffa5d4") {
			info!("GOTCHA");
			value.to_vec();
		}
		self.overlay.insert(value) 
	}
	fn emplace(&mut self, key: H256, value: Bytes) {
		if key == h256_from_hex("3567da57862169b0dc409933ec10da8113ef3810fd225ad81d4fc23c36ffa5d4") {
			info!("GOTCHA");
			value.to_vec();
		}
		self.overlay.emplace(key, value); 
	}
	fn kill(&mut self, key: &H256) { 
			if key == &h256_from_hex("3567da57862169b0dc409933ec10da8113ef3810fd225ad81d4fc23c36ffa5d4") {
			info!("DELETING");
			key.clone();
		}
		self.overlay.kill(key); }
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
