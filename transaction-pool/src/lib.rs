extern crate smallvec;

#[macro_use]
extern crate error_chain;

mod error;
mod pool;

pub use self::error::Result;
pub use self::pool::Pool;

// Types
#[derive(Debug)]
pub struct UnverifiedTransaction;
#[derive(Debug)]
pub struct SignedTransaction;
#[derive(Debug)]
pub struct VerifiedTransaction {
	pub hash: H256
}
impl VerifiedTransaction {
	pub fn hash(&self) -> H256 {
		self.hash.clone()
	}

	pub fn mem_usage(&self) -> usize {
		self.hash.0 as usize
	}

	pub fn sender(&self) -> Address {
		Address::default()
	}
}
#[derive(Debug)]
pub struct PendingTransaction;
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address;
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct U256(u64);
impl From<u64> for U256 {
	fn from(x: u64) -> Self {
		U256(x)
	}
}
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct H256(u64);
impl From<u64> for H256 {
	fn from(x: u64) -> Self {
		H256(x)
	}
}

/// Main part of the transaction verification is decoupled from the pool
pub trait Verifier {
	fn verify_transaction(&self, tx: UnverifiedTransaction) -> Result<VerifiedTransaction>;

	fn nonce(&self, sender: &Address) -> U256;

	fn balance(&self, sender: &Address) -> U256;
}

pub struct NoopVerifier;
impl Verifier for NoopVerifier {
	fn verify_transaction(&self, _tx: UnverifiedTransaction) -> Result<VerifiedTransaction> {
		unimplemented!()
	}

	fn nonce(&self, _sender: &Address) -> U256 {
		unimplemented!()
	}

	fn balance(&self, _sender: &Address) -> U256 {
		unimplemented!()
	}
}

pub trait Listener {
	fn added(&mut self, _tx: &VerifiedTransaction, _old: Option<&VerifiedTransaction>) {}
	fn rejected(&mut self, _tx: &VerifiedTransaction) {}
	fn dropped(&mut self, _tx: &VerifiedTransaction) {}
	fn invalid(&mut self, _tx: &SignedTransaction) {}
	fn cancelled(&mut self, _tx: &PendingTransaction) {}
}

pub struct NoopListener;
impl Listener for NoopListener {}

#[cfg(test)]
mod tests {
    #[test]
    fn _works() {
        assert_eq!(2 + 2, 4);
    }
}
