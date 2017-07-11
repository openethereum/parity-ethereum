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

//! Ethcore Webapplications for Parity
#![warn(missing_docs)]
#![cfg_attr(feature="nightly", feature(plugin))]
#![cfg_attr(feature="nightly", plugin(clippy))]

extern crate base32;
extern crate futures;
extern crate linked_hash_map;
extern crate mime_guess;
extern crate rand;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate unicase;
extern crate url as url_lib;
extern crate zip;

extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

extern crate ethcore_util as util;
extern crate fetch;
extern crate parity_dapps_glue as parity_dapps;
extern crate parity_hash_fetch as hash_fetch;
extern crate parity_reactor;
extern crate parity_ui;

#[macro_use]
extern crate log;
#[macro_use]
extern crate mime;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate ethcore_devtools as devtools;
#[cfg(test)]
extern crate env_logger;


mod endpoint;
mod apps;
mod page;
mod router;
mod handlers;
mod api;
mod proxypac;
mod url;
mod web;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

use jsonrpc_http_server::{self as http, hyper, Origin};

use fetch::Fetch;
use parity_reactor::Remote;

pub use hash_fetch::urlhint::ContractClient;

/// Indicates sync status
pub trait SyncStatus: Send + Sync {
	/// Returns true if there is a major sync happening.
	fn is_major_importing(&self) -> bool;
}

impl<F> SyncStatus for F where F: Fn() -> bool + Send + Sync {
	fn is_major_importing(&self) -> bool { self() }
}

/// Validates Web Proxy tokens
pub trait WebProxyTokens: Send + Sync {
	/// Should return a domain allowed to be accessed by this token or `None` if the token is not valid
	fn domain(&self, token: &str) -> Option<Origin>;
}

impl<F> WebProxyTokens for F where F: Fn(String) -> Option<Origin> + Send + Sync {
	fn domain(&self, token: &str) -> Option<Origin> { self(token.to_owned()) }
}

/// Current supported endpoints.
pub struct Endpoints {
	endpoints: endpoint::Endpoints,
}

impl Endpoints {
	/// Returns a current list of app endpoints.
	pub fn list(&self) -> Vec<apps::App> {
		self.endpoints.iter().filter_map(|(ref k, ref e)| {
			e.info().map(|ref info| apps::App::from_info(k, info))
		}).collect()
	}
}

/// Dapps server as `jsonrpc-http-server` request middleware.
pub struct Middleware {
	router: router::Router,
	endpoints: endpoint::Endpoints,
}

impl Middleware {
	/// Get local endpoints handle.
	pub fn endpoints(&self) -> Endpoints {
		Endpoints {
			endpoints: self.endpoints.clone(),
		}
	}

	/// Creates new middleware for UI server.
	pub fn ui<F: Fetch + Clone>(
		remote: Remote,
		registrar: Arc<ContractClient>,
		sync_status: Arc<SyncStatus>,
		fetch: F,
		dapps_domain: String,
	) -> Self {
		let content_fetcher = Arc::new(apps::fetcher::ContentFetcher::new(
			hash_fetch::urlhint::URLHintContract::new(registrar),
			sync_status,
			remote.clone(),
			fetch.clone(),
		).embeddable_on(None).allow_dapps(false));
		let special = {
			let mut special = special_endpoints(content_fetcher.clone());
			special.insert(router::SpecialEndpoint::Home, Some(apps::ui()));
			special
		};
		let router = router::Router::new(
			content_fetcher,
			None,
			special,
			None,
			dapps_domain,
		);

		Middleware {
			router: router,
			endpoints: Default::default(),
		}
	}

	/// Creates new Dapps server middleware.
	pub fn dapps<F: Fetch + Clone>(
		remote: Remote,
		ui_address: Option<(String, u16)>,
		dapps_path: PathBuf,
		extra_dapps: Vec<PathBuf>,
		dapps_domain: String,
		registrar: Arc<ContractClient>,
		sync_status: Arc<SyncStatus>,
		web_proxy_tokens: Arc<WebProxyTokens>,
		fetch: F,
	) -> Self {
		let embeddable = as_embeddable(ui_address.clone(), dapps_domain.clone());
		let content_fetcher = Arc::new(apps::fetcher::ContentFetcher::new(
			hash_fetch::urlhint::URLHintContract::new(registrar),
			sync_status,
			remote.clone(),
			fetch.clone(),
		).embeddable_on(embeddable.clone()).allow_dapps(true));
		let endpoints = apps::all_endpoints(
			dapps_path,
			extra_dapps,
			dapps_domain.clone(),
			ui_address.clone(),
			web_proxy_tokens,
			remote.clone(),
			fetch.clone(),
		);

		let special = {
			let mut special = special_endpoints(content_fetcher.clone());
			special.insert(
				router::SpecialEndpoint::Home,
				Some(apps::ui_redirection(ui_address.clone(), dapps_domain.clone())),
			);
			special
		};

		let router = router::Router::new(
			content_fetcher,
			Some(endpoints.clone()),
			special,
			embeddable,
			dapps_domain,
		);

		Middleware {
			router: router,
			endpoints: endpoints,
		}
	}
}

impl http::RequestMiddleware for Middleware {
	fn on_request(&self, req: &hyper::server::Request<hyper::net::HttpStream>, control: &hyper::Control) -> http::RequestMiddlewareAction {
		self.router.on_request(req, control)
	}
}

fn special_endpoints(content_fetcher: Arc<apps::fetcher::Fetcher>) -> HashMap<router::SpecialEndpoint, Option<Box<endpoint::Endpoint>>> {
	let mut special = HashMap::new();
	special.insert(router::SpecialEndpoint::Rpc, None);
	special.insert(router::SpecialEndpoint::Utils, Some(apps::utils()));
	special.insert(router::SpecialEndpoint::Api, Some(api::RestApi::new(content_fetcher)));
	special
}

fn address(host: &str, port: u16) -> String {
	format!("{}:{}", host, port)
}

fn as_embeddable(ui_address: Option<(String, u16)>, dapps_domain: String) -> Option<ParentFrameSettings> {
	ui_address.map(|(host, port)| ParentFrameSettings { host, port, dapps_domain, })
}

/// Random filename
fn random_filename() -> String {
	use ::rand::Rng;
	let mut rng = ::rand::OsRng::new().unwrap();
	rng.gen_ascii_chars().take(12).collect()
}

type Embeddable = Option<ParentFrameSettings>;

/// Parent frame host and port allowed to embed the content.
#[derive(Debug, Clone)]
pub struct ParentFrameSettings {
	/// Hostname
	pub host: String,
	/// Port
	pub port: u16,
	/// Dapps Domain (web3.site)
	pub dapps_domain: String,
}
