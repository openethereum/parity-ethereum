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

use std::sync::{Arc, Mutex};
use hyper;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::{ServerHandler, PanicHandler, AccessControlAllowOrigin};
use endpoint::{Endpoint, EndpointPath, Handler};

pub fn rpc(handler: Arc<IoHandler>, panic_handler: Arc<Mutex<Option<Box<Fn() -> () + Send>>>>) -> Box<Endpoint> {
	Box::new(RpcEndpoint {
		handler: handler,
		panic_handler: panic_handler,
		cors_domain: None,
		// NOTE [ToDr] We don't need to do any hosts validation here. It's already done in router.
		allowed_hosts: None,
	})
}

struct RpcEndpoint {
	handler: Arc<IoHandler>,
	panic_handler: Arc<Mutex<Option<Box<Fn() -> () + Send>>>>,
	cors_domain: Option<Vec<AccessControlAllowOrigin>>,
	allowed_hosts: Option<Vec<String>>,
}

impl Endpoint for RpcEndpoint {
	fn to_async_handler(&self, _path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		let panic_handler = PanicHandler { handler: self.panic_handler.clone() };
		Box::new(ServerHandler::new(
				self.handler.clone(),
				self.cors_domain.clone(),
				self.allowed_hosts.clone(),
				panic_handler,
				control,
		))
	}
}
