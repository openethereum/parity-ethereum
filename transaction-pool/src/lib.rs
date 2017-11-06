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

// Types
#[derive(Debug)]
pub struct UnverifiedTransaction;
#[derive(Debug, PartialEq)]
pub struct VerifiedTransaction {
	pub hash: H256,
	pub nonce: U256,
	pub gas_price: U256,
	pub gas: U256,
	pub sender: Address,
	pub insertion_id: u64,
}
impl VerifiedTransaction {
	pub fn hash(&self) -> H256 {
		self.hash.clone()
	}

	pub fn mem_usage(&self) -> usize {
		self.nonce.low_u64() as usize
	}

	pub fn sender(&self) -> Address {
		self.sender.clone()
	}
}

pub type SharedTransaction = Arc<VerifiedTransaction>;
