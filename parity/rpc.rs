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


use std::str::FromStr;
use std::sync::Arc;
use std::net::SocketAddr;
use util::panics::PanicHandler;
use die::*;
use jsonipc;
use rpc_apis;
use std::fmt;

#[cfg(feature = "rpc")]
pub use ethcore_rpc::Server as RpcServer;
#[cfg(feature = "rpc")]
use ethcore_rpc::{RpcServerError, RpcServer as Server};
#[cfg(not(feature = "rpc"))]
pub struct RpcServer;

pub struct HttpConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: String,
	pub cors: Vec<String>,
}

pub struct IpcConfiguration {
	pub enabled: bool,
	pub socket_addr: String,
	pub apis: String,
}

impl fmt::Display for IpcConfiguration {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.enabled {
			write!(f, "endpoint address [{}], api list [{}]", self.socket_addr, self.apis)
		}
		else {
			write!(f, "disabled")
		}
	}
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
}

pub fn new_http(conf: HttpConfiguration, deps: &Dependencies) -> Option<RpcServer> {
	if !conf.enabled {
		return None;
	}

	let apis = conf.apis.split(',').collect();
	let url = format!("{}:{}", conf.interface, conf.port);
	let addr = SocketAddr::from_str(&url).unwrap_or_else(|_| die!("{}: Invalid JSONRPC listen host/port given.", url));

	Some(setup_http_rpc_server(deps, &addr, conf.cors, apis))
}

fn setup_rpc_server(apis: Vec<&str>, deps: &Dependencies) -> Server {
	let apis = rpc_apis::from_str(apis);
	let server = Server::new();
	rpc_apis::setup_rpc(server, deps.apis.clone(), rpc_apis::ApiSet::List(apis))
}

#[cfg(not(feature = "rpc"))]
pub fn setup_http_rpc_server(
	_deps: &Dependencies,
	_url: &SocketAddr,
	_cors_domain: Vec<String>,
	_apis: Vec<&str>,
) -> ! {
	die!("Your Parity version has been compiled without JSON-RPC support.")
}

#[cfg(feature = "rpc")]
pub fn setup_http_rpc_server(
	dependencies: &Dependencies,
	url: &SocketAddr,
	cors_domains: Vec<String>,
	apis: Vec<&str>,
) -> RpcServer {
	let server = setup_rpc_server(apis, dependencies);
	let start_result = server.start_http(url, cors_domains);
	let ph = dependencies.panic_handler.clone();
	match start_result {
		Err(RpcServerError::IoError(err)) => die_with_io_error("RPC", err),
		Err(e) => die!("RPC: {:?}", e),
		Ok(server) => {
			server.set_panic_handler(move || {
				ph.notify_all("Panic in RPC thread.".to_owned());
			});
			server
		},
	}
}

#[cfg(not(feature = "rpc"))]
pub fn setup_ipc_rpc_server(_dependencies: &Dependencies, _addr: &str, _apis: Vec<&str>) -> ! {
	die!("Your Parity version has been compiled without JSON-RPC support.")
}

pub fn new_ipc(conf: IpcConfiguration, deps: &Dependencies) -> Option<jsonipc::Server> {
	if !conf.enabled { return None; }
	let apis = conf.apis.split(',').collect();
	Some(setup_ipc_rpc_server(deps, &conf.socket_addr, apis))
}

#[cfg(feature = "rpc")]
pub fn setup_ipc_rpc_server(dependencies: &Dependencies, addr: &str, apis: Vec<&str>) -> jsonipc::Server {
	let server = setup_rpc_server(apis, dependencies);
	match server.start_ipc(addr) {
		Err(jsonipc::Error::Io(io_error)) => die_with_io_error("RPC", io_error),
		Err(any_error) => die!("RPC: {:?}", any_error),
		Ok(server) => server
	}
}
