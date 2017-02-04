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

use tests::helpers::{serve_hosts, request};

#[test]
fn should_reject_invalid_host() {
	// given
	let server = serve_hosts(Some(vec!["localhost:8080".into()]));

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 403 Forbidden".to_owned());
	assert!(response.body.contains("Current Host Is Disallowed"), response.body);
}

#[test]
fn should_allow_valid_host() {
	// given
	let server = serve_hosts(Some(vec!["localhost:8080".into()]));

	// when
	let response = request(server,
		"\
			GET /ui/ HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
}

#[test]
fn should_serve_dapps_domains() {
	// given
	let server = serve_hosts(Some(vec!["localhost:8080".into()]));

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: ui.web3.site\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
}

#[test]
// NOTE [todr] This is required for error pages to be styled properly.
fn should_allow_parity_utils_even_on_invalid_domain() {
	// given
	let server = serve_hosts(Some(vec!["localhost:8080".into()]));

	// when
	let response = request(server,
		"\
			GET /parity-utils/styles.css HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
}

#[test]
fn should_not_return_cors_headers_for_rpc() {
	// given
	let server = serve_hosts(Some(vec!["localhost:8080".into()]));

	// when
	let response = request(server,
		"\
			POST /rpc HTTP/1.1\r\n\
			Host: localhost:8080\r\n\
			Origin: null\r\n\
			Content-Type: application/json\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
	assert!(
		!response.headers_raw.contains("Access-Control-Allow-Origin"),
		"CORS headers were not expected: {:?}",
		response.headers
	);
}

