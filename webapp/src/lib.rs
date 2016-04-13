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
#![warn(missing_docs)]
#![cfg_attr(feature="nightly", plugin(clippy))]

#[macro_use]
extern crate log;
extern crate hyper;
extern crate iron;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
extern crate ethcore_rpc as rpc;
extern crate parity_webapp;

use std::sync::Arc;
use self::jsonrpc_core::{IoDelegate, IoHandler};
use jsonrpc_http_server::ServerHandler;

mod apps;
mod page;
mod router;

use router::auth::{Authorization, HttpBasicAuth, NoAuth};

/// Http server.
pub struct WebappServer {
	handler: Arc<IoHandler>,
}

impl WebappServer {
	/// Construct new http server object
	pub fn new() -> Self {
		WebappServer { handler: Arc::new(IoHandler::new()) }
	}

	/// Add io delegate.
	pub fn add_delegate<D>(&self, delegate: IoDelegate<D>)
		where D: Send + Sync + 'static,
	{
		self.handler.add_delegate(delegate);
	}

	/// Asynchronously start server with no authentication,
	/// return result with `Listening` handle on success or an error.
	pub fn start_unsecure_http(&self, addr: &str, threads: usize) -> Result<Listening, WebappServerError> {
		self.start_http(addr, threads, NoAuth)
	}

	/// Asynchronously start server with `HTTP Basic Authentication`,
	/// return result with `Listening` handle on success or an error.
	pub fn start_basic_auth_http(&self, addr: &str, threads: usize, username: &str, password: &str) -> Result<Listening, WebappServerError> {
		self.start_http(addr, threads, HttpBasicAuth::single_user(username, password))
	}

	fn start_http<A: Authorization + 'static>(&self, addr: &str, threads: usize, authorization: A) -> Result<Listening, WebappServerError> {
		let addr = addr.to_owned();
		let handler = self.handler.clone();

		let cors_domain = jsonrpc_http_server::AccessControlAllowOrigin::Null;
		let rpc = ServerHandler::new(handler, cors_domain);
		let router = router::Router::new(rpc, apps::main_page(), apps::all_pages(), authorization);

		try!(hyper::Server::http(addr.as_ref() as &str))
			.handle_threads(router, threads)
			.map(|l| Listening { listening: l })
			.map_err(WebappServerError::from)
	}
}

/// Listening handle
pub struct Listening {
	listening: hyper::server::Listening,
}

impl Drop for Listening {
	fn drop(&mut self) {
		self.listening.close().unwrap();
	}
}

/// Webapp Server startup error
#[derive(Debug)]
pub enum WebappServerError {
	/// Wrapped `std::io::Error`
	IoError(std::io::Error),
	/// Other `hyper` error
	Other(hyper::error::Error),
}

impl From<hyper::error::Error> for WebappServerError {
	fn from(err: hyper::error::Error) -> Self {
		match err {
			hyper::error::Error::Io(e) => WebappServerError::IoError(e),
			e => WebappServerError::Other(e),
		}
	}
}
