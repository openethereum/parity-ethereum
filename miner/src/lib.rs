// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
#![cfg_attr(all(nightly, feature="dev"), feature(plugin))]
#![cfg_attr(all(nightly, feature="dev"), plugin(clippy))]

//! Miner module
//! Keeps track of transactions and mined block.
//!
//! Usage example:
//!
//! ```rust
//! extern crate ethcore_util as util;
//! extern crate ethcore;
//! extern crate ethminer;
//! use std::ops::Deref;
//! use std::env;
//! use std::sync::Arc;
//! use util::network::{NetworkService, NetworkConfiguration};
//! use ethcore::client::{Client, ClientConfig, BlockChainClient};
//! use ethcore::ethereum;
//! use ethminer::{Miner, MinerService};
//!
//! fn main() {
//! 	let mut service = NetworkService::start(NetworkConfiguration::new()).unwrap();
//! 	let dir = env::temp_dir();
//! 	let client = Client::new(ClientConfig::default(), ethereum::new_frontier(), &dir, service.io().channel()).unwrap();
//! 	let force_sealing = true; // Enable sealing even if no-one asks for work.
//!
//!		let miner = Miner::new(client, true);
//!
//!		// get status
//!		assert_eq!(miner.status().transactions_in_pending_queue, 0);
//!
//!		// Check block for sealing
//!		assert!(miner.map_sealing_work(|_| ()).is_some());
//! }
//! ```


#[macro_use]
extern crate log;
#[macro_use]
extern crate ethcore_util as util;
extern crate ethcore;
extern crate env_logger;
extern crate rayon;

mod miner;
mod transaction_queue;
mod client;

pub use transaction_queue::{TransactionQueue, AccountDetails};
pub use miner::{Miner};

use util::{H256, U256, Address, Bytes};
use ethcore::engine::Engine;
use ethcore::block::{ClosedBlock, OpenBlock};
use ethcore::error::{Error, ImportResult};
use ethcore::transaction::SignedTransaction;

/// Miner API
pub trait MinerService : Send + Sync {

	/// Returns miner's status.
	fn status(&self) -> MinerStatus;

	/// Get the author that we will seal blocks as.
	fn author(&self) -> Address;

	/// Set the author that we will seal blocks as.
	fn set_author(&self, author: Address);

	/// Get the extra_data that we will seal blocks with.
	fn extra_data(&self) -> Bytes;

	/// Set the extra_data that we will seal blocks with.
	fn set_extra_data(&self, extra_data: Bytes);

	/// Get current minimal gas price for transactions accepted to queue.
	fn minimal_gas_price(&self) -> U256;

	/// Set minimal gas price of transaction to be accepted for mining.
	fn set_minimal_gas_price(&self, min_gas_price: U256);

	/// Get the gas limit we wish to target when sealing a new block.
	fn gas_floor_target(&self) -> U256;

	/// Set the gas limit we wish to target when sealing a new block.
	fn set_gas_floor_target(&self, target: U256);

	/// Imports transactions to transaction queue.
	fn import_transactions(&self, transactions: Vec<SignedTransaction>) -> Vec<Result<(), Error>>;

	/// Returns hashes of transactions currently in pending
	fn pending_transactions_hashes(&self) -> Vec<H256>;

	/// Removes all transactions from the queue and restart mining operation.
	fn clear_and_reset(&self);

	/// Called when blocks are imported to chain, updates transactions queue.
	fn chain_new_blocks(&self, imported: &[H256], invalid: &[H256], enacted: &[H256], retracted: &[H256]);

	/// Get the sealing work package and if `Some`, apply some transform.
	fn map_sealing_work<F, T>(&self, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T;

	/// Submit `seal` as a valid solution for the header of `pow_hash`.
	fn submit_seal(&self, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error>;

	fn update_sealing(&self);

	/// Query pending transactions for hash.
	fn transaction(&self, hash: &H256) -> Option<SignedTransaction>;

	/// Get a list of all pending transactions.
	fn pending_transactions(&self) -> Vec<SignedTransaction>;

	/// Returns highest transaction nonce for given address.
	fn last_nonce(&self, address: &Address) -> Option<U256>;

	/// Suggested gas price.
	fn sensible_gas_price(&self) -> U256 { x!(20000000000u64) }

	/// Suggested gas limit.
	fn sensible_gas_limit(&self) -> U256 { x!(21000) }
}

/// `BlockChainClient` requirements for mining
pub trait MinerBlockChain : Send + Sync {
	/// Returns new `OpenBlock` given the parameters. `OpenBlock` can later be sealed and imported into the blockchain.
	fn open_block(&self, author: Address, gas_floor_target: U256, extra_data: Bytes) -> Option<OpenBlock>;

	/// Imports sealed block to `BlockChain`.
	fn import_block(&self, bytes: Bytes) -> ImportResult;

	/// Returns all transactions included in block identified by hash.
	fn block_transactions(&self, hash: &H256) -> Vec<SignedTransaction>;

	/// Returns gas limit of current best block.
	fn best_block_gas_limit(&self) -> U256;

	/// Returns number of current best block.
	fn best_block_number(&self) -> u64;

	/// Returns hash of current best block.
	fn best_block_hash(&self) -> H256;

	/// Returns details (balance & nonce) for account under given `address`.
	fn account_details(&self, address: &Address) -> AccountDetails;

	/// Returns current instance of engine.
	fn engine(&self) -> &Engine;
}

/// Mining status
pub struct MinerStatus {
	/// Number of transactions in queue with state `pending` (ready to be included in block)
	pub transactions_in_pending_queue: usize,
	/// Number of transactions in queue with state `future` (not yet ready to be included in block)
	pub transactions_in_future_queue: usize,
	/// Number of transactions included in currently mined block
	pub transactions_in_pending_block: usize,
}
