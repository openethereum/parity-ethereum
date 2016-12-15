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
use handlers::{ContentFetcherHandler, ContentHandler};
use hyper::{self, server, net, Next, Encoder, Decoder};

pub struct Web;

impl Web {
	pub fn boxed() -> Box<Endpoint> {
		Box::new(Web)
	}
}

impl Endpoint for Web {
	fn to_async_handler(&self, _path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		Box::new(WebHandler {
			control: control,
			state: State::Initial,
		})
	}
}

enum State {
	Initial,
	Error(ContentHandler),
	Fetching(ContentFetcherHandler),
}

struct WebHandler {
	control: hyper::Control,
	state: State,
}

impl server::Handler<net::HttpStream> for WebHandler {
	fn on_request(&mut self, request: server::Request<net::HttpStream>) -> Next {
		// First extract the URL (reject invalid URLs)
		// Reject non-GET request
		// Do some magic to check if it's a XHR request
		// (maybe some additional Header that has to be sent by the dapp)
		//
		// Spin-up the ContentFetchHandler
		unimplemented!()
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


