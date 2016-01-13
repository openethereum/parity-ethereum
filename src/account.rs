use util::*;

pub const SHA3_EMPTY: H256 = H256( [0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70] );

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Diff<T> where T: Eq {
	pub pre: T,
	pub post_opt: Option<T>,
}

impl<T> Diff<T> where T: Eq {
	pub fn new_opt(pre: T, post: T) -> Option<Self> { if pre == post { None } else { Some(Self::new(pre, post)) } }
	pub fn one_opt(t: T) -> Option<Self> { Some(Self::one(t)) }

	pub fn new(pre: T, post: T) -> Self { Diff { pre: pre, post_opt: Some(post) }}
	pub fn one(t: T) -> Self { Diff { pre: t, post_opt: None }}

	pub fn pre(&self) -> &T { &self.pre }
	pub fn post(&self) -> &T { match self.post_opt { Some(ref x) => x, None => &self.pre } }
}

#[derive(Debug,Clone,PartialEq,Eq)]
/// Genesis account data. Does not have a DB overlay cache.
pub struct PodAccount {
	// Balance of the account.
	pub balance: U256,
	// Nonce of the account.
	pub nonce: U256,
	pub code: Bytes,
	pub storage: BTreeMap<H256, H256>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PodAccountDiff {
	pub exists: Diff<bool>,
	pub balance: Option<Diff<U256>>,
	pub nonce: Option<Diff<U256>>,
	pub code: Option<Diff<Bytes>>,
	pub storage: BTreeMap<H256, Diff<H256>>,
}

type StateDiff = BTreeMap<Address, PodAccountDiff>;

pub fn diff(pre: &Option<PodAccount>, post: &Option<PodAccount>) -> Option<PodAccountDiff> {
	match (pre, post) {
		(&Some(ref x), &None) | (&None, &Some(ref x)) => Some(PodAccountDiff {
			exists: Diff::new(pre.is_some(), post.is_some()),
			balance: Diff::one_opt(x.balance.clone()),
			nonce: Diff::one_opt(x.nonce.clone()),
			code: Diff::one_opt(x.code.clone()),
			storage: x.storage.iter().fold(BTreeMap::new(), |mut m, (k, v)| {m.insert(k.clone(), Diff::one(v.clone())); m})
		}),
		(&Some(ref pre), &Some(ref post)) => {
			let pre_keys: BTreeSet<_> = pre.storage.keys().collect();
			let post_keys: BTreeSet<_> = post.storage.keys().collect();
			let storage: Vec<_> = pre_keys.union(&post_keys)
				.filter(|k| pre.storage.get(k).unwrap_or(&H256::new()) != post.storage.get(k).unwrap_or(&H256::new()))
				.collect();
			if pre.balance != post.balance || pre.nonce != post.nonce || pre.code != post.code || storage.len() > 0 {
				Some(PodAccountDiff {
					exists: Diff::one(true),
					balance: Diff::new_opt(pre.balance.clone(), post.balance.clone()),
					nonce: Diff::new_opt(pre.nonce.clone(), post.nonce.clone()),
					code: Diff::new_opt(pre.code.clone(), post.code.clone()),
					storage: storage.into_iter().fold(BTreeMap::new(), |mut m, k| {
						let v = Diff::new(pre.storage.get(&k).cloned().unwrap_or(H256::new()), post.storage.get(&k).cloned().unwrap_or(H256::new()));
						m.insert((*k).clone(), v);
						m
					}),
				})
			} else {
				None
			}
		},
		_ => None,
	}
}

#[test]
fn account_diff_existence() {
	let a = Some(PodAccount{balance: U256::from(69u64), nonce: U256::zero(), code: vec![], storage: BTreeMap::new()});
	assert_eq!(diff(&a, &a), None);
	assert_eq!(diff(&None, &a), Some(PodAccountDiff{
		exists: Diff::new(false, true),
		balance: Diff::one_opt(U256::from(69u64)),
		nonce: Diff::one_opt(U256::zero()),
		code: Diff::one_opt(vec![]),
		storage: BTreeMap::new(),
	}));
}

#[test]
fn account_diff_basic() {
	let a = Some(PodAccount{balance: U256::from(69u64), nonce: U256::zero(), code: vec![], storage: BTreeMap::new()});
	let b = Some(PodAccount{balance: U256::from(42u64), nonce: U256::from(1u64), code: vec![], storage: BTreeMap::new()});
	assert_eq!(diff(&a, &b), Some(PodAccountDiff {
		exists: Diff::one(true),
		balance: Diff::new_opt(U256::from(69u64), U256::from(42u64)),
		nonce: Diff::new_opt(U256::zero(), U256::from(1u64)),
		code: None,
		storage: BTreeMap::new(),
	}));
}

#[test]
fn account_diff_code() {
	let a = Some(PodAccount{balance: U256::zero(), nonce: U256::zero(), code: vec![], storage: BTreeMap::new()});
	let b = Some(PodAccount{balance: U256::zero(), nonce: U256::from(1u64), code: vec![0x00u8], storage: BTreeMap::new()});
	assert_eq!(diff(&a, &b), Some(PodAccountDiff {
		exists: Diff::one(true),
		balance: None,
		nonce: Diff::new_opt(U256::zero(), U256::from(1u64)),
		code: Diff::new_opt(vec![], vec![0x00u8]),
		storage: BTreeMap::new(),
	}));
}

pub fn h256_from_u8(v: u8) -> H256 {
	let mut r = H256::new();
	r[31] = v;
	r
}

#[test]
fn account_diff_storage() {
	let a = Some(PodAccount{balance: U256::zero(), nonce: U256::zero(), code: vec![], storage: vec![(1u8, 1u8), (2, 2), (3, 3), (4, 4), (5, 0), (6, 0), (7, 0)].into_iter().fold(BTreeMap::new(), |mut m, (k, v)|{m.insert(h256_from_u8(k), h256_from_u8(v)); m})});
	let b = Some(PodAccount{balance: U256::zero(), nonce: U256::zero(), code: vec![], storage: vec![(1u8, 1u8), (2, 3), (3, 0), (5, 0), (7, 7), (8, 0), (9, 9)].into_iter().fold(BTreeMap::new(), |mut m, (k, v)|{m.insert(h256_from_u8(k), h256_from_u8(v)); m})});
	assert_eq!(diff(&a, &b), Some(PodAccountDiff {
		exists: Diff::one(true),
		balance: None,
		nonce: None,
		code: None,
		storage: vec![
			(2u8, Diff::new(h256_from_u8(2), h256_from_u8(3))),
			(3, Diff::new(h256_from_u8(3), H256::new())),
			(4, Diff::new(h256_from_u8(4), H256::new())),
			(7, Diff::new(H256::new(), h256_from_u8(7))),
			(9, Diff::new(H256::new(), h256_from_u8(9))),
		].into_iter().fold(BTreeMap::new(), |mut m, (k, v)|{m.insert(h256_from_u8(k), v); m})
	}));
}

/// Single account in the system.
#[derive(Clone)]
pub struct Account {
	// Balance of the account.
	balance: U256,
	// Nonce of the account.
	nonce: U256,
	// Trie-backed storage.
	storage_root: H256,
	// Overlay on trie-backed storage.
	storage_overlay: RefCell<HashMap<H256, H256>>,
	// Code hash of the account. If None, means that it's a contract whose code has not yet been set.
	code_hash: Option<H256>,
	// Code cache of the account.
	code_cache: Bytes,
}

impl PodAccount {
	/// Convert Account to a PodAccount.
	/// NOTE: This will silently fail unless the account is fully cached.
	pub fn from_account(acc: &Account) -> PodAccount {
		PodAccount {
			balance: acc.balance.clone(),
			nonce: acc.nonce.clone(),
			storage: acc.storage_overlay.borrow().iter().fold(BTreeMap::new(), |mut m, (k, v)| {m.insert(k.clone(), v.clone()); m}),
			code: acc.code_cache.clone()
		}
	}

	pub fn rlp(&self) -> Bytes {
		let mut stream = RlpStream::new_list(4);
		stream.append(&self.nonce);
		stream.append(&self.balance);
		// TODO.
		stream.append(&SHA3_NULL_RLP);
		stream.append(&self.code.sha3());
		stream.out()
	}
}

impl Account {
	/// General constructor.
	pub fn new(balance: U256, nonce: U256, storage: HashMap<H256, H256>, code: Bytes) -> Account {
		Account {
			balance: balance,
			nonce: nonce,
			storage_root: SHA3_NULL_RLP,
			storage_overlay: RefCell::new(storage),
			code_hash: Some(code.sha3()),
			code_cache: code
		}
	}

	/// General constructor.
	pub fn from_pod(pod: PodAccount) -> Account {
		Account {
			balance: pod.balance,
			nonce: pod.nonce,
			storage_root: SHA3_NULL_RLP,
			storage_overlay: RefCell::new(pod.storage.into_iter().fold(HashMap::new(), |mut m, (k, v)| {m.insert(k, v); m})),
			code_hash: Some(pod.code.sha3()),
			code_cache: pod.code
		}
	}

	/// Create a new account with the given balance.
	pub fn new_basic(balance: U256, nonce: U256) -> Account {
		Account {
			balance: balance,
			nonce: nonce,
			storage_root: SHA3_NULL_RLP,
			storage_overlay: RefCell::new(HashMap::new()),
			code_hash: Some(SHA3_EMPTY),
			code_cache: vec![],
		}
	}

	/// Create a new account from RLP.
	pub fn from_rlp(rlp: &[u8]) -> Account {
		let r: Rlp = Rlp::new(rlp);
		Account {
			nonce: r.val_at(0),
			balance: r.val_at(1),
			storage_root: r.val_at(2),
			storage_overlay: RefCell::new(HashMap::new()),
			code_hash: Some(r.val_at(3)),
			code_cache: vec![],
		}
	}

	/// Create a new contract account.
	/// NOTE: make sure you use `init_code` on this before `commit`ing.
	pub fn new_contract(balance: U256) -> Account {
		Account {
			balance: balance,
			nonce: U256::from(0u8),
			storage_root: SHA3_NULL_RLP,
			storage_overlay: RefCell::new(HashMap::new()),
			code_hash: None,
			code_cache: vec![],
		}
	}

	/// Reset this account to the status of a not-yet-initialised contract.
	/// NOTE: Account should have `init_code()` called on it later.
	pub fn reset_code(&mut self) {
		self.code_hash = None;
		self.code_cache = vec![];
	}

	/// Set this account's code to the given code.
	/// NOTE: Account should have been created with `new_contract()` or have `reset_code()` called on it.
	pub fn init_code(&mut self, code: Bytes) {
		assert!(self.code_hash.is_none());
		self.code_cache = code;
	}

	/// Set (and cache) the contents of the trie's storage at `key` to `value`.
	pub fn set_storage(&mut self, key: H256, value: H256) {
		self.storage_overlay.borrow_mut().insert(key, value);
	}

	/// Get (and cache) the contents of the trie's storage at `key`.
	pub fn storage_at(&self, db: &HashDB, key: &H256) -> H256 {
		self.storage_overlay.borrow_mut().entry(key.clone()).or_insert_with(||{
			H256::from_slice(TrieDB::new(db, &self.storage_root).get(key.bytes()).unwrap_or(&[0u8;32][..]))
		}).clone()
	}

	/// return the balance associated with this account.
	pub fn balance(&self) -> &U256 { &self.balance }

	/// return the nonce associated with this account.
	pub fn nonce(&self) -> &U256 { &self.nonce }

	/// return the code hash associated with this account.
	pub fn code_hash(&self) -> H256 {
		self.code_hash.clone().unwrap_or(SHA3_EMPTY)
	}

	/// returns the account's code. If `None` then the code cache isn't available -
	/// get someone who knows to call `note_code`.
	pub fn code(&self) -> Option<&[u8]> {
		match self.code_hash {
			Some(SHA3_EMPTY) | None if self.code_cache.is_empty() => Some(&self.code_cache),
			Some(_) if !self.code_cache.is_empty() => Some(&self.code_cache),
			None => Some(&self.code_cache),
			_ => None,
		}
	}

	/// Provide a byte array which hashes to the `code_hash`. returns the hash as a result.
	pub fn note_code(&mut self, code: Bytes) -> Result<(), H256> {
		let h = code.sha3();
		match self.code_hash {
			Some(ref i) if h == *i => {
				self.code_cache = code;
				Ok(())
			},
			_ => Err(h)
		}
	}

	/// Is `code_cache` valid; such that code is going to return Some?
	pub fn is_cached(&self) -> bool {
		!self.code_cache.is_empty() || (self.code_cache.is_empty() && self.code_hash == Some(SHA3_EMPTY))
	}

	/// Provide a database to lookup `code_hash`. Should not be called if it is a contract without code.
	pub fn cache_code(&mut self, db: &HashDB) -> bool {
		// TODO: fill out self.code_cache;
/*		return !self.is_cached() ||
			match db.lookup(&self.code_hash.unwrap()) {	// why doesn't this work? unwrap causes move?!
				Some(x) => { self.code_cache = x.to_vec(); true },
				_ => { false }
			}*/
		if self.is_cached() { return true; }
		return if let Some(ref h) = self.code_hash {
			match db.lookup(&h) {
				Some(x) => { self.code_cache = x.to_vec(); true },
				_ => { false }
			}
		} else { false }
	}

	/// return the storage root associated with this account.
	pub fn base_root(&self) -> &H256 { &self.storage_root }
	
	/// return the storage root associated with this account or None if it has been altered via the overlay.
	pub fn storage_root(&self) -> Option<&H256> { if self.storage_overlay.borrow().is_empty() {Some(&self.storage_root)} else {None} }
	
	/// rturn the storage overlay.
	pub fn storage_overlay(&self) -> Ref<HashMap<H256, H256>> { self.storage_overlay.borrow() }

	/// Increment the nonce of the account by one.
	pub fn inc_nonce(&mut self) { self.nonce = self.nonce + U256::from(1u8); }

	/// Increment the nonce of the account by one.
	pub fn add_balance(&mut self, x: &U256) { self.balance = self.balance + *x; }

	/// Increment the nonce of the account by one.
	pub fn sub_balance(&mut self, x: &U256) { self.balance = self.balance - *x; }

	/// Commit the `storage_overlay` to the backing DB and update `storage_root`.
	pub fn commit_storage(&mut self, db: &mut HashDB) {
		let mut t = TrieDBMut::new(db, &mut self.storage_root);
		for (k, v) in self.storage_overlay.borrow().iter() {
			// cast key and value to trait type,
			// so we can call overloaded `to_bytes` method
			t.insert(k, v);
		}
		self.storage_overlay.borrow_mut().clear();
	}

	/// Commit any unsaved code. `code_hash` will always return the hash of the `code_cache` after this.
	pub fn commit_code(&mut self, db: &mut HashDB) {
		trace!("Commiting code of {:?} - {:?}, {:?}", self, self.code_hash.is_none(), self.code_cache.is_empty());
		match (self.code_hash.is_none(), self.code_cache.is_empty()) {
			(true, true) => self.code_hash = Some(SHA3_EMPTY),
			(true, false) => {
				println!("Writing into DB {:?}", self.code_cache);
				self.code_hash = Some(db.insert(&self.code_cache));
			},
			(false, _) => {},
		}
	}

	/// Export to RLP.
	pub fn rlp(&self) -> Bytes {
		let mut stream = RlpStream::new_list(4);
		stream.append(&self.nonce);
		stream.append(&self.balance);
		stream.append(&self.storage_root);
		stream.append(self.code_hash.as_ref().expect("Cannot form RLP of contract account without code."));
		stream.out()
	}
}

impl fmt::Debug for Account {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", PodAccount::from_account(self))
	}
}

#[cfg(test)]
mod tests {

use super::*;
use std::collections::HashMap;
use util::hash::*;
use util::bytes::*;
use util::rlp::*;
use util::uint::*;
use util::overlaydb::*;

#[test]
fn storage_at() {
	let mut db = OverlayDB::new_temp();
	let rlp = {
		let mut a = Account::new_contract(U256::from(69u8));
		a.set_storage(H256::from(&U256::from(0x00u64)), H256::from(&U256::from(0x1234u64)));
		a.commit_storage(&mut db);
		a.init_code(vec![]);
		a.commit_code(&mut db);
		a.rlp()
	};

	let a = Account::from_rlp(&rlp);
	assert_eq!(a.storage_root().unwrap().hex(), "3541f181d6dad5c504371884684d08c29a8bad04926f8ceddf5e279dbc3cc769");
	assert_eq!(a.storage_at(&mut db, &H256::from(&U256::from(0x00u64))), H256::from(&U256::from(0x1234u64)));
	assert_eq!(a.storage_at(&mut db, &H256::from(&U256::from(0x01u64))), H256::new());
}

#[test]
fn note_code() {
	let mut db = OverlayDB::new_temp();

	let rlp = {
		let mut a = Account::new_contract(U256::from(69u8));
		a.init_code(vec![0x55, 0x44, 0xffu8]);
		a.commit_code(&mut db);
		a.rlp()
	};

	let mut a = Account::from_rlp(&rlp);
	assert_eq!(a.cache_code(&db), true);

	let mut a = Account::from_rlp(&rlp);
	assert_eq!(a.note_code(vec![0x55, 0x44, 0xffu8]), Ok(()));
}

#[test]
fn commit_storage() {
	let mut a = Account::new_contract(U256::from(69u8));
	let mut db = OverlayDB::new_temp();
	a.set_storage(H256::from(&U256::from(0x00u64)), H256::from(&U256::from(0x1234u64)));
	assert_eq!(a.storage_root(), None);
	a.commit_storage(&mut db);
	assert_eq!(a.storage_root().unwrap().hex(), "3541f181d6dad5c504371884684d08c29a8bad04926f8ceddf5e279dbc3cc769");
}

#[test]
fn commit_code() {
	let mut a = Account::new_contract(U256::from(69u8));
	let mut db = OverlayDB::new_temp();
	a.init_code(vec![0x55, 0x44, 0xffu8]);
	assert_eq!(a.code_hash(), SHA3_EMPTY);
	a.commit_code(&mut db);
	assert_eq!(a.code_hash().hex(), "af231e631776a517ca23125370d542873eca1fb4d613ed9b5d5335a46ae5b7eb");
}

#[test]
fn rlpio() {
	let a = Account::new(U256::from(69u8), U256::from(0u8), HashMap::new(), Bytes::new());
	let b = Account::from_rlp(&a.rlp());
	assert_eq!(a.balance(), b.balance());
	assert_eq!(a.nonce(), b.nonce());
	assert_eq!(a.code_hash(), b.code_hash());
	assert_eq!(a.storage_root(), b.storage_root());
}

#[test]
fn new_account() {
	use rustc_serialize::hex::ToHex;

	let a = Account::new(U256::from(69u8), U256::from(0u8), HashMap::new(), Bytes::new());
	assert_eq!(a.rlp().to_hex(), "f8448045a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
	assert_eq!(a.balance(), &U256::from(69u8));
	assert_eq!(a.nonce(), &U256::from(0u8));
	assert_eq!(a.code_hash(), SHA3_EMPTY);
	assert_eq!(a.storage_root().unwrap(), &SHA3_NULL_RLP);
}

#[test]
fn create_account() {
	use rustc_serialize::hex::ToHex;

	let a = Account::new(U256::from(69u8), U256::from(0u8), HashMap::new(), Bytes::new());
	assert_eq!(a.rlp().to_hex(), "f8448045a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
}

}