use std::collections::HashMap;
use util::hash::*;
use util::hashdb::*;
use util::bytes::*;
use util::trie::*;
use util::rlp::*;
use util::uint::*;

enum HashOrData {
	Hash(H256),
	Data(Bytes),
	Both(H256, Bytes),
}

pub struct Account {
	balance: U256,
	nonce: U256,
	// Trie-backed storage.
	storage_root: H256,
	// Overlay on trie-backed storage.
	storage_overlay: HashMap<H256, H256>,
	code: HashOrData,
}

impl Account {
	pub fn new_with_balance(balance: U256) -> Account {
		Account {
			balance: balance,
			nonce: U256::from(0u8),
			code: HashOrData::Data(vec![]),
			storage_root: SHA3_NULL_RLP,
			storage_overlay: HashMap::new(),
		}
	}

	pub fn from_rlp(_rlp: &[u8]) -> Account {
		//TODO
		Account {
			balance: U256::from(0u8),
			nonce: U256::from(0u8),
			code: HashOrData::Hash(SHA3_NULL_RLP),
			storage_root: SHA3_NULL_RLP,
			storage_overlay: HashMap::new(),
		}
	}

	pub fn balance(&self) -> &U256 { &self.balance }
	pub fn nonce(&self) -> &U256 { &self.nonce }
	pub fn code_hash(&self) -> Option<&H256> {
		match self.code {
			HashOrData::Hash(ref h) | HashOrData::Both(ref h, _) => Some(h),
			_ => None,
		}
	}
	pub fn storage_root(&self) -> Option<&H256> { if self.storage_overlay.is_empty() {Some(&self.storage_root)} else {None} }

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

	/// Commit any unsaved code and ensure code is not `HashOrData::Data`.
	pub fn commit_code(&mut self, db: &mut HashDB) {
		if let Some(new_code) = match self.code {
			HashOrData::Data(ref d) => { Some(HashOrData::Both(db.insert(d), d.clone())) },
			_ => None,
		}{
			self.code = new_code;
		}
	}
}
