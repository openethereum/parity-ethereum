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

//! A minimal "state backend" trait: an abstraction over the sources of data
//! a blockchain state may draw upon.
//!
//! Currently assumes a very specific DB + cache structure, but
//! should become general over time to the point where not even a
//! merkle trie is strictly necessary.

use std::collections::{HashSet, HashMap};
use std::sync::Arc;

use state::Account;
use parking_lot::Mutex;
use ethereum_types::{Address, H256};
use memorydb::MemoryDB;
use hashdb::{AsHashDB, HashDB};
use kvdb::DBValue;
use keccak_hasher::KeccakHasher;

/// State backend. See module docs for more details.
pub trait Backend: Send {
	/// Treat the backend as a read-only hashdb.
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue>;

	/// Treat the backend as a writeable hashdb.
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue>;

	/// Add an account entry to the cache.
	fn add_to_account_cache(&mut self, addr: Address, data: Option<Account>, modified: bool);

	/// Add a global code cache entry. This doesn't need to worry about canonicality because
	/// it simply maps hashes to raw code and will always be correct in the absence of
	/// hash collisions.
	fn cache_code(&self, hash: H256, code: Arc<Vec<u8>>);

	/// Get basic copy of the cached account. Not required to include storage.
	/// Returns 'None' if cache is disabled or if the account is not cached.
	fn get_cached_account(&self, addr: &Address) -> Option<Option<Account>>;

	/// Get value from a cached account.
	/// `None` is passed to the closure if the account entry cached
	/// is known not to exist.
	/// `None` is returned if the entry is not cached.
	fn get_cached<F, U>(&self, a: &Address, f: F) -> Option<U>
		where F: FnOnce(Option<&mut Account>) -> U;

	/// Get cached code based on hash.
	fn get_cached_code(&self, hash: &H256) -> Option<Arc<Vec<u8>>>;

	/// Note that an account with the given address is non-null.
	fn note_non_null_account(&self, address: &Address);

	/// Check whether an account is known to be empty. Returns true if known to be
	/// empty, false otherwise.
	fn is_known_null(&self, address: &Address) -> bool;
}

/// A raw backend used to check proofs of execution.
///
/// This doesn't delete anything since execution proofs won't have mangled keys
/// and we want to avoid collisions.
// TODO: when account lookup moved into backends, this won't rely as tenuously on intended
// usage.
#[derive(Clone, PartialEq)]
pub struct ProofCheck(MemoryDB<KeccakHasher, DBValue>);

impl ProofCheck {
	/// Create a new `ProofCheck` backend from the given state items.
	pub fn new(proof: &[DBValue]) -> Self {
		let mut db = MemoryDB::<KeccakHasher, DBValue>::new();
		for item in proof { db.insert(item); }
		ProofCheck(db)
	}
}

impl HashDB<KeccakHasher, DBValue> for ProofCheck {
	fn keys(&self) -> HashMap<H256, i32> { self.0.keys() }
	fn get(&self, key: &H256) -> Option<DBValue> {
		self.0.get(key)
	}

	fn contains(&self, key: &H256) -> bool {
		self.0.contains(key)
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		self.0.insert(value)
	}

	fn emplace(&mut self, key: H256, value: DBValue) {
		self.0.emplace(key, value)
	}

	fn remove(&mut self, _key: &H256) { }
}

impl AsHashDB<KeccakHasher, DBValue> for ProofCheck {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
}

impl Backend for ProofCheck {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
	fn add_to_account_cache(&mut self, _addr: Address, _data: Option<Account>, _modified: bool) {}
	fn cache_code(&self, _hash: H256, _code: Arc<Vec<u8>>) {}
	fn get_cached_account(&self, _addr: &Address) -> Option<Option<Account>> { None }
	fn get_cached<F, U>(&self, _a: &Address, _f: F) -> Option<U>
		where F: FnOnce(Option<&mut Account>) -> U
	{
		None
	}
	fn get_cached_code(&self, _hash: &H256) -> Option<Arc<Vec<u8>>> { None }
	fn note_non_null_account(&self, _address: &Address) {}
	fn is_known_null(&self, _address: &Address) -> bool { false }
}

/// Proving state backend.
/// This keeps track of all state values loaded during usage of this backend.
/// The proof-of-execution can be extracted with `extract_proof`.
///
/// This doesn't cache anything or rely on the canonical state caches.
pub struct Proving<H: AsHashDB<KeccakHasher, DBValue>> {
	base: H, // state we're proving values from.
	changed: MemoryDB<KeccakHasher, DBValue>, // changed state via insertions.
	proof: Mutex<HashSet<DBValue>>,
}

impl<AH: AsHashDB<KeccakHasher, DBValue> + Send + Sync> AsHashDB<KeccakHasher, DBValue> for Proving<AH> {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
}

impl<H: AsHashDB<KeccakHasher, DBValue> + Send + Sync> HashDB<KeccakHasher, DBValue> for Proving<H> {
	fn keys(&self) -> HashMap<H256, i32> {
		let mut keys = self.base.as_hashdb().keys();
		keys.extend(self.changed.keys());
		keys
	}

	fn get(&self, key: &H256) -> Option<DBValue> {
		match self.base.as_hashdb().get(key) {
			Some(val) => {
				self.proof.lock().insert(val.clone());
				Some(val)
			}
			None => self.changed.get(key)
		}
	}

	fn contains(&self, key: &H256) -> bool {
		self.get(key).is_some()
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		self.changed.insert(value)
	}

	fn emplace(&mut self, key: H256, value: DBValue) {
		self.changed.emplace(key, value)
	}

	fn remove(&mut self, key: &H256) {
		// only remove from `changed`
		if self.changed.contains(key) {
			self.changed.remove(key)
		}
	}
}

impl<H: AsHashDB<KeccakHasher, DBValue> + Send + Sync> Backend for Proving<H> {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }

	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }

	fn add_to_account_cache(&mut self, _: Address, _: Option<Account>, _: bool) { }

	fn cache_code(&self, _: H256, _: Arc<Vec<u8>>) { }

	fn get_cached_account(&self, _: &Address) -> Option<Option<Account>> { None }

	fn get_cached<F, U>(&self, _: &Address, _: F) -> Option<U>
		where F: FnOnce(Option<&mut Account>) -> U
	{
		None
	}

	fn get_cached_code(&self, _: &H256) -> Option<Arc<Vec<u8>>> { None }
	fn note_non_null_account(&self, _: &Address) { }
	fn is_known_null(&self, _: &Address) -> bool { false }
}

impl<H: AsHashDB<KeccakHasher, DBValue>> Proving<H> {
	/// Create a new `Proving` over a base database.
	/// This will store all values ever fetched from that base.
	pub fn new(base: H) -> Self {
		Proving {
			base: base,
			changed: MemoryDB::<KeccakHasher, DBValue>::new(),
			proof: Mutex::new(HashSet::new()),
		}
	}

	/// Consume the backend, extracting the gathered proof in lexicographical order
	/// by value.
	pub fn extract_proof(self) -> Vec<DBValue> {
		self.proof.into_inner().into_iter().collect()
	}
}

impl<H: AsHashDB<KeccakHasher, DBValue> + Clone> Clone for Proving<H> {
	fn clone(&self) -> Self {
		Proving {
			base: self.base.clone(),
			changed: self.changed.clone(),
			proof: Mutex::new(self.proof.lock().clone()),
		}
	}
}

/// A basic backend. Just wraps the given database, directly inserting into and deleting from
/// it. Doesn't cache anything.
pub struct Basic<H>(pub H);

impl<H: AsHashDB<KeccakHasher, DBValue> + Send + Sync> Backend for Basic<H> {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> {
		self.0.as_hashdb()
	}

	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> {
		self.0.as_hashdb_mut()
	}

	fn add_to_account_cache(&mut self, _: Address, _: Option<Account>, _: bool) { }

	fn cache_code(&self, _: H256, _: Arc<Vec<u8>>) { }

	fn get_cached_account(&self, _: &Address) -> Option<Option<Account>> { None }

	fn get_cached<F, U>(&self, _: &Address, _: F) -> Option<U>
		where F: FnOnce(Option<&mut Account>) -> U
	{
		None
	}

	fn get_cached_code(&self, _: &H256) -> Option<Arc<Vec<u8>>> { None }
	fn note_non_null_account(&self, _: &Address) { }
	fn is_known_null(&self, _: &Address) -> bool { false }
}
