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

use std::cell::{RefCell, RefMut};
use std::collections::hash_map::Entry;
use common::*;
use engines::Engine;
use executive::{Executive, TransactOptions};
use factory::Factories;
use trace::FlatTrace;
use pod_account::*;
use pod_state::{self, PodState};
use types::state_diff::StateDiff;
use state_db::StateDB;

mod account;
mod substate;

pub use self::account::Account;
pub use self::substate::Substate;

/// Used to return information about an `State::apply` operation.
pub struct ApplyOutcome {
	/// The receipt for the applied transaction.
	pub receipt: Receipt,
	/// The trace for the applied transaction, if None if tracing is disabled.
	pub trace: Vec<FlatTrace>,
}

/// Result type for the execution ("application") of a transaction.
pub type ApplyResult = Result<ApplyOutcome, Error>;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
/// Account modification state. Used to check if the account was
/// Modified in between commits and overall.
enum AccountState {
	/// Account was never modified in this state object.
	Clean,
	/// Account has been modified and is not committed to the trie yet.
	/// This is set than any of the account data is changed, including
	/// storage and code.
	Dirty,
	/// Account was modified and committed to the trie.
	Commited,
}

#[derive(Debug)]
/// In-memory copy of the account data. Holds the optional account
/// and the modification status.
/// Account entry can contain existing (`Some`) or non-existing
/// account (`None`)
struct AccountEntry {
	account: Option<Account>,
	state: AccountState,
}

// Account cache item. Contains account data and
// modification state
impl AccountEntry {
	fn is_dirty(&self) -> bool {
		self.state == AccountState::Dirty
	}

	/// Clone dirty data into new `AccountEntry`. This includes
	/// basic account data and modified storage keys.
	/// Returns None if clean.
	fn clone_if_dirty(&self) -> Option<AccountEntry> {
		match self.is_dirty() {
			true => Some(self.clone_dirty()),
			false => None,
		}
	}

	/// Clone dirty data into new `AccountEntry`. This includes
	/// basic account data and modified storage keys.
	fn clone_dirty(&self) -> AccountEntry {
		AccountEntry {
			account: self.account.as_ref().map(Account::clone_dirty),
			state: self.state,
		}
	}

	// Create a new account entry and mark it as dirty.
	fn new_dirty(account: Option<Account>) -> AccountEntry {
		AccountEntry {
			account: account,
			state: AccountState::Dirty,
		}
	}

	// Create a new account entry and mark it as clean.
	fn new_clean(account: Option<Account>) -> AccountEntry {
		AccountEntry {
			account: account,
			state: AccountState::Clean,
		}
	}

	// Replace data with another entry but preserve storage cache.
	fn overwrite_with(&mut self, other: AccountEntry) {
		self.state = other.state;
		match other.account {
			Some(acc) => match self.account {
				Some(ref mut ours) => {
					ours.overwrite_with(acc);
				},
				None => {},
			},
			None => self.account = None,
		}
	}
}

/// Representation of the entire state of all accounts in the system.
///
/// `State` can work together with `StateDB` to share account cache.
///
/// Local cache contains changes made locally and changes accumulated
/// locally from previous commits. Global cache reflects the database
/// state and never contains any changes.
///
/// Cache items contains account data, or the flag that account does not exist
/// and modification state (see `AccountState`)
///
/// Account data can be in the following cache states:
/// * In global but not local - something that was queried from the database,
/// but never modified
/// * In local but not global - something that was just added (e.g. new account)
/// * In both with the same value - something that was changed to a new value,
/// but changed back to a previous block in the same block (same State instance)
/// * In both with different values - something that was overwritten with a
/// new value.
///
/// All read-only state queries check local cache/modifications first,
/// then global state cache. If data is not found in any of the caches
/// it is loaded from the DB to the local cache.
///
/// **** IMPORTANT *************************************************************
/// All the modifications to the account data must set the `Dirty` state in the
/// `AccountEntry`. This is done in `require` and `require_or_from`. So just
/// use that.
/// ****************************************************************************
///
/// Upon destruction all the local cache data merged into the global cache.
/// The merge might be rejected if current state is non-canonical.
///
/// State snapshotting.
///
/// A new snapshot can be created with `snapshot()`. Snapshots can be
/// created in a hierarchy.
/// When a snapshot is active all changes are applied directly into
/// `cache` and the original value is copied into an active snapshot.
/// Reverting a snapshot with `revert_to_snapshot` involves copying
/// original values from the latest snapshot back into `cache`. The code
/// takes care not to overwrite cached storage while doing that.
/// Snapshot can be discateded with `discard_snapshot`. All of the orignal
/// backed-up values are moved into a parent snapshot (if any).
///
pub struct State {
	db: StateDB,
	root: H256,
	cache: RefCell<HashMap<Address, AccountEntry>>,
	// The original account is preserved in
	snapshots: RefCell<Vec<HashMap<Address, Option<AccountEntry>>>>,
	account_start_nonce: U256,
	factories: Factories,
}

#[derive(Copy, Clone)]
enum RequireCache {
	None,
	CodeSize,
	Code,
}

const SEC_TRIE_DB_UNWRAP_STR: &'static str = "A state can only be created with valid root. Creating a SecTrieDB with a valid root will not fail. \
			 Therefore creating a SecTrieDB with this state's root will not fail.";

impl State {
	/// Creates new state with empty state root
	#[cfg(test)]
	pub fn new(mut db: StateDB, account_start_nonce: U256, factories: Factories) -> State {
		let mut root = H256::new();
		{
			// init trie and reset root too null
			let _ = factories.trie.create(db.as_hashdb_mut(), &mut root);
		}

		State {
			db: db,
			root: root,
			cache: RefCell::new(HashMap::new()),
			snapshots: RefCell::new(Vec::new()),
			account_start_nonce: account_start_nonce,
			factories: factories,
		}
	}

	/// Creates new state with existing state root
	pub fn from_existing(db: StateDB, root: H256, account_start_nonce: U256, factories: Factories) -> Result<State, TrieError> {
		if !db.as_hashdb().contains(&root) {
			return Err(TrieError::InvalidStateRoot(root));
		}

		let state = State {
			db: db,
			root: root,
			cache: RefCell::new(HashMap::new()),
			snapshots: RefCell::new(Vec::new()),
			account_start_nonce: account_start_nonce,
			factories: factories
		};

		Ok(state)
	}

	/// Create a recoverable snaphot of this state.
	pub fn snapshot(&mut self) {
		self.snapshots.borrow_mut().push(HashMap::new());
	}

	/// Merge last snapshot with previous.
	pub fn discard_snapshot(&mut self) {
		// merge with previous snapshot
		let last = self.snapshots.borrow_mut().pop();
		if let Some(mut snapshot) = last {
			if let Some(ref mut prev) = self.snapshots.borrow_mut().last_mut() {
				if prev.is_empty() {
					**prev = snapshot;
				} else {
					for (k, v) in snapshot.drain() {
						prev.entry(k).or_insert(v);
					}
				}
			}
		}
	}

	/// Revert to the last snapshot and discard it.
	pub fn revert_to_snapshot(&mut self) {
		if let Some(mut snapshot) = self.snapshots.borrow_mut().pop() {
			for (k, v) in snapshot.drain() {
				match v {
					Some(v) => {
						match self.cache.borrow_mut().entry(k) {
							Entry::Occupied(mut e) => {
								// Merge snapshotted changes back into the main account
								// storage preserving the cache.
								e.get_mut().overwrite_with(v);
							},
							Entry::Vacant(e) => {
								e.insert(v);
							}
						}
					},
					None => {
						match self.cache.borrow_mut().entry(k) {
							Entry::Occupied(e) => {
								if e.get().is_dirty() {
									e.remove();
								}
							},
							_ => {}
						}
					}
				}
			}
		}
	}

	fn insert_cache(&self, address: &Address, account: AccountEntry) {
		// Dirty account which is not in the cache means this is a new account.
		// It goes directly into the snapshot as there's nothing to rever to.
		//
		// In all other cases account is read as clean first, and after that made
		// dirty in and added to the snapshot with `note_cache`.
		if account.is_dirty() {
			if let Some(ref mut snapshot) = self.snapshots.borrow_mut().last_mut() {
				if !snapshot.contains_key(address) {
					snapshot.insert(address.clone(), self.cache.borrow_mut().insert(address.clone(), account));
					return;
				}
			}
		}
		self.cache.borrow_mut().insert(address.clone(), account);
	}

	fn note_cache(&self, address: &Address) {
		if let Some(ref mut snapshot) = self.snapshots.borrow_mut().last_mut() {
			if !snapshot.contains_key(address) {
				snapshot.insert(address.clone(), self.cache.borrow().get(address).map(AccountEntry::clone_dirty));
			}
		}
	}

	/// Destroy the current object and return root and database.
	pub fn drop(mut self) -> (H256, StateDB) {
		self.commit_cache();
		(self.root, self.db)
	}

	/// Return reference to root
	pub fn root(&self) -> &H256 {
		&self.root
	}

	/// Create a new contract at address `contract`. If there is already an account at the address
	/// it will have its code reset, ready for `init_code()`.
	pub fn new_contract(&mut self, contract: &Address, balance: U256) {
		self.insert_cache(contract, AccountEntry::new_dirty(Some(Account::new_contract(balance, self.account_start_nonce))));
	}

	/// Remove an existing account.
	pub fn kill_account(&mut self, account: &Address) {
		self.insert_cache(account, AccountEntry::new_dirty(None));
	}

	/// Determine whether an account exists.
	pub fn exists(&self, a: &Address) -> bool {
		self.ensure_cached(a, RequireCache::None, |a| a.is_some())
	}

	/// Get the balance of account `a`.
	pub fn balance(&self, a: &Address) -> U256 {
		self.ensure_cached(a, RequireCache::None,
			|a| a.as_ref().map_or(U256::zero(), |account| *account.balance()))
	}

	/// Get the nonce of account `a`.
	pub fn nonce(&self, a: &Address) -> U256 {
		self.ensure_cached(a, RequireCache::None,
			|a| a.as_ref().map_or(self.account_start_nonce, |account| *account.nonce()))
	}

	/// Mutate storage of account `address` so that it is `value` for `key`.
	pub fn storage_at(&self, address: &Address, key: &H256) -> H256 {
		// Storage key search and update works like this:
		// 1. If there's an entry for the account in the local cache check for the key and return it if found.
		// 2. If there's an entry for the account in the global cache check for the key or load it into that account.
		// 3. If account is missing in the global cache load it into the local cache and cache the key there.

		// check local cache first without updating
		{
			let local_cache = self.cache.borrow_mut();
			let mut local_account = None;
			if let Some(maybe_acc) = local_cache.get(address) {
				match maybe_acc.account {
					Some(ref account) => {
						if let Some(value) = account.cached_storage_at(key) {
							return value;
						} else {
							local_account = Some(maybe_acc);
						}
					},
					_ => return H256::new(),
				}
			}
			// check the global cache and and cache storage key there if found,
			// otherwise cache the account localy and cache storage key there.
			if let Some(result) = self.db.get_cached(address, |acc| acc.map_or(H256::new(), |a| {
					let account_db = self.factories.accountdb.readonly(self.db.as_hashdb(), a.address_hash(address));
					a.storage_at(account_db.as_hashdb(), key)
				})) {
				return result;
			}
			if let Some(ref mut acc) = local_account {
				if let Some(ref account) = acc.account {
					let account_db = self.factories.accountdb.readonly(self.db.as_hashdb(), account.address_hash(address));
					return account.storage_at(account_db.as_hashdb(), key)
				} else {
					return H256::new()
				}
			}
		}

		// check bloom before any requests to trie
		if !self.db.check_account_bloom(address) { return H256::zero() }

		// account is not found in the global cache, get from the DB and insert into local
		let db = self.factories.trie.readonly(self.db.as_hashdb(), &self.root).expect(SEC_TRIE_DB_UNWRAP_STR);
		let maybe_acc = match db.get(address) {
			Ok(acc) => acc.map(Account::from_rlp),
			Err(e) => panic!("Potential DB corruption encountered: {}", e),
		};
		let r = maybe_acc.as_ref().map_or(H256::new(), |a| {
			let account_db = self.factories.accountdb.readonly(self.db.as_hashdb(), a.address_hash(address));
			a.storage_at(account_db.as_hashdb(), key)
		});
		self.insert_cache(address, AccountEntry::new_clean(maybe_acc));
		r
	}

	/// Get accounts' code.
	pub fn code(&self, a: &Address) -> Option<Arc<Bytes>> {
		self.ensure_cached(a, RequireCache::Code,
			|a| a.as_ref().map_or(None, |a| a.code().clone()))
	}

	pub fn code_hash(&self, a: &Address) -> H256 {
		self.ensure_cached(a, RequireCache::None,
			|a| a.as_ref().map_or(SHA3_EMPTY, |a| a.code_hash()))
	}

	/// Get accounts' code size.
	pub fn code_size(&self, a: &Address) -> Option<usize> {
		self.ensure_cached(a, RequireCache::CodeSize,
			|a| a.as_ref().and_then(|a| a.code_size()))
	}

	/// Add `incr` to the balance of account `a`.
	pub fn add_balance(&mut self, a: &Address, incr: &U256) {
		trace!(target: "state", "add_balance({}, {}): {}", a, incr, self.balance(a));
		if !incr.is_zero() || !self.exists(a) {
			self.require(a, false).add_balance(incr);
		}
	}

	/// Subtract `decr` from the balance of account `a`.
	pub fn sub_balance(&mut self, a: &Address, decr: &U256) {
		trace!(target: "state", "sub_balance({}, {}): {}", a, decr, self.balance(a));
		if !decr.is_zero() || !self.exists(a) {
			self.require(a, false).sub_balance(decr);
		}
	}

	/// Subtracts `by` from the balance of `from` and adds it to that of `to`.
	pub fn transfer_balance(&mut self, from: &Address, to: &Address, by: &U256) {
		self.sub_balance(from, by);
		self.add_balance(to, by);
	}

	/// Increment the nonce of account `a` by 1.
	pub fn inc_nonce(&mut self, a: &Address) {
		self.require(a, false).inc_nonce()
	}

	/// Mutate storage of account `a` so that it is `value` for `key`.
	pub fn set_storage(&mut self, a: &Address, key: H256, value: H256) {
		if self.storage_at(a, &key) != value {
			self.require(a, false).set_storage(key, value)
		}
	}

	/// Initialise the code of account `a` so that it is `code`.
	/// NOTE: Account should have been created with `new_contract`.
	pub fn init_code(&mut self, a: &Address, code: Bytes) {
		self.require_or_from(a, true, || Account::new_contract(0.into(), self.account_start_nonce), |_|{}).init_code(code);
	}

	/// Reset the code of account `a` so that it is `code`.
	pub fn reset_code(&mut self, a: &Address, code: Bytes) {
		self.require_or_from(a, true, || Account::new_contract(0.into(), self.account_start_nonce), |_|{}).reset_code(code);
	}

	/// Execute a given transaction.
	/// This will change the state accordingly.
	pub fn apply(&mut self, env_info: &EnvInfo, engine: &Engine, t: &SignedTransaction, tracing: bool) -> ApplyResult {
//		let old = self.to_pod();

		let options = TransactOptions { tracing: tracing, vm_tracing: false, check_nonce: true };
		let vm_factory = self.factories.vm.clone();
		let e = try!(Executive::new(self, env_info, engine, &vm_factory).transact(t, options));

		// TODO uncomment once to_pod() works correctly.
//		trace!("Applied transaction. Diff:\n{}\n", state_diff::diff_pod(&old, &self.to_pod()));
		try!(self.commit());
		let receipt = Receipt::new(self.root().clone(), e.cumulative_gas_used, e.logs);
		trace!(target: "state", "Transaction receipt: {:?}", receipt);
		Ok(ApplyOutcome{receipt: receipt, trace: e.trace})
	}

	/// Commit accounts to SecTrieDBMut. This is similar to cpp-ethereum's dev::eth::commit.
	/// `accounts` is mutable because we may need to commit the code or storage and record that.
	#[cfg_attr(feature="dev", allow(match_ref_pats))]
	fn commit_into(
		factories: &Factories,
		db: &mut StateDB,
		root: &mut H256,
		accounts: &mut HashMap<Address, AccountEntry>
	) -> Result<(), Error> {
		// first, commit the sub trees.
		for (address, ref mut a) in accounts.iter_mut().filter(|&(_, ref a)| a.is_dirty()) {
			match a.account {
				Some(ref mut account) => {
					db.note_account_bloom(&address);
					let addr_hash = account.address_hash(address);
					let mut account_db = factories.accountdb.create(db.as_hashdb_mut(), addr_hash);
					account.commit_storage(&factories.trie, account_db.as_hashdb_mut());
					account.commit_code(account_db.as_hashdb_mut());
				}
				_ => {}
			}
		}

		{
			let mut trie = factories.trie.from_existing(db.as_hashdb_mut(), root).unwrap();
			for (address, ref mut a) in accounts.iter_mut().filter(|&(_, ref a)| a.is_dirty()) {
				a.state = AccountState::Commited;
				match a.account {
					Some(ref mut account) => {
						try!(trie.insert(address, &account.rlp()));
					},
					None => {
						try!(trie.remove(address));
					},
				}
			}
		}

		Ok(())
	}

	fn commit_cache(&mut self) {
		let mut addresses = self.cache.borrow_mut();
		trace!("Committing cache {:?} entries", addresses.len());
		for (address, a) in addresses.drain().filter(|&(_, ref a)| !a.is_dirty()) {
			self.db.cache_account(address, a.account);
		}
	}

	/// Commits our cached account changes into the trie.
	pub fn commit(&mut self) -> Result<(), Error> {
		assert!(self.snapshots.borrow().is_empty());
		Self::commit_into(&self.factories, &mut self.db, &mut self.root, &mut *self.cache.borrow_mut())
	}

	/// Clear state cache
	pub fn clear(&mut self) {
		self.cache.borrow_mut().clear();
	}

	#[cfg(test)]
	#[cfg(feature = "json-tests")]
	/// Populate the state from `accounts`.
	pub fn populate_from(&mut self, accounts: PodState) {
		assert!(self.snapshots.borrow().is_empty());
		for (add, acc) in accounts.drain().into_iter() {
			self.db.note_account_bloom(&add);
			self.cache.borrow_mut().insert(add, AccountEntry::new_dirty(Some(Account::from_pod(acc))));
		}
	}

	/// Populate a PodAccount map from this state.
	pub fn to_pod(&self) -> PodState {
		assert!(self.snapshots.borrow().is_empty());
		// TODO: handle database rather than just the cache.
		// will need fat db.
		PodState::from(self.cache.borrow().iter().fold(BTreeMap::new(), |mut m, (add, opt)| {
			if let Some(ref acc) = opt.account {
				m.insert(add.clone(), PodAccount::from_account(acc));
			}
			m
		}))
	}

	fn query_pod(&mut self, query: &PodState) {
		for (address, pod_account) in query.get() {
			self.ensure_cached(address, RequireCache::Code, |a| {
				if a.is_some() {
					for key in pod_account.storage.keys() {
						self.storage_at(address, key);
					}
				}
			});
		}
	}

	/// Returns a `StateDiff` describing the difference from `orig` to `self`.
	/// Consumes self.
	pub fn diff_from(&self, orig: State) -> StateDiff {
		let pod_state_post = self.to_pod();
		let mut state_pre = orig;
		state_pre.query_pod(&pod_state_post);
		pod_state::diff_pod(&state_pre.to_pod(), &pod_state_post)
	}

	fn update_account_cache(require: RequireCache, account: &mut Account, db: &HashDB) {
		match require {
			RequireCache::None => {},
			RequireCache::Code => {
				account.cache_code(db);
			}
			RequireCache::CodeSize => {
				account.cache_code_size(db);
			}
		}
	}

	/// Check caches for required data
	/// First searches for account in the local, then the shared cache.
	/// Populates local cache if nothing found.
	fn ensure_cached<F, U>(&self, a: &Address, require: RequireCache, f: F) -> U
		where F: Fn(Option<&Account>) -> U {
		// check local cache first
		if let Some(ref mut maybe_acc) = self.cache.borrow_mut().get_mut(a) {
			if let Some(ref mut account) = maybe_acc.account {
				let accountdb = self.factories.accountdb.readonly(self.db.as_hashdb(), account.address_hash(a));
				Self::update_account_cache(require, account, accountdb.as_hashdb());
				return f(Some(account));
			}
			return f(None);
		}
		// check global cache
		let result = self.db.get_cached(a, |mut acc| {
			if let Some(ref mut account) = acc {
				let accountdb = self.factories.accountdb.readonly(self.db.as_hashdb(), account.address_hash(a));
				Self::update_account_cache(require, account, accountdb.as_hashdb());
			}
			f(acc.map(|a| &*a))
		});
		match result {
			Some(r) => r,
			None => {
				// first check bloom if it is not in database for sure
				if !self.db.check_account_bloom(a) { return f(None); }

				// not found in the global cache, get from the DB and insert into local
				let db = self.factories.trie.readonly(self.db.as_hashdb(), &self.root).expect(SEC_TRIE_DB_UNWRAP_STR);
				let mut maybe_acc = match db.get(a) {
					Ok(acc) => acc.map(Account::from_rlp),
					Err(e) => panic!("Potential DB corruption encountered: {}", e),
				};
				if let Some(ref mut account) = maybe_acc.as_mut() {
					let accountdb = self.factories.accountdb.readonly(self.db.as_hashdb(), account.address_hash(a));
					Self::update_account_cache(require, account, accountdb.as_hashdb());
				}
				let r = f(maybe_acc.as_ref());
				match maybe_acc {
					Some(account) => self.insert_cache(a, AccountEntry::new_clean(Some(account))),
					None => self.insert_cache(a, AccountEntry::new_clean(None)),
				}
				r
			}
		}
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	fn require<'a>(&'a self, a: &Address, require_code: bool) -> RefMut<'a, Account> {
		self.require_or_from(a, require_code, || Account::new_basic(U256::from(0u8), self.account_start_nonce), |_|{})
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	/// If it doesn't exist, make account equal the evaluation of `default`.
	fn require_or_from<'a, F: FnOnce() -> Account, G: FnOnce(&mut Account)>(&'a self, a: &Address, require_code: bool, default: F, not_default: G)
		-> RefMut<'a, Account>
	{
		let contains_key = self.cache.borrow().contains_key(a);
		if !contains_key {
			match self.db.get_cached_account(a) {
				Some(Some(acc)) => self.insert_cache(a, AccountEntry::new_clean(Some(acc))),
				Some(None) => self.insert_cache(a, AccountEntry::new_clean(None)),
				None => {
					let maybe_acc = if self.db.check_account_bloom(a) {
						let db = self.factories.trie.readonly(self.db.as_hashdb(), &self.root).expect(SEC_TRIE_DB_UNWRAP_STR);
						let maybe_acc = match db.get(a) {
							Ok(Some(acc)) => AccountEntry::new_clean(Some(Account::from_rlp(acc))),
							Ok(None) => AccountEntry::new_clean(None),
							Err(e) => panic!("Potential DB corruption encountered: {}", e),
						};
						maybe_acc
					}
					else {
						AccountEntry::new_clean(None)
					};
					self.insert_cache(a, maybe_acc);
				}
			}
		}
		self.note_cache(a);

		match &mut self.cache.borrow_mut().get_mut(a).unwrap().account {
			&mut Some(ref mut acc) => not_default(acc),
			slot => *slot = Some(default()),
		}

		// at this point the account is guaranteed to be in the cache.
		RefMut::map(self.cache.borrow_mut(), |c| {
			let mut entry = c.get_mut(a).unwrap();
			// set the dirty flag after changing account data.
			entry.state = AccountState::Dirty;
			match entry.account {
				Some(ref mut account) => {
					if require_code {
						let addr_hash = account.address_hash(a);
						let accountdb = self.factories.accountdb.readonly(self.db.as_hashdb(), addr_hash);
						account.cache_code(accountdb.as_hashdb());
					}
					account
				},
				_ => panic!("Required account must always exist; qed"),
			}
		})
	}
}

impl fmt::Debug for State {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.cache.borrow())
	}
}

impl Clone for State {
	fn clone(&self) -> State {
		let cache = {
			let mut cache: HashMap<Address, AccountEntry> = HashMap::new();
			for (key, val) in self.cache.borrow().iter() {
				if let Some(entry) = val.clone_if_dirty() {
					cache.insert(key.clone(), entry);
				}
			}
			cache
		};

		State {
			db: self.db.boxed_clone(),
			root: self.root.clone(),
			cache: RefCell::new(cache),
			snapshots: RefCell::new(Vec::new()),
			account_start_nonce: self.account_start_nonce.clone(),
			factories: self.factories.clone(),
		}
	}
}

#[cfg(test)]
mod tests {

use std::sync::Arc;
use std::str::FromStr;
use rustc_serialize::hex::FromHex;
use super::*;
use util::{U256, H256, FixedHash, Address, Hashable};
use tests::helpers::*;
use devtools::*;
use env_info::*;
use spec::*;
use transaction::*;
use util::log::init_log;
use trace::{FlatTrace, TraceError, trace};
use types::executed::CallType;

#[test]
fn should_apply_create_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Create,
		value: 100.into(),
		data: FromHex::from_hex("601080600c6000396000f3006000355415600957005b60203560003555").unwrap(),
	}.sign(&"".sha3());

	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 0,
		action: trace::Action::Create(trace::Create {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			value: 100.into(),
			gas: 77412.into(),
			init: vec![96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85],
		}),
		result: trace::Res::Create(trace::CreateResult {
			gas_used: U256::from(3224),
			address: Address::from_str("8988167e088c87cd314df6d3c2b83da5acb93ace").unwrap(),
			code: vec![96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53]
		}),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_work_when_cloned() {
	init_log();

	let a = Address::zero();

	let temp = RandomTempPath::new();
	let mut state = {
		let mut state = get_temp_state_in(temp.as_path());
		assert_eq!(state.exists(&a), false);
		state.inc_nonce(&a);
		state.commit().unwrap();
		state.clone()
	};

	state.inc_nonce(&a);
	state.commit().unwrap();
}

#[test]
fn should_trace_failed_create_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Create,
		value: 100.into(),
		data: FromHex::from_hex("5b600056").unwrap(),
	}.sign(&"".sha3());

	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		action: trace::Action::Create(trace::Create {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			value: 100.into(),
			gas: 78792.into(),
			init: vec![91, 96, 0, 86],
		}),
		result: trace::Res::FailedCreate(TraceError::OutOfGas),
		subtraces: 0
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(3),
			output: vec![]
		}),
		subtraces: 0,
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_basic_call_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(0),
			output: vec![]
		}),
		subtraces: 0,
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_transaction_to_builtin() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = &*Spec::new_test().engine;

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0x1.into()),
		value: 0.into(),
		data: vec![],
	}.sign(&"".sha3());

	let result = state.apply(&info, engine, &t, true).unwrap();

	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: "0000000000000000000000000000000000000001".into(),
			value: 0.into(),
			gas: 79_000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(3000),
			output: vec![]
		}),
		subtraces: 0,
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_not_trace_subcall_transaction_to_builtin() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = &*Spec::new_test().engine;

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 0.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("600060006000600060006001610be0f1").unwrap());
	let result = state.apply(&info, engine, &t, true).unwrap();

	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 0.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(28_061),
			output: vec![]
		}),
		subtraces: 0,
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_not_trace_callcode() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = &*Spec::new_test().engine;

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 0.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("60006000600060006000600b611000f2").unwrap());
	state.init_code(&0xb.into(), FromHex::from_hex("6000").unwrap());
	let result = state.apply(&info, engine, &t, true).unwrap();

	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 0.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: 64.into(),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: 0xa.into(),
			to: 0xa.into(),
			value: 0.into(),
			gas: 4096.into(),
			input: vec![],
			call_type: CallType::CallCode,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: 3.into(),
			output: vec![],
		}),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_not_trace_delegatecall() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	info.number = 0x789b0;
	let engine = &*Spec::new_test().engine;

	println!("schedule.have_delegate_call: {:?}", engine.schedule(&info).have_delegate_call);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 0.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("6000600060006000600b618000f4").unwrap());
	state.init_code(&0xb.into(), FromHex::from_hex("6000").unwrap());
	let result = state.apply(&info, engine, &t, true).unwrap();

	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 0.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(61),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 0.into(),
			gas: 32768.into(),
			input: vec![],
			call_type: CallType::DelegateCall,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: 3.into(),
			output: vec![],
		}),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_failed_call_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("5b600056").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::FailedCall(TraceError::OutOfGas),
		subtraces: 0,
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_with_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&0xb.into(), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();

	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(69),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: 0xa.into(),
			to: 0xb.into(),
			value: 0.into(),
			gas: 78934.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(3),
			output: vec![]
		}),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_with_basic_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("60006000600060006045600b6000f1").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(31761),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: 0xa.into(),
			to: 0xb.into(),
			value: 69.into(),
			gas: 2300.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult::default()),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_not_trace_call_with_invalid_basic_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("600060006000600060ff600b6000f1").unwrap());	// not enough funds.
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(31761),
			output: vec![]
		}),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_failed_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],//600480600b6000396000f35b600056
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&0xb.into(), FromHex::from_hex("5b600056").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(79_000),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: 0xa.into(),
			to: 0xb.into(),
			value: 0.into(),
			gas: 78934.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::FailedCall(TraceError::OutOfGas),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_with_subcall_with_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&0xb.into(), FromHex::from_hex("60006000600060006000600c602b5a03f1").unwrap());
	state.init_code(&0xc.into(), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(135),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: 0xa.into(),
			to: 0xb.into(),
			value: 0.into(),
			gas: 78934.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(69),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0, 0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: 0xb.into(),
			to: 0xc.into(),
			value: 0.into(),
			gas: 78868.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(3),
			output: vec![]
		}),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_failed_subcall_with_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],//600480600b6000396000f35b600056
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&0xb.into(), FromHex::from_hex("60006000600060006000600c602b5a03f1505b601256").unwrap());
	state.init_code(&0xc.into(), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &(100.into()));
	let result = state.apply(&info, &engine, &t, true).unwrap();

	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(79_000),
			output: vec![]
		})
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 1,
			action: trace::Action::Call(trace::Call {
			from: 0xa.into(),
			to: 0xb.into(),
			value: 0.into(),
			gas: 78934.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::FailedCall(TraceError::OutOfGas),
	}, FlatTrace {
		trace_address: vec![0, 0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Call(trace::Call {
			from: 0xb.into(),
			to: 0xc.into(),
			value: 0.into(),
			gas: 78868.into(),
			call_type: CallType::Call,
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(3),
			output: vec![]
		}),
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_suicide() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = 1_000_000.into();
	let engine = TestEngine::new(5);

	let t = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 100_000.into(),
		action: Action::Call(0xa.into()),
		value: 100.into(),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&0xa.into(), FromHex::from_hex("73000000000000000000000000000000000000000bff").unwrap());
	state.add_balance(&0xa.into(), &50.into());
	state.add_balance(t.sender().as_ref().unwrap(), &100.into());
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = vec![FlatTrace {
		trace_address: Default::default(),
		subtraces: 1,
		action: trace::Action::Call(trace::Call {
			from: "9cce34f7ab185c7aba1b7c8140d620b4bda941d6".into(),
			to: 0xa.into(),
			value: 100.into(),
			gas: 79000.into(),
			input: vec![],
			call_type: CallType::Call,
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: 3.into(),
			output: vec![]
		}),
	}, FlatTrace {
		trace_address: vec![0].into_iter().collect(),
		subtraces: 0,
		action: trace::Action::Suicide(trace::Suicide {
			address: 0xa.into(),
			refund_address: 0xb.into(),
			balance: 150.into(),
		}),
		result: trace::Res::None,
	}];

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn code_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.require_or_from(&a, false, ||Account::new_contract(42.into(), 0.into()), |_|{});
		state.init_code(&a, vec![1, 2, 3]);
		assert_eq!(state.code(&a), Some(Arc::new([1u8, 2, 3].to_vec())));
		state.commit().unwrap();
		assert_eq!(state.code(&a), Some(Arc::new([1u8, 2, 3].to_vec())));
		state.drop()
	};

	let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
	assert_eq!(state.code(&a), Some(Arc::new([1u8, 2, 3].to_vec())));
}

#[test]
fn storage_at_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.set_storage(&a, H256::from(&U256::from(1u64)), H256::from(&U256::from(69u64)));
		state.commit().unwrap();
		state.drop()
	};

	let s = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
	assert_eq!(s.storage_at(&a, &H256::from(&U256::from(1u64))), H256::from(&U256::from(69u64)));
}

#[test]
fn get_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.inc_nonce(&a);
		state.add_balance(&a, &U256::from(69u64));
		state.commit().unwrap();
		assert_eq!(state.balance(&a), U256::from(69u64));
		state.drop()
	};

	let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
	assert_eq!(state.balance(&a), U256::from(69u64));
	assert_eq!(state.nonce(&a), U256::from(1u64));
}

#[test]
fn remove() {
	let a = Address::zero();
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	assert_eq!(state.exists(&a), false);
	state.inc_nonce(&a);
	assert_eq!(state.exists(&a), true);
	assert_eq!(state.nonce(&a), U256::from(1u64));
	state.kill_account(&a);
	assert_eq!(state.exists(&a), false);
	assert_eq!(state.nonce(&a), U256::from(0u64));
}

#[test]
fn remove_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.inc_nonce(&a);
		state.commit().unwrap();
		assert_eq!(state.exists(&a), true);
		assert_eq!(state.nonce(&a), U256::from(1u64));
		state.drop()
	};

	let (root, db) = {
		let mut state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
		assert_eq!(state.exists(&a), true);
		assert_eq!(state.nonce(&a), U256::from(1u64));
		state.kill_account(&a);
		state.commit().unwrap();
		assert_eq!(state.exists(&a), false);
		assert_eq!(state.nonce(&a), U256::from(0u64));
		state.drop()
	};

	let state = State::from_existing(db, root, U256::from(0u8), Default::default()).unwrap();
	assert_eq!(state.exists(&a), false);
	assert_eq!(state.nonce(&a), U256::from(0u64));
}

#[test]
fn alter_balance() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	let b = 1u64.into();
	state.add_balance(&a, &U256::from(69u64));
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.commit().unwrap();
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.sub_balance(&a, &U256::from(42u64));
	assert_eq!(state.balance(&a), U256::from(27u64));
	state.commit().unwrap();
	assert_eq!(state.balance(&a), U256::from(27u64));
	state.transfer_balance(&a, &b, &U256::from(18u64));
	assert_eq!(state.balance(&a), U256::from(9u64));
	assert_eq!(state.balance(&b), U256::from(18u64));
	state.commit().unwrap();
	assert_eq!(state.balance(&a), U256::from(9u64));
	assert_eq!(state.balance(&b), U256::from(18u64));
}

#[test]
fn alter_nonce() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.inc_nonce(&a);
	assert_eq!(state.nonce(&a), U256::from(1u64));
	state.inc_nonce(&a);
	assert_eq!(state.nonce(&a), U256::from(2u64));
	state.commit().unwrap();
	assert_eq!(state.nonce(&a), U256::from(2u64));
	state.inc_nonce(&a);
	assert_eq!(state.nonce(&a), U256::from(3u64));
	state.commit().unwrap();
	assert_eq!(state.nonce(&a), U256::from(3u64));
}

#[test]
fn balance_nonce() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	assert_eq!(state.balance(&a), U256::from(0u64));
	assert_eq!(state.nonce(&a), U256::from(0u64));
	state.commit().unwrap();
	assert_eq!(state.balance(&a), U256::from(0u64));
	assert_eq!(state.nonce(&a), U256::from(0u64));
}

#[test]
fn ensure_cached() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.require(&a, false);
	state.commit().unwrap();
	assert_eq!(state.root().hex(), "0ce23f3c809de377b008a4a3ee94a0834aac8bec1f86e28ffe4fdb5a15b0c785");
}

#[test]
fn snapshot_basic() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.snapshot();
	state.add_balance(&a, &U256::from(69u64));
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.discard_snapshot();
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.snapshot();
	state.add_balance(&a, &U256::from(1u64));
	assert_eq!(state.balance(&a), U256::from(70u64));
	state.revert_to_snapshot();
	assert_eq!(state.balance(&a), U256::from(69u64));
}

#[test]
fn snapshot_nested() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.snapshot();
	state.snapshot();
	state.add_balance(&a, &U256::from(69u64));
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.discard_snapshot();
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.revert_to_snapshot();
	assert_eq!(state.balance(&a), U256::from(0));
}

#[test]
fn create_empty() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	state.commit().unwrap();
	assert_eq!(state.root().hex(), "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");
}

}
