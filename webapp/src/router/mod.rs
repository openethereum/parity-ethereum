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

//! Router implementation
//! Processes request handling authorization and dispatching it to proper application.

mod api;
pub mod auth;

use std::sync::Arc;
use hyper;
use hyper::{header, server, uri};
use page::Page;
use apps::Pages;
use iron::request::Url;
use jsonrpc_http_server::ServerHandler;
use self::auth::{Authorization, Authorized};

pub struct Router<A: Authorization> {
	authorization: A,
	rpc: ServerHandler,
	api: api::RestApi,
	main_page: Box<Page>,
	pages: Arc<Pages>,
}

impl<A: Authorization> server::Handler for Router<A> {
	fn handle<'b, 'a>(&'a self, req: server::Request<'a, 'b>, res: server::Response<'a>) {
		let auth = self.authorization.handle(req, res);

		if let Authorized::Yes(req, res) = auth {
			let (path, req) = self.extract_request_path(req);
			match path {
				Some(ref url) if self.pages.contains_key(url) => {
					self.pages.get(url).unwrap().handle(req, res);
				}
				Some(ref url) if url == "api" => {
					self.api.handle(req, res);
				}
				_ if req.method == hyper::method::Method::Post => {
					self.rpc.handle(req, res)
				}
				_ => self.main_page.handle(req, res),
			}
		}
	}
}

impl<A: Authorization> Router<A> {
	pub fn new(rpc: ServerHandler, main_page: Box<Page>, pages: Pages, authorization: A) -> Self {
		let pages = Arc::new(pages);
		Router {
			authorization: authorization,
			rpc: rpc,
			api: api::RestApi { pages: pages.clone() },
			main_page: main_page,
			pages: pages,
		}
	}

	fn extract_url(&self, req: &server::Request) -> Option<Url> {
		match req.uri {
			uri::RequestUri::AbsoluteUri(ref url) => {
				match Url::from_generic_url(url.clone()) {
					Ok(url) => Some(url),
					_ => None,
				}
			}
			uri::RequestUri::AbsolutePath(ref path) => {
				// Attempt to prepend the Host header (mandatory in HTTP/1.1)
				let url_string = match req.headers.get::<header::Host>() {
					Some(ref host) => {
						format!("http://{}:{}{}", host.hostname, host.port.unwrap_or(80), path)
					}
					None => return None,
				};

				match Url::parse(&url_string) {
					Ok(url) => Some(url),
					_ => None,
				}
			}
			_ => None,
		}
	}

	fn extract_request_path<'a, 'b>(&self, mut req: server::Request<'a, 'b>) -> (Option<String>, server::Request<'a, 'b>) {
		let url = self.extract_url(&req);
		match url {
			Some(ref url) if url.path.len() > 1 => {
				let part = url.path[0].clone();
				let url = url.path[1..].join("/");
				req.uri = uri::RequestUri::AbsolutePath(url);
				(Some(part), req)
			}
			Some(url) => {
				let url = url.path.join("/");
				req.uri = uri::RequestUri::AbsolutePath(url);
				(None, req)
			}
			_ => {
				(None, req)
			}
		}
	}
}
