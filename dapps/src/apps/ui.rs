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

//! UI redirections

use hyper::StatusCode;
use futures::future;

use endpoint::{Endpoint, Request, Response, EndpointPath};
use {handlers, Embeddable};

/// Redirection to UI server.
pub struct Redirection {
	embeddable_on: Embeddable,
}

impl Redirection {
	pub fn new(
		embeddable_on: Embeddable,
	) -> Self {
		Redirection {
			embeddable_on,
		}
	}
}

impl Endpoint for Redirection {
	fn respond(&self, _path: EndpointPath, req: Request) -> Response {
		Box::new(future::ok(if let Some(ref frame) = self.embeddable_on {
			trace!(target: "dapps", "Redirecting to signer interface.");
			let protocol = req.uri().scheme().unwrap_or("http");
			handlers::Redirection::new(format!("{}://{}:{}", protocol, &frame.host, frame.port)).into()
		} else {
			trace!(target: "dapps", "Signer disabled, returning 404.");
			handlers::ContentHandler::error(
				StatusCode::NotFound,
				"404 Not Found",
				"Your homepage is not available when Trusted Signer is disabled.",
				Some("You can still access dapps by writing a correct address, though. Re-enable Signer to get your homepage back."),
				None,
			).into()
		}))
	}
}
