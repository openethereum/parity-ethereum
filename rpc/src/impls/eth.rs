//! Eth rpc implementation.
use std::sync::Arc;
use jsonrpc_core::*;
use util::hash::*;
use util::uint::*;
use util::sha3::*;
use ethcore::client::*;
use ethcore::views::*;
use traits::{Eth, EthFilter};
use types::Block;

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
	fn protocol_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(63)),
			_ => Err(Error::invalid_params())
		}
	}

	fn author(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => to_value(&Address::new()),
			_ => Err(Error::invalid_params())
		}
	}

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

	fn is_mining(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::Bool(false)),
			_ => Err(Error::invalid_params())
		}
	}

	fn hashrate(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(0)),
			_ => Err(Error::invalid_params())
		}
	}

	fn block_transaction_count(&self, _: Params) -> Result<Value, Error> {
		Ok(Value::U64(0))
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
