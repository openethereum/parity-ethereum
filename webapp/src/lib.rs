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
extern crate ethcore_util as util;
extern crate parity_webapp;

use std::sync::Arc;
use util::panics::PanicHandler;
use self::jsonrpc_core::{IoHandler, IoDelegate};
use jsonrpc_http_server::ServerHandler;

mod apps;
mod page;
mod router;

/// Http server.
pub struct WebappServer {
	handler: Arc<IoHandler>,
}

impl WebappServer {
	/// Construct new http server object
	pub fn new() -> Self {
		WebappServer {
			handler: Arc::new(IoHandler::new()),
		}
	}

	/// Add io delegate.
	pub fn add_delegate<D>(&self, delegate: IoDelegate<D>) where D: Send + Sync + 'static {
		self.handler.add_delegate(delegate);
	}

	/// Start server asynchronously and returns panic handler.
	pub fn start_http(&self, addr: &str, threads: usize) -> Arc<PanicHandler> {
		let addr = addr.to_owned();
		let panic_handler = PanicHandler::new_in_arc();
		let handler = self.handler.clone();

		let cors_domain = jsonrpc_http_server::AccessControlAllowOrigin::Null;
		let rpc = ServerHandler::new(handler, cors_domain);
		let router = router::Router::new(rpc, apps::all_pages());

		panic_handler.catch_panic(move || {
			hyper::Server::http(addr.as_ref() as &str).unwrap()
				.handle_threads(router, threads)
				.unwrap();
		}).unwrap();

		panic_handler
	}
}
