// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use address;
use std::cmp;
use std::sync::Arc;
use std::collections::HashMap;
use url::{Url, Host};
use hyper::{self, server, header, Next, Encoder, Decoder, Control, StatusCode};
use hyper::net::HttpStream;
use apps::{self, DAPPS_DOMAIN};
use apps::fetcher::Fetcher;
use endpoint::{Endpoint, Endpoints, EndpointPath};
use handlers::{self, Redirection, ContentHandler};
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
	signer_address: Option<(String, u16)>,
	endpoints: Arc<Endpoints>,
	fetch: Arc<Fetcher>,
	special: Arc<HashMap<SpecialEndpoint, Box<Endpoint>>>,
	authorization: Arc<A>,
	allowed_hosts: Option<Vec<String>>,
	handler: Box<server::Handler<HttpStream> + Send>,
}

impl<A: Authorization + 'static> server::Handler<HttpStream> for Router<A> {

	fn on_request(&mut self, req: server::Request<HttpStream>) -> Next {
		// Choose proper handler depending on path / domain
		let url = handlers::extract_url(&req);
		let endpoint = extract_endpoint(&url);
		let referer = extract_referer_endpoint(&req);
		let is_utils = endpoint.1 == SpecialEndpoint::Utils;
		let is_get_request = *req.method() == hyper::Method::Get;

		trace!(target: "dapps", "Routing request to {:?}. Details: {:?}", url, req);

		// Validate Host header
		if let Some(ref hosts) = self.allowed_hosts {
			trace!(target: "dapps", "Validating host headers against: {:?}", hosts);
			let is_valid = is_utils || host_validation::is_valid(&req, hosts, self.endpoints.keys().cloned().collect());
			if !is_valid {
				debug!(target: "dapps", "Rejecting invalid host header.");
				self.handler = host_validation::host_invalid_response();
				return self.handler.on_request(req);
			}
		}

		trace!(target: "dapps", "Checking authorization.");
		// Check authorization
		let auth = self.authorization.is_authorized(&req);
		if let Authorized::No(handler) = auth {
			debug!(target: "dapps", "Authorization denied.");
			self.handler = handler;
			return self.handler.on_request(req);
		}


		let control = self.control.take().expect("on_request is called only once; control is always defined at start; qed");
		debug!(target: "dapps", "Handling endpoint request: {:?}", endpoint);
		self.handler = match (endpoint.0, endpoint.1, referer) {
			// Handle invalid web requests that we can recover from
			(ref path, SpecialEndpoint::None, Some((ref referer, ref referer_url)))
				if referer.app_id == apps::WEB_PATH
					&& self.endpoints.contains_key(apps::WEB_PATH)
					&& !is_web_endpoint(path)
				=>
			{
				trace!(target: "dapps", "Redirecting to correct web request: {:?}", referer_url);
				let len = cmp::min(referer_url.path.len(), 2); // /web/<encoded>/
				let base = referer_url.path[..len].join("/");
				let requested = url.map(|u| u.path.join("/")).unwrap_or_default();
				Redirection::boxed(&format!("/{}/{}", base, requested))
			},
			// First check special endpoints
			(ref path, ref endpoint, _) if self.special.contains_key(endpoint) => {
				trace!(target: "dapps", "Resolving to special endpoint.");
				self.special.get(endpoint)
					.expect("special known to contain key; qed")
					.to_async_handler(path.clone().unwrap_or_default(), control)
			},
			// Then delegate to dapp
			(Some(ref path), _, _) if self.endpoints.contains_key(&path.app_id) => {
				trace!(target: "dapps", "Resolving to local/builtin dapp.");
				self.endpoints.get(&path.app_id)
					.expect("endpoints known to contain key; qed")
					.to_async_handler(path.clone(), control)
			},
			// Try to resolve and fetch the dapp
			(Some(ref path), _, _) if self.fetch.contains(&path.app_id) => {
				trace!(target: "dapps", "Resolving to fetchable content.");
				self.fetch.to_async_handler(path.clone(), control)
			},
			// NOTE [todr] /home is redirected to home page since some users may have the redirection cached
			// (in the past we used 301 instead of 302)
			// It should be safe to remove it in (near) future.
			//
			// 404 for non-existent content
			(Some(ref path), _, _) if is_get_request && path.app_id != "home" => {
				trace!(target: "dapps", "Resolving to 404.");
				Box::new(ContentHandler::error(
					StatusCode::NotFound,
					"404 Not Found",
					"Requested content was not found.",
					None,
					self.signer_address.clone(),
				))
			},
			// Redirect any other GET request to signer.
			_ if is_get_request => {
				if let Some(ref signer_address) = self.signer_address {
					trace!(target: "dapps", "Redirecting to signer interface.");
					Redirection::boxed(&format!("http://{}", address(signer_address)))
				} else {
					trace!(target: "dapps", "Signer disabled, returning 404.");
					Box::new(ContentHandler::error(
						StatusCode::NotFound,
						"404 Not Found",
						"Your homepage is not available when Trusted Signer is disabled.",
						Some("You can still access dapps by writing a correct address, though. Re-enable Signer to get your homepage back."),
						self.signer_address.clone(),
					))
				}
			},
			// RPC by default
			_ => {
				trace!(target: "dapps", "Resolving to RPC call.");
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
		signer_address: Option<(String, u16)>,
		content_fetcher: Arc<Fetcher>,
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
			signer_address: signer_address,
			endpoints: endpoints,
			fetch: content_fetcher,
			special: special,
			authorization: authorization,
			allowed_hosts: allowed_hosts,
			handler: handler,
		}
	}
}

fn is_web_endpoint(path: &Option<EndpointPath>) -> bool {
	match *path {
		Some(ref path) if path.app_id == apps::WEB_PATH => true,
		_ => false,
	}
}

fn extract_referer_endpoint(req: &server::Request<HttpStream>) -> Option<(EndpointPath, Url)> {
	let referer = req.headers().get::<header::Referer>();

	let url = referer.and_then(|referer| Url::parse(&referer.0).ok());
	url.and_then(|url| {
		let option = Some(url);
		extract_url_referer_endpoint(&option).or_else(|| {
			extract_endpoint(&option).0.map(|endpoint| (endpoint, option.expect("Just wrapped; qed")))
		})
	})
}

fn extract_url_referer_endpoint(url: &Option<Url>) -> Option<(EndpointPath, Url)> {
	let query = url.as_ref().and_then(|url| url.query.as_ref());
	match (url, query) {
		(&Some(ref url), Some(ref query)) if query.starts_with(apps::URL_REFERER) => {
			let referer_url = format!("http://{}:{}/{}", url.host, url.port, &query[apps::URL_REFERER.len()..]);
			debug!(target: "dapps", "Recovering referer from query parameter: {}", referer_url);

			let referer_url = Url::parse(&referer_url).ok();
			extract_endpoint(&referer_url).0.map(|endpoint| {
				(endpoint, referer_url.expect("Endpoint returned only when url `is_some`").clone())
			})
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
				let id = &domain[0..(domain.len() - DAPPS_DOMAIN.len())];
				let (id, params) = if let Some(split) = id.rfind('.') {
					let (params, id) = id.split_at(split);
					(id[1..].to_owned(), [params.to_owned()].into_iter().chain(&url.path).cloned().collect())
				} else {
					(id.to_owned(), url.path.clone())
				};

				(Some(EndpointPath {
					app_id: id,
					app_params: params,
					host: domain.clone(),
					port: url.port,
					using_dapps_domains: true,
				}), special_endpoint(url))
			},
			_ if url.path.len() > 1 => {
				let id = url.path[0].to_owned();
				(Some(EndpointPath {
					app_id: id,
					app_params: url.path[1..].to_vec(),
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
			app_params: vec!["index.html".to_owned()],
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
			app_params: vec!["".to_owned()],
			host: "localhost".to_owned(),
			port: 8080,
			using_dapps_domains: false,
		}), SpecialEndpoint::Rpc)
	);

	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.web3.site/parity-utils/inject.js").ok()),
		(Some(EndpointPath {
			app_id: "status".to_owned(),
			app_params: vec!["my".to_owned(), "parity-utils".into(), "inject.js".into()],
			host: "my.status.web3.site".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::Utils)
	);

	// By Subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://status.web3.site/test.html").ok()),
		(Some(EndpointPath {
			app_id: "status".to_owned(),
			app_params: vec!["test.html".to_owned()],
			host: "status.web3.site".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::None)
	);

	// RPC by subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.web3.site/rpc/").ok()),
		(Some(EndpointPath {
			app_id: "status".to_owned(),
			app_params: vec!["my".to_owned(), "rpc".into(), "".into()],
			host: "my.status.web3.site".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::Rpc)
	);

	// API by subdomain
	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.web3.site/api/").ok()),
		(Some(EndpointPath {
			app_id: "status".to_owned(),
			app_params: vec!["my".to_owned(), "api".into(), "".into()],
			host: "my.status.web3.site".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::Api)
	);
}
