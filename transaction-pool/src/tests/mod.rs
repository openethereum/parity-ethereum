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

mod helpers;
mod tx_builder;

use self::helpers::{DummyScoring, NonceReady};
use self::tx_builder::TransactionBuilder;

use std::sync::Arc;

use ethereum_types::{H256, U256, Address};
use super::*;

#[derive(Debug, PartialEq)]
pub struct Transaction {
	pub hash: H256,
	pub nonce: U256,
	pub gas_price: U256,
	pub gas: U256,
	pub sender: Address,
	pub mem_usage: usize,
}

impl VerifiedTransaction for Transaction {
	type Hash = H256;
	type Sender = Address;

	fn hash(&self) -> &H256 { &self.hash }
	fn mem_usage(&self) -> usize { self.mem_usage }
	fn sender(&self) -> &Address { &self.sender }
}

pub type SharedTransaction = Arc<Transaction>;

type TestPool = Pool<Transaction, DummyScoring>;

impl TestPool {
	pub fn with_limit(max_count: usize) -> Self {
		Self::with_options(Options {
			max_count,
			..Default::default()
		})
	}
}

#[test]
fn should_clear_queue() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();
	assert_eq!(txq.light_status(), LightStatus {
		mem_usage: 0,
		transaction_count: 0,
		senders: 0,
	});
	let tx1 = b.tx().nonce(0).new();
	let tx2 = b.tx().nonce(1).mem_usage(1).new();

	// add
	txq.import(tx1).unwrap();
	txq.import(tx2).unwrap();
	assert_eq!(txq.light_status(), LightStatus {
		mem_usage: 1,
		transaction_count: 2,
		senders: 1,
	});

	// when
	txq.clear();

	// then
	assert_eq!(txq.light_status(), LightStatus {
		mem_usage: 0,
		transaction_count: 0,
		senders: 0,
	});
}

#[test]
fn should_not_allow_same_transaction_twice() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();
	let tx1 = b.tx().nonce(0).new();
	let tx2 = b.tx().nonce(0).new();

	// when
	txq.import(tx1).unwrap();
	txq.import(tx2).unwrap_err();

	// then
	assert_eq!(txq.light_status().transaction_count, 1);
}

#[test]
fn should_replace_transaction() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();
	let tx1 = b.tx().nonce(0).gas_price(1).new();
	let tx2 = b.tx().nonce(0).gas_price(2).new();

	// when
	txq.import(tx1).unwrap();
	txq.import(tx2).unwrap();

	// then
	assert_eq!(txq.light_status().transaction_count, 1);
}

#[test]
fn should_reject_if_above_count() {
	let b = TransactionBuilder::default();
	let mut txq = TestPool::with_options(Options {
		max_count: 1,
		..Default::default()
	});

	// Reject second
	let tx1 = b.tx().nonce(0).new();
	let tx2 = b.tx().nonce(1).new();
	let hash = format!("{:?}", tx2.hash());
	txq.import(tx1).unwrap();
	assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash, "0x0".into()));
	assert_eq!(txq.light_status().transaction_count, 1);

	txq.clear();

	// Replace first
	let tx1 = b.tx().nonce(0).new();
	let tx2 = b.tx().nonce(0).sender(1).gas_price(2).new();
	txq.import(tx1).unwrap();
	txq.import(tx2).unwrap();
	assert_eq!(txq.light_status().transaction_count, 1);
}

#[test]
fn should_reject_if_above_mem_usage() {
	let b = TransactionBuilder::default();
	let mut txq = TestPool::with_options(Options {
		max_mem_usage: 1,
		..Default::default()
	});

	// Reject second
	let tx1 = b.tx().nonce(1).mem_usage(1).new();
	let tx2 = b.tx().nonce(2).mem_usage(2).new();
	let hash = format!("{:?}", tx2.hash());
	txq.import(tx1).unwrap();
	assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash, "0x0".into()));
	assert_eq!(txq.light_status().transaction_count, 1);

	txq.clear();

	// Replace first
	let tx1 = b.tx().nonce(1).mem_usage(1).new();
	let tx2 = b.tx().nonce(1).sender(1).gas_price(2).mem_usage(1).new();
	txq.import(tx1).unwrap();
	txq.import(tx2).unwrap();
	assert_eq!(txq.light_status().transaction_count, 1);
}

#[test]
fn should_reject_if_above_sender_count() {
	let b = TransactionBuilder::default();
	let mut txq = TestPool::with_options(Options {
		max_per_sender: 1,
		..Default::default()
	});

	// Reject second
	let tx1 = b.tx().nonce(1).new();
	let tx2 = b.tx().nonce(2).new();
	let hash = format!("{:x}", tx2.hash());
	txq.import(tx1).unwrap();
	assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash, "0x0".into()));
	assert_eq!(txq.light_status().transaction_count, 1);

	txq.clear();

	// Replace first
	let tx1 = b.tx().nonce(1).new();
	let tx2 = b.tx().nonce(2).gas_price(2).new();
	let hash = format!("{:x}", tx2.hash());
	txq.import(tx1).unwrap();
	// This results in error because we also compare nonces
	assert_eq!(txq.import(tx2).unwrap_err().kind(), &error::ErrorKind::TooCheapToEnter(hash, "0x0".into()));
	assert_eq!(txq.light_status().transaction_count, 1);
}

#[test]
fn should_construct_pending() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();

	let tx0 = txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	let tx1 = txq.import(b.tx().nonce(1).gas_price(5).new()).unwrap();
	let tx2 = txq.import(b.tx().nonce(2).new()).unwrap();
	// this transaction doesn't get to the block despite high gas price
	// because of block gas limit and simplistic ordering algorithm.
	txq.import(b.tx().nonce(3).gas_price(4).new()).unwrap();
	//gap
	txq.import(b.tx().nonce(5).new()).unwrap();

	let tx5 = txq.import(b.tx().sender(1).nonce(0).new()).unwrap();
	let tx6 = txq.import(b.tx().sender(1).nonce(1).new()).unwrap();
	let tx7 = txq.import(b.tx().sender(1).nonce(2).new()).unwrap();
	let tx8 = txq.import(b.tx().sender(1).nonce(3).gas_price(4).new()).unwrap();
	// gap
	txq.import(b.tx().sender(1).nonce(5).new()).unwrap();

	let tx9 = txq.import(b.tx().sender(2).nonce(0).new()).unwrap();
	assert_eq!(txq.light_status().transaction_count, 11);
	assert_eq!(txq.status(NonceReady::default()), Status {
		stalled: 0,
		pending: 9,
		future: 2,
	});
	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 3,
		pending: 6,
		future: 2,
	});

	// when
	let mut current_gas = U256::zero();
	let limit = (21_000 * 8).into();
	let mut pending = txq.pending(NonceReady::default()).take_while(|tx| {
		let should_take = tx.gas + current_gas <= limit;
		if should_take {
			current_gas = current_gas + tx.gas
		}
		should_take
	});

	assert_eq!(pending.next(), Some(tx0));
	assert_eq!(pending.next(), Some(tx1));
	assert_eq!(pending.next(), Some(tx9));
	assert_eq!(pending.next(), Some(tx5));
	assert_eq!(pending.next(), Some(tx6));
	assert_eq!(pending.next(), Some(tx7));
	assert_eq!(pending.next(), Some(tx8));
	assert_eq!(pending.next(), Some(tx2));
	assert_eq!(pending.next(), None);
}

#[test]
fn should_return_unordered_iterator() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();

	let tx0 = txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	let tx1 = txq.import(b.tx().nonce(1).gas_price(5).new()).unwrap();
	let tx2 = txq.import(b.tx().nonce(2).new()).unwrap();
	let tx3 = txq.import(b.tx().nonce(3).gas_price(4).new()).unwrap();
	//gap
	txq.import(b.tx().nonce(5).new()).unwrap();

	let tx5 = txq.import(b.tx().sender(1).nonce(0).new()).unwrap();
	let tx6 = txq.import(b.tx().sender(1).nonce(1).new()).unwrap();
	let tx7 = txq.import(b.tx().sender(1).nonce(2).new()).unwrap();
	let tx8 = txq.import(b.tx().sender(1).nonce(3).gas_price(4).new()).unwrap();
	// gap
	txq.import(b.tx().sender(1).nonce(5).new()).unwrap();

	let tx9 = txq.import(b.tx().sender(2).nonce(0).new()).unwrap();
	assert_eq!(txq.light_status().transaction_count, 11);
	assert_eq!(txq.status(NonceReady::default()), Status {
		stalled: 0,
		pending: 9,
		future: 2,
	});
	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 3,
		pending: 6,
		future: 2,
	});

	// when
	let all: Vec<_> = txq.unordered_pending(NonceReady::default()).collect();

	let chain1 = vec![tx0, tx1, tx2, tx3];
	let chain2 = vec![tx5, tx6, tx7, tx8];
	let chain3 = vec![tx9];

	assert_eq!(all.len(), chain1.len() + chain2.len() + chain3.len());

	let mut options = vec![
		vec![chain1.clone(), chain2.clone(), chain3.clone()],
		vec![chain2.clone(), chain1.clone(), chain3.clone()],
		vec![chain2.clone(), chain3.clone(), chain1.clone()],
		vec![chain3.clone(), chain2.clone(), chain1.clone()],
		vec![chain3.clone(), chain1.clone(), chain2.clone()],
		vec![chain1.clone(), chain3.clone(), chain2.clone()],
	].into_iter().map(|mut v| {
		let mut first = v.pop().unwrap();
		for mut x in v {
			first.append(&mut x);
		}
		first
	});

	assert!(options.any(|opt| all == opt));
}

#[test]
fn should_update_scoring_correctly() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();

	let tx0 = txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	let tx1 = txq.import(b.tx().nonce(1).gas_price(5).new()).unwrap();
	let tx2 = txq.import(b.tx().nonce(2).new()).unwrap();
	// this transaction doesn't get to the block despite high gas price
	// because of block gas limit and simplistic ordering algorithm.
	txq.import(b.tx().nonce(3).gas_price(4).new()).unwrap();
	//gap
	txq.import(b.tx().nonce(5).new()).unwrap();

	let tx5 = txq.import(b.tx().sender(1).nonce(0).new()).unwrap();
	let tx6 = txq.import(b.tx().sender(1).nonce(1).new()).unwrap();
	let tx7 = txq.import(b.tx().sender(1).nonce(2).new()).unwrap();
	let tx8 = txq.import(b.tx().sender(1).nonce(3).gas_price(4).new()).unwrap();
	// gap
	txq.import(b.tx().sender(1).nonce(5).new()).unwrap();

	let tx9 = txq.import(b.tx().sender(2).nonce(0).new()).unwrap();
	assert_eq!(txq.light_status().transaction_count, 11);
	assert_eq!(txq.status(NonceReady::default()), Status {
		stalled: 0,
		pending: 9,
		future: 2,
	});
	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 3,
		pending: 6,
		future: 2,
	});

	txq.update_scores(&0.into(), ());

	// when
	let mut current_gas = U256::zero();
	let limit = (21_000 * 8).into();
	let mut pending = txq.pending(NonceReady::default()).take_while(|tx| {
		let should_take = tx.gas + current_gas <= limit;
		if should_take {
			current_gas = current_gas + tx.gas
		}
		should_take
	});

	assert_eq!(pending.next(), Some(tx9));
	assert_eq!(pending.next(), Some(tx5));
	assert_eq!(pending.next(), Some(tx6));
	assert_eq!(pending.next(), Some(tx7));
	assert_eq!(pending.next(), Some(tx8));
	// penalized transactions
	assert_eq!(pending.next(), Some(tx0));
	assert_eq!(pending.next(), Some(tx1));
	assert_eq!(pending.next(), Some(tx2));
	assert_eq!(pending.next(), None);
}

#[test]
fn should_remove_transaction() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();

	let tx1 = txq.import(b.tx().nonce(0).new()).unwrap();
	let tx2 = txq.import(b.tx().nonce(1).new()).unwrap();
	txq.import(b.tx().nonce(2).new()).unwrap();
	assert_eq!(txq.light_status().transaction_count, 3);

	// when
	assert!(txq.remove(&tx2.hash(), false).is_some());

	// then
	assert_eq!(txq.light_status().transaction_count, 2);
	let mut pending = txq.pending(NonceReady::default());
	assert_eq!(pending.next(), Some(tx1));
	assert_eq!(pending.next(), None);
}

#[test]
fn should_cull_stalled_transactions() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();

	txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	txq.import(b.tx().nonce(1).new()).unwrap();
	txq.import(b.tx().nonce(3).new()).unwrap();

	txq.import(b.tx().sender(1).nonce(0).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(1).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(5).new()).unwrap();

	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 2,
		pending: 2,
		future: 2,
	});

	// when
	assert_eq!(txq.cull(None, NonceReady::new(1)), 2);

	// then
	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 0,
		pending: 2,
		future: 2,
	});
	assert_eq!(txq.light_status(), LightStatus {
		transaction_count: 4,
		senders: 2,
		mem_usage: 0,
	});
}

#[test]
fn should_cull_stalled_transactions_from_a_sender() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();

	txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	txq.import(b.tx().nonce(1).new()).unwrap();

	txq.import(b.tx().sender(1).nonce(0).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(1).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(2).new()).unwrap();

	assert_eq!(txq.status(NonceReady::new(2)), Status {
		stalled: 4,
		pending: 1,
		future: 0,
	});

	// when
	let sender = 0.into();
	assert_eq!(txq.cull(Some(&[sender]), NonceReady::new(2)), 2);

	// then
	assert_eq!(txq.status(NonceReady::new(2)), Status {
		stalled: 2,
		pending: 1,
		future: 0,
	});
	assert_eq!(txq.light_status(), LightStatus {
		transaction_count: 3,
		senders: 1,
		mem_usage: 0,
	});
}

#[test]
fn should_re_insert_after_cull() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();

	txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	txq.import(b.tx().nonce(1).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(0).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(1).new()).unwrap();
	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 2,
		pending: 2,
		future: 0,
	});

	// when
	assert_eq!(txq.cull(None, NonceReady::new(1)), 2);
	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 0,
		pending: 2,
		future: 0,
	});
	txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(0).new()).unwrap();

	assert_eq!(txq.status(NonceReady::new(1)), Status {
		stalled: 2,
		pending: 2,
		future: 0,
	});
}

#[test]
fn should_return_worst_transaction() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::default();
	assert!(txq.worst_transaction().is_none());

	// when
	txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	txq.import(b.tx().sender(1).nonce(0).gas_price(4).new()).unwrap();

	// then
	assert_eq!(txq.worst_transaction().unwrap().gas_price, 4.into());
}

#[test]
fn should_return_is_full() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::with_limit(2);
	assert!(!txq.is_full());

	// when
	txq.import(b.tx().nonce(0).gas_price(110).new()).unwrap();
	assert!(!txq.is_full());

	txq.import(b.tx().sender(1).nonce(0).gas_price(100).new()).unwrap();

	// then
	assert!(txq.is_full());
}

#[test]
fn should_import_even_if_limit_is_reached_and_should_replace_returns_insert_new() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::with_scoring(DummyScoring::always_insert(), Options {
		max_count: 1,
		..Default::default()
	});
	txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	assert_eq!(txq.light_status(), LightStatus {
		transaction_count: 1,
		senders: 1,
		mem_usage: 0,
	});

	// when
	txq.import(b.tx().nonce(1).gas_price(5).new()).unwrap();

	// then
	assert_eq!(txq.light_status(), LightStatus {
		transaction_count: 2,
		senders: 1,
		mem_usage: 0,
	});
}

#[test]
fn should_not_import_even_if_limit_is_reached_and_should_replace_returns_false() {
	// given
	let b = TransactionBuilder::default();
	let mut txq = TestPool::with_scoring(DummyScoring::default(), Options {
		max_count: 1,
		..Default::default()
	});
	txq.import(b.tx().nonce(0).gas_price(5).new()).unwrap();
	assert_eq!(txq.light_status(), LightStatus {
		transaction_count: 1,
		senders: 1,
		mem_usage: 0,
	});

	// when
	let err = txq.import(b.tx().nonce(1).gas_price(5).new()).unwrap_err();

	// then
	assert_eq!(err.kind(),
	&error::ErrorKind::TooCheapToEnter("0x00000000000000000000000000000000000000000000000000000000000001f5".into(), "0x5".into()));
	assert_eq!(txq.light_status(), LightStatus {
		transaction_count: 1,
		senders: 1,
		mem_usage: 0,
	});
}

mod listener {
	use std::cell::RefCell;
	use std::rc::Rc;

	use super::*;

	#[derive(Default)]
	struct MyListener(pub Rc<RefCell<Vec<&'static str>>>);

	impl Listener<Transaction> for MyListener {
		fn added(&mut self, _tx: &SharedTransaction, old: Option<&SharedTransaction>) {
			self.0.borrow_mut().push(if old.is_some() { "replaced" } else { "added" });
		}

		fn rejected(&mut self, _tx: &SharedTransaction, _reason: &error::ErrorKind) {
			self.0.borrow_mut().push("rejected".into());
		}

		fn dropped(&mut self, _tx: &SharedTransaction, _new: Option<&Transaction>) {
			self.0.borrow_mut().push("dropped".into());
		}

		fn invalid(&mut self, _tx: &SharedTransaction) {
			self.0.borrow_mut().push("invalid".into());
		}

		fn canceled(&mut self, _tx: &SharedTransaction) {
			self.0.borrow_mut().push("canceled".into());
		}

		fn mined(&mut self, _tx: &SharedTransaction) {
			self.0.borrow_mut().push("mined".into());
		}
	}

	#[test]
	fn insert_transaction() {
		let b = TransactionBuilder::default();
		let listener = MyListener::default();
		let results = listener.0.clone();
		let mut txq = Pool::new(listener, DummyScoring::default(), Options {
			max_per_sender: 1,
			max_count: 2,
			..Default::default()
		});
		assert!(results.borrow().is_empty());

		// Regular import
		txq.import(b.tx().nonce(1).new()).unwrap();
		assert_eq!(*results.borrow(), &["added"]);
		// Already present (no notification)
		txq.import(b.tx().nonce(1).new()).unwrap_err();
		assert_eq!(*results.borrow(), &["added"]);
		// Push out the first one
		txq.import(b.tx().nonce(1).gas_price(1).new()).unwrap();
		assert_eq!(*results.borrow(), &["added", "replaced"]);
		// Reject
		txq.import(b.tx().nonce(1).new()).unwrap_err();
		assert_eq!(*results.borrow(), &["added", "replaced", "rejected"]);
		results.borrow_mut().clear();
		// Different sender (accept)
		txq.import(b.tx().sender(1).nonce(1).gas_price(2).new()).unwrap();
		assert_eq!(*results.borrow(), &["added"]);
		// Third sender push out low gas price
		txq.import(b.tx().sender(2).nonce(1).gas_price(4).new()).unwrap();
		assert_eq!(*results.borrow(), &["added", "dropped", "added"]);
		// Reject (too cheap)
		txq.import(b.tx().sender(2).nonce(1).gas_price(2).new()).unwrap_err();
		assert_eq!(*results.borrow(), &["added", "dropped", "added", "rejected"]);

		assert_eq!(txq.light_status().transaction_count, 2);
	}

	#[test]
	fn remove_transaction() {
		let b = TransactionBuilder::default();
		let listener = MyListener::default();
		let results = listener.0.clone();
		let mut txq = Pool::new(listener, DummyScoring::default(), Options::default());

		// insert
		let tx1 = txq.import(b.tx().nonce(1).new()).unwrap();
		let tx2 = txq.import(b.tx().nonce(2).new()).unwrap();

		// then
		txq.remove(&tx1.hash(), false);
		assert_eq!(*results.borrow(), &["added", "added", "canceled"]);
		txq.remove(&tx2.hash(), true);
		assert_eq!(*results.borrow(), &["added", "added", "canceled", "invalid"]);
		assert_eq!(txq.light_status().transaction_count, 0);
	}

	#[test]
	fn clear_queue() {
		let b = TransactionBuilder::default();
		let listener = MyListener::default();
		let results = listener.0.clone();
		let mut txq = Pool::new(listener, DummyScoring::default(), Options::default());

		// insert
		txq.import(b.tx().nonce(1).new()).unwrap();
		txq.import(b.tx().nonce(2).new()).unwrap();

		// when
		txq.clear();

		// then
		assert_eq!(*results.borrow(), &["added", "added", "dropped", "dropped"]);
	}

	#[test]
	fn cull_stalled() {
		let b = TransactionBuilder::default();
		let listener = MyListener::default();
		let results = listener.0.clone();
		let mut txq = Pool::new(listener, DummyScoring::default(), Options::default());

		// insert
		txq.import(b.tx().nonce(1).new()).unwrap();
		txq.import(b.tx().nonce(2).new()).unwrap();

		// when
		txq.cull(None, NonceReady::new(3));

		// then
		assert_eq!(*results.borrow(), &["added", "added", "mined", "mined"]);
	}
}
