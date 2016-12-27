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

use devtools::http_client;
use rustc_serialize::hex::FromHex;
use tests::helpers::{
	serve_with_registrar, serve_with_registrar_and_sync, serve_with_registrar_and_fetch, serve_with_fetch,
	request, assert_security_headers_for_embed,
};

#[test]
fn should_resolve_dapp() {
	// given
	let (server, registrar) = serve_with_registrar();

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 1472a9e190620cdf6b31f383373e45efcfe869a820c91f9ccd7eb9fb45e4985d.parity\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 404 Not Found");
	assert_eq!(registrar.calls.lock().len(), 2);
	assert_security_headers_for_embed(&response.headers);
}

#[test]
fn should_return_503_when_syncing_but_should_make_the_calls() {
	// given
	let (server, registrar) = serve_with_registrar_and_sync();

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 1472a9e190620cdf6b31f383373e45efcfe869a820c91f9ccd7eb9fb45e4985d.parity\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 503 Service Unavailable");
	assert_eq!(registrar.calls.lock().len(), 4);
	assert_security_headers_for_embed(&response.headers);
}

const GAVCOIN_DAPP: &'static str = "00000000000000000000000000000000000000000000000000000000000000609faf32e1e3845e237cc6efd27187cee13b3b99db000000000000000000000000000000000000000000000000d8bd350823e28ff75e74a34215faefdc8a52fd8e00000000000000000000000000000000000000000000000000000000000000116761766f66796f726b2f676176636f696e000000000000000000000000000000";
const GAVCOIN_ICON: &'static str = "00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d8bd350823e28ff75e74a34215faefdc8a52fd8e000000000000000000000000000000000000000000000000000000000000007768747470733a2f2f7261772e67697468756275736572636f6e74656e742e636f6d2f657468636f72652f646170702d6173736574732f623838653938336162616131613661363334356238643934343863313562313137646462353430652f746f6b656e732f676176636f696e2d36347836342e706e67000000000000000000";

#[test]
fn should_return_502_on_hash_mismatch() {
	// given
	let (server, fetch, registrar) = serve_with_registrar_and_fetch();
	let gavcoin = GAVCOIN_DAPP.from_hex().unwrap();
	registrar.set_result(
		"94f093625c06887d94d9fee0d5f9cc4aaa46f33d24d1c7e4b5237e7c37d547dd".parse().unwrap(),
		Ok(gavcoin.clone())
	);

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 94f093625c06887d94d9fee0d5f9cc4aaa46f33d24d1c7e4b5237e7c37d547dd.parity\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	assert_eq!(registrar.calls.lock().len(), 4);

	fetch.assert_requested("https://codeload.github.com/gavofyork/gavcoin/zip/9faf32e1e3845e237cc6efd27187cee13b3b99db");
	fetch.assert_no_more_requests();

	response.assert_status("HTTP/1.1 502 Bad Gateway");
	assert!(response.body.contains("HashMismatch"), "Expected hash mismatch response, got: {:?}", response.body);
	assert_security_headers_for_embed(&response.headers);
}

#[test]
fn should_return_error_for_invalid_dapp_zip() {
	// given
	let (server, fetch, registrar) = serve_with_registrar_and_fetch();
	let gavcoin = GAVCOIN_DAPP.from_hex().unwrap();
	registrar.set_result(
		"2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e".parse().unwrap(),
		Ok(gavcoin.clone())
	);

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e.parity\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	assert_eq!(registrar.calls.lock().len(), 4);

	fetch.assert_requested("https://codeload.github.com/gavofyork/gavcoin/zip/9faf32e1e3845e237cc6efd27187cee13b3b99db");
	fetch.assert_no_more_requests();

	response.assert_status("HTTP/1.1 502 Bad Gateway");
	assert!(response.body.contains("InvalidArchive"), "Expected invalid zip response, got: {:?}", response.body);
	assert_security_headers_for_embed(&response.headers);
}

#[test]
fn should_return_fetched_content() {
	// given
	let (server, fetch, registrar) = serve_with_registrar_and_fetch();
	let gavcoin = GAVCOIN_ICON.from_hex().unwrap();
	registrar.set_result(
		"2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e".parse().unwrap(),
		Ok(gavcoin.clone())
	);

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e.parity\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	assert_eq!(registrar.calls.lock().len(), 4);

	fetch.assert_requested("https://raw.githubusercontent.com/ethcore/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/tokens/gavcoin-64x64.png");
	fetch.assert_no_more_requests();

	response.assert_status("HTTP/1.1 200 OK");
	response.assert_security_headers_present(None);
}

#[test]
fn should_cache_content() {
	// given
	let (server, fetch, registrar) = serve_with_registrar_and_fetch();
	let gavcoin = GAVCOIN_ICON.from_hex().unwrap();
	registrar.set_result(
		"2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e".parse().unwrap(),
		Ok(gavcoin.clone())
	);
	let request_str = "\
		GET / HTTP/1.1\r\n\
		Host: 2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e.parity\r\n\
		Connection: close\r\n\
		\r\n\
	";

	let response = http_client::request(server.addr(), request_str);
	fetch.assert_requested("https://raw.githubusercontent.com/ethcore/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/tokens/gavcoin-64x64.png");
	fetch.assert_no_more_requests();
	response.assert_status("HTTP/1.1 200 OK");

	// when
	let response = http_client::request(server.addr(), request_str);

	// then
	fetch.assert_no_more_requests();
	response.assert_status("HTTP/1.1 200 OK");
}

#[test]
fn should_stream_web_content() {
	// given
	let (server, fetch) = serve_with_fetch("token");

	// when
	let response = request(server,
		"\
			GET /web/token/https/ethcore.io/ HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_requested("https://ethcore.io/");
	fetch.assert_no_more_requests();
}

#[test]
fn should_return_error_on_invalid_token() {
	// given
	let (server, fetch) = serve_with_fetch("token");

	// when
	let response = request(server,
		"\
			GET /web/invalidtoken/https/ethcore.io/ HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 400 Bad Request");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_no_more_requests();
}

#[test]
fn should_return_error_on_invalid_protocol() {
	// given
	let (server, fetch) = serve_with_fetch("token");

	// when
	let response = request(server,
		"\
			GET /web/token/ftp/ethcore.io/ HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 400 Bad Request");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_no_more_requests();
}
