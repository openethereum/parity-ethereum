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
