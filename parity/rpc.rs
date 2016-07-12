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
use util::panics::PanicHandler;
use jsonipc;
use rpc_apis;
use rpc_apis::ApiSet;

pub use ethcore_rpc::Server as RpcServer;
use ethcore_rpc::{RpcServerError, RpcServer as Server};

#[derive(Debug, PartialEq)]
pub struct HttpConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: ApiSet,
	pub cors: Vec<String>,
}

impl Default for HttpConfiguration {
	fn default() -> Self {
		HttpConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8545,
			apis: ApiSet::UnsafeContext,
			cors: vec![],
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
			enabled: false,
			socket_addr: "".into(),
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

pub fn new_http(conf: HttpConfiguration, deps: &Dependencies) -> Result<Option<RpcServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let url = format!("{}:{}", conf.interface, conf.port);
	let addr = try!(url.parse().map_err(|_| format!("Invalid JSONRPC listen host/port given: {}", url)));
	Ok(Some(try!(setup_http_rpc_server(deps, &addr, conf.cors, conf.apis))))
}

fn setup_rpc_server(apis: ApiSet, deps: &Dependencies) -> Result<Server, String> {
	let server = Server::new();
	Ok(rpc_apis::setup_rpc(server, deps.apis.clone(), apis))
}

pub fn setup_http_rpc_server(
	dependencies: &Dependencies,
	url: &SocketAddr,
	cors_domains: Vec<String>,
	apis: ApiSet
) -> Result<RpcServer, String> {
	let server = try!(setup_rpc_server(apis, dependencies));
	let start_result = server.start_http(url, cors_domains);
	let ph = dependencies.panic_handler.clone();
	match start_result {
		Err(RpcServerError::IoError(err)) => Err(format!("RPC io error: {}", err)),
		Err(e) => Err(format!("RPC error: {:?}", e)),
		Ok(server) => {
			server.set_panic_handler(move || {
				ph.notify_all("Panic in RPC thread.".to_owned());
			});
			Ok(server)
		},
	}
}

pub fn new_ipc(conf: IpcConfiguration, deps: &Dependencies) -> Result<Option<jsonipc::Server>, String> {
	if !conf.enabled { return Ok(None); }
	Ok(Some(try!(setup_ipc_rpc_server(deps, &conf.socket_addr, conf.apis))))
}

pub fn setup_ipc_rpc_server(dependencies: &Dependencies, addr: &str, apis: ApiSet) -> Result<jsonipc::Server, String> {
	let server = try!(setup_rpc_server(apis, dependencies));
	match server.start_ipc(addr) {
		Err(jsonipc::Error::Io(io_error)) => Err(format!("RPC io error: {}", io_error)),
		Err(any_error) => Err(format!("Rpc error: {:?}", any_error)),
		Ok(server) => Ok(server)
	}
}
