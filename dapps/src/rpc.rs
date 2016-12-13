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

use ethcore_rpc::{Metadata, Origin};
use jsonrpc_core::reactor::RpcHandler;
use jsonrpc_http_server::{Rpc, ServerHandler, PanicHandler, AccessControlAllowOrigin, HttpMetaExtractor};
use endpoint::{Endpoint, EndpointPath, Handler};

pub fn rpc(handler: RpcHandler<Metadata>, panic_handler: Arc<Mutex<Option<Box<Fn() -> () + Send>>>>) -> Box<Endpoint> {
	Box::new(RpcEndpoint {
		handler: handler,
		meta_extractor: Arc::new(MetadataExtractor),
		panic_handler: panic_handler,
		cors_domain: None,
		// NOTE [ToDr] We don't need to do any hosts validation here. It's already done in router.
		allowed_hosts: None,
	})
}

struct RpcEndpoint {
	handler: RpcHandler<Metadata>,
	meta_extractor: Arc<HttpMetaExtractor<Metadata>>,
	panic_handler: Arc<Mutex<Option<Box<Fn() -> () + Send>>>>,
	cors_domain: Option<Vec<AccessControlAllowOrigin>>,
	allowed_hosts: Option<Vec<String>>,
}

impl Endpoint for RpcEndpoint {
	fn to_async_handler(&self, _path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		let panic_handler = PanicHandler { handler: self.panic_handler.clone() };
		Box::new(ServerHandler::new(
				Rpc::new(self.handler.clone(), self.meta_extractor.clone()),
				self.cors_domain.clone(),
				self.allowed_hosts.clone(),
				panic_handler,
				control,
		))
	}
}

struct MetadataExtractor;
impl HttpMetaExtractor<Metadata> for MetadataExtractor {
	fn read_metadata(&self, request: &hyper::server::Request<hyper::net::HttpStream>) -> Metadata {
		let dapp_id = request.headers().get::<hyper::header::Referer>()
			.and_then(|referer| hyper::Url::parse(referer).ok())
			.and_then(|url| {
				url.path_segments()
					.and_then(|mut split| split.next())
					.map(|app_id| app_id.to_owned())
			});
		Metadata {
			dapp_id: dapp_id,
			origin: Origin::Dapps,
		}
	}
}
