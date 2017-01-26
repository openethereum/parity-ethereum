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

//! Echo Handler

use std::io::Read;
use hyper::{server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use super::ContentHandler;

#[derive(Default)]
pub struct EchoHandler {
	content: String,
	handler: Option<ContentHandler>,
}

impl server::Handler<HttpStream> for EchoHandler {
	fn on_request(&mut self, _: server::Request<HttpStream>) -> Next {
		Next::read()
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<HttpStream>) -> Next {
		match decoder.read_to_string(&mut self.content) {
			Ok(0) => {
				self.handler = Some(ContentHandler::ok(self.content.clone(), mime!(Application/Json)));
				Next::write()
			},
			Ok(_) => Next::read(),
			Err(e) => match e.kind() {
				::std::io::ErrorKind::WouldBlock => Next::read(),
				_ => Next::end(),
			}
		}
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		self.handler.as_mut()
			.expect("handler always set in on_request, which is before now; qed")
			.on_response(res)
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		self.handler.as_mut()
			.expect("handler always set in on_request, which is before now; qed")
			.on_response_writable(encoder)
	}
}
