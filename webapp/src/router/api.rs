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
use hyper;
use hyper::status::StatusCode;
use hyper::header;
use hyper::uri::RequestUri::AbsolutePath as Path;
use apps::Pages;

pub struct RestApi {
	pub pages: Arc<Pages>,
}

impl RestApi {
	fn list_pages(&self) -> String {
		let mut s = "[".to_owned();
		for name in self.pages.keys() {
			s.push_str(&format!("\"{}\",", name));
		}
		s.push_str("\"rpc\"");
		s.push_str("]");
		s
	}
}

impl hyper::server::Handler for RestApi {
	fn handle<'b, 'a>(&'a self, req: hyper::server::Request<'a, 'b>, mut res: hyper::server::Response<'a>) {
		match req.uri {
			Path(ref path) if path == "apps" => {
				*res.status_mut() = StatusCode::Ok;
				res.headers_mut().set(header::ContentType("application/json".parse().unwrap()));
				let _ = res.send(self.list_pages().as_bytes());
			}
			_ => (),
		}
	}
}
