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

//! Transaction Readiness indicator
//!
//! Transaction readiness is responsible for indicating if
//! particular transaction can be included in the block.
//!
//! Regular transactions are ready iff the current state nonce
//! (obtained from `NonceClient`) equals to the transaction nonce.
//!
//! Let's define `S = state nonce`. Transactions are processed
//! in order, so we first include transaction with nonce `S`,
//! but then we are able to include the one with `S + 1` nonce.
//! So bear in mind that transactions can be included in chains
//! and their readiness is dependent on previous transactions from
//! the same sender.
//!
//! There are three possible outcomes:
//! - The transaction is old (stalled; state nonce > transaction nonce)
//! - The transaction is ready (current; state nonce == transaction nonce)
//! - The transaction is not ready yet (future; state nonce < transaction nonce)
//!
//! NOTE The transactions are always checked for readines in order they are stored within the queue.
//! First `Readiness::Future` response also causes all subsequent transactions from the same sender
//! to be marked as `Future`.

use std::cmp;
use std::collections::HashMap;

use ethereum_types::{U256, H160 as Address};
use transaction;
use txpool::{self, VerifiedTransaction as PoolVerifiedTransaction};

use super::client::NonceClient;
use super::VerifiedTransaction;

/// Checks readiness of transactions by comparing the nonce to state nonce.
#[derive(Debug)]
pub struct State<C> {
	nonces: HashMap<Address, U256>,
	state: C,
	max_nonce: Option<U256>,
	stale_id: Option<usize>,
}

impl<C> State<C> {
	/// Create new State checker, given client interface.
	pub fn new(
		state: C,
		stale_id: Option<usize>,
		max_nonce: Option<U256>,
	) -> Self {
		State {
			nonces: Default::default(),
			state,
			max_nonce,
			stale_id,
		}
	}
}

impl<C: NonceClient> txpool::Ready<VerifiedTransaction> for State<C> {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> txpool::Readiness {
		// Check max nonce
		match self.max_nonce {
			Some(nonce) if tx.transaction.nonce > nonce => {
				return txpool::Readiness::Future;
			},
			_ => {},
		}

		let sender = tx.sender();
		let state = &self.state;
		let state_nonce = || state.account_nonce(sender);
		let nonce = self.nonces.entry(*sender).or_insert_with(state_nonce);
		match tx.transaction.nonce.cmp(nonce) {
			// Before marking as future check for stale ids
			cmp::Ordering::Greater => match self.stale_id {
				Some(id) if tx.insertion_id() < id => txpool::Readiness::Stale,
				_ => txpool::Readiness::Future,
			},
			cmp::Ordering::Less => txpool::Readiness::Stale,
			cmp::Ordering::Equal => {
				*nonce = *nonce + 1.into();
				txpool::Readiness::Ready
			},
		}
	}
}

/// Checks readines of Pending transactions by comparing it with current time and block number.
#[derive(Debug)]
pub struct Condition {
	block_number: u64,
	now: u64,
}

impl Condition {
	/// Create a new condition checker given current block number and UTC timestamp.
	pub fn new(block_number: u64, now: u64) -> Self {
		Condition {
			block_number,
			now,
		}
	}
}

impl txpool::Ready<VerifiedTransaction> for Condition {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> txpool::Readiness {
		match tx.transaction.condition {
			Some(transaction::Condition::Number(block)) if block > self.block_number => txpool::Readiness::Future,
			Some(transaction::Condition::Timestamp(time)) if time > self.now => txpool::Readiness::Future,
			_ => txpool::Readiness::Ready,
		}
	}
}

/// Readiness checker that only relies on nonce cache (does actually go to state).
///
/// Checks readiness of transactions by comparing the nonce to state nonce. If nonce
/// isn't found in provided state nonce store, defaults to the tx nonce and updates
/// the nonce store. Useful for using with a state nonce cache when false positives are allowed.
pub struct OptionalState<C> {
	nonces: HashMap<Address, U256>,
	state: C,
}

impl<C> OptionalState<C> {
	pub fn new(state: C) -> Self {
		OptionalState {
			nonces: Default::default(),
			state,
		}
	}
}

impl<C: Fn(&Address) -> Option<U256>> txpool::Ready<VerifiedTransaction> for OptionalState<C> {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> txpool::Readiness {
		let sender = tx.sender();
		let state = &self.state;
		let nonce = self.nonces.entry(*sender).or_insert_with(|| {
			state(sender).unwrap_or_else(|| tx.transaction.nonce)
		});
		match tx.transaction.nonce.cmp(nonce) {
			cmp::Ordering::Greater => txpool::Readiness::Future,
			cmp::Ordering::Less => txpool::Readiness::Stale,
			cmp::Ordering::Equal => {
				*nonce = *nonce + 1.into();
				txpool::Readiness::Ready
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use txpool::Ready;
	use pool::tests::client::TestClient;
	use pool::tests::tx::{Tx, TxExt};

	#[test]
	fn should_return_correct_state_readiness() {
		// given
		let (tx1, tx2, tx3) = Tx::default().signed_triple();
		let (tx1, tx2, tx3) = (tx1.verified(), tx2.verified(), tx3.verified());

		// when
		assert_eq!(State::new(TestClient::new(), None, None).is_ready(&tx3), txpool::Readiness::Future);
		assert_eq!(State::new(TestClient::new(), None, None).is_ready(&tx2), txpool::Readiness::Future);

		let mut ready = State::new(TestClient::new(), None, None);

		// then
		assert_eq!(ready.is_ready(&tx1), txpool::Readiness::Ready);
		assert_eq!(ready.is_ready(&tx2), txpool::Readiness::Ready);
		assert_eq!(ready.is_ready(&tx3), txpool::Readiness::Ready);
	}

	#[test]
	fn should_return_future_if_nonce_cap_reached() {
		// given
		let tx = Tx::default().signed().verified();

		// when
		let res1 = State::new(TestClient::new(), None, Some(10.into())).is_ready(&tx);
		let res2 = State::new(TestClient::new(), None, Some(124.into())).is_ready(&tx);

		// then
		assert_eq!(res1, txpool::Readiness::Future);
		assert_eq!(res2, txpool::Readiness::Ready);
	}

	#[test]
	fn should_return_stale_if_nonce_does_not_match() {
		// given
		let tx = Tx::default().signed().verified();

		// when
		let res = State::new(TestClient::new().with_nonce(125), None, None).is_ready(&tx);

		// then
		assert_eq!(res, txpool::Readiness::Stale);
	}

	#[test]
	fn should_return_stale_for_old_transactions() {
		// given
		let (_, tx) = Tx::default().signed_pair().verified();

		// when
		let res = State::new(TestClient::new(), Some(1), None).is_ready(&tx);

		// then
		assert_eq!(res, txpool::Readiness::Stale);
	}

	#[test]
	fn should_check_readiness_of_condition() {
		// given
		let tx = Tx::default().signed();
		let v = |tx: transaction::PendingTransaction| TestClient::new().verify(tx);
		let tx1 = v(transaction::PendingTransaction::new(tx.clone(), transaction::Condition::Number(5).into()));
		let tx2 = v(transaction::PendingTransaction::new(tx.clone(), transaction::Condition::Timestamp(3).into()));
		let tx3 = v(transaction::PendingTransaction::new(tx.clone(), None));

		// when/then
		assert_eq!(Condition::new(0, 0).is_ready(&tx1), txpool::Readiness::Future);
		assert_eq!(Condition::new(0, 0).is_ready(&tx2), txpool::Readiness::Future);
		assert_eq!(Condition::new(0, 0).is_ready(&tx3), txpool::Readiness::Ready);
		assert_eq!(Condition::new(5, 0).is_ready(&tx1), txpool::Readiness::Ready);
		assert_eq!(Condition::new(0, 3).is_ready(&tx2), txpool::Readiness::Ready);
	}
}
