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
//! An extensible and performant implementation of Ethereum Transaction Pool.
//! The pool stores ordered, verified transactions according to some pluggable
//! `Scoring` implementation.
//! The pool also allows you to construct a set of `pending` transactions according
//! to some notion of `Readiness` (pluggable).

#![deny(missing_docs)]

extern crate smallvec;
extern crate ethcore_bigint as bigint;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

mod error;
mod listener;
mod options;
mod pool;
mod ready;
mod status;
mod transactions;
mod verifier;

#[cfg(test)]
mod tests;

pub mod scoring;

pub use self::listener::{Listener, NoopListener};
pub use self::options::Options;
pub use self::pool::Pool;
pub use self::ready::{Ready, Readiness};
pub use self::scoring::Scoring;
pub use self::status::{LightStatus, Status};
pub use self::verifier::Verifier;

use std::sync::Arc;

use self::bigint::prelude::{H256, U256};
type Address = bigint::hash::H160;

/// Verified transaction
#[derive(Debug, PartialEq)]
pub struct VerifiedTransaction {
	/// hash
	pub hash: H256,
	/// nonce
	pub nonce: U256,
	/// gas_price
	pub gas_price: U256,
	/// gas
	pub gas: U256,
	/// sender
	pub sender: Address,
	/// insertion_id
	pub insertion_id: u64,
}
impl VerifiedTransaction {
	/// Transaction hash
	pub fn hash(&self) -> H256 {
		// TODO [ToDr] return a reference
		self.hash.clone()
	}

	/// Memory usage
	pub fn mem_usage(&self) -> usize {
		self.nonce.low_u64() as usize
	}

	/// Transaction sender
	pub fn sender(&self) -> Address {
		// TODO [ToDr] return a reference
		self.sender.clone()
	}
}

/// Shared transaction
pub type SharedTransaction = Arc<VerifiedTransaction>;
