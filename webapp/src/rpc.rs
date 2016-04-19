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

use std::sync::Arc;
use hyper::server;
use hyper::net::HttpStream;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerHandler;
use jsonrpc_http_server::AccessControlAllowOrigin;
use endpoint::Endpoint;

pub fn rpc(handler: Arc<IoHandler>) -> Box<Endpoint> {
	Box::new(RpcEndpoint {
		handler: handler,
		cors_domain: Some(AccessControlAllowOrigin::Null)
	})
}

struct RpcEndpoint {
	handler: Arc<IoHandler>,
	cors_domain: Option<AccessControlAllowOrigin>,
}

impl Endpoint for RpcEndpoint {
	fn to_handler(&self, _prefix: &str) -> Box<server::Handler<HttpStream>> {
		Box::new(ServerHandler::new(self.handler.clone(), self.cors_domain.clone()))
	}
}
