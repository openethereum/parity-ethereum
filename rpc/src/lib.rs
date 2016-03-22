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

//! Ethcore rpc.
#![warn(missing_docs)]
#![cfg_attr(feature="nightly", feature(custom_derive, custom_attribute, plugin))]
#![cfg_attr(feature="nightly", plugin(serde_macros, clippy))]

#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
extern crate ethcore_util as util;
extern crate ethcore;
extern crate ethsync;
extern crate ethminer;
extern crate hyper;
extern crate iron;
extern crate transient_hashmap;
extern crate parity_webapp;
extern crate parity_wallet;
extern crate parity_demo;
extern crate parity_mist;

use std::sync::Arc;
use std::thread;
use util::panics::PanicHandler;
use self::jsonrpc_core::{IoHandler, IoDelegate};
use jsonrpc_http_server::ServerHandler;
use iron::request::Url;
use parity_demo::AdminApp;
use parity_wallet::App as WalletApp;
use parity_mist::App as MistApp;

pub mod v1;
mod page;

use page::Page;

/// Http server.
pub struct RpcServer {
	handler: Arc<IoHandler>,
}

impl RpcServer {
	/// Construct new http server object with given number of threads.
	pub fn new() -> RpcServer {
		RpcServer {
			handler: Arc::new(IoHandler::new()),
		}
	}

	/// Add io delegate.
	pub fn add_delegate<D>(&self, delegate: IoDelegate<D>) where D: Send + Sync + 'static {
		self.handler.add_delegate(delegate);
	}

	/// Start server asynchronously in new thread and returns panic handler.
	pub fn start_http(&self, addr: &str, cors_domain: &str, threads: usize) -> Arc<PanicHandler> {
		let addr = addr.to_owned();
		let cors_domain = cors_domain.to_owned();
		let panic_handler = PanicHandler::new_in_arc();
		let ph = panic_handler.clone();
		let handler = self.handler.clone();

		thread::Builder::new().name("jsonrpc_http".to_string()).spawn(move || {
			let cors_domain = jsonrpc_http_server::AccessControlAllowOrigin::Value(cors_domain);
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
	admin: Page<AdminApp>,
	wallet: Page<WalletApp>,
	mist: Page<MistApp>,
}

impl Router {
	pub fn new(rpc: ServerHandler) -> Self {
		Router {
			rpc: rpc,
			admin: Page { app: AdminApp::default() },
			wallet: Page { app: WalletApp::default() },
			mist: Page { app: MistApp::default() },
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
						format!("http://{}:1234{}", host.hostname, path)
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
			Some(ref url) if url == "admin" => {
				self.admin.handle(req, res);
			},
			Some(ref url) if url == "wallet" => {
				self.wallet.handle(req, res);
			},
			Some(ref url) if url == "mist" => {
				self.mist.handle(req, res);
			},
			_ => self.rpc.handle(req, res),
		}
	}
}
