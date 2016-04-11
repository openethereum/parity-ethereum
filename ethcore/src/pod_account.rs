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

use util::*;
use account::*;
use account_db::*;
use ethjson;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An account, expressed as Plain-Old-Data (hence the name).
/// Does not have a DB overlay cache, code hash or anything like that.
pub struct PodAccount {
	/// The balance of the account.
	pub balance: U256,
	/// The nonce of the account.
	pub nonce: U256,
	/// The code of the account.
	pub code: Bytes,
	/// The storage of the account.
	pub storage: BTreeMap<H256, H256>,
}

impl PodAccount {
	/// Construct new object.
	#[cfg(test)]
	pub fn new(balance: U256, nonce: U256, code: Bytes, storage: BTreeMap<H256, H256>) -> PodAccount {
		PodAccount { balance: balance, nonce: nonce, code: code, storage: storage }
	}

	/// Convert Account to a PodAccount.
	/// NOTE: This will silently fail unless the account is fully cached.
	pub fn from_account(acc: &Account) -> PodAccount {
		PodAccount {
			balance: *acc.balance(),
			nonce: *acc.nonce(),
			storage: acc.storage_overlay().iter().fold(BTreeMap::new(), |mut m, (k, &(_, ref v))| {m.insert(k.clone(), v.clone()); m}),
			code: acc.code().unwrap().to_vec(),
		}
	}

	/// Returns the RLP for this account.
	pub fn rlp(&self) -> Bytes {
		let mut stream = RlpStream::new_list(4);
		stream.append(&self.nonce);
		stream.append(&self.balance);
		stream.append(&sec_trie_root(self.storage.iter().map(|(k, v)| (k.to_vec(), encode(&U256::from(v.as_slice())).to_vec())).collect()));
		stream.append(&self.code.sha3());
		stream.out()
	}

	/// Place additional data into given hash DB.
	pub fn insert_additional(&self, db: &mut AccountDBMut) {
		if !self.code.is_empty() {
			db.insert(&self.code);
		}
		let mut r = H256::new();
		let mut t = SecTrieDBMut::new(db, &mut r);
		for (k, v) in &self.storage {
			t.insert(k, &encode(&U256::from(v.as_slice())));
		}
	}
}

impl From<ethjson::blockchain::Account> for PodAccount {
	fn from(a: ethjson::blockchain::Account) -> Self {
		PodAccount {
			balance: a.balance.into(),
			nonce: a.nonce.into(),
			code: a.code.into(),
			storage: a.storage.into_iter().map(|(key, value)| {
				let key: U256 = key.into();
				let value: U256 = value.into();
				(H256::from(key), H256::from(value))
			}).collect()
		}
	}
}

impl From<ethjson::spec::Account> for PodAccount {
	fn from(a: ethjson::spec::Account) -> Self {
		PodAccount {
			balance: a.balance.map_or_else(U256::zero, Into::into),
			nonce: a.nonce.map_or_else(U256::zero, Into::into),
			code: vec![],
			storage: BTreeMap::new()
		}
	}
}

impl fmt::Display for PodAccount {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "(bal={}; nonce={}; code={} bytes, #{}; storage={} items)", self.balance, self.nonce, self.code.len(), self.code.sha3(), self.storage.len())
	}
}

#[cfg(test)]
mod test {
	use common::*;
	use account_diff::*;
	use super::*;

	#[test]
	fn existence() {
		let a = PodAccount{balance: x!(69), nonce: x!(0), code: vec![], storage: map![]};
		assert_eq!(AccountDiff::diff_pod(Some(&a), Some(&a)), None);
		assert_eq!(AccountDiff::diff_pod(None, Some(&a)), Some(AccountDiff{
			balance: Diff::Born(x!(69)),
			nonce: Diff::Born(x!(0)),
			code: Diff::Born(vec![]),
			storage: map![],
		}));
	}

	#[test]
	fn basic() {
		let a = PodAccount{balance: x!(69), nonce: x!(0), code: vec![], storage: map![]};
		let b = PodAccount{balance: x!(42), nonce: x!(1), code: vec![], storage: map![]};
		assert_eq!(AccountDiff::diff_pod(Some(&a), Some(&b)), Some(AccountDiff {
			balance: Diff::Changed(x!(69), x!(42)),
			nonce: Diff::Changed(x!(0), x!(1)),
			code: Diff::Same,
			storage: map![],
		}));
	}

	#[test]
	fn code() {
		let a = PodAccount{balance: x!(0), nonce: x!(0), code: vec![], storage: map![]};
		let b = PodAccount{balance: x!(0), nonce: x!(1), code: vec![0], storage: map![]};
		assert_eq!(AccountDiff::diff_pod(Some(&a), Some(&b)), Some(AccountDiff {
			balance: Diff::Same,
			nonce: Diff::Changed(x!(0), x!(1)),
			code: Diff::Changed(vec![], vec![0]),
			storage: map![],
		}));
	}

	#[test]
	fn storage() {
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
		assert_eq!(AccountDiff::diff_pod(Some(&a), Some(&b)), Some(AccountDiff {
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
}
