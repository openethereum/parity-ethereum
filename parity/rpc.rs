// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::fmt;
use std::sync::Arc;
use std::net::SocketAddr;
use std::io;
use io::PanicHandler;
use ethcore_rpc::{RpcServerError, RpcServer as Server};
use jsonipc;
use rpc_apis;
use rpc_apis::ApiSet;
use helpers::parity_ipc_path;

pub use jsonipc::Server as IpcServer;
pub use ethcore_rpc::Server as HttpServer;

#[derive(Debug, PartialEq)]
pub struct HttpConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: ApiSet,
	pub cors: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
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
			socket_addr: parity_ipc_path("$HOME/.parity/jsonrpc.ipc"),
			apis: ApiSet::UnsafeContext,
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
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
}

pub fn new_http(conf: HttpConfiguration, deps: &Dependencies) -> Result<Option<HttpServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let url = format!("{}:{}", conf.interface, conf.port);
	let addr = try!(url.parse().map_err(|_| format!("Invalid JSONRPC listen host/port given: {}", url)));
	Ok(Some(try!(setup_http_rpc_server(deps, &addr, conf.cors, conf.hosts, conf.apis))))
}

fn setup_rpc_server(apis: ApiSet, deps: &Dependencies) -> Result<Server, String> {
	let server = Server::new();
	Ok(rpc_apis::setup_rpc(server, deps.apis.clone(), apis))
}

pub fn setup_http_rpc_server(
	dependencies: &Dependencies,
	url: &SocketAddr,
	cors_domains: Option<Vec<String>>,
	allowed_hosts: Option<Vec<String>>,
	apis: ApiSet
) -> Result<HttpServer, String> {
	let server = try!(setup_rpc_server(apis, dependencies));
	let ph = dependencies.panic_handler.clone();
	let start_result = server.start_http(url, cors_domains, allowed_hosts, ph);
	match start_result {
		Err(RpcServerError::IoError(err)) => match err.kind() {
			io::ErrorKind::AddrInUse => Err(format!("RPC address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --jsonrpc-port and --jsonrpc-interface options.", url)),
			_ => Err(format!("RPC io error: {}", err)),
		},
		Err(e) => Err(format!("RPC error: {:?}", e)),
		Ok(server) => Ok(server),
	}
}

pub fn new_ipc(conf: IpcConfiguration, deps: &Dependencies) -> Result<Option<IpcServer>, String> {
	if !conf.enabled { return Ok(None); }
	Ok(Some(try!(setup_ipc_rpc_server(deps, &conf.socket_addr, conf.apis))))
}

pub fn setup_ipc_rpc_server(dependencies: &Dependencies, addr: &str, apis: ApiSet) -> Result<IpcServer, String> {
	let server = try!(setup_rpc_server(apis, dependencies));
	match server.start_ipc(addr) {
		Err(jsonipc::Error::Io(io_error)) => Err(format!("RPC io error: {}", io_error)),
		Err(any_error) => Err(format!("Rpc error: {:?}", any_error)),
		Ok(server) => Ok(server)
	}
}
