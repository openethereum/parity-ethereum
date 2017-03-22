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


use apps::DAPPS_DOMAIN;
use hyper::{server, header, StatusCode};
use hyper::net::HttpStream;

use handlers::ContentHandler;
use jsonrpc_http_server;
use jsonrpc_server_utils::hosts;

pub fn is_valid(req: &server::Request<HttpStream>, allowed_hosts: &Option<Vec<hosts::Host>>) -> bool {
	let header_valid = jsonrpc_http_server::is_host_allowed(req, allowed_hosts);
	match (header_valid, req.headers().get::<header::Host>()) {
		(true, _) => true,
		(_, Some(host)) => host.hostname.ends_with(DAPPS_DOMAIN),
		_ => false,
	}
}

pub fn host_invalid_response() -> Box<server::Handler<HttpStream> + Send> {
	Box::new(ContentHandler::error(StatusCode::Forbidden,
		"Current Host Is Disallowed",
		"You are trying to access your node using incorrect address.",
		Some("Use allowed URL or specify different <code>hosts</code> CLI options."),
		None,
	))
}
