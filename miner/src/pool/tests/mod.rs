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

use ethereum_types::{U256, H256, Address};
use ethkey::{Random, Generator};
use rustc_hex::FromHex;
use transaction::{self, Transaction, SignedTransaction, UnverifiedTransaction, PendingTransaction};
use txpool;

use pool::{self, verifier, VerifiedTransaction, TransactionQueue};
use pool::client::AccountDetails;

#[derive(Debug, Clone)]
pub struct TestClient {
	account_details: AccountDetails,
	gas_required: U256,
	is_service_transaction: bool,
	local_address: Address,
}

impl Default for TestClient {
	fn default() -> Self {
		TestClient {
			account_details: AccountDetails {
				nonce: 123.into(),
				balance: 63_100.into(),
				is_local: false,
			},
			gas_required: 21_000.into(),
			is_service_transaction: false,
			local_address: Default::default(),
		}
	}
}

impl TestClient {
	pub fn new() -> Self {
		TestClient::default()
	}

	pub fn with_account(mut self, account_details: AccountDetails) -> Self {
		self.account_details = account_details;
		self
	}

	pub fn with_nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
		self.account_details.nonce = nonce.into();
		self
	}

	pub fn with_gas_required<T: Into<U256>>(mut self, gas_required: T) -> Self {
		self.gas_required = gas_required.into();
		self
	}

	pub fn with_local(mut self, address: &Address) -> Self {
		self.local_address = *address;
		self
	}

	pub fn with_service_transaction(mut self) -> Self {
		self.is_service_transaction = true;
		self
	}
}

impl pool::client::Client for TestClient {
	fn transaction_already_included(&self, _hash: &H256) -> bool {
		false
	}

	fn verify_transaction(&self, tx: UnverifiedTransaction)
		-> Result<SignedTransaction, transaction::Error>
	{
		Ok(SignedTransaction::new(tx)?)
	}

	fn account_details(&self, address: &Address) -> AccountDetails {
		let mut details = self.account_details.clone();
		if address == &self.local_address {
			details.is_local = true;
		}

		details
	}

	fn required_gas(&self, _tx: &Transaction) -> U256 {
		self.gas_required
	}

	fn transaction_type(&self, _tx: &SignedTransaction) -> pool::client::TransactionType {
		if self.is_service_transaction {
			pool::client::TransactionType::Service
		} else {
			pool::client::TransactionType::Regular
		}
	}
}

impl pool::client::StateClient for TestClient {
	fn account_nonce(&self, _address: &Address) -> U256 {
		self.account_details.nonce
	}
}

trait TxExt: Sized {
	type Out;
	type Hash;

	fn hash(&self) -> Self::Hash;

	fn local(self) -> Self::Out;

	fn retracted(self) -> Self::Out;

	fn unverified(self) -> Self::Out;
}

impl<A, B, O, H> TxExt for (A, B) where
	A: TxExt<Out=O, Hash=H>,
	B: TxExt<Out=O, Hash=H>,
{
	type Out = (O, O);
	type Hash = (H, H);

	fn hash(&self) -> Self::Hash { (self.0.hash(), self.1.hash()) }
	fn local(self) -> Self::Out { (self.0.local(), self.1.local()) }
	fn retracted(self) -> Self::Out { (self.0.retracted(), self.1.retracted()) }
	fn unverified(self) -> Self::Out { (self.0.unverified(), self.1.unverified()) }
}

impl TxExt for SignedTransaction {
	type Out = verifier::Transaction;
	type Hash = H256;

	fn hash(&self) -> Self::Hash {
		UnverifiedTransaction::hash(self)
	}

	fn local(self) -> Self::Out {
		verifier::Transaction::Local(self.into())
	}

	fn retracted(self) -> Self::Out {
		verifier::Transaction::Retracted(self.into())
	}

	fn unverified(self) -> Self::Out {
		verifier::Transaction::Unverified(self.into())
	}
}

impl TxExt for Vec<SignedTransaction> {
	type Out = Vec<verifier::Transaction>;
	type Hash = Vec<H256>;

	fn hash(&self) -> Self::Hash {
		self.iter().map(|tx| tx.hash()).collect()
	}

	fn local(self) -> Self::Out {
		self.into_iter().map(Into::into).map(verifier::Transaction::Local).collect()
	}

	fn retracted(self) -> Self::Out {
		self.into_iter().map(Into::into).map(verifier::Transaction::Retracted).collect()
	}

	fn unverified(self) -> Self::Out {
		self.into_iter().map(Into::into).map(verifier::Transaction::Unverified).collect()
	}
}

trait PairExt {
	type Type;

	fn into_vec(self) -> Vec<Self::Type>;
}

impl<A> PairExt for (A, A) {
	type Type = A;
	fn into_vec(self) -> Vec<A> {
		vec![self.0, self.1]
	}
}

#[derive(Clone)]
struct Tx {
	nonce: u64,
	gas: u64,
	gas_price: u64,
}

impl Default for Tx {
	fn default() -> Self {
		Tx {
			nonce: 123,
			gas: 21_000,
			gas_price: 1,
		}
	}
}

impl Tx {
	pub fn gas(gas: u64) -> Self {
		Tx {
			gas,
			..Default::default()
		}
	}

	pub fn gas_price(gas_price: u64) -> Self {
		Tx {
			gas_price,
			..Default::default()
		}
	}

	pub fn nonce(nonce: u64) -> Self {
		Tx {
			nonce,
			..Default::default()
		}
	}

	pub fn signed(self) -> SignedTransaction {
		let keypair = Random.generate().unwrap();
		self.unsigned().sign(keypair.secret(), None)
	}

	pub fn signed_pair(self) -> (SignedTransaction, SignedTransaction) {
		let (tx1, tx2, _) = self.signed_triple();
		(tx1, tx2)
	}

	pub fn signed_triple(mut self) -> (SignedTransaction, SignedTransaction, SignedTransaction) {
		let keypair = Random.generate().unwrap();
		let tx1 = self.clone().unsigned().sign(keypair.secret(), None);
		self.nonce += 1;
		let tx2 = self.clone().unsigned().sign(keypair.secret(), None);
		self.nonce += 1;
		let tx3 = self.unsigned().sign(keypair.secret(), None);


		(tx1, tx2, tx3)
	}

	pub fn signed_replacement(mut self) -> (SignedTransaction, SignedTransaction) {
		let keypair = Random.generate().unwrap();
		let tx1 = self.clone().unsigned().sign(keypair.secret(), None);
		self.gas_price += 1;
		let tx2 = self.unsigned().sign(keypair.secret(), None);

		(tx1, tx2)
	}

	pub fn unsigned(self) -> Transaction {
		Transaction {
			action: transaction::Action::Create,
			value: U256::from(100),
			data: "3331600055".from_hex().unwrap(),
			gas: self.gas.into(),
			gas_price: self.gas_price.into(),
			nonce: self.nonce.into()
		}
	}
}

fn new_queue() -> TransactionQueue {
	TransactionQueue::new(
		txpool::Options {
			max_count: 3,
			max_per_sender: 3,
			max_mem_usage: 50
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
		},
	)
}

#[test]
fn should_return_correct_nonces_when_dropped_because_of_limit() {
	// given
	let txq = TransactionQueue::new(
		txpool::Options {
			max_count: 3,
			max_per_sender: 1,
			max_mem_usage: 50
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
		},
	);
	let (tx1, tx2) = Tx::gas_price(2).signed_pair();
	let sender = tx1.sender();
	let nonce = tx1.nonce;

	// when
	let result = txq.import(TestClient::new(), vec![tx1, tx2].local());
	assert_eq!(result, vec![Ok(()), Err(transaction::Error::LimitReached)]);
	assert_eq!(txq.status().status.transaction_count, 1);

	// then
	assert_eq!(txq.next_nonce(TestClient::new(), &sender), Some(nonce + 1.into()));

	// when
	let tx1 = Tx::gas_price(2).signed();
	let tx2 = Tx::gas_price(2).signed();
	let tx3 = Tx::gas_price(1).signed();
	let tx4 = Tx::gas_price(3).signed();
	let res = txq.import(TestClient::new(), vec![tx1, tx2, tx3, tx4].local());

	// then
	assert_eq!(res, vec![Ok(()), Ok(()), Err(transaction::Error::LimitReached), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 3);
	// First inserted transacton got dropped because of limit
	assert_eq!(txq.next_nonce(TestClient::new(), &sender), None);
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
	let top = txq.pending(TestClient::new(), 0, 0);
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
	let top = txq.pending(TestClient::new(), 0, 0);
	assert_eq!(top[0].hash, hash);
	assert_eq!(top[1].hash, hash2);
}

#[test]
fn should_drop_transactions_from_senders_without_balance() {
	// given
	let txq = new_queue();
	let tx = Tx::default().signed();
	let client = TestClient::new().with_account(AccountDetails {
		nonce: 123.into(),
		balance: 1.into(),
		is_local: false,
	});

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
	let top = txq.pending(TestClient::new(), 0 ,0);
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
	let top = txq.pending(client, 0, 0);
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
	let top = txq.pending(TestClient::new(), 0, 0);
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
	let top = txq.pending(TestClient::new(), 0, 0);
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
	let top = txq.pending(TestClient::new(), 0, 0);
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
	let top = txq.pending(TestClient::new(), 0, 0);
	assert_eq!(top.len(), 0);
	let top = txq.pending(TestClient::new(), 1, 0);
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
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 1);

	// when
	let res = txq.import(TestClient::new(), vec![tx1.local()]);
	assert_eq!(res, vec![Ok(())]);

	// then
	assert_eq!(txq.status().status.transaction_count, 3);
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 3);
}

#[test]
fn should_remove_transaction() {
	// given
	let txq = new_queue();
	let (tx, _, tx2) = Tx::default().signed_triple();

	let res = txq.import(TestClient::default(), vec![tx, tx2].local());
	assert_eq!(res, vec![Ok(()), Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 1);

	// when
	txq.cull(TestClient::new().with_nonce(124));
	assert_eq!(txq.status().status.transaction_count, 1);
	assert_eq!(txq.pending(TestClient::new().with_nonce(125), 0, 0).len(), 1);
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
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 1);

	let res = txq.import(TestClient::new(), vec![tx].local());
	assert_eq!(res, vec![Ok(())]);
	assert_eq!(txq.status().status.transaction_count, 3);
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 3);

	// when
	txq.remove(vec![&hash], true);

	// then
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 1);
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
			max_mem_usage: 50
		},
		verifier::Options {
			minimal_gas_price: 1.into(),
			block_gas_limit: 1_000_000.into(),
			tx_gas_limit: 1_000_000.into(),
		},
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

	let top = txq.pending(TestClient::new(), 0, 0);
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
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 2);

	// when
	txq.remove(vec![&hash], true);
	assert_eq!(txq.status().status.transaction_count, 1);
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 0);

	let res = txq.import(TestClient::new(), vec![tx1].local());
	assert_eq!(res, vec![Ok(())]);

	// then
	assert_eq!(txq.status().status.transaction_count, 2);
	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 2);
}

// #[test]
// fn should_not_replace_same_transaction_if_the_fee_is_less_than_minimal_bump() {
// 	// given
// 	let txq = new_queue();
// 	let keypair = Random.generate().unwrap();
// 	let tx = new_unsigned_tx(123.into(), default_gas_val(), 20.into()).sign(keypair.secret(), None);
// 	let tx2 = {
// 		let mut tx2 = (**tx).clone();
// 		tx2.gas_price = U256::from(21);
// 		tx2.sign(keypair.secret(), None)
// 	};
//
// 	// when
// 	txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	let res = txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider());
//
// 	// then
// 	assert_eq!(unwrap_tx_err(res), transaction::Error::TooCheapToReplace);
// 	let stats = txq.status();
// 	assert_eq!(stats.pending, 1);
// 	assert_eq!(stats.future, 0);
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0)[0].gas_price, U256::from(20));
// }
//
// #[test]
// fn should_replace_same_transaction_when_has_higher_fee() {
// 	// given
// 	let txq = new_queue();
// 	let keypair = Random.generate().unwrap();
// 	let tx = new_unsigned_tx(123.into(), default_gas_val(), 10.into()).sign(keypair.secret(), None);
// 	let tx2 = {
// 		let mut tx2 = (**tx).clone();
// 		tx2.gas_price = U256::from(20);
// 		tx2.sign(keypair.secret(), None)
// 	};
//
// 	// when
// 	txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
//
// 	// then
// 	let stats = txq.status();
// 	assert_eq!(stats.pending, 1);
// 	assert_eq!(stats.future, 0);
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0)[0].gas_price, U256::from(20));
// }
//
// #[test]
// fn should_replace_same_transaction_when_importing_to_futures() {
// 	// given
// 	let txq = new_queue();
// 	let keypair = Random.generate().unwrap();
// 	let tx0 = new_unsigned_tx(123.into(), default_gas_val(), 1.into()).sign(keypair.secret(), None);
// 	let tx1 = {
// 		let mut tx1 = (**tx0).clone();
// 		tx1.nonce = U256::from(124);
// 		tx1.sign(keypair.secret(), None)
// 	};
// 	let tx2 = {
// 		let mut tx2 = (**tx1).clone();
// 		tx2.gas_price = U256::from(200);
// 		tx2.sign(keypair.secret(), None)
// 	};
//
// 	// when
// 	txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	assert_eq!(txq.status().future, 1);
// 	txq.add(tx0, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
//
// 	// then
// 	let stats = txq.status();
// 	assert_eq!(stats.future, 0);
// 	assert_eq!(stats.pending, 2);
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0)[1].gas_price, U256::from(200));
// }
//
// #[test]
// fn should_recalculate_height_when_removing_from_future() {
// 	// given
// 	let previous_nonce = default_account_details().nonce - U256::one();
// 	let txq = new_queue();
// 	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
// 	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(previous_nonce)).unwrap();
// 	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(previous_nonce)).unwrap();
// 	assert_eq!(txq.status().future, 2);
//
// 	// when
// 	txq.remove(&tx1.hash(), &|_| default_nonce() + 1.into(), RemovalReason::Invalid);
//
// 	// then
// 	let stats = txq.status();
// 	assert_eq!(stats.future, 0);
// 	assert_eq!(stats.pending, 1);
// }
//
// #[test]
// fn should_return_none_when_transaction_from_given_address_does_not_exist() {
// 	// given
// 	let txq = TransactionQueue::default();
//
// 	// then
// 	assert_eq!(txq.last_nonce(&Address::default()), None);
// }
//
// #[test]
// fn should_return_correct_nonce_when_transactions_from_given_address_exist() {
// 	// given
// 	let txq = new_queue();
// 	let tx = Tx::default();
// 	let from = tx.sender();
// 	let nonce = tx.nonce;
//
// 	// when
// 	txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce)).unwrap();
//
// 	// then
// 	assert_eq!(txq.last_nonce(&from), Some(nonce));
// }
//
// #[test]
// fn should_remove_old_transaction_even_if_newer_transaction_was_not_known() {
// 	// given
// 	let txq = new_queue();
// 	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
// 	let (nonce1, nonce2) = (tx1.nonce, tx2.nonce);
//
// 	// Insert first transaction
// 	txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce1)).unwrap();
//
// 	// when
// 	txq.cull(tx2.sender(), nonce2 + U256::one());
//
// 	// then
// 	assert!(txq.pending(TestClient::new(), 0, 0).is_empty());
// }
//
// #[test]
// fn should_return_valid_last_nonce_after_cull() {
// 	// given
// 	let txq = new_queue();
// 	let (tx1, tx2) = new_tx_pair_default(4.into(), 0.into());
// 	let sender = tx1.sender();
// 	let (nonce1, nonce2) = (tx1.nonce, tx2.nonce);
//
// 	// when
// 	// Insert first transaction
// 	assert_eq!(txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce1)).unwrap(), transaction::ImportResult::Current);
// 	// Second should go to future
// 	assert_eq!(txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(nonce1)).unwrap(), transaction::ImportResult::Future);
// 	// Now block is imported
// 	txq.cull(sender, nonce2 - U256::from(1));
// 	// tx2 should be not be promoted to current
// 	assert_eq!(txq.status().pending, 0);
// 	assert_eq!(txq.status().future, 1);
//
// 	// then
// 	assert_eq!(txq.last_nonce(&sender), None);
// }
//
// #[test]
// fn should_return_true_if_there_is_local_transaction_pending() {
// 	// given
// 	let txq = new_queue();
// 	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
// 	assert_eq!(txq.has_local_pending_transactions(), false);
//
// 	// when
// 	assert_eq!(txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap(), transaction::ImportResult::Current);
// 	assert_eq!(txq.has_local_pending_transactions(), false);
// 	assert_eq!(txq.add(tx2, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap(),
// 	transaction::ImportResult::Current);
//
// 	// then
// 	assert_eq!(txq.has_local_pending_transactions(), true);
// }
//
// #[test]
// fn should_keep_right_order_in_future() {
// 	// given
// 	let mut txq = TransactionQueue::with_limits(
// 		PrioritizationStrategy::GasPriceOnly,
// 		1,
// 		usize::max_value(),
// 		!U256::zero(),
// 		!U256::zero()
// 	);
// 	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
// 	let prev_nonce = default_account_details().nonce - U256::one();
//
// 	// when
// 	assert_eq!(txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap(), transaction::ImportResult::Future);
// 	assert_eq!(txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider().with_account_nonce(prev_nonce)).unwrap(), transaction::ImportResult::Future);
//
// 	// then
// 	assert_eq!(txq.future.by_priority.len(), 1);
// 	assert_eq!(txq.future.by_priority.iter().next().unwrap().hash, tx1.hash());
// }
//
// #[test]
// fn should_return_correct_last_nonce() {
// 	// given
// 	let txq = new_queue();
// 	let (tx1, tx2, tx2_2, tx3) = {
// 		let keypair = Random.generate().unwrap();
// 		let secret = &keypair.secret();
// 		let nonce = 123.into();
// 		let gas = default_gas_val();
// 		let tx = new_unsigned_tx(nonce, gas, 1.into());
// 		let tx2 = new_unsigned_tx(nonce + 1.into(), gas, 1.into());
// 		let tx2_2 = new_unsigned_tx(nonce + 1.into(), gas, 5.into());
// 		let tx3 = new_unsigned_tx(nonce + 2.into(), gas, 1.into());
//
//
// 		(tx.sign(secret, None), tx2.sign(secret, None), tx2_2.sign(secret, None), tx3.sign(secret, None))
// 	};
// 	let sender = tx1.sender();
// 	txq.add(tx1, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx2, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx3, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
// 	assert_eq!(txq.future.by_priority.len(), 0);
// 	assert_eq!(txq.current.by_priority.len(), 3);
//
// 	// when
// 	let res = txq.add(tx2_2, TransactionOrigin::Local, 0, None, &default_tx_provider());
//
// 	// then
// 	assert_eq!(txq.last_nonce(&sender).unwrap(), 125.into());
// 	assert_eq!(res.unwrap(), transaction::ImportResult::Current);
// 	assert_eq!(txq.current.by_priority.len(), 3);
// }
//
// #[test]
// fn should_reject_transactions_below_base_gas() {
// 	// given
// 	let txq = new_queue();
// 	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
// 	let high_gas = 100_001.into();
//
// 	// when
// 	let res1 = txq.add(tx1, TransactionOrigin::Local, 0, None, &default_tx_provider());
// 	let res2 = txq.add(tx2, TransactionOrigin::Local, 0, None, &default_tx_provider().with_tx_gas_required(high_gas));
//
// 	// then
// 	assert_eq!(res1.unwrap(), transaction::ImportResult::Current);
// 	assert_eq!(unwrap_tx_err(res2), transaction::Error::InsufficientGas {
// 		minimal: 100_001.into(),
// 		got: 100_000.into(),
// 	});
//
// }
//
// #[test]
// fn should_clear_all_old_transactions() {
// 	// given
// 	let txq = new_queue();
// 	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
// 	let (tx3, tx4) = new_tx_pair_default(1.into(), 0.into());
// 	let next_nonce = |_: &Address|
// 		AccountDetails { nonce: default_nonce() + U256::one(), balance: !U256::zero() };
//
// 	// Insert all transactions
// 	txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx3, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx4, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 4);
//
// 	// when
// 	txq.remove_old(&next_nonce, 0);
//
// 	// then
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 2);
// }
//
// #[test]
// fn should_remove_out_of_date_transactions_occupying_queue() {
// 	// given
// 	let txq = new_queue();
// 	let (tx1, tx2) = new_tx_pair_default(1.into(), 0.into());
// 	let (tx3, tx4) = new_tx_pair_default(2.into(), 0.into());
//
// 	// Insert all transactions
// 	txq.add(tx1.clone(), TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx2, TransactionOrigin::External, 5, None, &default_tx_provider()).unwrap();
// 	txq.add(tx3.clone(), TransactionOrigin::External, 10, None, &default_tx_provider()).unwrap();
// 	txq.add(tx4, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 3);
// 	assert_eq!(txq.future_transactions().len(), 1);
//
// 	// when
// 	txq.remove_old(&default_account_details_for_addr, 9 + super::DEFAULT_QUEUING_PERIOD);
//
// 	// then
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 2);
// 	assert_eq!(txq.future_transactions().len(), 0);
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0), vec![tx1, tx3]);
// }
//
// #[test]
// fn should_accept_local_service_transaction() {
// 	// given
// 	let tx = new_tx(123.into(), 0.into());
// 	let txq = new_queue();
// 	txq.set_minimal_gas_price(100.into());
//
// 	// when
// 	txq.add(tx, TransactionOrigin::Local, 0, None, &default_tx_provider()).unwrap();
//
// 	// then
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 1);
// }
//
// #[test]
// fn should_not_accept_external_service_transaction_if_sender_not_certified() {
// 	// given
// 	let tx1 = new_tx(123.into(), 0.into());
// 	let tx2 = new_tx(456.into(), 0.into());
// 	let txq = new_queue();
// 	txq.set_minimal_gas_price(100.into());
//
// 	// when
// 	assert_eq!(unwrap_tx_err(txq.add(tx1, TransactionOrigin::External, 0, None, &default_tx_provider())),
// 		transaction::Error::InsufficientGasPrice {
// 			minimal: 100.into(),
// 			got: 0.into(),
// 		});
// 	assert_eq!(unwrap_tx_err(txq.add(tx2, TransactionOrigin::RetractedBlock, 0, None, &default_tx_provider())),
// 		transaction::Error::InsufficientGasPrice {
// 			minimal: 100.into(),
// 			got: 0.into(),
// 		});
//
// 	// then
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 0);
// }
//
// #[test]
// fn should_not_accept_external_service_transaction_if_contract_returns_error() {
// 	// given
// 	let tx = new_tx(123.into(), 0.into());
// 	let txq = new_queue();
// 	txq.set_minimal_gas_price(100.into());
//
// 	// when
// 	let details_provider = default_tx_provider().service_transaction_checker_returns_error("Contract error");
// 	assert_eq!(unwrap_tx_err(txq.add(tx, TransactionOrigin::External, 0, None, &details_provider)),
// 		transaction::Error::InsufficientGasPrice {
// 			minimal: 100.into(),
// 			got: 0.into(),
// 		});
//
// 	// then
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 0);
// }
//
// #[test]
// fn should_accept_external_service_transaction_if_sender_is_certified() {
// 	// given
// 	let tx = new_tx(123.into(), 0.into());
// 	let txq = new_queue();
// 	txq.set_minimal_gas_price(100.into());
//
// 	// when
// 	let details_provider = default_tx_provider().service_transaction_checker_accepts(true);
// 	txq.add(tx, TransactionOrigin::External, 0, None, &details_provider).unwrap();
//
// 	// then
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 1);
// }
//
// #[test]
// fn should_not_order_transactions_by_hash() {
// 	// given
// 	let secret1 = "0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap();
// 	let secret2 = "0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap();
// 	let tx1 = new_unsigned_tx(123.into(), default_gas_val(), 0.into()).sign(&secret1, None);
// 	let tx2 = new_unsigned_tx(123.into(), default_gas_val(), 0.into()).sign(&secret2, None);
// 	let txq = new_queue();
//
// 	// when
// 	txq.add(tx1.clone(), TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	txq.add(tx2, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
//
// 	// then
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0)[0], tx1);
// 	assert_eq!(txq.pending(TestClient::new(), 0, 0).len(), 2);
// }
//
// #[test]
// fn should_not_return_transactions_over_nonce_cap() {
// 	// given
// 	let keypair = Random.generate().unwrap();
// 	let txq = new_queue();
// 	// when
// 	for nonce in 123..130 {
// 		let tx = new_unsigned_tx(nonce.into(), default_gas_val(), default_gas_price()).sign(keypair.secret(), None);
// 		txq.add(tx, TransactionOrigin::External, 0, None, &default_tx_provider()).unwrap();
// 	}
//
// 	// then
// 	assert_eq!(txq.top_transactions_at(BlockNumber::max_value(), u64::max_value(), Some(127.into())).len(), 4);
// }
