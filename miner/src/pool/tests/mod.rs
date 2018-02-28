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

use ethereum_types::{U256, Address};
use ethkey::{Random, Generator};
use rustc_hex::FromHex;
use transaction::{self, Transaction, SignedTransaction Unverified};

use pool::VerifiedTransaction;
use pool::client::AccountDetails;

pub struct TestClient {
	account_details: AccountDetails,
	gas_required: U256,
	is_service_transaction: bool,
}

impl Default for TestClient {
	fn default() -> Self {

	}
}

impl TestClient {
	pub fn with_account(mut self, account_details: AccountDetails) -> Self {
		self.account_details = account_details;
		self
	}

	pub fn with_account_nonce(mut self, nonce: U256) -> Self {
		self.account_details.nonce = nonce;
		self
	}

	pub fn with_tx_gas_required(mut self, gas_required: U256) -> Self {
		self.gas_required = gas_required;
		self
	}

	pub fn with_service_transaction(mut self) -> Self {
		self.is_service_transaction;
		self
	}
}

impl pool::client::Client for TestClient {
	fn transaction_already_included(&self, _hash: &H256) -> bool {
		false
	}

	fn verify_transaction(&self, _tx: UnverifiedTransaction)
		-> Result<SignedTransaction, transaction::Error>
	{
		unimplemented!()
	}

	fn account_details(&self, _address: &Address) -> AccountDetails {
		self.account_details.clone()
	}

	fn required_gas(&self, _tx: &Transaction) -> U256 {
		self.gas_required
	}

	fn transaction_type(&self, _tx: &SignedTransaction) -> pool::client::TransactionType {
		if is_service_transaction {
			pool::client::TransactionType::Service
		} else {
			pool::client::TransactionType::Regular
		}
	}
}

fn unwrap_tx_err(err: Result<transaction::ImportResult, transaction::Error>) -> transaction::Error {
	err.unwrap_err()
}

fn default_nonce() -> U256 { 123.into() }
fn default_gas_val() -> U256 { 100_000.into() }
fn default_gas_price() -> U256 { 1.into() }

fn new_unsigned_tx(nonce: U256, gas: U256, gas_price: U256) -> Transaction {
	Transaction {
		action: transaction::Action::Create,
		value: U256::from(100),
		data: "3331600055".from_hex().unwrap(),
		gas: gas,
		gas_price: gas_price,
		nonce: nonce
	}
}

fn new_tx(nonce: U256, gas_price: U256) -> SignedTransaction {
	let keypair = Random.generate().unwrap();
	new_unsigned_tx(nonce, default_gas_val(), gas_price).sign(keypair.secret(), None)
}

fn new_tx_with_gas(gas: U256, gas_price: U256) -> SignedTransaction {
	let keypair = Random.generate().unwrap();
	new_unsigned_tx(default_nonce(), gas, gas_price).sign(keypair.secret(), None)
}

fn new_tx_default() -> SignedTransaction {
	new_tx(default_nonce(), default_gas_price())
}

fn default_account_details() -> AccountDetails {
	AccountDetails {
		nonce: default_nonce(),
		balance: !U256::zero()
	}
}

fn default_account_details_for_addr(_a: &Address) -> AccountDetails {
	default_account_details()
}

fn default_tx_provider() -> DummyTransactionDetailsProvider {
	DummyTransactionDetailsProvider::default()
}

fn new_tx_pair(nonce: U256, gas_price: U256, nonce_increment: U256, gas_price_increment: U256) -> (SignedTransaction, SignedTransaction) {
	let tx1 = new_unsigned_tx(nonce, default_gas_val(), gas_price);
	let tx2 = new_unsigned_tx(nonce + nonce_increment, default_gas_val(), gas_price + gas_price_increment);

	let keypair = Random.generate().unwrap();
	let secret = &keypair.secret();
	(tx1.sign(secret, None).into(), tx2.sign(secret, None).into())
}

/// Returns two consecutive transactions, both with increased gas price
fn new_tx_pair_with_gas_price_increment(gas_price_increment: U256) -> (SignedTransaction, SignedTransaction) {
	let gas = default_gas_price() + gas_price_increment;
	let tx1 = new_unsigned_tx(default_nonce(), default_gas_val(), gas);
	let tx2 = new_unsigned_tx(default_nonce() + 1.into(), default_gas_val(), gas);

	let keypair = Random.generate().unwrap();
	let secret = &keypair.secret();
	(tx1.sign(secret, None).into(), tx2.sign(secret, None).into())
}

fn new_tx_pair_default(nonce_increment: U256, gas_price_increment: U256) -> (SignedTransaction, SignedTransaction) {
	new_tx_pair(default_nonce(), default_gas_price(), nonce_increment, gas_price_increment)
}

/// Returns two transactions with identical (sender, nonce) but different gas price/hash.
fn new_similar_tx_pair() -> (SignedTransaction, SignedTransaction) {
	new_tx_pair_default(0.into(), 1.into())
}

#[test]
fn should_return_correct_nonces_when_dropped_because_of_limit() {
	// given
	let mut txq = TransactionQueue::with_limits(
		PrioritizationStrategy::GasPriceOnly,
		2,
		usize::max_value(),
		!U256::zero(),
		!U256::zero(),
	);
	let (tx1, tx2) = new_tx_pair(123.into(), 1.into(), 1.into(), 0.into());
	let sender = tx1.sender();
	let nonce = tx1.nonce;
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 2);
	assert_eq!(txq.last_nonce(&sender), Some(nonce + 1.into()));

	// when
	let tx = new_tx(123.into(), 1.into());
	let res = txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	// No longer the case as we don't even consider a transaction that isn't above a full
	// queue's minimum gas price.
	// We may want to reconsider this in the near future so leaving this code in as a
	// possible alternative.
	/*
	assert_eq!(res.unwrap(), transaction::ImportResult::Current);
	assert_eq!(txq.status().pending, 2);
	assert_eq!(txq.last_nonce(&sender), Some(nonce));
	*/
	assert_eq!(unwrap_tx_err(res), transaction::Error::InsufficientGasPrice {
		minimal: 2.into(),
		got: 1.into(),
	});
	assert_eq!(txq.status().pending, 2);
	assert_eq!(txq.last_nonce(&sender), Some(tx2.nonce));
}

#[test]
fn should_handle_same_transaction_imported_twice_with_different_state_nonces() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx, tx2) = new_similar_tx_pair();
	let prev_nonce = default_account_details().nonce - U256::one();

	// First insert one transaction to future
	let res = txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce));
	assert_eq!(res.unwrap(), transaction::ImportResult::Future);
	assert_eq!(txq.status().future, 1);

	// now import second transaction to current
	let res = txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider());

	// and then there should be only one transaction in current (the one with higher gas_price)
	assert_eq!(res.unwrap(), transaction::ImportResult::Current);
	assert_eq!(txq.status().pending, 1);
	assert_eq!(txq.status().future, 0);
	assert_eq!(txq.current.by_priority.len(), 1);
	assert_eq!(txq.current.by_address.len(), 1);
	let top = txq.top_transactions();
	assert_eq!(top[0], tx2);
}

#[test]
fn should_move_all_transactions_from_future() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx, tx2) = new_tx_pair_default(1.into(), 1.into());
	let prev_nonce = default_account_details().nonce - U256::one();

	// First insert one transaction to future
	let res = txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce));
	assert_eq!(res.unwrap(), transaction::ImportResult::Future);
	assert_eq!(txq.status().future, 1);

	// now import second transaction to current
	let res = txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	assert_eq!(res.unwrap(), transaction::ImportResult::Current);
	assert_eq!(txq.status().pending, 2);
	assert_eq!(txq.status().future, 0);
	assert_eq!(txq.current.by_priority.len(), 2);
	assert_eq!(txq.current.by_address.len(), 2);
	let top = txq.top_transactions();
	assert_eq!(top[0], tx);
	assert_eq!(top[1], tx2);
}

#[test]
fn should_import_tx() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();

	// when
	let res = txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	assert_eq!(res.unwrap(), transaction::ImportResult::Current);
	let stats = txq.status();
	assert_eq!(stats.pending, 1);
}

#[test]
fn should_order_by_gas() {
	// given
	let mut txq = TransactionQueue::new(PrioritizationStrategy::GasAndGasPrice);
	let tx1 = new_tx_with_gas(50000.into(), 40.into());
	let tx2 = new_tx_with_gas(40000.into(), 30.into());
	let tx3 = new_tx_with_gas(30000.into(), 10.into());
	let tx4 = new_tx_with_gas(50000.into(), 20.into());
	txq.set_minimal_gas_price(15.into());

	// when
	let res1 = txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider());
	let res2 = txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider());
	let res3 = txq.add(tx3, TransactionOrigin::External, 0, None, &default_tx_provider());
	let res4 = txq.add(tx4, TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	assert_eq!(res1.unwrap(), transaction::ImportResult::Current);
	assert_eq!(res2.unwrap(), transaction::ImportResult::Current);
	assert_eq!(unwrap_tx_err(res3), transaction::Error::InsufficientGasPrice {
		minimal: U256::from(15),
		got: U256::from(10),
	});
	assert_eq!(res4.unwrap(), transaction::ImportResult::Current);
	let stats = txq.status();
	assert_eq!(stats.pending, 3);
	assert_eq!(txq.top_transactions()[0].gas, 40000.into());
	assert_eq!(txq.top_transactions()[1].gas, 50000.into());
	assert_eq!(txq.top_transactions()[2].gas, 50000.into());
	assert_eq!(txq.top_transactions()[1].gas_price, 40.into());
	assert_eq!(txq.top_transactions()[2].gas_price, 20.into());
}

#[test]
fn should_order_by_gas_factor() {
	// given
	let mut txq = TransactionQueue::new(PrioritizationStrategy::GasFactorAndGasPrice);

	let tx1 = new_tx_with_gas(150_000.into(), 40.into());
	let tx2 = new_tx_with_gas(40_000.into(), 16.into());
	let tx3 = new_tx_with_gas(30_000.into(), 15.into());
	let tx4 = new_tx_with_gas(150_000.into(), 62.into());
	txq.set_minimal_gas_price(15.into());

	// when
	let res1 = txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider());
	let res2 = txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider());
	let res3 = txq.add(tx3, TransactionOrigin::External, 0, None, &default_tx_provider());
	let res4 = txq.add(tx4, TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	assert_eq!(res1.unwrap(), transaction::ImportResult::Current);
	assert_eq!(res2.unwrap(), transaction::ImportResult::Current);
	assert_eq!(res3.unwrap(), transaction::ImportResult::Current);
	assert_eq!(res4.unwrap(), transaction::ImportResult::Current);
	let stats = txq.status();
	assert_eq!(stats.pending, 4);
	assert_eq!(txq.top_transactions()[0].gas, 30_000.into());
	assert_eq!(txq.top_transactions()[1].gas, 150_000.into());
	assert_eq!(txq.top_transactions()[2].gas, 40_000.into());
	assert_eq!(txq.top_transactions()[3].gas, 150_000.into());
	assert_eq!(txq.top_transactions()[0].gas_price, 15.into());
	assert_eq!(txq.top_transactions()[1].gas_price, 62.into());
	assert_eq!(txq.top_transactions()[2].gas_price, 16.into());
	assert_eq!(txq.top_transactions()[3].gas_price, 40.into());
}

#[test]
fn tx_gas_limit_should_never_overflow() {
	// given
	let mut txq = TransactionQueue::default();
	txq.set_gas_limit(U256::zero());
	assert_eq!(txq.block_gas_limit, U256::zero());

	// when
	txq.set_gas_limit(!U256::zero());

	// then
	assert_eq!(txq.block_gas_limit, !U256::zero());
}

#[test]
fn should_not_import_transaction_above_gas_limit() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	let gas = tx.gas;
	let limit = gas / U256::from(2);
	txq.set_gas_limit(limit);

	// when
	let res = txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	assert_eq!(unwrap_tx_err(res), transaction::Error::GasLimitExceeded {
		limit: U256::from(50_000),
		got: gas,
	});
	let stats = txq.status();
	assert_eq!(stats.pending, 0);
	assert_eq!(stats.future, 0);
}


#[test]
fn should_drop_transactions_from_senders_without_balance() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	let account = AccountDetails {
		nonce: default_account_details().nonce,
		balance: U256::one()
	};

	// when
	let res = txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider().with_account(account));

	// then
	assert_eq!(unwrap_tx_err(res), transaction::Error::InsufficientBalance {
		balance: U256::from(1),
		cost: U256::from(100_100),
	});
	let stats = txq.status();
	assert_eq!(stats.pending, 0);
	assert_eq!(stats.future, 0);
}

#[test]
fn should_not_import_transaction_below_min_gas_price_threshold_if_external() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	txq.set_minimal_gas_price(tx.gas_price + U256::one());

	// when
	let res = txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	assert_eq!(unwrap_tx_err(res), transaction::Error::InsufficientGasPrice {
		minimal: U256::from(2),
		got: U256::from(1),
	});
	let stats = txq.status();
	assert_eq!(stats.pending, 0);
	assert_eq!(stats.future, 0);
}

#[test]
fn should_import_transaction_below_min_gas_price_threshold_if_local() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	txq.set_minimal_gas_price(tx.gas_price + U256::one());

	// when
	let res = txq.add(tx, TransactionOrigin::Local, 0, None, &default_tx_provider());

	// then
	assert_eq!(res.unwrap(), transaction::ImportResult::Current);
	let stats = txq.status();
	assert_eq!(stats.pending, 1);
	assert_eq!(stats.future, 0);
}

#[test]
fn should_import_txs_from_same_sender() {
	// given
	let mut txq = TransactionQueue::default();

	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());

	// when
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	let top = txq.top_transactions();
	assert_eq!(top[0], tx);
	assert_eq!(top[1], tx2);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_prioritize_local_transactions_within_same_nonce_height() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	// the second one has same nonce but higher `gas_price`
	let (_, tx2) = new_similar_tx_pair();

	// when
	// first insert the one with higher gas price
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	// then the one with lower gas price, but local
	txq.add(tx.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();

	// then
	let top = txq.top_transactions();
	assert_eq!(top[0], tx); // local should be first
	assert_eq!(top[1], tx2);
	assert_eq!(top.len(), 2);
}

#[test]
fn when_importing_local_should_mark_others_from_the_same_sender_as_local() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	// the second one has same nonce but higher `gas_price`
	let (_, tx0) = new_similar_tx_pair();

	txq.add(tx0.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	// the one with higher gas price is first
	let top = txq.top_transactions();
	assert_eq!(top[0], tx0);
	assert_eq!(top[1], tx1);

	// when
	// insert second as local
	txq.add(tx2.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();

	// then
	// the order should be updated
	let top = txq.top_transactions();
	assert_eq!(top[0], tx1);
	assert_eq!(top[1], tx2);
	assert_eq!(top[2], tx0);
}

#[test]
fn should_prioritize_reimported_transactions_within_same_nonce_height() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	// the second one has same nonce but higher `gas_price`
	let (_, tx2) = new_similar_tx_pair();

	// when
	// first insert local one with higher gas price
	txq.add(tx2.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	// then the one with lower gas price, but from retracted block
	txq.add(tx.clone(), TransactionOrigin::RetractedBlock, 0, None, &default_tx_provider()).unwrap();

	// then
	let top = txq.top_transactions();
	assert_eq!(top[0], tx); // retracted should be first
	assert_eq!(top[1], tx2);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_not_prioritize_local_transactions_with_different_nonce_height() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());

	// when
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();

	// then
	let top = txq.top_transactions();
	assert_eq!(top[0], tx);
	assert_eq!(top[1], tx2);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_penalize_transactions_from_sender_in_future() {
	// given
	let prev_nonce = default_account_details().nonce - U256::one();
	let mut txq = TransactionQueue::default();
	// txa, txb - slightly bigger gas price to have consistent ordering
	let (txa, txb) = new_tx_pair_default(1.into(), 0.into());
	let (tx1, tx2) = new_tx_pair_with_gas_price_increment(3.into());

	// insert everything
	txq.add(txa.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap();
	txq.add(txb.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap();
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap();

	assert_eq!(txq.status().future, 4);

	// when
	txq.penalize(&tx1.hash());

	// then
	let top: Vec<_> = txq.future_transactions().into_iter().map(|tx| tx.transaction).collect();
	assert_eq!(top[0], txa);
	assert_eq!(top[1], txb);
	assert_eq!(top[2], tx1);
	assert_eq!(top[3], tx2);
	assert_eq!(top.len(), 4);
}

#[test]
fn should_not_penalize_local_transactions() {
	// given
	let mut txq = TransactionQueue::default();
	// txa, txb - slightly bigger gas price to have consistent ordering
	let (txa, txb) = new_tx_pair_default(1.into(), 0.into());
	let (tx1, tx2) = new_tx_pair_with_gas_price_increment(3.into());

	// insert everything
	txq.add(txa.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(txb.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx1.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();

	let top = txq.top_transactions();
	assert_eq!(top[0], tx1);
	assert_eq!(top[1], txa);
	assert_eq!(top[2], tx2);
	assert_eq!(top[3], txb);
	assert_eq!(top.len(), 4);

	// when
	txq.penalize(&tx1.hash());

	// then (order is the same)
	let top = txq.top_transactions();
	assert_eq!(top[0], tx1);
	assert_eq!(top[1], txa);
	assert_eq!(top[2], tx2);
	assert_eq!(top[3], txb);
	assert_eq!(top.len(), 4);
}

#[test]
fn should_penalize_transactions_from_sender() {
	// given
	let mut txq = TransactionQueue::default();
	// txa, txb - slightly bigger gas price to have consistent ordering
	let (txa, txb) = new_tx_pair_default(1.into(), 0.into());
	let (tx1, tx2) = new_tx_pair_with_gas_price_increment(3.into());

	// insert everything
	txq.add(txa.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(txb.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	let top = txq.top_transactions();
	assert_eq!(top[0], tx1);
	assert_eq!(top[1], txa);
	assert_eq!(top[2], tx2);
	assert_eq!(top[3], txb);
	assert_eq!(top.len(), 4);

	// when
	txq.penalize(&tx1.hash());

	// then
	let top = txq.top_transactions();
	assert_eq!(top[0], txa);
	assert_eq!(top[1], txb);
	assert_eq!(top[2], tx1);
	assert_eq!(top[3], tx2);
	assert_eq!(top.len(), 4);
}

#[test]
fn should_return_pending_hashes() {
	// given
	let mut txq = TransactionQueue::default();

	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());

	// when
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	let top = txq.pending_hashes();
	assert_eq!(top[0], tx.hash());
	assert_eq!(top[1], tx2.hash());
	assert_eq!(top.len(), 2);
}

#[test]
fn should_put_transaction_to_futures_if_gap_detected() {
	// given
	let mut txq = TransactionQueue::default();

	let (tx, tx2) = new_tx_pair_default(2.into(), 0.into());

	// when
	let res1 = txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	let res2 = txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	assert_eq!(res1, transaction::ImportResult::Current);
	assert_eq!(res2, transaction::ImportResult::Future);
	let stats = txq.status();
	assert_eq!(stats.pending, 1);
	assert_eq!(stats.future, 1);
	let top = txq.top_transactions();
	assert_eq!(top.len(), 1);
	assert_eq!(top[0], tx);
}

#[test]
fn should_handle_min_block() {
	// given
	let mut txq = TransactionQueue::default();

	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());

	// when
	let res1 = txq.add(tx.clone(), TransactionOrigin::External, 0, Some(transaction::Condition::Number(1)), &default_tx_provider()).unwrap();
	let res2 = txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	assert_eq!(res1, transaction::ImportResult::Current);
	assert_eq!(res2, transaction::ImportResult::Current);
	let top = txq.top_transactions_at(0, 0, None);
	assert_eq!(top.len(), 0);
	let top = txq.top_transactions_at(1, 0, None);
	assert_eq!(top.len(), 2);
}

#[test]
fn should_correctly_update_futures_when_removing() {
	// given
	let prev_nonce = default_account_details().nonce - U256::one();
	let next2_nonce = default_nonce() + U256::from(3);

	let mut txq = TransactionQueue::default();

	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap();
	assert_eq!(txq.status().future, 2);

	// when
	txq.cull(tx.sender(), next2_nonce);
	// should remove both transactions since they are not valid

	// then
	assert_eq!(txq.status().pending, 0);
	assert_eq!(txq.status().future, 0);
}

#[test]
fn should_move_transactions_if_gap_filled() {
	// given
	let mut txq = TransactionQueue::default();
	let kp = Random.generate().unwrap();
	let secret = kp.secret();
	let tx = new_unsigned_tx(123.into(), default_gas_val(), 1.into()).sign(secret, None).into();
	let tx1 = new_unsigned_tx(124.into(), default_gas_val(), 1.into()).sign(secret, None).into();
	let tx2 = new_unsigned_tx(125.into(), default_gas_val(), 1.into()).sign(secret, None).into();

	txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 1);
	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().future, 1);

	// when
	txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	let stats = txq.status();
	assert_eq!(stats.pending, 3);
	assert_eq!(stats.future, 0);
	assert_eq!(txq.future.by_priority.len(), 0);
	assert_eq!(txq.future.by_address.len(), 0);
	assert_eq!(txq.future.by_gas_price.len(), 0);
}

#[test]
fn should_remove_transaction() {
	// given
	let mut txq2 = TransactionQueue::default();
	let (tx, tx2) = new_tx_pair_default(3.into(), 0.into());
	txq2.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq2.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq2.status().pending, 1);
	assert_eq!(txq2.status().future, 1);

	// when
	txq2.cull(tx.sender(), tx.nonce + U256::one());
	txq2.cull(tx2.sender(), tx2.nonce + U256::one());

	// then
	let stats = txq2.status();
	assert_eq!(stats.pending, 0);
	assert_eq!(stats.future, 0);
}

#[test]
fn should_move_transactions_to_future_if_gap_introduced() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());
	let tx3 = new_tx_default();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().future, 1);
	txq.add(tx3.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 3);

	// when
	txq.remove(&tx.hash(), &|_| default_nonce(), RemovalReason::Invalid);

	// then
	let stats = txq.status();
	assert_eq!(stats.future, 1);
	assert_eq!(stats.pending, 1);
}

#[test]
fn should_clear_queue() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());

	// add
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	let stats = txq.status();
	assert_eq!(stats.pending, 2);

	// when
	txq.clear();

	// then
	let stats = txq.status();
	assert_eq!(stats.pending, 0);
}

#[test]
fn should_drop_old_transactions_when_hitting_the_limit() {
	// given
	let mut txq = TransactionQueue::with_limits(
		PrioritizationStrategy::GasPriceOnly,
		1,
		usize::max_value(),
		!U256::zero(),
		!U256::zero()
	);
	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());
	let sender = tx.sender();
	let nonce = tx.nonce;
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 1);

	// when
	let res = txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	let t = txq.top_transactions();
	assert_eq!(unwrap_tx_err(res), transaction::Error::InsufficientGasPrice { minimal: 2.into(), got: 1.into() });
	assert_eq!(txq.status().pending, 1);
	assert_eq!(t.len(), 1);
	assert_eq!(t[0], tx);
	assert_eq!(txq.last_nonce(&sender), Some(nonce));
}

#[test]
fn should_limit_future_transactions() {
	let mut txq = TransactionQueue::with_limits(
		PrioritizationStrategy::GasPriceOnly,
		1,
		usize::max_value(),
		!U256::zero(),
		!U256::zero(),
	);
	txq.current.set_limit(10);
	let (tx1, tx2) = new_tx_pair_default(4.into(), 1.into());
	let (tx3, tx4) = new_tx_pair_default(4.into(), 2.into());
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx3.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 2);

	// when
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().future, 1);
	txq.add(tx4.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	assert_eq!(txq.status().future, 1);
}

#[test]
fn should_limit_by_gas() {
	let mut txq = TransactionQueue::with_limits(
		PrioritizationStrategy::GasPriceOnly,
		100,
		usize::max_value(),
		default_gas_val() * U256::from(2),
		!U256::zero()
	);
	let (tx1, tx2) = new_tx_pair_default(U256::from(1), U256::from(1));
	let (tx3, tx4) = new_tx_pair_default(U256::from(1), U256::from(2));
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx3.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	// limited by gas
	txq.add(tx4.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap_err();
	assert_eq!(txq.status().pending, 2);
}

#[test]
fn should_keep_own_transactions_above_gas_limit() {
	let mut txq = TransactionQueue::with_limits(
		PrioritizationStrategy::GasPriceOnly,
		100,
		usize::max_value(),
		default_gas_val() * U256::from(2),
		!U256::zero()
	);
	let (tx1, tx2) = new_tx_pair_default(U256::from(1), U256::from(1));
	let (tx3, tx4) = new_tx_pair_default(U256::from(1), U256::from(2));
	let (tx5, _) = new_tx_pair_default(U256::from(1), U256::from(2));
	txq.add(tx1.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	// Not accepted because of limit
	txq.add(tx5.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap_err();
	txq.add(tx3.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx4.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 4);
}

#[test]
fn should_drop_transactions_with_old_nonces() {
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	let last_nonce = tx.nonce + U256::one();

	// when
	let res = txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(last_nonce));

	// then
	assert_eq!(unwrap_tx_err(res), transaction::Error::Old);
	let stats = txq.status();
	assert_eq!(stats.pending, 0);
	assert_eq!(stats.future, 0);
}

#[test]
fn should_not_insert_same_transaction_twice() {
	// given
	let nonce = default_account_details().nonce + U256::one();
	let mut txq = TransactionQueue::default();
	let (_tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().future, 1);
	assert_eq!(txq.status().pending, 0);

	// when
	let res = txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce));

	// then
	assert_eq!(unwrap_tx_err(res), transaction::Error::AlreadyImported);
	let stats = txq.status();
	assert_eq!(stats.future, 1);
	assert_eq!(stats.pending, 0);
}

#[test]
fn should_accept_same_transaction_twice_if_removed() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 2);

	// when
	txq.remove(&tx1.hash(), &|_| default_nonce(), RemovalReason::Invalid);
	assert_eq!(txq.status().pending, 0);
	assert_eq!(txq.status().future, 1);
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	let stats = txq.status();
	assert_eq!(stats.future, 0);
	assert_eq!(stats.pending, 2);
}

#[test]
fn should_not_move_to_future_if_state_nonce_is_higher() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx, tx2) = new_tx_pair_default(1.into(), 0.into());
	let tx3 = new_tx_default();
	txq.add(tx2.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().future, 1);
	txq.add(tx3.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().pending, 3);

	// when
	txq.cull(tx.sender(), default_nonce() + U256::one());

	// then
	let stats = txq.status();
	assert_eq!(stats.future, 0);
	assert_eq!(stats.pending, 2);
}

#[test]
fn should_not_replace_same_transaction_if_the_fee_is_less_than_minimal_bump() {
	// given
	let mut txq = TransactionQueue::default();
	let keypair = Random.generate().unwrap();
	let tx = new_unsigned_tx(123.into(), default_gas_val(), 20.into()).sign(keypair.secret(), None);
	let tx2 = {
		let mut tx2 = (**tx).clone();
		tx2.gas_price = U256::from(21);
		tx2.sign(keypair.secret(), None)
	};

	// when
	txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	let res = txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider());

	// then
	assert_eq!(unwrap_tx_err(res), transaction::Error::TooCheapToReplace);
	let stats = txq.status();
	assert_eq!(stats.pending, 1);
	assert_eq!(stats.future, 0);
	assert_eq!(txq.top_transactions()[0].gas_price, U256::from(20));
}

#[test]
fn should_replace_same_transaction_when_has_higher_fee() {
	// given
	let mut txq = TransactionQueue::default();
	let keypair = Random.generate().unwrap();
	let tx = new_unsigned_tx(123.into(), default_gas_val(), 10.into()).sign(keypair.secret(), None);
	let tx2 = {
		let mut tx2 = (**tx).clone();
		tx2.gas_price = U256::from(20);
		tx2.sign(keypair.secret(), None)
	};

	// when
	txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	let stats = txq.status();
	assert_eq!(stats.pending, 1);
	assert_eq!(stats.future, 0);
	assert_eq!(txq.top_transactions()[0].gas_price, U256::from(20));
}

#[test]
fn should_replace_same_transaction_when_importing_to_futures() {
	// given
	let mut txq = TransactionQueue::default();
	let keypair = Random.generate().unwrap();
	let tx0 = new_unsigned_tx(123.into(), default_gas_val(), 1.into()).sign(keypair.secret(), None);
	let tx1 = {
		let mut tx1 = (**tx0).clone();
		tx1.nonce = U256::from(124);
		tx1.sign(keypair.secret(), None)
	};
	let tx2 = {
		let mut tx2 = (**tx1).clone();
		tx2.gas_price = U256::from(200);
		tx2.sign(keypair.secret(), None)
	};

	// when
	txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.status().future, 1);
	txq.add(tx0, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	let stats = txq.status();
	assert_eq!(stats.future, 0);
	assert_eq!(stats.pending, 2);
	assert_eq!(txq.top_transactions()[1].gas_price, U256::from(200));
}

#[test]
fn should_recalculate_height_when_removing_from_future() {
	// given
	let previous_nonce = default_account_details().nonce - U256::one();
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(previous_nonce)).unwrap();
	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(previous_nonce)).unwrap();
	assert_eq!(txq.status().future, 2);

	// when
	txq.remove(&tx1.hash(), &|_| default_nonce() + 1.into(), RemovalReason::Invalid);

	// then
	let stats = txq.status();
	assert_eq!(stats.future, 0);
	assert_eq!(stats.pending, 1);
}

#[test]
fn should_return_none_when_transaction_from_given_address_does_not_exist() {
	// given
	let txq = TransactionQueue::default();

	// then
	assert_eq!(txq.last_nonce(&Address::default()), None);
}

#[test]
fn should_return_correct_nonce_when_transactions_from_given_address_exist() {
	// given
	let mut txq = TransactionQueue::default();
	let tx = new_tx_default();
	let from = tx.sender();
	let nonce = tx.nonce;

	// when
	txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce)).unwrap();

	// then
	assert_eq!(txq.last_nonce(&from), Some(nonce));
}

#[test]
fn should_remove_old_transaction_even_if_newer_transaction_was_not_known() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	let (nonce1, nonce2) = (tx1.nonce, tx2.nonce);

	// Insert first transaction
	txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce1)).unwrap();

	// when
	txq.cull(tx2.sender(), nonce2 + U256::one());

	// then
	assert!(txq.top_transactions().is_empty());
}

#[test]
fn should_return_valid_last_nonce_after_cull() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(4.into(), 0.into());
	let sender = tx1.sender();
	let (nonce1, nonce2) = (tx1.nonce, tx2.nonce);

	// when
	// Insert first transaction
	assert_eq!(txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce1)).unwrap(), transaction::ImportResult::Current);
	// Second should go to future
	assert_eq!(txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce1)).unwrap(), transaction::ImportResult::Future);
	// Now block is imported
	txq.cull(sender, nonce2 - U256::from(1));
	// tx2 should be not be promoted to current
	assert_eq!(txq.status().pending, 0);
	assert_eq!(txq.status().future, 1);

	// then
	assert_eq!(txq.last_nonce(&sender), None);
}

#[test]
fn should_return_true_if_there_is_local_transaction_pending() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	assert_eq!(txq.has_local_pending_transactions(), false);

	// when
	assert_eq!(txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap(), transaction::ImportResult::Current);
	assert_eq!(txq.has_local_pending_transactions(), false);
	assert_eq!(txq.add(tx2, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap(),
	transaction::ImportResult::Current);

	// then
	assert_eq!(txq.has_local_pending_transactions(), true);
}

#[test]
fn should_keep_right_order_in_future() {
	// given
	let mut txq = TransactionQueue::with_limits(
		PrioritizationStrategy::GasPriceOnly,
		1,
		usize::max_value(),
		!U256::zero(),
		!U256::zero()
	);
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	let prev_nonce = default_account_details().nonce - U256::one();

	// when
	assert_eq!(txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap(), transaction::ImportResult::Future);
	assert_eq!(txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap(), transaction::ImportResult::Future);

	// then
	assert_eq!(txq.future.by_priority.len(), 1);
	assert_eq!(txq.future.by_priority.iter().next().unwrap().hash, tx1.hash());
}

#[test]
fn should_return_correct_last_nonce() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2, tx2_2, tx3) = {
		let keypair = Random.generate().unwrap();
		let secret = &keypair.secret();
		let nonce = 123.into();
		let gas = default_gas_val();
		let tx = new_unsigned_tx(nonce, gas, 1.into());
		let tx2 = new_unsigned_tx(nonce + 1.into(), gas, 1.into());
		let tx2_2 = new_unsigned_tx(nonce + 1.into(), gas, 5.into());
		let tx3 = new_unsigned_tx(nonce + 2.into(), gas, 1.into());


		(tx.sign(secret, None), tx2.sign(secret, None), tx2_2.sign(secret, None), tx3.sign(secret, None))
	};
	let sender = tx1.sender();
	txq.add(tx1, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx3, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.future.by_priority.len(), 0);
	assert_eq!(txq.current.by_priority.len(), 3);

	// when
	let res = txq.add(tx2_2, TransactionOrigin::Local, 0, None, &default_tx_provider());

	// then
	assert_eq!(txq.last_nonce(&sender).unwrap(), 125.into());
	assert_eq!(res.unwrap(), transaction::ImportResult::Current);
	assert_eq!(txq.current.by_priority.len(), 3);
}

#[test]
fn should_reject_transactions_below_base_gas() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	let high_gas = 100_001.into();

	// when
	let res1 = txq.add(tx1, TransactionOrigin::Local, 0, None, &default_tx_provider());
	let res2 = txq.add(tx2, TransactionOrigin::Local, 0, None, &default_tx_provider().with_tx_gas_required(high_gas));

	// then
	assert_eq!(res1.unwrap(), transaction::ImportResult::Current);
	assert_eq!(unwrap_tx_err(res2), transaction::Error::InsufficientGas {
		minimal: 100_001.into(),
		got: 100_000.into(),
	});

}

#[test]
fn should_clear_all_old_transactions() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	let (tx3, tx4) = new_tx_pair_default(1.into(), 0.into());
	let next_nonce = |_: &Address|
		AccountDetails { nonce: default_nonce() + U256::one(), balance: !U256::zero() };

	// Insert all transactions
	txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx3, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx4, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.top_transactions().len(), 4);

	// when
	txq.remove_old(&next_nonce, 0);

	// then
	assert_eq!(txq.top_transactions().len(), 2);
}

#[test]
fn should_remove_out_of_date_transactions_occupying_queue() {
	// given
	let mut txq = TransactionQueue::default();
	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
	let (tx3, tx4) = new_tx_pair_default(2.into(), 0.into());

	// Insert all transactions
	txq.add(tx1.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2, TransactionOrigin::External, 5, None, &default_tx_provider()).unwrap();
	txq.add(tx3.clone(), TransactionOrigin::External, 10, None, &default_tx_provider()).unwrap();
	txq.add(tx4, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	assert_eq!(txq.top_transactions().len(), 3);
	assert_eq!(txq.future_transactions().len(), 1);

	// when
	txq.remove_old(&default_account_details_for_addr, 9 + super::DEFAULT_QUEUING_PERIOD);

	// then
	assert_eq!(txq.top_transactions().len(), 2);
	assert_eq!(txq.future_transactions().len(), 0);
	assert_eq!(txq.top_transactions(), vec![tx1, tx3]);
}

#[test]
fn should_accept_local_service_transaction() {
	// given
	let tx = new_tx(123.into(), 0.into());
	let mut txq = TransactionQueue::default();
	txq.set_minimal_gas_price(100.into());

	// when
	txq.add(tx, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();

	// then
	assert_eq!(txq.top_transactions().len(), 1);
}

#[test]
fn should_not_accept_external_service_transaction_if_sender_not_certified() {
	// given
	let tx1 = new_tx(123.into(), 0.into());
	let tx2 = new_tx(456.into(), 0.into());
	let mut txq = TransactionQueue::default();
	txq.set_minimal_gas_price(100.into());

	// when
	assert_eq!(unwrap_tx_err(txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider())),
		transaction::Error::InsufficientGasPrice {
			minimal: 100.into(),
			got: 0.into(),
		});
	assert_eq!(unwrap_tx_err(txq.add(tx2, TransactionOrigin::RetractedBlock, 0, None, &default_tx_provider())),
		transaction::Error::InsufficientGasPrice {
			minimal: 100.into(),
			got: 0.into(),
		});

	// then
	assert_eq!(txq.top_transactions().len(), 0);
}

#[test]
fn should_not_accept_external_service_transaction_if_contract_returns_error() {
	// given
	let tx = new_tx(123.into(), 0.into());
	let mut txq = TransactionQueue::default();
	txq.set_minimal_gas_price(100.into());

	// when
	let details_provider = default_tx_provider().service_transaction_checker_returns_error("Contract error");
	assert_eq!(unwrap_tx_err(txq.add(tx, TransactionOrigin::External, 0, None, &details_provider)),
		transaction::Error::InsufficientGasPrice {
			minimal: 100.into(),
			got: 0.into(),
		});

	// then
	assert_eq!(txq.top_transactions().len(), 0);
}

#[test]
fn should_accept_external_service_transaction_if_sender_is_certified() {
	// given
	let tx = new_tx(123.into(), 0.into());
	let mut txq = TransactionQueue::default();
	txq.set_minimal_gas_price(100.into());

	// when
	let details_provider = default_tx_provider().service_transaction_checker_accepts(true);
	txq.add(tx, TransactionOrigin::External, 0, None, &details_provider).unwrap();

	// then
	assert_eq!(txq.top_transactions().len(), 1);
}

#[test]
fn should_not_order_transactions_by_hash() {
	// given
	let secret1 = "0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap();
	let secret2 = "0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap();
	let tx1 = new_unsigned_tx(123.into(), default_gas_val(), 0.into()).sign(&secret1, None);
	let tx2 = new_unsigned_tx(123.into(), default_gas_val(), 0.into()).sign(&secret2, None);
	let mut txq = TransactionQueue::default();

	// when
	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();

	// then
	assert_eq!(txq.top_transactions()[0], tx1);
	assert_eq!(txq.top_transactions().len(), 2);
}

#[test]
fn should_not_return_transactions_over_nonce_cap() {
	// given
	let keypair = Random.generate().unwrap();
	let mut txq = TransactionQueue::default();
	// when
	for nonce in 123..130 {
		let tx = new_unsigned_tx(nonce.into(), default_gas_val(), default_gas_price()).sign(keypair.secret(), None);
		txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
	}

	// then
	assert_eq!(txq.top_transactions_at(BlockNumber::max_value(), u64::max_value(), Some(127.into())).len(), 4);
}
