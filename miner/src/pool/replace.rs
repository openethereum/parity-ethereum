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

//! Replacing Transactions
//!
//! When queue limits are reached, a new transaction may replace one already
//! in the pool. The decision whether to reject, replace or retain both is
//! delegated to an implementation of `ShouldReplace`.
//!
//! Here we decide based on the sender, the nonce and gas price, and finally
//! on the `Readiness` of the transactions when comparing them

use std::cmp;

use ethereum_types::{U256, H160 as Address};
use txpool::{self, scoring::{Choice, Scoring}, ReplaceTransaction};
use txpool::VerifiedTransaction;
use super::{client, ScoredTransaction};

/// Choose whether to replace based on the sender, the score and finally the
/// `Readiness` of the transactions being compared.
#[derive(Debug)]
pub struct ReplaceByScoreAndReadiness<S, C> {
	scoring: S,
	client: C,
}

impl<S, C> ReplaceByScoreAndReadiness<S, C> {
	/// Create a new `ReplaceByScoreAndReadiness`
	pub fn new(scoring: S, client: C) -> Self {
		ReplaceByScoreAndReadiness { scoring, client }
	}
}

impl<T, S, C> txpool::ShouldReplace<T> for ReplaceByScoreAndReadiness<S, C>
where
	T: VerifiedTransaction<Sender = Address> + ScoredTransaction + PartialEq,
	S: Scoring<T>,
	C: client::NonceClient,
{
	fn should_replace(
		&self,
		old: &ReplaceTransaction<T>,
		new: &ReplaceTransaction<T>,
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
				// Check if this is a replacement transaction.
				//
				// With replacement transactions we can safely return `InsertNew` here, because
				// we don't need to remove `old` (worst transaction in the pool) since `new` will replace
			    // some other transaction in the pool so we will never go above limit anyway.
				if let Some(txs) = new.pooled_by_sender {
					if let Ok(index) = txs.binary_search_by(|old| self.scoring.compare(old, new)) {
						return match self.scoring.choose(&txs[index], new) {
							Choice::ReplaceOld => Choice::InsertNew,
							choice => choice,
						}
					}
				}

				let state = &self.client;
				// calculate readiness based on state nonce + pooled txs from same sender
				let is_ready = |replace: &ReplaceTransaction<T>| {
					let mut nonce = state.account_nonce(replace.sender());
					if let Some(txs) = replace.pooled_by_sender {
						for tx in txs.iter() {
							if nonce == tx.nonce() && *tx.transaction != ***replace.transaction {
								nonce = nonce.saturating_add(U256::from(1))
							} else {
								break
							}
						}
					}
					nonce == replace.nonce()
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

	fn local_tx_verified(tx: Tx, keypair: &KeyPair) -> VerifiedTransaction {
		let mut verified_tx = tx.unsigned().sign(keypair.secret(), None).verified();
		verified_tx.priority = ::pool::Priority::Local;
		verified_tx
	}

	fn should_replace(replace: &ShouldReplace<VerifiedTransaction>, old: VerifiedTransaction, new: VerifiedTransaction) -> Choice {
		let old_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(old) };
		let new_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(new) };
		let old = ReplaceTransaction::new(&old_tx, Default::default());
		let new = ReplaceTransaction::new(&new_tx, Default::default());
		replace.should_replace(&old, &new)
	}

	#[test]
	fn should_always_accept_local_transactions_unless_same_sender_and_nonce() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(1);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

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

		assert_eq!(should_replace(&replace, same_sender_tx1.clone(), same_sender_tx2.clone()), InsertNew);
		assert_eq!(should_replace(&replace, same_sender_tx2.clone(), same_sender_tx1.clone()), InsertNew);

		assert_eq!(should_replace(&replace, different_sender_tx1.clone(), different_sender_tx2.clone()), InsertNew);
		assert_eq!(should_replace(&replace, different_sender_tx2.clone(), different_sender_tx1.clone()), InsertNew);

		// txs with same sender and nonce
		assert_eq!(should_replace(&replace, same_sender_tx2.clone(), same_sender_tx3.clone()), ReplaceOld);
		assert_eq!(should_replace(&replace, same_sender_tx3.clone(), same_sender_tx2.clone()), RejectNew);
	}

	#[test]
	fn should_replace_same_sender_by_nonce() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(1);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

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

		assert_eq!(should_replace(&replace, txs[0].clone(), txs[1].clone()), RejectNew);
		assert_eq!(should_replace(&replace, txs[1].clone(), txs[0].clone()), ReplaceOld);

		assert_eq!(should_replace(&replace, txs[1].clone(), txs[2].clone()), RejectNew);
		assert_eq!(should_replace(&replace, txs[2].clone(), txs[1].clone()), RejectNew);

		assert_eq!(should_replace(&replace, txs[1].clone(), txs[3].clone()), ReplaceOld);
		assert_eq!(should_replace(&replace, txs[3].clone(), txs[1].clone()), RejectNew);
	}

	#[test]
	fn should_replace_different_sender_by_priority_and_gas_price() {
		// given
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(0);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

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

		assert_eq!(should_replace(&replace, tx_regular_low_gas.clone(), tx_regular_high_gas.clone()), ReplaceOld);
		assert_eq!(should_replace(&replace, tx_regular_high_gas.clone(), tx_regular_low_gas.clone()), RejectNew);

		assert_eq!(should_replace(&replace, tx_regular_high_gas.clone(), tx_local_low_gas.clone()), ReplaceOld);
		assert_eq!(should_replace(&replace, tx_local_low_gas.clone(), tx_regular_high_gas.clone()), RejectNew);

		assert_eq!(should_replace(&replace, tx_local_low_gas.clone(), tx_local_high_gas.clone()), InsertNew);
		assert_eq!(should_replace(&replace, tx_local_high_gas.clone(), tx_regular_low_gas.clone()), RejectNew);
	}

	#[test]
	fn should_not_replace_ready_transaction_with_future_transaction() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(1);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

		let tx_ready_low_score = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			tx.signed().verified()
		};
		let tx_future_high_score = {
			let tx = Tx {
				nonce: 3, // future nonce
				gas_price: 10,
				..Default::default()
			};
			tx.signed().verified()
		};

		assert_eq!(should_replace(&replace, tx_ready_low_score, tx_future_high_score), RejectNew);
	}

	#[test]
	fn should_compute_readiness_with_pooled_transactions_from_the_same_sender_as_the_existing_transaction() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(1);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

		let old_sender = Random.generate().unwrap();
		let tx_old_ready_1 = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			tx.unsigned().sign(&old_sender.secret(), None).verified()
		};
		let tx_old_ready_2 = {
			let tx = Tx {
				nonce: 2,
				gas_price: 1,
				..Default::default()
			};
			tx.unsigned().sign(&old_sender.secret(), None).verified()
		};
		let tx_old_ready_3 = {
			let tx = Tx {
				nonce: 3,
				gas_price: 1,
				..Default::default()
			};
			tx.unsigned().sign(&old_sender.secret(), None).verified()
		};

		let new_tx = {
			let tx = Tx {
				nonce: 3, // future nonce
				gas_price: 10,
				..Default::default()
			};
			tx.signed().verified()
		};

		let old_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_old_ready_3) };
		let pooled_txs = [
			txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_old_ready_1) },
			txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_old_ready_2) },
		];

		let new_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(new_tx) };

		let old = ReplaceTransaction::new(&old_tx, Some(&pooled_txs));
		let new = ReplaceTransaction::new(&new_tx, Default::default());

		assert_eq!(replace.should_replace(&old, &new), RejectNew);
	}

	#[test]
	fn should_compute_readiness_with_pooled_transactions_from_the_same_sender_as_the_new_transaction() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(1);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

		// current transaction is ready but has a lower gas price than the new one
		let old_tx = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			tx.signed().verified()
		};

		let new_sender = Random.generate().unwrap();
		let tx_new_ready_1 = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			tx.unsigned().sign(&new_sender.secret(), None).verified()
		};
		let tx_new_ready_2 = {
			let tx = Tx {
				nonce: 2,
				gas_price: 1,
				..Default::default()
			};
			tx.unsigned().sign(&new_sender.secret(), None).verified()
		};
		let tx_new_ready_3 = {
			let tx = Tx {
				nonce: 3,
				gas_price: 10, // hi
				..Default::default()
			};
			tx.unsigned().sign(&new_sender.secret(), None).verified()
		};

		let old_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(old_tx) };

		let new_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_new_ready_3) };
		let pooled_txs = [
			txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_new_ready_1) },
			txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_new_ready_2) },
		];

		let old = ReplaceTransaction::new(&old_tx, None);
		let new = ReplaceTransaction::new(&new_tx, Some(&pooled_txs));

		assert_eq!(replace.should_replace(&old, &new), ReplaceOld);
	}

	#[test]
	fn should_accept_local_tx_with_same_sender_and_nonce_with_better_gas_price() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(1);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

		// current transaction is ready
		let old_tx = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			tx.signed().verified()
		};

		let new_sender = Random.generate().unwrap();
		let tx_new_ready_1 = local_tx_verified(Tx {
			nonce: 1,
			gas_price: 1,
			..Default::default()
		}, &new_sender);

		let tx_new_ready_2 = local_tx_verified(Tx {
			nonce: 1,
			gas_price: 2, // same nonce, higher gas price
			..Default::default()
		}, &new_sender);

		let old_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(old_tx) };

		let new_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_new_ready_2) };
		let pooled_txs = [
			txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_new_ready_1) },
		];

		let old = ReplaceTransaction::new(&old_tx, None);
		let new = ReplaceTransaction::new(&new_tx, Some(&pooled_txs));

		assert_eq!(replace.should_replace(&old, &new), InsertNew);
	}

	#[test]
	fn should_reject_local_tx_with_same_sender_and_nonce_with_worse_gas_price() {
		let scoring = NonceAndGasPrice(PrioritizationStrategy::GasPriceOnly);
		let client = TestClient::new().with_nonce(1);
		let replace = ReplaceByScoreAndReadiness::new(scoring, client);

		// current transaction is ready
		let old_tx = {
			let tx = Tx {
				nonce: 1,
				gas_price: 1,
				..Default::default()
			};
			tx.signed().verified()
		};

		let new_sender = Random.generate().unwrap();
		let tx_new_ready_1 = local_tx_verified(Tx {
			nonce: 1,
			gas_price: 2,
			..Default::default()
		}, &new_sender);

		let tx_new_ready_2 = local_tx_verified(Tx {
			nonce: 1,
			gas_price: 1, // same nonce, lower gas price
			..Default::default()
		}, &new_sender);

		let old_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(old_tx) };

		let new_tx = txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_new_ready_2) };
		let pooled_txs = [
			txpool::Transaction { insertion_id: 0, transaction: Arc::new(tx_new_ready_1) },
		];

		let old = ReplaceTransaction::new(&old_tx, None);
		let new = ReplaceTransaction::new(&new_tx, Some(&pooled_txs));

		assert_eq!(replace.should_replace(&old, &new), RejectNew);
	}
}
