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
extern crate hyper;
extern crate time;
extern crate url as url_lib;
extern crate unicase;
extern crate serde;
extern crate serde_json;
extern crate zip;
extern crate rand;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
extern crate mime_guess;
extern crate rustc_serialize;
extern crate ethcore_rpc;
extern crate ethcore_util as util;
extern crate parity_hash_fetch as hash_fetch;
extern crate linked_hash_map;
extern crate fetch;
extern crate parity_dapps_glue as parity_dapps;
extern crate futures;
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
mod rpc;
mod api;
mod proxypac;
mod url;
mod web;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::collections::HashMap;

use ethcore_rpc::{Metadata};
use fetch::{Fetch, Client as FetchClient};
use hash_fetch::urlhint::ContractClient;
use jsonrpc_core::Middleware;
use jsonrpc_core::reactor::RpcHandler;
use router::auth::{Authorization, NoAuth, HttpBasicAuth};
use parity_reactor::Remote;

use self::apps::{HOME_PAGE, DAPPS_DOMAIN};

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

/// Webapps HTTP+RPC server build.
pub struct ServerBuilder<T: Fetch = FetchClient> {
	dapps_path: PathBuf,
	extra_dapps: Vec<PathBuf>,
	registrar: Arc<ContractClient>,
	sync_status: Arc<SyncStatus>,
	web_proxy_tokens: Arc<WebProxyTokens>,
	signer_address: Option<(String, u16)>,
	allowed_hosts: Option<Vec<String>>,
	extra_cors: Option<Vec<String>>,
	remote: Remote,
	fetch: Option<T>,
}

impl ServerBuilder {
	/// Construct new dapps server
	pub fn new<P: AsRef<Path>>(dapps_path: P, registrar: Arc<ContractClient>, remote: Remote) -> Self {
		ServerBuilder {
			dapps_path: dapps_path.as_ref().to_owned(),
			extra_dapps: vec![],
			registrar: registrar,
			sync_status: Arc::new(|| false),
			web_proxy_tokens: Arc::new(|_| false),
			signer_address: None,
			allowed_hosts: Some(vec![]),
			extra_cors: None,
			remote: remote,
			fetch: None,
		}
	}
}

impl<T: Fetch> ServerBuilder<T> {
	/// Set a fetch client to use.
	pub fn fetch<X: Fetch>(self, fetch: X) -> ServerBuilder<X> {
		ServerBuilder {
			dapps_path: self.dapps_path,
			extra_dapps: vec![],
			registrar: self.registrar,
			sync_status: self.sync_status,
			web_proxy_tokens: self.web_proxy_tokens,
			signer_address: self.signer_address,
			allowed_hosts: self.allowed_hosts,
			extra_cors: self.extra_cors,
			remote: self.remote,
			fetch: Some(fetch),
		}
	}

	/// Change default sync status.
	pub fn sync_status(mut self, status: Arc<SyncStatus>) -> Self {
		self.sync_status = status;
		self
	}

	/// Change default web proxy tokens validator.
	pub fn web_proxy_tokens(mut self, tokens: Arc<WebProxyTokens>) -> Self {
		self.web_proxy_tokens = tokens;
		self
	}

	/// Change default signer port.
	pub fn signer_address(mut self, signer_address: Option<(String, u16)>) -> Self {
		self.signer_address = signer_address;
		self
	}

	/// Change allowed hosts.
	/// `None` - All hosts are allowed
	/// `Some(whitelist)` - Allow only whitelisted hosts (+ listen address)
	pub fn allowed_hosts(mut self, allowed_hosts: Option<Vec<String>>) -> Self {
		self.allowed_hosts = allowed_hosts;
		self
	}

	/// Extra cors headers.
	/// `None` - no additional CORS URLs
	pub fn extra_cors_headers(mut self, cors: Option<Vec<String>>) -> Self {
		self.extra_cors = cors;
		self
	}

	/// Change extra dapps paths (apart from `dapps_path`)
	pub fn extra_dapps<P: AsRef<Path>>(mut self, extra_dapps: &[P]) -> Self {
		self.extra_dapps = extra_dapps.iter().map(|p| p.as_ref().to_owned()).collect();
		self
	}

	/// Asynchronously start server with no authentication,
	/// returns result with `Server` handle on success or an error.
	pub fn start_unsecured_http<S: Middleware<Metadata>>(self, addr: &SocketAddr, handler: RpcHandler<Metadata, S>) -> Result<Server, ServerError> {
		let fetch = self.fetch_client()?;
		Server::start_http(
			addr,
			self.allowed_hosts,
			self.extra_cors,
			NoAuth,
			handler,
			self.dapps_path,
			self.extra_dapps,
			self.signer_address,
			self.registrar,
			self.sync_status,
			self.web_proxy_tokens,
			self.remote,
			fetch,
		)
	}

	/// Asynchronously start server with `HTTP Basic Authentication`,
	/// return result with `Server` handle on success or an error.
	pub fn start_basic_auth_http<S: Middleware<Metadata>>(self, addr: &SocketAddr, username: &str, password: &str, handler: RpcHandler<Metadata, S>) -> Result<Server, ServerError> {
		let fetch = self.fetch_client()?;
		Server::start_http(
			addr,
			self.allowed_hosts,
			self.extra_cors,
			HttpBasicAuth::single_user(username, password),
			handler,
			self.dapps_path,
			self.extra_dapps,
			self.signer_address,
			self.registrar,
			self.sync_status,
			self.web_proxy_tokens,
			self.remote,
			fetch,
		)
	}

	fn fetch_client(&self) -> Result<T, ServerError> {
		match self.fetch.clone() {
			Some(fetch) => Ok(fetch),
			None => T::new().map_err(|_| ServerError::FetchInitialization),
		}
	}
}

/// Webapps HTTP server.
pub struct Server {
	server: Option<hyper::server::Listening>,
	panic_handler: Arc<Mutex<Option<Box<Fn() -> () + Send>>>>,
}

impl Server {
	/// Returns a list of allowed hosts or `None` if all hosts are allowed.
	fn allowed_hosts(hosts: Option<Vec<String>>, bind_address: String) -> Option<Vec<String>> {
		let mut allowed = Vec::new();

		match hosts {
			Some(hosts) => allowed.extend_from_slice(&hosts),
			None => return None,
		}

		// Add localhost domain as valid too if listening on loopback interface.
		allowed.push(bind_address.replace("127.0.0.1", "localhost").into());
		allowed.push(bind_address.into());
		Some(allowed)
	}

	/// Returns a list of CORS domains for API endpoint.
	fn cors_domains(signer_address: Option<(String, u16)>, extra_cors: Option<Vec<String>>) -> Vec<String> {
		let basic_cors = match signer_address {
			Some(signer_address) => vec![
				format!("http://{}{}", HOME_PAGE, DAPPS_DOMAIN),
				format!("http://{}{}:{}", HOME_PAGE, DAPPS_DOMAIN, signer_address.1),
				format!("http://{}", address(&signer_address)),
				format!("https://{}{}", HOME_PAGE, DAPPS_DOMAIN),
				format!("https://{}{}:{}", HOME_PAGE, DAPPS_DOMAIN, signer_address.1),
				format!("https://{}", address(&signer_address)),
			],
			None => vec![],
		};

		match extra_cors {
			None => basic_cors,
			Some(extra_cors) => basic_cors.into_iter().chain(extra_cors).collect(),
		}
	}

	fn start_http<A: Authorization + 'static, F: Fetch, T: Middleware<Metadata>>(
		addr: &SocketAddr,
		hosts: Option<Vec<String>>,
		extra_cors: Option<Vec<String>>,
		authorization: A,
		handler: RpcHandler<Metadata, T>,
		dapps_path: PathBuf,
		extra_dapps: Vec<PathBuf>,
		signer_address: Option<(String, u16)>,
		registrar: Arc<ContractClient>,
		sync_status: Arc<SyncStatus>,
		web_proxy_tokens: Arc<WebProxyTokens>,
		remote: Remote,
		fetch: F,
	) -> Result<Server, ServerError> {
		let panic_handler = Arc::new(Mutex::new(None));
		let authorization = Arc::new(authorization);
		let content_fetcher = Arc::new(apps::fetcher::ContentFetcher::new(
			hash_fetch::urlhint::URLHintContract::new(registrar),
			sync_status,
			signer_address.clone(),
			remote.clone(),
			fetch.clone(),
		));
		let endpoints = Arc::new(apps::all_endpoints(
			dapps_path,
			extra_dapps,
			signer_address.clone(),
			web_proxy_tokens,
			remote.clone(),
			fetch.clone(),
		));
		let cors_domains = Self::cors_domains(signer_address.clone(), extra_cors);

		let special = Arc::new({
			let mut special = HashMap::new();
			special.insert(router::SpecialEndpoint::Rpc, rpc::rpc(handler, cors_domains.clone(), panic_handler.clone()));
			special.insert(router::SpecialEndpoint::Utils, apps::utils());
			special.insert(
				router::SpecialEndpoint::Api,
				api::RestApi::new(cors_domains, endpoints.clone(), content_fetcher.clone())
			);
			special
		});
		let hosts = Self::allowed_hosts(hosts, format!("{}", addr));

		hyper::Server::http(addr)?
			.handle(move |ctrl| router::Router::new(
				ctrl,
				signer_address.clone(),
				content_fetcher.clone(),
				endpoints.clone(),
				special.clone(),
				authorization.clone(),
				hosts.clone(),
			))
			.map(|(l, srv)| {

				::std::thread::spawn(move || {
					srv.run();
				});

				Server {
					server: Some(l),
					panic_handler: panic_handler,
				}
			})
			.map_err(ServerError::from)
	}

	/// Set callback for panics.
	pub fn set_panic_handler<F>(&self, handler: F) where F : Fn() -> () + Send + 'static {
		*self.panic_handler.lock().unwrap() = Some(Box::new(handler));
	}

	#[cfg(test)]
	/// Returns address that this server is bound to.
	pub fn addr(&self) -> &SocketAddr {
		self.server.as_ref()
			.expect("server is always Some at the start; it's consumed only when object is dropped; qed")
			.addrs()
			.first()
			.expect("You cannot start the server without binding to at least one address; qed")
	}
}

impl Drop for Server {
	fn drop(&mut self) {
		self.server.take().unwrap().close()
	}
}

/// Webapp Server startup error
#[derive(Debug)]
pub enum ServerError {
	/// Wrapped `std::io::Error`
	IoError(std::io::Error),
	/// Other `hyper` error
	Other(hyper::error::Error),
	/// Fetch service initialization error
	FetchInitialization,
}

impl From<hyper::error::Error> for ServerError {
	fn from(err: hyper::error::Error) -> Self {
		match err {
			hyper::error::Error::Io(e) => ServerError::IoError(e),
			e => ServerError::Other(e),
		}
	}
}

/// Random filename
fn random_filename() -> String {
	use ::rand::Rng;
	let mut rng = ::rand::OsRng::new().unwrap();
	rng.gen_ascii_chars().take(12).collect()
}

fn address(address: &(String, u16)) -> String {
	format!("{}:{}", address.0, address.1)
}

#[cfg(test)]
mod util_tests {
	use super::Server;

	#[test]
	fn should_return_allowed_hosts() {
		// given
		let bind_address = "127.0.0.1".to_owned();

		// when
		let all = Server::allowed_hosts(None, bind_address.clone());
		let address = Server::allowed_hosts(Some(Vec::new()), bind_address.clone());
		let some = Server::allowed_hosts(Some(vec!["ethcore.io".into()]), bind_address.clone());

		// then
		assert_eq!(all, None);
		assert_eq!(address, Some(vec!["localhost".into(), "127.0.0.1".into()]));
		assert_eq!(some, Some(vec!["ethcore.io".into(), "localhost".into(), "127.0.0.1".into()]));
	}

	#[test]
	fn should_return_cors_domains() {
		// given

		// when
		let none = Server::cors_domains(None, None);
		let some = Server::cors_domains(Some(("127.0.0.1".into(), 18180)), None);
		let extra = Server::cors_domains(None, Some(vec!["all".to_owned()]));

		// then
		assert_eq!(none, Vec::<String>::new());
		assert_eq!(some, vec![
			"http://parity.web3.site".to_owned(),
			"http://parity.web3.site:18180".into(),
			"http://127.0.0.1:18180".into(),
			"https://parity.web3.site".into(),
			"https://parity.web3.site:18180".into(),
			"https://127.0.0.1:18180".into()
		]);
		assert_eq!(extra, vec!["all".to_owned()]);
	}
}
