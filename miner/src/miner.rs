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

use util::{H256, U256, Address, Bytes, Uint};
use ethcore::views::{BlockView, HeaderView};
use ethcore::client::{BlockChainClient, BlockId};
use ethcore::block::{ClosedBlock, IsBlock};
use ethcore::error::{Error};
use ethcore::transaction::SignedTransaction;
use super::{MinerService, MinerStatus, TransactionQueue, AccountDetails};

struct SealingWork {
	/// Not yet being sealed by a miner, but if one asks for work, we'd prefer they do this.
	would_seal: Option<ClosedBlock>,
	/// Currently being sealed by miners.
	being_sealed: Vec<ClosedBlock>, 
}

impl SealingWork {
	// inspect the work that would be given.
	fn pending_ref(&self) -> Option<&ClosedBlock> {
		self.would_seal.as_ref().or(self.being_sealed.last().as_ref())
	}
	
	// return the a reference to forst block that returns true to `f`.
	fn find_if<F>(&self, f: F) -> Option<&ClosedBlock> where F: Fn(&ClosedBlock) -> bool {
		if would_seal.as_ref().map(&f).unwrap_or(false) {
			would_seal.as_ref()
		} else {
			being_sealed.iter().find_if(f)
		}
	}

	// used for getting the work to be done.
	fn use_pending_ref(&mut self) -> Option<&ClosedBlock> {
		if let Some(x) = self.would_seal.take() {
			self.being_sealed.push(x);
			if self.being_sealed.len() > MAX_SEALING_BLOCKS_CACHE {
				self.being_sealed.erase(0);
			}
		}
		self.being_sealed.last().as_ref()
	}

	// set new work to be done.
	fn set_pending(&mut self, b: ClosedBlock) {
		self.would_seal = Some(b);
	}

	// get the pending block if `f(pending)`. if there is no pending block or it doesn't pass `f`, None.
	// will not destroy a block if a reference to it has previously been returned by `use_pending_ref`.
	fn pending_if<F>(&self, f: F) -> Option<ClosedBlock> where F: Fn(&ClosedBlock) -> bool {
		// a bit clumsy - TODO: think about a nicer way of expressing this.
		if let Some(x) = self.would_seal.take() {
			if f(&x) {
				Some(x)
			} else {
				self.would_seal = x;
				None
			}
		} else {
			being_sealed.last().as_ref().filter(&b).map(|b| b.clone())
/*			being_sealed.last().as_ref().and_then(|b| if f(b) {
				Some(b.clone())
			} else {
				None
			})*/
		}
	}

	// clears everything.
	fn reset(&mut self) {
		self.would_seal = None;
		self.being_sealed.clear();
	}
}

/// Keeps track of transactions using priority queue and holds currently mined block.
pub struct Miner {
	transaction_queue: Mutex<TransactionQueue>,

	// for sealing...
	sealing_enabled: AtomicBool,
	sealing_block_last_request: Mutex<u64>,
	sealing_work: Mutex<SealingWork>,
	gas_floor_target: RwLock<U256>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,

}

/*
		let sealing_work = self.sealing_work.lock();

		// TODO: check to see if last ClosedBlock in would_seals is same.
		// if so, duplicate, re-open and push any new transactions.
		// if at least one was pushed successfully, close and enqueue new ClosedBlock;
		//   and remove first ClosedBlock from the queue..

*/

impl Default for Miner {
	fn default() -> Miner {
		Miner {
			transaction_queue: Mutex::new(TransactionQueue::new()),
			sealing_enabled: AtomicBool::new(false),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(SealingWork{
				would_seal: None,
				being_sealed: vec![],
			}),
			gas_floor_target: RwLock::new(U256::zero()),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
		}
	}
}

impl Miner {
	/// Creates new instance of miner
	pub fn new() -> Arc<Miner> {
		Arc::new(Miner::default())
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
	fn prepare_sealing(&self, chain: &BlockChainClient) {
		let transactions = self.transaction_queue.lock().unwrap().top_transactions();
		let b = chain.prepare_sealing(
			self.author(),
			self.gas_floor_target(),
			self.extra_data(),
			transactions,
		);

		if let Some((block, invalid_transactions)) = b {
			let mut queue = self.transaction_queue.lock().unwrap();
			queue.remove_all(
				&invalid_transactions.into_iter().collect::<Vec<H256>>(),
				|a: &Address| AccountDetails {
					nonce: chain.nonce(a),
					balance: chain.balance(a),
				}
			);
			self.sealing_work.lock().unwrap().set_pending(block);
		}
	}

	fn update_gas_limit(&self, chain: &BlockChainClient) {
		let gas_limit = HeaderView::new(&chain.best_block_header()).gas_limit();
		let mut queue = self.transaction_queue.lock().unwrap();
		queue.set_gas_limit(gas_limit);
	}
}

const SEALING_TIMEOUT_IN_BLOCKS : u64 = 5;

impl MinerService for Miner {

	fn clear_and_reset(&self, chain: &BlockChainClient) {
		self.transaction_queue.lock().unwrap().clear();
		self.update_sealing(chain);
	}

	fn status(&self) -> MinerStatus {
		let status = self.transaction_queue.lock().unwrap().status();
		let sealing_work = self.sealing_work.lock().unwrap();
		MinerStatus {
			transactions_in_pending_queue: status.pending,
			transactions_in_future_queue: status.future,
			transactions_in_pending_block: block.pending_ref().map_or(0, |b| b.transactions().len()),
		}
	}

	fn author(&self) -> Address {
		*self.author.read().unwrap()
	}

	fn extra_data(&self) -> Bytes {
		self.extra_data.read().unwrap().clone()
	}

	fn import_transactions<T>(&self, transactions: Vec<SignedTransaction>, fetch_account: T) -> Vec<Result<(), Error>>
		where T: Fn(&Address) -> AccountDetails {
		let mut transaction_queue = self.transaction_queue.lock().unwrap();
		transaction_queue.add_all(transactions, fetch_account)
	}

	fn pending_transactions_hashes(&self) -> Vec<H256> {
		let transaction_queue = self.transaction_queue.lock().unwrap();
		transaction_queue.pending_hashes()
	}

	fn update_sealing(&self, chain: &BlockChainClient) {
		let should_disable_sealing = {
			let current_no = chain.chain_info().best_block_number;
			let last_request = self.sealing_block_last_request.lock().unwrap();
			let is_greater = current_no > *last_request;
			is_greater && current_no - *last_request > SEALING_TIMEOUT_IN_BLOCKS
		};

		if should_disable_sealing {
			self.sealing_enabled.store(false, atomic::Ordering::Relaxed);
			*self.sealing_work.lock().unwrap().reset();
		} else if self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			self.prepare_sealing(chain);
		}
	}

	fn map_sealing_work<F, T>(&self, chain: &BlockChainClient, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		let have_work = self.sealing_work.lock().unwrap().pending_ref().is_none();
		if !have_work {
			self.sealing_enabled.store(true, atomic::Ordering::Relaxed);
			self.prepare_sealing(chain);
		}
		*self.sealing_block_last_request.lock().unwrap() = chain.chain_info().best_block_number;
		self.sealing_work.lock().unwrap().use_pending().map(f)
	}

	fn submit_seal(&self, chain: &BlockChainClient, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		if let Some(b) = self.sealing_work().lock().unwrap().take_if(|b| &b.hash() == &pow_hash) {
			match chain.try_seal(b.unwrap(), seal) {
				Err(old) => {
					Err(Error::PowInvalid)
				}
				Ok(sealed) => {
					// TODO: commit DB from `sealed.drain` and make a VerifiedBlock to skip running the transactions twice.
					try!(chain.import_block(sealed.rlp_bytes()));
					Ok(())
				}
			}
		} else {
			Err(Error::PowHashInvalid)
		}
	}

	fn chain_new_blocks(&self, chain: &BlockChainClient, imported: &[H256], invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		fn fetch_transactions(chain: &BlockChainClient, hash: &H256) -> Vec<SignedTransaction> {
			let block = chain
				.block(BlockId::Hash(*hash))
				// Client should send message after commit to db and inserting to chain.
				.expect("Expected in-chain blocks.");
			let block = BlockView::new(&block);
			block.transactions()
		}

		// First update gas limit in transaction queue
		self.update_gas_limit(chain);

		// Then import all transactions...
		{
			let out_of_chain = retracted
				.par_iter()
				.map(|h| fetch_transactions(chain, h));
			out_of_chain.for_each(|txs| {
				// populate sender
				for tx in &txs {
					let _sender = tx.sender();
				}
				let mut transaction_queue = self.transaction_queue.lock().unwrap();
				let _ = transaction_queue.add_all(txs, |a| AccountDetails {
					nonce: chain.nonce(a),
					balance: chain.balance(a)
				});
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
				.map(|h: &H256| fetch_transactions(chain, h));

			in_chain.for_each(|txs| {
				let hashes = txs.iter().map(|tx| tx.hash()).collect::<Vec<H256>>();
				let mut transaction_queue = self.transaction_queue.lock().unwrap();
				transaction_queue.remove_all(&hashes, |a| AccountDetails {
					nonce: chain.nonce(a),
					balance: chain.balance(a)
				});
			});
		}

		self.update_sealing(chain);
	}
}

#[cfg(test)]
mod tests {

	use MinerService;
	use super::{Miner};
	use ethcore::client::{TestBlockChainClient, EachBlockWith};

	// TODO [ToDr] To uncomment client is cleaned from mining stuff.
	#[ignore]
	#[test]
	fn should_prepare_block_to_seal() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::default();

		// when
		let res = miner.would_seal(&client);

		// then
		assert!(res.lock().unwrap().is_some(), "Expected closed block");
	}

	#[test]
	fn should_reset_seal_after_couple_of_blocks() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::default();
		let res = miner.would_seal(&client);
		// TODO [ToDr] Uncomment after fixing TestBlockChainClient
		// assert!(res.lock().unwrap().is_some(), "Expected closed block");

		// when
		client.add_blocks(10, EachBlockWith::Uncle);

		// then
		assert!(res.lock().unwrap().is_none(), "Expected to remove sealed block");
	}
}
