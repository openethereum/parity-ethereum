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

use tests::helpers::{serve, request, assert_security_headers, assert_security_headers_for_embed};

#[test]
fn should_redirect_to_home() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 302 Found");
	assert_eq!(response.headers.get(0).unwrap(), "Location: http://127.0.0.1:18180");
}

#[test]
fn should_redirect_to_home_with_domain() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: home.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 302 Found");
	assert_eq!(response.headers.get(0).unwrap(), "Location: http://127.0.0.1:18180");
}

#[test]
fn should_redirect_to_home_when_trailing_slash_is_missing() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			GET /app HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 302 Found");
	assert_eq!(response.headers.get(0).unwrap(), "Location: http://127.0.0.1:18180");
}

#[test]
fn should_display_404_on_invalid_dapp() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			GET /invaliddapp/ HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 404 Not Found");
	assert_security_headers_for_embed(&response.headers);
}

#[test]
fn should_display_404_on_invalid_dapp_with_domain() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: invaliddapp.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
		"
	);

	// then
	response.assert_status("HTTP/1.1 404 Not Found");
	assert_security_headers_for_embed(&response.headers);
}

#[test]
fn should_serve_rpc() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			POST / HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			Content-Type: application/json\r\n
			\r\n\
			{}
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_eq!(response.body, format!("4C\n{}\n\n0\n\n", r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#));
}

#[test]
fn should_serve_rpc_at_slash_rpc() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			POST /rpc HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			Content-Type: application/json\r\n
			\r\n\
			{}
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_eq!(response.body, format!("4C\n{}\n\n0\n\n", r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#));
}


#[test]
fn should_serve_proxy_pac() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			GET /proxy/proxy.pac HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_eq!(response.body, "DB\n\nfunction FindProxyForURL(url, host) {\n\tif (shExpMatch(host, \"home.web3.site\"))\n\t{\n\t\treturn \"PROXY 127.0.0.1:18180\";\n\t}\n\n\tif (shExpMatch(host, \"*.web3.site\"))\n\t{\n\t\treturn \"PROXY 127.0.0.1:8080\";\n\t}\n\n\treturn \"DIRECT\";\n}\n\n0\n\n".to_owned());
	assert_security_headers(&response.headers);
}

#[test]
fn should_serve_utils() {
	// given
	let server = serve();

	// when
	let response = request(server,
		"\
			GET /parity-utils/inject.js HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_eq!(response.body.contains("function(){"), true);
	assert_security_headers(&response.headers);
}
