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

//! Transaction Pool

use ethereum_types::{H256, H160 as Address};
use heapsize::HeapSizeOf;
use transaction;
use txpool;

pub mod client;
pub mod listener;
pub mod local_transactions;
pub mod queue;
pub mod ready;
pub mod scoring;
pub mod verifier;

#[cfg(test)]
pub mod tests;

pub use self::queue::{TransactionQueue, Status as QueueStatus};
pub use self::txpool::{VerifiedTransaction as PoolVerifiedTransaction, Options};

// TODO [ToDr] Actually use that parameter and implement more strategies.

/// How to prioritize transactions in the pool
pub enum PrioritizationStrategy {
	/// Simple gas-price based prioritization.
	GasPrice,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Priority {
	Local,
	Retracted,
	Regular,
}

impl Priority {
	fn is_local(&self) -> bool {
		match *self {
			Priority::Local => true,
			_ => false,
		}
	}
}

/// Verified transaction stored in the pool.
#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedTransaction {
	transaction: transaction::PendingTransaction,
	// TODO [ToDr] hash and sender should go directly from the transaction
	hash: H256,
	sender: Address,
	priority: Priority,
	insertion_id: usize,
}

impl VerifiedTransaction {
	/// Create `VerifiedTransaction` directly from `SignedTransaction`.
	///
	/// This method should be used only:
	/// 1. for tests
	/// 2. In case we are converting pending block transactions that are already in the queue to match the function signature.
	pub fn from_pending_block_transaction(tx: transaction::SignedTransaction) -> Self {
		let hash = tx.hash();
		let sender = tx.sender();
		VerifiedTransaction {
			transaction: tx.into(),
			hash,
			sender,
			priority: Priority::Retracted,
			insertion_id: 0,
		}
	}

	/// Gets transaction priority.
	pub(crate) fn priority(&self) -> Priority {
		self.priority
	}

	/// Gets wrapped `SignedTransaction`
	pub fn signed(&self) -> &transaction::SignedTransaction {
		&self.transaction
	}

	/// Gets wrapped `PendingTransaction`
	pub fn pending(&self) -> &transaction::PendingTransaction {
		&self.transaction
	}
}

impl txpool::VerifiedTransaction for VerifiedTransaction {
	fn hash(&self) -> &H256 {
		&self.hash
	}

	fn mem_usage(&self) -> usize {
		self.transaction.heap_size_of_children()
	}

	fn sender(&self) -> &Address {
		&self.sender
	}

	fn insertion_id(&self) -> u64 {
		self.insertion_id as u64
	}
}
