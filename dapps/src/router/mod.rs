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
use std::collections::HashMap;
use url::Host;
use hyper;
use hyper::{server, uri, header};
use hyper::{Next, Encoder, Decoder};
use hyper::net::HttpStream;
use apps;
use endpoint::{Endpoint, Endpoints, EndpointPath};
use self::url::Url;
use self::auth::{Authorization, Authorized};
use self::redirect::Redirection;

/// Special endpoints are accessible on every domain (every dapp)
#[derive(Debug, PartialEq, Hash, Eq)]
pub enum SpecialEndpoint {
	Rpc,
	Api,
	Utils,
	None,
}

pub struct Router<A: Authorization + 'static> {
	main_page: &'static str,
	endpoints: Arc<Endpoints>,
	special: Arc<HashMap<SpecialEndpoint, Box<Endpoint>>>,
	authorization: Arc<A>,
	handler: Box<server::Handler<HttpStream>>,
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
				let endpoint = extract_endpoint(&url);

				match endpoint {
					// First check special endpoints
					(ref path, ref endpoint) if self.special.contains_key(endpoint) => {
						self.special.get(endpoint).unwrap().to_handler(path.clone().unwrap_or_default())
					},
					// Then delegate to dapp
					(Some(ref path), _) if self.endpoints.contains_key(&path.app_id) => {
						self.endpoints.get(&path.app_id).unwrap().to_handler(path.clone())
					},
					// Redirection to main page
					_ if *req.method() == hyper::method::Method::Get => {
						Redirection::new(self.main_page)
					},
					// RPC by default
					_ => {
						self.special.get(&SpecialEndpoint::Rpc).unwrap().to_handler(EndpointPath::default())
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
		special: Arc<HashMap<SpecialEndpoint, Box<Endpoint>>>,
		authorization: Arc<A>) -> Self {

		let handler = special.get(&SpecialEndpoint::Rpc).unwrap().to_handler(EndpointPath::default());
		Router {
			main_page: main_page,
			endpoints: endpoints,
			special: special,
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

fn extract_endpoint(url: &Option<Url>) -> (Option<EndpointPath>, SpecialEndpoint) {
	fn special_endpoint(url: &Url) -> SpecialEndpoint {
		if url.path.len() <= 1 {
			return SpecialEndpoint::None;
		}

		match url.path[0].as_ref() {
			apps::RPC_PATH => SpecialEndpoint::Rpc,
			apps::API_PATH => SpecialEndpoint::Api,
			apps::UTILS_PATH => SpecialEndpoint::Utils,
			_ => SpecialEndpoint::None,
		}
	}

	match *url {
		Some(ref url) => match url.host {
			Host::Domain(ref domain) if domain.ends_with(DAPPS_DOMAIN) => {
				let len = domain.len() - DAPPS_DOMAIN.len();
				let id = domain[0..len].to_owned();

				(Some(EndpointPath {
					app_id: id,
					host: domain.clone(),
					port: url.port,
				}), special_endpoint(url))
			},
			_ if url.path.len() > 1 => {
				let id = url.path[0].clone();
				(Some(EndpointPath {
					app_id: id.clone(),
					host: format!("{}", url.host),
					port: url.port,
				}), special_endpoint(url))
			},
			_ => (None, special_endpoint(url)),
		},
		_ => (None, SpecialEndpoint::None)
	}
}

#[test]
fn should_extract_endpoint() {
	assert_eq!(extract_endpoint(&None), (None, SpecialEndpoint::None));

	// With path prefix
	assert_eq!(
		extract_endpoint(&Url::parse("http://localhost:8080/status/index.html").ok()),
		(Some(EndpointPath {
			app_id: "status".to_owned(),
			host: "localhost".to_owned(),
			port: 8080,
		}), SpecialEndpoint::None)
	);

	// With path prefix
	assert_eq!(
		extract_endpoint(&Url::parse("http://localhost:8080/rpc/").ok()),
		(Some(EndpointPath {
			app_id: "rpc".to_owned(),
			host: "localhost".to_owned(),
			port: 8080,
		}), SpecialEndpoint::Rpc)
	);

	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/parity-utils/inject.js").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
		}), SpecialEndpoint::Utils)
	);

	// By Subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/test.html").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
		}), SpecialEndpoint::None)
	);

	// RPC by subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/rpc/").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
		}), SpecialEndpoint::Rpc)
	);

	// API by subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/api/").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
		}), SpecialEndpoint::Api)
	);
}
