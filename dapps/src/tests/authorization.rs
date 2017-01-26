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

use tests::helpers::{serve_with_auth, request, assert_security_headers_for_embed};

#[test]
fn should_require_authorization() {
	// given
	let server = serve_with_auth("test", "test");

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
	assert_eq!(response.status, "HTTP/1.1 401 Unauthorized".to_owned());
	assert_eq!(response.headers.get(0).unwrap(), "WWW-Authenticate: Basic realm=\"Parity\"");
}

#[test]
fn should_reject_on_invalid_auth() {
	// given
	let server = serve_with_auth("test", "test");

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			Authorization: Basic QWxhZGRpbjpPcGVuU2VzYW1l\r\n
			\r\n\
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 401 Unauthorized".to_owned());
	assert!(response.body.contains("Unauthorized"), response.body);
	assert_eq!(response.headers_raw.contains("WWW-Authenticate"), false);
}

#[test]
fn should_allow_on_valid_auth() {
	// given
	let server = serve_with_auth("Aladdin", "OpenSesame");

	// when
	let response = request(server,
		"\
			GET /ui/ HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			Authorization: Basic QWxhZGRpbjpPcGVuU2VzYW1l\r\n
			\r\n\
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
	assert_security_headers_for_embed(&response.headers);
}
