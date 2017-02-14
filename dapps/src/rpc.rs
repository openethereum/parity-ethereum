// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use jsonrpc_core::{IoHandler, ResponseHandler, Request, Response};
use jsonrpc_http_server::{ServerHandler, PanicHandler, AccessControlAllowOrigin, RpcHandler};
use endpoint::{Endpoint, EndpointPath, Handler};

pub fn rpc(
	handler: Arc<IoHandler>,
	cors_domains: Vec<String>,
	panic_handler: Arc<Mutex<Option<Box<Fn() -> () + Send>>>>,
) -> Box<Endpoint> {
	Box::new(RpcEndpoint {
		handler: Arc::new(RpcMiddleware::new(handler)),
		panic_handler: panic_handler,
		cors_domain: Some(cors_domains.into_iter().map(AccessControlAllowOrigin::Value).collect()),
		// NOTE [ToDr] We don't need to do any hosts validation here. It's already done in router.
		allowed_hosts: None,
	})
}

struct RpcEndpoint {
	handler: Arc<RpcMiddleware>,
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

struct RpcMiddleware {
	handler: Arc<IoHandler>,
	methods: Vec<String>,
}

impl RpcMiddleware {
	fn new(handler: Arc<IoHandler>) -> Self {
		RpcMiddleware {
			handler: handler,
			methods: vec![
				"eth_accounts".into(),
				"eth_coinbase".into(),
				"parity_accountsInfo".into(),
				"parity_defaultAccount".into(),
			],
		}
	}

	/// Appends additional parameter for specific calls.
	fn augment_request(&self, request: &mut Request, meta: Option<Meta>) {
		use jsonrpc_core::{Call, Params, to_value};

		fn augment_call(call: &mut Call, meta: Option<&Meta>, methods: &Vec<String>) {
			match (call, meta) {
				(&mut Call::MethodCall(ref mut method_call), Some(meta)) if methods.contains(&method_call.method) => {
					let session = to_value(&meta.app_id);

					let params = match method_call.params {
						Some(Params::Array(ref vec)) if vec.len() == 0 => Some(Params::Array(vec![session])),
						// invalid params otherwise
						_ => None,
					};

					method_call.params = params;
				},
				_ => {}
			}
		}

		match *request {
			Request::Single(ref mut call) => augment_call(call, meta.as_ref(), &self.methods),
			Request::Batch(ref mut vec) => {
				for mut call in vec {
					augment_call(call, meta.as_ref(), &self.methods)
				}
			},
		}
	}
}

#[derive(Debug)]
struct Meta {
	app_id: String,
}

impl RpcHandler for RpcMiddleware {
	type Metadata = Meta;

	fn read_metadata(&self, request: &hyper::server::Request<hyper::net::HttpStream>) -> Option<Self::Metadata> {
		request.headers().get::<hyper::header::Referer>()
			.and_then(|referer| hyper::Url::parse(referer).ok())
			.and_then(|url| {
				url.path_segments()
					.and_then(|mut split| split.next())
					.map(|app_id| Meta {
						app_id: app_id.to_owned(),
					})
			})
	}

	fn handle_request<H>(&self, request_str: &str, response_handler: H, meta: Option<Self::Metadata>) where
		H: ResponseHandler<Option<String>, Option<String>> + 'static
	{
		let handler = IoHandler::convert_handler(response_handler);
		let request = IoHandler::read_request(request_str);
		trace!(target: "rpc", "Request metadata: {:?}", meta);

		match request {
			Ok(mut request) => {
				self.augment_request(&mut request, meta);
				self.handler.request_handler().handle_request(request, handler, None)
			},
			Err(error) => handler.send(Some(Response::from(error))),
		}
	}
}

