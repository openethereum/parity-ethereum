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

//! HTTP Redirection hyper handler

use hyper::{self, header, StatusCode};

#[derive(Clone)]
pub struct Redirection {
	to_url: String
}

impl Redirection {
	pub fn new<T: Into<String>>(url: T) -> Self {
		Redirection {
			to_url: url.into()
		}
	}
}

impl Into<hyper::Response> for Redirection {
	fn into(self) -> hyper::Response {
		// Don't use `MovedPermanently` here to prevent browser from caching the redirections.
		hyper::Response::new()
			.with_status(StatusCode::Found)
			.with_header(header::Location::new(self.to_url))
	}
}
