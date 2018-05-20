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
use txpool;
use super::{PrioritizationStrategy, VerifiedTransaction};

/// Transaction with the same (sender, nonce) can be replaced only if
/// `new_gas_price > old_gas_price + old_gas_price >> SHIFT`
const GAS_PRICE_BUMP_SHIFT: usize = 3; // 2 = 25%, 3 = 12.5%, 4 = 6.25%

/// Simple, gas-price based scoring for transactions.
///
/// NOTE: Currently penalization does not apply to new transactions that enter the pool.
/// We might want to store penalization status in some persistent state.
#[derive(Debug)]
pub struct NonceAndGasPrice(pub PrioritizationStrategy);

impl txpool::Scoring<VerifiedTransaction> for NonceAndGasPrice {
	type Score = U256;
	type Event = ();

	fn compare(&self, old: &VerifiedTransaction, other: &VerifiedTransaction) -> cmp::Ordering {
		old.transaction.nonce.cmp(&other.transaction.nonce)
	}

	fn choose(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> txpool::scoring::Choice {
		if old.transaction.nonce != new.transaction.nonce {
			return txpool::scoring::Choice::InsertNew
		}

		let old_gp = old.transaction.gas_price;
		let new_gp = new.transaction.gas_price;

		let min_required_gp = old_gp + (old_gp >> GAS_PRICE_BUMP_SHIFT);

		match min_required_gp.cmp(&new_gp) {
			cmp::Ordering::Greater => txpool::scoring::Choice::RejectNew,
			_ => txpool::scoring::Choice::ReplaceOld,
		}
	}

	fn update_scores(&self, txs: &[txpool::Transaction<VerifiedTransaction>], scores: &mut [U256], change: txpool::scoring::Change) {
		use self::txpool::scoring::Change;

		match change {
			Change::Culled(_) => {},
			Change::RemovedAt(_) => {}
			Change::InsertedAt(i) | Change::ReplacedAt(i) => {
				assert!(i < txs.len());
				assert!(i < scores.len());

				scores[i] = txs[i].transaction.transaction.gas_price;
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

	fn should_replace(&self, old: &VerifiedTransaction, new: &VerifiedTransaction) -> bool {
		if old.sender == new.sender {
			// prefer earliest transaction
			if new.transaction.nonce < old.transaction.nonce {
				return true
			}
		}

		// Always kick out non-local transactions in favour of local ones.
		if new.priority().is_local() && !old.priority().is_local() {
			return true;
		}
		// And never kick out local transactions in favour of external ones.
		if !new.priority().is_local() && old.priority.is_local() {
			return false;
		}

		self.choose(old, new) == txpool::scoring::Choice::ReplaceOld
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::sync::Arc;
	use pool::tests::tx::{Tx, TxExt};
	use txpool::Scoring;

	#[test]
	fn should_replace_non_local_transaction_with_local_one() {
		// given
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let tx1 = {
			let tx = Tx::default().signed().verified();
			txpool::Transaction {
				insertion_id: 0,
				transaction: Arc::new(tx),
			}
		};
		let tx2 = {
			let mut tx = Tx::default().signed().verified();
			tx.priority = ::pool::Priority::Local;
			txpool::Transaction {
				insertion_id: 0,
				transaction: Arc::new(tx),
			}
		};

		assert!(scoring.should_replace(&tx1, &tx2));
		assert!(!scoring.should_replace(&tx2, &tx1));
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
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::Culled(0));
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::Culled(1));
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::Culled(2));
		assert_eq!(scores, initial_scores);
		let mut scores = initial_scores.clone();
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::RemovedAt(0));
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::RemovedAt(1));
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::RemovedAt(2));
		assert_eq!(scores, initial_scores);

		// Compute score at given index
		let mut scores = initial_scores.clone();
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::InsertedAt(0));
		assert_eq!(scores, vec![32768.into(), 0.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::InsertedAt(1));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::InsertedAt(2));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 1.into()]);

		let mut scores = initial_scores.clone();
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::ReplacedAt(0));
		assert_eq!(scores, vec![32768.into(), 0.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::ReplacedAt(1));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 0.into()]);
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::ReplacedAt(2));
		assert_eq!(scores, vec![32768.into(), 1024.into(), 1.into()]);

		// Check penalization
		scoring.update_scores(&transactions, &mut *scores, txpool::scoring::Change::Event(()));
		assert_eq!(scores, vec![32768.into(), 128.into(), 0.into()]);
	}
}
