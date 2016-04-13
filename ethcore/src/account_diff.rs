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

//! Diff between two accounts.

use util::*;
#[cfg(test)]
use pod_account::*;

#[derive(Debug,Clone,PartialEq,Eq)]
/// Change in existance type. 
// TODO: include other types of change.
pub enum Existance {
	/// Item came into existance.
	Born,
	/// Item stayed in existance.
	Alive,
	/// Item went out of existance.
	Died,
}

impl fmt::Display for Existance {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Existance::Born => try!(write!(f, "+++")),
			Existance::Alive => try!(write!(f, "***")),
			Existance::Died => try!(write!(f, "XXX")),
		}
		Ok(())
	}
}

#[derive(Debug,Clone,PartialEq,Eq)]
/// Account diff.
pub struct AccountDiff {
	/// Change in balance, allowed to be `Diff::Same`.
	pub balance: Diff<U256>,
	/// Change in nonce, allowed to be `Diff::Same`.
	pub nonce: Diff<U256>, // Allowed to be Same
	/// Change in code, allowed to be `Diff::Same`.
	pub code: Diff<Bytes>, // Allowed to be Same
	/// Change in storage, values are not allowed to be `Diff::Same`.
	pub storage: BTreeMap<H256, Diff<H256>>,
}

impl AccountDiff {
	/// Get `Existance` projection.
	pub fn existance(&self) -> Existance {
		match self.balance {
			Diff::Born(_) => Existance::Born,
			Diff::Died(_) => Existance::Died,
			_ => Existance::Alive,
		}
	}

	#[cfg(test)]
	/// Determine difference between two optionally existant `Account`s. Returns None
	/// if they are the same.
	pub fn diff_pod(pre: Option<&PodAccount>, post: Option<&PodAccount>) -> Option<AccountDiff> {
		match (pre, post) {
			(None, Some(x)) => Some(AccountDiff {
				balance: Diff::Born(x.balance),
				nonce: Diff::Born(x.nonce),
				code: Diff::Born(x.code.clone()),
				storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Born(v.clone()))).collect(),
			}),
			(Some(x), None) => Some(AccountDiff {
				balance: Diff::Died(x.balance),
				nonce: Diff::Died(x.nonce),
				code: Diff::Died(x.code.clone()),
				storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Died(v.clone()))).collect(),
			}),
			(Some(pre), Some(post)) => {
				let storage: Vec<_> = pre.storage
				                         .keys()
				                         .merge(post.storage.keys())
				                         .filter(|k| pre.storage.get(k).unwrap_or(&H256::new()) != post.storage.get(k).unwrap_or(&H256::new()))
				                         .collect();
				let r = AccountDiff {
					balance: Diff::new(pre.balance, post.balance),
					nonce: Diff::new(pre.nonce, post.nonce),
					code: Diff::new(pre.code.clone(), post.code.clone()),
					storage: storage.into_iter()
					                .map(|k| (k.clone(), Diff::new(pre.storage.get(&k).cloned().unwrap_or_else(H256::new), post.storage.get(&k).cloned().unwrap_or_else(H256::new))))
					                .collect(),
				};
				if r.balance.is_same() && r.nonce.is_same() && r.code.is_same() && r.storage.is_empty() { None } else { Some(r) }
			}
			_ => None,
		}
	}
}

// TODO: refactor into something nicer.
fn interpreted_hash(u: &H256) -> String {
	if u <= &H256::from(0xffffffff) {
		format!("{} = 0x{:x}", U256::from(u.as_slice()).low_u32(), U256::from(u.as_slice()).low_u32())
	} else if u <= &H256::from(u64::max_value()) {
		format!("{} = 0x{:x}", U256::from(u.as_slice()).low_u64(), U256::from(u.as_slice()).low_u64())
		// 	} else if u <= &H256::from("0xffffffffffffffffffffffffffffffffffffffff") {
		// 		format!("@{}", Address::from(u))
	} else {
		format!("#{}", u)
	}
}

impl fmt::Display for AccountDiff {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.nonce {
			Diff::Born(ref x) => try!(write!(f, "  non {}", x)),
			Diff::Changed(ref pre, ref post) => try!(write!(f, "#{} ({} {} {})", post, pre, if pre > post { "-" } else { "+" }, *max(pre, post) - *min(pre, post))),
			_ => {}
		}
		match self.balance {
			Diff::Born(ref x) => try!(write!(f, "  bal {}", x)),
			Diff::Changed(ref pre, ref post) => try!(write!(f, "${} ({} {} {})", post, pre, if pre > post { "-" } else { "+" }, *max(pre, post) - *min(pre, post))),
			_ => {}
		}
		if let Diff::Born(ref x) = self.code {
			try!(write!(f, "  code {}", x.pretty()));
		}
		try!(write!(f, "\n"));
		for (k, dv) in &self.storage {
			match *dv {
				Diff::Born(ref v) => try!(write!(f, "    +  {} => {}\n", interpreted_hash(k), interpreted_hash(v))),
				Diff::Changed(ref pre, ref post) => try!(write!(f, "    *  {} => {} (was {})\n", interpreted_hash(k), interpreted_hash(post), interpreted_hash(pre))),
				Diff::Died(_) => try!(write!(f, "    X  {}\n", interpreted_hash(k))),
				_ => {}
			}
		}
		Ok(())
	}
}
