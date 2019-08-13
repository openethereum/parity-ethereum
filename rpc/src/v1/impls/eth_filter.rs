// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Eth Filter RPC implementation

use std::sync::Arc;
use std::collections::{BTreeSet, VecDeque};

use ethcore::client::BlockChainClient;
use ethcore::miner::{self, MinerService};
use ethereum_types::{H256, U256};
use parking_lot::Mutex;
use types::{
	ids::BlockId,
	filter::Filter as EthcoreFilter
};

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::{future, Future};
use jsonrpc_core::futures::future::Either;
use v1::traits::EthFilter;
use v1::types::{BlockNumber, Index, Filter, FilterChanges, Log};
use v1::helpers::{errors, SyncPollFilter, PollFilter, PollManager, limit_logs};
use v1::impls::eth::pending_logs;

/// Something which provides data that can be filtered over.
pub trait Filterable {
	/// Current best block number.
	fn best_block_number(&self) -> u64;

	/// Get a block hash by block id.
	fn block_hash(&self, id: BlockId) -> Option<H256>;

	/// pending transaction hashes at the given block (unordered).
	fn pending_transaction_hashes(&self) -> BTreeSet<H256>;

	/// Get logs that match the given filter.
	fn logs(&self, filter: EthcoreFilter) -> BoxFuture<Vec<Log>>;

	/// Get logs from the pending block.
	fn pending_logs(&self, block_number: u64, filter: &EthcoreFilter) -> Vec<Log>;

	/// Get a reference to the poll manager.
	fn polls(&self) -> &Mutex<PollManager<SyncPollFilter>>;

	/// Get removed logs within route from the given block to the nearest canon block, not including the canon block. Also returns how many logs have been traversed.
	fn removed_logs(&self, block_hash: H256, filter: &EthcoreFilter) -> (Vec<Log>, u64);
}

/// Eth filter rpc implementation for a full node.
pub struct EthFilterClient<C, M> {
	client: Arc<C>,
	miner: Arc<M>,
	polls: Mutex<PollManager<SyncPollFilter>>,
}

impl<C, M> EthFilterClient<C, M> {
	/// Creates new Eth filter client.
	pub fn new(client: Arc<C>, miner: Arc<M>, poll_lifetime: u32) -> Self {
		EthFilterClient {
			client,
			miner,
			polls: Mutex::new(PollManager::new(poll_lifetime)),
		}
	}
}

impl<C, M> Filterable for EthFilterClient<C, M> where
	C: miner::BlockChainClient + BlockChainClient,
	M: MinerService,
{
	fn best_block_number(&self) -> u64 {
		self.client.chain_info().best_block_number
	}

	fn block_hash(&self, id: BlockId) -> Option<H256> {
		self.client.block_hash(id)
	}

	fn pending_transaction_hashes(&self) -> BTreeSet<H256> {
		self.miner.pending_transaction_hashes(&*self.client)
	}

	fn logs(&self, filter: EthcoreFilter) -> BoxFuture<Vec<Log>> {
		Box::new(future::ok(self.client.logs(filter).unwrap_or_default().into_iter().map(Into::into).collect()))
	}

	fn pending_logs(&self, block_number: u64, filter: &EthcoreFilter) -> Vec<Log> {
		pending_logs(&*self.miner, block_number, filter)
	}

	fn polls(&self) -> &Mutex<PollManager<SyncPollFilter>> { &self.polls }

	fn removed_logs(&self, block_hash: H256, filter: &EthcoreFilter) -> (Vec<Log>, u64) {
		let inner = || -> Option<Vec<H256>> {
			let mut route = Vec::new();

			let mut current_block_hash = block_hash;
			let mut current_block_header = self.client.block_header(BlockId::Hash(current_block_hash))?;

			while current_block_hash != self.client.block_hash(BlockId::Number(current_block_header.number()))? {
				route.push(current_block_hash);

				current_block_hash = current_block_header.parent_hash();
				current_block_header = self.client.block_header(BlockId::Hash(current_block_hash))?;
			}

			Some(route)
		};

		let route = inner().unwrap_or_default();
		let route_len = route.len() as u64;
		(route.into_iter().flat_map(|block_hash| {
			let mut filter = filter.clone();
			filter.from_block = BlockId::Hash(block_hash);
			filter.to_block = filter.from_block;

			self.client.logs(filter).unwrap_or_default().into_iter().map(|log| {
				let mut log: Log = log.into();
				log.log_type = "removed".into();
				log.removed = true;

				log
			})
		}).collect(), route_len)
	}
}

impl<T: Filterable + Send + Sync + 'static> EthFilter for T {
	fn new_filter(&self, filter: Filter) -> Result<U256> {
		let mut polls = self.polls().lock();
		let block_number = self.best_block_number();
		let include_pending = filter.to_block == Some(BlockNumber::Pending);
		let filter = filter.try_into()?;
		let id = polls.create_poll(SyncPollFilter::new(PollFilter::Logs {
			block_number, filter, include_pending,
			last_block_hash: None,
			previous_logs: Default::default()
		}));
		Ok(id.into())
	}

	fn new_block_filter(&self) -> Result<U256> {
		let mut polls = self.polls().lock();
		// +1, since we don't want to include the current block
		let id = polls.create_poll(SyncPollFilter::new(PollFilter::Block {
			last_block_number: self.best_block_number(),
			recent_reported_hashes: VecDeque::with_capacity(PollFilter::MAX_BLOCK_HISTORY_SIZE),
		}));
		Ok(id.into())
	}

	fn new_pending_transaction_filter(&self) -> Result<U256> {
		let mut polls = self.polls().lock();
		let pending_transactions = self.pending_transaction_hashes();
		let id = polls.create_poll(SyncPollFilter::new(PollFilter::PendingTransaction(pending_transactions)));
		Ok(id.into())
	}

	fn filter_changes(&self, index: Index) -> BoxFuture<FilterChanges> {
		let filter = match self.polls().lock().poll_mut(&index.value()) {
			Some(filter) => filter.clone(),
			None => return Box::new(future::err(errors::filter_not_found())),
		};

		Box::new(filter.modify(|filter| match *filter {
			PollFilter::Block {
				ref mut last_block_number,
				ref mut recent_reported_hashes,
			} => {
				// Check validity of recently reported blocks -- in case of re-org, rewind block to last valid
				while let Some((num, hash)) = recent_reported_hashes.front().cloned() {
					if self.block_hash(BlockId::Number(num)) == Some(hash) { break; }
					*last_block_number = num - 1;
					recent_reported_hashes.pop_front();
				}
				let current_number = self.best_block_number();
				let mut hashes = Vec::new();
				for n in (*last_block_number + 1)..=current_number {
					let block_number = BlockId::Number(n);
					if let Some(hash) = self.block_hash(block_number) {
						*last_block_number = n;
						hashes.push(hash);
						// Only keep the most recent history
						if recent_reported_hashes.len() >= PollFilter::MAX_BLOCK_HISTORY_SIZE {
							recent_reported_hashes.pop_back();
						}
						recent_reported_hashes.push_front((n, hash));
					}
				}

				Either::A(future::ok(FilterChanges::Hashes(hashes)))
			},
			PollFilter::PendingTransaction(ref mut previous_hashes) => {
				// get hashes of pending transactions
				let current_hashes = self.pending_transaction_hashes();

				let new_hashes = {
					// find all new hashes
					current_hashes.difference(previous_hashes)
						.cloned()
						.map(Into::into)
						.collect()
				};

				// save all hashes of pending transactions
				*previous_hashes = current_hashes;

				// return new hashes
				Either::A(future::ok(FilterChanges::Hashes(new_hashes)))
			},
			PollFilter::Logs {
				ref mut block_number,
				ref mut last_block_hash,
				ref mut previous_logs,
				ref filter,
				include_pending,
			} => {
				// retrive the current block number
				let current_number = self.best_block_number();

				let mut filter = filter.clone();

				// retrieve reorg logs
				let (mut reorg, reorg_len) = last_block_hash.map_or_else(|| (Vec::new(), 0), |h| self.removed_logs(h, &filter));
				*block_number -= reorg_len as u64;

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

				// save the current block hash, which we used to get back to the
				// canon chain in case of reorg.
				*last_block_hash = self.block_hash(BlockId::Number(current_number));

				// retrieve logs in range from_block..min(BlockId::Latest..to_block)
				let limit = filter.limit;
				Either::B(self.logs(filter)
					.map(move |logs| { reorg.extend(logs); reorg }) // append reorg logs in the front
					.map(move |mut logs| { logs.extend(pending); logs }) // append fetched pending logs
					.map(move |logs| limit_logs(logs, limit)) // limit the logs
					.map(FilterChanges::Logs))
			}
		}))
	}

	fn filter_logs(&self, index: Index) -> BoxFuture<Vec<Log>> {
		let (filter, include_pending) = {
			let mut polls = self.polls().lock();

			match polls.poll(&index.value()).and_then(|f| f.modify(|filter| match *filter {
				PollFilter::Logs { ref filter, include_pending, .. } =>
					Some((filter.clone(), include_pending)),
				_ => None,
			})) {
				Some((filter, include_pending)) => (filter, include_pending),
				None => return Box::new(future::err(errors::filter_not_found())),
			}
		};

		// fetch pending logs.
		let pending = if include_pending {
			let best_block = self.best_block_number();
			self.pending_logs(best_block, &filter)
		} else {
			Vec::new()
		};

		// retrieve logs asynchronously, appending pending logs.
		let limit = filter.limit;
		let logs = self.logs(filter);
		Box::new(logs
			.map(move |mut logs| { logs.extend(pending); logs })
			.map(move |logs| limit_logs(logs, limit))
		)
	}

	fn uninstall_filter(&self, index: Index) -> Result<bool> {
		Ok(self.polls().lock().remove_poll(&index.value()))
	}
}
