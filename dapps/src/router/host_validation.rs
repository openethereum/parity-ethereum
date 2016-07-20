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


use DAPPS_DOMAIN;
use hyper::server;
use hyper::net::HttpStream;

use jsonrpc_http_server::{is_host_header_valid};
use handlers::ContentHandler;


pub fn is_valid(request: &server::Request<HttpStream>, bind_address: &str, endpoints: Vec<String>) -> bool {
	let mut endpoints = endpoints.into_iter()
		.map(|endpoint| format!("{}{}", endpoint, DAPPS_DOMAIN))
		.collect::<Vec<String>>();
	// Add localhost domain as valid too if listening on loopback interface.
	endpoints.push(bind_address.replace("127.0.0.1", "localhost").into());
	endpoints.push(bind_address.into());

	is_host_header_valid(request, &endpoints)
}

pub fn host_invalid_response() -> Box<server::Handler<HttpStream> + Send> {
	Box::new(ContentHandler::forbidden(
		r#"
		<h1>Request with disallowed <code>Host</code> header has been blocked.</h1>
		<p>Check the URL in your browser address bar.</p>
		"#.into(),
		"text/html".into()
	))
}
