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

use std::cmp;
use std::collections::HashMap;

use ethereum_types::{H160 as Sender, U256};
use {pool, scoring, Scoring, Ready, Readiness};
use super::Transaction;

#[derive(Debug, Default)]
pub struct DummyScoring;

impl Scoring<Transaction> for DummyScoring {
	type Score = U256;
	type Event = ();

	fn compare(&self, old: &Transaction, new: &Transaction) -> cmp::Ordering {
		old.nonce.cmp(&new.nonce)
	}

	fn choose(&self, old: &Transaction, new: &Transaction) -> scoring::Choice {
		if old.nonce == new.nonce {
			if new.gas_price > old.gas_price {
				scoring::Choice::ReplaceOld
			} else {
				scoring::Choice::RejectNew
			}
		} else {
			scoring::Choice::InsertNew
		}
	}

	fn update_scores(&self, txs: &[pool::Transaction<Transaction>], scores: &mut [Self::Score], change: scoring::Change) {
		if let scoring::Change::Event(_) = change {
			// In case of event reset all scores to 0
			for i in 0..txs.len() {
				scores[i] = 0.into();
			}
		} else {
			// Set to a gas price otherwise
			for i in 0..txs.len() {
				scores[i] = txs[i].gas_price;
			}
		}
	}

	fn should_replace(&self, old: &Transaction, new: &Transaction) -> bool {
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

impl Ready<Transaction> for NonceReady {
	fn is_ready(&mut self, tx: &Transaction) -> Readiness {
		let min = self.1;
		let nonce = self.0.entry(tx.sender).or_insert_with(|| min);
		match tx.nonce.cmp(nonce) {
			cmp::Ordering::Greater => Readiness::Future,
			cmp::Ordering::Equal => {
				*nonce = *nonce + 1.into();
				Readiness::Ready
			},
			cmp::Ordering::Less => Readiness::Stale,
		}
	}
}
