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

extern crate base32;
extern crate futures_cpupool;
extern crate itertools;
extern crate linked_hash_map;
extern crate mime_guess;
extern crate parking_lot;
extern crate rand;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate unicase;
extern crate zip;

extern crate jsonrpc_http_server;

extern crate ethcore_bytes as bytes;
extern crate ethereum_types;
extern crate fetch;
extern crate node_health;
extern crate parity_dapps_glue as parity_dapps;
extern crate parity_hash_fetch as hash_fetch;
extern crate parity_ui;
extern crate keccak_hash as hash;
extern crate parity_version;
extern crate registrar;

#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
extern crate ethcore_devtools as devtools;
#[cfg(test)]
extern crate jsonrpc_core;
#[cfg(test)]
extern crate parity_reactor;

mod endpoint;
mod apps;
mod page;
mod router;
mod handlers;
mod api;
mod proxypac;
mod web;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;
use futures_cpupool::CpuPool;
use jsonrpc_http_server::{self as http, hyper, Origin};
use parking_lot::RwLock;

use fetch::Fetch;
use node_health::NodeHealth;

pub use registrar::{RegistrarClient, Asynchronous};
pub use node_health::SyncStatus;


/// Validates Web Proxy tokens
pub trait WebProxyTokens: Send + Sync {
	/// Should return a domain allowed to be accessed by this token or `None` if the token is not valid
	fn domain(&self, token: &str) -> Option<Origin>;
}

impl<F> WebProxyTokens for F where F: Fn(String) -> Option<Origin> + Send + Sync {
	fn domain(&self, token: &str) -> Option<Origin> { self(token.to_owned()) }
}

/// Current supported endpoints.
#[derive(Default, Clone)]
pub struct Endpoints {
	local_endpoints: Arc<RwLock<Vec<String>>>,
	endpoints: Arc<RwLock<endpoint::Endpoints>>,
	dapps_path: PathBuf,
	embeddable: Option<ParentFrameSettings>,
	pool: Option<CpuPool>,
}

impl Endpoints {
	/// Returns a current list of app endpoints.
	pub fn list(&self) -> Vec<apps::App> {
		self.endpoints.read().iter().filter_map(|(ref k, ref e)| {
			e.info().map(|ref info| info.with_id(k))
		}).collect()
	}

	/// Check for any changes in the local dapps folder and update.
	pub fn refresh_local_dapps(&self) {
		let pool = match self.pool.as_ref() {
			None => return,
			Some(pool) => pool,
		};
		let new_local = apps::fs::local_endpoints(&self.dapps_path, self.embeddable.clone(), pool.clone());
		let old_local = mem::replace(&mut *self.local_endpoints.write(), new_local.keys().cloned().collect());
		let (_, to_remove): (_, Vec<_>) = old_local
			.into_iter()
			.partition(|k| new_local.contains_key(&k.clone()));

		let mut endpoints = self.endpoints.write();
		// remove the dead dapps
		for k in to_remove {
			endpoints.remove(&k);
		}
		// new dapps to be added
		for (k, v) in new_local {
			if !endpoints.contains_key(&k) {
				endpoints.insert(k, v);
			}
		}
	}
}

/// Dapps server as `jsonrpc-http-server` request middleware.
pub struct Middleware {
	endpoints: Endpoints,
	router: router::Router,
}

impl Middleware {
	/// Get local endpoints handle.
	pub fn endpoints(&self) -> &Endpoints {
		&self.endpoints
	}

	/// Creates new middleware for UI server.
	pub fn ui<F: Fetch>(
		pool: CpuPool,
		health: NodeHealth,
		dapps_domain: &str,
		registrar: Arc<RegistrarClient<Call=Asynchronous>>,
		sync_status: Arc<SyncStatus>,
		fetch: F,
	) -> Self {
		let content_fetcher = Arc::new(apps::fetcher::ContentFetcher::new(
			hash_fetch::urlhint::URLHintContract::new(registrar),
			sync_status.clone(),
			fetch.clone(),
			pool.clone(),
		).embeddable_on(None).allow_dapps(false));
		let special = {
			let mut special = special_endpoints(
				pool.clone(),
				health,
				content_fetcher.clone(),
			);
			special.insert(router::SpecialEndpoint::Home, Some(apps::ui(pool.clone())));
			special
		};
		let router = router::Router::new(
			content_fetcher,
			None,
			special,
			None,
			dapps_domain.to_owned(),
		);

		Middleware {
			endpoints: Default::default(),
			router: router,
		}
	}

	/// Creates new Dapps server middleware.
	pub fn dapps<F: Fetch>(
		pool: CpuPool,
		health: NodeHealth,
		ui_address: Option<(String, u16)>,
		extra_embed_on: Vec<(String, u16)>,
		extra_script_src: Vec<(String, u16)>,
		dapps_path: PathBuf,
		extra_dapps: Vec<PathBuf>,
		dapps_domain: &str,
		registrar: Arc<RegistrarClient<Call=Asynchronous>>,
		sync_status: Arc<SyncStatus>,
		web_proxy_tokens: Arc<WebProxyTokens>,
		fetch: F,
	) -> Self {
		let embeddable = as_embeddable(ui_address, extra_embed_on, extra_script_src, dapps_domain);
		let content_fetcher = Arc::new(apps::fetcher::ContentFetcher::new(
			hash_fetch::urlhint::URLHintContract::new(registrar),
			sync_status.clone(),
			fetch.clone(),
			pool.clone(),
		).embeddable_on(embeddable.clone()).allow_dapps(true));
		let (local_endpoints, endpoints) = apps::all_endpoints(
			dapps_path.clone(),
			extra_dapps,
			dapps_domain,
			embeddable.clone(),
			web_proxy_tokens,
			fetch.clone(),
			pool.clone(),
		);
		let endpoints = Endpoints {
			endpoints: Arc::new(RwLock::new(endpoints)),
			dapps_path,
			local_endpoints: Arc::new(RwLock::new(local_endpoints)),
			embeddable: embeddable.clone(),
			pool: Some(pool.clone()),
		};

		let special = {
			let mut special = special_endpoints(
				pool.clone(),
				health,
				content_fetcher.clone(),
			);
			special.insert(
				router::SpecialEndpoint::Home,
				Some(apps::ui_redirection(embeddable.clone())),
			);
			special
		};

		let router = router::Router::new(
			content_fetcher,
			Some(endpoints.clone()),
			special,
			embeddable,
			dapps_domain.to_owned(),
		);

		Middleware {
			endpoints,
			router,
		}
	}
}

impl http::RequestMiddleware for Middleware {
	fn on_request(&self, req: hyper::Request) -> http::RequestMiddlewareAction {
		self.router.on_request(req)
	}
}

fn special_endpoints(
	pool: CpuPool,
	health: NodeHealth,
	content_fetcher: Arc<apps::fetcher::Fetcher>,
) -> HashMap<router::SpecialEndpoint, Option<Box<endpoint::Endpoint>>> {
	let mut special = HashMap::new();
	special.insert(router::SpecialEndpoint::Rpc, None);
	special.insert(router::SpecialEndpoint::Utils, Some(apps::utils(pool)));
	special.insert(router::SpecialEndpoint::Api, Some(api::RestApi::new(
		content_fetcher,
		health,
	)));
	special
}

fn address(host: &str, port: u16) -> String {
	format!("{}:{}", host, port)
}

fn as_embeddable(
	ui_address: Option<(String, u16)>,
	extra_embed_on: Vec<(String, u16)>,
	extra_script_src: Vec<(String, u16)>,
	dapps_domain: &str,
) -> Option<ParentFrameSettings> {
	ui_address.map(|(host, port)| ParentFrameSettings {
		host,
		port,
		extra_embed_on,
		extra_script_src,
		dapps_domain: dapps_domain.to_owned(),
	})
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
	/// Additional URLs the dapps can be embedded on.
	pub extra_embed_on: Vec<(String, u16)>,
	/// Additional URLs the dapp scripts can be loaded from.
	pub extra_script_src: Vec<(String, u16)>,
	/// Dapps Domain (web3.site)
	pub dapps_domain: String,
}
