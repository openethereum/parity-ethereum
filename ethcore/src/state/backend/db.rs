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

use rlp;
use util::trie::{Trie, TrieError, TrieFactory};
use util::{Bytes, H256, Address, Hashable, U256, Uint};

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use account_db::Factory as AccountDBFactory;
use error::Error;
use state::{self, Account, AccountEntry, SEC_TRIE_DB_UNWRAP_STR};
use state_db::StateDB;

/// Fully database-backed state backend, with account-key mangling.
/// This is expected to be a full node.
///
/// Uses `StateDB` for caching, although it may be broken up into composable
/// backends in the future.
pub struct Database {
	backing: StateDB,
	root: H256,
	trie_factory: TrieFactory,
	db_factory: AccountDBFactory,
	// TODO: move this into StateDB so it's persistent.
	address_hashes: RefCell<HashMap<Address, H256>>,
}

impl Database {
	/// Create a new database backend with empty state root.
	#[cfg(test)]
	pub fn new(mut backing: StateDB, trie_factory: TrieFactory, db_factory: AccountDBFactory) -> Self {
		let mut root = Default::default();

		{
			let _ = trie_factory.create(backing.as_hashdb_mut(), &mut root);
		}

		Database {
			backing: backing,
			root: root,
			trie_factory: trie_factory,
			address_hashes: RefCell::new(HashMap::new()),
			db_factory: db_factory
		}
	}

	/// Create a new database backend.
	pub fn from_existing(backing: StateDB, root: H256, factory: TrieFactory, db_factory: AccountDBFactory) -> Result<Self, TrieError> {
		if !backing.as_hashdb().contains(&root) {
			return Err(TrieError::InvalidStateRoot(root));
		}

		Ok(Database {
			backing: backing,
			root: root,
			trie_factory: factory,
			address_hashes: RefCell::new(HashMap::new()),
			db_factory: db_factory
		})
	}

	/// Consume the backend, turning it into its components.
	pub fn into_inner(self) -> (H256, StateDB) {
		(self.root, self.backing)
	}

	/// Commit local cache into the state db.
	// TODO: refactor this out into backend trait. `CachingBackend`?
	pub fn commit_cache(&mut self, cache: &mut HashMap<Address, AccountEntry>) {
		for (address, a) in cache.drain() {
			match a {
				AccountEntry::Cached(account) => {
					if !account.is_dirty() {
						self.backing.cache_account(address, Some(account));
					}
				},
				AccountEntry::Missing => {
					self.backing.cache_account(address, None);
				},
				_ => {},
			}
		}
	}

	// get the mapped address hash for the given address.
	fn addr_hash(&self, address: Address) -> H256 {
		self.address_hashes.borrow_mut().entry(address.clone())
			.or_insert_with(|| address.sha3()).clone()
	}
}

impl Clone for Database {
	fn clone(&self) -> Self {
		Database {
			backing: self.backing.boxed_clone(),
			root: self.root.clone(),
			trie_factory: self.trie_factory.clone(),
			db_factory: self.db_factory.clone(),
			address_hashes: RefCell::new(self.address_hashes.borrow().clone())
		}
	}
}

impl state::Backend for Database {
	fn code(&self, address: Address, code_hash: &H256) -> Option<Arc<Bytes>> {
		let addr_hash = self.addr_hash(address.clone());

		// first check the global state cache.
		match self.backing.get_cached(&address, |acc| acc.and_then(|acc| acc.code())) {
			Some(code) => code,
			None => {
				// if that fails, do a DB lookup.
				self.db_factory
					.readonly(self.backing.as_hashdb(), addr_hash)
					.get(code_hash).map(|b| Arc::new(b.to_owned()))
			}
		}
	}

	fn account(&self, address: &Address) -> Option<Account> {
		if let Some(cached) = self.backing.get_cached_account(address) {
			return cached;
		}

		// check bloom before any requests to trie
		if !self.backing.check_account_bloom(address) { return None }

		// not found in the global cache, get from the DB.
		// TODO: insert directly into `StateDB` cache here?
		// or wait until commit_cache?
		let db = self.trie_factory.readonly(self.backing.as_hashdb(), &self.root)
			.expect(SEC_TRIE_DB_UNWRAP_STR);

		// get the account from the backing database, panicking if any nodes aren't there
		// as this is expected to be a full node.
		match db.get(address) {
			Ok(maybe_acc) => maybe_acc.map(Account::from_rlp),
			Err(e) => panic!("Potential DB corruption encountered: {}", e),
		}
	}

	fn storage(&self, address: Address, storage_root: &H256, key: &H256) -> H256 {
		// 1. If there's an entry for the account in the global cache check for the key or load it into that account.
		// 2. If account is missing in the global cache load it into the local cache and cache the key there.

		let addr_hash = self.addr_hash(address);

		// check the global cache and and cache storage key there if found,
		// otherwise cache the account localy and cache storage key there.
		if let Some(Some(result)) = self.backing.get_cached(&address, |acc| acc.map_or(None, |a| {
			a.cached_storage_at(key)
		})) {
			return result;
		}

		// check the account trie for the storage key.
		let account_db = self.db_factory.readonly(self.backing.as_hashdb(), addr_hash);
		let db = self.trie_factory.readonly(account_db.as_hashdb(), storage_root)
			.expect("account storage root always valid; qed");

		let item: U256 = match db.get(key) {
			Ok(x) => x.map_or_else(U256::zero, rlp::decode),
			Err(e) => panic!("Potential DB corruption encountered: {}", e),
		};
		item.into()
	}

	fn commit(&mut self, accounts: &mut HashMap<Address, AccountEntry>)
		-> Result<(), Error>
	{
		// first commit the sub trees.
		for (address, a) in accounts.iter_mut() {
			match *a {
				AccountEntry::Cached(ref mut account) if account.is_dirty() => {
					self.backing.note_account_bloom(&address);

					let addr_hash = self.addr_hash(address.clone());
					let mut account_db = self.db_factory.create(self.backing.as_hashdb_mut(), addr_hash);
					account.commit_storage(&self.trie_factory, account_db.as_hashdb_mut());
					account.commit_code(account_db.as_hashdb_mut());
				}
				_ => {}
			}
		}

		{
			let mut trie = try!(self.trie_factory.from_existing(self.backing.as_hashdb_mut(), &mut self.root));
			for (address, a) in accounts.iter_mut() {
				match *a {
					AccountEntry::Cached(ref mut account) if account.is_dirty() => {
						account.set_clean();
						try!(trie.insert(address, &account.rlp()))
					}
					AccountEntry::Killed => {
						try!(trie.remove(address));
						*a = AccountEntry::Missing;
					}
					_ => {}
				}
			}
		}

		Ok(())
	}

	fn root(&self) -> &H256 { &self.root }
}