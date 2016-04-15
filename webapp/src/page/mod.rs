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

use std::sync::Arc;
use std::io::Write;
use hyper::uri::RequestUri;
use hyper::server;
use hyper::header;
use hyper::status::StatusCode;
use hyper::net::HttpStream;
use hyper::{Decoder, Encoder, Next};
use endpoint::Endpoint;
use parity_webapp::WebApp;

pub struct PageEndpoint<T : WebApp + 'static> {
	pub app: Arc<T>,
}

impl<T: WebApp + 'static> PageEndpoint<T> {
	pub fn new(app: T) -> Self {
		PageEndpoint {
			app: Arc::new(app)
		}
	}
}

impl<T: WebApp> Endpoint for PageEndpoint<T> {
	fn to_handler(&self, prefix: &str) -> Box<server::Handler<HttpStream>> {
		Box::new(PageHandler {
			app: self.app.clone(),
			prefix: prefix.to_owned(),
			prefix_with_slash: prefix.to_owned() + "/",
			path: None,
			write_pos: 0,
		})
	}
}

struct PageHandler<T: WebApp + 'static> {
	app: Arc<T>,
	prefix: String,
	prefix_with_slash: String,
	path: Option<String>,
	write_pos: usize,
}

impl<T: WebApp + 'static> server::Handler<HttpStream> for PageHandler<T> {
	fn on_request(&mut self, req: server::Request) -> Next {
		if let RequestUri::AbsolutePath(ref path) = *req.uri() {
			// Index file support
			self.path = match path == &self.prefix || path == &self.prefix_with_slash {
				true => Some("index.html".to_owned()),
				false => Some(path[self.prefix_with_slash.len()..].to_owned()),
			};
		}
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		if let Some(f) = self.path.as_ref().and_then(|f| self.app.file(f)) {
			res.set_status(StatusCode::Ok);
			res.headers_mut().set(header::ContentType(f.content_type.parse().unwrap()));
			Next::write()
		} else {
			res.set_status(StatusCode::NotFound);
			Next::write()
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		let (wrote, res) = {
			let file = self.path.as_ref().and_then(|f| self.app.file(f));
			match file {
				None => (None, Next::end()),
				Some(f) if self.write_pos == f.content.len() => (None, Next::end()),
				Some(f) => match encoder.write(&f.content[self.write_pos..]) {
					Ok(bytes) => (Some(bytes), Next::write()),
					Err(e) => match e.kind() {
						::std::io::ErrorKind::WouldBlock => (None, Next::write()),
						_ => (None, Next::end())
					},
				}
			}
		};
		if let Some(bytes) = wrote {
			self.write_pos += bytes;
		}
		res
	}
}
