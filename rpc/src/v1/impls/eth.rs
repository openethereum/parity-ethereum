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
use std::sync::Arc;
use ethsync::{EthSync, SyncState};
use jsonrpc_core::*;
use util::hash::*;
use util::uint::*;
use util::sha3::*;
use ethcore::client::*;
use ethcore::views::*;
use ethcore::ethereum::denominations::shannon;
use v1::traits::{Eth, EthFilter};
use v1::types::{Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo, Transaction, OptionalValue, Index};

/// Eth rpc implementation.
pub struct EthClient {
	client: Arc<Client>,
	sync: Arc<EthSync>
}

impl EthClient {
	/// Creates new EthClient.
	pub fn new(client: Arc<Client>, sync: Arc<EthSync>) -> Self {
		EthClient {
			client: client,
			sync: sync
		}
	}

	fn block(&self, id: BlockId, include_txs: bool) -> Result<Value, Error> {
		match (self.client.block(id.clone()), self.client.block_total_difficulty(id)) {
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
		match self.client.transaction(id) {
			Some(t) => to_value(&Transaction::from(t)),
			None => Ok(Value::Null)
		}
	}
}

impl Eth for EthClient {
	fn protocol_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&U256::from(self.sync.status().protocol_version)),
			_ => Err(Error::invalid_params())
		}
	}

	fn syncing(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => {
				let status = self.sync.status();
				let res = match status.state {
					SyncState::NotSynced | SyncState::Idle => SyncStatus::None,
					SyncState::Waiting | SyncState::Blocks | SyncState::NewBlocks => SyncStatus::Info(SyncInfo {
						starting_block: U256::from(status.start_block_number),
						current_block: U256::from(self.client.chain_info().best_block_number),
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
			Params::None => Ok(Value::Bool(false)),
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: return real hashrate once we have mining
	fn hashrate(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&U256::zero()),
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
			Params::None => to_value(&U256::from(self.client.chain_info().best_block_number)),
			_ => Err(Error::invalid_params())
		}
	}

	fn block_transaction_count(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| match self.client.block(BlockId::Hash(hash)) {
				Some(bytes) => to_value(&BlockView::new(&bytes).transactions_count()),
				None => Ok(Value::Null)
			})
	}

	fn block_uncles_count(&self, params: Params) -> Result<Value, Error> {
		from_params::<(H256,)>(params)
			.and_then(|(hash,)| match self.client.block(BlockId::Hash(hash)) {
				Some(bytes) => to_value(&BlockView::new(&bytes).uncles_count()),
				None => Ok(Value::Null)
			})
	}

	// TODO: do not ignore block number param
	fn code_at(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Address, BlockNumber)>(params)
			.and_then(|(address, _block_number)| to_value(&self.client.code(&address).map_or_else(Bytes::default, Bytes::new)))
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
}


/// Eth filter rpc implementation.
pub struct EthFilterClient {
	client: Arc<Client>
}

impl EthFilterClient {
	/// Creates new Eth filter client.
	pub fn new(client: Arc<Client>) -> Self {
		EthFilterClient {
			client: client
		}
	}
}

impl EthFilter for EthFilterClient {
	fn new_block_filter(&self, _params: Params) -> Result<Value, Error> {
		Ok(Value::U64(0))
	}

	fn new_pending_transaction_filter(&self, _params: Params) -> Result<Value, Error> {
		Ok(Value::U64(1))
	}

	fn filter_changes(&self, _: Params) -> Result<Value, Error> {
		to_value(&self.client.chain_info().best_block_hash).map(|v| Value::Array(vec![v]))
	}
}
