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

//! Single account in the system.

use util::*;
use pod_account::*;
use account_db::*;

/// Single account in the system.
#[derive(Clone)]
pub struct Account {
	// Balance of the account.
	balance: U256,
	// Nonce of the account.
	nonce: U256,
	// Trie-backed storage.
	storage_root: H256,
	// Overlay on trie-backed storage - tuple is (<clean>, <value>).
	storage_overlay: RefCell<HashMap<H256, (Filth, H256)>>,
	// Code hash of the account. If None, means that it's a contract whose code has not yet been set.
	code_hash: Option<H256>,
	// Code cache of the account.
	code_cache: Bytes,
}

impl Account {
	#[cfg(test)]
	/// General constructor.
	pub fn new(balance: U256, nonce: U256, storage: HashMap<H256, H256>, code: Bytes) -> Account {
		Account {
			balance: balance,
			nonce: nonce,
			storage_root: SHA3_NULL_RLP,
			storage_overlay: RefCell::new(storage.into_iter().map(|(k, v)| (k, (Filth::Dirty, v))).collect()),
			code_hash: Some(code.sha3()),
			code_cache: code
		}
	}

	#[cfg(test)]
	#[cfg(feature = "json-tests")]
	/// General constructor.
	pub fn from_pod(pod: PodAccount) -> Account {
		Account {
			balance: pod.balance,
			nonce: pod.nonce,
			storage_root: SHA3_NULL_RLP,
			storage_overlay: RefCell::new(pod.storage.into_iter().map(|(k, v)| (k, (Filth::Dirty, v))).collect()),
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
	pub fn new_contract(balance: U256, nonce: U256) -> Account {
		Account {
			balance: balance,
			nonce: nonce,
			storage_root: SHA3_NULL_RLP,
			storage_overlay: RefCell::new(HashMap::new()),
			code_hash: None,
			code_cache: vec![],
		}
	}

	/// Set this account's code to the given code.
	/// NOTE: Account should have been created with `new_contract()`
	pub fn init_code(&mut self, code: Bytes) {
		assert!(self.code_hash.is_none());
		self.code_cache = code;
	}

	/// Set (and cache) the contents of the trie's storage at `key` to `value`.
	pub fn set_storage(&mut self, key: H256, value: H256) {
		self.storage_overlay.borrow_mut().insert(key, (Filth::Dirty, value));
	}

	/// Get (and cache) the contents of the trie's storage at `key`.
	pub fn storage_at(&self, db: &AccountDB, key: &H256) -> H256 {
		self.storage_overlay.borrow_mut().entry(key.clone()).or_insert_with(||{
			(Filth::Clean, H256::from(SecTrieDB::new(db, &self.storage_root).get(key.bytes()).map_or(U256::zero(), |v| -> U256 {decode(v)})))
		}).1.clone()
	}

	/// return the balance associated with this account.
	pub fn balance(&self) -> &U256 { &self.balance }

	/// return the nonce associated with this account.
	pub fn nonce(&self) -> &U256 { &self.nonce }

	#[cfg(test)]
	/// return the code hash associated with this account.
	pub fn code_hash(&self) -> H256 {
		self.code_hash.clone().unwrap_or(SHA3_EMPTY)
	}

	/// returns the account's code. If `None` then the code cache isn't available -
	/// get someone who knows to call `note_code`.
	pub fn code(&self) -> Option<&[u8]> {
		match self.code_hash {
			Some(c) if c == SHA3_EMPTY && self.code_cache.is_empty() => Some(&self.code_cache),
			Some(_) if !self.code_cache.is_empty() => Some(&self.code_cache),
			None => Some(&self.code_cache),
			_ => None,
		}
	}

	#[cfg(test)]
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
	pub fn cache_code(&mut self, db: &AccountDB) -> bool {
		// TODO: fill out self.code_cache;
		trace!("Account::cache_code: ic={}; self.code_hash={:?}, self.code_cache={}", self.is_cached(), self.code_hash, self.code_cache.pretty());
		self.is_cached() ||
			match self.code_hash {
				Some(ref h) => match db.lookup(h) {
					Some(x) => { self.code_cache = x.to_vec(); true },
					_ => {
						warn!("Failed reverse lookup of {}", h);
						false
					},
				},
				_ => false,
			}
	}

	#[cfg(test)]
	/// Determine whether there are any un-`commit()`-ed storage-setting operations.
	pub fn storage_is_clean(&self) -> bool { self.storage_overlay.borrow().iter().find(|&(_, &(f, _))| f == Filth::Dirty).is_none() }

	#[cfg(test)]
	/// return the storage root associated with this account or None if it has been altered via the overlay.
	pub fn storage_root(&self) -> Option<&H256> { if self.storage_is_clean() {Some(&self.storage_root)} else {None} }

	/// return the storage overlay.
	pub fn storage_overlay(&self) -> Ref<HashMap<H256, (Filth, H256)>> { self.storage_overlay.borrow() }

	/// Increment the nonce of the account by one.
	pub fn inc_nonce(&mut self) { self.nonce = self.nonce + U256::from(1u8); }

	/// Increment the nonce of the account by one.
	pub fn add_balance(&mut self, x: &U256) { self.balance = self.balance + *x; }

	/// Increment the nonce of the account by one.
	/// Panics if balance is less than `x`
	pub fn sub_balance(&mut self, x: &U256) {
		assert!(self.balance >= *x);
		self.balance = self.balance - *x;
	}

	/// Commit the `storage_overlay` to the backing DB and update `storage_root`.
	pub fn commit_storage(&mut self, db: &mut AccountDBMut) {
		let mut t = SecTrieDBMut::from_existing(db, &mut self.storage_root);
		for (k, &mut (ref mut f, ref mut v)) in self.storage_overlay.borrow_mut().iter_mut() {
			if f == &Filth::Dirty {
				// cast key and value to trait type,
				// so we can call overloaded `to_bytes` method
				match v.is_zero() {
					true => { t.remove(k); },
					false => { t.insert(k, &encode(&U256::from(v.as_slice()))); },
				}
				*f = Filth::Clean;
			}
		}
	}

	/// Commit any unsaved code. `code_hash` will always return the hash of the `code_cache` after this.
	pub fn commit_code(&mut self, db: &mut AccountDBMut) {
		trace!("Commiting code of {:?} - {:?}, {:?}", self, self.code_hash.is_none(), self.code_cache.is_empty());
		match (self.code_hash.is_none(), self.code_cache.is_empty()) {
			(true, true) => self.code_hash = Some(SHA3_EMPTY),
			(true, false) => {
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

	use util::*;
	use super::*;
	use account_db::*;

	#[test]
	fn storage_at() {
		let mut db = MemoryDB::new();
		let mut db = AccountDBMut::new(&mut db, &Address::new());
		let rlp = {
			let mut a = Account::new_contract(x!(69), x!(0));
			a.set_storage(H256::from(&U256::from(0x00u64)), H256::from(&U256::from(0x1234u64)));
			a.commit_storage(&mut db);
			a.init_code(vec![]);
			a.commit_code(&mut db);
			a.rlp()
		};

		let a = Account::from_rlp(&rlp);
		assert_eq!(a.storage_root().unwrap().hex(), "c57e1afb758b07f8d2c8f13a3b6e44fa5ff94ab266facc5a4fd3f062426e50b2");
		assert_eq!(a.storage_at(&db.immutable(), &H256::from(&U256::from(0x00u64))), H256::from(&U256::from(0x1234u64)));
		assert_eq!(a.storage_at(&db.immutable(), &H256::from(&U256::from(0x01u64))), H256::new());
	}

	#[test]
	fn note_code() {
		let mut db = MemoryDB::new();
		let mut db = AccountDBMut::new(&mut db, &Address::new());

		let rlp = {
			let mut a = Account::new_contract(x!(69), x!(0));
			a.init_code(vec![0x55, 0x44, 0xffu8]);
			a.commit_code(&mut db);
			a.rlp()
		};

		let mut a = Account::from_rlp(&rlp);
		assert!(a.cache_code(&db.immutable()));

		let mut a = Account::from_rlp(&rlp);
		assert_eq!(a.note_code(vec![0x55, 0x44, 0xffu8]), Ok(()));
	}

	#[test]
	fn commit_storage() {
		let mut a = Account::new_contract(x!(69), x!(0));
		let mut db = MemoryDB::new();
		let mut db = AccountDBMut::new(&mut db, &Address::new());
		a.set_storage(x!(0), x!(0x1234));
		assert_eq!(a.storage_root(), None);
		a.commit_storage(&mut db);
		assert_eq!(a.storage_root().unwrap().hex(), "c57e1afb758b07f8d2c8f13a3b6e44fa5ff94ab266facc5a4fd3f062426e50b2");
	}

	#[test]
	fn commit_remove_commit_storage() {
		let mut a = Account::new_contract(x!(69), x!(0));
		let mut db = MemoryDB::new();
		let mut db = AccountDBMut::new(&mut db, &Address::new());
		a.set_storage(x!(0), x!(0x1234));
		a.commit_storage(&mut db);
		a.set_storage(x!(1), x!(0x1234));
		a.commit_storage(&mut db);
		a.set_storage(x!(1), x!(0));
		a.commit_storage(&mut db);
		assert_eq!(a.storage_root().unwrap().hex(), "c57e1afb758b07f8d2c8f13a3b6e44fa5ff94ab266facc5a4fd3f062426e50b2");
	}

	#[test]
	fn commit_code() {
		let mut a = Account::new_contract(x!(69), x!(0));
		let mut db = MemoryDB::new();
		let mut db = AccountDBMut::new(&mut db, &Address::new());
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
