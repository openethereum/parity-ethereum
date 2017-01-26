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

//! HTTP Authorization implementations

use std::collections::HashMap;
use hyper::{server, net, header, status};
use endpoint::Handler;
use handlers::{AuthRequiredHandler, ContentHandler};

/// Authorization result
pub enum Authorized {
	/// Authorization was successful.
	Yes,
	/// Unsuccessful authorization. Handler for further work is returned.
	No(Box<Handler>),
}

/// Authorization interface
pub trait Authorization : Send + Sync {
	/// Checks if authorization is valid.
	fn is_authorized(&self, req: &server::Request<net::HttpStream>)-> Authorized;
}

/// HTTP Basic Authorization handler
pub struct HttpBasicAuth {
	users: HashMap<String, String>,
}

/// No-authorization implementation (authorization disabled)
pub struct NoAuth;

impl Authorization for NoAuth {
	fn is_authorized(&self, _req: &server::Request<net::HttpStream>)-> Authorized {
		Authorized::Yes
	}
}

impl Authorization for HttpBasicAuth {
	fn is_authorized(&self, req: &server::Request<net::HttpStream>) -> Authorized {
		let auth = self.check_auth(&req);

		match auth {
			Access::Denied => {
				Authorized::No(Box::new(ContentHandler::error(
					status::StatusCode::Unauthorized,
					"Unauthorized",
					"You need to provide valid credentials to access this page.",
					None,
					None,
				)))
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

	fn check_auth(&self, req: &server::Request<net::HttpStream>) -> Access {
		match req.headers().get::<header::Authorization<header::Basic>>() {
			Some(&header::Authorization(
				header::Basic { ref username, password: Some(ref password) }
			)) if self.is_authorized(username, password) => Access::Granted,
			Some(_) => Access::Denied,
			None => Access::AuthRequired,
		}
	}
}
