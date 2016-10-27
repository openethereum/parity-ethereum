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

//! Disk-backed `HashDB` implementation.

use error::*;
use hash::*;
use rlp::*;
use hashdb::*;
use memorydb::*;
use std::sync::*;
use std::collections::HashMap;
use kvdb::{Database, DBTransaction};

/// Implementation of the `HashDB` trait for a disk-backed database with a memory overlay.
///
/// The operations `insert()` and `remove()` take place on the memory overlay; batches of
/// such operations may be flushed to the disk-backed DB with `commit()` or discarded with
/// `revert()`.
///
/// `lookup()` and `contains()` maintain normal behaviour - all `insert()` and `remove()`
/// queries have an immediate effect in terms of these functions.
#[derive(Clone)]
pub struct OverlayDB {
	overlay: MemoryDB,
	backing: Arc<Database>,
	column: Option<u32>,
}

impl OverlayDB {
	/// Create a new instance of OverlayDB given a `backing` database.
	pub fn new(backing: Arc<Database>, col: Option<u32>) -> OverlayDB {
		OverlayDB{ overlay: MemoryDB::new(), backing: backing, column: col }
	}

	/// Create a new instance of OverlayDB with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> OverlayDB {
		let mut dir = ::std::env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(Arc::new(Database::open_default(dir.to_str().unwrap()).unwrap()), None)
	}

	/// Commit all operations in a single batch.
	#[cfg(test)]
	pub fn commit(&mut self) -> Result<u32, UtilError> {
		let mut batch = self.backing.transaction();
		let res = try!(self.commit_to_batch(&mut batch));
		self.backing.write(batch).map(|_| res).map_err(|e| e.into())
	}

	/// Commit all operations to given batch.
	pub fn commit_to_batch(&mut self, batch: &mut DBTransaction) -> Result<u32, UtilError> {
		let mut ret = 0u32;
		let mut deletes = 0usize;
		for i in self.overlay.drain() {
			let (key, (value, rc)) = i;
			if rc != 0 {
				match self.payload(&key) {
					Some(x) => {
						let (back_value, back_rc) = x;
						let total_rc: i32 = back_rc as i32 + rc;
						if total_rc < 0 {
							return Err(From::from(BaseDataError::NegativelyReferencedHash(key)));
						}
						deletes += if self.put_payload_in_batch(batch, &key, (back_value, total_rc as u32)) {1} else {0};
					}
					None => {
						if rc < 0 {
							return Err(From::from(BaseDataError::NegativelyReferencedHash(key)));
						}
						self.put_payload_in_batch(batch, &key, (value, rc as u32));
					}
				};
				ret += 1;
			}
		}
		trace!("OverlayDB::commit() deleted {} nodes", deletes);
		Ok(ret)
	}

	/// Revert all operations on this object (i.e. `insert()`s and `remove()`s) since the
	/// last `commit()`.
	pub fn revert(&mut self) { self.overlay.clear(); }

	/// Get the number of references that would be committed.
	pub fn commit_refs(&self, key: &H256) -> i32 { self.overlay.raw(key).map_or(0, |(_, refs)| refs) }

	/// Get the refs and value of the given key.
	fn payload(&self, key: &H256) -> Option<(DBValue, u32)> {
		self.backing.get(self.column, key)
			.expect("Low-level database error. Some issue with your hard disk?")
			.map(|d| {
				let r = Rlp::new(&d);
				(DBValue::from_slice(r.at(1).data()), r.at(0).as_val())
			})
	}

	/// Put the refs and value of the given key, possibly deleting it from the db.
	fn put_payload_in_batch(&self, batch: &mut DBTransaction, key: &H256, payload: (DBValue, u32)) -> bool {
		if payload.1 > 0 {
			let mut s = RlpStream::new_list(2);
			s.append(&payload.1);
			s.append(&&*payload.0);
			batch.put(self.column, key, s.as_raw());
			false
		} else {
			batch.delete(self.column, key);
			true
		}
	}
}

impl HashDB for OverlayDB {
	fn keys(&self) -> HashMap<H256, i32> {
		let mut ret: HashMap<H256, i32> = HashMap::new();
		for (key, _) in self.backing.iter(self.column) {
			let h = H256::from_slice(&*key);
			let r = self.payload(&h).unwrap().1;
			ret.insert(h, r as i32);
		}

		for (key, refs) in self.overlay.keys() {
			let refs = *ret.get(&key).unwrap_or(&0) + refs;
			ret.insert(key, refs);
		}
		ret
	}
	fn get(&self, key: &H256) -> Option<DBValue> {
		// return ok if positive; if negative, check backing - might be enough references there to make
		// it positive again.
		let k = self.overlay.raw(key);
		let memrc = {
			if let Some((d, rc)) = k {
				if rc > 0 { return Some(d); }
				rc
			} else {
				0
			}
		};
		match self.payload(key) {
			Some(x) => {
				let (d, rc) = x;
				if rc as i32 + memrc > 0 {
					Some(d)
				}
				else {
					None
				}
			}
			// Replace above match arm with this once https://github.com/rust-lang/rust/issues/15287 is done.
			//Some((d, rc)) if rc + memrc > 0 => Some(d),
			_ => None,
		}
	}
	fn contains(&self, key: &H256) -> bool {
		// return ok if positive; if negative, check backing - might be enough references there to make
		// it positive again.
		let k = self.overlay.raw(key);
		match k {
			Some((_, rc)) if rc > 0 => true,
			_ => {
				let memrc = k.map_or(0, |(_, rc)| rc);
				match self.payload(key) {
					Some(x) => {
						let (_, rc) = x;
						rc as i32 + memrc > 0
					}
					// Replace above match arm with this once https://github.com/rust-lang/rust/issues/15287 is done.
					//Some((d, rc)) if rc + memrc > 0 => true,
					_ => false,
				}
			}
		}
	}
	fn insert(&mut self, value: &[u8]) -> H256 { self.overlay.insert(value) }
	fn emplace(&mut self, key: H256, value: DBValue) { self.overlay.emplace(key, value); }
	fn remove(&mut self, key: &H256) { self.overlay.remove(key); }
}

#[test]
#[cfg_attr(feature="dev", allow(blacklisted_name))]
fn overlaydb_revert() {
	let mut m = OverlayDB::new_temp();
	let foo = m.insert(b"foo");          // insert foo.
	let mut batch = m.backing.transaction();
	m.commit_to_batch(&mut batch).unwrap();  // commit - new operations begin here...
	m.backing.write(batch).unwrap();
	let bar = m.insert(b"bar");          // insert bar.
	m.remove(&foo);                      // remove foo.
	assert!(!m.contains(&foo));          // foo is gone.
	assert!(m.contains(&bar));           // bar is here.
	m.revert();                          // revert the last two operations.
	assert!(m.contains(&foo));           // foo is here.
	assert!(!m.contains(&bar));          // bar is gone.
}

#[test]
fn overlaydb_overlay_insert_and_remove() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	assert_eq!(trie.get(&h).unwrap(), DBValue::from_slice(b"hello world"));
	trie.remove(&h);
	assert_eq!(trie.get(&h), None);
}

#[test]
fn overlaydb_backing_insert_revert() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	assert_eq!(trie.get(&h).unwrap(), DBValue::from_slice(b"hello world"));
	trie.commit().unwrap();
	assert_eq!(trie.get(&h).unwrap(), DBValue::from_slice(b"hello world"));
	trie.revert();
	assert_eq!(trie.get(&h).unwrap(), DBValue::from_slice(b"hello world"));
}

#[test]
fn overlaydb_backing_remove() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	trie.commit().unwrap();
	trie.remove(&h);
	assert_eq!(trie.get(&h), None);
	trie.commit().unwrap();
	assert_eq!(trie.get(&h), None);
	trie.revert();
	assert_eq!(trie.get(&h), None);
}

#[test]
fn overlaydb_backing_remove_revert() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	trie.commit().unwrap();
	trie.remove(&h);
	assert_eq!(trie.get(&h), None);
	trie.revert();
	assert_eq!(trie.get(&h).unwrap(), DBValue::from_slice(b"hello world"));
}

#[test]
fn overlaydb_negative() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	trie.commit().unwrap();
	trie.remove(&h);
	trie.remove(&h);	//bad - sends us into negative refs.
	assert_eq!(trie.get(&h), None);
	assert!(trie.commit().is_err());
}

#[test]
fn overlaydb_complex() {
	let mut trie = OverlayDB::new_temp();
	let hfoo = trie.insert(b"foo");
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	let hbar = trie.insert(b"bar");
	assert_eq!(trie.get(&hbar).unwrap(), DBValue::from_slice(b"bar"));
	trie.commit().unwrap();
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	assert_eq!(trie.get(&hbar).unwrap(), DBValue::from_slice(b"bar"));
	trie.insert(b"foo");	// two refs
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	trie.commit().unwrap();
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	assert_eq!(trie.get(&hbar).unwrap(), DBValue::from_slice(b"bar"));
	trie.remove(&hbar);		// zero refs - delete
	assert_eq!(trie.get(&hbar), None);
	trie.remove(&hfoo);		// one ref - keep
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	trie.commit().unwrap();
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	trie.remove(&hfoo);		// zero ref - would delete, but...
	assert_eq!(trie.get(&hfoo), None);
	trie.insert(b"foo");	// one ref - keep after all.
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	trie.commit().unwrap();
	assert_eq!(trie.get(&hfoo).unwrap(), DBValue::from_slice(b"foo"));
	trie.remove(&hfoo);		// zero ref - delete
	assert_eq!(trie.get(&hfoo), None);
	trie.commit().unwrap();	//
	assert_eq!(trie.get(&hfoo), None);
}

#[test]
fn playpen() {
	use std::fs;
	{
		let db = Database::open_default("/tmp/test").unwrap();
		let mut batch = db.transaction();
		batch.put(None, b"test", b"test2");
		db.write(batch).unwrap();
		match db.get(None, b"test") {
			Ok(Some(value)) => println!("Got value {:?}", &*value),
			Ok(None) => println!("No value for that key"),
			Err(..) => println!("Gah"),
		}
		let mut batch = db.transaction();
		batch.delete(None, b"test");
		db.write(batch).unwrap();
	}
	fs::remove_dir_all("/tmp/test").unwrap();
}
