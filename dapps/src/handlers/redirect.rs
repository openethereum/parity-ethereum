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

//! HTTP Redirection hyper handler

use hyper::{header, server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use hyper::status::StatusCode;

#[derive(Clone)]
pub struct Redirection {
	to_url: String
}

impl Redirection {
	pub fn new(url: &str) -> Self {
		Redirection {
			to_url: url.to_owned()
		}
	}

	pub fn boxed(url: &str) -> Box<Self> {
		Box::new(Self::new(url))
	}
}

impl server::Handler<HttpStream> for Redirection {
	fn on_request(&mut self, _request: server::Request<HttpStream>) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		// Don't use `MovedPermanently` here to prevent browser from caching the redirections.
		res.set_status(StatusCode::Found);
		res.headers_mut().set(header::Location(self.to_url.to_owned()));
		Next::write()
	}
	fn on_response_writable(&mut self, _encoder: &mut Encoder<HttpStream>) -> Next {
		Next::end()
	}
}



