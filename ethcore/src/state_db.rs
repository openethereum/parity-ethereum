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

use lru_cache::LruCache;
use util::journaldb::JournalDB;
use util::hash::H256;
use util::hashdb::HashDB;
use util::{Arc, Address, DBTransaction, UtilError, Mutex};
use account::Account;
use util::{Hashable, BytesConvertable};
use bloomfilter::Bloom;

const STATE_CACHE_ITEMS: usize = 65536;

struct AccountCache {
	accounts: LruCache<Address, Option<Account>>,
}

/// State database abstraction.
/// Manages shared global state cache.
/// A clone of `StateDB` may be created as canonical or not.
/// For canonical clones cache changes are accumulated and applied
/// on commit.
/// For non-canonical clones cache is cleared on commit.
pub struct StateDB {
	db: Box<JournalDB>,
	account_cache: Arc<Mutex<AccountCache>>,
	cache_overlay: Vec<(Address, Option<Account>)>,
	account_bloom: Arc<Mutex<Bloom>>,
	is_canon: bool,
}

pub const ACCOUNT_BLOOM_SPACE: usize = 1048576;
pub const DEFAULT_ACCOUNT_PRESET: usize = 1000000;

pub const ACCOUNT_BLOOM_SPACE_COLUMN: &'static[u8] = b"accounts_bloom";
pub const ACCOUNT_BLOOM_HASHCOUNT_COLUMN: &'static[u8] = b"account_hash_count";

impl StateDB {

	fn new_account_bloom() -> Bloom {
		Bloom::new(ACCOUNT_BLOOM_SPACE, DEFAULT_ACCOUNT_PRESET)
	}

	pub fn check_account_bloom(&self, address: &Address) -> bool {
		trace!(target: "state_bloom", "Check account bloom: {:?}", address);
		let bloom = self.account_bloom.lock();
		bloom.check(address.sha3().as_slice())
	}

	pub fn note_account_bloom(&self, address: &Address) {
		trace!(target: "state_bloom", "Note account bloom: {:?}", address);
		let mut bloom = self.account_bloom.lock();
		bloom.set(address.sha3().as_slice());
	}

	/// Create a new instance wrapping `JournalDB`
	pub fn new(db: Box<JournalDB>) -> StateDB {
		let bloom = match db.backing().get(None, ACCOUNT_BLOOM_SPACE_COLUMN).expect("Low-level database error") {
			Some(val) => {
				if val.len() != ACCOUNT_BLOOM_SPACE {
					Self::new_account_bloom()
				}
				else {
					let hash_count_bytes  =
						db.backing()
							.get(None, ACCOUNT_BLOOM_HASHCOUNT_COLUMN)
							.expect("Low-level database error")
							.expect("account_bloom present but account_hash_count is not");
					assert_eq!(hash_count_bytes.len(), 1);
					let hash_count = hash_count_bytes[0];
					Bloom::from_bytes(&val, hash_count as u32)
				}
			}
			None => Self::new_account_bloom(),
		};

		StateDB {
			db: db,
			account_cache: Arc::new(Mutex::new(AccountCache { accounts: LruCache::new(STATE_CACHE_ITEMS) })),
			cache_overlay: Vec::new(),
			is_canon: false,
			account_bloom: Arc::new(Mutex::new(bloom)),
		}
	}

	/// Commit all recent insert operations and canonical historical commits' removals from the
	/// old era to the backing database, reverting any non-canonical historical commit's inserts.
	pub fn commit(&mut self, batch: &DBTransaction, now: u64, id: &H256, end: Option<(u64, H256)>) -> Result<u32, UtilError> {
		// commit bloom
		let (bloom_bytes, hash_count) = self.account_bloom.lock().to_bytes();
		try!(batch.put(None, ACCOUNT_BLOOM_SPACE_COLUMN, &bloom_bytes));
		assert!(hash_count <= 255);
		try!(batch.put(None, ACCOUNT_BLOOM_HASHCOUNT_COLUMN, &vec![hash_count as u8]));

		// commit cache
		let records = try!(self.db.commit(batch, now, id, end));
		if self.is_canon {
			self.commit_cache();
		} else {
			self.clear_cache();
		}
		Ok(records)
	}

	/// Returns an interface to HashDB.
	pub fn as_hashdb(&self) -> &HashDB {
		self.db.as_hashdb()
	}

	/// Returns an interface to mutable HashDB.
	pub fn as_hashdb_mut(&mut self) -> &mut HashDB {
		self.db.as_hashdb_mut()
	}

	/// Clone the database.
	pub fn boxed_clone(&self) -> StateDB {
		StateDB {
			db: self.db.boxed_clone(),
			account_cache: self.account_cache.clone(),
			cache_overlay: Vec::new(),
			is_canon: false,
			account_bloom: self.account_bloom.clone(),
		}
	}

	/// Clone the database for a canonical state.
	pub fn boxed_clone_canon(&self) -> StateDB {
		StateDB {
			db: self.db.boxed_clone(),
			account_cache: self.account_cache.clone(),
			cache_overlay: Vec::new(),
			is_canon: true,
			account_bloom: self.account_bloom.clone(),
		}
	}

	/// Check if pruning is enabled on the database.
	pub fn is_pruned(&self) -> bool {
		self.db.is_pruned()
	}

	/// Heap size used.
	pub fn mem_used(&self) -> usize {
		self.db.mem_used() //TODO: + self.account_cache.lock().heap_size_of_children()
	}

	/// Returns underlying `JournalDB`.
	pub fn journal_db(&self) -> &JournalDB {
		&*self.db
	}

	/// Enqueue cache change.
	pub fn cache_account(&mut self, addr: Address, data: Option<Account>) {
		self.cache_overlay.push((addr, data));
	}

	/// Apply pending cache changes.
	fn commit_cache(&mut self) {
		let mut cache = self.account_cache.lock();
		for (address, account) in self.cache_overlay.drain(..) {
			if let Some(&mut Some(ref mut existing)) = cache.accounts.get_mut(&address) {
				if let Some(new) = account {
					existing.merge_with(new);
					continue;
				}
			}
			cache.accounts.insert(address, account);
		}
	}

	/// Clear the cache.
	pub fn clear_cache(&mut self) {
		self.cache_overlay.clear();
		let mut cache = self.account_cache.lock();
		cache.accounts.clear();
	}

	/// Get basic copy of the cached account. Does not include storage.
	/// Returns 'None' if the state is non-canonical and cache is disabled
	/// or if the account is not cached.
	pub fn get_cached_account(&self, addr: &Address) -> Option<Option<Account>> {
		if !self.is_canon {
			return None;
		}
		let mut cache = self.account_cache.lock();
		cache.accounts.get_mut(&addr).map(|a| a.as_ref().map(|a| a.clone_basic()))
	}

	/// Get value from a cached account.
	/// Returns 'None' if the state is non-canonical and cache is disabled
	/// or if the account is not cached.
	pub fn get_cached<F, U>(&self, a: &Address, f: F) -> Option<U>
		where F: FnOnce(Option<&mut Account>) -> U {
		if !self.is_canon {
			return None;
		}
		let mut cache = self.account_cache.lock();
		cache.accounts.get_mut(a).map(|c| f(c.as_mut()))
	}
}

