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

use tests::helpers::{serve_with_registrar, serve_with_registrar_and_sync, request, assert_security_headers};

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
	assert_eq!(response.status, "HTTP/1.1 404 Not Found".to_owned());
	assert_eq!(registrar.calls.lock().len(), 2);
	assert_security_headers(&response.headers);
}

#[test]
fn should_return_503_when_syncing_but_should_make_the_calls() {
	// given
	let (server, registrar) = serve_with_registrar_and_sync();
	{
		let mut responses = registrar.responses.lock();
		let res1 = responses.get(0).unwrap().clone();
		let res2 = responses.get(1).unwrap().clone();
		// Registrar will be called twice - fill up the responses.
		responses.push(res1);
		responses.push(res2);
	}

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
	assert_eq!(response.status, "HTTP/1.1 503 Service Unavailable".to_owned());
	assert_eq!(registrar.calls.lock().len(), 4);
	assert_security_headers(&response.headers);
}
