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

//! HTTP Authorization implementations

use std::io::Write;
use std::collections::HashMap;
use hyper::{header, server, Decoder, Encoder, Next};
use hyper::net::HttpStream;
use hyper::status::StatusCode;

/// Authorization result
pub enum Authorized {
	/// Authorization was successful.
	Yes,
	/// Unsuccessful authorization. Handler for further work is returned.
	No(Box<server::Handler<HttpStream>>),
}

/// Authorization interface
pub trait Authorization : Send + Sync {
	/// Checks if authorization is valid.
	fn is_authorized(&self, req: &server::Request)-> Authorized;
}

/// HTTP Basic Authorization handler
pub struct HttpBasicAuth {
	users: HashMap<String, String>,
}

/// No-authorization implementation (authorization disabled)
pub struct NoAuth;

impl Authorization for NoAuth {
	fn is_authorized(&self, _req: &server::Request)-> Authorized {
		Authorized::Yes
	}
}

impl Authorization for HttpBasicAuth {
	fn is_authorized(&self, req: &server::Request) -> Authorized {
		let auth = self.check_auth(&req);

		match auth {
			Access::Denied => {
				Authorized::No(Box::new(UnauthorizedHandler { write_pos: 0 }))
			},
			Access::AuthRequired => {
				Authorized::No(Box::new(AuthRequiredHandler))
			},
			Access::Granted => {
				Authorized::Yes
			},
		}
	}
}

#[derive(Debug)]
enum Access {
	Granted,
	Denied,
	AuthRequired,
}

impl HttpBasicAuth {
	/// Creates `HttpBasicAuth` instance with only one user.
	pub fn single_user(username: &str, password: &str) -> Self {
		let mut users = HashMap::new();
		users.insert(username.to_owned(), password.to_owned());
		HttpBasicAuth {
			users: users
		}
	}

	fn is_authorized(&self, username: &str, password: &str) -> bool {
		self.users.get(&username.to_owned()).map_or(false, |pass| pass == password)
	}

	fn check_auth(&self, req: &server::Request) -> Access {
		match req.headers().get::<header::Authorization<header::Basic>>() {
			Some(&header::Authorization(
				header::Basic { ref username, password: Some(ref password) }
			)) if self.is_authorized(username, password) => Access::Granted,
			Some(_) => Access::Denied,
			None => Access::AuthRequired,
		}
	}
}

pub struct UnauthorizedHandler {
	write_pos: usize,
}

impl server::Handler<HttpStream> for UnauthorizedHandler {
	fn on_request(&mut self, _request: server::Request) -> Next {
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

pub struct AuthRequiredHandler;

impl server::Handler<HttpStream> for AuthRequiredHandler {
	fn on_request(&mut self, _request: server::Request) -> Next {
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
