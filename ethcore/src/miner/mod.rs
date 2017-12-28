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

#![warn(missing_docs)]

//! Miner module
//! Keeps track of transactions and mined block.
//!
//! Usage example:
//!
//! ```rust
//! extern crate ethcore_util as util;
//! extern crate ethcore;
//! use std::env;
//! use ethcore::ethereum;
//! use ethcore::client::{Client, ClientConfig};
//! use ethcore::miner::{Miner, MinerService};
//!
//! fn main() {
//!		let miner: Miner = Miner::with_spec(&ethereum::new_foundation(&env::temp_dir()));
//!		// get status
//!		assert_eq!(miner.status().transactions_in_pending_queue, 0);
//!
//!		// Check block for sealing
//!		//assert!(miner.sealing_block(&*client).lock().is_some());
//! }
//! ```

mod banning_queue;
mod external;
mod local_transactions;
mod miner;
mod service_transaction_checker;
mod transaction_queue;
mod work_notify;
mod stratum;

pub use self::external::{ExternalMiner, ExternalMinerService};

pub use self::miner::{Miner, MinerOptions, Banning, PendingSet, GasPricer, GasPriceCalibratorOptions, GasLimit};
pub use self::transaction_queue::{TransactionQueue, RemovalReason, TransactionDetailsProvider as TransactionQueueDetailsProvider,
	PrioritizationStrategy, AccountDetails, TransactionOrigin};
pub use self::local_transactions::{Status as LocalTransactionStatus};
pub use client::TransactionImportResult;
pub use self::work_notify::NotifyWork;
pub use self::stratum::{Stratum, Error as StratumError, Options as StratumOptions};

use std::collections::BTreeMap;
use bigint::prelude::U256;
use bigint::hash::H256;
use util::Address;
use bytes::Bytes;
use client::{MiningBlockChainClient};
use block::ClosedBlock;
use header::BlockNumber;
use receipt::{RichReceipt, Receipt};
use error::{Error};
use transaction::{UnverifiedTransaction, PendingTransaction};

/// Miner client API
pub trait MinerService : Send + Sync {

	/// Returns miner's status.
	fn status(&self) -> MinerStatus;

	/// Get the author that we will seal blocks as.
	fn author(&self) -> Address;

	/// Set the author that we will seal blocks as.
	fn set_author(&self, author: Address);

	/// Set info necessary to sign consensus messages.
	fn set_engine_signer(&self, address: Address, password: String) -> Result<(), ::account_provider::SignError>;

	/// Get the extra_data that we will seal blocks with.
	fn extra_data(&self) -> Bytes;

	/// Set the extra_data that we will seal blocks with.
	fn set_extra_data(&self, extra_data: Bytes);

	/// Get current minimal gas price for transactions accepted to queue.
	fn minimal_gas_price(&self) -> U256;

	/// Set minimal gas price of transaction to be accepted for mining.
	fn set_minimal_gas_price(&self, min_gas_price: U256);

	/// Get the lower bound of the gas limit we wish to target when sealing a new block.
	fn gas_floor_target(&self) -> U256;

	/// Get the upper bound of the gas limit we wish to target when sealing a new block.
	fn gas_ceil_target(&self) -> U256;

	// TODO: coalesce into single set_range function.
	/// Set the lower bound of gas limit we wish to target when sealing a new block.
	fn set_gas_floor_target(&self, target: U256);

	/// Set the upper bound of gas limit we wish to target when sealing a new block.
	fn set_gas_ceil_target(&self, target: U256);

	/// Get current transactions limit in queue.
	fn transactions_limit(&self) -> usize;

	/// Set maximal number of transactions kept in the queue (both current and future).
	fn set_transactions_limit(&self, limit: usize);

	/// Set maximum amount of gas allowed for any single transaction to mine.
	fn set_tx_gas_limit(&self, limit: U256);

	/// Imports transactions to transaction queue.
	fn import_external_transactions(&self, chain: &MiningBlockChainClient, transactions: Vec<UnverifiedTransaction>) ->
		Vec<Result<TransactionImportResult, Error>>;

	/// Imports own (node owner) transaction to queue.
	fn import_own_transaction(&self, chain: &MiningBlockChainClient, transaction: PendingTransaction) ->
		Result<TransactionImportResult, Error>;

	/// Returns hashes of transactions currently in pending
	fn pending_transactions_hashes(&self, best_block: BlockNumber) -> Vec<H256>;

	/// Removes all transactions from the queue and restart mining operation.
	fn clear_and_reset(&self, chain: &MiningBlockChainClient);

	/// Called when blocks are imported to chain, updates transactions queue.
	fn chain_new_blocks(&self, chain: &MiningBlockChainClient, imported: &[H256], invalid: &[H256], enacted: &[H256], retracted: &[H256]);

	/// PoW chain - can produce work package
	fn can_produce_work_package(&self) -> bool;

	/// New chain head event. Restart mining operation.
	fn update_sealing(&self, chain: &MiningBlockChainClient);

	/// Submit `seal` as a valid solution for the header of `pow_hash`.
	/// Will check the seal, but not actually insert the block into the chain.
	fn submit_seal(&self, chain: &MiningBlockChainClient, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error>;

	/// Get the sealing work package and if `Some`, apply some transform.
	fn map_sealing_work<F, T>(&self, chain: &MiningBlockChainClient, f: F) -> Option<T>
		where F: FnOnce(&ClosedBlock) -> T, Self: Sized;

	/// Query pending transactions for hash.
	fn transaction(&self, best_block: BlockNumber, hash: &H256) -> Option<PendingTransaction>;

	/// Removes transaction from the queue.
	/// NOTE: The transaction is not removed from pending block if mining.
	fn remove_pending_transaction(&self, chain: &MiningBlockChainClient, hash: &H256) -> Option<PendingTransaction>;

	/// Get a list of all pending transactions in the queue.
	fn pending_transactions(&self) -> Vec<PendingTransaction>;

	/// Get a list of all transactions that can go into the given block.
	fn ready_transactions(&self, best_block: BlockNumber, best_block_timestamp: u64) -> Vec<PendingTransaction>;

	/// Get a list of all future transactions.
	fn future_transactions(&self) -> Vec<PendingTransaction>;

	/// Get a list of local transactions with statuses.
	fn local_transactions(&self) -> BTreeMap<H256, LocalTransactionStatus>;

	/// Get a list of all pending receipts.
	fn pending_receipts(&self, best_block: BlockNumber) -> BTreeMap<H256, Receipt>;

	/// Get a particular reciept.
	fn pending_receipt(&self, best_block: BlockNumber, hash: &H256) -> Option<RichReceipt>;

	/// Returns highest transaction nonce for given address.
	fn last_nonce(&self, address: &Address) -> Option<U256>;

	/// Is it currently sealing?
	fn is_currently_sealing(&self) -> bool;

	/// Suggested gas price.
	fn sensible_gas_price(&self) -> U256;

	/// Suggested gas limit.
	fn sensible_gas_limit(&self) -> U256 { 21000.into() }
}

/// Mining status
#[derive(Debug)]
pub struct MinerStatus {
	/// Number of transactions in queue with state `pending` (ready to be included in block)
	pub transactions_in_pending_queue: usize,
	/// Number of transactions in queue with state `future` (not yet ready to be included in block)
	pub transactions_in_future_queue: usize,
	/// Number of transactions included in currently mined block
	pub transactions_in_pending_block: usize,
}
