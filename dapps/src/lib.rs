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
extern crate rustc_serialize;
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

use jsonrpc_http_server::{self as http, hyper, AccessControlAllowOrigin};

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
	/// Should return true if token is a valid web proxy access token.
	fn is_web_proxy_token_valid(&self, token: &str) -> bool;
}

impl<F> WebProxyTokens for F where F: Fn(String) -> bool + Send + Sync {
	fn is_web_proxy_token_valid(&self, token: &str) -> bool { self(token.to_owned()) }
}

/// Dapps server as `jsonrpc-http-server` request middleware.
pub struct Middleware {
	router: router::Router,
}

impl Middleware {
	/// Creates new Dapps server middleware.
	pub fn new<F: Fetch + Clone>(
		remote: Remote,
		signer_address: Option<(String, u16)>,
		dapps_path: PathBuf,
		extra_dapps: Vec<PathBuf>,
		registrar: Arc<ContractClient>,
		sync_status: Arc<SyncStatus>,
		web_proxy_tokens: Arc<WebProxyTokens>,
		fetch: F,
	) -> Self {
		let content_fetcher = Arc::new(apps::fetcher::ContentFetcher::new(
			hash_fetch::urlhint::URLHintContract::new(registrar),
			sync_status,
			signer_address.clone(),
			remote.clone(),
			fetch.clone(),
		));
		let endpoints = apps::all_endpoints(
			dapps_path,
			extra_dapps,
			signer_address.clone(),
			web_proxy_tokens,
			remote.clone(),
			fetch.clone(),
		);

		let cors_domains = cors_domains(signer_address.clone());

		let special = {
			let mut special = HashMap::new();
			special.insert(router::SpecialEndpoint::Rpc, None);
			special.insert(router::SpecialEndpoint::Utils, Some(apps::utils()));
			special.insert(
				router::SpecialEndpoint::Api,
				Some(api::RestApi::new(
					cors_domains.clone(),
					&endpoints,
					content_fetcher.clone()
				)),
			);
			special
		};

		let router = router::Router::new(
			signer_address,
			content_fetcher,
			endpoints,
			special,
		);

		Middleware {
			router: router,
		}
	}
}

impl http::RequestMiddleware for Middleware {
	fn on_request(&self, req: &hyper::server::Request<hyper::net::HttpStream>, control: &hyper::Control) -> http::RequestMiddlewareAction {
		self.router.on_request(req, control)
	}
}

/// Returns a list of CORS domains for API endpoint.
fn cors_domains(signer_address: Option<(String, u16)>) -> Vec<AccessControlAllowOrigin> {
	use self::apps::{HOME_PAGE, DAPPS_DOMAIN};

	match signer_address {
		Some(signer_address) => [
			format!("http://{}{}", HOME_PAGE, DAPPS_DOMAIN),
			format!("http://{}{}:{}", HOME_PAGE, DAPPS_DOMAIN, signer_address.1),
			format!("http://{}", address(&signer_address)),
			format!("https://{}{}", HOME_PAGE, DAPPS_DOMAIN),
			format!("https://{}{}:{}", HOME_PAGE, DAPPS_DOMAIN, signer_address.1),
			format!("https://{}", address(&signer_address)),
		].into_iter().map(|val| AccessControlAllowOrigin::Value(val.into())).collect(),
		None => vec![],
	}
}

fn address(address: &(String, u16)) -> String {
	format!("{}:{}", address.0, address.1)
}

/// Random filename
fn random_filename() -> String {
	use ::rand::Rng;
	let mut rng = ::rand::OsRng::new().unwrap();
	rng.gen_ascii_chars().take(12).collect()
}

#[cfg(test)]
mod util_tests {
	use super::cors_domains;
	use jsonrpc_http_server::AccessControlAllowOrigin;

	#[test]
	fn should_return_cors_domains() {
		// given

		// when
		let none = cors_domains(None);
		let some = cors_domains(Some(("127.0.0.1".into(), 18180)));

		// then
		assert_eq!(none, Vec::<AccessControlAllowOrigin>::new());
		assert_eq!(some, vec![
			"http://parity.web3.site".into(),
			"http://parity.web3.site:18180".into(),
			"http://127.0.0.1:18180".into(),
			"https://parity.web3.site".into(),
			"https://parity.web3.site:18180".into(),
			"https://127.0.0.1:18180".into(),
		]);
	}
}
