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

use hyper::{Method, StatusCode};

use api::response;
use apps::fetcher::Fetcher;
use endpoint::{Endpoint, Request, Response, EndpointPath};
use futures::{future, Future};
use node_health::{NodeHealth, HealthStatus};

#[derive(Clone)]
pub struct RestApi {
	fetcher: Arc<Fetcher>,
	health: NodeHealth,
}

impl Endpoint for RestApi {
	fn respond(&self, mut path: EndpointPath, req: Request) -> Response {
		if let Method::Options = *req.method() {
			return Box::new(future::ok(response::empty()));
		}

		let endpoint = path.app_params.get(0).map(String::to_owned);
		let hash = path.app_params.get(1).map(String::to_owned);

		// at this point path.app_id contains 'api', adjust it to the hash properly, otherwise
		// we will try and retrieve 'api' as the hash when doing the /api/content route
		if let Some(ref hash) = hash {
			path.app_id = hash.to_owned();
		}

		trace!(target: "dapps", "Handling /api request: {:?}/{:?}", endpoint, hash);
		match endpoint.as_ref().map(String::as_str) {
			Some("ping") => Box::new(future::ok(response::ping(req))),
			Some("health") => self.health(),
			Some("content") => self.resolve_content(hash.as_ref().map(String::as_str), path, req),
			_ => Box::new(future::ok(response::not_found())),
		}
	}
}

impl RestApi {
	pub fn new(
		fetcher: Arc<Fetcher>,
		health: NodeHealth,
	) -> Box<Endpoint> {
		Box::new(RestApi {
			fetcher,
			health,
		})
	}

	fn resolve_content(&self, hash: Option<&str>, path: EndpointPath, req: Request) -> Response {
		trace!(target: "dapps", "Resolving content: {:?} from path: {:?}", hash, path);
		match hash {
			Some(hash) if self.fetcher.contains(hash) => {
				self.fetcher.respond(path, req)
			},
			_ => Box::new(future::ok(response::not_found())),
		}
	}

	fn health(&self) -> Response {
		Box::new(self.health.health()
			.then(|health| {
				let status = match health {
					Ok(ref health) => {
						if [&health.peers.status, &health.sync.status].iter().any(|x| *x != &HealthStatus::Ok) {
							StatusCode::PreconditionFailed // HTTP 412
						} else {
							StatusCode::Ok // HTTP 200
						}
					},
					_ => StatusCode::ServiceUnavailable, // HTTP 503
				};

				Ok(response::as_json(status, &health).into())
			})
		)
	}
}
