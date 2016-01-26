use std::sync::Arc;
use serde_json;
use jsonrpc_core::*;
use util::hash::*;
use ethcore::client::*;
use traits::{Eth, EthFilter};
use types::{Block, as_value, from_value};

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
			Params::None => Ok(as_value(&Address::new())),
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
			if let [ref h, Value::Bool(ref include_transactions)] = arr as &[Value] {
				if let Ok(hash) = from_value::<H256>(h.clone()) {
					return Ok(as_value(&Block::default()))
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
		Ok(Value::Array(vec![as_value(&self.client.chain_info().best_block_hash)]))
	}
}
