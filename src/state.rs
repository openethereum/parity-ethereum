use std::collections::HashMap;
use util::hash::*;
use util::hashdb::*;
use util::overlaydb::*;
use util::trie::*;
use util::rlp::*;
use util::uint::*;
use account::Account;

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

	/// Expose the underlying database; good to use for calling `state.db().commit()`.
	pub fn db(&mut self) -> &mut OverlayDB {
		&mut self.db
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
	fn ensure_cached(&mut self, a: &Address, require_code: bool, force_create: bool) {
		if self.cache.get(a).is_none() {
			// load from trie.
			self.cache.insert(a.clone(), TrieDB::new(&mut self.db, &mut self.root).at(&a).map(|rlp| Account::from_rlp(rlp)));
		}

		if self.cache.get(a).unwrap().is_none() {
			if !force_create { return; }
			// create a new account
			self.cache.insert(a.clone(), Some(Account::new_basic(U256::from(0u8))));
		}

		if require_code {
			if let &mut Some(ref mut account) = self.cache.get_mut(a).unwrap() {
				account.ensure_cached(&self.db);
			}
		}
	}
}

#[cfg(test)]
mod tests {

use super::*;
use util::hash::*;
use util::trie::*;
use util::rlp::*;
use std::str::FromStr;

#[test]
fn playpen() {
}

#[test]
fn ensure_cached() {
	let mut s = State::new_temp();
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	s.ensure_cached(&a, false, true);
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