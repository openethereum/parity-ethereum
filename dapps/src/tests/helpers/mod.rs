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
use std::sync::Arc;
use env_logger::LogBuilder;

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

pub fn init_server<F, B>(hosts: Option<Vec<String>>, process: F, remote: Remote) -> (Server, Arc<FakeRegistrar>) where
	F: FnOnce(ServerBuilder) -> ServerBuilder<B>,
	B: Fetch,
{
	init_logger();
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");
	let server = process(ServerBuilder::new(
		&dapps_path, registrar.clone(), remote,
	))
		.signer_address(Some(("127.0.0.1".into(), SIGNER_PORT)))
		.start_unsecured_http(&"127.0.0.1:0".parse().unwrap(), hosts).unwrap();
	(
		server,
		registrar,
	)
}

pub fn serve_with_auth(user: &str, pass: &str) -> Server {
	init_logger();
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");
	ServerBuilder::new(&dapps_path, registrar.clone(), Remote::new_sync())
		.signer_address(Some(("127.0.0.1".into(), SIGNER_PORT)))
		.start_basic_auth_http(&"127.0.0.1:0".parse().unwrap(), None, user, pass).unwrap()
}

pub fn serve_hosts(hosts: Option<Vec<String>>) -> Server {
	init_server(hosts, |builder| builder, Remote::new_sync()).0
}

pub fn serve_with_registrar() -> (Server, Arc<FakeRegistrar>) {
	init_server(None, |builder| builder, Remote::new_sync())
}

pub fn serve_with_registrar_and_sync() -> (Server, Arc<FakeRegistrar>) {
	init_server(None, |builder| {
		builder.sync_status(Arc::new(|| true))
	}, Remote::new_sync())
}

pub fn serve_with_registrar_and_fetch() -> (Server, FakeFetch, Arc<FakeRegistrar>) {
	serve_with_registrar_and_fetch_and_threads(false)
}

pub fn serve_with_registrar_and_fetch_and_threads(multi_threaded: bool) -> (Server, FakeFetch, Arc<FakeRegistrar>) {
	let fetch = FakeFetch::default();
	let f = fetch.clone();
	let (server, reg) = init_server(None, move |builder| {
		builder.fetch(f.clone())
	}, if multi_threaded { Remote::new_thread_per_future() } else { Remote::new_sync() });

	(server, fetch, reg)
}

pub fn serve_with_fetch(web_token: &'static str) -> (Server, FakeFetch) {
	let fetch = FakeFetch::default();
	let f = fetch.clone();
	let (server, _) = init_server(None, move |builder| {
		builder
			.fetch(f.clone())
			.web_proxy_tokens(Arc::new(move |token| &token == web_token))
	}, Remote::new_sync());

	(server, fetch)
}

pub fn serve() -> Server {
	init_server(None, |builder| builder, Remote::new_sync()).0
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
