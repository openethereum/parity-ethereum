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

//! Serving web-based content (proxying)

use endpoint::{Endpoint, Handler, EndpointPath};
use handlers::{ContentFetcherHandler, ContentHandler, ContentValidator, Redirection, extract_url};
use page::{LocalPageEndpoint};
use fetch::{self, Fetch};
use url::Url;
use hyper::{self, server, net, Next, Encoder, Decoder};
use hyper::status::StatusCode;
use parity_reactor::Remote;
use apps::WEB_PATH;

pub struct Web<F> {
	remote: Remote,
	fetch: F,
}

impl<F: Fetch> Web<F> {
	pub fn boxed(remote: Remote, fetch: F) -> Box<Endpoint> {
		Box::new(Web {
			remote: remote,
			fetch: fetch,
		})
	}
}

impl<F: Fetch> Endpoint for Web<F> {
	fn to_async_handler(&self, path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		Box::new(WebHandler {
			control: control,
			state: State::Initial,
			path: path,
			remote: self.remote.clone(),
			fetch: self.fetch.clone(),
		})
	}
}

pub struct WebInstaller;

impl ContentValidator for WebInstaller {
	type Error = String;

	fn validate_and_install(&self, _response: fetch::Response) -> Result<LocalPageEndpoint, String> {
		// let path = unimplemented!();
		// let mime = response.content_type().unwrap_or(mime!(Text/Html));
		// Ok(LocalPageEndpoint::single_file(path, mime, PageCache::Enabled))
		Err("unimplemented".into())
	}
}

enum State<F: Fetch> {
	Initial,
	Error(ContentHandler),
	Redirecting(Redirection),
	Fetching(ContentFetcherHandler<WebInstaller, F>),
}

struct WebHandler<F: Fetch> {
	control: hyper::Control,
	state: State<F>,
	path: EndpointPath,
	remote: Remote,
	fetch: F,
}

impl<F: Fetch> WebHandler<F> {
	fn extract_target_url(url: Option<Url>) -> Result<String, State<F>> {
		let path = match url {
			Some(url) => url.path,
			None => {
				return Err(State::Error(
					ContentHandler::error(StatusCode::BadRequest, "Invalid URL", "Couldn't parse URL", None, None)
				));
			}
		};

		// TODO [ToDr] Check if token supplied in URL is correct.

		// Support domain based routing.
		let idx = match path.get(0).map(|m| m.as_ref()) {
			Some(WEB_PATH) => 1,
			_ => 0,
		};

		// Validate protocol
		let protocol = match path.get(idx).map(|a| a.as_str()) {
			Some("http") => "http",
			Some("https") => "https",
			_ => {
				return Err(State::Error(
					ContentHandler::error(StatusCode::BadRequest, "Invalid Protocol", "Invalid protocol used", None, None)
				));
			}
		};

		// Redirect if address to main page does not end with /
		if let None = path.get(idx + 2) {
			return Err(State::Redirecting(
				Redirection::new(&format!("/{}/", path.join("/")))
			));
		}

		Ok(format!("{}://{}", protocol, path[2..].join("/")))
	}
}

impl<F: Fetch> server::Handler<net::HttpStream> for WebHandler<F> {
	fn on_request(&mut self, request: server::Request<net::HttpStream>) -> Next {
		let url = extract_url(&request);

		// First extract the URL (reject invalid URLs)
		let target_url = match Self::extract_target_url(url) {
			Ok(url) => url,
			Err(error) => {
				self.state = error;
				return Next::write();
			}
		};

		let (mut handler, _control) = ContentFetcherHandler::new(
			target_url,
			self.path.clone(),
			self.control.clone(),
			WebInstaller,
			None,
			self.remote.clone(),
			self.fetch.clone(),
		);
		let res = handler.on_request(request);
		self.state = State::Fetching(handler);

		res
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<net::HttpStream>) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_request_readable(decoder),
			State::Redirecting(ref mut handler) => handler.on_request_readable(decoder),
			State::Fetching(ref mut handler) => handler.on_request_readable(decoder),
		}
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_response(res),
			State::Redirecting(ref mut handler) => handler.on_response(res),
			State::Fetching(ref mut handler) => handler.on_response(res),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<net::HttpStream>) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_response_writable(encoder),
			State::Redirecting(ref mut handler) => handler.on_response_writable(encoder),
			State::Fetching(ref mut handler) => handler.on_response_writable(encoder),
		}
	}
}


