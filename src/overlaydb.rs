//! Disk-backed HashDB implementation.

use error::*;
use hash::*;
use bytes::*;
use rlp::*;
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
	pub fn new(backing: DB) -> OverlayDB {
		OverlayDB{ overlay: MemoryDB::new(), backing: Arc::new(backing) }
	}

	/// Commit all memory operations to the backing database. 
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