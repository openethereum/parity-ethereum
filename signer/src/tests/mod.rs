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

use std::ops::{Deref, DerefMut};
use std::time;
use std::sync::Arc;
use devtools::{http_client, RandomTempPath};
use rpc::ConfirmationsQueue;
use util::Hashable;
use rand;

use ServerBuilder;
use Server;
use AuthCodes;

pub struct GuardedAuthCodes {
	authcodes: AuthCodes,
	path: RandomTempPath,
}
impl Deref for GuardedAuthCodes {
	type Target = AuthCodes;
	fn deref(&self) -> &Self::Target {
		&self.authcodes
	}
}
impl DerefMut for GuardedAuthCodes {
	fn deref_mut(&mut self) -> &mut AuthCodes {
		&mut self.authcodes
	}
}

pub fn serve() -> (Server, usize, GuardedAuthCodes) {
	let mut path = RandomTempPath::new();
	path.panic_on_drop_failure = false;
	let queue = Arc::new(ConfirmationsQueue::default());
	let builder = ServerBuilder::new(queue, path.to_path_buf());
	let port = 35000 + rand::random::<usize>() % 10000;
	let res = builder.start(format!("127.0.0.1:{}", port).parse().unwrap()).unwrap();

	(res, port, GuardedAuthCodes {
		authcodes: AuthCodes::from_file(&path).unwrap(),
		path: path,
	})
}

pub fn request(server: Server, request: &str) -> http_client::Response {
	http_client::request(server.addr(), request)
}

#[test]
fn should_reject_invalid_host() {
	// given
	let server = serve().0;

	// when
	let response = request(server,
		"\
			GET / HTTP/1.1\r\n\
			Host: test:8180\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 403 FORBIDDEN".to_owned());
	assert!(response.body.contains("URL Blocked"));
	http_client::assert_security_headers_present(&response.headers, None);
}

#[test]
fn should_allow_home_parity_host() {
	// given
	let server = serve().0;

	// when
	let response = request(server,
		"\
			GET http://home.parity/ HTTP/1.1\r\n\
			Host: home.parity\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
	http_client::assert_security_headers_present(&response.headers, None);
}

#[test]
fn should_serve_styles_even_on_disallowed_domain() {
	// given
	let server = serve().0;

	// when
	let response = request(server,
		"\
			GET /styles.css HTTP/1.1\r\n\
			Host: test:8180\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 200 OK".to_owned());
	http_client::assert_security_headers_present(&response.headers, None);
}

#[test]
fn should_return_200_ok_for_connect_requests() {
	// given
	let server = serve().0;

	// when
	let response = request(server,
		"\
			CONNECT home.parity:8080 HTTP/1.1\r\n\
			Host: home.parity\r\n\
			Connection: close\r\n\
			\r\n\
			{}
		"
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
	assert_eq!(response.status, "HTTP/1.1 403 FORBIDDEN".to_owned());
	http_client::assert_security_headers_present(&response.headers, None);
}

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
			Sec-WebSocket-Protocol: {:?}_{}\r\n\
			Sec-WebSocket-Version: 13\r\n\
			\r\n\
			{{}}
		",
		port,
		format!("{}:{}", code, timestamp).sha3(),
		timestamp,
		)
	);

	// then
	assert_eq!(response.status, "HTTP/1.1 101 Switching Protocols".to_owned());
}

#[test]
fn should_allow_initial_connection_but_only_once() {
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
			Sec-WebSocket-Protocol:{:?}_{}\r\n\
			Sec-WebSocket-Version: 13\r\n\
			\r\n\
			{{}}
		",
		port,
		format!("{}:{}", code, timestamp).sha3(),
		timestamp,
		)
	);
	let response2 = http_client::request(server.addr(),
		&format!("\
			GET / HTTP/1.1\r\n\
			Host: 127.0.0.1:{}\r\n\
			Connection: Close\r\n\
			Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==\r\n\
			Sec-WebSocket-Protocol:{:?}_{}\r\n\
			Sec-WebSocket-Version: 13\r\n\
			\r\n\
			{{}}
		",
		port,
		format!("{}:{}", code, timestamp).sha3(),
		timestamp,
		)
	);


	// then
	assert_eq!(response1.status, "HTTP/1.1 101 Switching Protocols".to_owned());
	assert_eq!(response2.status, "HTTP/1.1 403 FORBIDDEN".to_owned());
	http_client::assert_security_headers_present(&response2.headers, None);
}

