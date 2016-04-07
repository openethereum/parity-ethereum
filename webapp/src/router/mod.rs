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

use hyper;
use page::Page;
use apps::Pages;
use iron::request::Url;
use jsonrpc_http_server::ServerHandler;

pub struct Router {
	rpc: ServerHandler,
	main_page: Box<Page>,
	pages: Pages,
}

impl hyper::server::Handler for Router {
	fn handle<'b, 'a>(&'a self, req: hyper::server::Request<'a, 'b>, res: hyper::server::Response<'a>) {
		let (path, req) = Router::extract_request_path(req);
		match path {
			Some(ref url) if self.pages.contains_key(url) => {
				self.pages.get(url).unwrap().handle(req, res);
			}
			_ if req.method == hyper::method::Method::Post => {
				self.rpc.handle(req, res)
			},
			_ => self.main_page.handle(req, res),
		}
	}
}

impl Router {
	pub fn new(rpc: ServerHandler, main_page: Box<Page>, pages: Pages) -> Self {
		Router {
			rpc: rpc,
			main_page: main_page,
			pages: pages,
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
			Some(ref url) if url.path.len() > 1 => {
				let part = url.path[0].clone();
				let url = url.path[1..].join("/");
				req.uri = hyper::uri::RequestUri::AbsolutePath(url);
				(Some(part), req)
			},
			Some(url) => {
				let url = url.path.join("/");
				req.uri = hyper::uri::RequestUri::AbsolutePath(url);
				(None, req)
			},
			_ => {
				(None, req)
			}
		}
	}
}
