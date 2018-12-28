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

use ethereum_types::U256;
use transaction::{self, PendingTransaction};
use txpool;

use pool::{verifier, TransactionQueue, PrioritizationStrategy, PendingSettings, PendingOrdering};

pub mod tx;
pub mod client;

use self::tx::{Tx, TxExt, PairExt};
use self::client::TestClient;

// max mem for 3 transaction, this is relative
// to the global use allocator, the value is currently
// set to reflect malloc usage.
// 50 was enough when using jmalloc.
const TEST_QUEUE_MAX_MEM: usize = 80;

fn new_queue() -> TransactionQueue {
	TransactionQueue::new(
		txpool::Options {
			max_count: 3,
			max_per_sender: 3,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	)
}
#[test]
fn should_return_correct_nonces_when_dropped_because_of_limit() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 3,
			max_per_sender: 1,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	let (tx1, tx2) = Tx::gas_price(2).signed_pair();
	let sender = tx1.sender();
	let nonce = tx1.nonce;

	// when
	let r1 = txq.import(TestClient::new(), vec![tx1].retracted());
	let r2 = txq.import(TestClient::new(), vec![tx2].retracted());
	assert_eq!(r1, vec![Ok(())]);
	assert_eq!(r2, vec![Err(transaction::Error::LimitReached)]);
	assert_eq!(txq.status().status.transaction_count, 1);

	// then
	assert_eq!(txq.next_nonce(TestClient::new(), &sender), Some(nonce + 1));

	// when
	let tx1 = Tx::gas_price(2).signed();
	let tx2 = Tx::gas_price(2).signed();
	let sender = tx2.sender();
	let tx3 = Tx::gas_price(1).signed();
	let tx4 = Tx::gas_price(3).signed();
	let res = txq.import(TestClient::new(), vec![tx1, tx2].retracted());
	let res2 = txq.import(TestClient::new(), vec![tx3, tx4].retracted());

	// then
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(res2, vec![
			   // The error here indicates reaching the limit
			   // and minimal effective gas price taken into account.
			   Err(transaction::Error::InsufficientGasPrice { minimal: 2.into(), got: 1.into() }),
			   Ok(())
	]);
	assert_eq!(txq.status().status.transaction_count, 3);
	// tx2 transacton got dropped because of limit
	// tx1 and tx1' are kept, because they have lower insertion_ids so they are preferred.
	assert_eq!(txq.next_nonce(TestClient::new(), &sender), None);
}

#[test]
fn should_never_drop_local_transactions_from_different_senders() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 3,
			max_per_sender: 1,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	let (tx1, tx2) = Tx::gas_price(2).signed_pair();
	let sender = tx1.sender();
	let nonce = tx1.nonce;

	// when
	let r1 = txq.import(TestClient::new(), vec![tx1].local());
	let r2 = txq.import(TestClient::new(), vec![tx2].local());
	assert_eq!(r1, vec![Ok(())]);
	assert_eq!(r2, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);

	// then
	assert_eq!(txq.next_nonce(TestClient::new(), &sender), Some(nonce + 2));

	// when
	let tx1 = Tx::gas_price(2).signed();
	let tx2 = Tx::gas_price(2).signed();
	let tx3 = Tx::gas_price(1).signed();
	let tx4 = Tx::gas_price(3).signed();
	let res = txq.import(TestClient::new(), vec![tx1, tx2].local());
	let res2 = txq.import(TestClient::new(), vec![tx3, tx4].local());

	// then
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(res2, vec![Ok(()), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 6);
	assert_eq!(txq.next_nonce(TestClient::new(), &sender), Some(nonce + 2));
}

#[test]
fn should_handle_same_transaction_imported_twice_with_different_state_nonces() {
	// given
	let txq = new_queue();
	let (tx, tx2) = Tx::default().signed_replacement();
	let hash = tx2.hash();
	let client = TestClient::new().with_nonce(122);

	// First insert one transaction to future
	let res = txq.import(client.clone(), vec![tx].local());
	assert_eq!(res, vec![Ok(())]);
	// next_nonce === None -> transaction is in future
	assert_eq!(txq.next_nonce(client.clone(), &tx2.sender()), None);

	// now import second transaction to current
	let res = txq.import(TestClient::new(), vec![tx2.local()]);

	// and then there should be only one transaction in current (the one with higher gas_price)
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(top[0].hash, hash);
}

#[test]
fn should_move_all_transactions_from_future() {
	// given
	let txq = new_queue();
	let txs = Tx::default().signed_pair();
	let (hash, hash2) = txs.hash();
	let (tx, tx2) = txs;
	let client = TestClient::new().with_nonce(122);

	// First insert one transaction to future
	let res = txq.import(client.clone(), vec![tx.local()]);
	assert_eq!(res, vec![Ok(())]);
	// next_nonce === None -> transaction is in future
	assert_eq!(txq.next_nonce(client.clone(), &tx2.sender()), None);

	// now import second transaction to current
	let res = txq.import(client.clone(), vec![tx2.local()]);

	// then
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(top[0].hash, hash);
	assert_eq!(top[1].hash, hash2);
}

#[test]
fn should_drop_transactions_from_senders_without_balance() {
	// given
	let txq = new_queue();
	let tx = Tx::default().signed();
	let client = TestClient::new().with_balance(1);

	// when
	let res = txq.import(client, vec![tx.local()]);

	// then
	assert_eq!(res, vec![Err(transaction::Error::InsufficientBalance {
		balance: U256::from(1),
		cost: U256::from(21_100),
	})]);
	assert_eq!(txq.status().status.transaction_count, 0);
}

#[test]
fn should_not_import_transaction_below_min_gas_price_threshold_if_external() {
	// given
	let txq = new_queue();
	let tx = Tx::default();
	txq.set_verifier_options(verifier::Options {
		minimal_gas_price: 3.into(),
		..Default::default()
	});

	// when
	let res = txq.import(TestClient::new(), vec![tx.signed().unverified()]);

	// then
	assert_eq!(res, vec![Err(transaction::Error::InsufficientGasPrice {
		minimal: U256::from(3),
		got: U256::from(1),
	})]);
	assert_eq!(txq.status().status.transaction_count, 0);
}

#[test]
fn should_import_transaction_below_min_gas_price_threshold_if_local() {
	// given
	let txq = new_queue();
	let tx = Tx::default();
	txq.set_verifier_options(verifier::Options {
		minimal_gas_price: 3.into(),
		..Default::default()
	});

	// when
	let res = txq.import(TestClient::new(), vec![tx.signed().local()]);

	// then
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);
}

#[test]
fn should_import_txs_from_same_sender() {
	// given
	let txq = new_queue();

	let txs = Tx::default().signed_pair();
	let (hash, hash2) = txs.hash();

	// when
	txq.import(TestClient::new(), txs.local().into_vec());

	// then
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0 ,0));
	assert_eq!(top[0].hash, hash);
	assert_eq!(top[1].hash, hash2);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_prioritize_local_transactions_within_same_nonce_height() {
	// given
	let txq = new_queue();
	let tx = Tx::default().signed();
	// the second one has same nonce but higher `gas_price`
	let tx2 = Tx::gas_price(2).signed();
	let (hash, hash2) = (tx.hash(), tx2.hash());
	let client = TestClient::new().with_local(&tx.sender());

	// when
	// first insert the one with higher gas price
	let res = txq.import(client.clone(), vec![tx.local(), tx2.unverified()]);
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// then
	let top = txq.pending(client, PendingSettings::all_prioritized(0, 0));
	assert_eq!(top[0].hash, hash); // local should be first
	assert_eq!(top[1].hash, hash2);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_prioritize_reimported_transactions_within_same_nonce_height() {
	// given
	let txq = new_queue();
	let tx = Tx::default().signed();
	// the second one has same nonce but higher `gas_price`
	let tx2 = Tx::gas_price(2).signed();
	let (hash, hash2) = (tx.hash(), tx2.hash());

	// when
	// first insert local one with higher gas price
	// then the one with lower gas price, but from retracted block
	let res = txq.import(TestClient::new(), vec![tx2.unverified(), tx.retracted()]);
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// then
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(top[0].hash, hash); // retracted should be first
	assert_eq!(top[1].hash, hash2);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_not_prioritize_local_transactions_with_different_nonce_height() {
	// given
	let txq = new_queue();
	let txs = Tx::default().signed_pair();
	let (hash, hash2) = txs.hash();
	let (tx, tx2) = txs;

	// when
	let res = txq.import(TestClient::new(), vec![tx.unverified(), tx2.local()]);
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// then
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(top[0].hash, hash);
	assert_eq!(top[1].hash, hash2);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_put_transaction_to_futures_if_gap_detected() {
	// given
	let txq = new_queue();
	let (tx, _, tx2) = Tx::default().signed_triple();
	let hash = tx.hash();

	// when
	let res = txq.import(TestClient::new(), vec![tx, tx2].local());

	// then
	assert_eq!(res, vec![Ok(()), Ok(())]);
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(top.len(), 1);
	assert_eq!(top[0].hash, hash);
}

#[test]
fn should_handle_min_block() {
	// given
	let txq = new_queue();

	let (tx, tx2) = Tx::default().signed_pair();

	// when
	let res = txq.import(TestClient::new(), vec![
		verifier::Transaction::Local(PendingTransaction::new(tx, transaction::Condition::Number(1).into())),
		tx2.local()
	]);
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// then
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(top.len(), 0);
	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(1, 0));
	assert_eq!(top.len(), 2);
}

#[test]
fn should_correctly_update_futures_when_removing() {
	// given
	let txq = new_queue();
	let txs= Tx::default().signed_pair();

	let res = txq.import(TestClient::new().with_nonce(121), txs.local().into_vec());
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);

	// when
	txq.cull(TestClient::new().with_nonce(125));
	// should remove both transactions since they are stalled

	// then
	assert_eq!(txq.status().status.transaction_count, 0);
}

#[test]
fn should_move_transactions_if_gap_filled() {
	// given
	let txq = new_queue();
	let (tx, tx1, tx2) = Tx::default().signed_triple();

	let res = txq.import(TestClient::new(), vec![tx, tx2].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 1);

	// when
	let res = txq.import(TestClient::new(), vec![tx1.local()]);
	assert_eq!(res, vec![Ok(())]);

	// then
	assert_eq!(txq.status().status.transaction_count, 3);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 3);
}

#[test]
fn should_remove_transaction() {
	// given
	let txq = new_queue();
	let (tx, _, tx2) = Tx::default().signed_triple();

	let res = txq.import(TestClient::default(), vec![tx, tx2].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 1);

	// when
	txq.cull(TestClient::new().with_nonce(124));
	assert_eq!(txq.status().status.transaction_count, 1);
	assert_eq!(txq.pending(TestClient::new().with_nonce(125), PendingSettings::all_prioritized(0, 0)).len(), 1);
	txq.cull(TestClient::new().with_nonce(126));

	// then
	assert_eq!(txq.status().status.transaction_count, 0);
}

#[test]
fn should_move_transactions_to_future_if_gap_introduced() {
	// given
	let txq = new_queue();
	let (tx, tx2) = Tx::default().signed_pair();
	let hash = tx.hash();
	let tx3 = Tx::default().signed();

	let res = txq.import(TestClient::new(), vec![tx3, tx2].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 1);

	let res = txq.import(TestClient::new(), vec![tx].local());
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 3);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 3);

	// when
	txq.remove(vec![&hash], true);

	// then
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 1);
}

#[test]
fn should_clear_queue() {
	// given
	let txq = new_queue();
	let txs = Tx::default().signed_pair();

	// add
	txq.import(TestClient::new(), txs.local().into_vec());
	assert_eq!(txq.status().status.transaction_count, 2);

	// when
	txq.clear();

	// then
	assert_eq!(txq.status().status.transaction_count, 0);
}

#[test]
fn should_prefer_current_transactions_when_hitting_the_limit() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 1,
			max_per_sender: 2,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	let (tx, tx2) = Tx::default().signed_pair();
	let hash = tx.hash();
	let sender = tx.sender();

	let res = txq.import(TestClient::new(), vec![tx2.unverified()]);
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);

	// when
	let res = txq.import(TestClient::new(), vec![tx.unverified()]);

	// then
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);

	let top = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(top.len(), 1);
	assert_eq!(top[0].hash, hash);
	assert_eq!(txq.next_nonce(TestClient::new(), &sender), Some(124.into()));
}

#[test]
fn should_drop_transactions_with_old_nonces() {
	let txq = new_queue();
	let tx = Tx::default().signed();

	// when
	let res = txq.import(TestClient::new().with_nonce(125), vec![tx.unverified()]);

	// then
	assert_eq!(res, vec![Err(transaction::Error::Old)]);
	assert_eq!(txq.status().status.transaction_count, 0);
}

#[test]
fn should_not_insert_same_transaction_twice() {
	// given
	let txq = new_queue();
	let (_tx1, tx2) = Tx::default().signed_pair();
	let res = txq.import(TestClient::new(), vec![tx2.clone().local()]);
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);

	// when
	let res = txq.import(TestClient::new(), vec![tx2.local()]);

	// then
	assert_eq!(res, vec![Err(transaction::Error::AlreadyImported)]);
	assert_eq!(txq.status().status.transaction_count, 1);
}

#[test]
fn should_accept_same_transaction_twice_if_removed() {
	// given
	let txq = new_queue();
	let txs = Tx::default().signed_pair();
	let (tx1, _) = txs.clone();
	let (hash, _) = txs.hash();

	let res = txq.import(TestClient::new(), txs.local().into_vec());
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 2);

	// when
	txq.remove(vec![&hash], true);
	assert_eq!(txq.status().status.transaction_count, 1);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 0);

	let res = txq.import(TestClient::new(), vec![tx1].local());
	assert_eq!(res, vec![Ok(())]);

	// then
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 2);
}

#[test]
fn should_not_replace_same_transaction_if_the_fee_is_less_than_minimal_bump() {
	// given
	let txq = new_queue();
	let (tx, tx2) = Tx::gas_price(20).signed_replacement();
	let (tx3, tx4) = Tx::gas_price(1).signed_replacement();
	let client = TestClient::new().with_balance(1_000_000);

	// when
	let res = txq.import(client.clone(), vec![tx, tx3].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);

	let res = txq.import(client.clone(), vec![tx2, tx4].local());

	// then
	assert_eq!(res, vec![Err(transaction::Error::TooCheapToReplace), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(client.clone(), PendingSettings::all_prioritized(0, 0))[0].signed().gas_price, U256::from(20));
	assert_eq!(txq.pending(client.clone(), PendingSettings::all_prioritized(0, 0))[1].signed().gas_price, U256::from(2));
}

#[test]
fn should_return_none_when_transaction_from_given_address_does_not_exist() {
	// given
	let txq = new_queue();

	// then
	assert_eq!(txq.next_nonce(TestClient::new(), &Default::default()), None);
}

#[test]
fn should_return_correct_nonce_when_transactions_from_given_address_exist() {
	// given
	let txq = new_queue();
	let tx = Tx::default().signed();
	let from = tx.sender();
	let nonce = tx.nonce;

	// when
	txq.import(TestClient::new(), vec![tx.local()]);

	// then
	assert_eq!(txq.next_nonce(TestClient::new(), &from), Some(nonce + 1 ));
}

#[test]
fn should_return_valid_last_nonce_after_cull() {
	// given
	let txq = new_queue();
	let (tx1, _, tx2) = Tx::default().signed_triple();
	let sender = tx1.sender();

	// when
	// Second should go to future
	let res = txq.import(TestClient::new(), vec![tx1, tx2].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);
	// Now block is imported
	let client = TestClient::new().with_nonce(124);
	txq.cull(client.clone());
	// tx2 should be not be promoted to current
	assert_eq!(txq.pending(client.clone(), PendingSettings::all_prioritized(0, 0)).len(), 0);

	// then
	assert_eq!(txq.next_nonce(client.clone(), &sender), None);
	assert_eq!(txq.next_nonce(client.with_nonce(125), &sender), Some(126.into()));
}

#[test]
fn should_return_true_if_there_is_local_transaction_pending() {
	// given
	let txq = new_queue();
	let (tx1, tx2) = Tx::default().signed_pair();
	assert_eq!(txq.has_local_pending_transactions(), false);
	let client = TestClient::new().with_local(&tx1.sender());

	// when
	let res = txq.import(client.clone(), vec![tx1.unverified(), tx2.local()]);
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// then
	assert_eq!(txq.has_local_pending_transactions(), true);
}

#[test]
fn should_reject_transactions_below_base_gas() {
	// given
	let txq = new_queue();
	let tx = Tx::default().signed();

	// when
	let res = txq.import(TestClient::new().with_gas_required(100_001), vec![tx].local());

	// then
	assert_eq!(res, vec![Err(transaction::Error::InsufficientGas {
		minimal: 100_001.into(),
		got: 21_000.into(),
	})]);
}

#[test]
fn should_remove_out_of_date_transactions_occupying_queue() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 105,
			max_per_sender: 3,
			max_mem_usage: 5_000_000,
		},
		verifier::Options {
			minimal_gas_price: 10.into(),
			..Default::default()
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	// that transaction will be occupying the queue
	let (_, tx) = Tx::default().signed_pair();
	let res = txq.import(TestClient::new(), vec![tx.local()]);
	assert_eq!(res, vec![Ok(())]);
	// This should not clear the transaction (yet)
	txq.cull(TestClient::new());
	assert_eq!(txq.status().status.transaction_count, 1);

	// Now insert at least 100 transactions to have the other one marked as future.
	for _ in 0..34 {
		let (tx1, tx2, tx3) = Tx::default().signed_triple();
		txq.import(TestClient::new(), vec![tx1, tx2, tx3].local());
	}
	assert_eq!(txq.status().status.transaction_count, 103);

	// when
	txq.cull(TestClient::new());

	// then
	assert_eq!(txq.status().status.transaction_count, 102);
}

#[test]
fn should_accept_local_transactions_below_min_gas_price() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 3,
			max_per_sender: 3,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 10.into(),
			..Default::default()
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	let tx = Tx::gas_price(1).signed();

	// when
	let res = txq.import(TestClient::new(), vec![tx.local()]);
	assert_eq!(res, vec![Ok(())]);

	// then
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 1);
}

#[test]
fn should_accept_local_service_transaction() {
	// given
	let txq = new_queue();
	let tx = Tx::gas_price(0).signed();

	// when
	let res = txq.import(
		TestClient::new()
			.with_local(&tx.sender()),
		vec![tx.local()]
	);
	assert_eq!(res, vec![Ok(())]);

	// then
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0)).len(), 1);
}

#[test]
fn should_not_accept_external_service_transaction_if_sender_not_certified() {
	// given
	let txq = new_queue();
	let tx1 = Tx::gas_price(0).signed().unverified();
	let tx2 = Tx::gas_price(0).signed().retracted();
	let tx3 = Tx::gas_price(0).signed().unverified();

	// when
	let res = txq.import(TestClient::new(), vec![tx1, tx2]);
	assert_eq!(res, vec![
		Err(transaction::Error::InsufficientGasPrice {
			minimal: 1.into(),
			got: 0.into(),
		}),
		Err(transaction::Error::InsufficientGasPrice {
			minimal: 1.into(),
			got: 0.into(),
		}),
	]);

	// then
	let res = txq.import(TestClient::new().with_service_transaction(), vec![tx3]);
	assert_eq!(res, vec![Ok(())]);
}

#[test]
fn should_not_return_transactions_over_nonce_cap() {
	// given
	let txq = new_queue();
	let (tx1, tx2, tx3) = Tx::default().signed_triple();
	let res = txq.import(
		TestClient::new(),
		vec![tx1, tx2, tx3].local()
	);
	assert_eq!(res, vec![Ok(()), Ok(()), Ok(())]);

	// when
	let all = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	// This should invalidate the cache!
	let limited = txq.pending(TestClient::new(), PendingSettings {
		block_number: 0,
		current_timestamp: 0,
		nonce_cap: Some(123.into()),
		max_len: usize::max_value(),
		ordering: PendingOrdering::Priority,
	});

	// then
	assert_eq!(all.len(), 3);
	assert_eq!(limited.len(), 1);
}

#[test]
fn should_return_cached_pending_even_if_unordered_is_requested() {
	// given
	let txq = new_queue();
	let tx1 = Tx::default().signed();
	let (tx2_1, tx2_2)= Tx::default().signed_pair();
	let tx2_1_hash = tx2_1.hash();
	let res = txq.import(TestClient::new(), vec![tx1].unverified());
	assert_eq!(res, vec![Ok(())]);
	let res = txq.import(TestClient::new(), vec![tx2_1, tx2_2].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// when
	let all = txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 0));
	assert_eq!(all[0].hash, tx2_1_hash);
	assert_eq!(all.len(), 3);

	// This should not invalidate the cache!
	let limited = txq.pending(TestClient::new(), PendingSettings {
		block_number: 0,
		current_timestamp: 0,
		nonce_cap: None,
		max_len: 3,
		ordering: PendingOrdering::Unordered,
	});

	// then
	assert_eq!(all, limited);
}

#[test]
fn should_return_unordered_and_not_populate_the_cache() {
	// given
	let txq = new_queue();
	let tx1 = Tx::default().signed();
	let (tx2_1, tx2_2)= Tx::default().signed_pair();
	let res = txq.import(TestClient::new(), vec![tx1].unverified());
	assert_eq!(res, vec![Ok(())]);
	let res = txq.import(TestClient::new(), vec![tx2_1, tx2_2].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// when
	// This should not invalidate the cache!
	let limited = txq.pending(TestClient::new(), PendingSettings {
		block_number: 0,
		current_timestamp: 0,
		nonce_cap: None,
		max_len: usize::max_value(),
		ordering: PendingOrdering::Unordered,
	});

	// then
	assert_eq!(limited.len(), 3);
	assert!(!txq.is_pending_cached());
}

#[test]
fn should_clear_cache_after_timeout_for_local() {
	// given
	let txq = new_queue();
	let (tx, tx2) = Tx::default().signed_pair();
	let res = txq.import(TestClient::new(), vec![
		verifier::Transaction::Local(PendingTransaction::new(tx, transaction::Condition::Timestamp(1000).into())),
		tx2.local()
	]);
	assert_eq!(res, vec![Ok(()), Ok(())]);

	// This should populate cache and set timestamp to 1
	// when
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 1)).len(), 0);
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 1000)).len(), 0);

	// This should invalidate the cache and trigger transaction ready.
	// then
	assert_eq!(txq.pending(TestClient::new(), PendingSettings::all_prioritized(0, 1002)).len(), 2);
}

#[test]
fn should_reject_big_transaction() {
	let txq = new_queue();
	let big_tx = Tx::default().big_one();
	let res = txq.import(TestClient::new(), vec![
		verifier::Transaction::Local(PendingTransaction::new(big_tx, transaction::Condition::Timestamp(1000).into()))
	]);
	assert_eq!(res, vec![Err(transaction::Error::TooBig)]);
}

#[test]
fn should_include_local_transaction_to_a_full_pool() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 1,
			max_per_sender: 2,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	let tx1 = Tx::gas_price(10_000).signed().unverified();
	let tx2 = Tx::gas_price(1).signed().local();

	let res = txq.import(TestClient::new().with_balance(1_000_000_000), vec![tx1]);
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);

	// when
	let res = txq.import(TestClient::new(), vec![tx2]);
	assert_eq!(res, vec![Ok(())]);

	// then
	assert_eq!(txq.status().status.transaction_count, 1);
}

#[test]
fn should_avoid_verifying_transaction_already_in_pool() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 1,
			max_per_sender: 2,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	let client = TestClient::new().with_balance(1_000_000_000);
	let tx1 = Tx::gas_price(2).signed().unverified();

	let res = txq.import(client.clone(), vec![tx1.clone()]);
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);
	assert!(client.was_verification_triggered());

	// when
	let client = TestClient::new();
	let res = txq.import(client.clone(), vec![tx1]);
	assert_eq!(res, vec![Err(transaction::Error::AlreadyImported)]);
	assert!(!client.was_verification_triggered());

	// then
	assert_eq!(txq.status().status.transaction_count, 1);
}

#[test]
fn should_avoid_reverifying_recently_rejected_transactions() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 1,
			max_per_sender: 2,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	);

	let client = TestClient::new();
	let tx1 = Tx::gas_price(10_000).signed().unverified();

	let res = txq.import(client.clone(), vec![tx1.clone()]);
	assert_eq!(res, vec![Err(transaction::Error::InsufficientBalance {
		balance: 0xf67c.into(),
		cost: 0xc8458e4.into(),
	})]);
	assert_eq!(txq.status().status.transaction_count, 0);
	assert!(client.was_verification_triggered());

	// when
	let client = TestClient::new();
	let res = txq.import(client.clone(), vec![tx1]);
	assert_eq!(res, vec![Err(transaction::Error::InsufficientBalance {
		balance: 0xf67c.into(),
		cost: 0xc8458e4.into(),
	})]);
	assert!(!client.was_verification_triggered());

	// then
	assert_eq!(txq.status().status.transaction_count, 0);
}

#[test]
fn should_reject_early_in_case_gas_price_is_less_than_min_effective() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 1,
			max_per_sender: 2,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: false,
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	let client = TestClient::new().with_balance(1_000_000_000);
	let tx1 = Tx::gas_price(2).signed().unverified();

	let res = txq.import(client.clone(), vec![tx1]);
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);
	assert!(client.was_verification_triggered());

	// when
	let client = TestClient::new();
	let tx1 = Tx::default().signed().unverified();
	let res = txq.import(client.clone(), vec![tx1]);
	assert_eq!(res, vec![Err(transaction::Error::InsufficientGasPrice {
		minimal: 2.into(),
		got: 1.into(),
	})]);
	assert!(!client.was_verification_triggered());

	// then
	assert_eq!(txq.status().status.transaction_count, 1);
}


#[test]
fn should_not_reject_early_in_case_gas_price_is_less_than_min_effective() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 1,
			max_per_sender: 2,
			max_mem_usage: TEST_QUEUE_MAX_MEM
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
			no_early_reject: true,
		},
		PrioritizationStrategy::GasPriceOnly,
	);
	// when
	let tx1 = Tx::gas_price(2).signed();
	let client = TestClient::new().with_local(&tx1.sender());
	let res = txq.import(client.clone(), vec![tx1.unverified()]);

	// then
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 1);
	assert!(client.was_verification_triggered());

	// when
	let tx1 = Tx::gas_price(1).signed();
	let client = TestClient::new().with_local(&tx1.sender());
	let res = txq.import(client.clone(), vec![tx1.unverified()]);

	// then
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	assert!(client.was_verification_triggered());
}
