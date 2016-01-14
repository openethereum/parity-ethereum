use util::*;
use account::*;

#[derive(Debug,Clone,PartialEq,Eq)]
/// Genesis account data. Does not have a DB overlay cache.
pub struct PodAccount {
	pub balance: U256,
	pub nonce: U256,
	pub code: Bytes,
	pub storage: BTreeMap<H256, H256>,
}

impl PodAccount {
	/// Construct new object.
	pub fn new(balance: U256, nonce: U256, code: Bytes, storage: BTreeMap<H256, H256>) -> PodAccount {
		PodAccount { balance: balance, nonce: nonce, code: code, storage: storage }
	}

	/// Convert Account to a PodAccount.
	/// NOTE: This will silently fail unless the account is fully cached.
	pub fn from_account(acc: &Account) -> PodAccount {
		PodAccount {
			balance: acc.balance().clone(),
			nonce: acc.nonce().clone(),
			storage: acc.storage_overlay().iter().fold(BTreeMap::new(), |mut m, (k, &(_, ref v))| {m.insert(k.clone(), v.clone()); m}),
			code: acc.code().unwrap().to_vec(),
		}
	}

	pub fn rlp(&self) -> Bytes {
		let mut stream = RlpStream::new_list(4);
		stream.append(&self.nonce);
		stream.append(&self.balance);
		// TODO.
		stream.append(&SHA3_NULL_RLP);
		stream.append(&self.code.sha3());
		stream.out()
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
	use super::*;
	use account_diff::*;

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
