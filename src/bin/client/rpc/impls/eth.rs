use std::sync::{Arc, RwLock};
use rustc_serialize::hex::ToHex;
use util::hash::*;
use ethcore::client::*;
use rpc::jsonrpc_core::*;
use rpc::{Eth, EthFilter};

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
			Params::None => Ok(Value::String(Address::new().to_hex())),
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
		println!("filter changes: {:?}", self.client.chain_info().best_block_hash.to_hex());
		Ok(Value::Array(vec![Value::String(self.client.chain_info().best_block_hash.to_hex())]))
	}
}
