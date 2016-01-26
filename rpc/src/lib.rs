#![feature(custom_derive, custom_attribute, plugin)]
#![feature(slice_patterns)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
extern crate ethcore_util as util;
extern crate ethcore;

use self::jsonrpc_core::{IoHandler, IoDelegate};

macro_rules! rpcerr {
	() => (Err(Error::internal_error()))
}

pub mod traits;
mod impls;
mod types;

pub use self::traits::{Web3, Eth, EthFilter, Net};
pub use self::impls::*;

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
