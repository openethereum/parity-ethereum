// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::io;
use hyper::{self, header, mime, StatusCode};

use handlers::{add_security_headers, Reader};
use Embeddable;

pub struct StreamingHandler<R> {
	initial: Vec<u8>,
	content: R,
	status: StatusCode,
	mimetype: mime::Mime,
	safe_to_embed_on: Embeddable,
}

impl<R: io::Read> StreamingHandler<R> {
	pub fn new(content: R, status: StatusCode, mimetype: mime::Mime, safe_to_embed_on: Embeddable) -> Self {
		StreamingHandler {
			initial: Vec::new(),
			content,
			status,
			mimetype,
			safe_to_embed_on,
		}
	}

	pub fn set_initial_content(&mut self, content: &str) {
		self.initial = content.as_bytes().to_vec();
	}

	pub fn into_response(self) -> (Reader<R>, hyper::Response) {
		let (reader, body) = Reader::pair(self.content, self.initial);
		let mut res = hyper::Response::new()
			.with_status(self.status)
			.with_header(header::ContentType(self.mimetype))
			.with_body(body);
		add_security_headers(&mut res.headers_mut(), self.safe_to_embed_on, false);

		(reader, res)
	}
}
