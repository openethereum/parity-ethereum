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

//! URL Endpoint traits

use hyper::status::StatusCode;
use hyper::{header, server, Decoder, Encoder, Next};
use hyper::net::HttpStream;

use std::io::Write;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct EndpointPath {
	pub app_id: String,
	pub host: String,
	pub port: u16,
}

#[derive(Debug, PartialEq)]
pub struct EndpointInfo {
	pub name: String,
	pub description: String,
	pub version: String,
	pub author: String,
	pub icon_url: String,
}

pub trait Endpoint : Send + Sync {
	fn info(&self) -> Option<EndpointInfo> { None }

	fn to_handler(&self, path: EndpointPath) -> Box<server::Handler<HttpStream>>;
}

pub type Endpoints = HashMap<String, Box<Endpoint>>;
pub type Handler = server::Handler<HttpStream>;

pub struct ContentHandler {
	content: String,
	mimetype: String,
	write_pos: usize,
}

impl ContentHandler {
	pub fn new(content: String, mimetype: String) -> Self {
		ContentHandler {
			content: content,
			mimetype: mimetype,
			write_pos: 0
		}
	}
}

impl server::Handler<HttpStream> for ContentHandler {
	fn on_request(&mut self, _request: server::Request) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		res.set_status(StatusCode::Ok);
		res.headers_mut().set(header::ContentType(self.mimetype.parse().unwrap()));
		Next::write()
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		let bytes = self.content.as_bytes();
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
