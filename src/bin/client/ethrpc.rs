extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

use std::sync::{Arc, RwLock};
use rustc_serialize::hex::ToHex;
use self::jsonrpc_core::{IoHandler, IoDelegate, Params, Value, Error, ErrorCode};
use ethcore::client::*;
use util::hash::*;

macro_rules! rpcerr {
	() => (Err(Error::internal_error()))
}

/// Web3 rpc interface.
pub trait Web3: Sized + Send + Sync + 'static {
	/// Returns current client version.
	fn client_version(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("web3_clientVersion", Web3::client_version);
		delegate
	}
}


/// Eth rpc interface.
pub trait Eth: Sized + Send + Sync + 'static {
	/// Returns protocol version.
	fn protocol_version(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns block author.
	fn author(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns current gas_price.
	fn gas_price(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns highest block number.
	fn block_number(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns block with given index / hash.
	fn block(&self, _: Params) -> Result<Value, Error> { rpcerr!() }
	
	/// Returns true if client is actively mining new blocks.
	fn is_mining(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns the number of hashes per second that the node is mining with.
	fn hashrate(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns the number of transactions in a block.
	fn block_transaction_count(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("eth_protocolVersion", Eth::protocol_version);
		delegate.add_method("eth_coinbase", Eth::author);
		delegate.add_method("eth_gasPrice", Eth::gas_price);
		delegate.add_method("eth_blockNumber", Eth::block_number);
		delegate.add_method("eth_getBlockByNumber", Eth::block);
		delegate.add_method("eth_mining", Eth::is_mining);
		delegate.add_method("eth_hashrate", Eth::hashrate);
		delegate.add_method("eth_getBlockTransactionCountByNumber", Eth::block_transaction_count);
		delegate
	}
}

/// Net rpc interface.
pub trait Net: Sized + Send + Sync + 'static {
	/// Returns protocol version.
	fn version(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	/// Returns number of peers connected to node.
	fn peer_count(&self, _: Params) -> Result<Value, Error> { rpcerr!() }

	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("peer_count", Net::version);
		delegate.add_method("net_version", Net::version);
		delegate
	}
}

pub struct Web3Client;

impl Web3Client {
	pub fn new() -> Self { Web3Client }
}

impl Web3 for Web3Client {
	fn client_version(&self, params: Params) -> Result<Value, Error> {
		match params {
			Params::None => Ok(Value::String("parity/0.1.0/-/rust1.7-nightly".to_string())),
			_ => Err(Error::invalid_params())
		}
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

	pub fn add_delegate<D>(&mut self, delegate: IoDelegate<D>) where D: Send + Sync + 'static {
		self.handler.add_delegate(delegate);
	}

	pub fn start_async(self, addr: &str) {
		let server = jsonrpc_http_server::Server::new(self.handler, self.threads);
		server.start_async(addr)
	}
}
