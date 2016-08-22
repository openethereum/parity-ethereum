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

use util::journaldb::JournalDB;
use util::trie::TrieFactory;
use util::rlp::decode;
use util::{Bytes, H256, Address, Hashable, HashDB, U256, Uint};

use std::cell::RefCell;
use std::collections::HashMap;

use account_db::{AccountDB, AccountDBMut};
use error::Error;
use state::{self, Account, SEC_TRIE_DB_UNWRAP_STR};

/// Fully database-backed state backend, with account-key mangling.
/// This is expected to be a full node.
pub struct Database {
	backing: Box<JournalDB>,
	trie_factory: TrieFactory,
	address_hashes: RefCell<HashMap<Address, H256>>,
}

impl Database {
	/// Create a new database backend.
	pub fn new(backing: Box<JournalDB>, factory: TrieFactory) -> Self {
		Database {
			backing: backing,
			trie_factory: factory,
			address_hashes: RefCell::new(HashMap::new()),
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
			trie_factory: self.trie_factory.clone(),
			address_hashes: RefCell::new(self.address_hashes.borrow().clone())
		}
	}
}

impl state::Backend for Database {
	fn code(&self, address: Address, code_hash: &H256) -> Option<Bytes> {
		let addr_hash = self.addr_hash(address);
		AccountDB::from_hash(self.backing.as_hashdb(), addr_hash).get(code_hash).map(Into::into)
	}

	fn account(&self, root: &H256, address: &Address) -> Option<Account> {
		let db = self.trie_factory.readonly(self.backing.as_hashdb(), root)
			.expect(SEC_TRIE_DB_UNWRAP_STR);

		// get the account from the backing database, panicking if any nodes aren't there
		// as this is expected to be a full node.
		match db.get(address) {
			Ok(maybe_acc) => maybe_acc.map(Account::from_rlp),
			Err(e) => panic!("Potential DB corruption encountered: {}", e),
		}
	}

	fn storage(&self, address: Address, storage_root: &H256, key: &H256) -> H256 {
		let addr_hash = self.addr_hash(address);
		let account_db = AccountDB::from_hash(self.backing.as_hashdb(), addr_hash);
		let db = self.trie_factory.readonly(&account_db, storage_root)
			.expect("account storage root always valid; qed");

		let item: U256 = match db.get(key) {
			Ok(x) => x.map_or_else(U256::zero, decode),
			Err(e) => panic!("Potential DB corruption encountered: {}", e),
		};
		item.into()
	}

	fn commit(&mut self, root: &mut H256, accounts: &mut HashMap<Address, Option<Account>>)
		-> Result<(), Error>
	{
		// first commit the sub trees.
		for (address, a) in accounts.iter_mut() {
			match *a {
				Some(ref mut account) if account.is_dirty() => {
					let addr_hash = self.addr_hash(address.clone());
					let mut account_db = AccountDBMut::from_hash(self.backing.as_hashdb_mut(), addr_hash);
					account.commit_storage(&self.trie_factory, &mut account_db);
					account.commit_code(&mut account_db);
				}
				_ => {}
			}
		}

		{
			let mut trie = try!(self.trie_factory.from_existing(self.backing.as_hashdb_mut(), root));
			for (address, a) in accounts.iter_mut() {
				match *a {
					Some(ref mut account) if account.is_dirty() => {
						account.set_clean();
						try!(trie.insert(address, &account.rlp()))
					}
					None => try!(trie.remove(address)),
					_ => {}
				}
			}
		}

		Ok(())
	}
}