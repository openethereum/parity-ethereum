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

use std::sync::Arc;
use std::collections::HashMap;

use futures::future;
use hyper::{self, header, Uri};
use jsonrpc_http_server as http;

use apps;
use apps::fetcher::Fetcher;
use endpoint::{self, Endpoint, EndpointPath};
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

enum Response {
	Some(endpoint::Response),
	None(hyper::Request),
}

/// An endpoint router.
/// Dispatches the request to particular Endpoint by requested uri/path.
pub struct Router {
	endpoints: Option<Endpoints>,
	fetch: Arc<Fetcher>,
	special: HashMap<SpecialEndpoint, Option<Box<Endpoint>>>,
	embeddable_on: Embeddable,
	dapps_domain: String,
}

impl Router {
	fn resolve_request(&self, req: hyper::Request, refresh_dapps: bool) -> (bool, Response) {
		// Choose proper handler depending on path / domain
		let endpoint = extract_endpoint(req.uri(), req.headers().get(), &self.dapps_domain);
		let referer = extract_referer_endpoint(&req, &self.dapps_domain);
		let is_utils = endpoint.1 == SpecialEndpoint::Utils;
		let is_get_request = *req.method() == hyper::Method::Get;
		let is_head_request = *req.method() == hyper::Method::Head;
		let has_dapp = |dapp: &str| self.endpoints
			.as_ref()
			.map_or(false, |endpoints| endpoints.endpoints.read().contains_key(dapp));

		trace!(target: "dapps", "Routing request to {:?}. Details: {:?}", req.uri(), req);
		debug!(target: "dapps", "Handling endpoint request: {:?}, referer: {:?}", endpoint, referer);

		(is_utils, match (endpoint.0, endpoint.1, referer) {
			// Handle invalid web requests that we can recover from
			(ref path, SpecialEndpoint::None, Some(ref referer))
				if referer.app_id == apps::WEB_PATH
					&& has_dapp(apps::WEB_PATH)
					&& !is_web_endpoint(path)
				=>
			{
				let token = referer.app_params.get(0).map(String::as_str).unwrap_or("");
				let requested = req.uri().path();
				let query = req.uri().query().map_or_else(String::new, |query| format!("?{}", query));
				let redirect_url = format!("/{}/{}{}{}", apps::WEB_PATH, token, requested, query);
				trace!(target: "dapps", "Redirecting to correct web request: {:?}", redirect_url);
				Response::Some(Box::new(future::ok(
					handlers::Redirection::new(redirect_url).into()
				)))
			},
			// First check special endpoints
			(ref path, ref endpoint, _) if self.special.contains_key(endpoint) => {
				trace!(target: "dapps", "Resolving to special endpoint.");
				let special = self.special.get(endpoint).expect("special known to contain key; qed");
				match *special {
					Some(ref special) => Response::Some(special.respond(path.clone().unwrap_or_default(), req)),
					None => Response::None(req),
				}
			},
			// Then delegate to dapp
			(Some(ref path), _, _) if has_dapp(&path.app_id) => {
				trace!(target: "dapps", "Resolving to local/builtin dapp.");
				Response::Some(self.endpoints
					.as_ref()
					.expect("endpoints known to be set; qed")
					.endpoints
					.read()
					.get(&path.app_id)
					.expect("endpoints known to contain key; qed")
					.respond(path.clone(), req))
			},
			// Try to resolve and fetch the dapp
			(Some(ref path), _, _) if self.fetch.contains(&path.app_id) => {
				trace!(target: "dapps", "Resolving to fetchable content.");
				Response::Some(self.fetch.respond(path.clone(), req))
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
					return self.resolve_request(req, false);
				} else {
					Response::Some(Box::new(future::ok(handlers::ContentHandler::error(
						hyper::StatusCode::NotFound,
						"404 Not Found",
						"Requested content was not found.",
						None,
						self.embeddable_on.clone(),
					).into())))
				}
			},
			// Any other GET|HEAD requests to home page.
			_ if (is_get_request || is_head_request) && self.special.contains_key(&SpecialEndpoint::Home) => {
				trace!(target: "dapps", "Resolving to home page.");
				let special = self.special.get(&SpecialEndpoint::Home).expect("special known to contain key; qed");
				match *special {
					Some(ref special) => {
						let mut endpoint = EndpointPath::default();
						endpoint.app_params = req.uri().path().split('/').map(str::to_owned).collect();
						Response::Some(special.respond(endpoint, req))
					},
					None => Response::None(req),
				}
			},
			// RPC by default
			_ if self.special.contains_key(&SpecialEndpoint::Rpc) => {
				trace!(target: "dapps", "Resolving to RPC call.");
				Response::None(req)
			},
			// 404 otherwise
			_ => {
				Response::Some(Box::new(future::ok(handlers::ContentHandler::error(
					hyper::StatusCode::NotFound,
					"404 Not Found",
					"Requested content was not found.",
					None,
					self.embeddable_on.clone(),
				).into())))
			},
		})
	}
}

impl http::RequestMiddleware for Router {
	fn on_request(&self, req: hyper::Request) -> http::RequestMiddlewareAction {
		let is_origin_set = req.headers().get::<header::Origin>().is_some();
		let (is_utils, response) = self.resolve_request(req, self.endpoints.is_some());
		match response {
			Response::Some(response) => http::RequestMiddlewareAction::Respond {
				should_validate_hosts: !is_utils,
				response,
			},
			Response::None(request) => http::RequestMiddlewareAction::Proceed {
				should_continue_on_invalid_cors: !is_origin_set,
				request,
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

fn extract_referer_endpoint(req: &hyper::Request, dapps_domain: &str) -> Option<EndpointPath> {
	let referer = req.headers().get::<header::Referer>();

	let url = referer.and_then(|referer| referer.parse().ok());
	url.and_then(|url| {
		extract_url_referer_endpoint(&url, dapps_domain).or_else(|| {
			extract_endpoint(&url, None, dapps_domain).0
		})
	})
}

fn extract_url_referer_endpoint(url: &Uri, dapps_domain: &str) -> Option<EndpointPath> {
	let query = url.query();
	match query {
		Some(query) if query.starts_with(apps::URL_REFERER) => {
			let scheme = url.scheme().unwrap_or("http");
			let host = url.host().unwrap_or("unknown");
			let port = default_port(url, None);
			let referer_url = format!("{}://{}:{}/{}", scheme, host, port, &query[apps::URL_REFERER.len()..]);
			debug!(target: "dapps", "Recovering referer from query parameter: {}", referer_url);

			if let Some(referer_url) = referer_url.parse().ok() {
				extract_endpoint(&referer_url, None, dapps_domain).0
			} else {
				None
			}
		},
		_ => None,
	}
}

fn extract_endpoint(url: &Uri, extra_host: Option<&header::Host>, dapps_domain: &str) -> (Option<EndpointPath>, SpecialEndpoint) {
	fn special_endpoint(path: &[&str]) -> SpecialEndpoint {
		if path.len() <= 1 {
			return SpecialEndpoint::None;
		}

		match path[0].as_ref() {
			apps::RPC_PATH => SpecialEndpoint::Rpc,
			apps::API_PATH => SpecialEndpoint::Api,
			apps::UTILS_PATH => SpecialEndpoint::Utils,
			apps::HOME_PAGE => SpecialEndpoint::Home,
			_ => SpecialEndpoint::None,
		}
	}

	let port = default_port(url, extra_host.as_ref().and_then(|h| h.port()));
	let host = url.host().or_else(|| extra_host.as_ref().map(|h| h.hostname()));
	let query = url.query().map(str::to_owned);
	let mut path_segments = url.path().split('/').skip(1).collect::<Vec<_>>();
	trace!(
		target: "dapps",
		"Extracting endpoint from: {:?} (dapps: {}). Got host {:?}:{} with path {:?}",
		url, dapps_domain, host, port, path_segments
	);
	match host {
		Some(host) if host.ends_with(dapps_domain) => {
			let id = &host[0..(host.len() - dapps_domain.len())];
			let special = special_endpoint(&path_segments);

			// remove special endpoint id from params
			if special != SpecialEndpoint::None {
				path_segments.remove(0);
			}

			let (app_id, app_params) = if let Some(split) = id.rfind('.') {
				let (params, id) = id.split_at(split);
				path_segments.insert(0, params);
				(id[1..].to_owned(), path_segments)
			} else {
				(id.to_owned(), path_segments)
			};

			(Some(EndpointPath {
				app_id,
				app_params: app_params.into_iter().map(Into::into).collect(),
				query,
				host: host.to_owned(),
				port,
				using_dapps_domains: true,
			}), special)
		},
		Some(host) if path_segments.len() > 1 => {
			let special = special_endpoint(&path_segments);
			let id = path_segments.remove(0);
			(Some(EndpointPath {
				app_id: id.to_owned(),
				app_params: path_segments.into_iter().map(Into::into).collect(),
				query,
				host: host.to_owned(),
				port,
				using_dapps_domains: false,
			}), special)
		},
		_ => (None, special_endpoint(&path_segments)),
	}
}

fn default_port(url: &Uri, extra_port: Option<u16>) -> u16 {
	let scheme = url.scheme().unwrap_or("http");
	url.port().or(extra_port).unwrap_or_else(|| match scheme {
		"http" => 80,
		"https" => 443,
		_ => 80,
	})
}

#[cfg(test)]
mod tests {
	use super::{SpecialEndpoint, EndpointPath, extract_endpoint};

	#[test]
	fn should_extract_endpoint() {
		let dapps_domain = ".web3.site";

		// With path prefix
		assert_eq!(
			extract_endpoint(&"http://localhost:8080/status/index.html?q=1".parse().unwrap(), None, dapps_domain),
			(Some(EndpointPath {
				app_id: "status".to_owned(),
				app_params: vec!["index.html".to_owned()],
				query: Some("q=1".into()),
				host: "localhost".to_owned(),
				port: 8080,
				using_dapps_domains: false,
			}), SpecialEndpoint::None)
		);

		// With path prefix
		assert_eq!(
			extract_endpoint(&"http://localhost:8080/rpc/".parse().unwrap(), None, dapps_domain),
			(Some(EndpointPath {
				app_id: "rpc".to_owned(),
				app_params: vec!["".to_owned()],
				query: None,
				host: "localhost".to_owned(),
				port: 8080,
				using_dapps_domains: false,
			}), SpecialEndpoint::Rpc)
		);

		assert_eq!(
			extract_endpoint(&"http://my.status.web3.site/parity-utils/inject.js".parse().unwrap(), None, dapps_domain),
			(Some(EndpointPath {
				app_id: "status".to_owned(),
				app_params: vec!["my".into(), "inject.js".into()],
				query: None,
				host: "my.status.web3.site".to_owned(),
				port: 80,
				using_dapps_domains: true,
			}), SpecialEndpoint::Utils)
		);

		assert_eq!(
			extract_endpoint(&"http://my.status.web3.site/inject.js".parse().unwrap(), None, dapps_domain),
			(Some(EndpointPath {
				app_id: "status".to_owned(),
				app_params: vec!["my".into(), "inject.js".into()],
				query: None,
				host: "my.status.web3.site".to_owned(),
				port: 80,
				using_dapps_domains: true,
			}), SpecialEndpoint::None)
		);

		// By Subdomain
		assert_eq!(
			extract_endpoint(&"http://status.web3.site/test.html".parse().unwrap(), None, dapps_domain),
			(Some(EndpointPath {
				app_id: "status".to_owned(),
				app_params: vec!["test.html".to_owned()],
				query: None,
				host: "status.web3.site".to_owned(),
				port: 80,
				using_dapps_domains: true,
			}), SpecialEndpoint::None)
		);

		// RPC by subdomain
		assert_eq!(
			extract_endpoint(&"http://my.status.web3.site/rpc/".parse().unwrap(), None, dapps_domain),
			(Some(EndpointPath {
				app_id: "status".to_owned(),
				app_params: vec!["my".into(), "".into()],
				query: None,
				host: "my.status.web3.site".to_owned(),
				port: 80,
				using_dapps_domains: true,
			}), SpecialEndpoint::Rpc)
		);

		// API by subdomain
		assert_eq!(
			extract_endpoint(&"http://my.status.web3.site/api/".parse().unwrap(), None, dapps_domain),
			(Some(EndpointPath {
				app_id: "status".to_owned(),
				app_params: vec!["my".into(), "".into()],
				query: None,
				host: "my.status.web3.site".to_owned(),
				port: 80,
				using_dapps_domains: true,
			}), SpecialEndpoint::Api)
		);
	}
}
