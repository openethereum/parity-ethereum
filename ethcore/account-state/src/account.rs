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

//! Single account in the system.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;

use ethereum_types::{Address, BigEndianHash, H256, U256};
use hash_db::HashDB;
use keccak_hash::{keccak, KECCAK_EMPTY, KECCAK_NULL_RLP};
use kvdb::DBValue;
use log::{trace, warn};
use lru_cache::LruCache;
use parity_bytes::{Bytes, ToPretty};
use rlp::{DecoderError, encode};
use trie_db::{Recorder, Trie};
use common_types::basic_account::BasicAccount;
use ethtrie::{Result as TrieResult, SecTrieDB, TrieDB, TrieFactory};
use keccak_hasher::KeccakHasher;
use pod::PodAccount;

const STORAGE_CACHE_ITEMS: usize = 8192;

/// Boolean type for clean/dirty status.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Filth {
	/// Data has not been changed.
	Clean,
	/// Data has been changed.
	Dirty,
}

/// Single account in the system.
/// Keeps track of changes to the code and storage.
/// The changes are applied in `commit_storage` and `commit_code`
pub struct Account {
	// Balance of the account.
	balance: U256,
	// Nonce of the account.
	nonce: U256,
	// Trie-backed storage.
	storage_root: H256,
	// LRU Cache of the trie-backed storage.
	// This is limited to `STORAGE_CACHE_ITEMS` recent queries
	storage_cache: RefCell<LruCache<H256, H256>>,
	// LRU Cache of the trie-backed storage for original value.
	// This is only used when the initial storage root is different compared to
	// what is in the database. That is, it is only used for new contracts.
	original_storage_cache: Option<(H256, RefCell<LruCache<H256, H256>>)>,
	// Modified storage. Accumulates changes to storage made in `set_storage`
	// Takes precedence over `storage_cache`.
	storage_changes: HashMap<H256, H256>,
	// Code hash of the account.
	code_hash: H256,
	// Size of the account code.
	code_size: Option<usize>,
	// Code cache of the account.
	code_cache: Arc<Bytes>,
	// Version of the account.
	code_version: U256,
	// Account code new or has been modified.
	code_filth: Filth,
	// Cached address hash.
	address_hash: Cell<Option<H256>>,
}

impl From<BasicAccount> for Account {
	fn from(basic: BasicAccount) -> Self {
		Account {
			balance: basic.balance,
			nonce: basic.nonce,
			storage_root: basic.storage_root,
			storage_cache: Self::empty_storage_cache(),
			original_storage_cache: None,
			storage_changes: HashMap::new(),
			code_hash: basic.code_hash,
			code_size: None,
			code_cache: Arc::new(vec![]),
			code_version: basic.code_version,
			code_filth: Filth::Clean,
			address_hash: Cell::new(None),
		}
	}
}

impl Account {
	#[cfg(test)]
	/// General constructor.
	pub fn new(balance: U256, nonce: U256, storage: HashMap<H256, H256>, code: Bytes, version: U256) -> Account {
		Account {
			balance: balance,
			nonce: nonce,
			storage_root: KECCAK_NULL_RLP,
			storage_cache: Self::empty_storage_cache(),
			original_storage_cache: None,
			storage_changes: storage,
			code_hash: keccak(&code),
			code_size: Some(code.len()),
			code_cache: Arc::new(code),
			code_version: version,
			code_filth: Filth::Dirty,
			address_hash: Cell::new(None),
		}
	}

	fn empty_storage_cache() -> RefCell<LruCache<H256, H256>> {
		RefCell::new(LruCache::new(STORAGE_CACHE_ITEMS))
	}

	/// General constructor.
	pub fn from_pod(pod: PodAccount) -> Account {
		Account {
			balance: pod.balance,
			nonce: pod.nonce,
			storage_root: KECCAK_NULL_RLP,
			storage_cache: Self::empty_storage_cache(),
			original_storage_cache: None,
			storage_changes: pod.storage.into_iter().collect(),
			code_hash: pod.code.as_ref().map_or(KECCAK_EMPTY, |c| keccak(c)),
			code_filth: Filth::Dirty,
			code_size: Some(pod.code.as_ref().map_or(0, |c| c.len())),
			code_cache: Arc::new(pod.code.map_or_else(|| { warn!("POD account with unknown code is being created! Assuming no code."); vec![] }, |c| c)),
			code_version: pod.version,
			address_hash: Cell::new(None),
		}
	}

	/// Convert Account to a PodAccount.
	/// NOTE: This will silently fail unless the account is fully cached.
	pub fn to_pod(&self) -> PodAccount {
		PodAccount {
			balance: self.balance,
			nonce: self.nonce,
			storage: self.storage_changes.iter().fold(BTreeMap::new(), |mut m, (k, v)| {
				m.insert(k.clone(), v.clone());
				m
			}),
			code: self.code().map(|x| x.to_vec()),
			version: self.code_version,
		}
	}

	/// Create a new account with the given balance.
	pub fn new_basic(balance: U256, nonce: U256) -> Account {
		Account {
			balance: balance,
			nonce: nonce,
			storage_root: KECCAK_NULL_RLP,
			storage_cache: Self::empty_storage_cache(),
			original_storage_cache: None,
			storage_changes: HashMap::new(),
			code_hash: KECCAK_EMPTY,
			code_cache: Arc::new(vec![]),
			code_size: Some(0),
			code_version: U256::zero(),
			code_filth: Filth::Clean,
			address_hash: Cell::new(None),
		}
	}

	/// Create a new account from RLP.
	pub fn from_rlp(rlp: &[u8]) -> Result<Account, DecoderError> {
		::rlp::decode::<BasicAccount>(rlp)
			.map(|ba| ba.into())
	}

	/// Create a new contract account.
	/// NOTE: make sure you use `init_code` on this before `commit`ing.
	pub fn new_contract(balance: U256, nonce: U256, version: U256, original_storage_root: H256) -> Account {
		Account {
			balance,
			nonce,
			storage_root: KECCAK_NULL_RLP,
			storage_cache: Self::empty_storage_cache(),
			original_storage_cache: if original_storage_root == KECCAK_NULL_RLP {
				None
			} else {
				Some((original_storage_root, Self::empty_storage_cache()))
			},
			storage_changes: HashMap::new(),
			code_hash: KECCAK_EMPTY,
			code_cache: Arc::new(vec![]),
			code_size: None,
			code_version: version,
			code_filth: Filth::Clean,
			address_hash: Cell::new(None),
		}
	}

	/// Set this account's code to the given code.
	/// NOTE: Account should have been created with `new_contract()`
	pub fn init_code(&mut self, code: Bytes) {
		self.code_hash = keccak(&code);
		self.code_cache = Arc::new(code);
		self.code_size = Some(self.code_cache.len());
		self.code_filth = Filth::Dirty;
	}

	/// Reset this account's code to the given code.
	pub fn reset_code(&mut self, code: Bytes) {
		self.init_code(code);
	}

	/// Reset this account's code and storage to given values.
	pub fn reset_code_and_storage(&mut self, code: Arc<Bytes>, storage: HashMap<H256, H256>) {
		self.code_hash = keccak(&*code);
		self.code_cache = code;
		self.code_size = Some(self.code_cache.len());
		self.code_filth = Filth::Dirty;
		self.storage_cache = Self::empty_storage_cache();
		self.storage_changes = storage;
		if self.storage_root != KECCAK_NULL_RLP {
			self.original_storage_cache = Some((self.storage_root, Self::empty_storage_cache()));
		}
		self.storage_root = KECCAK_NULL_RLP;
	}

	/// Set (and cache) the contents of the trie's storage at `key` to `value`.
	pub fn set_storage(&mut self, key: H256, value: H256) {
		self.storage_changes.insert(key, value);
	}

	/// Get (and cache) the contents of the trie's storage at `key`.
	/// Takes modified storage into account.
	pub fn storage_at(&self, db: &dyn HashDB<KeccakHasher, DBValue>, key: &H256) -> TrieResult<H256> {
		if let Some(value) = self.cached_storage_at(key) {
			return Ok(value);
		}
		Self::get_and_cache_storage(
			&self.storage_root,
			&mut self.storage_cache.borrow_mut(),
			db,
			key)
	}

	/// Get (and cache) the contents of the trie's storage at `key`.
	/// Does not take modified storage into account.
	pub fn original_storage_at(&self, db: &dyn HashDB<KeccakHasher, DBValue>, key: &H256) -> TrieResult<H256> {
		if let Some(value) = self.cached_original_storage_at(key) {
			return Ok(value);
		}
		match &self.original_storage_cache {
			Some((ref original_storage_root, ref original_storage_cache)) =>
				Self::get_and_cache_storage(
					original_storage_root,
					&mut original_storage_cache.borrow_mut(),
					db,
					key
				),
			None =>
				Self::get_and_cache_storage(
					&self.storage_root,
					&mut self.storage_cache.borrow_mut(),
					db,
					key
				),
		}
	}

	fn get_and_cache_storage(storage_root: &H256, storage_cache: &mut LruCache<H256, H256>, db: &dyn HashDB<KeccakHasher, DBValue>, key: &H256) -> TrieResult<H256> {
		let db = SecTrieDB::new(&db, storage_root)?;
		let panicky_decoder = |bytes:&[u8]| ::rlp::decode(&bytes).expect("decoding db value failed");
		let item: U256 = db.get_with(key.as_bytes(), panicky_decoder)?.unwrap_or_else(U256::zero);
		let value: H256 = BigEndianHash::from_uint(&item);
		storage_cache.insert(key.clone(), value.clone());
		Ok(value)
	}

	/// Get cached storage value if any. Returns `None` if the
	/// key is not in the cache.
	pub fn cached_storage_at(&self, key: &H256) -> Option<H256> {
		if let Some(value) = self.storage_changes.get(key) {
			return Some(value.clone())
		}
		self.cached_moved_original_storage_at(key)
	}

	/// Get cached original storage value after last state commitment. Returns `None` if the key is not in the cache.
	pub fn cached_original_storage_at(&self, key: &H256) -> Option<H256> {
		match &self.original_storage_cache {
			Some((_, ref original_storage_cache)) => {
				if let Some(value) = original_storage_cache.borrow_mut().get_mut(key) {
					Some(value.clone())
				} else {
					None
				}
			},
			None => {
				self.cached_moved_original_storage_at(key)
			},
		}
	}

	/// Get cached original storage value since last contract creation on this address. Returns `None` if the key is not in the cache.
	fn cached_moved_original_storage_at(&self, key: &H256) -> Option<H256> {
		// If storage root is empty RLP, then early return zero value. Practically, this makes it so that if
		// `original_storage_cache` is used, then `storage_cache` will always remain empty.
		if self.storage_root == KECCAK_NULL_RLP {
			return Some(H256::zero());
		}

		if let Some(value) = self.storage_cache.borrow_mut().get_mut(key) {
			Some(value.clone())
		} else {
			None
		}
	}

	/// return the balance associated with this account.
	pub fn balance(&self) -> &U256 { &self.balance }

	/// return the nonce associated with this account.
	pub fn nonce(&self) -> &U256 { &self.nonce }

	/// return the code version associated with this account.
	pub fn code_version(&self) -> &U256 { &self.code_version }

	/// return the code hash associated with this account.
	pub fn code_hash(&self) -> H256 {
		self.code_hash.clone()
	}

	/// return and cache `keccak(address)`, `address` must be the address of this
	/// account.
	pub fn address_hash(&self, address: &Address) -> H256 {
		let hash = self.address_hash.get();
		hash.unwrap_or_else(|| {
			let hash = keccak(address);
			self.address_hash.set(Some(hash.clone()));
			hash
		})
	}

	/// returns the account's code. If `None` then the code cache isn't available -
	/// get someone who knows to call `note_code`.
	pub fn code(&self) -> Option<Arc<Bytes>> {
		if self.code_hash != KECCAK_EMPTY && self.code_cache.is_empty() {
			return None;
		}
		Some(self.code_cache.clone())
	}

	/// returns the account's code size. If `None` then the code cache or code size cache isn't available -
	/// get someone who knows to call `note_code`.
	pub fn code_size(&self) -> Option<usize> {
		self.code_size.clone()
	}

	#[cfg(test)]
	/// Provide a byte array which hashes to the `code_hash`. returns the hash as a result.
	pub fn note_code(&mut self, code: Bytes) -> Result<(), H256> {
		let h = keccak(&code);
		if self.code_hash == h {
			self.code_cache = Arc::new(code);
			self.code_size = Some(self.code_cache.len());
			Ok(())
		} else {
			Err(h)
		}
	}

	/// Is `code_cache` valid; such that code is going to return Some?
	pub fn is_cached(&self) -> bool {
		!self.code_cache.is_empty() || (self.code_cache.is_empty() && self.code_hash == KECCAK_EMPTY)
	}

	/// Provide a database to get `code_hash`. Should not be called if it is a contract without code. Returns the cached code, if successful.
	#[must_use]
	pub fn cache_code(&mut self, db: &dyn HashDB<KeccakHasher, DBValue>) -> Option<Arc<Bytes>> {
		// TODO: fill out self.code_cache;
		trace!("Account::cache_code: ic={}; self.code_hash={:?}, self.code_cache={}", self.is_cached(), self.code_hash, self.code_cache.pretty());

		if self.is_cached() { return Some(self.code_cache.clone()); }

		match db.get(&self.code_hash, hash_db::EMPTY_PREFIX) {
			Some(x) => {
				self.code_size = Some(x.len());
				self.code_cache = Arc::new(x.into_vec());
				Some(self.code_cache.clone())
			},
			_ => {
				warn!("Failed reverse get of {}", self.code_hash);
				None
			},
		}
	}

	/// Provide code to cache. For correctness, should be the correct code for the account.
	pub fn cache_given_code(&mut self, code: Arc<Bytes>) {
		trace!("Account::cache_given_code: ic={}; self.code_hash={:?}, self.code_cache={}", self.is_cached(), self.code_hash, self.code_cache.pretty());

		self.code_size = Some(code.len());
		self.code_cache = code;
	}

	/// Provide a database to get `code_size`. Should not be called if it is a contract without code. Returns whether
	/// the cache succeeds.
	#[must_use]
	pub fn cache_code_size(&mut self, db: &dyn HashDB<KeccakHasher, DBValue>) -> bool {
		// TODO: fill out self.code_cache;
		trace!("Account::cache_code_size: ic={}; self.code_hash={:?}, self.code_cache={}", self.is_cached(), self.code_hash, self.code_cache.pretty());
		self.code_size.is_some() ||
			if self.code_hash != KECCAK_EMPTY {
				match db.get(&self.code_hash, hash_db::EMPTY_PREFIX) {
					Some(x) => {
						self.code_size = Some(x.len());
						true
					},
					_ => {
						warn!("Failed reverse get of {}", self.code_hash);
						false
					},
				}
			} else {
				// If the code hash is empty hash, then the code size is zero.
				self.code_size = Some(0);
				true
			}
	}

	/// Determine whether there are any un-`commit()`-ed storage-setting operations.
	pub fn storage_is_clean(&self) -> bool { self.storage_changes.is_empty() }

	/// Check if account has zero nonce, balance, no code and no storage.
	///
	/// NOTE: Will panic if `!self.storage_is_clean()`
	pub fn is_empty(&self) -> bool {
		assert!(self.storage_is_clean(), "Account::is_empty() may only legally be called when storage is clean.");
		self.is_null() && self.storage_root == KECCAK_NULL_RLP
	}

	/// Check if account has zero nonce, balance, no code.
	pub fn is_null(&self) -> bool {
		self.balance.is_zero() &&
		self.nonce.is_zero() &&
		self.code_hash == KECCAK_EMPTY
	}

	/// Check if account is basic (Has no code).
	pub fn is_basic(&self) -> bool {
		self.code_hash == KECCAK_EMPTY
	}

	/// Return the storage root associated with this account or None if it has been altered via the overlay.
	pub fn storage_root(&self) -> Option<H256> {
		if self.storage_is_clean() {
			Some(self.storage_root)
		} else {
			None
		}
	}

	/// Return the original storage root of this account.
	pub fn original_storage_root(&self) -> H256 {
		if let Some((original_storage_root, _)) = self.original_storage_cache {
			original_storage_root
		} else {
			self.storage_root
		}
	}

	/// Whether the base storage root of this account is unchanged.
	pub fn is_base_storage_root_unchanged(&self) -> bool {
		self.original_storage_cache.is_none()
	}

	/// Storage root where the account changes are based upon.
	pub fn base_storage_root(&self) -> H256 {
		self.storage_root
	}

	/// Return the storage overlay.
	pub fn storage_changes(&self) -> &HashMap<H256, H256> { &self.storage_changes }

	/// Increment the nonce of the account by one.
	pub fn inc_nonce(&mut self) {
		self.nonce = self.nonce.saturating_add(U256::from(1u8));
	}

	/// Increase account balance.
	pub fn add_balance(&mut self, x: &U256) {
		self.balance = self.balance.saturating_add(*x);
	}

	/// Decrease account balance.
	/// Panics if balance is less than `x`
	pub fn sub_balance(&mut self, x: &U256) {
		assert!(self.balance >= *x);
		self.balance = self.balance - *x;
	}

	/// Commit the `storage_changes` to the backing DB and update `storage_root`.
	pub fn commit_storage(&mut self, trie_factory: &TrieFactory, db: &mut dyn HashDB<KeccakHasher, DBValue>) -> TrieResult<()> {
		let mut t = trie_factory.from_existing(db, &mut self.storage_root)?;
		for (k, v) in self.storage_changes.drain() {
			// cast key and value to trait type,
			// so we can call overloaded `to_bytes` method
			match v.is_zero() {
				true => t.remove(k.as_bytes())?,
				false => t.insert(k.as_bytes(), &encode(&v.into_uint()))?,
			};

			self.storage_cache.borrow_mut().insert(k, v);
		}
		self.original_storage_cache = None;
		Ok(())
	}

	/// Commit any unsaved code. `code_hash` will always return the hash of the `code_cache` after this.
	pub fn commit_code(&mut self, db: &mut dyn HashDB<KeccakHasher, DBValue>) {
		trace!("Commiting code of {:?} - {:?}, {:?}", self, self.code_filth == Filth::Dirty, self.code_cache.is_empty());
		match (self.code_filth == Filth::Dirty, self.code_cache.is_empty()) {
			(true, true) => {
				self.code_size = Some(0);
				self.code_filth = Filth::Clean;
			},
			(true, false) => {
				db.emplace(self.code_hash.clone(), hash_db::EMPTY_PREFIX, DBValue::from_slice(&*self.code_cache));
				self.code_size = Some(self.code_cache.len());
				self.code_filth = Filth::Clean;
			},
			(false, _) => {},
		}
	}

	/// Export to RLP.
	pub fn rlp(&self) -> Bytes {
		let basic = BasicAccount {
			nonce: self.nonce,
			balance: self.balance,
			storage_root: self.storage_root,
			code_hash: self.code_hash,
			code_version: self.code_version,
		};

		rlp::encode(&basic)
	}

	/// Clone basic account data
	pub fn clone_basic(&self) -> Account {
		Account {
			balance: self.balance.clone(),
			nonce: self.nonce.clone(),
			storage_root: self.storage_root.clone(),
			storage_cache: Self::empty_storage_cache(),
			original_storage_cache: self.original_storage_cache.as_ref().map(|(r, _)| (*r, Self::empty_storage_cache())),
			storage_changes: HashMap::new(),
			code_hash: self.code_hash.clone(),
			code_size: self.code_size.clone(),
			code_cache: self.code_cache.clone(),
			code_version: self.code_version,
			code_filth: self.code_filth,
			address_hash: self.address_hash.clone(),
		}
	}

	/// Clone account data and dirty storage keys
	pub fn clone_dirty(&self) -> Account {
		let mut account = self.clone_basic();
		account.storage_changes = self.storage_changes.clone();
		account
	}

	/// Clone account data, dirty storage keys and cached storage keys.
	pub fn clone_all(&self) -> Account {
		let mut account = self.clone_dirty();
		account.storage_cache = self.storage_cache.clone();
		account.original_storage_cache = self.original_storage_cache.clone();
		account
	}

	/// Replace self with the data from other account merging storage cache.
	/// Basic account data and all modifications are overwritten
	/// with new values.
	pub fn overwrite_with(&mut self, other: Account) {
		self.balance = other.balance;
		self.nonce = other.nonce;
		self.code_hash = other.code_hash;
		self.code_filth = other.code_filth;
		self.code_cache = other.code_cache;
		self.code_size = other.code_size;
		self.code_version = other.code_version;
		self.address_hash = other.address_hash;
		if self.storage_root == other.storage_root {
			let mut cache = self.storage_cache.borrow_mut();
			for (k, v) in other.storage_cache.into_inner() {
				cache.insert(k, v);
			}
		} else {
			self.storage_cache = other.storage_cache;
		}
		self.original_storage_cache = other.original_storage_cache;
		self.storage_root = other.storage_root;
		self.storage_changes = other.storage_changes;
	}
}

// light client storage proof.
impl Account {
	/// Prove a storage key's existence or nonexistence in the account's storage
	/// trie.
	/// `storage_key` is the hash of the desired storage key, meaning
	/// this will only work correctly under a secure trie.
	pub fn prove_storage(&self, db: &dyn HashDB<KeccakHasher, DBValue>, storage_key: H256) -> TrieResult<(Vec<Bytes>, H256)> {
		let mut recorder = Recorder::new();

		let trie = TrieDB::new(&db, &self.storage_root)?;
		let item: U256 = {
			let panicky_decoder = |bytes:&[u8]| ::rlp::decode(bytes).expect("decoding db value failed");
			let query = (&mut recorder, panicky_decoder);
			trie.get_with(storage_key.as_bytes(), query)?.unwrap_or_else(U256::zero)
		};

		Ok((recorder.drain().into_iter().map(|r| r.data).collect(), BigEndianHash::from_uint(&item)))
	}
}

impl fmt::Debug for Account {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Account")
			.field("balance", &self.balance)
			.field("nonce", &self.nonce)
			.field("code", &self.code())
			.field("storage", &self.storage_changes.iter().collect::<BTreeMap<_, _>>())
			.finish()
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use ethereum_types::{Address, H256};
	use parity_bytes::Bytes;

	use account_db::*;
	use journaldb::new_memory_db;
	use rlp_compress::{compress, decompress, snapshot_swapper};

	use super::*;

	#[test]
	fn account_compress() {
		let raw = Account::new_basic(2.into(), 4.into()).rlp();
		let compact_vec = compress(&raw, snapshot_swapper());
		assert!(raw.len() > compact_vec.len());
		let again_raw = decompress(&compact_vec, snapshot_swapper());
		assert_eq!(raw, again_raw.into_vec());
	}

	#[test]
	fn storage_at() {
		let mut db = new_memory_db();
		let mut db = AccountDBMut::from_hash(&mut db, keccak(&Address::zero()));
		let rlp = {
			let mut a = Account::new_contract(69.into(), 0.into(), 0.into(), KECCAK_NULL_RLP);
			a.set_storage(H256::zero(), H256::from_low_u64_be(0x1234));
			a.commit_storage(&Default::default(), &mut db).unwrap();
			a.init_code(vec![]);
			a.commit_code(&mut db);
			a.rlp()
		};

		let a = Account::from_rlp(&rlp).expect("decoding db value failed");
		assert_eq!(a.storage_root().unwrap(), H256::from_str("c57e1afb758b07f8d2c8f13a3b6e44fa5ff94ab266facc5a4fd3f062426e50b2").unwrap());
		assert_eq!(a.storage_at(&db.immutable(), &H256::zero()).unwrap(), H256::from_low_u64_be(0x1234));
		assert_eq!(a.storage_at(&db.immutable(), &H256::from_low_u64_be(0x01)).unwrap(), H256::zero());
	}

	#[test]
	fn note_code() {
		let mut db = new_memory_db();
		let mut db = AccountDBMut::from_hash(&mut db, keccak(&Address::zero()));

		let rlp = {
			let mut a = Account::new_contract(69.into(), 0.into(), 0.into(), KECCAK_NULL_RLP);
			a.init_code(vec![0x55, 0x44, 0xffu8]);
			a.commit_code(&mut db);
			a.rlp()
		};

		let mut a = Account::from_rlp(&rlp).expect("decoding db value failed");
		assert!(a.cache_code(&db.immutable()).is_some());

		let mut a = Account::from_rlp(&rlp).expect("decoding db value failed");
		assert_eq!(a.note_code(vec![0x55, 0x44, 0xffu8]), Ok(()));
	}

	#[test]
	fn commit_storage() {
		let mut a = Account::new_contract(69.into(), 0.into(), 0.into(), KECCAK_NULL_RLP);
		let mut db = new_memory_db();
		let mut db = AccountDBMut::from_hash(&mut db, keccak(&Address::zero()));
		a.set_storage(H256::from_low_u64_be(0), H256::from_low_u64_be(0x1234));
		assert_eq!(a.storage_root(), None);
		a.commit_storage(&Default::default(), &mut db).unwrap();
		assert_eq!(a.storage_root().unwrap(), H256::from_str("c57e1afb758b07f8d2c8f13a3b6e44fa5ff94ab266facc5a4fd3f062426e50b2").unwrap());
	}

	#[test]
	fn commit_remove_commit_storage() {
		let mut a = Account::new_contract(69.into(), 0.into(), 0.into(), KECCAK_NULL_RLP);
		let mut db = new_memory_db();
		let mut db = AccountDBMut::from_hash(&mut db, keccak(&Address::zero()));
		a.set_storage(H256::from_low_u64_be(0), H256::from_low_u64_be(0x1234));
		a.commit_storage(&Default::default(), &mut db).unwrap();
		a.set_storage(H256::from_low_u64_be(1), H256::from_low_u64_be(0x1234));
		a.commit_storage(&Default::default(), &mut db).unwrap();
		a.set_storage(H256::from_low_u64_be(1), H256::from_low_u64_be(0));
		a.commit_storage(&Default::default(), &mut db).unwrap();
		assert_eq!(a.storage_root().unwrap(), H256::from_str("c57e1afb758b07f8d2c8f13a3b6e44fa5ff94ab266facc5a4fd3f062426e50b2").unwrap());
	}

	#[test]
	fn commit_code() {
		let mut a = Account::new_contract(69.into(), 0.into(), 0.into(), KECCAK_NULL_RLP);
		let mut db = new_memory_db();
		let mut db = AccountDBMut::from_hash(&mut db, keccak(&Address::zero()));
		a.init_code(vec![0x55, 0x44, 0xffu8]);
		assert_eq!(a.code_filth, Filth::Dirty);
		assert_eq!(a.code_size(), Some(3));
		a.commit_code(&mut db);
		assert_eq!(a.code_hash(), H256::from_str("af231e631776a517ca23125370d542873eca1fb4d613ed9b5d5335a46ae5b7eb").unwrap());
	}

	#[test]
	fn reset_code() {
		let mut a = Account::new_contract(69.into(), 0.into(), 0.into(), KECCAK_NULL_RLP);
		let mut db = new_memory_db();
		let mut db = AccountDBMut::from_hash(&mut db, keccak(&Address::zero()));
		a.init_code(vec![0x55, 0x44, 0xffu8]);
		assert_eq!(a.code_filth, Filth::Dirty);
		a.commit_code(&mut db);
		assert_eq!(a.code_filth, Filth::Clean);
		assert_eq!(a.code_hash(), H256::from_str("af231e631776a517ca23125370d542873eca1fb4d613ed9b5d5335a46ae5b7eb").unwrap());
		a.reset_code(vec![0x55]);
		assert_eq!(a.code_filth, Filth::Dirty);
		a.commit_code(&mut db);
		assert_eq!(a.code_hash(), H256::from_str("37bf2238b11b68cdc8382cece82651b59d3c3988873b6e0f33d79694aa45f1be").unwrap());
	}

	#[test]
	fn rlpio() {
		let a = Account::new(69u8.into(), 0u8.into(), HashMap::new(), Bytes::new(), 0.into());
		let b = Account::from_rlp(&a.rlp()).unwrap();
		assert_eq!(a.balance(), b.balance());
		assert_eq!(a.nonce(), b.nonce());
		assert_eq!(a.code_hash(), b.code_hash());
		assert_eq!(a.storage_root(), b.storage_root());
	}

	#[test]
	fn new_account() {
		let a = Account::new(69u8.into(), 0u8.into(), HashMap::new(), Bytes::new(), 0.into());
		assert_eq!(a.rlp().to_hex(), "f8448045a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
		assert_eq!(*a.balance(), 69u8.into());
		assert_eq!(*a.nonce(), 0u8.into());
		assert_eq!(a.code_hash(), KECCAK_EMPTY);
		assert_eq!(a.storage_root().unwrap(), KECCAK_NULL_RLP);
	}
}
