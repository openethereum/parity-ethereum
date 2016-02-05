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
use jsonrpc_core::*;
use util::hash::*;
use util::uint::*;
use util::sha3::*;
use ethcore::client::*;
use ethcore::views::*;
use v1::traits::{Eth, EthFilter};
use v1::types::{Block, SyncStatus};

/// Eth rpc implementation.
pub struct EthClient {
	client: Arc<Client>,
}

impl EthClient {
	/// Creates new EthClient.
	pub fn new(client: Arc<Client>) -> Self {
		EthClient {
			client: client
		}
	}
}

impl Eth for EthClient {
	// TODO: do not hardcode protocol version
	fn protocol_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(63)),
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: do no hardcode default sync status
	fn syncing(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&SyncStatus::default()),
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
			Params::None => Ok(Value::U64(0)),
			_ => Err(Error::invalid_params())
		}
	}

	// TODO: do not hardode gas_price
	fn gas_price(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(0)),
			_ => Err(Error::invalid_params())
		}
	}

	fn block_number(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(self.client.chain_info().best_block_number)),
			_ => Err(Error::invalid_params())
		}
	}

	fn block_transaction_count(&self, params: Params) -> Result<Value, Error> {
		match from_params::<H256>(params) {
			Ok(hash) => match self.client.block(&hash) {
				Some(bytes) => to_value(&BlockView::new(&bytes).transactions_count()),
				None => Ok(Value::Null)
			},
			Err(err) => Err(err)
		}
	}

	fn block_uncles_count(&self, params: Params) -> Result<Value, Error> {
		match from_params::<H256>(params) {
			Ok(hash) => match self.client.block(&hash) {
				Some(bytes) => to_value(&BlockView::new(&bytes).uncles_count()),
				None => Ok(Value::Null)
			},
			Err(err) => Err(err)
		}
	}

	fn block(&self, params: Params) -> Result<Value, Error> {
		match from_params::<(H256, bool)>(params) {
			Ok((hash, _include_txs)) => match (self.client.block_header(&hash), self.client.block_total_difficulty(&hash)) {
				(Some(bytes), Some(total_difficulty)) => {
					let view = HeaderView::new(&bytes);
					let block = Block {
						hash: view.sha3(),
						parent_hash: view.parent_hash(),
						uncles_hash: view.uncles_hash(),
						author: view.author(),
						miner: view.author(),
						state_root: view.state_root(),
						transactions_root: view.transactions_root(),
						receipts_root: view.receipts_root(),
						number: U256::from(view.number()),
						gas_used: view.gas_used(),
						gas_limit: view.gas_limit(),
						logs_bloom: view.log_bloom(),
						timestamp: U256::from(view.timestamp()),
						difficulty: view.difficulty(),
						total_difficulty: total_difficulty,
						uncles: vec![],
						transactions: vec![]
					};
					to_value(&block)
				},
				_ => Ok(Value::Null)
			},
			Err(err) => Err(err)
		}
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
