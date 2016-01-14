use util::*;
use itertools::Itertools;

pub const SHA3_EMPTY: H256 = H256( [0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70] );

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Diff<T> where T: Eq {
	Same,
	Born(T),
	Changed(T, T),
	Died(T),
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Existance {
	Born,
	Alive,
	Died,
}

impl<T> Diff<T> where T: Eq {
	pub fn new(pre: T, post: T) -> Self { if pre == post { Diff::Same } else { Diff::Changed(pre, post) } }
	pub fn pre(&self) -> Option<&T> { match self { &Diff::Died(ref x) | &Diff::Changed(ref x, _) => Some(x), _ => None } }
	pub fn post(&self) -> Option<&T> { match self { &Diff::Born(ref x) | &Diff::Changed(_, ref x) => Some(x), _ => None } }
	pub fn is_same(&self) -> bool { match self { &Diff::Same => true, _ => false }}
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
pub struct AccountDiff {
	pub balance: Diff<U256>,				// Allowed to be Same
	pub nonce: Diff<U256>,					// Allowed to be Same
	pub code: Diff<Bytes>,					// Allowed to be Same
	pub storage: BTreeMap<H256, Diff<H256>>,// Not allowed to be Same
}

impl AccountDiff {
	pub fn existance(&self) -> Existance {
		match self.balance {
			Diff::Born(_) => Existance::Born,
			Diff::Died(_) => Existance::Died,
			_ => Existance::Alive,
		}
	}
}

fn format(u: &H256) -> String {
	if u <= &H256::from(0xffffffff) {
		format!("{} = 0x{:x}", U256::from(u.as_slice()).low_u32(), U256::from(u.as_slice()).low_u32())
	} else if u <= &H256::from(u64::max_value()) {
		format!("{} = 0x{:x}", U256::from(u.as_slice()).low_u64(), U256::from(u.as_slice()).low_u64())
//	} else if u <= &H256::from("0xffffffffffffffffffffffffffffffffffffffff") {
//		format!("@{}", Address::from(u))
	} else {
		format!("#{}", u)
	}
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct StateDiff (BTreeMap<Address, AccountDiff>);

impl fmt::Display for AccountDiff {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.nonce {
			Diff::Born(ref x) => try!(write!(f, "  non {}", x)),
			Diff::Changed(ref pre, ref post) => try!(write!(f, "#{} ({} {} {})", post, pre, if pre > post {"-"} else {"+"}, *max(pre, post) - *	min(pre, post))),
			_ => {},
		}
		match self.balance {
			Diff::Born(ref x) => try!(write!(f, "  bal {}", x)),
			Diff::Changed(ref pre, ref post) => try!(write!(f, "${} ({} {} {})", post, pre, if pre > post {"-"} else {"+"}, *max(pre, post) - *min(pre, post))),
			_ => {},
		}
		match self.code {
			Diff::Born(ref x) => try!(write!(f, "  code {}", x.pretty())),
			_ => {},
		}
		try!(write!(f, "\n"));
		for (k, dv) in self.storage.iter() {
			match dv {
				&Diff::Born(ref v) => try!(write!(f, "    +  {} => {}\n", format(k), format(v))),
				&Diff::Changed(ref pre, ref post) => try!(write!(f, "    *  {} => {} (was {})\n", format(k), format(post), format(pre))),
				&Diff::Died(_) => try!(write!(f, "    X  {}\n", format(k))),
				_ => {},
			}
		}
		Ok(())
	}
}

impl fmt::Display for Existance {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Existance::Born => try!(write!(f, "+++")),
			&Existance::Alive => try!(write!(f, "***")),
			&Existance::Died => try!(write!(f, "XXX")),
		}
		Ok(())
	}
}

impl fmt::Display for StateDiff {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (add, acc) in self.0.iter() {
			try!(write!(f, "{} {}: {}", acc.existance(), add, acc));
		}
		Ok(())
	}
}

pub fn pod_diff(pre: Option<&PodAccount>, post: Option<&PodAccount>) -> Option<AccountDiff> {
	match (pre, post) {
		(None, Some(x)) => Some(AccountDiff {
			balance: Diff::Born(x.balance.clone()),
			nonce: Diff::Born(x.nonce.clone()),
			code: Diff::Born(x.code.clone()),
			storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Born(v.clone()))).collect(),
		}),
		(Some(x), None) => Some(AccountDiff {
			balance: Diff::Died(x.balance.clone()),
			nonce: Diff::Died(x.nonce.clone()),
			code: Diff::Died(x.code.clone()),
			storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Died(v.clone()))).collect(),
		}),
		(Some(pre), Some(post)) => {
			let storage: Vec<_> = pre.storage.keys().merge(post.storage.keys())
				.filter(|k| pre.storage.get(k).unwrap_or(&H256::new()) != post.storage.get(k).unwrap_or(&H256::new()))
				.collect();
			let r = AccountDiff {
				balance: Diff::new(pre.balance.clone(), post.balance.clone()),
				nonce: Diff::new(pre.nonce.clone(), post.nonce.clone()),
				code: Diff::new(pre.code.clone(), post.code.clone()),
				storage: storage.into_iter().map(|k|
					(k.clone(), Diff::new(
						pre.storage.get(&k).cloned().unwrap_or(H256::new()),
						post.storage.get(&k).cloned().unwrap_or(H256::new())
					))).collect(),
			};
			if r.balance.is_same() && r.nonce.is_same() && r.code.is_same() && r.storage.len() == 0 {
				None
			} else {
				Some(r)
			}
		},
		_ => None,
	}
}

pub fn pod_map_diff(pre: &BTreeMap<Address, PodAccount>, post: &BTreeMap<Address, PodAccount>) -> StateDiff {
	StateDiff(pre.keys().merge(post.keys()).filter_map(|acc| pod_diff(pre.get(acc), post.get(acc)).map(|d|(acc.clone(), d))).collect())
}

#[test]
fn state_diff_create_delete() {
	let a = map![
		x!(1) => PodAccount{
			balance: x!(69),
			nonce: x!(0),
			code: vec![],
			storage: map![]
		}
	];
	assert_eq!(pod_map_diff(&a, &map![]), StateDiff(map![
		x!(1) => AccountDiff{
			balance: Diff::Died(x!(69)),
			nonce: Diff::Died(x!(0)),
			code: Diff::Died(vec![]),
			storage: map![],
		}
	]));
	assert_eq!(pod_map_diff(&map![], &a), StateDiff(map![
		x!(1) => AccountDiff{
			balance: Diff::Born(x!(69)),
			nonce: Diff::Born(x!(0)),
			code: Diff::Born(vec![]),
			storage: map![],
		}
	]));
}

#[test]
fn state_diff_cretae_delete_with_unchanged() {
	let a = map![
		x!(1) => PodAccount{
			balance: x!(69),
			nonce: x!(0),
			code: vec![],
			storage: map![]
		}
	];
	let b = map![
		x!(1) => PodAccount{
			balance: x!(69),
			nonce: x!(0),
			code: vec![],
			storage: map![]
		},
		x!(2) => PodAccount{
			balance: x!(69),
			nonce: x!(0),
			code: vec![],
			storage: map![]
		}
	];
	assert_eq!(pod_map_diff(&a, &b), StateDiff(map![
		x!(2) => AccountDiff{
			balance: Diff::Born(x!(69)),
			nonce: Diff::Born(x!(0)),
			code: Diff::Born(vec![]),
			storage: map![],
		}
	]));
	assert_eq!(pod_map_diff(&b, &a), StateDiff(map![
		x!(2) => AccountDiff{
			balance: Diff::Died(x!(69)),
			nonce: Diff::Died(x!(0)),
			code: Diff::Died(vec![]),
			storage: map![],
		}
	]));
}

#[test]
fn state_diff_change_with_unchanged() {
	let a = map![
		x!(1) => PodAccount{
			balance: x!(69),
			nonce: x!(0),
			code: vec![],
			storage: map![]
		},
		x!(2) => PodAccount{
			balance: x!(69),
			nonce: x!(0),
			code: vec![],
			storage: map![]
		}
	];
	let b = map![
		x!(1) => PodAccount{
			balance: x!(69),
			nonce: x!(1),
			code: vec![],
			storage: map![]
		},
		x!(2) => PodAccount{
			balance: x!(69),
			nonce: x!(0),
			code: vec![],
			storage: map![]
		}
	];
	assert_eq!(pod_map_diff(&a, &b), StateDiff(map![
		x!(1) => AccountDiff{
			balance: Diff::Same,
			nonce: Diff::Changed(x!(0), x!(1)),
			code: Diff::Same,
			storage: map![],
		}
	]));
}

#[test]
fn account_diff_existence() {
	let a = PodAccount{balance: x!(69), nonce: x!(0), code: vec![], storage: map![]};
	assert_eq!(pod_diff(Some(&a), Some(&a)), None);
	assert_eq!(pod_diff(None, Some(&a)), Some(AccountDiff{
		balance: Diff::Born(x!(69)),
		nonce: Diff::Born(x!(0)),
		code: Diff::Born(vec![]),
		storage: map![],
	}));
}

#[test]
fn account_diff_basic() {
	let a = PodAccount{balance: x!(69), nonce: x!(0), code: vec![], storage: map![]};
	let b = PodAccount{balance: x!(42), nonce: x!(1), code: vec![], storage: map![]};
	assert_eq!(pod_diff(Some(&a), Some(&b)), Some(AccountDiff {
		balance: Diff::Changed(x!(69), x!(42)),
		nonce: Diff::Changed(x!(0), x!(1)),
		code: Diff::Same,
		storage: map![],
	}));
}

#[test]
fn account_diff_code() {
	let a = PodAccount{balance: x!(0), nonce: x!(0), code: vec![], storage: map![]};
	let b = PodAccount{balance: x!(0), nonce: x!(1), code: vec![0], storage: map![]};
	assert_eq!(pod_diff(Some(&a), Some(&b)), Some(AccountDiff {
		balance: Diff::Same,
		nonce: Diff::Changed(x!(0), x!(1)),
		code: Diff::Changed(vec![], vec![0]),
		storage: map![],
	}));
}

#[test]
fn account_diff_storage() {
	let a = PodAccount {
		balance: x!(0),
		nonce: x!(0),
		code: vec![],
		storage: mapx![1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 0, 6 => 0, 7 => 0]
	};
	let b = PodAccount {
		balance: x!(0),
		nonce: x!(0),
		code: vec![],
		storage: mapx![1 => 1, 2 => 3, 3 => 0, 5 => 0, 7 => 7, 8 => 0, 9 => 9]
	};
	assert_eq!(pod_diff(Some(&a), Some(&b)), Some(AccountDiff {
		balance: Diff::Same,
		nonce: Diff::Same,
		code: Diff::Same,
		storage: map![
			x!(2) => Diff::new(x!(2), x!(3)),
			x!(3) => Diff::new(x!(3), x!(0)),
			x!(4) => Diff::new(x!(4), x!(0)),
			x!(7) => Diff::new(x!(0), x!(7)),
			x!(9) => Diff::new(x!(0), x!(9))
		],
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