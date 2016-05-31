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

//! State of all accounts in the system expressed in Plain Old Data.

use util::*;
use pod_account::{self, PodAccount};
use types::state_diff::StateDiff;
use ethjson;

/// State of all accounts in the system expressed in Plain Old Data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PodState (BTreeMap<Address, PodAccount>);

impl PodState {
	/// Contruct a new object from the `m`.
	pub fn new() -> PodState { Default::default() }

	/// Contruct a new object from the `m`.
	pub fn from(m: BTreeMap<Address, PodAccount>) -> PodState { PodState(m) }

	/// Get the underlying map.
	pub fn get(&self) -> &BTreeMap<Address, PodAccount> { &self.0 }

	/// Get the root hash of the trie of the RLP of this.
	pub fn root(&self) -> H256 {
		sec_trie_root(self.0.iter().map(|(k, v)| (k.to_vec(), v.rlp())).collect())
	}

	/// Drain object to get the underlying map.
	pub fn drain(self) -> BTreeMap<Address, PodAccount> { self.0 }
}

impl From<ethjson::blockchain::State> for PodState {
	fn from(s: ethjson::blockchain::State) -> PodState {
		let state = s.into_iter().map(|(addr, acc)| (addr.into(), PodAccount::from(acc))).collect();
		PodState(state)
	}
}

impl From<ethjson::spec::State> for PodState {
	fn from(s: ethjson::spec::State) -> PodState {
		let state: BTreeMap<_,_> = s.into_iter()
			.filter(|pair| !pair.1.is_empty())
			.map(|(addr, acc)| (addr.into(), PodAccount::from(acc)))
			.collect();
		PodState(state)
	}
}

impl fmt::Display for PodState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (add, acc) in &self.0 {
			try!(writeln!(f, "{} => {}", add, acc));
		}
		Ok(())
	}
}

/// Calculate and return diff between `pre` state and `post` state.
pub fn diff_pod(pre: &PodState, post: &PodState) -> StateDiff {
	StateDiff(pre.get().keys().merge(post.get().keys()).filter_map(|acc| pod_account::diff_pod(pre.get().get(acc), post.get().get(acc)).map(|d|(acc.clone(), d))).collect())
}

#[cfg(test)]
mod test {
	use common::*;
	use types::state_diff::*;
	use types::account_diff::*;
	use pod_account::{self, PodAccount};
	use super::PodState;

	#[test]
	fn create_delete() {
		let a = PodState::from(map![ x!(1) => PodAccount::new(x!(69), x!(0), vec![], map![]) ]);
		assert_eq!(super::diff_pod(&a, &PodState::new()), StateDiff(map![
			x!(1) => AccountDiff{
				balance: Diff::Died(x!(69)),
				nonce: Diff::Died(x!(0)),
				code: Diff::Died(vec![]),
				storage: map![],
			}
		]));
		assert_eq!(super::diff_pod(&PodState::new(), &a), StateDiff(map![
			x!(1) => AccountDiff{
				balance: Diff::Born(x!(69)),
				nonce: Diff::Born(x!(0)),
				code: Diff::Born(vec![]),
				storage: map![],
			}
		]));
	}

	#[test]
	fn create_delete_with_unchanged() {
		let a = PodState::from(map![ x!(1) => PodAccount::new(x!(69), x!(0), vec![], map![]) ]);
		let b = PodState::from(map![
			x!(1) => PodAccount::new(x!(69), x!(0), vec![], map![]),
			x!(2) => PodAccount::new(x!(69), x!(0), vec![], map![])
		]);
		assert_eq!(super::diff_pod(&a, &b), StateDiff(map![
			x!(2) => AccountDiff{
				balance: Diff::Born(x!(69)),
				nonce: Diff::Born(x!(0)),
				code: Diff::Born(vec![]),
				storage: map![],
			}
		]));
		assert_eq!(super::diff_pod(&b, &a), StateDiff(map![
			x!(2) => AccountDiff{
				balance: Diff::Died(x!(69)),
				nonce: Diff::Died(x!(0)),
				code: Diff::Died(vec![]),
				storage: map![],
			}
		]));
	}

	#[test]
	fn change_with_unchanged() {
		let a = PodState::from(map![
			x!(1) => PodAccount::new(x!(69), x!(0), vec![], map![]),
			x!(2) => PodAccount::new(x!(69), x!(0), vec![], map![])
		]);
		let b = PodState::from(map![
			x!(1) => PodAccount::new(x!(69), x!(1), vec![], map![]),
			x!(2) => PodAccount::new(x!(69), x!(0), vec![], map![])
		]);
		assert_eq!(super::diff_pod(&a, &b), StateDiff(map![
			x!(1) => AccountDiff{
				balance: Diff::Same,
				nonce: Diff::Changed(x!(0), x!(1)),
				code: Diff::Same,
				storage: map![],
			}
		]));
	}

}
