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

use std::sync::Arc;
use hyper;

use ethcore_rpc::{Metadata, Origin};
use jsonrpc_core::{Middleware, MetaIoHandler};
use jsonrpc_http_server::{self as http, AccessControlAllowOrigin, HttpMetaExtractor};
use jsonrpc_http_server::tokio_core::reactor::Remote;
use endpoint::{Endpoint, EndpointPath, Handler};

pub fn rpc<T: Middleware<Metadata>>(
	handler: MetaIoHandler<Metadata, T>,
	remote: Remote,
	cors_domains: Vec<AccessControlAllowOrigin>,
) -> Box<Endpoint> {
	Box::new(RpcEndpoint {
		handler: Arc::new(handler),
		remote: remote,
		meta_extractor: Arc::new(MetadataExtractor),
		cors_domain: Some(cors_domains),
		// NOTE [ToDr] We don't need to do any hosts validation here. It's already done in router.
		allowed_hosts: None,
	})
}

struct RpcEndpoint<T: Middleware<Metadata>> {
	handler: Arc<MetaIoHandler<Metadata, T>>,
	remote: Remote,
	meta_extractor: Arc<HttpMetaExtractor<Metadata>>,
	cors_domain: Option<Vec<AccessControlAllowOrigin>>,
	allowed_hosts: Option<Vec<http::Host>>,
}

#[derive(Default)]
struct NoopMiddleware;
impl http::RequestMiddleware for NoopMiddleware {
	fn on_request(&self, _request: &hyper::server::Request<hyper::net::HttpStream>) -> http::RequestMiddlewareAction {
		http::RequestMiddlewareAction::Proceed
	}
}

impl<T: Middleware<Metadata>> Endpoint for RpcEndpoint<T> {
	fn to_async_handler(&self, _path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		Box::new(http::ServerHandler::new(
				http::Rpc {
					handler: self.handler.clone(),
					remote: self.remote.clone(),
					extractor: self.meta_extractor.clone(),
				},
				self.cors_domain.clone(),
				self.allowed_hosts.clone(),
				Arc::new(NoopMiddleware),
				control,
		))
	}
}

struct MetadataExtractor;
impl HttpMetaExtractor<Metadata> for MetadataExtractor {
	fn read_metadata(&self, request: &hyper::server::Request<hyper::net::HttpStream>) -> Metadata {
		let dapp_id = request.headers().get::<hyper::header::Origin>()
			.map(|origin| format!("{}://{}", origin.scheme, origin.host))
			.or_else(|| {
				// fallback to custom header, but only if origin is null
				request.headers().get_raw("origin")
					.and_then(|raw| raw.one())
					.and_then(|raw| if raw == "null".as_bytes() {
						request.headers().get_raw("x-parity-origin")
							.and_then(|raw| raw.one())
							.map(|raw| String::from_utf8_lossy(raw).into_owned())
					} else {
						None
					})
			});
		Metadata {
			origin: Origin::Dapps(dapp_id.map(Into::into).unwrap_or_default()),
		}
	}
}
