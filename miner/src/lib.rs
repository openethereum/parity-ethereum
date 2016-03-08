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
#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate ethcore_util as util;
extern crate ethcore;
extern crate env_logger;
extern crate rayon;

mod miner;
mod transaction_queue;

use util::{Bytes, H256, Address};
use std::ops::*;
use std::sync::*;
use util::TimerToken;
use ethcore::block::*;
use ethcore::error::*;
use ethcore::client::{Client, BlockChainClient};
use ethcore::transaction::*;
use miner::Miner;

pub struct EthMiner {
	miner: Miner,
	/// Shared blockchain client. TODO: this should evetually become an IPC endpoint
	chain: Arc<Client>,
}

impl EthMiner {
	/// Creates and register protocol with the network service
	pub fn new(chain: Arc<Client>) -> Arc<EthMiner> {
		Arc::new(EthMiner {
			miner: Miner::new(),
			chain: chain,
		})
	}

	pub fn sealing_block(&self) -> &Mutex<Option<ClosedBlock>> {
		self.miner.sealing_block(self.chain.deref())
	}

	pub fn submit_seal(&self, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		self.miner.submit_seal(self.chain.deref(), pow_hash, seal)
	}

	/// Set the author that we will seal blocks as.
	pub fn set_author(&self, author: Address) {
		self.miner.set_author(author);
	}

	/// Set the extra_data that we will seal blocks with.
	pub fn set_extra_data(&self, extra_data: Bytes) {
		self.miner.set_extra_data(extra_data);
	}

	pub fn import_transactions(&self, transactions: Vec<SignedTransaction>) {
		let chain = self.chain.deref();
		let fetch_latest_nonce = |a : &Address| chain.nonce(a);

		self.miner.import_transactions(transactions, fetch_latest_nonce);
	}

	pub fn chain_new_blocks(&self, good: &[H256], bad: &[H256], retracted: &[H256]) {
		let mut chain = self.chain.deref();
		self.miner.chain_new_blocks(chain, good, bad, retracted);
	}
}
