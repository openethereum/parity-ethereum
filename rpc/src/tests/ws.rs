// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! WebSockets server tests.

use std::sync::Arc;

use jsonrpc_core::MetaIoHandler;
use ws;

use v1::{extractors, informant};
use tests::helpers::{GuardedAuthCodes, Server};
use tests::http_client;

/// Setup a mock signer for tests
pub fn serve() -> (Server<ws::Server>, usize, GuardedAuthCodes) {
	let address = "127.0.0.1:0".parse().unwrap();
	let io = MetaIoHandler::default();
	let authcodes = GuardedAuthCodes::default();
	let stats = Arc::new(informant::RpcStats::default());

	let res = Server::new(|_| ::start_ws(
		&address,
		io,
		ws::DomainsValidation::Disabled,
		ws::DomainsValidation::Disabled,
		5,
		extractors::WsExtractor::new(Some(&authcodes.path)),
		extractors::WsExtractor::new(Some(&authcodes.path)),
		extractors::WsStats::new(stats),
	).unwrap());
	let port = res.addr().port() as usize;

	(res, port, authcodes)
}

/// Test a single request to running server
pub fn request(server: Server<ws::Server>, request: &str) -> http_client::Response {
	http_client::request(server.server.addr(), request)
}

#[cfg(test)]
mod testing {
	use std::time;
	use hash::keccak;
	use super::{serve, request, http_client};

	#[test]
	fn should_not_redirect_to_parity_host() {
		// given
		let (server, port, _) = serve();

		// when
		let response = request(server,
			&format!("\
				GET / HTTP/1.1\r\n\
				Host: 127.0.0.1:{}\r\n\
				Connection: close\r\n\
				\r\n\
				{{}}
			", port)
		);

		// then
		assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
	}

	#[test]
	fn should_block_if_authorization_is_incorrect() {
		// given
		let (server, port, _) = serve();

		// when
		let response = request(server,
			&format!("\
				GET / HTTP/1.1\r\n\
				Host: 127.0.0.1:{}\r\n\
				Connection: Upgrade\r\n\
				Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==\r\n\
				Sec-WebSocket-Protocol: wrong\r\n\
				Sec-WebSocket-Version: 13\r\n\
				\r\n\
				{{}}
			", port)
		);

		// then
		assert_eq!(response.status, "HTTP/1.1 403 Forbidden".to_owned());
		http_client::assert_security_headers_present(&response.headers, None);
	}

	#[cfg(not(target_os = "windows"))]
	#[test]
	fn should_allow_if_authorization_is_correct() {
		// given
		let (server, port, mut authcodes) = serve();
		let code = authcodes.generate_new().unwrap().replace("-", "");
		authcodes.to_file(&authcodes.path).unwrap();
		let timestamp = time::UNIX_EPOCH.elapsed().unwrap().as_secs();

		// when
		let response = request(server,
			&format!("\
				GET / HTTP/1.1\r\n\
				Host: 127.0.0.1:{}\r\n\
				Connection: Close\r\n\
				Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==\r\n\
				Sec-WebSocket-Protocol: {:x}_{}\r\n\
				Sec-WebSocket-Version: 13\r\n\
				\r\n\
				{{}}
			",
			port,
			keccak(format!("{}:{}", code, timestamp)),
			timestamp,
			)
		);

		// then
		assert_eq!(response.status, "HTTP/1.1 101 Switching Protocols".to_owned());
	}

	#[test]
	fn should_not_allow_initial_connection_even_once() {
		// given
		let (server, port, authcodes) = serve();
		let code = "initial";
		let timestamp = time::UNIX_EPOCH.elapsed().unwrap().as_secs();
		assert!(authcodes.is_empty());

		// when
		let response1 = http_client::request(server.addr(),
			&format!("\
				GET / HTTP/1.1\r\n\
				Host: 127.0.0.1:{}\r\n\
				Connection: Close\r\n\
				Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==\r\n\
				Sec-WebSocket-Protocol:{:x}_{}\r\n\
				Sec-WebSocket-Version: 13\r\n\
				\r\n\
				{{}}
			",
			port,
			keccak(format!("{}:{}", code, timestamp)),
			timestamp,
			)
		);

		// then
		assert_eq!(response1.status, "HTTP/1.1 403 Forbidden".to_owned());
		http_client::assert_security_headers_present(&response1.headers, None);
	}
}
