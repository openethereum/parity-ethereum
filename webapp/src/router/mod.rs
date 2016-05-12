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

mod url;
mod redirect;
pub mod auth;

use DAPPS_DOMAIN;
use std::sync::Arc;
use url::Host;
use hyper;
use hyper::{server, uri, header};
use hyper::{Next, Encoder, Decoder};
use hyper::net::HttpStream;
use endpoint::{Endpoint, Endpoints, HostInfo};
use self::url::Url;
use self::auth::{Authorization, Authorized};
use self::redirect::Redirection;

pub struct Router<A: Authorization + 'static> {
	main_page: &'static str,
	endpoints: Arc<Endpoints>,
	rpc: Arc<Box<Endpoint>>,
	api: Arc<Box<Endpoint>>,
	authorization: Arc<A>,
	handler: Box<server::Handler<HttpStream>>,
}

#[derive(Debug, PartialEq)]
struct AppId {
	id: String,
	prefix: String,
	is_rpc: bool,
}

impl<A: Authorization + 'static> server::Handler<HttpStream> for Router<A> {

	fn on_request(&mut self, req: server::Request) -> Next {
		// Check authorization
		let auth = self.authorization.is_authorized(&req);

		// Choose proper handler depending on path / domain
		self.handler = match auth {
			Authorized::No(handler) => handler,
			Authorized::Yes => {
				let url = extract_url(&req);
				let app_id = extract_app_id(&url);
				let host = url.map(|u| HostInfo {
					host: u.host,
					port: u.port
				});

				match app_id {
					// First check RPC requests
					Some(ref app_id) if app_id.is_rpc && *req.method() != hyper::method::Method::Get => {
						self.rpc.to_handler("", host)
					},
					// Then delegate to dapp
					Some(ref app_id) if self.endpoints.contains_key(&app_id.id) => {
						self.endpoints.get(&app_id.id).unwrap().to_handler(&app_id.prefix, host)
					},
					Some(ref app_id) if app_id.id == "api" => {
						self.api.to_handler(&app_id.prefix, host)
					},
					// Redirection to main page
					_ if *req.method() == hyper::method::Method::Get => {
						Redirection::new(self.main_page)
					},
					// RPC by default
					_ => {
						self.rpc.to_handler("", host)
					}
				}
			}
		};

		// Delegate on_request to proper handler
		self.handler.on_request(req)
	}

	/// This event occurs each time the `Request` is ready to be read from.
	fn on_request_readable(&mut self, decoder: &mut Decoder<HttpStream>) -> Next {
		self.handler.on_request_readable(decoder)
	}

	/// This event occurs after the first time this handled signals `Next::write()`.
	fn on_response(&mut self, response: &mut server::Response) -> Next {
		self.handler.on_response(response)
	}

	/// This event occurs each time the `Response` is ready to be written to.
	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		self.handler.on_response_writable(encoder)
	}
}

impl<A: Authorization> Router<A> {
	pub fn new(
		main_page: &'static str,
		endpoints: Arc<Endpoints>,
		rpc: Arc<Box<Endpoint>>,
		api: Arc<Box<Endpoint>>,
		authorization: Arc<A>) -> Self {

		let handler = rpc.to_handler("", None);
		Router {
			main_page: main_page,
			endpoints: endpoints,
			rpc: rpc,
			api: api,
			authorization: authorization,
			handler: handler,
		}
	}
}

fn extract_url(req: &server::Request) -> Option<Url> {
	match *req.uri() {
		uri::RequestUri::AbsoluteUri(ref url) => {
			match Url::from_generic_url(url.clone()) {
				Ok(url) => Some(url),
				_ => None,
			}
		},
		uri::RequestUri::AbsolutePath(ref path) => {
			// Attempt to prepend the Host header (mandatory in HTTP/1.1)
			let url_string = match req.headers().get::<header::Host>() {
				Some(ref host) => {
					format!("http://{}:{}{}", host.hostname, host.port.unwrap_or(80), path)
				},
				None => return None,
			};

			match Url::parse(&url_string) {
				Ok(url) => Some(url),
				_ => None,
			}
		},
		_ => None,
	}
}

fn extract_app_id(url: &Option<Url>) -> Option<AppId> {
	fn is_rpc(url: &Url) -> bool {
		url.path.len() > 1 && url.path[0] == "rpc"
	}

	match *url {
		Some(ref url) => match url.host {
			Host::Domain(ref domain) if domain.ends_with(DAPPS_DOMAIN) => {
				let len = domain.len() - DAPPS_DOMAIN.len();
				let id = domain[0..len].to_owned();

				Some(AppId {
					id: id,
					prefix: "".to_owned(),
					is_rpc: is_rpc(url),
				})
			},
			_ if url.path.len() > 1 => {
				let id = url.path[0].clone();
				Some(AppId {
					id: id.clone(),
					prefix: "/".to_owned() + &id,
					is_rpc: is_rpc(url),
				})
			},
			_ => None,
		},
		_ => None
	}
}

#[test]
fn should_extract_app_id() {
	assert_eq!(extract_app_id(&None), None);

	// With path prefix
	assert_eq!(
		extract_app_id(&Url::parse("http://localhost:8080/status/index.html").ok()),
		Some(AppId {
			id: "status".to_owned(),
			prefix: "/status".to_owned(),
			is_rpc: false,
		}));

	// With path prefix
	assert_eq!(
		extract_app_id(&Url::parse("http://localhost:8080/rpc/").ok()),
		Some(AppId {
			id: "rpc".to_owned(),
			prefix: "/rpc".to_owned(),
			is_rpc: true,
		}));

	// By Subdomain
	assert_eq!(
		extract_app_id(&Url::parse("http://my.status.dapp/test.html").ok()),
		Some(AppId {
			id: "my.status".to_owned(),
			prefix: "".to_owned(),
			is_rpc: false,
		}));

	// RPC by subdomain
	assert_eq!(
		extract_app_id(&Url::parse("http://my.status.dapp/rpc/").ok()),
		Some(AppId {
			id: "my.status".to_owned(),
			prefix: "".to_owned(),
			is_rpc: true,
		}));

}
