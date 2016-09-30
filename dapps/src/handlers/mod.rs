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

//! Hyper handlers implementations.

mod auth;
mod echo;
mod content;
mod redirect;
mod fetch;

pub use self::auth::AuthRequiredHandler;
pub use self::echo::EchoHandler;
pub use self::content::ContentHandler;
pub use self::redirect::Redirection;
pub use self::fetch::{ContentFetcherHandler, ContentValidator, FetchControl};

use url::Url;
use hyper::{server, header, net, uri};

pub fn extract_url(req: &server::Request<net::HttpStream>) -> Option<Url> {
	match *req.uri() {
		uri::RequestUri::AbsoluteUri(ref url) => {
			match Url::from_generic_url(url.clone()) {
				Ok(url) => Some(url),
				_ => None,
			}
		},
		uri::RequestUri::AbsolutePath(ref path) => {
			// Attempt to prepend the Host header (mandatory in HTTP/1.1)
			let url_string = match req.headers().get::<header::Host>() {
				Some(ref host) => {
					format!("http://{}:{}{}", host.hostname, host.port.unwrap_or(80), path)
				},
				None => return None,
			};

			match Url::parse(&url_string) {
				Ok(url) => Some(url),
				_ => None,
			}
		},
		_ => None,
	}
}

