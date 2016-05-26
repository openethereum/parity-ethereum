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
use std::sync::atomic::AtomicBool;

use util::*;
use util::keys::store::{AccountService, AccountProvider};
use ethcore::views::{BlockView, HeaderView};
use ethcore::client::{BlockChainClient, BlockID};
use ethcore::block::{ClosedBlock, IsBlock};
use ethcore::error::*;
use ethcore::client::{Executive, Executed, EnvInfo, TransactOptions};
use ethcore::transaction::SignedTransaction;
use ethcore::receipt::{Receipt};
use ethcore::spec::Spec;
use ethcore::engine::Engine;
use super::{MinerService, MinerStatus, TransactionQueue, AccountDetails, TransactionImportResult, TransactionOrigin};

/// Keeps track of transactions using priority queue and holds currently mined block.
pub struct Miner {
	transaction_queue: Mutex<TransactionQueue>,

	// for sealing...
	force_sealing: bool,
	sealing_enabled: AtomicBool,
	sealing_block_last_request: Mutex<u64>,
	sealing_work: Mutex<UsingQueue<ClosedBlock>>,
	gas_floor_target: RwLock<U256>,
	author: RwLock<Address>,
	extra_data: RwLock<Bytes>,
	spec: Spec,

	accounts: RwLock<Option<Arc<AccountService>>>,		// TODO: this is horrible since AccountService already contains a single RwLock field. refactor.
}

impl Default for Miner {
	fn default() -> Miner {
		Miner {
			transaction_queue: Mutex::new(TransactionQueue::new()),
			force_sealing: false,
			sealing_enabled: AtomicBool::new(false),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(UsingQueue::new(5)),
			gas_floor_target: RwLock::new(U256::zero()),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			accounts: RwLock::new(None),
			spec: Spec::new_test(),
		}
	}
}

impl Miner {
	/// Creates new instance of miner
	pub fn new(force_sealing: bool, spec: Spec) -> Arc<Miner> {
		Arc::new(Miner {
			transaction_queue: Mutex::new(TransactionQueue::new()),
			force_sealing: force_sealing,
			sealing_enabled: AtomicBool::new(force_sealing),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(UsingQueue::new(5)),
			gas_floor_target: RwLock::new(U256::zero()),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			accounts: RwLock::new(None),
			spec: spec,
		})
	}

	/// Creates new instance of miner
	pub fn with_accounts(force_sealing: bool, spec: Spec, accounts: Arc<AccountService>) -> Arc<Miner> {
		Arc::new(Miner {
			transaction_queue: Mutex::new(TransactionQueue::new()),
			force_sealing: force_sealing,
			sealing_enabled: AtomicBool::new(force_sealing),
			sealing_block_last_request: Mutex::new(0),
			sealing_work: Mutex::new(UsingQueue::new(5)),
			gas_floor_target: RwLock::new(U256::zero()),
			author: RwLock::new(Address::default()),
			extra_data: RwLock::new(Vec::new()),
			accounts: RwLock::new(Some(accounts)),
			spec: spec,
		})
	}

	fn engine(&self) -> &Engine {
		self.spec.engine.deref()
	}

	/// Prepares new block for sealing including top transactions from queue.
	#[cfg_attr(feature="dev", allow(match_same_arms))]
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn prepare_sealing(&self, chain: &BlockChainClient) {
		trace!(target: "miner", "prepare_sealing: entering");
		let transactions = self.transaction_queue.lock().unwrap().top_transactions();
		let mut sealing_work = self.sealing_work.lock().unwrap();
		let best_hash = chain.best_block_header().sha3();

/*
		// check to see if last ClosedBlock in would_seals is actually same parent block.
		// if so
		//   duplicate, re-open and push any new transactions.
		//   if at least one was pushed successfully, close and enqueue new ClosedBlock;
		//   otherwise, leave everything alone.
		// otherwise, author a fresh block.
*/

		let (b, invalid_transactions) = match sealing_work.pop_if(|b| b.block().fields().header.parent_hash() == &best_hash) {
			Some(old_block) => {
				trace!(target: "miner", "Already have previous work; updating and returning");
				// add transactions to old_block
				let e = self.engine();
				let mut invalid_transactions = HashSet::new();
				let mut block = old_block.reopen(e, chain.vm_factory());
				let block_number = block.block().fields().header.number();

				// TODO: push new uncles, too.
				// TODO: refactor with chain.prepare_sealing
				for tx in transactions {
					let hash = tx.hash();
					let res = block.push_transaction(tx, None);
					match res {
						Err(Error::Execution(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, .. })) => {
							trace!(target: "miner", "Skipping adding transaction to block because of gas limit: {:?}", hash);
							// Exit early if gas left is smaller then min_tx_gas
							let min_tx_gas: U256 = x!(21000);	// TODO: figure this out properly.
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
						_ => {}	// imported ok
					}
				}
				(Some(block.close()), invalid_transactions)
			}
			None => {
				// block not found - create it.
				trace!(target: "miner", "No existing work - making new block");
				chain.prepare_sealing(
					self.author(),
					self.gas_floor_target(),
					self.extra_data(),
					transactions,
				)
			}
		};
		let mut queue = self.transaction_queue.lock().unwrap();
		let fetch_account = |a: &Address| AccountDetails {
			nonce: chain.nonce(a),
			balance: chain.balance(a),
		};
		for hash in invalid_transactions.into_iter() {
			queue.remove_invalid(&hash, &fetch_account);
		}
		if let Some(block) = b {
			if !block.transactions().is_empty() {
				trace!(target: "miner", "prepare_sealing: block has transaction - attempting internal seal.");
				// block with transactions - see if we can seal immediately.
				let a = self.accounts.read().unwrap();
				let s = self.engine().generate_seal(block.block(), match *a.deref() {
					Some(ref x) => Some(x.deref() as &AccountProvider),
					None => None,
				});
				if let Some(seal) = s {
					trace!(target: "miner", "prepare_sealing: managed internal seal. importing...");
					if let Ok(sealed) = chain.try_seal(block.lock(), seal) {
						if let Ok(_) = chain.import_block(sealed.rlp_bytes()) {
							trace!(target: "miner", "prepare_sealing: sealed internally and imported. leaving.");
						} else {
							warn!("prepare_sealing: ERROR: could not import internally sealed block. WTF?");
						}
					} else {
						warn!("prepare_sealing: ERROR: try_seal failed when given internally generated seal. WTF?");
					}
					return;
				} else {
					trace!(target: "miner", "prepare_sealing: unable to generate seal internally");
				}
			}
			if sealing_work.peek_last_ref().map_or(true, |pb| pb.block().fields().header.hash() != block.block().fields().header.hash()) {
				trace!(target: "miner", "Pushing a new, refreshed or borrowed pending {}...", block.block().fields().header.hash());
				sealing_work.push(block);
			}
		}
		trace!(target: "miner", "prepare_sealing: leaving (last={:?})", sealing_work.peek_last_ref().map(|b| b.block().fields().header.hash()));
	}

	fn update_gas_limit(&self, chain: &BlockChainClient) {
		let gas_limit = HeaderView::new(&chain.best_block_header()).gas_limit();
		let mut queue = self.transaction_queue.lock().unwrap();
		queue.set_gas_limit(gas_limit);
	}

	/// Returns true if we had to prepare new pending block
	fn enable_and_prepare_sealing(&self, chain: &BlockChainClient) -> bool {
		trace!(target: "miner", "enable_and_prepare_sealing: entering");
		let have_work = self.sealing_work.lock().unwrap().peek_last_ref().is_some();
		trace!(target: "miner", "enable_and_prepare_sealing: have_work={}", have_work);
		if !have_work {
			self.sealing_enabled.store(true, atomic::Ordering::Relaxed);
			self.prepare_sealing(chain);
		}
		let mut sealing_block_last_request = self.sealing_block_last_request.lock().unwrap();
		let best_number = chain.chain_info().best_block_number;
		if *sealing_block_last_request != best_number {
			trace!(target: "miner", "enable_and_prepare_sealing: Miner received request (was {}, now {}) - waking up.", *sealing_block_last_request, best_number);
			*sealing_block_last_request = best_number;
		}

		// Return if
		!have_work
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
			transactions_in_pending_block: sealing_work.peek_last_ref().map_or(0, |b| b.transactions().len()),
		}
	}

	fn call(&self, chain: &BlockChainClient, t: &SignedTransaction) -> Result<Executed, ExecutionError> {
		let sealing_work = self.sealing_work.lock().unwrap();
		match sealing_work.peek_last_ref() {
			Some(work) => {
				let block = work.block();
				let header = block.header();
				let last_hashes = chain.last_hashes();
				let env_info = EnvInfo {
					number: header.number(),
					author: *header.author(),
					timestamp: header.timestamp(),
					difficulty: *header.difficulty(),
					last_hashes: last_hashes,
					gas_used: U256::zero(),
					gas_limit: U256::max_value(),
				};
				// that's just a copy of the state.
				let mut state = block.state().clone();
				let sender = try!(t.sender().map_err(|e| {
					let message = format!("Transaction malformed: {:?}", e);
					ExecutionError::TransactionMalformed(message)
				}));
				let balance = state.balance(&sender);
				// give the sender max balance
				state.sub_balance(&sender, &balance);
				state.add_balance(&sender, &U256::max_value());
				let options = TransactOptions { tracing: false, check_nonce: false };

				Executive::new(&mut state, &env_info, self.engine(), chain.vm_factory()).transact(t, options)
			},
			None => {
				chain.call(t)
			}
		}
	}

	fn balance(&self, chain: &BlockChainClient, address: &Address) -> U256 {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(|| chain.balance(address), |b| b.block().fields().state.balance(address))
	}

	fn storage_at(&self, chain: &BlockChainClient, address: &Address, position: &H256) -> H256 {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(|| chain.storage_at(address, position), |b| b.block().fields().state.storage_at(address, position))
	}

	fn nonce(&self, chain: &BlockChainClient, address: &Address) -> U256 {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(|| chain.nonce(address), |b| b.block().fields().state.nonce(address))
	}

	fn code(&self, chain: &BlockChainClient, address: &Address) -> Option<Bytes> {
		let sealing_work = self.sealing_work.lock().unwrap();
		sealing_work.peek_last_ref().map_or_else(|| chain.code(address), |b| b.block().fields().state.code(address))
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

	fn transactions_limit(&self) -> usize {
		self.transaction_queue.lock().unwrap().limit()
	}

	fn set_transactions_limit(&self, limit: usize) {
		self.transaction_queue.lock().unwrap().set_limit(limit)
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

	fn import_transactions<T>(&self, transactions: Vec<SignedTransaction>, fetch_account: T) ->
		Vec<Result<TransactionImportResult, Error>>
		where T: Fn(&Address) -> AccountDetails {
		let mut transaction_queue = self.transaction_queue.lock().unwrap();
		transactions.into_iter()
			.map(|tx| transaction_queue.add(tx, &fetch_account, TransactionOrigin::External))
			.collect()
	}

	fn import_own_transaction<T>(&self, chain: &BlockChainClient, transaction: SignedTransaction, fetch_account: T) ->
		Result<TransactionImportResult, Error>
		where T: Fn(&Address) -> AccountDetails {
		let hash = transaction.hash();
		trace!(target: "own_tx", "Importing transaction: {:?}", transaction);

		let imported = {
			// Be sure to release the lock before we call enable_and_prepare_sealing
			let mut transaction_queue = self.transaction_queue.lock().unwrap();
			let import = transaction_queue.add(transaction, &fetch_account, TransactionOrigin::Local);

			match import {
				Ok(ref res) => {
					trace!(target: "own_tx", "Imported transaction to {:?} (hash: {:?})", res, hash);
					trace!(target: "own_tx", "Status: {:?}", transaction_queue.status());
				},
				Err(ref e) => {
					trace!(target: "own_tx", "Failed to import transaction {:?} (hash: {:?})", e, hash);
					trace!(target: "own_tx", "Status: {:?}", transaction_queue.status());
				},
			}
			import
		};

		if imported.is_ok() {
			// Make sure to do it after transaction is imported and lock is droped.
			// We need to create pending block and enable sealing
			let prepared = self.enable_and_prepare_sealing(chain);
			// If new block has not been prepared (means we already had one)
			// we need to update sealing
			if !prepared {
				self.update_sealing(chain);
			}
		}

		imported
	}

	fn pending_transactions_hashes(&self) -> Vec<H256> {
		match (self.sealing_enabled.load(atomic::Ordering::Relaxed), self.sealing_work.lock().unwrap().peek_last_ref()) {
			(true, Some(pending)) => pending.transactions().iter().map(|t| t.hash()).collect(),
			_ => {
				let queue = self.transaction_queue.lock().unwrap();
				queue.pending_hashes()
			}
		}
	}

	fn transaction(&self, hash: &H256) -> Option<SignedTransaction> {
		match (self.sealing_enabled.load(atomic::Ordering::Relaxed), self.sealing_work.lock().unwrap().peek_last_ref()) {
			(true, Some(pending)) => pending.transactions().iter().find(|t| &t.hash() == hash).map(|t| t.clone()),
			_ => {
				let queue = self.transaction_queue.lock().unwrap();
				queue.find(hash)
			}
		}
	}

	fn all_transactions(&self) -> Vec<SignedTransaction> {
		let queue = self.transaction_queue.lock().unwrap();
		queue.top_transactions()
	}

	fn pending_transactions(&self) -> Vec<SignedTransaction> {
		// TODO: should only use the sealing_work when it's current (it could be an old block)
		match (self.sealing_enabled.load(atomic::Ordering::Relaxed), self.sealing_work.lock().unwrap().peek_last_ref()) {
			(true, Some(pending)) => pending.transactions().clone(),
			_ => {
				let queue = self.transaction_queue.lock().unwrap();
				queue.top_transactions()
			}
		}
	}

	fn pending_receipts(&self) -> BTreeMap<H256, Receipt> {
		match (self.sealing_enabled.load(atomic::Ordering::Relaxed), self.sealing_work.lock().unwrap().peek_last_ref()) {
			(true, Some(pending)) => {
				let hashes = pending.transactions()
					.iter()
					.map(|t| t.hash());

				let receipts = pending.receipts().clone().into_iter();

				hashes.zip(receipts).collect()
			},
			_ => BTreeMap::new()
		}
	}

	fn last_nonce(&self, address: &Address) -> Option<U256> {
		self.transaction_queue.lock().unwrap().last_nonce(address)
	}

	fn update_sealing(&self, chain: &BlockChainClient) {
		if self.sealing_enabled.load(atomic::Ordering::Relaxed) {
			let current_no = chain.chain_info().best_block_number;
			let has_local_transactions = self.transaction_queue.lock().unwrap().has_local_pending_transactions();
			let last_request = *self.sealing_block_last_request.lock().unwrap();
			let should_disable_sealing = !self.force_sealing
				&& !has_local_transactions
				&& current_no > last_request
				&& current_no - last_request > SEALING_TIMEOUT_IN_BLOCKS;

			if should_disable_sealing {
				trace!(target: "miner", "Miner sleeping (current {}, last {})", current_no, last_request);
				self.sealing_enabled.store(false, atomic::Ordering::Relaxed);
				self.sealing_work.lock().unwrap().reset();
			} else if self.sealing_enabled.load(atomic::Ordering::Relaxed) {
				self.prepare_sealing(chain);
			}
		}
	}

	fn map_sealing_work<F, T>(&self, chain: &BlockChainClient, f: F) -> Option<T> where F: FnOnce(&ClosedBlock) -> T {
		trace!(target: "miner", "map_sealing_work: entering");
		self.enable_and_prepare_sealing(chain);
		trace!(target: "miner", "map_sealing_work: sealing prepared");
		let mut sealing_work = self.sealing_work.lock().unwrap();
		let ret = sealing_work.use_last_ref();
		trace!(target: "miner", "map_sealing_work: leaving use_last_ref={:?}", ret.as_ref().map(|b| b.block().fields().header.hash()));
		ret.map(f)
	}

	fn submit_seal(&self, chain: &BlockChainClient, pow_hash: H256, seal: Vec<Bytes>) -> Result<(), Error> {
		if let Some(b) = self.sealing_work.lock().unwrap().take_used_if(|b| &b.hash() == &pow_hash) {
			match chain.try_seal(b.lock(), seal) {
				Err(_) => {
					info!(target: "miner", "Mined block rejected, PoW was invalid.");
					Err(Error::PowInvalid)
				}
				Ok(sealed) => {
					info!(target: "miner", "New block mined, hash: {}", sealed.header().hash());
					// TODO: commit DB from `sealed.drain` and make a VerifiedBlock to skip running the transactions twice.
					let b = sealed.rlp_bytes();
					let h = b.sha3();
					try!(chain.import_block(b));
					info!("Block {} submitted and imported.", h);
					Ok(())
				}
			}
		} else {
			info!(target: "miner", "Mined block rejected, PoW hash invalid or out of date.");
			Err(Error::PowHashInvalid)
		}
	}

	fn chain_new_blocks(&self, chain: &BlockChainClient, _imported: &[H256], _invalid: &[H256], enacted: &[H256], retracted: &[H256]) {
		fn fetch_transactions(chain: &BlockChainClient, hash: &H256) -> Vec<SignedTransaction> {
			let block = chain
				.block(BlockID::Hash(*hash))
				// Client should send message after commit to db and inserting to chain.
				.expect("Expected in-chain blocks.");
			let block = BlockView::new(&block);
			block.transactions()
		}

		// 1. We ignore blocks that were `imported` (because it means that they are not in canon-chain, and transactions
		//	  should be still available in the queue.
		// 2. We ignore blocks that are `invalid` because it doesn't have any meaning in terms of the transactions that
		//    are in those blocks

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
				let _ = self.import_transactions(txs, |a| AccountDetails {
					nonce: chain.nonce(a),
					balance: chain.balance(a),
				});
			});
		}

		// ...and at the end remove old ones
		{
			let in_chain = enacted
				.par_iter()
				.map(|h: &H256| fetch_transactions(chain, h));

			in_chain.for_each(|mut txs| {
				let mut transaction_queue = self.transaction_queue.lock().unwrap();

				let to_remove = txs.drain(..)
						.map(|tx| {
							tx.sender().expect("Transaction is in block, so sender has to be defined.")
						})
						.collect::<HashSet<Address>>();
				for sender in to_remove.into_iter() {
					transaction_queue.remove_all(sender, chain.nonce(&sender));
				}
			});
		}

		self.update_sealing(chain);
	}
}

#[cfg(test)]
mod tests {

	use MinerService;
	use super::{Miner};
	use util::*;
	use ethcore::client::{TestBlockChainClient, EachBlockWith};
	use ethcore::block::*;

	// TODO [ToDr] To uncomment when TestBlockChainClient can actually return a ClosedBlock.
	#[ignore]
	#[test]
	fn should_prepare_block_to_seal() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::default();

		// when
		let sealing_work = miner.map_sealing_work(&client, |_| ());
		assert!(sealing_work.is_some(), "Expected closed block");
	}

	#[ignore]
	#[test]
	fn should_still_work_after_a_couple_of_blocks() {
		// given
		let client = TestBlockChainClient::default();
		let miner = Miner::default();

		let res = miner.map_sealing_work(&client, |b| b.block().fields().header.hash());
		assert!(res.is_some());
		assert!(miner.submit_seal(&client, res.unwrap(), vec![]).is_ok());

		// two more blocks mined, work requested.
		client.add_blocks(1, EachBlockWith::Uncle);
		miner.map_sealing_work(&client, |b| b.block().fields().header.hash());

		client.add_blocks(1, EachBlockWith::Uncle);
		miner.map_sealing_work(&client, |b| b.block().fields().header.hash());

		// solution to original work submitted.
		assert!(miner.submit_seal(&client, res.unwrap(), vec![]).is_ok());
	}
}
