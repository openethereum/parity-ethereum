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

pub mod auth;
mod host_validation;

use DAPPS_DOMAIN;
use std::sync::Arc;
use std::collections::HashMap;
use url::{Url, Host};
use hyper::{self, server, Next, Encoder, Decoder, Control, StatusCode};
use hyper::net::HttpStream;
use apps;
use apps::fetcher::ContentFetcher;
use endpoint::{Endpoint, Endpoints, EndpointPath};
use handlers::{Redirection, extract_url, ContentHandler};
use self::auth::{Authorization, Authorized};

/// Special endpoints are accessible on every domain (every dapp)
#[derive(Debug, PartialEq, Hash, Eq)]
pub enum SpecialEndpoint {
	Rpc,
	Api,
	Utils,
	None,
}

pub struct Router<A: Authorization + 'static> {
	control: Option<Control>,
	main_page: &'static str,
	endpoints: Arc<Endpoints>,
	fetch: Arc<ContentFetcher>,
	special: Arc<HashMap<SpecialEndpoint, Box<Endpoint>>>,
	authorization: Arc<A>,
	allowed_hosts: Option<Vec<String>>,
	handler: Box<server::Handler<HttpStream> + Send>,
}

impl<A: Authorization + 'static> server::Handler<HttpStream> for Router<A> {

	fn on_request(&mut self, req: server::Request<HttpStream>) -> Next {

		// Choose proper handler depending on path / domain
		let url = extract_url(&req);
		let endpoint = extract_endpoint(&url);
		let is_utils = endpoint.1 == SpecialEndpoint::Utils;

		// Validate Host header
		if let Some(ref hosts) = self.allowed_hosts {
			let is_valid = is_utils || host_validation::is_valid(&req, hosts, self.endpoints.keys().cloned().collect());
			if !is_valid {
				self.handler = host_validation::host_invalid_response();
				return self.handler.on_request(req);
			}
		}

		// Check authorization
		let auth = self.authorization.is_authorized(&req);
		if let Authorized::No(handler) = auth {
			self.handler = handler;
			return self.handler.on_request(req);
		}

		let control = self.control.take().expect("on_request is called only once; control is always defined at start; qed");
		self.handler = match endpoint {
			// First check special endpoints
			(ref path, ref endpoint) if self.special.contains_key(endpoint) => {
				self.special.get(endpoint)
					.expect("special known to contain key; qed")
					.to_async_handler(path.clone().unwrap_or_default(), control)
			},
			// Then delegate to dapp
			(Some(ref path), _) if self.endpoints.contains_key(&path.app_id) => {
				self.endpoints.get(&path.app_id)
					.expect("special known to contain key; qed")
					.to_async_handler(path.clone(), control)
			},
			// Try to resolve and fetch the dapp
			(Some(ref path), _) if self.fetch.contains(&path.app_id) => {
				self.fetch.to_async_handler(path.clone(), control)
			},
			// 404 for non-existent content
			(Some(ref path), _) if *req.method() == hyper::method::Method::Get => {
				let address = apps::redirection_address(path.using_dapps_domains, self.main_page);
				Box::new(ContentHandler::error(
					StatusCode::NotFound,
					"404 Not Found",
					"Requested content was not found.",
					Some(&format!("Go back to the <a href=\"{}\">Home Page</a>.", address))
				))
			},
			// Redirect any GET request to home.
			_ if *req.method() == hyper::method::Method::Get => {
				let address = apps::redirection_address(false, self.main_page);
				Redirection::boxed(address.as_str())
			},
			// RPC by default
			_ => {
				self.special.get(&SpecialEndpoint::Rpc)
					.expect("RPC endpoint always stored; qed")
					.to_async_handler(EndpointPath::default(), control)
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
		control: Control,
		main_page: &'static str,
		content_fetcher: Arc<ContentFetcher>,
		endpoints: Arc<Endpoints>,
		special: Arc<HashMap<SpecialEndpoint, Box<Endpoint>>>,
		authorization: Arc<A>,
		allowed_hosts: Option<Vec<String>>,
		) -> Self {

		let handler = special.get(&SpecialEndpoint::Utils)
			.expect("Utils endpoint always stored; qed")
			.to_handler(EndpointPath::default());
		Router {
			control: Some(control),
			main_page: main_page,
			endpoints: endpoints,
			fetch: content_fetcher,
			special: special,
			authorization: authorization,
			allowed_hosts: allowed_hosts,
			handler: handler,
		}
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
					using_dapps_domains: true,
				}), special_endpoint(url))
			},
			_ if url.path.len() > 1 => {
				let id = url.path[0].clone();
				(Some(EndpointPath {
					app_id: id.clone(),
					host: format!("{}", url.host),
					port: url.port,
					using_dapps_domains: false,
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
			using_dapps_domains: false,
		}), SpecialEndpoint::None)
	);

	// With path prefix
	assert_eq!(
		extract_endpoint(&Url::parse("http://localhost:8080/rpc/").ok()),
		(Some(EndpointPath {
			app_id: "rpc".to_owned(),
			host: "localhost".to_owned(),
			port: 8080,
			using_dapps_domains: false,
		}), SpecialEndpoint::Rpc)
	);

	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/parity-utils/inject.js").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::Utils)
	);

	// By Subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/test.html").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::None)
	);

	// RPC by subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/rpc/").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::Rpc)
	);

	// API by subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.parity/api/").ok()),
		(Some(EndpointPath {
			app_id: "my.status".to_owned(),
			host: "my.status.parity".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::Api)
	);
}
