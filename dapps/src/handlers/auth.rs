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

//! Authorization Handlers

use std::io::Write;
use hyper::{server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use hyper::status::StatusCode;

pub struct AuthRequiredHandler;

impl server::Handler<HttpStream> for AuthRequiredHandler {
	fn on_request(&mut self, _request: server::Request<HttpStream>) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		res.set_status(StatusCode::Unauthorized);
		res.headers_mut().set_raw("WWW-Authenticate", vec![b"Basic realm=\"Parity\"".to_vec()]);
		Next::write()
	}

	fn on_response_writable(&mut self, _encoder: &mut Encoder<HttpStream>) -> Next {
		Next::end()
	}
}

pub struct UnauthorizedHandler {
	pub write_pos: usize,
}

impl server::Handler<HttpStream> for UnauthorizedHandler {
	fn on_request(&mut self, _request: server::Request<HttpStream>) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		res.set_status(StatusCode::Unauthorized);
		Next::write()
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		let response = "Unauthorized".as_bytes();

		if self.write_pos == response.len() {
			return Next::end();
		}

		match encoder.write(&response[self.write_pos..]) {
			Ok(bytes) => {
				self.write_pos += bytes;
				Next::write()
			},
			Err(e) => match e.kind() {
				::std::io::ErrorKind::WouldBlock => Next::write(),
				_ => Next::end()
			},
		}
	}
}

