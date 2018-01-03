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

//! Transaction Readiness indicator
//!
//! Transaction readiness is responsible for indicating if
//! particular transaction can be included in the block.
//!
//! Regular transactions are ready iff the current state nonce
//! (obtained from `Client`) equals to the transaction nonce.
//!
//! Let's define `S = state nonce`. Transactions are processed
//! in order, so we first include transaction with nonce `S`,
//! but then we are able to include the one with `S + 1` nonce.
//! So bear in mind that transactions can be included in chains
//! and their readiness is dependent on previous transactions from
//! the same sender.
//!
//! There are three possible outcomes:
//! - The transaction is old (stalled; state nonce < transaction nonce)
//! - The transaction is ready (current; state nonce == transaction nonce)
//! - The transaction is not ready yet (future; state nonce > transaction nonce)
//!
//! NOTE The transactions are always checked for readines in order they are stored within the queue.
//! First `Readiness::Future` response also causes all subsequent transactions from the same sender
//! to be marked as `Future`.

use std::cmp;
use std::collections::HashMap;

use ethereum_types::{U256, H160 as Address};
use txpool::{self, VerifiedTransaction as IVerifiedTransaction};

use super::client::Client;
use super::VerifiedTransaction;

/// Checks readiness of transactions by comparing the nonce to state nonce.
#[derive(Debug)]
pub struct ClientReadiness<C> {
	nonces: HashMap<Address, U256>,
	state: C,
}

impl<C: Client> txpool::Ready<VerifiedTransaction> for ClientReadiness<C> {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> txpool::Readiness {
		let sender = tx.sender();
		let state = &self.state;
		// TODO [ToDr] Handle null-sender transactions
		let state_nonce = || state.account_nonce(sender);
		let nonce = self.nonces.entry(*sender).or_insert_with(state_nonce);
		match tx.transaction.nonce.cmp(nonce) {
			cmp::Ordering::Greater => txpool::Readiness::Future,
			cmp::Ordering::Less => txpool::Readiness::Stalled,
			cmp::Ordering::Equal => {
				*nonce = *nonce + 1.into();
				txpool::Readiness::Ready
			},
		}
	}
}


