//! Disk-backed HashDB implementation.

use error::*;
use hash::*;
use bytes::*;
use sha3::*;
use hashdb::*;
use memorydb::*;
use std::ops::*;
use std::sync::*;
use rocksdb::{DB, Writable};

#[derive(Clone)]
pub struct OverlayDB {
	overlay: MemoryDB,
	backing: Arc<DB>,
}

impl OverlayDB {
	/// Create a new instance of OverlayDB given a `backing` database.
	fn new(backing: DB) -> OverlayDB {
		OverlayDB{ overlay: MemoryDB::new(), backing: Arc::new(backing) }
	}

	/// Commit all memory operations to the backing database. 
	fn commit(&mut self) -> Result<u32, EthcoreError> {
		let mut ret = 0u32;
		for i in self.overlay.drain().into_iter() {
			let (key, (value, rc)) = i;
			if rc != 0 {
				let new_entry = match self.payload(&key) {
					Some(x) => {
						let (back_value, back_rc) = x;
						if back_rc + rc < 0 {
							return Err(From::from(BaseDataError::NegativelyReferencedHash));
						}
						self.put_payload(&key, (&back_value, rc + back_rc));
					}
					None => {
						self.put_payload(&key, (&value, rc));
					}
				};
				ret += 1;
			}
		}
		Ok(ret)
	}

	/// Get the refs and value of the given key.
	fn payload(&self, key: &H256) -> Option<(Bytes, i32)> {
		unimplemented!();
	}

	/// Get the refs and value of the given key.
	fn put_payload(&self, key: &H256, payload: (&Bytes, i32)) {
		unimplemented!();
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
						if rc + memrc > 0 {
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
			Some(&(ref d, rc)) if rc > 0 => true,
			_ => {
				let memrc = k.map(|&(_, rc)| rc).unwrap_or(0);
				match self.payload(key) {
					Some(x) => {
						let (d, rc) = x;
						if rc + memrc > 0 {
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
fn playpen() {
	let mut db: DB = DB::open_default("/tmp/test").unwrap();
	db.put(b"test", b"test2");
	match db.get(b"test") {
		Ok(Some(value)) => println!("Got value {:?}", value.deref()),
		Ok(None) => println!("No value for that key"),
		Err(e) => println!("Gah"),
	}
	db.delete(b"test");
	assert!(false);
}