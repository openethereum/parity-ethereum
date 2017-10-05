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
use jsonrpc_core::MetaIoHandler;
use http::{self, hyper};

use {HttpServer};
use tests::helpers::Server;
use v1::{extractors, Metadata};

fn serve(handler: Option<MetaIoHandler<Metadata>>) -> Server<HttpServer> {
	let address = "127.0.0.1:0".parse().unwrap();
	let handler = handler.unwrap_or_default();

	Server::new(|remote| ::start_http(
		&address,
		http::DomainsValidation::Disabled,
		http::DomainsValidation::Disabled,
		handler,
		remote,
		extractors::RpcExtractor,
		Some(|request: hyper::Request| {
			http::RequestMiddlewareAction::Proceed {
				should_continue_on_invalid_cors: false,
				request,
			}
		}),
		1,
	).unwrap())
}

/// Test a single request to running server
fn request(server: Server<HttpServer>, request: &str) -> http_client::Response {
	http_client::request(server.server.address(), request)
}

#[cfg(test)]
mod testsing {
	use jsonrpc_core::{MetaIoHandler, Value};
	use v1::Metadata;
	use super::{request, Server};

	fn serve() -> (Server<::HttpServer>, ::std::net::SocketAddr) {
		let mut io = MetaIoHandler::default();
		io.add_method_with_meta("hello", |_, meta: Metadata| {
			Ok(Value::String(format!("{}", meta.origin)))
		});
		let server = super::serve(Some(io));
		let address = server.server.address().to_owned();

		(server, address)
	}

	#[test]
	fn should_extract_rpc_origin() {
		// given
		let (server, address) = serve();

		// when
		let req = r#"{"method":"hello","params":[],"jsonrpc":"2.0","id":1}"#;
		let expected = "34\n{\"jsonrpc\":\"2.0\",\"result\":\"unknown via RPC\",\"id\":1}\n\n0\n\n";
		let res = request(server,
			&format!("\
				POST / HTTP/1.1\r\n\
				Host: {}\r\n\
				Content-Type: application/json\r\n\
				Content-Length: {}\r\n\
				Connection: close\r\n\
				\r\n\
				{}
			", address, req.len(), req)
		);

		// then
		res.assert_status("HTTP/1.1 200 OK");
		assert_eq!(res.body, expected);
	}

	#[test]
	fn should_extract_rpc_origin_with_service() {
		// given
		let (server, address) = serve();

		// when
		let req = r#"{"method":"hello","params":[],"jsonrpc":"2.0","id":1}"#;
		let expected = "38\n{\"jsonrpc\":\"2.0\",\"result\":\"curl/7.16.3 via RPC\",\"id\":1}\n\n0\n\n";
		let res = request(server,
			&format!("\
				POST / HTTP/1.1\r\n\
				Host: {}\r\n\
				Content-Type: application/json\r\n\
				Content-Length: {}\r\n\
				Connection: close\r\n\
				User-Agent: curl/7.16.3\r\n\
				\r\n\
				{}
			", address, req.len(), req)
		);

		// then
		res.assert_status("HTTP/1.1 200 OK");
		assert_eq!(res.body, expected);
	}

	#[test]
	fn should_extract_dapp_origin() {
		// given
		let (server, address) = serve();

		// when
		let req = r#"{"method":"hello","params":[],"jsonrpc":"2.0","id":1}"#;
		let expected = "3A\n{\"jsonrpc\":\"2.0\",\"result\":\"Dapp http://parity.io\",\"id\":1}\n\n0\n\n";
		let res = request(server,
			&format!("\
				POST / HTTP/1.1\r\n\
				Host: {}\r\n\
				Content-Type: application/json\r\n\
				Content-Length: {}\r\n\
				Origin: http://parity.io\r\n\
				Connection: close\r\n\
				User-Agent: curl/7.16.3\r\n\
				\r\n\
				{}
			", address, req.len(), req)
		);

		// then
		res.assert_status("HTTP/1.1 200 OK");
		assert_eq!(res.body, expected);
	}

	#[test]
	fn should_extract_dapp_origin_from_extension() {
		// given
		let (server, address) = serve();

		// when
		let req = r#"{"method":"hello","params":[],"jsonrpc":"2.0","id":1}"#;
		let expected = "44\n{\"jsonrpc\":\"2.0\",\"result\":\"Dapp http://wallet.ethereum.org\",\"id\":1}\n\n0\n\n";
		let res = request(server,
			&format!("\
				POST / HTTP/1.1\r\n\
				Host: {}\r\n\
				Content-Type: application/json\r\n\
				Content-Length: {}\r\n\
				Origin: null\r\n\
				X-Parity-Origin: http://wallet.ethereum.org\r\n\
				Connection: close\r\n\
				User-Agent: curl/7.16.3\r\n\
				\r\n\
				{}
			", address, req.len(), req)
		);

		// then
		res.assert_status("HTTP/1.1 200 OK");
		assert_eq!(res.body, expected);
	}
}
