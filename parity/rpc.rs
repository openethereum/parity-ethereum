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

use std::{io, fmt};
use std::sync::Arc;

use dapps;
use dir::default_data_path;
use ethcore_rpc::informant::{RpcStats, Middleware};
use ethcore_rpc::{self as rpc, HttpServerError, Metadata, Origin, AccessControlAllowOrigin, Host};
use helpers::parity_ipc_path;
use jsonrpc_core::MetaIoHandler;
use parity_reactor::TokioRemote;
use rpc_apis::{self, ApiSet};

pub use ethcore_rpc::{IpcServer, HttpServer, RequestMiddleware};

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
		let data_dir = default_data_path();
		IpcConfiguration {
			enabled: true,
			socket_addr: parity_ipc_path(&data_dir, "$BASE/jsonrpc.ipc"),
			apis: ApiSet::IpcContext,
		}
	}
}

impl fmt::Display for IpcConfiguration {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.enabled {
			write!(f, "endpoint address [{}], api list [{:?}]", self.socket_addr, self.apis)
		} else {
			write!(f, "disabled")
		}
	}
}

pub struct Dependencies {
	pub apis: Arc<rpc_apis::Dependencies>,
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

fn setup_apis(apis: ApiSet, deps: &Dependencies) -> MetaIoHandler<Metadata, Middleware> {
	rpc_apis::setup_rpc(deps.stats.clone(), deps.apis.clone(), apis)
}

pub fn new_http(conf: HttpConfiguration, deps: &Dependencies, middleware: Option<dapps::Middleware>) -> Result<Option<HttpServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let url = format!("{}:{}", conf.interface, conf.port);
	let addr = url.parse().map_err(|_| format!("Invalid JSONRPC listen host/port given: {}", url))?;
	let handler = setup_apis(conf.apis, deps);
	let remote = deps.remote.clone();

	let cors_domains: Option<Vec<_>> = conf.cors.map(|domains| domains.into_iter().map(AccessControlAllowOrigin::from).collect());
	let allowed_hosts: Option<Vec<_>> = conf.hosts.map(|hosts| hosts.into_iter().map(Host::from).collect());

	let start_result = rpc::start_http(
		&addr,
		cors_domains.into(),
		allowed_hosts.into(),
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
		Err(HttpServerError::Io(err)) => match err.kind() {
			io::ErrorKind::AddrInUse => Err(
				format!("RPC address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --jsonrpc-port and --jsonrpc-interface options.", url)
			),
			_ => Err(format!("RPC io error: {}", err)),
		},
		Err(e) => Err(format!("RPC error: {:?}", e)),
	}
}

pub fn new_ipc(conf: IpcConfiguration, dependencies: &Dependencies) -> Result<Option<IpcServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}
	let handler = setup_apis(conf.apis, dependencies);
	let remote = dependencies.remote.clone();
	match rpc::start_ipc(&conf.socket_addr, handler, remote, RpcExtractor) {
		Ok(server) => Ok(Some(server)),
		Err(io_error) => Err(format!("RPC io error: {}", io_error)),
	}
}

#[cfg(test)]
mod tests {
	use super::RpcExtractor;
	use ethcore_rpc::{HttpMetaExtractor, Origin};

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
