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
//!
//!		let miner: Miner = Miner::default();
//!		// get status
//!		assert_eq!(miner.status().transactions_in_pending_queue, 0);
//!
//!		// Check block for sealing
//!		//assert!(miner.sealing_block(client.deref()).lock().unwrap().is_some());
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

pub use transaction_queue::{TransactionQueue, AccountDetails};
pub use miner::{Miner};

use util::{H256, Address, FixedHash, Bytes};
use ethcore::client::{BlockChainClient};
use ethcore::block::{ClosedBlock};
use ethcore::error::{Error};
use ethcore::transaction::SignedTransaction;

/// Miner client API
pub trait MinerService : Send + Sync {

	/// Returns miner's status.
	fn status(&self) -> MinerStatus;

	/// Get the author that we will seal blocks as.
	fn author(&self) -> Address { Address::zero() }

	/// Get the extra_data that we will seal blocks wuth.
	fn extra_data(&self) -> Bytes { vec![] }

	/// Imports transactions to transaction queue.
	fn import_transactions<T>(&self, transactions: Vec<SignedTransaction>, fetch_account: T) -> Vec<Result<(), Error>>
		where T: Fn(&Address) -> AccountDetails;

	/// Returns hashes of transactions currently in pending
	fn pending_transactions_hashes(&self) -> Vec<H256>;

	/// Removes all transactions from the queue and restart mining operation.
	fn clear_and_reset(&self, chain: &BlockChainClient);

	/// Called when blocks are imported to chain, updates transactions queue.
	fn chain_new_blocks(&self, chain: &BlockChainClient, imported: &[H256], invalid: &[H256], enacted: &[H256], retracted: &[H256]);

	/// New chain head event. Restart mining operation.
	fn update_sealing(&self, chain: &BlockChainClient);

	/// Submit `seal` as a valid solution for the header of `pow_hash`.
	/// Will check the seal, but not actually insert the block into the chain.
	fn submit_seal(&self, chain: &BlockChainClient, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error>;

	/// Get the sealing work package and if `Some`, apply some transform.
	fn map_sealing_work<F, T>(&self, chain: &BlockChainClient, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T;

	/// Query pending transactions for hash
	fn transaction(&self, hash: &H256) -> Option<SignedTransaction>;
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
