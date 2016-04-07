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

use rpc::v1::*;
use std::sync::Arc;
use std::thread;

use util::panics::PanicHandler;
use iron::request::Url;
use self::jsonrpc_core::{IoHandler, IoDelegate};
use jsonrpc_http_server::ServerHandler;

/// Http server.
pub struct WebappServer {
	handler: Arc<IoHandler>,
}

impl WebappServer {
	/// Construct new http server object
	pub fn new() -> Self {
		let server = WebappServer {
			handler: Arc::new(IoHandler::new()),
		};
		// TODO add more
		server.add_delegate(Web3Client::new().to_delegate());

		server
	}

	/// Add io delegate.
	fn add_delegate<D>(&self, delegate: IoDelegate<D>) where D: Send + Sync + 'static {
		self.handler.add_delegate(delegate);
	}

	/// Start server asynchronously in new thread and returns panic handler.
	pub fn start_http(&self, addr: &str, threads: usize) -> Arc<PanicHandler> {
		let addr = addr.to_owned();
		let panic_handler = PanicHandler::new_in_arc();
		let ph = panic_handler.clone();
		let handler = self.handler.clone();

		thread::Builder::new().name("jsonrpc_http".to_string()).spawn(move || {
			let cors_domain = jsonrpc_http_server::AccessControlAllowOrigin::Null;
			let rpc = ServerHandler::new(handler, cors_domain);
			let router = Router::new(rpc);

			ph.catch_panic(move || {
				hyper::Server::http(addr.as_ref() as &str).unwrap()
					.handle_threads(router, threads)
					.unwrap();
			}).unwrap()
		}).expect("Error while creating jsonrpc http thread");

		panic_handler
	}
}


struct Router {
	rpc: ServerHandler,
	// admin: Page<AdminApp>,
	// wallet: Page<WalletApp>,
	// mist: Page<MistApp>,
}

impl Router {
	pub fn new(rpc: ServerHandler) -> Self {
		Router {
			rpc: rpc,
			// admin: Page { app: AdminApp::default() },
			// wallet: Page { app: WalletApp::default() },
			// mist: Page { app: MistApp::default() },
		}
	}

	fn extract_url(req: &hyper::server::Request) -> Option<Url> {
		match req.uri {
			hyper::uri::RequestUri::AbsoluteUri(ref url) => {
				match Url::from_generic_url(url.clone()) {
					Ok(url) => Some(url),
					_ => None,
				}
			},
			hyper::uri::RequestUri::AbsolutePath(ref path) => {
				// Attempt to prepend the Host header (mandatory in HTTP/1.1)
				let url_string = match req.headers.get::<hyper::header::Host>() {
					Some(ref host) => {
						format!("http://{}:{}{}", host.hostname, host.port.unwrap_or(80), path)
					},
					None => return None
				};

				match Url::parse(&url_string) {
					Ok(url) => Some(url),
					_ => None,
				}
			}
			_ => None,
		}
	}

	fn extract_request_path<'a, 'b>(mut req: hyper::server::Request<'a, 'b>) -> (Option<String>, hyper::server::Request<'a, 'b>) {
		let url = Router::extract_url(&req);
		match url {
			Some(url) => {
				let part = url.path[0].clone();
				let url = url.path[1..].join("/");
				req.uri = hyper::uri::RequestUri::AbsolutePath(url);
				(Some(part), req)
			},
			None => {
				(None, req)
			}
		}
	}
}

impl hyper::server::Handler for Router {
	fn handle<'b, 'a>(&'a self, req: hyper::server::Request<'a, 'b>, res: hyper::server::Response<'a>) {
		let (path, req) = Router::extract_request_path(req);
		match path {
			// Some(ref url) if url == "admin" => {
			// 	self.admin.handle(req, res);
			// },
			// Some(ref url) if url == "wallet" => {
			// 	self.wallet.handle(req, res);
			// },
			// Some(ref url) if url == "mist" => {
			// 	self.mist.handle(req, res);
			// },
			_ => self.rpc.handle(req, res),
		}
	}
}
