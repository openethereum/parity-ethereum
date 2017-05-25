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

//! UI redirections

use hyper::{Control, StatusCode};

use endpoint::{Endpoint, Handler, EndpointPath};
use {address, handlers};

/// Redirection to UI server.
pub struct Redirection {
	signer_address: Option<(String, u16)>,
}

impl Redirection {
	pub fn new(
		signer_address: Option<(String, u16)>,
	) -> Self {
		Redirection {
			signer_address: signer_address,
		}
	}
}

impl Endpoint for Redirection {
	fn to_async_handler(&self, _path: EndpointPath, _control: Control) -> Box<Handler> {
		if let Some(ref signer_address) = self.signer_address {
			trace!(target: "dapps", "Redirecting to signer interface.");
			handlers::Redirection::boxed(&format!("http://{}", address(signer_address)))
		} else {
			trace!(target: "dapps", "Signer disabled, returning 404.");
			Box::new(handlers::ContentHandler::error(
				StatusCode::NotFound,
				"404 Not Found",
				"Your homepage is not available when Trusted Signer is disabled.",
				Some("You can still access dapps by writing a correct address, though. Re-enable Signer to get your homepage back."),
				self.signer_address.clone(),
			))
		}
	}
}
