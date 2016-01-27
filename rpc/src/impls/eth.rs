use std::sync::Arc;
use jsonrpc_core::*;
use util::hash::*;
use util::uint::*;
use util::sha3::*;
use ethcore::client::*;
use ethcore::views::*;
use traits::{Eth, EthFilter};
use types::{Block, to_value, from_value};

pub struct EthClient {
	client: Arc<Client>,
}

impl EthClient {
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
			Params::None => Ok(to_value(&Address::new())),
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
		if let Params::Array(ref arr) = params {
			if let [ref h, Value::Bool(ref _include_txs)] = arr as &[Value] {
				if let Ok(hash) = from_value::<H256>(h.clone()) {
					return match (self.client.block_header(&hash), self.client.block_details(&hash)) {
						(Some(bytes), Some(details)) => {
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
								total_difficulty: details.total_difficulty,
								uncles: vec![],
								transactions: vec![]
							};
							Ok(to_value(&block))
						},
						_ => Ok(Value::Null),
					}
				}
			}
		}
		Err(Error::invalid_params())
	}
}

pub struct EthFilterClient {
	client: Arc<Client>
}

impl EthFilterClient {
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
		Ok(Value::Array(vec![to_value(&self.client.chain_info().best_block_hash)]))
	}
}
