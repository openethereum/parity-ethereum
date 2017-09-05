// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::fmt;
use std::collections::BTreeMap;
use itertools::Itertools;
use hash::{keccak};
use bigint::prelude::U256;
use bigint::hash::H256;
use triehash::sec_trie_root;
use util::*;
use state::Account;
use ethjson;
use types::account_diff::*;
use rlp::{self, RlpStream};

#[derive(Debug, Clone, PartialEq, Eq)]
/// An account, expressed as Plain-Old-Data (hence the name).
/// Does not have a DB overlay cache, code hash or anything like that.
pub struct PodAccount {
	/// The balance of the account.
	pub balance: U256,
	/// The nonce of the account.
	pub nonce: U256,
	/// The code of the account or `None` in the special case that it is unknown.
	pub code: Option<Bytes>,
	/// The storage of the account.
	pub storage: BTreeMap<H256, H256>,
}

impl PodAccount {
	/// Construct new object.
	#[cfg(test)]
	pub fn new(balance: U256, nonce: U256, code: Bytes, storage: BTreeMap<H256, H256>) -> PodAccount {
		PodAccount { balance: balance, nonce: nonce, code: Some(code), storage: storage }
	}

	/// Convert Account to a PodAccount.
	/// NOTE: This will silently fail unless the account is fully cached.
	pub fn from_account(acc: &Account) -> PodAccount {
		PodAccount {
			balance: *acc.balance(),
			nonce: *acc.nonce(),
			storage: acc.storage_changes().iter().fold(BTreeMap::new(), |mut m, (k, v)| {m.insert(k.clone(), v.clone()); m}),
			code: acc.code().map(|x| x.to_vec()),
		}
	}

	/// Returns the RLP for this account.
	pub fn rlp(&self) -> Bytes {
		let mut stream = RlpStream::new_list(4);
		stream.append(&self.nonce);
		stream.append(&self.balance);
		stream.append(&sec_trie_root(self.storage.iter().map(|(k, v)| (k.to_vec(), rlp::encode(&U256::from(&**v)).to_vec())).collect()));
		stream.append(&keccak(&self.code.as_ref().unwrap_or(&vec![])));
		stream.out()
	}

	/// Place additional data into given hash DB.
	pub fn insert_additional(&self, db: &mut HashDB, factory: &TrieFactory) {
		match self.code {
			Some(ref c) if !c.is_empty() => { db.insert(c); }
			_ => {}
		}
		let mut r = H256::new();
		let mut t = factory.create(db, &mut r);
		for (k, v) in &self.storage {
			if let Err(e) = t.insert(k, &rlp::encode(&U256::from(&**v))) {
				warn!("Encountered potential DB corruption: {}", e);
			}
		}
	}
}

impl From<ethjson::blockchain::Account> for PodAccount {
	fn from(a: ethjson::blockchain::Account) -> Self {
		PodAccount {
			balance: a.balance.into(),
			nonce: a.nonce.into(),
			code: Some(a.code.into()),
			storage: a.storage.into_iter().map(|(key, value)| {
				let key: U256 = key.into();
				let value: U256 = value.into();
				(H256::from(key), H256::from(value))
			}).collect(),
		}
	}
}

impl From<ethjson::spec::Account> for PodAccount {
	fn from(a: ethjson::spec::Account) -> Self {
		PodAccount {
			balance: a.balance.map_or_else(U256::zero, Into::into),
			nonce: a.nonce.map_or_else(U256::zero, Into::into),
			code: Some(a.code.map_or_else(Vec::new, Into::into)),
			storage: a.storage.map_or_else(BTreeMap::new, |s| s.into_iter().map(|(key, value)| {
				let key: U256 = key.into();
				let value: U256 = value.into();
				(H256::from(key), H256::from(value))
			}).collect()),
		}
	}
}

impl fmt::Display for PodAccount {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "(bal={}; nonce={}; code={} bytes, #{}; storage={} items)",
			self.balance,
			self.nonce,
			self.code.as_ref().map_or(0, |c| c.len()),
			self.code.as_ref().map_or_else(H256::new, |c| keccak(c)),
			self.storage.len(),
		)
	}
}

/// Determine difference between two optionally existant `Account`s. Returns None
/// if they are the same.
pub fn diff_pod(pre: Option<&PodAccount>, post: Option<&PodAccount>) -> Option<AccountDiff> {
	match (pre, post) {
		(None, Some(x)) => Some(AccountDiff {
			balance: Diff::Born(x.balance),
			nonce: Diff::Born(x.nonce),
			code: Diff::Born(x.code.as_ref().expect("account is newly created; newly created accounts must be given code; all caches should remain in place; qed").clone()),
			storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Born(v.clone()))).collect(),
		}),
		(Some(x), None) => Some(AccountDiff {
			balance: Diff::Died(x.balance),
			nonce: Diff::Died(x.nonce),
			code: Diff::Died(x.code.as_ref().expect("account is deleted; only way to delete account is running SUICIDE; account must have had own code cached to make operation; all caches should remain in place; qed").clone()),
			storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Died(v.clone()))).collect(),
		}),
		(Some(pre), Some(post)) => {
			let storage: Vec<_> = pre.storage.keys().merge(post.storage.keys())
				.filter(|k| pre.storage.get(k).unwrap_or(&H256::new()) != post.storage.get(k).unwrap_or(&H256::new()))
				.collect();
			let r = AccountDiff {
				balance: Diff::new(pre.balance, post.balance),
				nonce: Diff::new(pre.nonce, post.nonce),
				code: match (pre.code.clone(), post.code.clone()) {
					(Some(pre_code), Some(post_code)) => Diff::new(pre_code, post_code),
					_ => Diff::Same,
				},
				storage: storage.into_iter().map(|k|
					(k.clone(), Diff::new(
						pre.storage.get(k).cloned().unwrap_or_else(H256::new),
						post.storage.get(k).cloned().unwrap_or_else(H256::new)
					))).collect(),
			};
			if r.balance.is_same() && r.nonce.is_same() && r.code.is_same() && r.storage.is_empty() {
				None
			} else {
				Some(r)
			}
		},
		_ => None,
	}
}


#[cfg(test)]
mod test {
	use std::collections::BTreeMap;
	use types::account_diff::*;
	use super::{PodAccount, diff_pod};

	#[test]
	fn existence() {
		let a = PodAccount{balance: 69.into(), nonce: 0.into(), code: Some(vec![]), storage: map![]};
		assert_eq!(diff_pod(Some(&a), Some(&a)), None);
		assert_eq!(diff_pod(None, Some(&a)), Some(AccountDiff{
			balance: Diff::Born(69.into()),
			nonce: Diff::Born(0.into()),
			code: Diff::Born(vec![]),
			storage: map![],
		}));
	}

	#[test]
	fn basic() {
		let a = PodAccount{balance: 69.into(), nonce: 0.into(), code: Some(vec![]), storage: map![]};
		let b = PodAccount{balance: 42.into(), nonce: 1.into(), code: Some(vec![]), storage: map![]};
		assert_eq!(diff_pod(Some(&a), Some(&b)), Some(AccountDiff {
			balance: Diff::Changed(69.into(), 42.into()),
			nonce: Diff::Changed(0.into(), 1.into()),
			code: Diff::Same,
			storage: map![],
		}));
	}

	#[test]
	fn code() {
		let a = PodAccount{balance: 0.into(), nonce: 0.into(), code: Some(vec![]), storage: map![]};
		let b = PodAccount{balance: 0.into(), nonce: 1.into(), code: Some(vec![0]), storage: map![]};
		assert_eq!(diff_pod(Some(&a), Some(&b)), Some(AccountDiff {
			balance: Diff::Same,
			nonce: Diff::Changed(0.into(), 1.into()),
			code: Diff::Changed(vec![], vec![0]),
			storage: map![],
		}));
	}

	#[test]
	fn storage() {
		let a = PodAccount {
			balance: 0.into(),
			nonce: 0.into(),
			code: Some(vec![]),
			storage: map_into![1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 0, 6 => 0, 7 => 0]
		};
		let b = PodAccount {
			balance: 0.into(),
			nonce: 0.into(),
			code: Some(vec![]),
			storage: map_into![1 => 1, 2 => 3, 3 => 0, 5 => 0, 7 => 7, 8 => 0, 9 => 9]
		};
		assert_eq!(diff_pod(Some(&a), Some(&b)), Some(AccountDiff {
			balance: Diff::Same,
			nonce: Diff::Same,
			code: Diff::Same,
			storage: map![
				2.into() => Diff::new(2.into(), 3.into()),
				3.into() => Diff::new(3.into(), 0.into()),
				4.into() => Diff::new(4.into(), 0.into()),
				7.into() => Diff::new(0.into(), 7.into()),
				9.into() => Diff::new(0.into(), 9.into())
			],
		}));
	}
}
