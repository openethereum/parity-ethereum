//! Disk-backed HashDB implementation.

use error::*;
use hash::*;
use bytes::*;
use rlp::*;
use hashdb::*;
use memorydb::*;
use std::ops::*;
use std::sync::*;
use std::env;
use rocksdb::{DB, Writable};

#[derive(Clone)]
/// Implementation of the HashDB trait for a disk-backed database with a memory overlay.
///
/// The operations `insert()` and `kill()` take place on the memory overlay; batches of
/// such operations may be flushed to the disk-backed DB with `commit()` or discarded with
/// `revert()`.
///
/// `lookup()` and `exists()` maintain normal behaviour - all `insert()` and `kill()` 
/// queries have an immediate effect in terms of these functions.
pub struct OverlayDB {
	overlay: MemoryDB,
	backing: Arc<DB>,
}

impl OverlayDB {
	/// Create a new instance of OverlayDB given a `backing` database.
	pub fn new(backing: DB) -> OverlayDB {
		OverlayDB{ overlay: MemoryDB::new(), backing: Arc::new(backing) }
	}

	/// Create a new instance of OverlayDB with an anonymous temporary database.
	pub fn new_temp() -> OverlayDB {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		Self::new(DB::open_default(dir.to_str().unwrap()).unwrap())
	}

	/// Commit all memory operations to the backing database.
	///
	/// Returns either an error or the number of items changed in the backing database.
	/// 
	/// Will return an error if the number of `kill()`s ever exceeds the number of
	/// `insert()`s for any key. This will leave the database in an undeterminate
	/// state. Don't ever let it happen.
	///
	/// # Example
	/// ```
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::overlaydb::*;
	/// fn main() {
	///   let mut m = OverlayDB::new_temp();
	///   let key = m.insert(b"foo");			// insert item.
	///   assert!(m.exists(&key));				// key exists (in memory).
	///   assert_eq!(m.commit().unwrap(), 1);	// 1 item changed.
	///   assert!(m.exists(&key));				// key still exists (in backing).
	///   m.kill(&key);							// delete item.
	///   assert!(!m.exists(&key));				// key "doesn't exist" (though still does in backing).
	///   m.kill(&key);							// oh dear... more kills than inserts for the key...
	///   //m.commit().unwrap();				// this commit/unwrap would cause a panic.
	///   m.revert();							// revert both kills.
	///   assert!(m.exists(&key));				// key now still exists.
	/// }
	/// ```
	pub fn commit(&mut self) -> Result<u32, EthcoreError> {
		let mut ret = 0u32;
		for i in self.overlay.drain().into_iter() {
			let (key, (value, rc)) = i;
			if rc != 0 {
				match self.payload(&key) {
					Some(x) => {
						let (back_value, back_rc) = x;
						let total_rc: i32 = back_rc as i32 + rc;
						if total_rc < 0 {
							return Err(From::from(BaseDataError::NegativelyReferencedHash));
						}
						self.put_payload(&key, (back_value, total_rc as u32));
					}
					None => {
						if rc < 0 {
							return Err(From::from(BaseDataError::NegativelyReferencedHash));
						}
						self.put_payload(&key, (value, rc as u32));
					}
				};
				ret += 1;
			}
		}
		Ok(ret)
	}

	/// Revert all changes though `insert()` and `kill()` to this object since the
	/// last `commit()`.
	pub fn revert(&mut self) { self.overlay.clear(); }

	/// Get the refs and value of the given key.
	fn payload(&self, key: &H256) -> Option<(Bytes, u32)> {
		self.backing.get(&key.bytes())
			.expect("Low-level database error. Some issue with your hard disk?")
			.map(|d| {
				let r = Rlp::new(d.deref());
				(Bytes::decode(&r.at(1).unwrap()).unwrap(), u32::decode(&r.at(0).unwrap()).unwrap())
			})
	}

	/// Get the refs and value of the given key.
	fn put_payload(&self, key: &H256, payload: (Bytes, u32)) {
		let mut s = RlpStream::new_list(2);
		s.append(&payload.1);
		s.append(&payload.0);
		self.backing.put(&key.bytes(), &s.out().unwrap()).expect("Low-level database error. Some issue with your hard disk?");
	}
}

impl HashDB for OverlayDB {
	fn lookup(&self, key: &H256) -> Option<Bytes> {
		// return ok if positive; if negative, check backing - might be enough references there to make
		// it positive again.
		let k = self.overlay.raw(key);
		match k {
			Some(&(ref d, rc)) if rc > 0 => Some(d.clone()),
			_ => {
				let memrc = k.map(|&(_, rc)| rc).unwrap_or(0);
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
		}
	}
	fn exists(&self, key: &H256) -> bool {
		// return ok if positive; if negative, check backing - might be enough references there to make
		// it positive again.
		let k = self.overlay.raw(key);
		match k {
			Some(&(_, rc)) if rc > 0 => true,
			_ => {
				let memrc = k.map(|&(_, rc)| rc).unwrap_or(0);
				match self.payload(key) {
					Some(x) => {
						let (_, rc) = x;
						if rc as i32 + memrc > 0 {
							true
						}
						else {
							false
						}
					}
					// Replace above match arm with this once https://github.com/rust-lang/rust/issues/15287 is done.
					//Some((d, rc)) if rc + memrc > 0 => true,
					_ => false,
				}
			}
		}
	}
	fn insert(&mut self, value: &[u8]) -> H256 { self.overlay.insert(value) }
	fn kill(&mut self, key: &H256) { self.overlay.kill(key); }
}

#[test]
fn overlaydb_overlay_insert_and_kill() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	assert_eq!(trie.lookup(&h), Some(b"hello world".to_vec()));
	trie.kill(&h);
	assert_eq!(trie.lookup(&h), None);
}

#[test]
fn overlaydb_backing_insert_revert() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	assert_eq!(trie.lookup(&h), Some(b"hello world".to_vec()));
	trie.commit().unwrap();
	assert_eq!(trie.lookup(&h), Some(b"hello world".to_vec()));
	trie.revert();
	assert_eq!(trie.lookup(&h), Some(b"hello world".to_vec()));
}

#[test]
fn overlaydb_backing_kill() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	trie.commit().unwrap();
	trie.kill(&h);
	assert_eq!(trie.lookup(&h), None);
	trie.commit().unwrap();
	assert_eq!(trie.lookup(&h), None);
	trie.revert();
	assert_eq!(trie.lookup(&h), None);
}

#[test]
fn overlaydb_backing_kill_revert() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	trie.commit().unwrap();
	trie.kill(&h);
	assert_eq!(trie.lookup(&h), None);
	trie.revert();
	assert_eq!(trie.lookup(&h), Some(b"hello world".to_vec()));
}

#[test]
fn overlaydb_negative() {
	let mut trie = OverlayDB::new_temp();
	let h = trie.insert(b"hello world");
	trie.commit().unwrap();
	trie.kill(&h);
	trie.kill(&h);	//bad - sends us into negative refs.
	assert_eq!(trie.lookup(&h), None);
	assert!(trie.commit().is_err());
}

#[test]
fn overlaydb_complex() {
	let mut trie = OverlayDB::new_temp();
	let hfoo = trie.insert(b"foo");
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	let hbar = trie.insert(b"bar");
	assert_eq!(trie.lookup(&hbar), Some(b"bar".to_vec()));
	trie.commit().unwrap();
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	assert_eq!(trie.lookup(&hbar), Some(b"bar".to_vec()));
	trie.insert(b"foo");	// two refs
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	trie.commit().unwrap();
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	assert_eq!(trie.lookup(&hbar), Some(b"bar".to_vec()));
	trie.kill(&hbar);		// zero refs - delete
	assert_eq!(trie.lookup(&hbar), None);
	trie.kill(&hfoo);		// one ref - keep
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	trie.commit().unwrap();
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	trie.kill(&hfoo);		// zero ref - would delete, but...
	assert_eq!(trie.lookup(&hfoo), None);
	trie.insert(b"foo");	// one ref - keep after all.
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	trie.commit().unwrap();
	assert_eq!(trie.lookup(&hfoo), Some(b"foo".to_vec()));
	trie.kill(&hfoo);		// zero ref - delete
	assert_eq!(trie.lookup(&hfoo), None);
	trie.commit().unwrap();	// 
	assert_eq!(trie.lookup(&hfoo), None);
}

#[test]
fn playpen() {
	use std::fs;
	{
		let db: DB = DB::open_default("/tmp/test").unwrap();
		db.put(b"test", b"test2").unwrap();
		match db.get(b"test") {
			Ok(Some(value)) => println!("Got value {:?}", value.deref()),
			Ok(None) => println!("No value for that key"),
			Err(..) => println!("Gah"),
		}
		db.delete(b"test").unwrap();
	}
	fs::remove_dir_all("/tmp/test").unwrap();
}