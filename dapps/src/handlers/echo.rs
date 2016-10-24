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

//! Echo Handler

use std::io::Read;
use hyper::{header, server, Decoder, Encoder, Next};
use hyper::method::Method;
use hyper::net::HttpStream;
use unicase::UniCase;
use super::ContentHandler;

#[derive(Debug, PartialEq)]
/// Type of Cross-Origin request
enum Cors {
	/// Not a Cross-Origin request - no headers needed
	No,
	/// Cross-Origin request with valid Origin
	Allowed(String),
	/// Cross-Origin request with invalid Origin
	Forbidden,
}

pub struct EchoHandler {
	safe_origins: Vec<String>,
	content: String,
	cors: Cors,
	handler: Option<ContentHandler>,
}

impl EchoHandler {

	pub fn cors(safe_origins: Vec<String>) -> Self {
		EchoHandler {
			safe_origins: safe_origins,
			content: String::new(),
			cors: Cors::Forbidden,
			handler: None,
		}
	}

	fn cors_header(&self, origin: Option<String>) -> Cors {
		fn origin_is_allowed(origin: &str, safe_origins: &[String]) -> bool {
			for safe in safe_origins {
				if origin.starts_with(safe) {
					return true;
				}
			}
			false
		}

		match origin {
			Some(ref origin) if origin_is_allowed(origin, &self.safe_origins) => {
				Cors::Allowed(origin.clone())
			},
			None => Cors::No,
			_ => Cors::Forbidden,
		}
	}
}

impl server::Handler<HttpStream> for EchoHandler {
	fn on_request(&mut self, request: server::Request<HttpStream>) -> Next {
		let origin = request.headers().get_raw("origin")
			.and_then(|list| list.get(0))
			.and_then(|origin| String::from_utf8(origin.clone()).ok());

		self.cors = self.cors_header(origin);

		// Don't even read the payload if origin is forbidden!
		if let Cors::Forbidden = self.cors {
			self.handler = Some(ContentHandler::ok(String::new(), mime!(Text/Plain)));
			Next::write()
		} else {
			Next::read()
		}
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
		if let Cors::Allowed(ref domain) = self.cors {
			let mut headers = res.headers_mut();
			headers.set(header::Allow(vec![Method::Options, Method::Post, Method::Get]));
			headers.set(header::AccessControlAllowHeaders(vec![
				UniCase("origin".to_owned()),
				UniCase("content-type".to_owned()),
				UniCase("accept".to_owned()),
			]));
			headers.set(header::AccessControlAllowOrigin::Value(domain.clone()));
		}
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

#[test]
fn should_return_correct_cors_value() {
	// given
	let safe_origins = vec!["chrome-extension://".to_owned(), "http://localhost:8080".to_owned()];
	let cut = EchoHandler {
		safe_origins: safe_origins,
		content: String::new(),
		cors: Cors::No,
		handler: None,
	};

	// when
	let res1 = cut.cors_header(Some("http://ethcore.io".into()));
	let res2 = cut.cors_header(Some("http://localhost:8080".into()));
	let res3 = cut.cors_header(Some("chrome-extension://deadbeefcafe".into()));
	let res4 = cut.cors_header(None);


	// then
	assert_eq!(res1, Cors::Forbidden);
	assert_eq!(res2, Cors::Allowed("http://localhost:8080".into()));
	assert_eq!(res3, Cors::Allowed("chrome-extension://deadbeefcafe".into()));
	assert_eq!(res4, Cors::No);
}
