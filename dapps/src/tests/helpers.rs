// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use std::env;
use std::str;
use std::ops::Deref;
use std::sync::Arc;
use rustc_serialize::hex::FromHex;
use env_logger::LogBuilder;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_core::reactor::RpcEventLoop;

use ServerBuilder;
use Server;
use hash_fetch::urlhint::ContractClient;
use util::{Bytes, Address, Mutex, ToPretty};
use devtools::http_client;
use parity_reactor::Remote;

const REGISTRAR: &'static str = "8e4e9b13d4b45cb0befc93c3061b1408f67316b2";
const URLHINT: &'static str = "deadbeefcafe0000000000000000000000000000";
const SIGNER_PORT: u16 = 18180;

pub struct FakeRegistrar {
	pub calls: Arc<Mutex<Vec<(String, String)>>>,
	pub responses: Mutex<Vec<Result<Bytes, String>>>,
}

impl FakeRegistrar {
	fn new() -> Self {
		FakeRegistrar {
			calls: Arc::new(Mutex::new(Vec::new())),
			responses: Mutex::new(
				vec![
					Ok(format!("000000000000000000000000{}", URLHINT).from_hex().unwrap()),
					Ok(Vec::new())
				]
			),
		}
	}
}

impl ContractClient for FakeRegistrar {
	fn registrar(&self) -> Result<Address, String> {
		Ok(REGISTRAR.parse().unwrap())
	}

	fn call(&self, address: Address, data: Bytes) -> Result<Bytes, String> {
		self.calls.lock().push((address.to_hex(), data.to_hex()));
		self.responses.lock().remove(0)
	}
}

fn init_logger() {
	// Initialize logger
	if let Ok(log) = env::var("RUST_LOG") {
		let mut builder = LogBuilder::new();
		builder.parse(&log);
		let _ = builder.init();	// ignore errors since ./test.sh will call this multiple times.
	}
}

pub struct ServerLoop {
	pub server: Server,
	pub event_loop: RpcEventLoop,
}

impl Deref for ServerLoop {
	type Target = Server;

	fn deref(&self) -> &Self::Target {
		&self.server
	}
}

pub fn init_server(hosts: Option<Vec<String>>, is_syncing: bool) -> (ServerLoop, Arc<FakeRegistrar>) {
	init_logger();
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");

	let event_loop = RpcEventLoop::spawn();
	let handler = event_loop.handler(Arc::new(MetaIoHandler::default()));
	let server = ServerBuilder::new(dapps_path.to_str().unwrap().into(), registrar.clone(), Remote::new(event_loop.remote()))
		.sync_status(Arc::new(move || is_syncing))
		.signer_address(Some(("127.0.0.1".into(), SIGNER_PORT)))
		.allowed_hosts(hosts)
		.start_unsecured_http(&"127.0.0.1:0".parse().unwrap(), handler).unwrap();
	(
		ServerLoop {
			server: server,
			event_loop: event_loop,
		},
		registrar,
	)
}

pub fn serve_with_auth(user: &str, pass: &str) -> ServerLoop {
	init_logger();
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");

	let event_loop = RpcEventLoop::spawn();
	let handler = event_loop.handler(Arc::new(MetaIoHandler::default()));
	let server = ServerBuilder::new(dapps_path.to_str().unwrap().into(), registrar, Remote::new(event_loop.remote()))
		.signer_address(Some(("127.0.0.1".into(), SIGNER_PORT)))
		.allowed_hosts(None)
		.start_basic_auth_http(&"127.0.0.1:0".parse().unwrap(), user, pass, handler).unwrap();
	ServerLoop {
		server: server,
		event_loop: event_loop,
	}
}

pub fn serve_hosts(hosts: Option<Vec<String>>) -> ServerLoop {
	init_server(hosts, false).0
}

pub fn serve_with_registrar() -> (ServerLoop, Arc<FakeRegistrar>) {
	init_server(None, false)
}

pub fn serve_with_registrar_and_sync() -> (ServerLoop, Arc<FakeRegistrar>) {
	init_server(None, true)
}

pub fn serve() -> ServerLoop {
	init_server(None, false).0
}

pub fn request(server: ServerLoop, request: &str) -> http_client::Response {
	http_client::request(server.addr(), request)
}

pub fn assert_security_headers(headers: &[String]) {
	http_client::assert_security_headers_present(headers, None)
}
pub fn assert_security_headers_for_embed(headers: &[String]) {
	http_client::assert_security_headers_present(headers, Some(SIGNER_PORT))
}
