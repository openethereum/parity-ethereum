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

//! Eth Filter RPC implementation

use std::sync::{Arc, Weak};
use std::collections::HashSet;
use jsonrpc_core::*;
use ethcore::miner::MinerService;
use ethcore::filter::Filter as EthcoreFilter;
use ethcore::client::{BlockChainClient, BlockID};
use util::Mutex;
use v1::traits::EthFilter;
use v1::types::{BlockNumber, Index, Filter, Log, H256 as RpcH256, U256 as RpcU256};
use v1::helpers::{PollFilter, PollManager};
use v1::helpers::params::expect_no_params;
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

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C, M> EthFilter for EthFilterClient<C, M> where
	C: BlockChainClient + 'static,
	M: MinerService + 'static {

	fn new_filter(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(Filter,)>(params)
			.and_then(|(filter,)| {
				let mut polls = self.polls.lock();
				let block_number = take_weak!(self.client).chain_info().best_block_number;
				let id = polls.create_poll(PollFilter::Logs(block_number, Default::default(), filter));
				to_value(&RpcU256::from(id))
			})
	}

	fn new_block_filter(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		let mut polls = self.polls.lock();
		let id = polls.create_poll(PollFilter::Block(take_weak!(self.client).chain_info().best_block_number));
		to_value(&RpcU256::from(id))
	}

	fn new_pending_transaction_filter(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		try!(expect_no_params(params));

		let mut polls = self.polls.lock();
		let best_block = take_weak!(self.client).chain_info().best_block_number;
		let pending_transactions = take_weak!(self.miner).pending_transactions_hashes(best_block);
		let id = polls.create_poll(PollFilter::PendingTransaction(pending_transactions));

		to_value(&RpcU256::from(id))
	}

	fn filter_changes(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		let client = take_weak!(self.client);
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
				let mut polls = self.polls.lock();
				match polls.poll_mut(&index.value()) {
					None => Ok(Value::Array(vec![] as Vec<Value>)),
					Some(filter) => match *filter {
						PollFilter::Block(ref mut block_number) => {
							// + 1, cause we want to return hashes including current block hash.
							let current_number = client.chain_info().best_block_number + 1;
							let hashes = (*block_number..current_number).into_iter()
								.map(BlockID::Number)
								.filter_map(|id| client.block_hash(id))
								.map(Into::into)
								.collect::<Vec<RpcH256>>();

							*block_number = current_number;

							to_value(&hashes)
						},
						PollFilter::PendingTransaction(ref mut previous_hashes) => {
							// get hashes of pending transactions
							let current_number = client.chain_info().best_block_number;
							let current_hashes = take_weak!(self.miner).pending_transactions_hashes(current_number);

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
							to_value(&new_hashes)
						},
						PollFilter::Logs(ref mut block_number, ref mut previous_logs, ref filter) => {
							// retrive the current block number
							let current_number = client.chain_info().best_block_number;

							// check if we need to check pending hashes
							let include_pending = filter.to_block == Some(BlockNumber::Pending);

							// build appropriate filter
							let mut filter: EthcoreFilter = filter.clone().into();
							filter.from_block = BlockID::Number(*block_number);
							filter.to_block = BlockID::Latest;

							// retrieve logs in range from_block..min(BlockID::Latest..to_block)
							let mut logs = client.logs(filter.clone())
								.into_iter()
								.map(From::from)
								.collect::<Vec<Log>>();

							// additionally retrieve pending logs
							if include_pending {
								let pending_logs = pending_logs(&*take_weak!(self.miner), current_number, &filter);

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

							// save the number of the next block as a first block from which
							// we want to get logs
							*block_number = current_number + 1;
							to_value(&logs)
						}
					}
				}
			})
	}

	fn filter_logs(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
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

						to_value(&logs)
					},
					// just empty array
					_ => Ok(Value::Array(vec![] as Vec<Value>)),
				}
			})
	}

	fn uninstall_filter(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
				self.polls.lock().remove_poll(&index.value());
				to_value(&true)
			})
	}
}
