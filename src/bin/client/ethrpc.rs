extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

use std::sync::{Arc, RwLock};
use self::jsonrpc_core::{IoHandler, IoDelegate, Params, Value, Error, ErrorCode};
use ethcore::client::*;

struct Eth {
	client: Arc<RwLock<Client>>
}

impl Eth {
	fn new(client: Arc<RwLock<Client>>) -> Self {
		Eth {
			client: client
		}
	}

	fn block_number(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::U64(self.client.read().unwrap().chain_info().best_block_number)),
			_ => Err(Error::new(ErrorCode::InvalidParams)),
		}
	}
}

struct EthRpc;

impl EthRpc {
	fn build_handler(client: Arc<RwLock<Client>>) -> IoHandler {
		let mut handler = IoHandler::new();
		let mut eth = IoDelegate::new(Arc::new(Eth::new(client)));
		eth.add_method("eth_blockNumber", Eth::block_number);
		handler.add_delegate(eth);
		handler
	}
}

pub struct HttpServer {
	server: jsonrpc_http_server::Server
}

impl HttpServer {
	pub fn new(client: Arc<RwLock<Client>>, threads: usize) -> HttpServer {
		HttpServer {
			server: jsonrpc_http_server::Server::new(EthRpc::build_handler(client), threads)
		}
	}

	pub fn start_async(self, addr: &str) {
		self.server.start_async(addr)
	}
}
