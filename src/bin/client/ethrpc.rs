extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

use std::sync::{Arc, RwLock};
use self::jsonrpc_core::{IoHandler, IoDelegate, Params, Value, Error, ErrorCode};
use ethcore::client::*;

macro_rules! rpcerr {
	() => (Err(Error::new(ErrorCode::InternalError)))
}

/// This could be a part of `jsonrpc_core`. Unfortunately, 
/// "only traits defined in the current crate can be implemented for a type parameter".
pub trait IntoDelegate<T> where T: Send + Sync + 'static {
	/// This function should be called to translate custom type into IoDelegate
	fn into_delegate(self) -> IoDelegate<T>;
}

/// eth rpc interface
pub trait Eth {
	/// returns protocol version
	fn protocol_version(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// returns block author
	fn author(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// returns current gas_price
	fn gas_price(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// returns highest block number
	fn block_number(&self, _: Params) -> Result<Value, Error> { rpcerr!() }
}

impl<D> IntoDelegate<D> for D where D: Eth + Send + Sync + 'static {
	fn into_delegate(self) -> IoDelegate<D> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("eth_protocolVersion", D::protocol_version);
		delegate.add_method("eth_coinbase", D::author);
		delegate.add_method("eth_gasPrice", D::gas_price);
		delegate.add_method("eth_blockNumber", D::block_number);
		delegate
	}
}

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
	fn block_number(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(self.client.read().unwrap().chain_info().best_block_number)),
			_ => Err(Error::new(ErrorCode::InvalidParams)),
		}
	}
}


pub struct HttpServer {
	handler: IoHandler,
	threads: usize
}

impl HttpServer {
	pub fn new(threads: usize) -> HttpServer {
		HttpServer {
			handler: IoHandler::new(),
			threads: threads
		}
	}

	pub fn add_delegate<I, D>(&mut self, delegate: I) where D: Send + Sync + 'static, I: IntoDelegate<D> {
		self.handler.add_delegate(delegate.into_delegate());
	}

	pub fn start_async(self, addr: &str) {
		let server = jsonrpc_http_server::Server::new(self.handler, self.threads);
		server.start_async(addr)
	}
}
