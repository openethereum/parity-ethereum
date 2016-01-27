//! Ethcore rpc.
#![warn(missing_docs)]
#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(serde_macros)]
#![plugin(clippy)]

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

pub mod v1;

/// Http server.
pub struct HttpServer {
	handler: IoHandler,
	threads: usize
}

impl HttpServer {
	/// Construct new http server object with given number of threads.
	pub fn new(threads: usize) -> HttpServer {
		HttpServer {
			handler: IoHandler::new(),
			threads: threads
		}
	}

	/// Add io delegate.
	pub fn add_delegate<D>(&mut self, delegate: IoDelegate<D>) where D: Send + Sync + 'static {
		self.handler.add_delegate(delegate);
	}

	/// Start server asynchronously in new thread
	pub fn start_async(self, addr: &str) {
		let server = jsonrpc_http_server::Server::new(self.handler, self.threads);
		server.start_async(addr)
	}
}
