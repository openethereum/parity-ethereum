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

//! Simple REST API

use std::io::Write;
use std::sync::Arc;
use hyper::status::StatusCode;
use hyper::{header, server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use endpoint::{Endpoint, Endpoints};

pub struct RestApi {
	endpoints: Arc<Endpoints>,
}

impl RestApi {
	pub fn new(endpoints: Arc<Endpoints>) -> Box<Endpoint> {
		Box::new(RestApi {
			endpoints: endpoints
		})
	}

	fn list_pages(&self) -> String {
		let mut s = "[".to_owned();
		for name in self.endpoints.keys() {
			s.push_str(&format!("\"{}\",", name));
		}
		s.push_str("\"rpc\"");
		s.push_str("]");
		s
	}
}

impl Endpoint for RestApi {
	fn to_handler(&self, _prefix: &str) -> Box<server::Handler<HttpStream>> {
		Box::new(RestApiHandler {
			pages: self.list_pages(),
			write_pos: 0,
		})
	}
}

struct RestApiHandler {
	pages: String,
	write_pos: usize,
}

impl server::Handler<HttpStream> for RestApiHandler {
	fn on_request(&mut self, _request: server::Request) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		res.set_status(StatusCode::Ok);
		res.headers_mut().set(header::ContentType("application/json".parse().unwrap()));
		Next::write()
	}
	
	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		let bytes = self.pages.as_bytes();
		if self.write_pos == bytes.len() {
			return Next::end();
		}

		match encoder.write(&bytes[self.write_pos..]) {
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
