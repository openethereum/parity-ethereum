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

use std::collections::HashMap;
use hyper::{header, server};
use hyper::status::StatusCode;

/// Authorization result
pub enum Authorized<'a, 'b> where 'b : 'a {
	/// Authorization was successful. Request and Response are returned for further processing.
	Yes(server::Request<'a, 'b>, server::Response<'a>),
	/// Unsuccessful authorization. Request and Response has been consumed.
	No,
}

/// Authorization interface
pub trait Authorization : Send + Sync {
	/// Handle authorization process and return `Request` and `Response` when authorization is successful.
	fn handle<'b, 'a>(&'a self, req: server::Request<'a, 'b>, res: server::Response<'a>)-> Authorized<'a, 'b>;
}

/// HTTP Basic Authorization handler
pub struct HttpBasicAuth {
	users: HashMap<String, String>,
}

/// No-authorization implementation (authorization disabled)
pub struct NoAuth;

impl Authorization for NoAuth {
	fn handle<'b, 'a>(&'a self, req: server::Request<'a, 'b>, res: server::Response<'a>)-> Authorized<'a, 'b> {
		Authorized::Yes(req, res)
	}
}

impl Authorization for HttpBasicAuth {

	fn handle<'b, 'a>(&'a self, req: server::Request<'a, 'b>, res: server::Response<'a>)-> Authorized<'a, 'b> {
		let auth = self.check_auth(&req);

		match auth {
			Access::Denied => {
				self.respond_with_unauthorized(res);
				Authorized::No
			},
			Access::AuthRequired => {
				self.respond_with_auth_required(res);
				Authorized::No
			},
			Access::Granted => {
				Authorized::Yes(req, res)
			},
		}
	}
}

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
		match req.headers.get::<header::Authorization<header::Basic>>() {
			Some(&header::Authorization(
				header::Basic { ref username, password: Some(ref password) }
			)) if self.is_authorized(username, password) => Access::Granted,
			Some(_) => Access::Denied,
			None => Access::AuthRequired,
		}
	}

	fn respond_with_unauthorized(&self, mut res: server::Response) {
		*res.status_mut() = StatusCode::Unauthorized;
		let _ = res.send(b"Unauthorized");
	}

	fn respond_with_auth_required(&self, mut res: server::Response) {
		*res.status_mut() = StatusCode::Unauthorized;
		res.headers_mut().set_raw("WWW-Authenticate", vec![b"Basic realm=\"Parity\"".to_vec()]);
	}
}

