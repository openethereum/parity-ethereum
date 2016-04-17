// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Ethcore Webapplications for Parity
//! ```
//! extern crate jsonrpc_core;
//! extern crate ethcore_webapp;
//!
//! use std::sync::Arc;
//! use jsonrpc_core::IoHandler;
//! use ethcore_webapp::*;
//!
//! struct SayHello;
//! impl MethodCommand for SayHello {
//! 	fn execute(&self, _params: Params) -> Result<Value, Error> {
//! 		Ok(Value::String("hello".to_string()))
//! 	}
//! }
//!
//! fn main() {
//! 	let io = IoHandler::new();
//! 	io.add_method("say_hello", SayHello);
//! 	let _server = Server::start_unsecure_http(
//!		&"127.0.0.1:3030".parse().unwrap(),
//!		Arc::new(io)
//!	);
//! }
//! ```
//!
#![warn(missing_docs)]
#![cfg_attr(feature="nightly", plugin(clippy))]

#[macro_use]
extern crate log;
extern crate url;
extern crate hyper;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
extern crate parity_webapp;

mod endpoint;
mod apps;
mod page;
mod router;
mod rpc;
mod api;

use std::sync::Arc;
use std::net::SocketAddr;
use jsonrpc_core::{IoHandler, IoDelegate};
use router::auth::{Authorization, NoAuth, HttpBasicAuth};

/// Webapps HTTP+RPC server build.
pub struct ServerBuilder {
	handler: Arc<IoHandler>,
}

impl ServerBuilder {
	/// Construct new webapps
	pub fn new() -> Self {
		ServerBuilder {
			handler: Arc::new(IoHandler::new())
		}
	}

	/// Add io delegate.
	pub fn add_delegate<D>(&self, delegate: IoDelegate<D>) where D: Send + Sync + 'static {
		self.handler.add_delegate(delegate);
	}

	/// Asynchronously start server with no authentication,
	/// returns result with `Server` handle on success or an error.
	pub fn start_unsecure_http(&self, addr: &SocketAddr) -> Result<Server, ServerError> {
		Server::start_http(addr, NoAuth, self.handler.clone())
	}

	/// Asynchronously start server with `HTTP Basic Authentication`,
	/// return result with `Server` handle on success or an error.
	pub fn start_basic_auth_http(&self, addr: &SocketAddr, username: &str, password: &str) -> Result<Server, ServerError> {
		Server::start_http(addr, HttpBasicAuth::single_user(username, password), self.handler.clone())
	}
}

/// Webapps HTTP server.
pub struct Server {
	server: Option<hyper::server::Listening>,
}

impl Server {
	fn start_http<A: Authorization + 'static>(addr: &SocketAddr, authorization: A, handler: Arc<IoHandler>) -> Result<Server, ServerError> {
		let endpoints = Arc::new(apps::all_endpoints());
		let authorization = Arc::new(authorization);
		let rpc_endpoint = Arc::new(rpc::rpc(handler));
		let api = Arc::new(api::RestApi::new(endpoints.clone()));

		try!(hyper::Server::http(addr))
			.handle(move |_| router::Router::new(
				apps::main_page(),
				endpoints.clone(),
				rpc_endpoint.clone(),
				api.clone(),
				authorization.clone(),
			))
			.map(|l| Server { server: Some(l) })
			.map_err(ServerError::from)
	}
}

impl Drop for Server {
	fn drop(&mut self) {
		self.server.take().unwrap().close()
	}
}

/// Webapp Server startup error
#[derive(Debug)]
pub enum ServerError {
	/// Wrapped `std::io::Error`
	IoError(std::io::Error),
	/// Other `hyper` error
	Other(hyper::error::Error),
}

impl From<hyper::error::Error> for ServerError {
	fn from(err: hyper::error::Error) -> Self {
		match err {
			hyper::error::Error::Io(e) => ServerError::IoError(e),
			e => ServerError::Other(e),
		}
	}
}
