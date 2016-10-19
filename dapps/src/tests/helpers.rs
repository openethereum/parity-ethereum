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

use std::env;
use std::str;
use std::sync::Arc;
use rustc_serialize::hex::FromHex;

use ServerBuilder;
use Server;
use apps::urlhint::ContractClient;
use util::{Bytes, Address, Mutex, ToPretty};
use devtools::http_client;

const REGISTRAR: &'static str = "8e4e9b13d4b45cb0befc93c3061b1408f67316b2";
const URLHINT: &'static str = "deadbeefcafe0000000000000000000000000000";

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

pub fn init_server(hosts: Option<Vec<String>>) -> (Server, Arc<FakeRegistrar>) {
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");
	let builder = ServerBuilder::new(dapps_path.to_str().unwrap().into(), registrar.clone());
	(
		builder.start_unsecured_http(&"127.0.0.1:0".parse().unwrap(), hosts).unwrap(),
		registrar,
	)
}

pub fn serve_with_auth(user: &str, pass: &str) -> Server {
	let registrar = Arc::new(FakeRegistrar::new());
	let mut dapps_path = env::temp_dir();
	dapps_path.push("non-existent-dir-to-prevent-fs-files-from-loading");
	let builder = ServerBuilder::new(dapps_path.to_str().unwrap().into(), registrar);
	builder.start_basic_auth_http(&"127.0.0.1:0".parse().unwrap(), None, user, pass).unwrap()
}

pub fn serve_hosts(hosts: Option<Vec<String>>) -> Server {
	init_server(hosts).0
}

pub fn serve_with_registrar() -> (Server, Arc<FakeRegistrar>) {
	init_server(None)
}

pub fn serve() -> Server {
	init_server(None).0
}

pub fn request(server: Server, request: &str) -> http_client::Response {
	http_client::request(server.addr(), request)
}

pub fn assert_security_headers(headers: &[String]) {
	http_client::assert_security_headers_present(headers)
}
