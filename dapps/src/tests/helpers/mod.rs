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

use std::{env, io, str};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use env_logger::LogBuilder;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::{self as http, Host, DomainsValidation};
use parity_reactor::Remote;

use devtools::http_client;
use registrar::{RegistrarClient, Asynchronous};
use fetch::{Fetch, Client as FetchClient};
use node_health::{NodeHealth, TimeChecker, CpuPool};

use {Middleware, SyncStatus, WebProxyTokens};

mod registrar;
mod fetch;

use self::registrar::FakeRegistrar;
use self::fetch::FakeFetch;

const SIGNER_PORT: u16 = 18180;

#[derive(Debug)]
struct FakeSync(bool);
impl SyncStatus for FakeSync {
	fn is_major_importing(&self) -> bool { self.0 }
	fn peers(&self) -> (usize, usize) { (0, 5) }
}

fn init_logger() {
	// Initialize logger
	if let Ok(log) = env::var("RUST_LOG") {
		let mut builder = LogBuilder::new();
		builder.parse(&log);
		let _ = builder.init();	// ignore errors since ./test.sh will call this multiple times.
	}
}

pub fn init_server<F, B>(process: F, io: IoHandler) -> (Server, Arc<FakeRegistrar>) where
	F: FnOnce(ServerBuilder) -> ServerBuilder<B>,
	B: Fetch,
{
	init_logger();
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");

	let mut builder = ServerBuilder::new(FetchClient::new().unwrap(), &dapps_path, registrar.clone());
	builder.signer_address = Some(("127.0.0.1".into(), SIGNER_PORT));
	let server = process(builder).start_unsecured_http(&"127.0.0.1:0".parse().unwrap(), io).unwrap();
	(
		server,
		registrar,
	)
}

pub fn serve_with_rpc(io: IoHandler) -> Server {
	init_server(|builder| builder, io).0
}

pub fn serve_hosts(hosts: Option<Vec<String>>) -> Server {
	let hosts = hosts.map(|hosts| hosts.into_iter().map(Into::into).collect());
	init_server(|mut builder| {
		builder.allowed_hosts = hosts.into();
		builder
	}, Default::default()).0
}

pub fn serve_with_registrar() -> (Server, Arc<FakeRegistrar>) {
	init_server(|builder| builder, Default::default())
}

pub fn serve_with_registrar_and_sync() -> (Server, Arc<FakeRegistrar>) {
	init_server(|mut builder| {
		builder.sync_status = Arc::new(FakeSync(true));
		builder
	}, Default::default())
}

pub fn serve_with_registrar_and_fetch() -> (Server, FakeFetch, Arc<FakeRegistrar>) {
	let fetch = FakeFetch::default();
	let f = fetch.clone();
	let (server, reg) = init_server(move |builder| {
		builder.fetch(f.clone())
	}, Default::default());

	(server, fetch, reg)
}

pub fn serve_with_fetch(web_token: &'static str, domain: &'static str) -> (Server, FakeFetch) {
	let fetch = FakeFetch::default();
	let f = fetch.clone();
	let (server, _) = init_server(move |mut builder| {
		builder.web_proxy_tokens = Arc::new(move |token| {
			if &token == web_token { Some(domain.into()) } else { None }
		});
		builder.fetch(f.clone())
	}, Default::default());

	(server, fetch)
}

pub fn serve() -> Server {
	init_server(|builder| builder, Default::default()).0
}

pub fn serve_ui() -> Server {
	init_server(|mut builder| {
		builder.serve_ui = true;
		builder
	}, Default::default()).0
}

pub fn request(server: Server, request: &str) -> http_client::Response {
	http_client::request(server.addr(), request)
}

pub fn assert_security_headers(headers: &[String]) {
	http_client::assert_security_headers_present(headers, None)
}
pub fn assert_security_headers_for_embed(headers: &[String]) {
	http_client::assert_security_headers_present(headers, Some(SIGNER_PORT))
}


/// Webapps HTTP+RPC server build.
pub struct ServerBuilder<T: Fetch = FetchClient> {
	dapps_path: PathBuf,
	registrar: Arc<RegistrarClient<Call=Asynchronous>>,
	sync_status: Arc<SyncStatus>,
	web_proxy_tokens: Arc<WebProxyTokens>,
	signer_address: Option<(String, u16)>,
	allowed_hosts: DomainsValidation<Host>,
	fetch: T,
	serve_ui: bool,
}

impl ServerBuilder {
	/// Construct new dapps server
	pub fn new<P: AsRef<Path>>(fetch: FetchClient, dapps_path: P, registrar: Arc<RegistrarClient<Call=Asynchronous>>) -> Self {
		ServerBuilder {
			dapps_path: dapps_path.as_ref().to_owned(),
			registrar: registrar,
			sync_status: Arc::new(FakeSync(false)),
			web_proxy_tokens: Arc::new(|_| None),
			signer_address: None,
			allowed_hosts: DomainsValidation::Disabled,
			fetch: fetch,
			serve_ui: false,
		}
	}
}

impl<T: Fetch> ServerBuilder<T> {
	/// Set a fetch client to use.
	pub fn fetch<X: Fetch>(self, fetch: X) -> ServerBuilder<X> {
		ServerBuilder {
			dapps_path: self.dapps_path,
			registrar: self.registrar,
			sync_status: self.sync_status,
			web_proxy_tokens: self.web_proxy_tokens,
			signer_address: self.signer_address,
			allowed_hosts: self.allowed_hosts,
			fetch: fetch,
			serve_ui: self.serve_ui,
		}
	}

	/// Asynchronously start server with no authentication,
	/// returns result with `Server` handle on success or an error.
	pub fn start_unsecured_http(self, addr: &SocketAddr, io: IoHandler) -> io::Result<Server> {
		Server::start_http(
			addr,
			io,
			self.allowed_hosts,
			self.signer_address,
			self.dapps_path,
			vec![],
			self.registrar,
			self.sync_status,
			self.web_proxy_tokens,
			Remote::new_sync(),
			self.fetch,
			self.serve_ui,
		)
	}
}

const DAPPS_DOMAIN: &'static str = "web3.site";

/// Webapps HTTP server.
pub struct Server {
	server: Option<http::Server>,
}

impl Server {
	fn start_http<F: Fetch>(
		addr: &SocketAddr,
		io: IoHandler,
		allowed_hosts: DomainsValidation<Host>,
		signer_address: Option<(String, u16)>,
		dapps_path: PathBuf,
		extra_dapps: Vec<PathBuf>,
		registrar: Arc<RegistrarClient<Call=Asynchronous>>,
		sync_status: Arc<SyncStatus>,
		web_proxy_tokens: Arc<WebProxyTokens>,
		remote: Remote,
		fetch: F,
		serve_ui: bool,
	) -> io::Result<Server> {
		let health = NodeHealth::new(
			sync_status.clone(),
			TimeChecker::new::<String>(&[], CpuPool::new(1)),
			remote.clone(),
		);
		let pool = ::futures_cpupool::CpuPool::new(1);
		let middleware = if serve_ui {
			Middleware::ui(
				pool,
				health,
				DAPPS_DOMAIN.into(),
				registrar,
				sync_status,
				fetch,
				false,
			)
		} else {
			Middleware::dapps(
				pool,
				health,
				signer_address,
				vec![],
				vec![],
				dapps_path,
				extra_dapps,
				DAPPS_DOMAIN.into(),
				registrar,
				sync_status,
				web_proxy_tokens,
				fetch,
			)
		};

		let mut allowed_hosts: Option<Vec<Host>> = allowed_hosts.into();
		allowed_hosts.as_mut().map(|hosts| {
			hosts.push(format!("http://*.{}:*", DAPPS_DOMAIN).into());
			hosts.push(format!("http://*.{}", DAPPS_DOMAIN).into());
		});

		http::ServerBuilder::new(io)
			.request_middleware(middleware)
			.allowed_hosts(allowed_hosts.into())
			.cors(http::DomainsValidation::Disabled)
			.start_http(addr)
			.map(|server| Server {
				server: Some(server),
			})
	}

	/// Returns address that this server is bound to.
	pub fn addr(&self) -> &SocketAddr {
		self.server.as_ref()
			.expect("server is always Some at the start; it's consumed only when object is dropped; qed")
			.address()
	}
}

impl Drop for Server {
	fn drop(&mut self) {
		self.server.take().unwrap().close()
	}
}
