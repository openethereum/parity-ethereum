// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::cmp;

use ethereum_types::U256;
use txpool::{self, scoring::{Choice, Scoring}, ReplaceTransaction};
use txpool::{VerifiedTransaction as PoolVerifiedTransaction};
use super::{client, ScoredTransaction, VerifiedTransaction};

#[derive(Debug)]
pub struct NonceAndGasPriceAndReadiness<S, C> {
	scoring: S,
	client: C,
}

impl<S, C> NonceAndGasPriceAndReadiness<S, C> {
	pub fn new(scoring: S, client: C) -> Self {
		NonceAndGasPriceAndReadiness { scoring, client }
	}
}

impl<S, C> txpool::ShouldReplace<VerifiedTransaction> for NonceAndGasPriceAndReadiness<S, C>
	where S: Scoring<VerifiedTransaction>, C: client::NonceClient {

	fn should_replace(
		&mut self,
		old: &ReplaceTransaction<VerifiedTransaction>,
		new: &ReplaceTransaction<VerifiedTransaction>,
	) -> Choice {
		let both_local = old.priority().is_local() && new.priority().is_local();
		if old.sender() == new.sender() {
			// prefer earliest transaction
			match new.nonce().cmp(&old.nonce()) {
				cmp::Ordering::Equal => self.scoring.choose(&old, &new),
				_ if both_local => Choice::InsertNew,
				cmp::Ordering::Less => Choice::ReplaceOld,
				cmp::Ordering::Greater => Choice::RejectNew,
			}
		} else if both_local {
			Choice::InsertNew
		} else {
			let old_score = (old.priority(), old.gas_price());
			let new_score = (new.priority(), new.gas_price());
			if new_score > old_score {
				let state = &self.client;
				// calculate readiness based on state nonce + pooled txs from same sender
				let is_ready = |replace: &ReplaceTransaction<VerifiedTransaction>| {
					let state_nonce = state.account_nonce(replace.transaction.sender());
					let nonce =
						replace.pooled_by_sender.map_or(state_nonce, |txs| {
							txs.iter().fold(state_nonce, |nonce, tx| {
								if nonce == tx.transaction.nonce() && tx.transaction != replace.transaction.transaction {
									nonce.saturating_add(U256::from(1))
								} else {
									nonce
								}
							})
						});
					nonce == replace.transaction.transaction.nonce()
				};

				if !is_ready(new) && is_ready(old) {
					// prevent a ready transaction being replace by a non-ready transaction
					Choice::RejectNew
				} else {
					Choice::ReplaceOld
				}
			} else {
				Choice::RejectNew
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::sync::Arc;
	use ethkey::{Random, Generator, KeyPair};
	use pool::tests::tx::{Tx, TxExt};
	use pool::tests::client::TestClient;
	use pool::scoring::*;
	use pool::{PrioritizationStrategy, VerifiedTransaction};
	use txpool::scoring::Choice::*;
	use txpool::ShouldReplace;

	fn local_tx_verified(tx: Tx, keypair: &KeyPair) -> ReplaceTransaction<VerifiedTransaction> {
		let mut verified_tx = tx.unsigned().sign(keypair.secret(), None).verified();
		verified_tx.priority = ::pool::Priority::Local;
		replace_tx(verified_tx)
	}

	fn replace_tx<'a>(tx: VerifiedTransaction, ) -> ReplaceTransaction<'a, VerifiedTransaction> {
		let tx = txpool::Transaction {
			insertion_id: 0,
			transaction: Arc::new(tx),
		};
		ReplaceTransaction::new(tx, Default::default())
	}

	#[test]
	fn should_always_accept_local_transactions_unless_same_sender_and_nonce() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let mut replace = NonceAndGasPriceAndReadiness::new(scoring, TestClient::new());

		// same sender txs
		let keypair = Random.generate().unwrap();

		let same_sender_tx1 = local_tx_verified(Tx {
			nonce: 1,
			gas_price: 1,
			..Default::default()
		}, &keypair);

		let same_sender_tx2 = local_tx_verified(Tx {
			nonce: 2,
			gas_price: 100,
			..Default::default()
		}, &keypair);

		let same_sender_tx3 = local_tx_verified(Tx {
			nonce: 2,
			gas_price: 200,
			..Default::default()
		}, &keypair);

		// different sender txs
		let sender1 = Random.generate().unwrap();
		let different_sender_tx1 = local_tx_verified(Tx {
			nonce: 2,
			gas_price: 1,
			..Default::default()
		}, &sender1);

		let sender2 = Random.generate().unwrap();
		let different_sender_tx2 = local_tx_verified(Tx {
			nonce: 1,
			gas_price: 10,
			..Default::default()
		}, &sender2);

		assert_eq!(replace.should_replace(&same_sender_tx1, &same_sender_tx2), InsertNew);
		assert_eq!(replace.should_replace(&same_sender_tx2, &same_sender_tx1), InsertNew);

		assert_eq!(replace.should_replace(&different_sender_tx1, &different_sender_tx2), InsertNew);
		assert_eq!(replace.should_replace(&different_sender_tx2, &different_sender_tx1), InsertNew);

		// txs with same sender and nonce
		assert_eq!(replace.should_replace(&same_sender_tx2, &same_sender_tx3), ReplaceOld);
		assert_eq!(replace.should_replace(&same_sender_tx3, &same_sender_tx2), RejectNew);
	}

	#[test]
	fn should_replace_same_sender_by_nonce() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let mut replace = NonceAndGasPriceAndReadiness::new(scoring, TestClient::new());

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
			replace_tx(tx.unsigned().sign(keypair.secret(), None).verified())
		}).collect::<Vec<_>>();

		assert_eq!(replace.should_replace(&txs[0], &txs[1]), RejectNew);
		assert_eq!(replace.should_replace(&txs[1], &txs[0]), ReplaceOld);

		assert_eq!(replace.should_replace(&txs[1], &txs[2]), RejectNew);
		assert_eq!(replace.should_replace(&txs[2], &txs[1]), RejectNew);

		assert_eq!(replace.should_replace(&txs[1], &txs[3]), ReplaceOld);
		assert_eq!(replace.should_replace(&txs[3], &txs[1]), RejectNew);
	}

	#[test]
	fn should_replace_different_sender_by_priority_and_gas_price() {
		// given
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let mut replace = NonceAndGasPriceAndReadiness::new(scoring, TestClient::new());

		let tx_regular_low_gas = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			replace_tx(tx.signed().verified())
		};
		let tx_regular_high_gas = {
			let tx = Tx {
				nonce: 2,
				gas_price: 10,
				..Default::default()
			};
			replace_tx(tx.signed().verified())
		};
		let tx_local_low_gas = {
			let tx = Tx {
				nonce: 2,
				gas_price: 1,
				..Default::default()
			};
			let mut verified_tx = tx.signed().verified();
			verified_tx.priority = ::pool::Priority::Local;
			replace_tx(verified_tx)
		};
		let tx_local_high_gas = {
			let tx = Tx {
				nonce: 1,
				gas_price: 10,
				..Default::default()
			};
			let mut verified_tx = tx.signed().verified();
			verified_tx.priority = ::pool::Priority::Local;
			replace_tx(verified_tx)
		};

		assert_eq!(replace.should_replace(&tx_regular_low_gas, &tx_regular_high_gas), ReplaceOld);
		assert_eq!(replace.should_replace(&tx_regular_high_gas, &tx_regular_low_gas), RejectNew);

		assert_eq!(replace.should_replace(&tx_regular_high_gas, &tx_local_low_gas), ReplaceOld);
		assert_eq!(replace.should_replace(&tx_local_low_gas, &tx_regular_high_gas), RejectNew);

		assert_eq!(replace.should_replace(&tx_local_low_gas, &tx_local_high_gas), InsertNew);
		assert_eq!(replace.should_replace(&tx_local_high_gas, &tx_regular_low_gas), RejectNew);
	}

//	#[test]
//	fn should_not_replace_ready_transaction_with_future_transaction() {
//		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
//		let tx_ready_low_score = {
//			let tx = Tx {
//				nonce: 1,
//				gas_price: 1,
//				..Default::default()
//			};
//			tx.signed().verified()
//		};
//		let tx_future_high_score = {
//			let tx = Tx {
//				nonce: 3, // future nonce
//				gas_price: 10,
//				..Default::default()
//			};
//			tx.signed().verified()
//		};
//
//		assert_eq!(should_replace(&scoring, &tx_ready_low_score, &tx_future_high_score), RejectNew);
//	}
}
