use std::collections::HashMap;
use util::hash::*;
use util::sha3::*;
use util::hashdb::*;
use util::bytes::*;
use util::trie::*;
use util::rlp::*;
use util::uint::*;

pub const SHA3_EMPTY: H256 = H256( [0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70] );

/// Single account in the system.
pub struct Account {
	// Balance of the account.
	balance: U256,
	// Nonce of the account.
	nonce: U256,
	// Trie-backed storage.
	storage_root: H256,
	// Overlay on trie-backed storage.
	storage_overlay: HashMap<H256, H256>,
	// Code hash of the account. If None, means that it's a contract whose code has not yet been set.
	code_hash: Option<H256>,
	// Code cache of the account.
	code_cache: Bytes,
}

impl Account {
	/// General constructor.
	pub fn new(balance: U256, nonce: U256, storage: HashMap<H256, H256>, code: Bytes) -> Account {
		Account {
			balance: balance,
			nonce: nonce,
			storage_root: SHA3_NULL_RLP,
			storage_overlay: storage,
			code_hash: Some(code.sha3()),
			code_cache: code
		}
	}

	/// Create a new account with the given balance.
	pub fn new_basic(balance: U256) -> Account {
		Account {
			balance: balance,
			nonce: U256::from(0u8),
			storage_root: SHA3_NULL_RLP,
			storage_overlay: HashMap::new(),
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
			storage_overlay: HashMap::new(),
			code_hash: Some(r.val_at(3)),
			code_cache: vec![],
		}
	}

	/// Create a new contract account.
	/// NOTE: make sure you use `set_code` on this before `commit`ing.
	pub fn new_contract(balance: U256) -> Account {
		Account {
			balance: balance,
			nonce: U256::from(0u8),
			storage_root: SHA3_NULL_RLP,
			storage_overlay: HashMap::new(),
			code_hash: None,
			code_cache: vec![],
		}
	}

	/// Set this account's code to the given code.
	/// NOTE: Account should have been created with `new_contract`.
	pub fn set_code(&mut self, code: Bytes) {
		assert!(self.code_hash.is_none());
		self.code_cache = code;
	}

	pub fn set_storage(&mut self, key: H256, value: H256) {
		self.storage_overlay.insert(key, value);
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
			_ => None,
		}
	}

	/// Provide a byte array which hashes to the `code_hash`. returns the hash as a result.
	pub fn note_code(&mut self, code: Bytes) -> Result<H256, H256> {
		let h = code.sha3();
		match self.code_hash {
			Some(ref i) if h == *i => {
				self.code_cache = code;
				Ok(h)
			},
			_ => Err(h)
		}
	}

	/// return the storage root associated with this account.
	pub fn base_root(&self) -> &H256 { &self.storage_root }
	/// return the storage root associated with this account or None if it has been altered via the overlay.
	pub fn storage_root(&self) -> Option<&H256> { if self.storage_overlay.is_empty() {Some(&self.storage_root)} else {None} }
	/// rturn the storage overlay.
	pub fn storage_overlay(&self) -> &HashMap<H256, H256> { &self.storage_overlay }

	/// Increment the nonce of the account by one.
	pub fn inc_nonce(&mut self) { self.nonce = self.nonce + U256::from(1u8); }

	/// Increment the nonce of the account by one.
	pub fn add_balance(&mut self, x: &U256) { self.balance = self.balance + *x; }

	/// Increment the nonce of the account by one.
	pub fn sub_balance(&mut self, x: &U256) { self.balance = self.balance - *x; }


	/// Commit the `storage_overlay` to the backing DB and update `storage_root`.
	pub fn commit_storage(&mut self, db: &mut HashDB) {
		let mut t = TrieDB::new(db, &mut self.storage_root);
		for (k, v) in self.storage_overlay.iter() {
			// cast key and value to trait type,
			// so we can call overloaded `to_bytes` method
			t.insert(k, v);
		}
		self.storage_overlay.clear();
	}

	/// Commit any unsaved code. `code_hash` will always return the hash of the `code_cache` after this.
	pub fn commit_code(&mut self, db: &mut HashDB) {
		if self.code_hash.is_none() && !self.code_cache.is_empty() {
			self.code_hash = Some(db.insert(&self.code_cache));
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

#[test]
fn playpen() {
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
