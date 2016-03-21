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

use rayon::prelude::*;
use std::ops::Deref;
use std::sync::{Mutex, RwLock, Arc};
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::collections::HashSet;

use util::{H256, U256, Address, Bytes, Uint};
use ethcore::engine::Engine;
use ethcore::block::{ClosedBlock, IsBlock};
use ethcore::error::{Error, ExecutionError};
use ethcore::transaction::SignedTransaction;
use super::{MinerService, MinerStatus, TransactionQueue, AccountDetails, MinerBlockChain};

/// Keeps track of transactions using priority queue and holds currently mined block.
pub struct Miner<C: MinerBlockChain> {
	engine: Arc<Box<Engine>>,
	chain: Arc<C>,
	transaction_queue: Mutex<TransactionQueue>,

	// for sealing...
	sealing_enabled: AtomicBool,
	sealing_block_last_request: Mutex<u64>,
	sealing_block: Mutex<Option<ClosedBlock>>,
	gas_floor_target: RwLock<U256>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,

}

impl<C : MinerBlockChain> Miner<C> {
	/// Creates new instance of miner
	pub fn new(engine: Arc<Box<Engine>>, chain: Arc<C>) -> Arc<Miner<C>> {
		Arc::new(Miner {
			engine: engine,
			chain: chain,
			transaction_queue: Mutex::new(TransactionQueue::new()),
			sealing_enabled: AtomicBool::new(false),
			sealing_block_last_request: Mutex::new(0),
			sealing_block: Mutex::new(None),
			gas_floor_target: RwLock::new(U256::zero()),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
		})
	}

	/// Get the author that we will seal blocks as.
	fn author(&self) -> Address {
		*self.author.read().unwrap()
	}

	/// Get the extra_data that we will seal blocks wuth.
	fn extra_data(&self) -> Bytes {
		self.extra_data.read().unwrap().clone()
	}

	/// Get the extra_data that we will seal blocks wuth.
	fn gas_floor_target(&self) -> U256 {
		*self.gas_floor_target.read().unwrap()
	}

	/// Set the author that we will seal blocks as.
	pub fn set_author(&self, author: Address) {
		*self.author.write().unwrap() = author;
	}

	/// Set the extra_data that we will seal blocks with.
	pub fn set_extra_data(&self, extra_data: Bytes) {
		*self.extra_data.write().unwrap() = extra_data;
	}

	/// Set the gas limit we wish to target when sealing a new block.
	pub fn set_gas_floor_target(&self, target: U256) {
		*self.gas_floor_target.write().unwrap() = target;
	}

	/// Set minimal gas price of transaction to be accepted for mining.
	pub fn set_minimal_gas_price(&self, min_gas_price: U256) {
		self.transaction_queue.lock().unwrap().set_minimal_gas_price(min_gas_price);
	}

	/// Prepares new block for sealing including top transactions from queue.
	fn prepare_sealing(&self) {
		let transactions = self.transaction_queue.lock().unwrap().top_transactions();

		let mut b = self.chain.open_block(
			self.author(),
			self.gas_floor_target(),
			self.extra_data()
		);
		// Add transactions
		let block_number = b.block().header().number();
		let min_tx_gas = U256::from(self.engine.schedule(&b.env_info()).tx_gas);
		let mut invalid_transactions = HashSet::new();

		for tx in transactions {
			// Push transaction to block
			let hash = tx.hash();
			let import = b.push_transaction(tx, None);

			match import {
				Err(Error::Execution(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, .. })) => {
					trace!(target: "miner", "Skipping adding transaction to block because of gas limit: {:?}", hash);
					// Exit early if gas left is smaller then min_tx_gas
					if gas_limit - gas_used < min_tx_gas {
						break;
					}
				},
				Err(e) => {
					invalid_transactions.insert(hash);
					trace!(target: "miner",
						   "Error adding transaction to block: number={}. transaction_hash={:?}, Error: {:?}",
						   block_number, hash, e);
				},
				_ => {}
			}
		}

		let mut queue = self.transaction_queue.lock().unwrap();
		queue.remove_all(
			&invalid_transactions.into_iter().collect::<Vec<H256>>(),
			|a: &Address| self.chain.account_details(a)
		);

		// And close
		let b = b.close();
		trace!(target: "miner", "Sealing: number={}, hash={}, diff={}",
			   b.block().header().number(),
			   b.hash(),
			   b.block().header().difficulty()
		);

		*self.sealing_block.lock().unwrap() = Some(b);
	}

	fn update_gas_limit(&self) {
		let gas_limit = self.chain.best_block_gas_limit();
		let mut queue = self.transaction_queue.lock().unwrap();
		queue.set_gas_limit(gas_limit);
	}

}

const SEALING_TIMEOUT_IN_BLOCKS : u64 = 5;

impl<C: MinerBlockChain> MinerService for Miner<C> {

	fn clear_and_reset(&self) {
		self.transaction_queue.lock().unwrap().clear();
		self.update_sealing();
	}

	fn status(&self) -> MinerStatus {
		let status = self.transaction_queue.lock().unwrap().status();
		let block = self.sealing_block.lock().unwrap();
		MinerStatus {
			transactions_in_pending_queue: status.pending,
			transactions_in_future_queue: status.future,
			transactions_in_pending_block: block.as_ref().map_or(0, |b| b.transactions().len()),
		}
	}

	fn import_transactions(&self, transactions: Vec<SignedTransaction>) -> Vec<Result<(), Error>> {
		let mut transaction_queue = self.transaction_queue.lock().unwrap();
		transaction_queue.add_all(transactions, |a: &Address| self.chain.account_details(a))
	}

	fn pending_transactions_hashes(&self) -> Vec<H256> {
		let transaction_queue = self.transaction_queue.lock().unwrap();
		transaction_queue.pending_hashes()
	}

	fn update_sealing(&self) {
		let should_disable_sealing = {
			let current_no = self.chain.best_block_number();
			let last_request = self.sealing_block_last_request.lock().unwrap();
			let is_greater = current_no > *last_request;
			is_greater && current_no - *last_request > SEALING_TIMEOUT_IN_BLOCKS
		};

		if should_disable_sealing {
			self.sealing_enabled.store(false, atomic::Ordering::Relaxed);
			*self.sealing_block.lock().unwrap() = None;
		} else if self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			self.prepare_sealing();
		}
	}

	fn sealing_block(&self) -> &Mutex<Option<ClosedBlock>> {
		if self.sealing_block.lock().unwrap().is_none() {
			self.sealing_enabled.store(true, atomic::Ordering::Relaxed);

			self.prepare_sealing();
		}
		*self.sealing_block_last_request.lock().unwrap() = self.chain.best_block_number();
		&self.sealing_block
	}

	fn submit_seal(&self, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		let mut maybe_b = self.sealing_block.lock().unwrap();
		match *maybe_b {
			Some(ref b) if b.hash() == pow_hash => {}
			_ => { return Err(Error::PowHashInvalid); }
		}

		let block = maybe_b.take().unwrap();

		match block.try_seal(self.engine.deref().deref(), seal) {
			Err(old) => {
				*maybe_b = Some(old);
				Err(Error::PowInvalid)
			}
			Ok(sealed) => {
				// TODO: commit DB from `sealed.drain` and make a VerifiedBlock to skip running the transactions twice.
				try!(self.chain.import_block(sealed.rlp_bytes()));
				Ok(())
			}
		}
	}

	fn chain_new_blocks(&self, imported: &[H256], invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		// First update gas limit in transaction queue
		self.update_gas_limit();

		// Then import all transactions...
		{
			let out_of_chain = retracted
				.par_iter()
				.map(|h: &H256| self.chain.block_transactions(h));
			out_of_chain.for_each(|txs| {
				// populate sender
				for tx in &txs {
					let _sender = tx.sender();
				}
				let mut transaction_queue = self.transaction_queue.lock().unwrap();
				let _ = transaction_queue.add_all(txs, |a: &Address| self.chain.account_details(a));
			});
		}

		// ...and after that remove old ones
		{
			let in_chain = {
				let mut in_chain = HashSet::new();
				in_chain.extend(imported);
				in_chain.extend(enacted);
				in_chain.extend(invalid);
				in_chain
					.into_iter()
					.collect::<Vec<H256>>()
			};

			let in_chain = in_chain
				.par_iter()
				.map(|h: &H256| self.chain.block_transactions(h));

			in_chain.for_each(|txs| {
				let hashes = txs.iter().map(|tx| tx.hash()).collect::<Vec<H256>>();
				let mut transaction_queue = self.transaction_queue.lock().unwrap();
				transaction_queue.remove_all(&hashes, |a: &Address| self.chain.account_details(a));
			});
		}

		self.update_sealing();
	}
}

#[cfg(test)]
mod tests {

	use std::sync::Arc;

	use MinerService;
	use super::{Miner};
	use ethcore::client::{TestBlockChainClient, EachBlockWith};

	// TODO [ToDr] To uncomment client is cleaned from mining stuff.
	#[ignore]
	#[test]
	fn should_prepare_block_to_seal() {
		// given
		let engine = unimplemented!();
		let client = Arc::new(TestBlockChainClient::default());
		let miner = Miner::new(engine, client);

		// when
		let res = miner.sealing_block();

		// then
		assert!(res.lock().unwrap().is_some(), "Expected closed block");
	}

	#[test]
	fn should_reset_seal_after_couple_of_blocks() {
		// given
		let engine = unimplemented!();
		let client = Arc::new(TestBlockChainClient::default());
		let miner = Miner::new(engine, client.clone());
		let res = miner.sealing_block();
		// TODO [ToDr] Uncomment after fixing TestBlockChainClient
		// assert!(res.lock().unwrap().is_some(), "Expected closed block");

		// when
		client.add_blocks(10, EachBlockWith::Uncle);

		// then
		assert!(res.lock().unwrap().is_none(), "Expected to remove sealed block");
	}
    //
	// #[test]
	// fn can_mine() {
	// 	let dummy_blocks = get_good_dummy_block_seq(2);
	// 	let client_result = get_test_client_with_blocks(vec![dummy_blocks[0].clone()]);
	// 	let client = client_result.reference();
    //
	// 	let b = client.prepare_sealing(Address::default(), x!(31415926), vec![], vec![]).unwrap();
    //
	// 	assert_eq!(*b.block().header().parent_hash(), BlockView::new(&dummy_blocks[0]).header_view().sha3());
	// 	assert!(client.try_seal(b, vec![]).is_ok());
	// }
}
