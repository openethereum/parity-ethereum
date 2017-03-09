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

use std::env;
use std::str;
use std::ops::Deref;
use std::sync::Arc;
use env_logger::LogBuilder;
use ethcore_rpc::Metadata;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_core::reactor::RpcEventLoop;

use ServerBuilder;
use Server;
use fetch::Fetch;
use devtools::http_client;
use parity_reactor::Remote;

mod registrar;
mod fetch;

use self::registrar::FakeRegistrar;
use self::fetch::FakeFetch;

const SIGNER_PORT: u16 = 18180;

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

pub fn init_server<F, B>(process: F, io: MetaIoHandler<Metadata>, remote: Remote) -> (ServerLoop, Arc<FakeRegistrar>) where
	F: FnOnce(ServerBuilder) -> ServerBuilder<B>,
	B: Fetch,
{
	init_logger();
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");

	// TODO [ToDr] When https://github.com/ethcore/jsonrpc/issues/26 is resolved
	// this additional EventLoop wouldn't be needed, we should be able to re-use remote.
	let event_loop = RpcEventLoop::spawn();
	let handler = event_loop.handler(Arc::new(io));
	let server = process(ServerBuilder::new(
		&dapps_path, registrar.clone(), remote,
	))
		.signer_address(Some(("127.0.0.1".into(), SIGNER_PORT)))
		.start_unsecured_http(&"127.0.0.1:0".parse().unwrap(), handler).unwrap();
	(
		ServerLoop { server: server, event_loop: event_loop },
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
	let server = ServerBuilder::new(&dapps_path, registrar, Remote::new(event_loop.remote()))
		.signer_address(Some(("127.0.0.1".into(), SIGNER_PORT)))
		.allowed_hosts(None)
		.start_basic_auth_http(&"127.0.0.1:0".parse().unwrap(), user, pass, handler).unwrap();
	ServerLoop {
		server: server,
		event_loop: event_loop,
	}
}

pub fn serve_with_rpc(io: MetaIoHandler<Metadata>) -> ServerLoop {
	init_server(|builder| builder.allowed_hosts(None), io, Remote::new_sync()).0
}

pub fn serve_hosts(hosts: Option<Vec<String>>) -> ServerLoop {
	init_server(|builder| builder.allowed_hosts(hosts), Default::default(), Remote::new_sync()).0
}

pub fn serve_extra_cors(extra_cors: Option<Vec<String>>) -> ServerLoop {
	init_server(|builder| builder.allowed_hosts(None).extra_cors_headers(extra_cors), Default::default(), Remote::new_sync()).0
}

pub fn serve_with_registrar() -> (ServerLoop, Arc<FakeRegistrar>) {
	init_server(|builder| builder.allowed_hosts(None), Default::default(), Remote::new_sync())
}

pub fn serve_with_registrar_and_sync() -> (ServerLoop, Arc<FakeRegistrar>) {
	init_server(|builder| {
		builder
			.sync_status(Arc::new(|| true))
			.allowed_hosts(None)
	}, Default::default(), Remote::new_sync())
}

pub fn serve_with_registrar_and_fetch() -> (ServerLoop, FakeFetch, Arc<FakeRegistrar>) {
	serve_with_registrar_and_fetch_and_threads(false)
}

pub fn serve_with_registrar_and_fetch_and_threads(multi_threaded: bool) -> (ServerLoop, FakeFetch, Arc<FakeRegistrar>) {
	let fetch = FakeFetch::default();
	let f = fetch.clone();
	let (server, reg) = init_server(move |builder| {
		builder.allowed_hosts(None).fetch(f.clone())
	}, Default::default(), if multi_threaded { Remote::new_thread_per_future() } else { Remote::new_sync() });

	(server, fetch, reg)
}

pub fn serve_with_fetch(web_token: &'static str) -> (ServerLoop, FakeFetch) {
	let fetch = FakeFetch::default();
	let f = fetch.clone();
	let (server, _) = init_server(move |builder| {
		builder
			.allowed_hosts(None)
			.fetch(f.clone())
			.web_proxy_tokens(Arc::new(move |token| &token == web_token))
	}, Default::default(), Remote::new_sync());

	(server, fetch)
}

pub fn serve() -> ServerLoop {
	init_server(|builder| builder.allowed_hosts(None), Default::default(), Remote::new_sync()).0
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
