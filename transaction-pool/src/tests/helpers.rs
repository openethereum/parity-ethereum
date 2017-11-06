use std::cmp;
use std::collections::HashMap;

use {scoring, Scoring, Ready, Readiness, Address as Sender, U256, VerifiedTransaction, SharedTransaction};

#[derive(Default)]
pub struct DummyScoring;

impl Scoring for DummyScoring {
	type Score = U256;

	fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering {
		old.nonce.cmp(&other.nonce)
	}

	fn choose(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> scoring::Choice {
		let decision = if old.nonce == new.nonce {
			if new.gas_price > old.gas_price {
				scoring::Choice::ReplaceOld
			} else {
				scoring::Choice::RejectNew
			}
		} else {
			scoring::Choice::InsertNew
		};

		decision
	}

	fn update_scores(&self, txs: &[SharedTransaction], scores: &mut [Self::Score], _change: scoring::Change) {
		for i in 0..txs.len() {
			scores[i] = txs[i].gas_price;
		}
	}

	fn should_replace(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> bool {
		new.gas_price > old.gas_price
	}
}

#[derive(Default)]
pub struct NonceReady(HashMap<Sender, U256>, U256);

impl NonceReady {
	pub fn new<T: Into<U256>>(min: T) -> Self {
		let mut n = NonceReady::default();
		n.1 = min.into();
		n
	}
}

impl Ready for NonceReady {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> Readiness {
		let min = self.1;
		let nonce = self.0.entry(tx.sender()).or_insert_with(|| min);
		match tx.nonce.cmp(nonce) {
			cmp::Ordering::Greater => Readiness::Future,
			cmp::Ordering::Equal => {
				*nonce = *nonce + 1.into();
				Readiness::Ready
			},
			cmp::Ordering::Less => Readiness::Stalled,
		}
	}
}
