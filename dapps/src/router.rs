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
//! Dispatch requests to proper application.

use std::cmp;
use std::sync::Arc;
use std::collections::HashMap;

use url::{Url, Host};
use hyper::{self, server, header, Control};
use hyper::net::HttpStream;
use jsonrpc_http_server as http;

use apps;
use apps::fetcher::Fetcher;
use endpoint::{Endpoint, EndpointPath, Handler};
use Endpoints;
use handlers;
use Embeddable;

/// Special endpoints are accessible on every domain (every dapp)
#[derive(Debug, PartialEq, Hash, Eq)]
pub enum SpecialEndpoint {
	Rpc,
	Api,
	Utils,
	Home,
	None,
}

pub struct Router {
	endpoints: Option<Endpoints>,
	fetch: Arc<Fetcher>,
	special: HashMap<SpecialEndpoint, Option<Box<Endpoint>>>,
	embeddable_on: Embeddable,
	dapps_domain: String,
}

impl Router {
	fn resolve_request(&self, req: &server::Request<HttpStream>, control: Control, refresh_dapps: bool) -> (bool, Option<Box<Handler>>) {
		// Choose proper handler depending on path / domain
		let url = handlers::extract_url(req);
		let endpoint = extract_endpoint(&url, &self.dapps_domain);
		let referer = extract_referer_endpoint(req, &self.dapps_domain);
		let is_utils = endpoint.1 == SpecialEndpoint::Utils;
		let is_get_request = *req.method() == hyper::Method::Get;
		let is_head_request = *req.method() == hyper::Method::Head;
		let has_dapp = |dapp: &str| self.endpoints
			.as_ref()
			.map_or(false, |endpoints| endpoints.endpoints.read().contains_key(dapp));

		trace!(target: "dapps", "Routing request to {:?}. Details: {:?}", url, req);
		debug!(target: "dapps", "Handling endpoint request: {:?}", endpoint);

		(is_utils, match (endpoint.0, endpoint.1, referer) {
			// Handle invalid web requests that we can recover from
			(ref path, SpecialEndpoint::None, Some((ref referer, ref referer_url)))
				if referer.app_id == apps::WEB_PATH
					&& has_dapp(apps::WEB_PATH)
					&& !is_web_endpoint(path)
				=>
			{
				trace!(target: "dapps", "Redirecting to correct web request: {:?}", referer_url);
				let len = cmp::min(referer_url.path.len(), 2); // /web/<encoded>/
				let base = referer_url.path[..len].join("/");
				let requested = url.map(|u| u.path.join("/")).unwrap_or_default();
				Some(handlers::Redirection::boxed(&format!("/{}/{}", base, requested)))
			},
			// First check special endpoints
			(ref path, ref endpoint, _) if self.special.contains_key(endpoint) => {
				trace!(target: "dapps", "Resolving to special endpoint.");
				self.special.get(endpoint)
					.expect("special known to contain key; qed")
					.as_ref()
					.map(|special| special.to_async_handler(path.clone().unwrap_or_default(), control))
			},
			// Then delegate to dapp
			(Some(ref path), _, _) if has_dapp(&path.app_id) => {
				trace!(target: "dapps", "Resolving to local/builtin dapp.");
				Some(self.endpoints
					.as_ref()
					.expect("endpoints known to be set; qed")
					.endpoints
					.read()
					.get(&path.app_id)
					.expect("endpoints known to contain key; qed")
					.to_async_handler(path.clone(), control))
			},
			// Try to resolve and fetch the dapp
			(Some(ref path), _, _) if self.fetch.contains(&path.app_id) => {
				trace!(target: "dapps", "Resolving to fetchable content.");
				Some(self.fetch.to_async_handler(path.clone(), control))
			},
			// 404 for non-existent content (only if serving endpoints and not homepage)
			(Some(ref path), _, _)
				if (is_get_request || is_head_request)
					&& self.endpoints.is_some()
					&& path.app_id != apps::HOME_PAGE
				=>
			{
				trace!(target: "dapps", "Resolving to 404.");
				if refresh_dapps {
					debug!(target: "dapps", "Refreshing dapps and re-trying.");
					self.endpoints.as_ref().map(|endpoints| endpoints.refresh_local_dapps());
					return self.resolve_request(req, control, false)
				} else {
					Some(Box::new(handlers::ContentHandler::error(
						hyper::StatusCode::NotFound,
						"404 Not Found",
						"Requested content was not found.",
						None,
						self.embeddable_on.clone(),
					)))
				}
			},
			// Any other GET|HEAD requests to home page.
			_ if (is_get_request || is_head_request) && self.special.contains_key(&SpecialEndpoint::Home) => {
				self.special.get(&SpecialEndpoint::Home)
					.expect("special known to contain key; qed")
					.as_ref()
					.map(|special| special.to_async_handler(Default::default(), control))
			},
			// RPC by default
			_ => {
				trace!(target: "dapps", "Resolving to RPC call.");
				None
			}
		})
	}
}

impl http::RequestMiddleware for Router {
	fn on_request(&self, req: &server::Request<HttpStream>, control: &Control) -> http::RequestMiddlewareAction {
		let control = control.clone();
		let is_origin_set = req.headers().get::<header::Origin>().is_some();
		let (is_utils, handler) = self.resolve_request(req, control, self.endpoints.is_some());
		match handler {
			Some(handler) => http::RequestMiddlewareAction::Respond {
				should_validate_hosts: !is_utils,
				handler: handler,
			},
			None => http::RequestMiddlewareAction::Proceed {
				should_continue_on_invalid_cors: !is_origin_set,
			},
		}
	}
}

impl Router {
	pub fn new(
		content_fetcher: Arc<Fetcher>,
		endpoints: Option<Endpoints>,
		special: HashMap<SpecialEndpoint, Option<Box<Endpoint>>>,
		embeddable_on: Embeddable,
		dapps_domain: String,
	) -> Self {
		Router {
			endpoints: endpoints,
			fetch: content_fetcher,
			special: special,
			embeddable_on: embeddable_on,
			dapps_domain: format!(".{}", dapps_domain),
		}
	}
}

fn is_web_endpoint(path: &Option<EndpointPath>) -> bool {
	match *path {
		Some(ref path) if path.app_id == apps::WEB_PATH => true,
		_ => false,
	}
}

fn extract_referer_endpoint(req: &server::Request<HttpStream>, dapps_domain: &str) -> Option<(EndpointPath, Url)> {
	let referer = req.headers().get::<header::Referer>();

	let url = referer.and_then(|referer| Url::parse(&referer.0).ok());
	url.and_then(|url| {
		let option = Some(url);
		extract_url_referer_endpoint(&option, dapps_domain).or_else(|| {
			extract_endpoint(&option, dapps_domain).0.map(|endpoint| (endpoint, option.expect("Just wrapped; qed")))
		})
	})
}

fn extract_url_referer_endpoint(url: &Option<Url>, dapps_domain: &str) -> Option<(EndpointPath, Url)> {
	let query = url.as_ref().and_then(|url| url.query.as_ref());
	match (url, query) {
		(&Some(ref url), Some(ref query)) if query.starts_with(apps::URL_REFERER) => {
			let referer_url = format!("http://{}:{}/{}", url.host, url.port, &query[apps::URL_REFERER.len()..]);
			debug!(target: "dapps", "Recovering referer from query parameter: {}", referer_url);

			let referer_url = Url::parse(&referer_url).ok();
			extract_endpoint(&referer_url, dapps_domain).0.map(|endpoint| {
				(endpoint, referer_url.expect("Endpoint returned only when url `is_some`").clone())
			})
		},
		_ => None,
	}
}

fn extract_endpoint(url: &Option<Url>, dapps_domain: &str) -> (Option<EndpointPath>, SpecialEndpoint) {
	fn special_endpoint(url: &Url) -> SpecialEndpoint {
		if url.path.len() <= 1 {
			return SpecialEndpoint::None;
		}

		match url.path[0].as_ref() {
			apps::RPC_PATH => SpecialEndpoint::Rpc,
			apps::API_PATH => SpecialEndpoint::Api,
			apps::UTILS_PATH => SpecialEndpoint::Utils,
			apps::HOME_PAGE => SpecialEndpoint::Home,
			_ => SpecialEndpoint::None,
		}
	}

	match *url {
		Some(ref url) => match url.host {
			Host::Domain(ref domain) if domain.ends_with(dapps_domain) => {
				let id = &domain[0..(domain.len() - dapps_domain.len())];
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
	let dapps_domain = ".web3.site";
	assert_eq!(extract_endpoint(&None, dapps_domain), (None, SpecialEndpoint::None));

	// With path prefix
	assert_eq!(
		extract_endpoint(&Url::parse("http://localhost:8080/status/index.html").ok(), dapps_domain),
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
		extract_endpoint(&Url::parse("http://localhost:8080/rpc/").ok(), dapps_domain),
		(Some(EndpointPath {
			app_id: "rpc".to_owned(),
			app_params: vec!["".to_owned()],
			host: "localhost".to_owned(),
			port: 8080,
			using_dapps_domains: false,
		}), SpecialEndpoint::Rpc)
	);

	assert_eq!(
		extract_endpoint(&Url::parse("http://my.status.web3.site/parity-utils/inject.js").ok(), dapps_domain),
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
		extract_endpoint(&Url::parse("http://status.web3.site/test.html").ok(), dapps_domain),
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
		extract_endpoint(&Url::parse("http://my.status.web3.site/rpc/").ok(), dapps_domain),
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
		extract_endpoint(&Url::parse("http://my.status.web3.site/api/").ok(), dapps_domain),
		(Some(EndpointPath {
			app_id: "status".to_owned(),
			app_params: vec!["my".to_owned(), "api".into(), "".into()],
			host: "my.status.web3.site".to_owned(),
			port: 80,
			using_dapps_domains: true,
		}), SpecialEndpoint::Api)
	);
}
