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

//! Transaction Verifier
//!
//! Responsible for verifying a transaction before importing to the pool.
//! Should make sure that the transaction is structuraly valid.
//!
//! May have some overlap with `Readiness` since we don't want to keep around
//! stalled transactions.

use std::cmp;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicUsize};

use ethereum_types::{U256, H256};
use rlp::Encodable;
use transaction;
use txpool;

use super::client::{Client, TransactionType};
use super::VerifiedTransaction;

/// Verification options.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
	/// Minimal allowed gas price.
	pub minimal_gas_price: U256,
	/// Current block gas limit.
	pub block_gas_limit: U256,
	/// Maximal gas limit for a single transaction.
	pub tx_gas_limit: U256,
}

#[cfg(test)]
impl Default for Options {
	fn default() -> Self {
		Options {
			minimal_gas_price: 0.into(),
			block_gas_limit: U256::max_value(),
			tx_gas_limit: U256::max_value(),
		}
	}
}

/// Transaction to verify.
pub enum Transaction {
	/// Fresh, never verified transaction.
	///
	/// We need to do full verification of such transactions
	Unverified(transaction::UnverifiedTransaction),

	/// Transaction from retracted block.
	///
	/// We could skip some parts of verification of such transactions
	Retracted(transaction::UnverifiedTransaction),

	/// Locally signed or retracted transaction.
	///
	/// We can skip consistency verifications and just verify readiness.
	Local(transaction::PendingTransaction),
}

impl Transaction {
	fn hash(&self) -> H256 {
		match *self {
			Transaction::Unverified(ref tx) => tx.hash(),
			Transaction::Retracted(ref tx) => tx.hash(),
			Transaction::Local(ref tx) => tx.hash(),
		}
	}

	fn gas(&self) -> &U256 {
		match *self {
			Transaction::Unverified(ref tx) => &tx.gas,
			Transaction::Retracted(ref tx) => &tx.gas,
			Transaction::Local(ref tx) => &tx.gas,
		}
	}


	fn gas_price(&self) -> &U256 {
		match *self {
			Transaction::Unverified(ref tx) => &tx.gas_price,
			Transaction::Retracted(ref tx) => &tx.gas_price,
			Transaction::Local(ref tx) => &tx.gas_price,
		}
	}

	fn transaction(&self) -> &transaction::Transaction {
		match *self {
			Transaction::Unverified(ref tx) => &*tx,
			Transaction::Retracted(ref tx) => &*tx,
			Transaction::Local(ref tx) => &*tx,
		}
	}

	fn is_local(&self) -> bool {
		match *self {
			Transaction::Local(..) => true,
			_ => false,
		}
	}

	fn is_retracted(&self) -> bool {
		match *self {
			Transaction::Retracted(..) => true,
			_ => false,
		}
	}
}

/// Transaction verifier.
///
/// Verification can be run in parallel for all incoming transactions.
#[derive(Debug)]
pub struct Verifier<C> {
	client: C,
	options: Options,
	id: Arc<AtomicUsize>,
}

impl<C> Verifier<C> {
	/// Creates new transaction verfier with specified options.
	pub fn new(client: C, options: Options, id: Arc<AtomicUsize>) -> Self {
		Verifier {
			client,
			options,
			id,
		}
	}
}

impl<C: Client> txpool::Verifier<Transaction> for Verifier<C> {
	type Error = transaction::Error;
	type VerifiedTransaction = VerifiedTransaction;

	fn verify_transaction(&self, tx: Transaction) -> Result<Self::VerifiedTransaction, Self::Error> {
		// The checks here should be ordered by cost/complexity.
		// Cheap checks should be done as early as possible to discard unneeded transactions early.

		let hash = tx.hash();

		if self.client.transaction_already_included(&hash) {
			trace!(target: "txqueue", "[{:?}] Rejected tx already in the blockchain", hash);
			bail!(transaction::Error::AlreadyImported)
		}

		let gas_limit = cmp::min(self.options.tx_gas_limit, self.options.block_gas_limit);
		if tx.gas() > &gas_limit {
			debug!(
				target: "txqueue",
				"[{:?}] Dropping transaction above gas limit: {} > min({}, {})",
				hash,
				tx.gas(),
				self.options.block_gas_limit,
				self.options.tx_gas_limit,
			);
			bail!(transaction::Error::GasLimitExceeded {
				limit: gas_limit,
				got: *tx.gas(),
			});
		}

		let minimal_gas = self.client.required_gas(tx.transaction());
		if tx.gas() < &minimal_gas {
			trace!(target: "txqueue",
				"[{:?}] Dropping transaction with insufficient gas: {} < {}",
				hash,
				tx.gas(),
				minimal_gas,
			);

			bail!(transaction::Error::InsufficientGas {
				minimal: minimal_gas,
				got: *tx.gas(),
			})
		}

		let is_own = tx.is_local();
		// Quick exit for non-service transactions
		if tx.gas_price() < &self.options.minimal_gas_price
			&& !tx.gas_price().is_zero()
			&& !is_own
		{
			trace!(
				target: "txqueue",
				"[{:?}] Rejected tx below minimal gas price threshold: {} < {}",
				hash,
				tx.gas_price(),
				self.options.minimal_gas_price,
			);
			bail!(transaction::Error::InsufficientGasPrice {
				minimal: self.options.minimal_gas_price,
				got: *tx.gas_price(),
			});
		}

		// Some more heavy checks below.
		// Actually recover sender and verify that transaction
		let is_retracted = tx.is_retracted();
		let transaction = match tx {
			Transaction::Retracted(tx) | Transaction::Unverified(tx) => match self.client.verify_transaction(tx) {
				Ok(signed) => signed.into(),
				Err(err) => {
					debug!(target: "txqueue", "[{:?}] Rejected tx {:?}", hash, err);
					bail!(err)
				},
			},
			Transaction::Local(tx) => tx,
		};

		// Verify RLP payload
		if let Err(err) = self.client.decode_transaction(&transaction.rlp_bytes()) {
			debug!(target: "txqueue", "[{:?}] Rejected transaction's rlp payload", err);
			bail!(err)
		}

		let sender = transaction.sender();
		let account_details = self.client.account_details(&sender);

		if transaction.gas_price < self.options.minimal_gas_price {
			let transaction_type = self.client.transaction_type(&transaction);
			if let TransactionType::Service = transaction_type {
				debug!(target: "txqueue", "Service tx {:?} below minimal gas price accepted", hash);
			} else if is_own || account_details.is_local {
				info!(target: "own_tx", "Local tx {:?} below minimal gas price accepted", hash);
			} else {
				trace!(
					target: "txqueue",
					"[{:?}] Rejected tx below minimal gas price threshold: {} < {}",
					hash,
					transaction.gas_price,
					self.options.minimal_gas_price,
				);
				bail!(transaction::Error::InsufficientGasPrice {
					minimal: self.options.minimal_gas_price,
					got: transaction.gas_price,
				});
			}
		}

		let cost = transaction.value + transaction.gas_price * transaction.gas;
		if account_details.balance < cost {
			debug!(
				target: "txqueue",
				"[{:?}] Rejected tx with not enough balance: {} < {}",
				hash,
				account_details.balance,
				cost,
			);
			bail!(transaction::Error::InsufficientBalance {
				cost: cost,
				balance: account_details.balance,
			});
		}

		if transaction.nonce < account_details.nonce {
			debug!(
				target: "txqueue",
				"[{:?}] Rejected tx with old nonce ({} < {})",
				hash,
				transaction.nonce,
				account_details.nonce,
			);
			bail!(transaction::Error::Old);
		}

		let priority = match (is_own || account_details.is_local, is_retracted) {
			(true, _) => super::Priority::Local,
			(false, false) => super::Priority::Regular,
			(false, true) => super::Priority::Retracted,
		};
		Ok(VerifiedTransaction {
			transaction,
			priority,
			hash,
			sender,
			insertion_id: self.id.fetch_add(1, atomic::Ordering::AcqRel),
		})
	}
}
