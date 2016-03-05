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

//! Eth rpc implementation.
use std::collections::HashMap;
use std::sync::{Arc, Weak, Mutex, RwLock};
use ethsync::{EthSync, SyncState};
use jsonrpc_core::*;
use util::numbers::*;
use util::sha3::*;
use util::rlp::encode;
use ethcore::client::*;
use ethcore::block::{IsBlock};
use ethcore::views::*;
//#[macro_use] extern crate log;
use ethcore::ethereum::Ethash;
use ethcore::ethereum::denominations::shannon;
use v1::traits::{Eth, EthFilter};
use v1::types::{Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo, Transaction, OptionalValue, Index, Filter, Log};
use v1::helpers::{PollFilter, PollManager};

/// Eth rpc implementation.
pub struct EthClient {
	client: Weak<Client>,
	sync: Weak<EthSync>,
	hashrates: RwLock<HashMap<H256, u64>>,
}

impl EthClient {
	/// Creates new EthClient.
	pub fn new(client: &Arc<Client>, sync: &Arc<EthSync>) -> Self {
		EthClient {
			client: Arc::downgrade(client),
			sync: Arc::downgrade(sync),
			hashrates: RwLock::new(HashMap::new()),
		}
	}

	fn block(&self, id: BlockId, include_txs: bool) -> Result<Value, Error> {
		let client = take_weak!(self.client);
		match (client.block(id.clone()), client.block_total_difficulty(id)) {
			(Some(bytes), Some(total_difficulty)) => {
				let block_view = BlockView::new(&bytes);
				let view = block_view.header_view();
				let block = Block {
					hash: OptionalValue::Value(view.sha3()),
					parent_hash: view.parent_hash(),
					uncles_hash: view.uncles_hash(),
					author: view.author(),
					miner: view.author(),
					state_root: view.state_root(),
					transactions_root: view.transactions_root(),
					receipts_root: view.receipts_root(),
					number: OptionalValue::Value(U256::from(view.number())),
					gas_used: view.gas_used(),
					gas_limit: view.gas_limit(),
					logs_bloom: view.log_bloom(),
					timestamp: U256::from(view.timestamp()),
					difficulty: view.difficulty(),
					total_difficulty: total_difficulty,
					uncles: vec![],
					transactions: {
						if include_txs {
							BlockTransactions::Full(block_view.localized_transactions().into_iter().map(From::from).collect())
						} else {
							BlockTransactions::Hashes(block_view.transaction_hashes())
						}
					},
					extra_data: Bytes::default()
				};
				to_value(&block)
			},
			_ => Ok(Value::Null)
		}
	}

	fn transaction(&self, id: TransactionId) -> Result<Value, Error> {
		match take_weak!(self.client).transaction(id) {
			Some(t) => to_value(&Transaction::from(t)),
			None => Ok(Value::Null)
		}
	}
}

impl Eth for EthClient {
	fn protocol_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&U256::from(take_weak!(self.sync).status().protocol_version)),
			_ => Err(Error::invalid_params())
		}
	}

	fn syncing(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let status = take_weak!(self.sync).status();
				let res = match status.state {
					SyncState::NotSynced | SyncState::Idle => SyncStatus::None,
					SyncState::Waiting | SyncState::Blocks | SyncState::NewBlocks => SyncStatus::Info(SyncInfo {
						starting_block: U256::from(status.start_block_number),
						current_block: U256::from(take_weak!(self.client).chain_info().best_block_number),
						highest_block: U256::from(status.highest_block_number.unwrap_or(status.start_block_number))
					})
				};
				to_value(&res)
			}
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: do not hardcode author.
	fn author(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&Address::new()),
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: return real value of mining once it's implemented.
	fn is_mining(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&!self.hashrates.read().unwrap().is_empty()),
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: return real hashrate once we have mining
	fn hashrate(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&self.hashrates.read().unwrap().iter().fold(0u64, |sum, (_, v)| sum + v)),
			_ => Err(Error::invalid_params())
		}
	}

	fn gas_price(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&(shannon() * U256::from(50))),
			_ => Err(Error::invalid_params())
		}
	}

	fn block_number(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&U256::from(take_weak!(self.client).chain_info().best_block_number)),
			_ => Err(Error::invalid_params())
		}
	}

	fn block_transaction_count_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| match take_weak!(self.client).block(BlockId::Hash(hash)) {
				Some(bytes) => to_value(&BlockView::new(&bytes).transactions_count()),
				None => Ok(Value::Null)
			})
	}

	fn block_transaction_count_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber,)>(params)
			.and_then(|(block_number,)| match block_number {
				BlockNumber::Pending => to_value(&take_weak!(self.sync).status().transaction_queue_pending),
				_ => match take_weak!(self.client).block(block_number.into()) {
					Some(bytes) => to_value(&BlockView::new(&bytes).transactions_count()),
					None => Ok(Value::Null)
				}
			})
	}

	fn block_uncles_count(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| match take_weak!(self.client).block(BlockId::Hash(hash)) {
				Some(bytes) => to_value(&BlockView::new(&bytes).uncles_count()),
				None => Ok(Value::Null)
			})
	}

	// TODO: do not ignore block number param
	fn code_at(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, BlockNumber)>(params)
			.and_then(|(address, _block_number)| to_value(&take_weak!(self.client).code(&address).map_or_else(Bytes::default, Bytes::new)))
	}

	fn block_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, bool)>(params)
			.and_then(|(hash, include_txs)| self.block(BlockId::Hash(hash), include_txs))
	}

	fn block_by_number(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, bool)>(params)
			.and_then(|(number, include_txs)| self.block(number.into(), include_txs))
	}

	fn transaction_by_hash(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| self.transaction(TransactionId::Hash(hash)))
	}

	fn transaction_by_block_hash_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256, Index)>(params)
			.and_then(|(hash, index)| self.transaction(TransactionId::Location(BlockId::Hash(hash), index.value())))
	}

	fn transaction_by_block_number_and_index(&self, params: Params) -> Result<Value, Error> {
		from_params::<(BlockNumber, Index)>(params)
			.and_then(|(number, index)| self.transaction(TransactionId::Location(number.into(), index.value())))
	}

	fn logs(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Filter,)>(params)
			.and_then(|(filter,)| {
				let logs = take_weak!(self.client).logs(filter.into())
					.into_iter()
					.map(From::from)
					.collect::<Vec<Log>>();
				to_value(&logs)
			})
	}

	fn work(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let c = take_weak!(self.client);
				let u = c.sealing_block().lock().unwrap();
				match *u {
					Some(ref b) => {
						let pow_hash = b.hash();
						let target = Ethash::difficulty_to_boundary(b.block().header().difficulty());
						let seed_hash = Ethash::get_seedhash(b.block().header().number());
						to_value(&(pow_hash, seed_hash, target))
					}
					_ => Err(Error::invalid_params())
				}
			},
			_ => Err(Error::invalid_params())
		}
	}

	fn submit_work(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H64, H256, H256)>(params).and_then(|(nonce, pow_hash, mix_hash)| {
//			trace!("Decoded: nonce={}, pow_hash={}, mix_hash={}", nonce, pow_hash, mix_hash);
			let c = take_weak!(self.client);
			let seal = vec![encode(&mix_hash).to_vec(), encode(&nonce).to_vec()];
			let r = c.submit_seal(pow_hash, seal);
			to_value(&r.is_ok())
		})
	}

	fn submit_hashrate(&self, params: Params) -> Result<Value, Error> {
		// TODO: Index should be U256.
		from_params::<(Index, H256)>(params).and_then(|(rate, id)| {
			self.hashrates.write().unwrap().insert(id, rate.value() as u64);
			to_value(&true)
		})
	}
}

/// Eth filter rpc implementation.
pub struct EthFilterClient {
	client: Weak<Client>,
	polls: Mutex<PollManager<PollFilter>>,
}

impl EthFilterClient {
	/// Creates new Eth filter client.
	pub fn new(client: &Arc<Client>) -> Self {
		EthFilterClient {
			client: Arc::downgrade(client),
			polls: Mutex::new(PollManager::new())
		}
	}
}

impl EthFilter for EthFilterClient {
	fn new_filter(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Filter,)>(params)
			.and_then(|(filter,)| {
				let mut polls = self.polls.lock().unwrap();
				let id = polls.create_poll(PollFilter::Logs(filter.into()), take_weak!(self.client).chain_info().best_block_number);
				to_value(&U256::from(id))
			})
	}

	fn new_block_filter(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let mut polls = self.polls.lock().unwrap();
				let id = polls.create_poll(PollFilter::Block, take_weak!(self.client).chain_info().best_block_number);
				to_value(&U256::from(id))
			},
			_ => Err(Error::invalid_params())
		}
	}

	fn new_pending_transaction_filter(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let mut polls = self.polls.lock().unwrap();
				let id = polls.create_poll(PollFilter::PendingTransaction, take_weak!(self.client).chain_info().best_block_number);
				to_value(&U256::from(id))
			},
			_ => Err(Error::invalid_params())
		}
	}

	fn filter_changes(&self, params: Params) -> Result<Value, Error> {
		let client = take_weak!(self.client);
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
				let info = self.polls.lock().unwrap().get_poll_info(&index.value()).cloned();
				match info {
					None => Ok(Value::Array(vec![] as Vec<Value>)),
					Some(info) => match info.filter {
						PollFilter::Block => {
							let current_number = client.chain_info().best_block_number;
							let hashes = (info.block_number..current_number).into_iter()
								.map(BlockId::Number)
								.filter_map(|id| client.block_hash(id))
								.collect::<Vec<H256>>();

							self.polls.lock().unwrap().update_poll(&index.value(), current_number);

							to_value(&hashes)
						},
						PollFilter::PendingTransaction => {
							// TODO: fix implementation once TransactionQueue is merged
							to_value(&vec![] as &Vec<H256>)
						},
						PollFilter::Logs(mut filter) => {
							filter.from_block = BlockId::Number(info.block_number);
							filter.to_block = BlockId::Latest;
							let logs = client.logs(filter)
								.into_iter()
								.map(From::from)
								.collect::<Vec<Log>>();

							let current_number = client.chain_info().best_block_number;
							self.polls.lock().unwrap().update_poll(&index.value(), current_number);

							to_value(&logs)
						}
					}
				}
			})
	}

	fn uninstall_filter(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Index,)>(params)
			.and_then(|(index,)| {
				self.polls.lock().unwrap().remove_poll(&index.value());
				to_value(&true)
			})
	}
}
