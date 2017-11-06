use {UnverifiedTransaction, VerifiedTransaction};

/// Main part of the transaction verification is decoupled from the pool
pub trait Verifier {
	type Error;

	fn verify_transaction(&self, tx: UnverifiedTransaction) -> Result<VerifiedTransaction, Self::Error>;
}
