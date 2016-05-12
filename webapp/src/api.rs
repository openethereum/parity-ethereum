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

//! Simple REST API

use std::sync::Arc;
use endpoint::{Endpoint, Endpoints, ContentHandler, Handler, HostInfo};

pub struct RestApi {
	endpoints: Arc<Endpoints>,
}

impl RestApi {
	pub fn new(endpoints: Arc<Endpoints>) -> Box<Endpoint> {
		Box::new(RestApi {
			endpoints: endpoints
		})
	}

	fn list_pages(&self) -> String {
		let mut s = "[".to_owned();
		for name in self.endpoints.keys() {
			s.push_str(&format!("\"{}\",", name));
		}
		s.push_str("\"rpc\"");
		s.push_str("]");
		s
	}
}

impl Endpoint for RestApi {
	fn to_handler(&self, _prefix: &str, _host: Option<HostInfo>) -> Box<Handler> {
		Box::new(ContentHandler::new(self.list_pages(), "application/json".to_owned()))
	}
}

