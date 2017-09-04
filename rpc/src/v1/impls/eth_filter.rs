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

//! Eth Filter RPC implementation

use std::sync::Arc;
use std::collections::HashSet;

use jsonrpc_core::*;
use ethcore::miner::MinerService;
use ethcore::filter::Filter as EthcoreFilter;
use ethcore::client::{BlockChainClient, BlockId};
use bigint::hash::H256;
use util::Mutex;

use futures::{future, Future, BoxFuture};

use v1::traits::EthFilter;
use v1::types::{BlockNumber, Index, Filter, FilterChanges, Log, H256 as RpcH256, U256 as RpcU256};
use v1::helpers::{PollFilter, PollManager, limit_logs};
use v1::impls::eth::pending_logs;

/// Something which provides data that can be filtered over.
pub trait Filterable {
	/// Current best block number.
	fn best_block_number(&self) -> u64;

	/// Get a block hash by block id.
	fn block_hash(&self, id: BlockId) -> Option<RpcH256>;

	/// pending transaction hashes at the given block.
	fn pending_transactions_hashes(&self, block_number: u64) -> Vec<H256>;

	/// Get logs that match the given filter.
	fn logs(&self, filter: EthcoreFilter) -> BoxFuture<Vec<Log>, Error>;

	/// Get logs from the pending block.
	fn pending_logs(&self, block_number: u64, filter: &EthcoreFilter) -> Vec<Log>;

	/// Get a reference to the poll manager.
	fn polls(&self) -> &Mutex<PollManager<PollFilter>>;
}

/// Eth filter rpc implementation for a full node.
pub struct EthFilterClient<C, M> where
	C: BlockChainClient,
	M: MinerService {

	client: Arc<C>,
	miner: Arc<M>,
	polls: Mutex<PollManager<PollFilter>>,
}

impl<C, M> EthFilterClient<C, M> where C: BlockChainClient, M: MinerService {
	/// Creates new Eth filter client.
	pub fn new(client: Arc<C>, miner: Arc<M>) -> Self {
		EthFilterClient {
			client: client,
			miner: miner,
			polls: Mutex::new(PollManager::new()),
		}
	}
}

impl<C, M> Filterable for EthFilterClient<C, M> where C: BlockChainClient, M: MinerService {
	fn best_block_number(&self) -> u64 {
		self.client.chain_info().best_block_number
	}

	fn block_hash(&self, id: BlockId) -> Option<RpcH256> {
		self.client.block_hash(id).map(Into::into)
	}

	fn pending_transactions_hashes(&self, best: u64) -> Vec<H256> {
		self.miner.pending_transactions_hashes(best)
	}

	fn logs(&self, filter: EthcoreFilter) -> BoxFuture<Vec<Log>, Error> {
		future::ok(self.client.logs(filter).into_iter().map(Into::into).collect()).boxed()
	}

	fn pending_logs(&self, block_number: u64, filter: &EthcoreFilter) -> Vec<Log> {
		pending_logs(&*self.miner, block_number, filter)
	}

	fn polls(&self) -> &Mutex<PollManager<PollFilter>> { &self.polls }
}



impl<T: Filterable + Send + Sync + 'static> EthFilter for T {
	fn new_filter(&self, filter: Filter) -> Result<RpcU256, Error> {
		let mut polls = self.polls().lock();
		let block_number = self.best_block_number();
		let id = polls.create_poll(PollFilter::Logs(block_number, Default::default(), filter));
		Ok(id.into())
	}

	fn new_block_filter(&self) -> Result<RpcU256, Error> {
		let mut polls = self.polls().lock();
		let id = polls.create_poll(PollFilter::Block(self.best_block_number()));
		Ok(id.into())
	}

	fn new_pending_transaction_filter(&self) -> Result<RpcU256, Error> {
		let mut polls = self.polls().lock();
		let best_block = self.best_block_number();
		let pending_transactions = self.pending_transactions_hashes(best_block);
		let id = polls.create_poll(PollFilter::PendingTransaction(pending_transactions));
		Ok(id.into())
	}

	fn filter_changes(&self, index: Index) -> BoxFuture<FilterChanges, Error> {
		let mut polls = self.polls().lock();
		match polls.poll_mut(&index.value()) {
			None => future::ok(FilterChanges::Empty).boxed(),
			Some(filter) => match *filter {
				PollFilter::Block(ref mut block_number) => {
					// + 1, cause we want to return hashes including current block hash.
					let current_number = self.best_block_number() + 1;
					let hashes = (*block_number..current_number).into_iter()
						.map(BlockId::Number)
						.filter_map(|id| self.block_hash(id))
						.collect::<Vec<RpcH256>>();

					*block_number = current_number;

					future::ok(FilterChanges::Hashes(hashes)).boxed()
				},
				PollFilter::PendingTransaction(ref mut previous_hashes) => {
					// get hashes of pending transactions
					let best_block = self.best_block_number();
					let current_hashes = self.pending_transactions_hashes(best_block);

					let new_hashes =
					{
						let previous_hashes_set = previous_hashes.iter().collect::<HashSet<_>>();

						//	find all new hashes
						current_hashes
							.iter()
							.filter(|hash| !previous_hashes_set.contains(hash))
							.cloned()
							.map(Into::into)
							.collect::<Vec<RpcH256>>()
					};

					// save all hashes of pending transactions
					*previous_hashes = current_hashes;

					// return new hashes
					future::ok(FilterChanges::Hashes(new_hashes)).boxed()
				},
				PollFilter::Logs(ref mut block_number, ref mut previous_logs, ref filter) => {
					// retrive the current block number
					let current_number = self.best_block_number();

					// check if we need to check pending hashes
					let include_pending = filter.to_block == Some(BlockNumber::Pending);

					// build appropriate filter
					let mut filter: EthcoreFilter = filter.clone().into();
					filter.from_block = BlockId::Number(*block_number);
					filter.to_block = BlockId::Latest;

					// retrieve pending logs
					let pending = if include_pending {
						let pending_logs = self.pending_logs(current_number, &filter);

						// remove logs about which client was already notified about
						let new_pending_logs: Vec<_> = pending_logs.iter()
							.filter(|p| !previous_logs.contains(p))
							.cloned()
							.collect();

						// save all logs retrieved by client
						*previous_logs = pending_logs.into_iter().collect();

						new_pending_logs
					} else {
						Vec::new()
					};

					// save the number of the next block as a first block from which
					// we want to get logs
					*block_number = current_number + 1;

					// retrieve logs in range from_block..min(BlockId::Latest..to_block)
					let limit = filter.limit;
					self.logs(filter)
						.map(move |mut logs| { logs.extend(pending); logs }) // append fetched pending logs
						.map(move |logs| limit_logs(logs, limit)) // limit the logs
						.map(FilterChanges::Logs)
						.boxed()
				}
			}
		}
	}

	fn filter_logs(&self, index: Index) -> BoxFuture<Vec<Log>, Error> {
		let mut polls = self.polls().lock();
		match polls.poll(&index.value()) {
			Some(&PollFilter::Logs(ref _block_number, ref _previous_log, ref filter)) => {
				let include_pending = filter.to_block == Some(BlockNumber::Pending);
				let filter: EthcoreFilter = filter.clone().into();

				// fetch pending logs.
				let pending = if include_pending {
					let best_block = self.best_block_number();
					self.pending_logs(best_block, &filter)
				} else {
					Vec::new()
				};

				// retrieve logs asynchronously, appending pending logs.
				let limit = filter.limit;
				self.logs(filter)
					.map(move |mut logs| { logs.extend(pending); logs })
					.map(move |logs| limit_logs(logs, limit))
					.boxed()
			},
			// just empty array
			_ => future::ok(Vec::new()).boxed()
		}
	}

	fn uninstall_filter(&self, index: Index) -> Result<bool, Error> {
		self.polls().lock().remove_poll(&index.value());
		Ok(true)
	}
}
