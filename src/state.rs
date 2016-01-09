use util::*;
use account::Account;
use transaction::Transaction;
use receipt::Receipt;
use env_info::EnvInfo;
use engine::Engine;

/// Information concerning the result of the `State::apply` operation.
pub struct ApplyInfo {
	pub receipt: Receipt,
}

pub type ApplyResult = Result<ApplyInfo, EthcoreError>;

/// Representation of the entire state of all accounts in the system.
pub struct State {
	db: OverlayDB,
	root: H256,
	cache: RefCell<HashMap<Address, Option<Account>>>,

	account_start_nonce: U256,
}

impl State {
	/// Creates new state with empty state root
	pub fn new(mut db: OverlayDB, account_start_nonce: U256) -> State {
		let mut root = H256::new();
		{
			// init trie and reset root too null
			let _ = SecTrieDBMut::new(&mut db, &mut root);
		}

		State {
			db: db,
			root: root,
			cache: RefCell::new(HashMap::new()),
			account_start_nonce: account_start_nonce,
		}
	}

	/// Creates new state with existing state root
	pub fn from_existing(db: OverlayDB, root: H256, account_start_nonce: U256) -> State {
		{
			// trie should panic! if root does not exist
			let _ = SecTrieDB::new(&db, &root);
		}

		State {
			db: db,
			root: root,
			cache: RefCell::new(HashMap::new()),
			account_start_nonce: account_start_nonce,
		}
	}

	/// Create temporary state object
	pub fn new_temp() -> State {
		Self::new(OverlayDB::new_temp(), U256::from(0u8))
	}

	/// Destroy the current object and return root and database.
	pub fn drop(self) -> (H256, OverlayDB) {
		(self.root, self.db)
	}

	/// Return reference to root
	pub fn root(&self) -> &H256 {
		&self.root
	}

	/// Expose the underlying database; good to use for calling `state.db().commit()`.
	pub fn db(&mut self) -> &mut OverlayDB {
		&mut self.db
	}

	/// Create a new contract at address `contract`. If there is already an account at the address
	/// it will have its code reset, ready for `init_code()`.
	pub fn new_contract(&mut self, contract: &Address) {
		self.require_or_from(contract, false, || Account::new_contract(U256::from(0u8)), |r| r.reset_code());
	}

	/// Get the balance of account `a`.
	pub fn balance(&self, a: &Address) -> U256 {
		self.get(a, false).as_ref().map(|account| account.balance().clone()).unwrap_or(U256::from(0u8))
	}

	/// Get the nonce of account `a`.
	pub fn nonce(&self, a: &Address) -> U256 {
		self.get(a, false).as_ref().map(|account| account.nonce().clone()).unwrap_or(U256::from(0u8))
	}

	/// Mutate storage of account `a` so that it is `value` for `key`.
	pub fn storage_at(&self, a: &Address, key: &H256) -> H256 {
		self.get(a, false).as_ref().map(|a|a.storage_at(&self.db, key)).unwrap_or(H256::new())	
	}

	/// Mutate storage of account `a` so that it is `value` for `key`.
	pub fn code(&self, a: &Address) -> Option<Vec<u8>> {
		self.get(a, true).as_ref().map(|a|a.code().map(|x|x.to_vec())).unwrap_or(None)
	}

	/// Add `incr` to the balance of account `a`.
	pub fn add_balance(&mut self, a: &Address, incr: &U256) {
		self.require(a, false).add_balance(incr)
	}

	/// Subtract `decr` from the balance of account `a`.
	pub fn sub_balance(&mut self, a: &Address, decr: &U256) {
		self.require(a, false).sub_balance(decr)
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
		self.require(a, false).set_storage(key, value);
	}

	/// Initialise the code of account `a` so that it is `value` for `key`.
	/// NOTE: Account should have been created with `new_contract`.
	pub fn init_code(&mut self, a: &Address, code: Bytes) {
		self.require_or_from(a, true, || Account::new_contract(U256::from(0u8)), |_|{}).init_code(code);
	}

	/// Execute a given transaction.
	/// This will change the state accordingly.
	pub fn apply(&mut self, _env_info: &EnvInfo, _engine: &Engine, _t: &Transaction, _is_permanent: bool) -> ApplyResult {
		unimplemented!();
	}

	/// Convert into a JSON representation.
	pub fn as_json(&self) -> String {
		unimplemented!();
	}

	/// Commit accounts to SecTrieDBMut. This is similar to cpp-ethereum's dev::eth::commit.
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
			let mut trie = SecTrieDBMut::from_existing(db, &mut root);
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
		self.root = Self::commit_into(&mut self.db, r, self.cache.borrow_mut().deref_mut());
	}

	/// Populate the state from `accounts`. Just uses `commit_into`.
	pub fn populate_from(&mut self, _accounts: &mut HashMap<Address, Option<Account>>) {
		unimplemented!();
	}

	/// Pull account `a` in our cache from the trie DB and return it.
	/// `require_code` requires that the code be cached, too.
	fn get(&self, a: &Address, require_code: bool) -> Ref<Option<Account>> {
		self.cache.borrow_mut().entry(a.clone()).or_insert_with(||
			SecTrieDB::new(&self.db, &self.root).get(&a).map(|rlp| Account::from_rlp(rlp)));
		if require_code {
			if let Some(ref mut account) = self.cache.borrow_mut().get_mut(a).unwrap().as_mut() {
				account.cache_code(&self.db);
			}
		}
		Ref::map(self.cache.borrow(), |m| m.get(a).unwrap())
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	fn require(&self, a: &Address, require_code: bool) -> RefMut<Account> {
		self.require_or_from(a, require_code, || Account::new_basic(U256::from(0u8), self.account_start_nonce), |_|{})
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	/// If it doesn't exist, make account equal the evaluation of `default`.
	fn require_or_from<F: FnOnce() -> Account, G: FnOnce(&mut Account)>(&self, a: &Address, require_code: bool, default: F, not_default: G) -> RefMut<Account> {
		self.cache.borrow_mut().entry(a.clone()).or_insert_with(||
			SecTrieDB::new(&self.db, &self.root).get(&a).map(|rlp| Account::from_rlp(rlp)));
		let preexists = self.cache.borrow().get(a).unwrap().is_none();
		if preexists {
			self.cache.borrow_mut().insert(a.clone(), Some(default()));
		} else {
			not_default(self.cache.borrow_mut().get_mut(a).unwrap().as_mut().unwrap());
		}

		let b = self.cache.borrow_mut();
		RefMut::map(b, |m| m.get_mut(a).unwrap().as_mut().map(|account| {
			if require_code {
				account.cache_code(&self.db);
			}
			account
		}).unwrap())
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
use account::*;

#[test]
fn code_from_database() {
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	let (r, db) = {
		let mut s = State::new_temp();
		s.require_or_from(&a, false, ||Account::new_contract(U256::from(42u32)), |_|{});
		s.init_code(&a, vec![1, 2, 3]);
		assert_eq!(s.code(&a), Some([1u8, 2, 3].to_vec()));
		s.commit();
		assert_eq!(s.code(&a), Some([1u8, 2, 3].to_vec()));
		s.drop()
	};

	let s = State::from_existing(db, r, U256::from(0u8));
	assert_eq!(s.code(&a), Some([1u8, 2, 3].to_vec()));
}

#[test]
fn storage_at_from_database() {
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	let (r, db) = {
		let mut s = State::new_temp();
		s.set_storage(&a, H256::from(&U256::from(01u64)), H256::from(&U256::from(69u64)));
		s.commit();
		s.drop()
	};

	let s = State::from_existing(db, r, U256::from(0u8));
	assert_eq!(s.storage_at(&a, &H256::from(&U256::from(01u64))), H256::from(&U256::from(69u64)));
}

#[test]
fn get_from_database() {
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	let (r, db) = {
		let mut s = State::new_temp();
		s.inc_nonce(&a);
		s.add_balance(&a, &U256::from(69u64));
		s.commit();
		assert_eq!(s.balance(&a), U256::from(69u64));
		s.drop()
	};

	let s = State::from_existing(db, r, U256::from(0u8));
	assert_eq!(s.balance(&a), U256::from(69u64));
	assert_eq!(s.nonce(&a), U256::from(1u64));
}

#[test]
fn alter_balance() {
	let mut s = State::new_temp();
	let a = Address::from_str("0000000000000000000000000000000000000000").unwrap();
	let b = Address::from_str("0000000000000000000000000000000000000001").unwrap();
	s.add_balance(&a, &U256::from(69u64));
	assert_eq!(s.balance(&a), U256::from(69u64));
	s.commit();
	assert_eq!(s.balance(&a), U256::from(69u64));
	s.sub_balance(&a, &U256::from(42u64));
	assert_eq!(s.balance(&a), U256::from(27u64));
	s.commit();
	assert_eq!(s.balance(&a), U256::from(27u64));
	s.transfer_balance(&a, &b, &U256::from(18u64));
	assert_eq!(s.balance(&a), U256::from(9u64));
	assert_eq!(s.balance(&b), U256::from(18u64));
	s.commit();
	assert_eq!(s.balance(&a), U256::from(9u64));
	assert_eq!(s.balance(&b), U256::from(18u64));
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
	assert_eq!(s.root().hex(), "0ce23f3c809de377b008a4a3ee94a0834aac8bec1f86e28ffe4fdb5a15b0c785");
}

#[test]
fn create_empty() {
	let mut s = State::new_temp();
	s.commit();
	assert_eq!(s.root().hex(), "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");
}

}