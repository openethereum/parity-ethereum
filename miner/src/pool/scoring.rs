// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Transaction Scoring and Ordering
//!
//! Ethereum transactions from the same sender are ordered by `nonce`.
//! Low nonces need to be included first. If there are two transactions from the same sender
//! and with the same `nonce` only one of them can be included.
//! We choose the one with higher gas price, but also require that gas price increment
//! is high enough to prevent attacking miners by requiring them to reshuffle/reexecute
//! the queue too often.
//!
//! Transactions between senders are prioritized using `gas price`. Higher `gas price`
//! yields more profits for miners. Additionally we prioritize transactions that originate
//! from our local node (own transactions).

use std::cmp;

use ethereum_types::U256;
use txpool::{self, scoring};
use super::{verifier, PrioritizationStrategy, VerifiedTransaction, ScoredTransaction};

/// Transaction with the same (sender, nonce) can be replaced only if
/// `new_gas_price > old_gas_price + old_gas_price >> SHIFT`
const GAS_PRICE_BUMP_SHIFT: usize = 3; // 2 = 25%, 3 = 12.5%, 4 = 6.25%

/// Calculate minimal gas price requirement.
#[inline]
fn bump_gas_price(old_gp: U256) -> U256 {
	old_gp.saturating_add(old_gp >> GAS_PRICE_BUMP_SHIFT)
}

/// Simple, gas-price based scoring for transactions.
///
/// NOTE: Currently penalization does not apply to new transactions that enter the pool.
/// We might want to store penalization status in some persistent state.
#[derive(Debug, Clone)]
pub struct NonceAndGasPrice(pub PrioritizationStrategy);

impl NonceAndGasPrice {
	/// Decide if the transaction should even be considered into the pool (if the pool is full).
	///
	/// Used by Verifier to quickly reject transactions that don't have any chance to get into the pool later on,
	/// and save time on more expensive checks like sender recovery, etc.
	///
	/// NOTE The method is never called for zero-gas-price transactions or local transactions
	/// (such transactions are always considered to the pool and potentially rejected later on)
	pub fn should_reject_early(&self, old: &VerifiedTransaction, new: &verifier::Transaction) -> bool {
		if old.priority().is_local() {
			return true
		}

		&old.transaction.gas_price > new.gas_price()
	}
}

impl<P> txpool::Scoring<P> for NonceAndGasPrice where P: ScoredTransaction + txpool::VerifiedTransaction {
	type Score = U256;
	type Event = ();

	fn compare(&self, old: &P, other: &P) -> cmp::Ordering {
		old.nonce().cmp(&other.nonce())
	}

	fn choose(&self, old: &P, new: &P) -> scoring::Choice {
		if old.nonce() != new.nonce() {
			return scoring::Choice::InsertNew
		}

		let old_gp = old.gas_price();
		let new_gp = new.gas_price();

		let min_required_gp = bump_gas_price(*old_gp);

		match min_required_gp.cmp(&new_gp) {
			cmp::Ordering::Greater => scoring::Choice::RejectNew,
			_ => scoring::Choice::ReplaceOld,
		}
	}

	fn update_scores(&self, txs: &[txpool::Transaction<P>], scores: &mut [U256], change: scoring::Change) {
		use self::scoring::Change;

		match change {
			Change::Culled(_) => {},
			Change::RemovedAt(_) => {}
			Change::InsertedAt(i) | Change::ReplacedAt(i) => {
				assert!(i < txs.len());
				assert!(i < scores.len());

				scores[i] = *txs[i].transaction.gas_price();
				let boost = match txs[i].priority() {
					super::Priority::Local => 15,
					super::Priority::Retracted => 10,
					super::Priority::Regular => 0,
				};
				scores[i] = scores[i] << boost;
			},
			// We are only sending an event in case of penalization.
			// So just lower the priority of all non-local transactions.
			Change::Event(_) => {
				for (score, tx) in scores.iter_mut().zip(txs) {
					// Never penalize local transactions.
					if !tx.priority().is_local() {
						*score = *score >> 3;
					}
				}
			},
		}
	}

	fn should_replace(&self, old: &P, new: &P) -> scoring::Choice {
		if old.sender() == new.sender() {
			// prefer earliest transaction
			match new.nonce().cmp(&old.nonce()) {
				cmp::Ordering::Less => scoring::Choice::ReplaceOld,
				cmp::Ordering::Greater => scoring::Choice::RejectNew,
				cmp::Ordering::Equal => self.choose(old, new),
			}
		} else if old.priority().is_local() && new.priority().is_local() {
			// accept local transactions over the limit
			scoring::Choice::InsertNew
		} else {
			let old_score = (old.priority(), old.gas_price());
			let new_score = (new.priority(), new.gas_price());
			if new_score > old_score {
				scoring::Choice::ReplaceOld
			} else {
				scoring::Choice::RejectNew
			}
	 	}
	}

	fn should_ignore_sender_limit(&self, new: &P) -> bool {
		new.priority().is_local()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::sync::Arc;
	use ethkey::{Random, Generator};
	use pool::tests::tx::{Tx, TxExt};
	use txpool::Scoring;
	use txpool::scoring::Choice::*;

	#[test]
	fn should_replace_same_sender_by_nonce() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);

		let tx1 = Tx {
			nonce: 1,
			gas_price: 1,
			..Default::default()
		};
		let tx2 = Tx {
			nonce: 2,
			gas_price: 100,
			..Default::default()
		};
		let tx3 = Tx {
			nonce: 2,
			gas_price: 110,
			..Default::default()
		};
		let tx4 = Tx {
			nonce: 2,
			gas_price: 130,
			..Default::default()
		};

		let keypair = Random.generate().unwrap();
		let txs = vec![tx1, tx2, tx3, tx4].into_iter().map(|tx| {
			tx.unsigned().sign(keypair.secret(), None).verified()
		}).collect::<Vec<_>>();

		assert_eq!(scoring.should_replace(&txs[0], &txs[1]), RejectNew);
		assert_eq!(scoring.should_replace(&txs[1], &txs[0]), ReplaceOld);

		assert_eq!(scoring.should_replace(&txs[1], &txs[2]), RejectNew);
		assert_eq!(scoring.should_replace(&txs[2], &txs[1]), RejectNew);

		assert_eq!(scoring.should_replace(&txs[1], &txs[3]), ReplaceOld);
		assert_eq!(scoring.should_replace(&txs[3], &txs[1]), RejectNew);
	}

	#[test]
	fn should_replace_different_sender_by_priority_and_gas_price() {
		// given
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let tx_regular_low_gas = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			tx.signed().verified()
		};
		let tx_regular_high_gas = {
			let tx = Tx {
				nonce: 2,
				gas_price: 10,
				..Default::default()
			};
			tx.signed().verified()
		};
		let tx_local_low_gas = {
			let tx = Tx {
				nonce: 2,
				gas_price: 1,
				..Default::default()
			};
			let mut verified_tx = tx.signed().verified();
			verified_tx.priority = ::pool::Priority::Local;
			verified_tx
		};
		let tx_local_high_gas = {
			let tx = Tx {
				nonce: 1,
				gas_price: 10,
				..Default::default()
			};
			let mut verified_tx = tx.signed().verified();
			verified_tx.priority = ::pool::Priority::Local;
			verified_tx
		};

		assert_eq!(scoring.should_replace(&tx_regular_low_gas, &tx_regular_high_gas), ReplaceOld);
		assert_eq!(scoring.should_replace(&tx_regular_high_gas, &tx_regular_low_gas), RejectNew);

		assert_eq!(scoring.should_replace(&tx_regular_high_gas, &tx_local_low_gas), ReplaceOld);
		assert_eq!(scoring.should_replace(&tx_local_low_gas, &tx_regular_high_gas), RejectNew);

		assert_eq!(scoring.should_replace(&tx_local_low_gas, &tx_local_high_gas), InsertNew);
		assert_eq!(scoring.should_replace(&tx_local_high_gas, &tx_regular_low_gas), RejectNew);
	}

	#[test]
	fn should_calculate_score_correctly() {
		// given
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let (tx1, tx2, tx3) = Tx::default().signed_triple();
		let transactions = vec![tx1, tx2, tx3].into_iter().enumerate().map(|(i, tx)| {
			let mut verified = tx.verified();
			verified.priority = match i {
				0 => ::pool::Priority::Local,
				1 => ::pool::Priority::Retracted,
				_ => ::pool::Priority::Regular,
			};
			txpool::Transaction {
				insertion_id: 0,
				transaction: Arc::new(verified),
			}
		}).collect::<Vec<_>>();
		let initial_scores = vec![U256::from(0), 0.into(), 0.into()];

		// No update required
		let mut scores = initial_scores.clone();
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::Culled(0));
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::Culled(1));
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::Culled(2));
		assert_eq!(scores, initial_scores);
		let mut scores = initial_scores.clone();
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::RemovedAt(0));
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::RemovedAt(1));
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::RemovedAt(2));
		assert_eq!(scores, initial_scores);

		// Compute score at given index
		let mut scores = initial_scores.clone();
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::InsertedAt(0));
		assert_eq!(scores, vec![32768.into(), 0.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::InsertedAt(1));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::InsertedAt(2));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 1.into()]);

		let mut scores = initial_scores.clone();
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::ReplacedAt(0));
		assert_eq!(scores, vec![32768.into(), 0.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::ReplacedAt(1));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::ReplacedAt(2));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 1.into()]);

		// Check penalization
		scoring.update_scores(&transactions, &mut *scores, scoring::Change::Event(()));
		assert_eq!(scores, vec![32768.into(), 128.into(), 0.into()]);
	}
}
