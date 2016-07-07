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
use hyper::{server, net, Decoder, Encoder, Next};
use api::types::{App, ApiError};
use api::response::{as_json, as_json_error, ping_response};
use handlers::extract_url;
use endpoint::{Endpoint, Endpoints, Handler, EndpointPath};

#[derive(Clone)]
pub struct RestApi {
	local_domain: String,
	endpoints: Arc<Endpoints>,
}

impl RestApi {
	pub fn new(local_domain: String, endpoints: Arc<Endpoints>) -> Box<Endpoint> {
		Box::new(RestApi {
			local_domain: local_domain,
			endpoints: endpoints,
		})
	}

	fn list_apps(&self) -> Vec<App> {
		self.endpoints.iter().filter_map(|(ref k, ref e)| {
			e.info().map(|ref info| App::from_info(k, info))
		}).collect()
	}
}

impl Endpoint for RestApi {
	fn to_handler(&self, _path: EndpointPath) -> Box<Handler> {
		Box::new(RestApiRouter {
			api: self.clone(),
			handler: as_json_error(&ApiError {
				code: "404".into(),
				title: "Not Found".into(),
				detail: "Resource you requested has not been found.".into(),
			}),
		})
	}
}

struct RestApiRouter {
	api: RestApi,
	handler: Box<Handler>,
}

impl server::Handler<net::HttpStream> for RestApiRouter {

	fn on_request(&mut self, request: server::Request<net::HttpStream>) -> Next {
		let url = extract_url(&request);
		if url.is_none() {
			// Just return 404 if we can't parse URL
			return Next::write();
		}

		let url = url.expect("Check for None is above; qed");
		let endpoint = url.path.get(1).map(|v| v.as_str());

		let handler = endpoint.and_then(|v| match v {
			"apps" => Some(as_json(&self.api.list_apps())),
			"ping" => Some(ping_response(&self.api.local_domain)),
			_ => None,
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
