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
#[cfg(test)]
use pod_state::*;
use account_diff::*;

#[derive(Debug,Clone,PartialEq,Eq)]
/// Expression for the delta between two system states. Encoded the
/// delta of every altered account.
pub struct StateDiff (BTreeMap<Address, AccountDiff>);

impl StateDiff {
	#[cfg(test)]
	/// Calculate and return diff between `pre` state and `post` state.
	pub fn diff_pod(pre: &PodState, post: &PodState) -> StateDiff {
		StateDiff(pre.get().keys().merge(post.get().keys()).filter_map(|acc| AccountDiff::diff_pod(pre.get().get(acc), post.get().get(acc)).map(|d|(acc.clone(), d))).collect())
	}
}

impl fmt::Display for StateDiff {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (add, acc) in &self.0 {
			try!(write!(f, "{} {}: {}", acc.existance(), add, acc));
		}
		Ok(())
	}
}

impl Deref for StateDiff {
	type Target = BTreeMap<Address, AccountDiff>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[cfg(test)]
mod test {
	use common::*;
	use pod_state::*;
	use account_diff::*;
	use pod_account::*;
	use super::*;

	#[test]
	fn create_delete() {
		let a = PodState::from(map![ x!(1) => PodAccount::new(x!(69), x!(0), vec![], map![]) ]);
		assert_eq!(StateDiff::diff_pod(&a, &PodState::new()), StateDiff(map![
			x!(1) => AccountDiff{
				balance: Diff::Died(x!(69)),
				nonce: Diff::Died(x!(0)),
				code: Diff::Died(vec![]),
				storage: map![],
			}
		]));
		assert_eq!(StateDiff::diff_pod(&PodState::new(), &a), StateDiff(map![
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
		assert_eq!(StateDiff::diff_pod(&a, &b), StateDiff(map![
			x!(2) => AccountDiff{
				balance: Diff::Born(x!(69)),
				nonce: Diff::Born(x!(0)),
				code: Diff::Born(vec![]),
				storage: map![],
			}
		]));
		assert_eq!(StateDiff::diff_pod(&b, &a), StateDiff(map![
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
		assert_eq!(StateDiff::diff_pod(&a, &b), StateDiff(map![
			x!(1) => AccountDiff{
				balance: Diff::Same,
				nonce: Diff::Changed(x!(0), x!(1)),
				code: Diff::Same,
				storage: map![],
			}
		]));
	}

}
