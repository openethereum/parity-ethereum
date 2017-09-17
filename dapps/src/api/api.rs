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

use hyper::{server, net, Decoder, Encoder, Next, Control};
use hyper::method::Method;
use hyper::status::StatusCode;

use api::{response, types};
use apps::fetcher::Fetcher;
use handlers::{self, extract_url};
use endpoint::{Endpoint, Handler, EndpointPath};
use node_health::{NodeHealth, HealthStatus, Health};
use parity_reactor::Remote;

#[derive(Clone)]
pub struct RestApi {
	fetcher: Arc<Fetcher>,
	health: NodeHealth,
	remote: Remote,
}

impl RestApi {
	pub fn new(
		fetcher: Arc<Fetcher>,
		health: NodeHealth,
		remote: Remote,
	) -> Box<Endpoint> {
		Box::new(RestApi {
			fetcher,
			health,
			remote,
		})
	}
}

impl Endpoint for RestApi {
	fn to_async_handler(&self, path: EndpointPath, control: Control) -> Box<Handler> {
		Box::new(RestApiRouter::new((*self).clone(), path, control))
	}
}

struct RestApiRouter {
	api: RestApi,
	path: Option<EndpointPath>,
	control: Option<Control>,
	handler: Box<Handler>,
}

impl RestApiRouter {
	fn new(api: RestApi, path: EndpointPath, control: Control) -> Self {
		RestApiRouter {
			path: Some(path),
			control: Some(control),
			api: api,
			handler: Box::new(response::as_json_error(StatusCode::NotFound, &types::ApiError {
				code: "404".into(),
				title: "Not Found".into(),
				detail: "Resource you requested has not been found.".into(),
			})),
		}
	}

	fn resolve_content(&self, hash: Option<&str>, path: EndpointPath, control: Control) -> Option<Box<Handler>> {
		trace!(target: "dapps", "Resolving content: {:?} from path: {:?}", hash, path);
		match hash {
			Some(hash) if self.api.fetcher.contains(hash) => {
				Some(self.api.fetcher.to_async_handler(path, control))
			},
			_ => None
		}
	}

	fn health(&self, control: Control) -> Box<Handler> {
		let map = move |health: Result<Result<Health, ()>, ()>| {
			let status = match health {
				Ok(Ok(ref health)) => {
					if [&health.peers.status, &health.sync.status].iter().any(|x| *x != &HealthStatus::Ok) {
						StatusCode::PreconditionFailed // HTTP 412
					} else {
						StatusCode::Ok // HTTP 200
					}
				},
				_ => StatusCode::ServiceUnavailable, // HTTP 503
			};

			response::as_json(status, &health)
		};
		let health = self.api.health.health();
		let remote = self.api.remote.clone();
		Box::new(handlers::AsyncHandler::new(health, map, remote, control))
	}
}

impl server::Handler<net::HttpStream> for RestApiRouter {
	fn on_request(&mut self, request: server::Request<net::HttpStream>) -> Next {
		if let Method::Options = *request.method() {
			self.handler = response::empty();
			return Next::write();
		}

		// TODO [ToDr] Consider using `path.app_params` instead
		let url = extract_url(&request);
		if url.is_none() {
			// Just return 404 if we can't parse URL
			return Next::write();
		}

		let url = url.expect("Check for None early-exists above; qed");
		let mut path = self.path.take().expect("on_request called only once, and path is always defined in new; qed");
		let control = self.control.take().expect("on_request called only once, and control is always defined in new; qed");

		let endpoint = url.path.get(1).map(|v| v.as_str());
		let hash = url.path.get(2).map(|v| v.as_str());
		// at this point path.app_id contains 'api', adjust it to the hash properly, otherwise
		// we will try and retrieve 'api' as the hash when doing the /api/content route
		if let Some(ref hash) = hash { path.app_id = hash.clone().to_owned() }

		let handler = endpoint.and_then(|v| match v {
			"ping" => Some(response::ping()),
			"health" => Some(self.health(control)),
			"content" => self.resolve_content(hash, path, control),
			_ => None
		});

		// Overwrite default
		if let Some(h) = handler {
			self.handler = h;
		}

		self.handler.on_request(request)
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<net::HttpStream>) -> Next {
		self.handler.on_request_readable(decoder)
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		self.handler.on_response(res)
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<net::HttpStream>) -> Next {
		self.handler.on_response_writable(encoder)
	}
}
