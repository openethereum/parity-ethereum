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

use std::sync::{Arc, Weak};
use std::collections::HashSet;
use jsonrpc_core::*;
use ethcore::miner::MinerService;
use ethcore::filter::Filter as EthcoreFilter;
use ethcore::client::{BlockChainClient, BlockId};
use util::Mutex;
use v1::traits::EthFilter;
use v1::types::{BlockNumber, Index, Filter, FilterChanges, Log, H256 as RpcH256, U256 as RpcU256};
use v1::helpers::{PollFilter, PollManager, limit_logs};
use v1::impls::eth::pending_logs;

/// Eth filter rpc implementation.
pub struct EthFilterClient<C, M> where
	C: BlockChainClient,
	M: MinerService {

	client: Weak<C>,
	miner: Weak<M>,
	polls: Mutex<PollManager<PollFilter>>,
}

impl<C, M> EthFilterClient<C, M> where
	C: BlockChainClient,
	M: MinerService {

	/// Creates new Eth filter client.
	pub fn new(client: &Arc<C>, miner: &Arc<M>) -> Self {
		EthFilterClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			polls: Mutex::new(PollManager::new()),
		}
	}
}

impl<C, M> EthFilter for EthFilterClient<C, M>
	where C: BlockChainClient + 'static, M: MinerService + 'static
{
	fn new_filter(&self, filter: Filter) -> Result<RpcU256, Error> {
		let mut polls = self.polls.lock();
		let block_number = take_weak!(self.client).chain_info().best_block_number;
		let id = polls.create_poll(PollFilter::Logs(block_number, Default::default(), filter));
		Ok(id.into())
	}

	fn new_block_filter(&self) -> Result<RpcU256, Error> {
		let mut polls = self.polls.lock();
		let id = polls.create_poll(PollFilter::Block(take_weak!(self.client).chain_info().best_block_number));
		Ok(id.into())
	}

	fn new_pending_transaction_filter(&self) -> Result<RpcU256, Error> {
		let mut polls = self.polls.lock();
		let best_block = take_weak!(self.client).chain_info().best_block_number;
		let pending_transactions = take_weak!(self.miner).pending_transactions_hashes(best_block);
		let id = polls.create_poll(PollFilter::PendingTransaction(pending_transactions));
		Ok(id.into())
	}

	fn filter_changes(&self, index: Index) -> Result<FilterChanges, Error> {
		let client = take_weak!(self.client);
		let mut polls = self.polls.lock();
		match polls.poll_mut(&index.value()) {
			None => Ok(FilterChanges::Empty),
			Some(filter) => match *filter {
				PollFilter::Block(ref mut block_number) => {
					// + 1, cause we want to return hashes including current block hash.
					let current_number = client.chain_info().best_block_number + 1;
					let hashes = (*block_number..current_number).into_iter()
						.map(BlockId::Number)
						.filter_map(|id| client.block_hash(id))
						.map(Into::into)
						.collect::<Vec<RpcH256>>();

					*block_number = current_number;

					Ok(FilterChanges::Hashes(hashes))
				},
				PollFilter::PendingTransaction(ref mut previous_hashes) => {
					// get hashes of pending transactions
					let best_block = take_weak!(self.client).chain_info().best_block_number;
					let current_hashes = take_weak!(self.miner).pending_transactions_hashes(best_block);

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
					Ok(FilterChanges::Hashes(new_hashes))
				},
				PollFilter::Logs(ref mut block_number, ref mut previous_logs, ref filter) => {
					// retrive the current block number
					let current_number = client.chain_info().best_block_number;

					// check if we need to check pending hashes
					let include_pending = filter.to_block == Some(BlockNumber::Pending);

					// build appropriate filter
					let mut filter: EthcoreFilter = filter.clone().into();
					filter.from_block = BlockId::Number(*block_number);
					filter.to_block = BlockId::Latest;

					// retrieve logs in range from_block..min(BlockId::Latest..to_block)
					let mut logs = client.logs(filter.clone())
						.into_iter()
						.map(From::from)
						.collect::<Vec<Log>>();

					// additionally retrieve pending logs
					if include_pending {
						let best_block = take_weak!(self.client).chain_info().best_block_number;
						let pending_logs = pending_logs(&*take_weak!(self.miner), best_block, &filter);

						// remove logs about which client was already notified about
						let new_pending_logs: Vec<_> = pending_logs.iter()
							.filter(|p| !previous_logs.contains(p))
							.cloned()
							.collect();

						// save all logs retrieved by client
						*previous_logs = pending_logs.into_iter().collect();

						// append logs array with new pending logs
						logs.extend(new_pending_logs);
					}

					let logs = limit_logs(logs, filter.limit);

					// save the number of the next block as a first block from which
					// we want to get logs
					*block_number = current_number + 1;

					Ok(FilterChanges::Logs(logs))
				}
			}
		}
	}

	fn filter_logs(&self, index: Index) -> Result<Vec<Log>, Error> {
		let mut polls = self.polls.lock();
		match polls.poll(&index.value()) {
			Some(&PollFilter::Logs(ref _block_number, ref _previous_log, ref filter)) => {
				let include_pending = filter.to_block == Some(BlockNumber::Pending);
				let filter: EthcoreFilter = filter.clone().into();
				let mut logs = take_weak!(self.client).logs(filter.clone())
					.into_iter()
					.map(From::from)
					.collect::<Vec<Log>>();

				if include_pending {
					let best_block = take_weak!(self.client).chain_info().best_block_number;
					logs.extend(pending_logs(&*take_weak!(self.miner), best_block, &filter));
				}

				let logs = limit_logs(logs, filter.limit);

				Ok(logs)
			},
			// just empty array
			_ => Ok(Vec::new()),
		}
	}

	fn uninstall_filter(&self, index: Index) -> Result<bool, Error> {
		self.polls.lock().remove_poll(&index.value());
		Ok(true)
	}
}
