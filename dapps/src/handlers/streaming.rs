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

//! Content Stream Response

use std::io::{self, Read};

use hyper::{header, server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use hyper::mime::Mime;
use hyper::status::StatusCode;

use handlers::add_security_headers;

const BUFFER_SIZE: usize = 1024;

pub struct StreamingHandler<R: io::Read> {
	buffer: [u8; BUFFER_SIZE],
	buffer_leftover: usize,
	status: StatusCode,
	content: io::BufReader<R>,
	mimetype: Mime,
	safe_to_embed_on: Option<(String, u16)>,
}

impl<R: io::Read> StreamingHandler<R> {
	pub fn new(content: R, status: StatusCode, mimetype: Mime, embeddable_on: Option<(String, u16)>) -> Self {
		StreamingHandler {
			buffer: [0; BUFFER_SIZE],
			buffer_leftover: 0,
			status: status,
			content: io::BufReader::new(content),
			mimetype: mimetype,
			safe_to_embed_on: embeddable_on,
		}
	}

	pub fn set_initial_content(&mut self, content: &str) {
		assert_eq!(self.buffer_leftover, 0);
		let bytes = content.as_bytes();
		self.buffer_leftover = bytes.len();
		self.buffer[0..self.buffer_leftover].copy_from_slice(bytes);
	}
}

impl<R: io::Read> server::Handler<HttpStream> for StreamingHandler<R> {
	fn on_request(&mut self, _request: server::Request<HttpStream>) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		res.set_status(self.status);
		res.headers_mut().set(header::ContentType(self.mimetype.clone()));
		add_security_headers(&mut res.headers_mut(), self.safe_to_embed_on.clone());
		Next::write()
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		fn handle_error(e: io::Error) -> Next {
			match e.kind() {
				::std::io::ErrorKind::WouldBlock => Next::write(),
				_ => Next::end(),
			}
		}

		let write_pos = self.buffer_leftover;
		match self.content.read(&mut self.buffer[write_pos..]) {
			Err(e) => handle_error(e),
			Ok(read) => match encoder.write(&self.buffer[..write_pos + read]) {
				Err(e) => handle_error(e),
				Ok(0) => Next::end(),
				Ok(wrote) => {
					self.buffer_leftover = write_pos + read - wrote;
					if self.buffer_leftover > 0 {
						for i in self.buffer_leftover..write_pos + read {
							self.buffer.swap(i, i - self.buffer_leftover);
						}
					}
					Next::write()
				},
			},
		}
	}
}
