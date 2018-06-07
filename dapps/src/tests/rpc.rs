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

use jsonrpc_core::{IoHandler, Value};

use tests::helpers::{serve_with_rpc, request};

#[test]
fn should_serve_rpc() {
	// given
	let mut io = IoHandler::default();
	io.add_method("rpc_test", |_| {
		Ok(Value::String("Hello World!".into()))
	});
	let server = serve_with_rpc(io);

	// when
	let req = r#"{"jsonrpc":"2.0","id":1,"method":"rpc_test","params":[]}"#;
	let response = request(server, &format!(
		"\
			POST /rpc/ HTTP/1.1\r\n\
			Host: 127.0.0.1:8080\r\n\
			Connection: close\r\n\
			Content-Type: application/json\r\n\
			Content-Length: {}\r\n\
			\r\n\
			{}\r\n\
		",
		req.as_bytes().len(),
		req,
	));

	// then
	response.assert_status("HTTP/1.1 200 OK");
	assert_eq!(response.body, "31\n{\"jsonrpc\":\"2.0\",\"result\":\"Hello World!\",\"id\":1}\n\n0\n\n".to_owned());
}
