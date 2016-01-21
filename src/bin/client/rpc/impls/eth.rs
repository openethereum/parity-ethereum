use std::sync::{Arc, RwLock};
use rustc_serialize::hex::ToHex;
use util::hash::*;
use ethcore::client::*;
use rpc::jsonrpc_core::*;
use rpc::Eth;

pub struct EthClient {
	client: Arc<RwLock<Client>>,
}

impl EthClient {
	pub fn new(client: Arc<RwLock<Client>>) -> Self {
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

	fn block_number(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(self.client.read().unwrap().chain_info().best_block_number)),
			_ => Err(Error::invalid_params())
		}
	}

	fn is_mining(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::Bool(false)),
			_ => Err(Error::invalid_params())
		}
	}
}
