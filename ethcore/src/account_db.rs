//! DB backend wrapper for Account trie
use util::*;

static NULL_RLP_STATIC: [u8; 1] = [0x80; 1];

// TODO: introduce HashDBMut?
/// DB backend wrapper for Account trie
/// Transforms trie node keys for the database
pub struct AccountDB<'db> {
	db: &'db HashDB,
	address: H256,
}

#[inline]
fn combine_key<'a>(address: &'a H256, key: &'a H256) -> H256 {
	address ^ key
}

impl<'db> AccountDB<'db> {
	pub fn new(db: &'db HashDB, address: &Address) -> AccountDB<'db> {
		AccountDB {
			db: db,
			address: x!(address),
		}
	}
}

impl<'db> HashDB for AccountDB<'db> {
	fn keys(&self) -> HashMap<H256, i32> {
		unimplemented!()
	}

	fn lookup(&self, key: &H256) -> Option<&[u8]> {
		if key == &SHA3_NULL_RLP {
			return Some(&NULL_RLP_STATIC);
		}
		self.db.lookup(&combine_key(&self.address, key))
	}

	fn exists(&self, key: &H256) -> bool {
		if key == &SHA3_NULL_RLP {
			return true;
		}
		self.db.exists(&combine_key(&self.address, key))
	}

	fn insert(&mut self, _value: &[u8]) -> H256 {
		unimplemented!()
	}

	fn emplace(&mut self, _key: H256, _value: Bytes) {
		unimplemented!()
	}

	fn kill(&mut self, _key: &H256) {
		unimplemented!()
	}
}

/// DB backend wrapper for Account trie
pub struct AccountDBMut<'db> {
	db: &'db mut HashDB,
	address: H256,
}

impl<'db> AccountDBMut<'db> {
	pub fn new(db: &'db mut HashDB, address: &Address) -> AccountDBMut<'db> {
		AccountDBMut {
			db: db,
			address: x!(address),
		}
	}

	#[allow(dead_code)]
	pub fn immutable(&'db self) -> AccountDB<'db> {
		AccountDB {
			db: self.db,
			address: self.address.clone(),
		}
	}
}

impl<'db> HashDB for AccountDBMut<'db> {
	fn keys(&self) -> HashMap<H256, i32> {
		unimplemented!()
	}

	fn lookup(&self, key: &H256) -> Option<&[u8]> {
		if key == &SHA3_NULL_RLP {
			return Some(&NULL_RLP_STATIC);
		}
		self.db.lookup(&combine_key(&self.address, key))
	}

	fn exists(&self, key: &H256) -> bool {
		if key == &SHA3_NULL_RLP {
			return true;
		}
		self.db.exists(&combine_key(&self.address, key))
	}

	fn insert(&mut self, value: &[u8]) -> H256 {
		if value == &NULL_RLP {
			return SHA3_NULL_RLP.clone();
		}
		let k = value.sha3();
		let ak = combine_key(&self.address, &k);
		self.db.emplace(ak, value.to_vec());
		k
	}

	fn emplace(&mut self, key: H256, value: Bytes) {
		if key == SHA3_NULL_RLP {
			return;
		}
		let key = combine_key(&self.address, &key);
		self.db.emplace(key, value.to_vec())
	}

	fn kill(&mut self, key: &H256) {
		if key == &SHA3_NULL_RLP {
			return;
		}
		let key = combine_key(&self.address, key);
		self.db.kill(&key)
	}
}
