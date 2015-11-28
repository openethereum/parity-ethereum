//! Disk-backed HashDB implementation.

use hash::*;
use bytes::*;
use sha3::*;
use hashdb::*;
use memorydb::*;
use std::ops::*;
use rocksdb::{DB, Writable};

#[derive(Clone)]
pub struct OverlayDB {
	overlay: MemoryDB,
	backing: DB,
}

impl OverlayDB {
	/// Create a new instance of OverlayDB given a `backing` database.
	fn new(backing: DB) {
		self.backing = backing;
		overlay = MemoryDB::new();
	}

	/// Commit all memory operations to the backing database.
	fn commit(&mut self) {
		unimplemented!();
	}

	/// Get the refs and value of the given key.
	fn payload(&self, key: &H256) -> Option<(Bytes, i32)> {
		unimplemented!();
	}
}

impl HashDB for OverlayDB {
	fn lookup(&self, key: &H256) -> Option<Bytes> {
		// TODO: return ok if positive; if negative, check backing - might be enough references there to make
		// it positive again.
		let k = self.overlay.data.get(key);
		match k {
			Some(&(ref d, rc)) if rc > 0 => Some(d.clone()),
			_ => {
				let memrc = k.map(|&(_, rc)| rc).unwrap_or(0);
				match self.payload(key) {
					Some((d, rc)) if rc + memrc > 0 => Some(d),
					_ => None,
				}
			}
		}
	}
	fn exists(&self, key: &H256) -> bool {
		// TODO: copy and adapt code above.
		m_overlay.exists(key)
	}
	fn insert(&mut self, value: &[u8]) -> H256 { m_overlay.insert(value) }
	fn kill(&mut self, key: &H256) { m_overlay.kill(key); }
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