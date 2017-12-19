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

use devtools::http_client;
use rustc_hex::FromHex;
use tests::helpers::{
	serve_with_registrar, serve_with_registrar_and_sync, serve_with_fetch,
	serve_with_registrar_and_fetch,
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
			Host: 1472a9e190620cdf6b31f383373e45efcfe869a820c91f9ccd7eb9fb45e4985d.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 404 Not Found");
	assert_eq!(registrar.calls.lock().len(), 4);
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
			Host: 1472a9e190620cdf6b31f383373e45efcfe869a820c91f9ccd7eb9fb45e4985d.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 503 Service Unavailable");
	assert_eq!(registrar.calls.lock().len(), 2);
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
			Host: 94f093625c06887d94d9fee0d5f9cc4aaa46f33d24d1c7e4b5237e7c37d547dd.web3.site\r\n\
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
			Host: 2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e.web3.site\r\n\
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
fn should_return_fetched_dapp_content() {
	// given
	let (server, fetch, registrar) = serve_with_registrar_and_fetch();
	let gavcoin = GAVCOIN_DAPP.from_hex().unwrap();
	registrar.set_result(
		"9c94e154dab8acf859b30ee80fc828fb1d38359d938751b65db71d460588d82a".parse().unwrap(),
		Ok(gavcoin.clone())
	);
	fetch.set_response(include_bytes!("../../res/gavcoin.zip"));

	// when
	let response1 = http_client::request(server.addr(),
		"\
			GET /index.html HTTP/1.1\r\n\
			Host: 9c94e154dab8acf859b30ee80fc828fb1d38359d938751b65db71d460588d82a.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);
	let response2 = http_client::request(server.addr(),
		"\
			GET /manifest.json HTTP/1.1\r\n\
			Host: 9c94e154dab8acf859b30ee80fc828fb1d38359d938751b65db71d460588d82a.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	assert_eq!(registrar.calls.lock().len(), 4);

	fetch.assert_requested("https://codeload.github.com/gavofyork/gavcoin/zip/9faf32e1e3845e237cc6efd27187cee13b3b99db");
	fetch.assert_no_more_requests();

	response1.assert_status("HTTP/1.1 200 OK");
	assert_security_headers_for_embed(&response1.headers);
	assert!(
		response1.body.contains(r#"18
<h1>Hello Gavcoin!</h1>

0

"#),
		"Expected Gavcoin body: {}",
		response1.body
	);

	response2.assert_status("HTTP/1.1 200 OK");
	assert_security_headers_for_embed(&response2.headers);
	assert_eq!(
		response2.body,
		r#"D2
{
  "id": "9c94e154dab8acf859b30ee80fc828fb1d38359d938751b65db71d460588d82a",
  "name": "Gavcoin",
  "description": "Gavcoin",
  "version": "1.0.0",
  "author": "",
  "iconUrl": "icon.png",
  "localUrl": null
}
0

"#
	);
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
			Host: 2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e.web3.site\r\n\
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
		Host: 2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e.web3.site\r\n\
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
fn should_not_request_content_twice() {
	use std::thread;

	// given
	let (server, fetch, registrar) = serve_with_registrar_and_fetch();
	let gavcoin = GAVCOIN_ICON.from_hex().unwrap();
	registrar.set_result(
		"2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e".parse().unwrap(),
		Ok(gavcoin.clone())
	);
	let request_str = "\
		GET / HTTP/1.1\r\n\
		Host: 2be00befcf008bc0e7d9cdefc194db9c75352e8632f48498b5a6bfce9f02c88e.web3.site\r\n\
		Connection: close\r\n\
		\r\n\
	";
	let fire_request = || {
		let addr = server.addr().to_owned();
		let req = request_str.to_owned();
		thread::spawn(move || {
			http_client::request(&addr, &req)
		})
	};
	let control = fetch.manual();

	// when

	// Fire two requests at the same time
	let r1 = fire_request();
	let r2 = fire_request();

	// wait for single request in fetch, the second one should go into waiting state.
	control.wait_for_requests(1);
	control.respond();

	let response1 = r1.join().unwrap();
	let response2 = r2.join().unwrap();

	// then
	fetch.assert_requested("https://raw.githubusercontent.com/ethcore/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/tokens/gavcoin-64x64.png");
	fetch.assert_no_more_requests();
	response1.assert_status("HTTP/1.1 200 OK");
	response2.assert_status("HTTP/1.1 200 OK");
}

#[test]
fn should_encode_and_decode_base32() {
	use base32;

	let encoded = base32::encode(base32::Alphabet::Crockford, "token+https://parity.io".as_bytes());
	assert_eq!("EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY", &encoded);

	let data = base32::decode(base32::Alphabet::Crockford, "EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY").unwrap();
	assert_eq!("token+https://parity.io", &String::from_utf8(data).unwrap());
}

#[test]
fn should_stream_web_content() {
	// given
	let (server, fetch) = serve_with_fetch("token", "https://parity.io");

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY.web.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_requested("https://parity.io/");
	fetch.assert_no_more_requests();
}

#[test]
fn should_support_base32_encoded_web_urls() {
	// given
	let (server, fetch) = serve_with_fetch("token", "https://parity.io");

	// when
	let response = request(server,
		"\
			GET /styles.css?test=123 HTTP/1.1\r\n\
			Host: EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY.web.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_requested("https://parity.io/styles.css?test=123");
	fetch.assert_no_more_requests();
}

#[test]
fn should_correctly_handle_long_label_when_splitted() {
	// given
	let (server, fetch) = serve_with_fetch("xolrg9fePeQyKLnL", "https://contribution.melonport.com");

	// when
	let response = request(server,
		"\
			GET /styles.css?test=123 HTTP/1.1\r\n\
			Host: f1qprwk775k6am35a5wmpk3e9gnpgx3me1sk.mbsfcdqpwx3jd5h7ax39dxq2wvb5dhqpww3fe9t2wrvfdm.web.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_requested("https://contribution.melonport.com/styles.css?test=123");
	fetch.assert_no_more_requests();
}


#[test]
fn should_support_base32_encoded_web_urls_as_path() {
	// given
	let (server, fetch) = serve_with_fetch("token", "https://parity.io");

	// when
	let response = request(server,
		"\
			GET /web/EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY/styles.css?test=123 HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_requested("https://parity.io/styles.css?test=123");
	fetch.assert_no_more_requests();
}

#[test]
fn should_return_error_on_non_whitelisted_domain() {
	// given
	let (server, fetch) = serve_with_fetch("token", "https://ethcore.io");

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY.web.web3.site\r\n\
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
fn should_return_error_on_invalid_token() {
	// given
	let (server, fetch) = serve_with_fetch("test", "https://parity.io");

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY.web.web3.site\r\n\
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
	let (server, fetch) = serve_with_fetch("token", "ftp://parity.io");

	// when
	let response = request(server,
		"\
			GET /web/token/ftp/parity.io/ HTTP/1.1\r\n\
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
fn should_disallow_non_get_requests() {
	// given
	let (server, fetch) = serve_with_fetch("token", "https://parity.io");

	// when
	let response = request(server,
		"\
			POST / HTTP/1.1\r\n\
			Host: EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY.web.web3.site\r\n\
			Content-Type: application/json\r\n\
			Connection: close\r\n\
			\r\n\
			123\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 405 Method Not Allowed");
	assert_security_headers_for_embed(&response.headers);

	fetch.assert_no_more_requests();
}

#[test]
fn should_fix_absolute_requests_based_on_referer() {
	// given
	let (server, fetch) = serve_with_fetch("token", "https://parity.io");

	// when
	let response = request(server,
		"\
			GET /styles.css HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Connection: close\r\n\
			Referer: http://localhost:8080/web/EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY/\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 302 Found");
	response.assert_header("Location", "/web/EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY/styles.css");

	fetch.assert_no_more_requests();
}

#[test]
fn should_fix_absolute_requests_based_on_referer_in_url() {
	// given
	let (server, fetch) = serve_with_fetch("token", "https://parity.io");

	// when
	let response = request(server,
		"\
			GET /styles.css HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Connection: close\r\n\
			Referer: http://localhost:8080/?__referer=web/EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY/\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 302 Found");
	response.assert_header("Location", "/web/EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY/styles.css");

	fetch.assert_no_more_requests();
}
