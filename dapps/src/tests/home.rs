// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use tests::helpers::{serve_ui, request, assert_security_headers};

#[test]
fn should_serve_home_js() {
	// given
	let server = serve_ui();

	// when
	let response = request(server,
		"\
			GET /inject.js HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	response.assert_status("HTTP/1.1 200 OK");
	response.assert_header("Content-Type", "application/javascript");
	assert_eq!(response.body.contains("function(){"), true, "Expected function in: {}", response.body);
	assert_security_headers(&response.headers);
}

#[test]
fn should_serve_home() {
	// given
	let server = serve_ui();

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
	response.assert_status("HTTP/1.1 200 OK");
	response.assert_header("Content-Type", "text/html");
	assert_security_headers(&response.headers);
}
