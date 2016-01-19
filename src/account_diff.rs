use util::*;
use pod_account::*;

#[derive(Debug,Clone,PartialEq,Eq)]
/// Change in existance type. 
// TODO: include other types of change.
pub enum Existance {
	Born,
	Alive,
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
pub struct AccountDiff {
	pub balance: Diff<U256>,				// Allowed to be Same
	pub nonce: Diff<U256>,					// Allowed to be Same
	pub code: Diff<Bytes>,					// Allowed to be Same
	pub storage: BTreeMap<H256, Diff<H256>>,// Not allowed to be Same
}

impl AccountDiff {
	pub fn existance(&self) -> Existance {
		match self.balance {
			Diff::Born(_) => Existance::Born,
			Diff::Died(_) => Existance::Died,
			_ => Existance::Alive,
		}
	}

	pub fn diff_pod(pre: Option<&PodAccount>, post: Option<&PodAccount>) -> Option<AccountDiff> {
		match (pre, post) {
			(None, Some(x)) => Some(AccountDiff {
				balance: Diff::Born(x.balance.clone()),
				nonce: Diff::Born(x.nonce.clone()),
				code: Diff::Born(x.code.clone()),
				storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Born(v.clone()))).collect(),
			}),
			(Some(x), None) => Some(AccountDiff {
				balance: Diff::Died(x.balance.clone()),
				nonce: Diff::Died(x.nonce.clone()),
				code: Diff::Died(x.code.clone()),
				storage: x.storage.iter().map(|(k, v)| (k.clone(), Diff::Died(v.clone()))).collect(),
			}),
			(Some(pre), Some(post)) => {
				let storage: Vec<_> = pre.storage.keys().merge(post.storage.keys())
					.filter(|k| pre.storage.get(k).unwrap_or(&H256::new()) != post.storage.get(k).unwrap_or(&H256::new()))
					.collect();
				let r = AccountDiff {
					balance: Diff::new(pre.balance.clone(), post.balance.clone()),
					nonce: Diff::new(pre.nonce.clone(), post.nonce.clone()),
					code: Diff::new(pre.code.clone(), post.code.clone()),
					storage: storage.into_iter().map(|k|
						(k.clone(), Diff::new(
							pre.storage.get(&k).cloned().unwrap_or(H256::new()),
							post.storage.get(&k).cloned().unwrap_or(H256::new())
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
}

// TODO: refactor into something nicer.
fn interpreted_hash(u: &H256) -> String {
	if u <= &H256::from(0xffffffff) {
		format!("{} = 0x{:x}", U256::from(u.as_slice()).low_u32(), U256::from(u.as_slice()).low_u32())
	} else if u <= &H256::from(u64::max_value()) {
		format!("{} = 0x{:x}", U256::from(u.as_slice()).low_u64(), U256::from(u.as_slice()).low_u64())
//	} else if u <= &H256::from("0xffffffffffffffffffffffffffffffffffffffff") {
//		format!("@{}", Address::from(u))
	} else {
		format!("#{}", u)
	}
}

impl fmt::Display for AccountDiff {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.nonce {
			Diff::Born(ref x) => try!(write!(f, "  non {}", x)),
			Diff::Changed(ref pre, ref post) => try!(write!(f, "#{} ({} {} {})", post, pre, if pre > post {"-"} else {"+"}, *max(pre, post) - *	min(pre, post))),
			_ => {},
		}
		match self.balance {
			Diff::Born(ref x) => try!(write!(f, "  bal {}", x)),
			Diff::Changed(ref pre, ref post) => try!(write!(f, "${} ({} {} {})", post, pre, if pre > post {"-"} else {"+"}, *max(pre, post) - *min(pre, post))),
			_ => {},
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
				_ => {},
			}
		}
		Ok(())
	}
}

