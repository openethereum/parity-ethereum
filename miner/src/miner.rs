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
use std::sync::{Mutex, RwLock, Arc};
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::collections::HashSet;

use util::{H256, U256, Address, Bytes, Uint, UsingQueue};
use ethcore::engine::Engine;
use ethcore::block::{ClosedBlock, OpenBlock, IsBlock};
use ethcore::error::{Error, ExecutionError, TransactionError};
use ethcore::transaction::SignedTransaction;
use super::{MinerService, MinerStatus, TransactionQueue, MinerBlockChain};

/// Keeps track of transactions using priority queue and holds currently mined block.
pub struct Miner<C: MinerBlockChain> {
	chain: Arc<C>,
	transaction_queue: Mutex<TransactionQueue>,
	// for sealing...
	force_sealing: bool,
	sealing_enabled: AtomicBool,
	sealing_block_last_request: Mutex<u64>,
	sealing_work: Mutex<UsingQueue<ClosedBlock>>,
	gas_floor_target: RwLock<U256>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,
}

impl<C : MinerBlockChain> Miner<C> {
	/// Creates new instance of miner
	pub fn new(chain: Arc<C>, force_sealing: bool) -> Arc<Miner<C>> {
		Arc::new(Miner {
			chain: chain,
			transaction_queue: Mutex::new(TransactionQueue::new()),
			force_sealing: force_sealing,
			sealing_enabled: AtomicBool::new(false),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(UsingQueue::new(5)),
			gas_floor_target: RwLock::new(U256::zero()),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
		})
	}

	/// Prepares new block for sealing including top transactions from queue.
	#[cfg_attr(feature="dev", allow(match_same_arms))]
	fn prepare_sealing(&self) {
		trace!(target: "miner", "prepare_sealing: entering");
		let transactions = self.transaction_queue.lock().unwrap().top_transactions();
		let mut sealing_work = self.sealing_work.lock().unwrap();
		let best_hash = self.chain.best_block_hash();

		// check to see if last ClosedBlock in would_seals is actually same parent block.
		// if so
		//   duplicate, re-open and push any new transactions.
		//   if at least one was pushed successfully, close and enqueue new ClosedBlock;
		//   otherwise, leave everything alone.
		// otherwise, author a fresh block.
		let mut block = match sealing_work.pop_if(|b| b.block().fields().header.parent_hash() == &best_hash) {
			Some(old_block) => {
				trace!(target: "miner", "Already have previous work; updating and returning");
				old_block.reopen(self.chain.engine())
			}
			None => {
				// block not found - create it.
				trace!(target: "miner", "No existing work - making new block");
				let block = self.chain.open_block(self.author(), self.gas_floor_target(), self.extra_data());
				match block {
					None => {
						trace!(
							target: "miner",
							"prepare_sealing: couldn't open block, leaving (last={:?})",
							sealing_work.peek_last_ref().map(|b| b.block().fields().header.hash())
						);
						return;
					},
					Some(block) => {
						block
					},
				}
			}
		};

		let min_tx_gas = U256::from(self.chain.engine().schedule(&block.env_info()).tx_gas);
		// TODO: If block has been reopened push new uncles, too.
		let invalid_transactions = Self::push_transactions_to_block(&mut block, transactions, min_tx_gas);

		// And close
		let block = block.close();
		trace!(target: "miner", "Sealing: number={}, hash={}, diff={}",
			   block.header().number(),
			   block.hash(),
			   block.header().difficulty()
			  );

		// Remove invalid transactions from queue
		let mut queue = self.transaction_queue.lock().unwrap();
		let fetch_account = |a: &Address| self.chain.account_details(a);
		for hash in invalid_transactions.into_iter() {
			queue.remove_invalid(&hash, &fetch_account);
		}

		// And save the block
		let hash = block.block().fields().header.hash();
		if sealing_work.peek_last_ref().map_or(true, |pb| pb.block().fields().header.hash() != hash) {
			trace!(target: "miner", "Pushing a new, refreshed or borrowed pending {}...", hash);
			sealing_work.push(block);
		}
		trace!(target: "miner", "prepare_sealing: leaving (last={:?})", sealing_work.peek_last_ref().map(|b| b.block().fields().header.hash()));
	}

	fn update_gas_limit(&self) {
		let gas_limit = self.chain.best_block_gas_limit();
		let mut queue = self.transaction_queue.lock().unwrap();
		queue.set_gas_limit(gas_limit);
	}

	fn push_transactions_to_block(block: &mut OpenBlock, transactions: Vec<SignedTransaction>, min_tx_gas: U256) -> HashSet<H256> {
		let block_number = block.block().header().number();
		let mut invalid_transactions = HashSet::new();

		for tx in transactions {
			// Push transaction to block
			let hash = tx.hash();
			let import = block.push_transaction(tx, None);

			match import {
				Err(Error::Execution(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, .. })) => {
					trace!(target: "miner", "Skipping adding transaction to block because of gas limit: {:?}", hash);
					// Exit early if gas left is smaller then min_tx_gas
					if gas_limit - gas_used < min_tx_gas {
						break;
					}
				},
				Err(Error::Transaction(TransactionError::AlreadyImported)) => {}	// already have transaction - ignore
				Err(e) => {
					invalid_transactions.insert(hash);
					trace!(target: "miner",
						   "Error adding transaction to block: number={}. transaction_hash={:?}, Error: {:?}",
						   block_number, hash, e);
				},
				_ => {}
			}
		}

		invalid_transactions
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
		let sealing_work = self.sealing_work.lock().unwrap();
		MinerStatus {
			transactions_in_pending_queue: status.pending,
			transactions_in_future_queue: status.future,
			transactions_in_pending_block: sealing_work.peek_last_ref().map_or(0, |b| b.transactions().len()),
		}
	}

	fn set_author(&self, author: Address) {
		*self.author.write().unwrap() = author;
	}

	fn set_extra_data(&self, extra_data: Bytes) {
		*self.extra_data.write().unwrap() = extra_data;
	}

	/// Set the gas limit we wish to target when sealing a new block.
	fn set_gas_floor_target(&self, target: U256) {
		*self.gas_floor_target.write().unwrap() = target;
	}

	fn set_minimal_gas_price(&self, min_gas_price: U256) {
		self.transaction_queue.lock().unwrap().set_minimal_gas_price(min_gas_price);
	}

	fn minimal_gas_price(&self) -> U256 {
		*self.transaction_queue.lock().unwrap().minimal_gas_price()
	}

	fn sensible_gas_price(&self) -> U256 {
		// 10% above our minimum.
		*self.transaction_queue.lock().unwrap().minimal_gas_price() * x!(110) / x!(100)
	}

	fn sensible_gas_limit(&self) -> U256 {
		*self.gas_floor_target.read().unwrap() / x!(5)
	}

	/// Get the author that we will seal blocks as.
	fn author(&self) -> Address {
		*self.author.read().unwrap()
	}

	/// Get the extra_data that we will seal blocks with.
	fn extra_data(&self) -> Bytes {
		self.extra_data.read().unwrap().clone()
	}

	/// Get the gas limit we wish to target when sealing a new block.
	fn gas_floor_target(&self) -> U256 {
		*self.gas_floor_target.read().unwrap()
	}

	fn import_transactions(&self, transactions: Vec<SignedTransaction>) -> Vec<Result<(), Error>> {
		let mut transaction_queue = self.transaction_queue.lock().unwrap();
		transaction_queue.add_all(transactions, |a: &Address| self.chain.account_details(a))
	}

	fn pending_transactions_hashes(&self) -> Vec<H256> {
		let transaction_queue = self.transaction_queue.lock().unwrap();
		transaction_queue.pending_hashes()
	}

	fn transaction(&self, hash: &H256) -> Option<SignedTransaction> {
		let queue = self.transaction_queue.lock().unwrap();
		queue.find(hash)
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		let queue = self.transaction_queue.lock().unwrap();
		queue.top_transactions()
	}

	fn last_nonce(&self, address: &Address) -> Option<U256> {
		self.transaction_queue.lock().unwrap().last_nonce(address)
	}

	fn update_sealing(&self) {
		if self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			let current_no = self.chain.best_block_number();
			let last_request = *self.sealing_block_last_request.lock().unwrap();
			let should_disable_sealing = !self.force_sealing && current_no > last_request && current_no - last_request > SEALING_TIMEOUT_IN_BLOCKS;

			if should_disable_sealing {
				trace!(target: "miner", "Miner sleeping (current {}, last {})", current_no, last_request);
				self.sealing_enabled.store(false, atomic::Ordering::Relaxed);
				self.sealing_work.lock().unwrap().reset();
			} else if self.sealing_enabled.load(atomic::Ordering::Relaxed) {
				self.prepare_sealing();
			}
		}
	}

	fn map_sealing_work<F, T>(&self, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		trace!(target: "miner", "map_sealing_work: entering");
		let have_work = self.sealing_work.lock().unwrap().peek_last_ref().is_some();
		trace!(target: "miner", "map_sealing_work: have_work={}", have_work);
		if !have_work {
			self.sealing_enabled.store(true, atomic::Ordering::Relaxed);
			self.prepare_sealing();
		}
		let mut sealing_block_last_request = self.sealing_block_last_request.lock().unwrap();
		let best_number = self.chain.best_block_number();
		if *sealing_block_last_request != best_number {
			trace!(target: "miner", "map_sealing_work: Miner received request (was {}, now {}) - waking up.", *sealing_block_last_request, best_number);
			*sealing_block_last_request = best_number;
		}

		let mut sealing_work = self.sealing_work.lock().unwrap();
		let ret = sealing_work.use_last_ref();
		trace!(target: "miner", "map_sealing_work: leaving use_last_ref={:?}", ret.as_ref().map(|b| b.block().fields().header.hash()));
		ret.map(f)
	}

	fn submit_seal(&self, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		if let Some(b) = self.sealing_work.lock().unwrap().take_used_if(|b| &b.hash() == &pow_hash) {
			match b.lock().try_seal(self.chain.engine(), seal) {
				Err(_) => {
					Err(Error::PowInvalid)
				}
				Ok(sealed) => {
					// TODO: commit DB from `sealed.drain` and make a VerifiedBlock to skip running the transactions twice.
					try!(self.chain.import_block(sealed.rlp_bytes()));
					Ok(())
				}
			}
		} else {
			Err(Error::PowHashInvalid)
		}
	}

	fn chain_new_blocks(&self, _imported: &[H256], _invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		// 1. We ignore blocks that were `imported` (because it means that they are not in canon-chain, and transactions
		//	  should be still available in the queue.
		// 2. We ignore blocks that are `invalid` because it doesn't have any meaning in terms of the transactions that
		//    are in those blocks

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

		// ...and at the end remove old ones
		{
			let in_chain = enacted
				.par_iter()
				.map(|h: &H256| self.chain.block_transactions(h));

			in_chain.for_each(|mut txs| {
				let mut transaction_queue = self.transaction_queue.lock().unwrap();

				let to_remove = txs.drain(..)
						.map(|tx| {
							tx.sender().expect("Transaction is in block, so sender has to be defined.")
						})
						.collect::<HashSet<Address>>();
				for sender in to_remove.into_iter() {
					transaction_queue.remove_all(sender, self.chain.account_details(&sender).nonce);
				}
			});
		}

		self.update_sealing();
	}
}

#[cfg(test)]
mod tests {

	use MinerService;
	use super::{Miner};
	use util::*;
	use ethcore::client::{TestBlockChainClient, EachBlockWith};
	use ethcore::block::*;

	#[test]
	fn should_prepare_block_to_seal() {
		// given
		let client = Arc::new(TestBlockChainClient::default());
		let miner = Miner::new(client, true);

		// when
		let sealing_work = miner.map_sealing_work(|_| ());

		// then
		assert!(sealing_work.is_some(), "Expected closed block");
	}

	#[test]
	fn should_still_work_after_a_couple_of_blocks() {
		// given
		let client = Arc::new(TestBlockChainClient::default());
		let miner = Miner::new(client.clone(), false);

		let res = miner.map_sealing_work(|b| b.block().fields().header.hash());
		assert!(res.is_some());
		assert!(miner.submit_seal(res.unwrap(), vec![]).is_ok());

		// two more blocks mined, work requested.
		client.add_blocks(1, EachBlockWith::Uncle);
		miner.update_sealing();
		let h1 = miner.map_sealing_work(|b| b.block().fields().header.hash());

		client.add_blocks(1, EachBlockWith::Uncle);
		miner.update_sealing();
		let h2 = miner.map_sealing_work(|b| b.block().fields().header.hash());

		// solution to original work submitted.
		assert!(h1 != h2);
		assert!(miner.submit_seal(res.unwrap(), vec![]).is_ok());
	}

}
