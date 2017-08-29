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

use std::io;
use std::sync::Arc;
use std::path::PathBuf;
use std::collections::HashSet;

use dapps;
use dir::default_data_path;
use helpers::{parity_ipc_path, replace_home};
use jsonrpc_core::MetaIoHandler;
use parity_reactor::TokioRemote;
use parity_rpc::informant::{RpcStats, Middleware};
use parity_rpc::{self as rpc, Metadata, DomainsValidation};
use rpc_apis::{self, ApiSet};

pub use parity_rpc::{IpcServer, HttpServer, RequestMiddleware};
pub use parity_rpc::ws::Server as WsServer;
pub use parity_rpc::informant::CpuPool;

pub const DAPPS_DOMAIN: &'static str = "web3.site";

#[derive(Debug, Clone, PartialEq)]
pub struct HttpConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: ApiSet,
	pub cors: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
	pub server_threads: Option<usize>,
	pub processing_threads: usize,
}

impl HttpConfiguration {
	pub fn address(&self) -> Option<(String, u16)> {
		match self.enabled {
			true => Some((self.interface.clone(), self.port)),
			false => None,
		}
	}
}

impl Default for HttpConfiguration {
	fn default() -> Self {
		HttpConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8545,
			apis: ApiSet::UnsafeContext,
			cors: None,
			hosts: Some(Vec::new()),
			server_threads: None,
			processing_threads: 0,
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct UiConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub hosts: Option<Vec<String>>,
}

impl UiConfiguration {
	pub fn address(&self) -> Option<(String, u16)> {
		match self.enabled {
			true => Some((self.interface.clone(), self.port)),
			false => None,
		}
	}
}

impl From<UiConfiguration> for HttpConfiguration {
	fn from(conf: UiConfiguration) -> Self {
		HttpConfiguration {
			enabled: conf.enabled,
			interface: conf.interface,
			port: conf.port,
			apis: rpc_apis::ApiSet::UnsafeContext,
			cors: None,
			hosts: conf.hosts,
			server_threads: None,
			processing_threads: 0,
		}
	}
}

impl Default for UiConfiguration {
	fn default() -> Self {
		UiConfiguration {
			enabled: true && cfg!(feature = "ui-enabled"),
			port: 8180,
			interface: "127.0.0.1".into(),
			hosts: Some(vec![]),
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct IpcConfiguration {
	pub enabled: bool,
	pub socket_addr: String,
	pub apis: ApiSet,
}

impl Default for IpcConfiguration {
	fn default() -> Self {
		IpcConfiguration {
			enabled: true,
			socket_addr: if cfg!(windows) {
				r"\\.\pipe\jsonrpc.ipc".into()
			} else {
				let data_dir = ::dir::default_data_path();
				parity_ipc_path(&data_dir, "$BASE/jsonrpc.ipc", 0)
			},
			apis: ApiSet::IpcContext,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct WsConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: ApiSet,
	pub origins: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
	pub signer_path: PathBuf,
	pub support_token_api: bool,
	pub ui_address: Option<(String, u16)>,
}

impl Default for WsConfiguration {
	fn default() -> Self {
		let data_dir = default_data_path();
		WsConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8546,
			apis: ApiSet::UnsafeContext,
			origins: Some(vec!["chrome-extension://*".into(), "moz-extension://*".into()]),
			hosts: Some(Vec::new()),
			signer_path: replace_home(&data_dir, "$BASE/signer").into(),
			support_token_api: true,
			ui_address: Some(("127.0.0.1".to_owned(), 8180)),
		}
	}
}

impl WsConfiguration {
	pub fn address(&self) -> Option<(String, u16)> {
		match self.enabled {
			true => Some((self.interface.clone(), self.port)),
			false => None,
		}
	}
}

pub struct Dependencies<D: rpc_apis::Dependencies> {
	pub apis: Arc<D>,
	pub remote: TokioRemote,
	pub stats: Arc<RpcStats>,
	pub pool: Option<CpuPool>,
}

pub fn new_ws<D: rpc_apis::Dependencies>(
	conf: WsConfiguration,
	deps: &Dependencies<D>,
) -> Result<Option<WsServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let domain = DAPPS_DOMAIN;
	let ws_address = (conf.interface, conf.port);
	let url = format!("{}:{}", ws_address.0, ws_address.1);
	let addr = url.parse().map_err(|_| format!("Invalid WebSockets listen host/port given: {}", url))?;


	let pool = deps.pool.clone();
	let full_handler = setup_apis(rpc_apis::ApiSet::SafeContext, deps, pool.clone());
	let handler = {
		let mut handler = MetaIoHandler::with_middleware((
			rpc::WsDispatcher::new(full_handler),
			Middleware::new(deps.stats.clone(), deps.apis.activity_notifier(), pool)
		));
		let apis = conf.apis.list_apis();
		deps.apis.extend_with_set(&mut handler, &apis);

		handler
	};

	let remote = deps.remote.clone();
	let ui_address = conf.ui_address.clone();
	let allowed_origins = into_domains(with_domain(conf.origins, domain, &[ui_address]));
	let allowed_hosts = into_domains(with_domain(conf.hosts, domain, &[Some(ws_address)]));

	let signer_path;
	let path = match conf.support_token_api && conf.ui_address.is_some() {
		true => {
			signer_path = ::signer::codes_path(&conf.signer_path);
			Some(signer_path.as_path())
		},
		false => None
	};
	let start_result = rpc::start_ws(
		&addr,
		handler,
		remote.clone(),
		allowed_origins,
		allowed_hosts,
		rpc::WsExtractor::new(path.clone()),
		rpc::WsExtractor::new(path.clone()),
		rpc::WsStats::new(deps.stats.clone()),
	);

	match start_result {
		Ok(server) => Ok(Some(server)),
		Err(rpc::ws::Error::Io(ref err)) if err.kind() == io::ErrorKind::AddrInUse => Err(
			format!("WebSockets address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --ws-port and --ws-interface options.", url)
		),
		Err(e) => Err(format!("WebSockets error: {:?}", e)),
	}
}

pub fn new_http<D: rpc_apis::Dependencies>(
	id: &str,
	options: &str,
	conf: HttpConfiguration,
	deps: &Dependencies<D>,
	middleware: Option<dapps::Middleware>,
) -> Result<Option<HttpServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let domain = DAPPS_DOMAIN;
	let http_address = (conf.interface, conf.port);
	let url = format!("{}:{}", http_address.0, http_address.1);
	let addr = url.parse().map_err(|_| format!("Invalid {} listen host/port given: {}", id, url))?;
	let pool = deps.pool.clone();
	let handler = setup_apis(conf.apis, deps, pool);
	let remote = deps.remote.clone();

	let cors_domains = into_domains(conf.cors);
	let allowed_hosts = into_domains(with_domain(conf.hosts, domain, &[Some(http_address)]));

	let start_result = rpc::start_http(
		&addr,
		cors_domains,
		allowed_hosts,
		handler,
		remote,
		rpc::RpcExtractor,
		match (conf.server_threads, middleware) {
			(Some(threads), None) => rpc::HttpSettings::Threads(threads),
			(None, middleware) => rpc::HttpSettings::Dapps(middleware),
			(Some(_), Some(_)) => {
				return Err("Dapps and fast multi-threaded RPC server cannot be enabled at the same time.".into())
			},
		}
	);

	match start_result {
		Ok(server) => Ok(Some(server)),
		Err(rpc::HttpServerError::Io(ref err)) if err.kind() == io::ErrorKind::AddrInUse => Err(
			format!("{} address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --{}-port and --{}-interface options.", id, url, options, options)
		),
		Err(e) => Err(format!("{} error: {:?}", id, e)),
	}
}

pub fn new_ipc<D: rpc_apis::Dependencies>(
	conf: IpcConfiguration,
	dependencies: &Dependencies<D>
) -> Result<Option<IpcServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let pool = dependencies.pool.clone();
	let handler = setup_apis(conf.apis, dependencies, pool);
	let remote = dependencies.remote.clone();
	match rpc::start_ipc(&conf.socket_addr, handler, remote, rpc::RpcExtractor) {
		Ok(server) => Ok(Some(server)),
		Err(io_error) => Err(format!("IPC error: {}", io_error)),
	}
}

fn into_domains<T: From<String>>(items: Option<Vec<String>>) -> DomainsValidation<T> {
	items.map(|vals| vals.into_iter().map(T::from).collect()).into()
}

fn with_domain(items: Option<Vec<String>>, domain: &str, addresses: &[Option<(String, u16)>]) -> Option<Vec<String>> {
	items.map(move |items| {
		let mut items = items.into_iter().collect::<HashSet<_>>();
		for address in addresses {
			if let Some((host, port)) = address.clone() {
				items.insert(format!("{}:{}", host, port));
				items.insert(format!("{}:{}", host.replace("127.0.0.1", "localhost"), port));
				items.insert(format!("http://*.{}:{}", domain, port));
				items.insert(format!("http://*.{}", domain)); //proxypac
			}
		}
		items.into_iter().collect()
	})
}

fn setup_apis<D>(apis: ApiSet, deps: &Dependencies<D>, pool: Option<CpuPool>) -> MetaIoHandler<Metadata, Middleware<D::Notifier>>
	where D: rpc_apis::Dependencies
{
	let mut handler = MetaIoHandler::with_middleware(
		Middleware::new(deps.stats.clone(), deps.apis.activity_notifier(), pool)
	);
	let apis = apis.list_apis();
	deps.apis.extend_with_set(&mut handler, &apis);

	handler
}
