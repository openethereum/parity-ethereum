use util::*;
use pod_state::*;
use account_diff::*;

#[derive(Debug,Clone,PartialEq,Eq)]
/// TODO [Gav Wood] Please document me
pub struct StateDiff (BTreeMap<Address, AccountDiff>);

impl StateDiff {
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
