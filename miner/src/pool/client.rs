//! Transaction Pool state client.
//!
//! `Client` encapsulates all external data required for the verifaction and readiness.
//! It includes any Ethereum state parts required for checking the transaction and
//! any consensus-required structure of the transaction.

use std::fmt;

use ethereum_types::{U256, H256, H160 as Address};
use transaction;

/// Account Details
#[derive(Debug)]
pub struct AccountDetails {
	/// Current account nonce
	pub nonce: U256,
	/// Current account balance
	pub balance: U256,
	/// Is this account a local account?
	pub is_local: bool,
}

/// Transaction type
#[derive(Debug, PartialEq)]
pub enum TransactionType {
	/// Regular transaction
	Regular,
	/// Service transaction (allowed by a contract)
	Service,
	/// Denied transaction (disallowed by a contract)
	Denied,
}

/// State client.
pub trait Client: fmt::Debug {
	/// Is transaction with given hash already in the blockchain?
	fn transaction_already_included(&self, hash: &H256) -> bool;

	/// Structurarily verify given transaction.
	fn verify_transaction(&self, tx: transaction::UnverifiedTransaction)
		-> Result<transaction::SignedTransaction, transaction::Error>;

	/// Fetch account details for given sender.
	fn account_details(&self, address: &Address) -> AccountDetails;

	/// Fetch only account nonce for given sender.
	fn account_nonce(&self, address: &Address) -> U256;

	/// Estimate minimal gas requirurement for given transaction.
	fn required_gas(&self, tx: &transaction::SignedTransaction) -> U256;

	/// Classify transaction (check if transaction is filtered by some contracts).
	fn transaction_type(&self, tx: &transaction::SignedTransaction) -> TransactionType;
}
