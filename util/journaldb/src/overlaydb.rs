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

use std::{
	collections::{HashMap, hash_map::Entry},
	io,
	sync::Arc,
};

use ethereum_types::H256;
use hash_db::{HashDB, Prefix};
use keccak_hasher::KeccakHasher;
use kvdb::{KeyValueDB, DBTransaction, DBValue};
use log::trace;
use rlp::{Rlp, RlpStream, Encodable, DecoderError, Decodable, encode, decode};

use crate::{error_negatively_reference_hash, new_memory_db};

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
	overlay: super::MemoryDB,
	backing: Arc<dyn KeyValueDB>,
	column: Option<u32>,
}

struct Payload {
	count: u32,
	value: DBValue,
}

impl Payload {
	fn new(count: u32, value: DBValue) -> Self {
		Payload {
			count,
			value,
		}
	}
}

impl Encodable for Payload {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.count);
		s.append(&&*self.value);
	}
}

impl Decodable for Payload {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		let payload = Payload {
			count: rlp.val_at(0)?,
			value: DBValue::from_slice(rlp.at(1)?.data()?),
		};

		Ok(payload)
	}
}

impl OverlayDB {
	/// Create a new instance of OverlayDB given a `backing` database.
	pub fn new(backing: Arc<dyn KeyValueDB>, column: Option<u32>) -> OverlayDB {
		OverlayDB {
			overlay: new_memory_db(),
			backing,
			column,
		}
	}

	/// Create a new instance of OverlayDB with an anonymous temporary database.
	#[cfg(test)]
	pub fn new_temp() -> OverlayDB {
		let backing = Arc::new(::kvdb_memorydb::create(0));
		Self::new(backing, None)
	}

	/// Commit all operations in a single batch.
	#[cfg(test)]
	pub fn commit(&mut self) -> io::Result<u32> {
		let mut batch = self.backing.transaction();
		let res = self.commit_to_batch(&mut batch)?;
		self.backing.write(batch).map(|_| res).map_err(|e| e.into())
	}

	/// Commit all operations to given batch.
	pub fn commit_to_batch(&mut self, batch: &mut DBTransaction) -> io::Result<u32> {
		let mut ret = 0u32;
		let mut deletes = 0usize;
		for i in self.overlay.drain() {
			let (key, (value, rc)) = i;
			if rc != 0 {
				match self.payload(&key) {
					Some(x) => {
						let total_rc: i32 = x.count as i32 + rc;
						if total_rc < 0 {
							return Err(error_negatively_reference_hash(&key));
						}
						let payload = Payload::new(total_rc as u32, x.value);
						deletes += if self.put_payload_in_batch(batch, &key, &payload) {1} else {0};
					}
					None => {
						if rc < 0 {
							return Err(error_negatively_reference_hash(&key));
						}
						let payload = Payload::new(rc as u32, value);
						self.put_payload_in_batch(batch, &key, &payload);
					}
				};
				ret += 1;
			}
		}
		trace!("OverlayDB::commit() deleted {} nodes", deletes);
		Ok(ret)
	}

	/// Get the refs and value of the given key.
	fn payload(&self, key: &H256) -> Option<Payload> {
		self.backing.get(self.column, key.as_bytes())
			.expect("Low-level database error. Some issue with your hard disk?")
			.map(|ref d| decode(d).expect("decoding db value failed") )
	}

	/// Put the refs and value of the given key, possibly deleting it from the db.
	fn put_payload_in_batch(&self, batch: &mut DBTransaction, key: &H256, payload: &Payload) -> bool {
		if payload.count > 0 {
			batch.put(self.column, key.as_bytes(), &encode(payload));
			false
		} else {
			batch.delete(self.column, key.as_bytes());
			true
		}
	}

	pub fn keys(&self) -> HashMap<H256, i32> {
		let mut ret: HashMap<H256, i32> = self.backing.iter(self.column)
			.map(|(key, _)| {
				let h = H256::from_slice(&*key);
				let r = self.payload(&h).unwrap().count;
				(h, r as i32)
			})
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

impl HashDB<KeccakHasher, DBValue> for OverlayDB {
	fn get(&self, key: &H256, prefix: Prefix) -> Option<DBValue> {
		// return ok if positive; if negative, check backing - might be enough references there to make
		// it positive again.
		let k = self.overlay.raw(key, prefix);
		let memrc = {
			if let Some((d, rc)) = k {
				if rc > 0 { return Some(d.clone()); }
				rc
			} else {
				0
			}
		};
		match self.payload(key) {
			Some(x) => {
				if x.count as i32 + memrc > 0 {
					Some(x.value)
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

	fn contains(&self, key: &H256, prefix: Prefix) -> bool {
		// return ok if positive; if negative, check backing - might be enough references there to make
		// it positive again.
		let k = self.overlay.raw(key, prefix);
		match k {
			Some((_, rc)) if rc > 0 => true,
			_ => {
				let memrc = k.map_or(0, |(_, rc)| rc);
				match self.payload(key) {
					Some(x) => {
						x.count as i32 + memrc > 0
					}
					// Replace above match arm with this once https://github.com/rust-lang/rust/issues/15287 is done.
					//Some((d, rc)) if rc + memrc > 0 => true,
					_ => false,
				}
			}
		}
	}

	fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H256 { self.overlay.insert(prefix, value) }
	fn emplace(&mut self, key: H256, prefix: Prefix, value: DBValue) { self.overlay.emplace(key, prefix, value); }
	fn remove(&mut self, key: &H256, prefix: Prefix) { self.overlay.remove(key, prefix); }
}

#[cfg(test)]
mod tests {
	use hash_db::EMPTY_PREFIX;
	use super::*;

	#[test]
	fn overlaydb_revert() {
		let mut m = OverlayDB::new_temp();
		let foo = m.insert(EMPTY_PREFIX, b"foo");          // insert foo.
		let mut batch = m.backing.transaction();
		m.commit_to_batch(&mut batch).unwrap();  // commit - new operations begin here...
		m.backing.write(batch).unwrap();
		let bar = m.insert(EMPTY_PREFIX, b"bar");          // insert bar.
		m.remove(&foo, EMPTY_PREFIX);                      // remove foo.
		assert!(!m.contains(&foo, EMPTY_PREFIX));          // foo is gone.
		assert!(m.contains(&bar, EMPTY_PREFIX));           // bar is here.
	}

	#[test]
	fn overlaydb_overlay_insert_and_remove() {
		let mut trie = OverlayDB::new_temp();
		let h = trie.insert(EMPTY_PREFIX, b"hello world");
		assert_eq!(trie.get(&h, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"hello world"));
		trie.remove(&h, EMPTY_PREFIX);
		assert_eq!(trie.get(&h, EMPTY_PREFIX), None);
	}

	#[test]
	fn overlaydb_backing_insert_revert() {
		let mut trie = OverlayDB::new_temp();
		let h = trie.insert(EMPTY_PREFIX, b"hello world");
		assert_eq!(trie.get(&h, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"hello world"));
		trie.commit().unwrap();
		assert_eq!(trie.get(&h, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"hello world"));
	}

	#[test]
	fn overlaydb_backing_remove() {
		let mut trie = OverlayDB::new_temp();
		let h = trie.insert(EMPTY_PREFIX, b"hello world");
		trie.commit().unwrap();
		trie.remove(&h, EMPTY_PREFIX);
		assert_eq!(trie.get(&h, EMPTY_PREFIX), None);
		trie.commit().unwrap();
		assert_eq!(trie.get(&h, EMPTY_PREFIX), None);
	}

	#[test]
	fn overlaydb_backing_remove_revert() {
		let mut trie = OverlayDB::new_temp();
		let h = trie.insert(EMPTY_PREFIX, b"hello world");
		trie.commit().unwrap();
		trie.remove(&h, EMPTY_PREFIX);
		assert_eq!(trie.get(&h, EMPTY_PREFIX), None);
	}

	#[test]
	fn overlaydb_negative() {
		let mut trie = OverlayDB::new_temp();
		let h = trie.insert(EMPTY_PREFIX, b"hello world");
		trie.commit().unwrap();
		trie.remove(&h, EMPTY_PREFIX);
		trie.remove(&h, EMPTY_PREFIX);	//bad - sends us into negative refs.
		assert_eq!(trie.get(&h, EMPTY_PREFIX), None);
		assert!(trie.commit().is_err());
	}

	#[test]
	fn overlaydb_complex() {
		let mut trie = OverlayDB::new_temp();
		let hfoo = trie.insert(EMPTY_PREFIX, b"foo");
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		let hbar = trie.insert(EMPTY_PREFIX, b"bar");
		assert_eq!(trie.get(&hbar, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"bar"));
		trie.commit().unwrap();
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		assert_eq!(trie.get(&hbar, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"bar"));
		trie.insert(EMPTY_PREFIX, b"foo");	// two refs
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		trie.commit().unwrap();
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		assert_eq!(trie.get(&hbar, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"bar"));
		trie.remove(&hbar, EMPTY_PREFIX);		// zero refs - delete
		assert_eq!(trie.get(&hbar, EMPTY_PREFIX), None);
		trie.remove(&hfoo, EMPTY_PREFIX);		// one ref - keep
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		trie.commit().unwrap();
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		trie.remove(&hfoo, EMPTY_PREFIX);		// zero ref - would delete, but...
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX), None);
		trie.insert(EMPTY_PREFIX, b"foo");	// one ref - keep after all.
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		trie.commit().unwrap();
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX).unwrap(), DBValue::from_slice(b"foo"));
		trie.remove(&hfoo, EMPTY_PREFIX);		// zero ref - delete
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX), None);
		trie.commit().unwrap();	//
		assert_eq!(trie.get(&hfoo, EMPTY_PREFIX), None);
	}
}
