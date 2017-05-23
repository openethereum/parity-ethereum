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

use dapps;
use parity_rpc::informant::{RpcStats, Middleware};
use parity_rpc::{self as rpc, HttpServerError, Metadata, Origin, DomainsValidation};
use helpers::parity_ipc_path;
use jsonrpc_core::MetaIoHandler;
use parity_reactor::TokioRemote;
use rpc_apis::{self, ApiSet};

pub use parity_rpc::{IpcServer, HttpServer, RequestMiddleware};
pub use parity_rpc::ws::Server as WsServer;

#[derive(Debug, Clone, PartialEq)]
pub struct HttpConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: ApiSet,
	pub cors: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
	pub threads: Option<usize>,
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
			threads: None,
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

#[derive(Debug, PartialEq)]
pub struct WsConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: ApiSet,
	pub origins: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
}

impl Default for WsConfiguration {
	fn default() -> Self {
		WsConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8546,
			apis: ApiSet::UnsafeContext,
			origins: Some(Vec::new()),
			hosts: Some(Vec::new()),
		}
	}
}

pub struct Dependencies<D: rpc_apis::Dependencies> {
	pub apis: Arc<D>,
	pub remote: TokioRemote,
	pub stats: Arc<RpcStats>,
}

pub struct RpcExtractor;
impl rpc::HttpMetaExtractor for RpcExtractor {
	type Metadata = Metadata;

	fn read_metadata(&self, origin: String, dapps_origin: Option<String>) -> Metadata {
		let mut metadata = Metadata::default();

		metadata.origin = match (origin.as_str(), dapps_origin) {
			("null", Some(dapp)) => Origin::Dapps(dapp.into()),
			_ => Origin::Rpc(origin),
		};

		metadata
	}
}

impl rpc::IpcMetaExtractor<Metadata> for RpcExtractor {
	fn extract(&self, _req: &rpc::IpcRequestContext) -> Metadata {
		let mut metadata = Metadata::default();
		// TODO [ToDr] Extract proper session id when it's available in context.
		metadata.origin = Origin::Ipc(1.into());
		metadata
	}
}

struct WsRpcExtractor;
impl rpc::ws::MetaExtractor<Metadata> for WsRpcExtractor {
	fn extract(&self, req: &rpc::ws::RequestContext) -> Metadata {
		let mut metadata = Metadata::default();
		let id = req.session_id as u64;
		metadata.origin = Origin::Ws(id.into());
		metadata.session = Some(Arc::new(rpc::PubSubSession::new(req.sender())));
		metadata
	}
}

struct WsStats {
	stats: Arc<RpcStats>,
}

impl rpc::ws::SessionStats for WsStats {
	fn open_session(&self, _id: rpc::ws::SessionId) {
		self.stats.open_session()
	}

	fn close_session(&self, _id: rpc::ws::SessionId) {
		self.stats.close_session()
	}
}

fn setup_apis<D>(apis: ApiSet, deps: &Dependencies<D>) -> MetaIoHandler<Metadata, Middleware<D::Notifier>>
	where D: rpc_apis::Dependencies
{
	rpc_apis::setup_rpc(deps.stats.clone(), &*deps.apis, apis)
}

pub fn new_ws<D: rpc_apis::Dependencies>(
	conf: WsConfiguration,
	deps: &Dependencies<D>,
) -> Result<Option<WsServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let url = format!("{}:{}", conf.interface, conf.port);
	let addr = url.parse().map_err(|_| format!("Invalid WebSockets listen host/port given: {}", url))?;
	let handler = setup_apis(conf.apis, deps);
	let remote = deps.remote.clone();
	let allowed_origins = into_domains(conf.origins);
	let allowed_hosts = into_domains(conf.hosts);

	let start_result = rpc::start_ws(
		&addr,
		handler,
		remote.clone(),
		allowed_origins,
		allowed_hosts,
		WsRpcExtractor,
		WsStats {
			stats: deps.stats.clone(),
		},
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
	conf: HttpConfiguration,
	deps: &Dependencies<D>,
	middleware: Option<dapps::Middleware>
) -> Result<Option<HttpServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let url = format!("{}:{}", conf.interface, conf.port);
	let addr = url.parse().map_err(|_| format!("Invalid HTTP JSON-RPC listen host/port given: {}", url))?;
	let handler = setup_apis(conf.apis, deps);
	let remote = deps.remote.clone();

	let cors_domains = into_domains(conf.cors);
	let allowed_hosts = into_domains(conf.hosts);

	let start_result = rpc::start_http(
		&addr,
		cors_domains,
		allowed_hosts,
		handler,
		remote,
		RpcExtractor,
		match (conf.threads, middleware) {
			(Some(threads), None) => rpc::HttpSettings::Threads(threads),
			(None, middleware) => rpc::HttpSettings::Dapps(middleware),
			(Some(_), Some(_)) => {
				return Err("Dapps and fast multi-threaded RPC server cannot be enabled at the same time.".into())
			},
		}
	);

	match start_result {
		Ok(server) => Ok(Some(server)),
		Err(HttpServerError::Io(ref err)) if err.kind() == io::ErrorKind::AddrInUse => Err(
			format!("HTTP address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --jsonrpc-port and --jsonrpc-interface options.", url)
		),
		Err(e) => Err(format!("HTTP error: {:?}", e)),
	}
}

fn into_domains<T: From<String>>(items: Option<Vec<String>>) -> DomainsValidation<T> {
	items.map(|vals| vals.into_iter().map(T::from).collect()).into()
}

pub fn new_ipc<D: rpc_apis::Dependencies>(
	conf: IpcConfiguration,
	dependencies: &Dependencies<D>
) -> Result<Option<IpcServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let handler = setup_apis(conf.apis, dependencies);
	let remote = dependencies.remote.clone();
	let ipc = rpc::start_ipc(
		&conf.socket_addr,
		handler,
		remote,
		RpcExtractor,
	);

	match ipc {
		Ok(server) => Ok(Some(server)),
		Err(io_error) => Err(format!("IPC error: {}", io_error)),
	}
}

#[cfg(test)]
mod tests {
	use super::RpcExtractor;
	use parity_rpc::{HttpMetaExtractor, Origin};

	#[test]
	fn should_extract_rpc_origin() {
		// given
		let extractor = RpcExtractor;

		// when
		let meta = extractor.read_metadata("http://parity.io".into(), None);
		let meta1 = extractor.read_metadata("http://parity.io".into(), Some("ignored".into()));

		// then
		assert_eq!(meta.origin, Origin::Rpc("http://parity.io".into()));
		assert_eq!(meta1.origin, Origin::Rpc("http://parity.io".into()));
	}

	#[test]
	fn should_dapps_origin() {
		// given
		let extractor = RpcExtractor;
		let dapp = "https://wallet.ethereum.org".to_owned();

		// when
		let meta = extractor.read_metadata("null".into(), Some(dapp.clone()));

		// then
		assert_eq!(meta.origin, Origin::Dapps(dapp.into()));
	}
}
