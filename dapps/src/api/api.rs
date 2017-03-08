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
use unicase::UniCase;
use hyper::{server, net, Decoder, Encoder, Next, Control};
use hyper::header;
use hyper::method::Method;
use hyper::header::AccessControlAllowOrigin;

use api::types::{App, ApiError};
use api::response;
use apps::fetcher::Fetcher;

use handlers::extract_url;
use endpoint::{Endpoint, Endpoints, Handler, EndpointPath};
use jsonrpc_http_server::cors;

#[derive(Clone)]
pub struct RestApi {
	cors_domains: Option<Vec<AccessControlAllowOrigin>>,
	endpoints: Arc<Endpoints>,
	fetcher: Arc<Fetcher>,
}

impl RestApi {
	pub fn new(cors_domains: Vec<String>, endpoints: Arc<Endpoints>, fetcher: Arc<Fetcher>) -> Box<Endpoint> {
		Box::new(RestApi {
			cors_domains: Some(cors_domains.into_iter().map(|domain| match domain.as_ref() {
				"all" | "*" | "any" => AccessControlAllowOrigin::Any,
				"null" => AccessControlAllowOrigin::Null,
				other => AccessControlAllowOrigin::Value(other.into()),
			}).collect()),
			endpoints: endpoints,
			fetcher: fetcher,
		})
	}

	fn list_apps(&self) -> Vec<App> {
		self.endpoints.iter().filter_map(|(ref k, ref e)| {
			e.info().map(|ref info| App::from_info(k, info))
		}).collect()
	}
}

impl Endpoint for RestApi {
	fn to_async_handler(&self, path: EndpointPath, control: Control) -> Box<Handler> {
		Box::new(RestApiRouter::new(self.clone(), path, control))
	}
}

struct RestApiRouter {
	api: RestApi,
	origin: Option<String>,
	path: Option<EndpointPath>,
	control: Option<Control>,
	handler: Box<Handler>,
}

impl RestApiRouter {
	fn new(api: RestApi, path: EndpointPath, control: Control) -> Self {
		RestApiRouter {
			path: Some(path),
			origin: None,
			control: Some(control),
			api: api,
			handler: response::as_json_error(&ApiError {
				code: "404".into(),
				title: "Not Found".into(),
				detail: "Resource you requested has not been found.".into(),
			}),
		}
	}

	fn resolve_content(&self, hash: Option<&str>, path: EndpointPath, control: Control) -> Option<Box<Handler>> {
		match hash {
			Some(hash) if self.api.fetcher.contains(hash) => {
				Some(self.api.fetcher.to_async_handler(path, control))
			},
			_ => None
		}
	}

	/// Returns basic headers for a response (it may be overwritten by the handler)
	fn response_headers(&self) -> header::Headers {
		let mut headers = header::Headers::new();
		headers.set(header::AccessControlAllowCredentials);
		headers.set(header::AccessControlAllowMethods(vec![
			Method::Options,
			Method::Post,
			Method::Get,
		]));
		headers.set(header::AccessControlAllowHeaders(vec![
			UniCase("origin".to_owned()),
			UniCase("content-type".to_owned()),
			UniCase("accept".to_owned()),
		]));

		if let Some(cors_header) = cors::get_cors_header(&self.api.cors_domains, &self.origin) {
			headers.set(cors_header);
		}

		headers
	}
}

impl server::Handler<net::HttpStream> for RestApiRouter {

	fn on_request(&mut self, request: server::Request<net::HttpStream>) -> Next {
		self.origin = cors::read_origin(&request);

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
			"apps" => Some(response::as_json(&self.api.list_apps())),
			"ping" => Some(response::ping()),
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
		*res.headers_mut() = self.response_headers();
		self.handler.on_response(res)
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<net::HttpStream>) -> Next {
		self.handler.on_response_writable(encoder)
	}

}
