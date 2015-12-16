use std::collections::HashMap;
use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::trie::*;
use util::rlp::*;
use util::uint::*;
use std::mem;
use account::Account;
/*
enum ValueOrRef<'self, 'db: 'self> {
	Value(OverlayDB),
	Ref(&'db mut OverlayDB)
}

impl<'self, 'db> ValueOrRef<'self, 'db: 'self> {
	pub fn get_mut(&mut self) -> &mut OverlayDB {
		match self {
			Value(ref mut x) => x,
			Ref(x) => x,
		}
	}
	pub fn get(&self) -> &OverlayDB {
		match self {
			Value(ref x) => x,
			Ref(x) => x,
		}
	}
}
*/

/// Representation of the entire state of all accounts in the system.
pub struct State {
	db: OverlayDB,
	root: H256,
	cache: HashMap<Address, Option<Account>>,

	_account_start_nonce: U256,
}

impl State {
	/// Creates new state with empty state root
	pub fn new(mut db: OverlayDB, account_start_nonce: U256) -> State {
		let mut root = H256::new();
		{
			// init trie and reset root too null
			let _ = TrieDB::new(&mut db, &mut root);
		}

		State {
			db: db,
			root: root,
			cache: HashMap::new(),
			_account_start_nonce: account_start_nonce,
		}
	}

	/// Creates new state with existing state root
	pub fn new_existing(mut db: OverlayDB, mut root: H256, account_start_nonce: U256) -> State {
		{
			// trie should panic! if root does not exist
			let _ = TrieDB::new_existing(&mut db, &mut root);
		}

		State {
			db: db,
			root: root,
			cache: HashMap::new(),
			_account_start_nonce: account_start_nonce,
		}
	}

	/// Create temporary state object
	pub fn new_temp() -> State {
		Self::new(OverlayDB::new_temp(), U256::from(0u8))
	}

	/// Return reference to root
	pub fn root(&self) -> &H256 {
		&self.root
	}

	/// Desttroy the current database and return it.
	/// WARNING: the struct should be dropped immediately following this.
	pub fn take_db(&mut self) -> OverlayDB {
		mem::replace(&mut self.db, OverlayDB::new_temp())
	}

	/// Destroy the current object and return root and database.
	pub fn drop(mut self) -> (H256, OverlayDB) {
		(mem::replace(&mut self.root, H256::new()), mem::replace(&mut self.db, OverlayDB::new_temp()))
	}

	/// Expose the underlying database; good to use for calling `state.db().commit()`.
	pub fn db(&mut self) -> &mut OverlayDB {
		&mut self.db
	}

	/// Get the balance of account `a`.
	// TODO: make immutable
	pub fn balance(&mut self, a: &Address) -> U256 {
		self.get(a, false).as_ref().map(|account| account.balance().clone()).unwrap_or(U256::from(0u8))
	}

	/// Add `incr` to the balance of account `a`.
	pub fn add_balance(&mut self, a: &Address, incr: &U256) {
		self.require(a, false).add_balance(incr)
	}

	/// Subtract `decr` from the balance of account `a`.
	pub fn sub_balance(&mut self, a: &Address, decr: &U256) {
		self.require(a, false).sub_balance(decr)
	}

	/// Get the nonce of account `a`.
	// TODO: make immutable
	pub fn nonce(&mut self, a: &Address) -> U256 {
		self.get(a, false).as_ref().map(|account| account.nonce().clone()).unwrap_or(U256::from(0u8))
	}

	/// Increment the nonce of account `a` by 1.
	pub fn inc_nonce(&mut self, a: &Address) {
		self.require(a, false).inc_nonce()
	}

	/// Commit accounts to TrieDB. This is similar to cpp-ethereum's dev::eth::commit.
	/// `accounts` is mutable because we may need to commit the code or storage and record that.
	pub fn commit_into(db: &mut HashDB, mut root: H256, accounts: &mut HashMap<Address, Option<Account>>) -> H256 {
		// first, commit the sub trees.
		// TODO: is this necessary or can we dispense with the `ref mut a` for just `a`?
		for (_, ref mut a) in accounts.iter_mut() {
			match a {
				&mut&mut Some(ref mut account) => {
					account.commit_storage(db);
					account.commit_code(db);
				}
				&mut&mut None => {}
			}
		}

		{
			let mut trie = TrieDB::new_existing(db, &mut root);
			for (address, ref a) in accounts.iter() {
				match a {
					&&Some(ref account) => trie.insert(address, &account.rlp()),
					&&None => trie.remove(address),
				}
			}
		}
		root
	}

	/// Commits our cached account changes into the trie.
	pub fn commit(&mut self) {
		let r = self.root.clone();	// would prefer not to do this, really. 
		self.root = Self::commit_into(&mut self.db, r, &mut self.cache);
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	/// `force_create` creates a new, empty basic account if there is not currently an active account.
	// TODO: make immutable.
	fn get(&mut self, a: &Address, require_code: bool) -> Option<&mut Account> {
		if self.cache.get(a).is_none() {
			// load from trie.
			let t = TrieDB::new_existing(&mut self.db, &mut self.root);
			self.cache.insert(a.clone(), t.at(&a).map(|rlp| { println!("RLP: {:?}", rlp); Account::from_rlp(rlp) }));
		}

		let db = &self.db;
		self.cache.get_mut(a).unwrap().as_mut().map(|account| {
			if require_code {
				account.cache_code(db);
			}
			account
		})
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	/// `force_create` creates a new, empty basic account if there is not currently an active account.
	fn require(&mut self, a: &Address, require_code: bool) -> &mut Account {
		if self.cache.get(a).is_none() {
			// load from trie.
			self.cache.insert(a.clone(), TrieDB::new(&mut self.db, &mut self.root).at(&a).map(|rlp| Account::from_rlp(rlp)));
		}

		if self.cache.get(a).unwrap().is_none() {
			self.cache.insert(a.clone(), Some(Account::new_basic(U256::from(0u8))));
		}

		let db = &self.db;
		self.cache.get_mut(a).unwrap().as_mut().map(|account| {
			if require_code {
				account.cache_code(db);
			}
			account
		}).unwrap()
	}
}

#[cfg(test)]
mod tests {

use super::*;
use util::hash::*;
use util::trie::*;
use util::rlp::*;
use util::uint::*;
use std::str::FromStr;

#[test]
fn get_from_database() {
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	let (r, db) = {
		let mut s = State::new_temp();
		s.inc_nonce(&a);
		s.add_balance(&a, &U256::from(69u64));
		s.commit();
		assert_eq!(s.balance(&a), U256::from(69u64));
		(s.root().clone(), s.take_db())
	};

	let mut s = State::new_existing(db, r, U256::from(0u8));
	assert_eq!(s.balance(&a), U256::from(69u64));
	assert_eq!(s.nonce(&a), U256::from(1u64));
}

#[test]
fn alter_balance() {
	let mut s = State::new_temp();
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	s.add_balance(&a, &U256::from(69u64));
	assert_eq!(s.balance(&a), U256::from(69u64));
	s.commit();
	assert_eq!(s.balance(&a), U256::from(69u64));
	s.sub_balance(&a, &U256::from(42u64));
	assert_eq!(s.balance(&a), U256::from(27u64));
	s.commit();
	assert_eq!(s.balance(&a), U256::from(27u64));
}

#[test]
fn alter_nonce() {
	let mut s = State::new_temp();
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	s.inc_nonce(&a);
	assert_eq!(s.nonce(&a), U256::from(1u64));
	s.inc_nonce(&a);
	assert_eq!(s.nonce(&a), U256::from(2u64));
	s.commit();
	assert_eq!(s.nonce(&a), U256::from(2u64));
	s.inc_nonce(&a);
	assert_eq!(s.nonce(&a), U256::from(3u64));
	s.commit();
	assert_eq!(s.nonce(&a), U256::from(3u64));
}

#[test]
fn balance_nonce() {
	let mut s = State::new_temp();
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	assert_eq!(s.balance(&a), U256::from(0u64));
	assert_eq!(s.nonce(&a), U256::from(0u64));
	s.commit();
	assert_eq!(s.balance(&a), U256::from(0u64));
	assert_eq!(s.nonce(&a), U256::from(0u64));
}

#[test]
fn ensure_cached() {
	let mut s = State::new_temp();
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	s.require(&a, false);
	s.commit();
	assert_eq!(s.root().hex(), "ec68b85fa2e0526dc0e821a5b33135459114f19173ce0479f5c09b21cc25b9a4");
}

#[test]
fn create_empty() {
	let mut s = State::new_temp();
	s.commit();
	assert_eq!(s.root().hex(), "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");
}

}