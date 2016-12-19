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

use std::path::PathBuf;
use endpoint::{Endpoint, Handler, EndpointPath};
use handlers::{ContentFetcherHandler, ContentHandler, ContentValidator, extract_url};
use page::{LocalPageEndpoint, PageCache};
use url::Url;
use hyper::{self, server, net, Next, Encoder, Decoder};
use hyper::status::StatusCode;

pub struct Web;

impl Web {
	pub fn boxed() -> Box<Endpoint> {
		Box::new(Web)
	}
}

impl Endpoint for Web {
	fn to_async_handler(&self, path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		Box::new(WebHandler {
			control: control,
			state: State::Initial,
			path: path,
		})
	}
}

pub struct WebInstaller;

impl ContentValidator for WebInstaller {
	type Error = String;

	fn validate_and_install(&self, path: PathBuf) -> Result<LocalPageEndpoint, Self::Error> {
		Ok(LocalPageEndpoint::single_file(path, "text/html".into(), PageCache::Enabled))
	}
}

enum State {
	Initial,
	Error(ContentHandler),
	Fetching(ContentFetcherHandler<WebInstaller>),
}

struct WebHandler {
	control: hyper::Control,
	state: State,
	path: EndpointPath,
}

impl WebHandler {
	fn extract_target_url(url: Option<Url>) -> Result<String, ContentHandler> {
		let path = match url {
			Some(url) => url.path,
			None => {
				return Err(ContentHandler::error(StatusCode::BadRequest, "Invalid URL", "Couldn't parse URL", None, None));
			}
		};
		println!("Path: {:?}", path);
		let protocol = match path.get(1).map(|a| a.as_str()) {
			Some("http") => "http",
			Some("https") => "https",
			_ => {
				return Err(ContentHandler::error(StatusCode::BadRequest, "Invalid Protocol", "Invalid protocol used", None, None));
			}
		};

		Ok(format!("{}://{}", protocol, path[2..].join("/")))
	}
}

impl server::Handler<net::HttpStream> for WebHandler {
	fn on_request(&mut self, request: server::Request<net::HttpStream>) -> Next {
		let url = extract_url(&request);

		let target_url = match Self::extract_target_url(url) {
			Ok(url) => url,
			Err(error) => {
				self.state = State::Error(error);
				return Next::write();
			}
		};

		let (mut handler, _control) = ContentFetcherHandler::new(
			target_url,
			self.path.clone(),
			self.control.clone(),
			WebInstaller,
			None
		);
		let res = handler.on_request(request);
		self.state = State::Fetching(handler);
		// First extract the URL (reject invalid URLs)
		// Reject non-GET request
		// Do some magic to check if it's a XHR request
		// (maybe some additional Header that has to be sent by the dapp)
		//
		// Spin-up the ContentFetchHandler
		res
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<net::HttpStream>) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_request_readable(decoder),
			State::Fetching(ref mut handler) => handler.on_request_readable(decoder),
		}
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_response(res),
			State::Fetching(ref mut handler) => handler.on_response(res),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<net::HttpStream>) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_response_writable(encoder),
			State::Fetching(ref mut handler) => handler.on_response_writable(encoder),
		}
	}
}


