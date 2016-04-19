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

//! HTTP Redirection hyper handler

use std::io::Write;
use hyper::{header, server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use hyper::status::StatusCode;

pub struct Redirection {
	to_url: &'static str
}

impl Redirection {
	pub fn new(url: &'static str) -> Box<Self> {
		Box::new(Redirection {
			to_url: url
		})
	}
}

impl server::Handler<HttpStream> for Redirection {
	fn on_request(&mut self, _request: server::Request) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		res.set_status(StatusCode::MovedPermanently);
		res.headers_mut().set(header::Location(self.to_url.to_owned()));
		Next::end()
	}
	fn on_response_writable(&mut self, _encoder: &mut Encoder<HttpStream>) -> Next {
		Next::end()
	}
}



