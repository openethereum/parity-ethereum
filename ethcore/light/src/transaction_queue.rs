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

//! Light Transaction Queue.
//!
//! Manages local transactions,
//! but stores all local transactions, removing only on invalidated nonce.
//!
//! Under the assumption that light nodes will have a relatively limited set of
//! accounts for which they create transactions, this queue is structured in an
//! address-wise manner.

use ethcore::transaction::PendingTransaction;
use util::{Address, U256};

/// Light transaction queue. See module docs for more details.
pub struct TransactionQueue;

impl TransactionQueue {
	/// Insert a pending transaction to be queued.
	pub fn insert(&mut self, tx: PendingTransaction) {
		unimplemented!()
	}

	/// Get the next nonce for a given address based on what's within the queue.
	/// If the address has no queued transactions
	pub fn next_nonce(&mut self, address: &Address) -> Option<U256> {
		unimplemented!()
	}

	/// Get pending transactions, ready to be propagated.
	/// `best_block_number` and `best_block_timestamp` are used to filter out conditionally
	/// propagated transactions.
	pub fn pending_transactions(&self, best_block_number: u64, best_block_timestamp: u64) -> Vec<PendingTransaction> {
		unimplemented!()
	}

	/// Cull out all transactions by the given address which are invalidated by the given nonce.
	pub fn cull(&mut self, address: Address, last_nonce: U256) {
		unimplemented!()
	}
}
