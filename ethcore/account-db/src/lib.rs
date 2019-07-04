// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! DB backend wrapper for Account trie
use ethereum_types::H256;
use keccak_hash::{KECCAK_NULL_RLP, keccak};
use hash_db::{HashDB, AsHashDB, Prefix};
use keccak_hasher::KeccakHasher;
use kvdb::DBValue;
use rlp::NULL_RLP;

// Combines a key with an address hash to ensure uniqueness.
// leaves the first 96 bits untouched in order to support partial key lookup.
#[inline]
fn combine_key<'a>(address_hash: &'a H256, key: &'a H256) -> H256 {
	let mut dst = key.clone();
	{
		let last_src: &[u8] = address_hash.as_bytes();
		let last_dst: &mut [u8] = dst.as_bytes_mut();
		for (k, a) in last_dst[12..].iter_mut().zip(&last_src[12..]) {
			*k ^= *a
		}
	}

	dst
}

/// A factory for different kinds of account dbs.
#[derive(Debug, Clone)]
pub enum Factory {
	/// Mangle hashes based on address. This is the default.
	Mangled,
	/// Don't mangle hashes.
	Plain,
}

impl Default for Factory {
	fn default() -> Self { Factory::Mangled }
}

impl Factory {
	/// Create a read-only accountdb.
	/// This will panic when write operations are called.
	pub fn readonly<'db>(&self, db: &'db dyn HashDB<KeccakHasher, DBValue>, address_hash: H256) -> Box<dyn HashDB<KeccakHasher, DBValue> + 'db> {
		match *self {
			Factory::Mangled => Box::new(AccountDB::from_hash(db, address_hash)),
			Factory::Plain => Box::new(Wrapping(db)),
		}
	}

	/// Create a new mutable hashdb.
	pub fn create<'db>(&self, db: &'db mut dyn HashDB<KeccakHasher, DBValue>, address_hash: H256) -> Box<dyn HashDB<KeccakHasher, DBValue> + 'db> {
		match *self {
			Factory::Mangled => Box::new(AccountDBMut::from_hash(db, address_hash)),
			Factory::Plain => Box::new(WrappingMut(db)),
		}
	}
}

// TODO: introduce HashDBMut?
/// DB backend wrapper for Account trie
/// Transforms trie node keys for the database
pub struct AccountDB<'db> {
	db: &'db dyn HashDB<KeccakHasher, DBValue>,
	address_hash: H256,
}

impl<'db> AccountDB<'db> {
	/// Create a new AccountDB from an address' hash.
	pub fn from_hash(db: &'db dyn HashDB<KeccakHasher, DBValue>, address_hash: H256) -> Self {
		AccountDB { db, address_hash }
	}
}

impl<'db> AsHashDB<KeccakHasher, DBValue> for AccountDB<'db> {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

impl<'db> HashDB<KeccakHasher, DBValue> for AccountDB<'db> {
	fn get(&self, key: &H256, prefix: Prefix) -> Option<DBValue> {
		if key == &KECCAK_NULL_RLP {
			return Some(DBValue::from_slice(&NULL_RLP));
		}
		self.db.get(&combine_key(&self.address_hash, key), prefix)
	}

	fn contains(&self, key: &H256, prefix: Prefix) -> bool {
		if key == &KECCAK_NULL_RLP {
			return true;
		}
		self.db.contains(&combine_key(&self.address_hash, key), prefix)
	}

	fn insert(&mut self, _prefix: Prefix, _value: &[u8]) -> H256 {
		unimplemented!()
	}

	fn emplace(&mut self, _key: H256, _prefix: Prefix, _value: DBValue) {
		unimplemented!()
	}

	fn remove(&mut self, _key: &H256, _prefix: Prefix) {
		unimplemented!()
	}
}

/// DB backend wrapper for Account trie
pub struct AccountDBMut<'db> {
	db: &'db mut dyn HashDB<KeccakHasher, DBValue>,
	address_hash: H256,
}

impl<'db> AccountDBMut<'db> {
	/// Create a new `AccountDBMut` from an address' hash.
	pub fn from_hash(db: &'db mut dyn HashDB<KeccakHasher, DBValue>, address_hash: H256) -> Self {
		AccountDBMut { db, address_hash }
	}

	/// Create an `AccountDB` from an `AccountDBMut` (used in tests).
	pub fn immutable(&'db self) -> AccountDB<'db> {
		AccountDB { db: self.db, address_hash: self.address_hash.clone() }
	}
}

impl<'db> HashDB<KeccakHasher, DBValue> for AccountDBMut<'db>{
	fn get(&self, key: &H256, prefix: Prefix) -> Option<DBValue> {
		if key == &KECCAK_NULL_RLP {
			return Some(DBValue::from_slice(&NULL_RLP));
		}
		self.db.get(&combine_key(&self.address_hash, key), prefix)
	}

	fn contains(&self, key: &H256, prefix: Prefix) -> bool {
		if key == &KECCAK_NULL_RLP {
			return true;
		}
		self.db.contains(&combine_key(&self.address_hash, key), prefix)
	}

	fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H256 {
		if value == &NULL_RLP {
			return KECCAK_NULL_RLP.clone();
		}
		let k = keccak(value);
		let ak = combine_key(&self.address_hash, &k);
		self.db.emplace(ak, prefix, DBValue::from_slice(value));
		k
	}

	fn emplace(&mut self, key: H256, prefix: Prefix, value: DBValue) {
		if key == KECCAK_NULL_RLP {
			return;
		}
		let key = combine_key(&self.address_hash, &key);
		self.db.emplace(key, prefix, value)
	}

	fn remove(&mut self, key: &H256, prefix: Prefix) {
		if key == &KECCAK_NULL_RLP {
			return;
		}
		let key = combine_key(&self.address_hash, key);
		self.db.remove(&key, prefix)
	}
}

impl<'db> AsHashDB<KeccakHasher, DBValue> for AccountDBMut<'db> {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

struct Wrapping<'db>(&'db dyn HashDB<KeccakHasher, DBValue>);

impl<'db> AsHashDB<KeccakHasher, DBValue> for Wrapping<'db> {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

impl<'db> HashDB<KeccakHasher, DBValue> for Wrapping<'db> {
	fn get(&self, key: &H256, prefix: Prefix) -> Option<DBValue> {
		if key == &KECCAK_NULL_RLP {
			return Some(DBValue::from_slice(&NULL_RLP));
		}
		self.0.get(key, prefix)
	}

	fn contains(&self, key: &H256, prefix: Prefix) -> bool {
		if key == &KECCAK_NULL_RLP {
			return true;
		}
		self.0.contains(key, prefix)
	}

	fn insert(&mut self, _prefix: Prefix, _value: &[u8]) -> H256 {
		unimplemented!()
	}

	fn emplace(&mut self, _key: H256, _prefix: Prefix, _value: DBValue) {
		unimplemented!()
	}

	fn remove(&mut self, _key: &H256, _prefix: Prefix) {
		unimplemented!()
	}
}

struct WrappingMut<'db>(&'db mut dyn HashDB<KeccakHasher, DBValue>);
impl<'db> AsHashDB<KeccakHasher, DBValue> for WrappingMut<'db> {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

impl<'db> HashDB<KeccakHasher, DBValue> for WrappingMut<'db>{
	fn get(&self, key: &H256, prefix: Prefix) -> Option<DBValue> {
		if key == &KECCAK_NULL_RLP {
			return Some(DBValue::from_slice(&NULL_RLP));
		}
		self.0.get(key, prefix)
	}

	fn contains(&self, key: &H256, prefix: Prefix) -> bool {
		if key == &KECCAK_NULL_RLP {
			return true;
		}
		self.0.contains(key, prefix)
	}

	fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H256 {
		if value == &NULL_RLP {
			return KECCAK_NULL_RLP.clone();
		}
		self.0.insert(prefix, value)
	}

	fn emplace(&mut self, key: H256, prefix: Prefix, value: DBValue) {
		if key == KECCAK_NULL_RLP {
			return;
		}
		self.0.emplace(key, prefix, value)
	}

	fn remove(&mut self, key: &H256, prefix: Prefix) {
		if key == &KECCAK_NULL_RLP {
			return;
		}
		self.0.remove(key, prefix)
	}
}
