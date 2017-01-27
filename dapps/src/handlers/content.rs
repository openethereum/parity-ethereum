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

//! Simple Content Handler

use hyper::{header, server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use hyper::mime::Mime;
use hyper::status::StatusCode;

use util::version;

use handlers::add_security_headers;

#[derive(Clone)]
pub struct ContentHandler {
	code: StatusCode,
	content: String,
	mimetype: Mime,
	write_pos: usize,
	safe_to_embed_on: Option<(String, u16)>,
}

impl ContentHandler {
	pub fn ok(content: String, mimetype: Mime) -> Self {
		Self::new(StatusCode::Ok, content, mimetype)
	}

	pub fn not_found(content: String, mimetype: Mime) -> Self {
		Self::new(StatusCode::NotFound, content, mimetype)
	}

	pub fn html(code: StatusCode, content: String, embeddable_on: Option<(String, u16)>) -> Self {
		Self::new_embeddable(code, content, mime!(Text/Html), embeddable_on)
	}

	pub fn error(code: StatusCode, title: &str, message: &str, details: Option<&str>, embeddable_on: Option<(String, u16)>) -> Self {
		Self::html(code, format!(
			include_str!("../error_tpl.html"),
			title=title,
			message=message,
			details=details.unwrap_or_else(|| ""),
			version=version(),
		), embeddable_on)
	}

	pub fn new(code: StatusCode, content: String, mimetype: Mime) -> Self {
		Self::new_embeddable(code, content, mimetype, None)
	}

	pub fn new_embeddable(code: StatusCode, content: String, mimetype: Mime, embeddable_on: Option<(String, u16)>) -> Self {
		ContentHandler {
			code: code,
			content: content,
			mimetype: mimetype,
			write_pos: 0,
			safe_to_embed_on: embeddable_on,
		}
	}
}

impl server::Handler<HttpStream> for ContentHandler {
	fn on_request(&mut self, _request: server::Request<HttpStream>) -> Next {
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		res.set_status(self.code);
		res.headers_mut().set(header::ContentType(self.mimetype.clone()));
		add_security_headers(&mut res.headers_mut(), self.safe_to_embed_on.clone());
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
