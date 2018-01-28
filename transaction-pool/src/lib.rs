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

//! Generic Transaction Pool
//!
//! An extensible and performant implementation of Ethereum Transaction Pool.
//! The pool stores ordered, verified transactions according to some pluggable
//! `Scoring` implementation.
//! The pool also allows you to construct a set of `pending` transactions according
//! to some notion of `Readiness` (pluggable).
//!
//! The pool is generic over transactions and should make no assumptions about them.
//! The only thing we can rely on is the `Scoring` that defines:
//!  - the ordering of transactions from a single sender
//!  - the priority of the transaction compared to other transactions from different senders
//!
//! NOTE: the transactions from a single sender are not ordered by priority,
//! but still when constructing pending set we always need to maintain the ordering
//! (i.e. `txs[1]` always needs to be included after `txs[0]` even if it has higher priority)
//!
//! ### Design Details
//!
//! Performance assumptions:
//! - Possibility to handle tens of thousands of transactions
//! - Fast insertions and replacements `O(per-sender + log(senders))`
//! - Reasonably fast removal of stalled transactions `O(per-sender)`
//! - Reasonably fast construction of pending set `O(txs * (log(senders) + log(per-sender))`
//!
//! The removal performance could be improved by trading some memory. Currently `SmallVec` is used
//! to store senders transactions, instead we could use `VecDeque` and efficiently `pop_front`
//! the best transactions.
//!
//! The pending set construction and insertion complexity could be reduced by introducing
//! a notion of `nonce` - an absolute, numeric ordering of transactions.
//! We don't do that because of possible implications of EIP208 where nonce might not be
//! explicitly available.
//!
//! 1. The pool groups transactions from particular sender together
//!    and stores them ordered by `Scoring` within that group
//!    i.e. `HashMap<Sender, Vec<Transaction>>`.
//! 2. Additionaly we maintain the best and the worst transaction from each sender
//!    (by `Scoring` not `priority`) ordered by `priority`.
//!    It means that we can easily identify the best transaction inside the entire pool
//!    and the worst transaction.
//! 3. Whenever new transaction is inserted to the queue:
//!    - first check all the limits (overall, memory, per-sender)
//!    - retrieve all transactions from a sender
//!    - binary search for position to insert the transaction
//!    - decide if we are replacing existing transaction (3 outcomes: drop, replace, insert)
//!    - update best and worst transaction from that sender if affected
//! 4. Pending List construction:
//!    - Take the best transaction (by priority) from all senders to the List
//!    - Replace the transaction with next transaction (by ordering) from that sender (if any)
//!    - Repeat

#![warn(missing_docs)]

extern crate smallvec;
extern crate ethereum_types;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

#[cfg(test)]
mod tests;

mod error;
mod listener;
mod options;
mod pool;
mod ready;
mod status;
mod transactions;
mod verifier;

pub mod scoring;

pub use self::listener::{Listener, NoopListener};
pub use self::options::Options;
pub use self::pool::Pool;
pub use self::ready::{Ready, Readiness};
pub use self::scoring::Scoring;
pub use self::status::{LightStatus, Status};
pub use self::verifier::Verifier;

use std::fmt;

use ethereum_types::{H256, Address};

/// Already verified transaction that can be safely queued.
pub trait VerifiedTransaction: fmt::Debug {
	/// Transaction hash
	fn hash(&self) -> &H256;

	/// Memory usage
	fn mem_usage(&self) -> usize;

	/// Transaction sender
	fn sender(&self) -> &Address;

	/// Unique index of insertion (lower = older).
	fn insertion_id(&self) -> u64;
}
