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
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use devtools::http_client;
use rpc::ConfirmationsQueue;
use rand;

use ServerBuilder;
use Server;

pub fn serve() -> Server {
	let queue = Arc::new(ConfirmationsQueue::default());
	let builder = ServerBuilder::new(queue, env::temp_dir());
	let port = 35000 + rand::random::<usize>() % 10000;
	let res = builder.start(format!("127.0.0.1:{}", port).parse().unwrap()).unwrap();
	thread::sleep(Duration::from_millis(25));
	res
}

pub fn request(server: Server, request: &str) -> http_client::Response {
	http_client::request(server.addr(), request)
}

#[test]
fn should_reject_invalid_host() {
	// given
	let server = serve();

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
}

#[test]
fn should_serve_styles_even_on_disallowed_domain() {
	// given
	let server = serve();

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
}

