// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Transaction Pool state client.
//!
//! `Client` encapsulates all external data required for the verifaction and readiness.
//! It includes any Ethereum state parts required for checking the transaction and
//! any consensus-required structure of the transaction.

use std::fmt;

use ethereum_types::{U256, H256, H160 as Address};
use transaction;

/// Account Details
#[derive(Debug, Clone)]
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
	/// Service transaction (allowed by a contract to have gas_price=0)
	Service,
}

/// Verification client.
pub trait Client: fmt::Debug + Sync {
	/// Is transaction with given hash already in the blockchain?
	fn transaction_already_included(&self, hash: &H256) -> bool;

	/// Structurarily verify given transaction.
	fn verify_transaction(&self, tx: transaction::UnverifiedTransaction)
		-> Result<transaction::SignedTransaction, transaction::Error>;

	/// Estimate minimal gas requirurement for given transaction.
	fn required_gas(&self, tx: &transaction::Transaction) -> U256;

	/// Fetch account details for given sender.
	fn account_details(&self, address: &Address) -> AccountDetails;

	/// Classify transaction (check if transaction is filtered by some contracts).
	fn transaction_type(&self, tx: &transaction::SignedTransaction) -> TransactionType;

	/// Performs pre-validation of RLP decoded transaction
	fn decode_transaction(&self, transaction: &[u8])
		-> Result<transaction::UnverifiedTransaction, transaction::Error>;
}

/// State nonce client
pub trait NonceClient: fmt::Debug + Sync {
	/// Fetch only account nonce for given sender.
	fn account_nonce(&self, address: &Address) -> U256;
}
