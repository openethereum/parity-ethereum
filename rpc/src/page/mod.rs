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

use hyper::uri::RequestUri;
use hyper::server;
use hyper::header;
use hyper::status::StatusCode;
use parity_webapp::WebApp;

pub struct Page<T : WebApp> {
	pub app: T
}

impl<T: WebApp> Page<T> {
	fn serve_file(&self, mut path: &str, mut res: server::Response) {
		// Support index file
		if path == "" {
			path = "index.html"
		}
		let file = self.app.file(path);
		if let Some(f) = file {
			*res.status_mut() = StatusCode::Ok;
			res.headers_mut().set(header::ContentType(f.content_type.parse().unwrap()));
			res.send(f.content).expect("Error while writing response");
		}
	}
}

impl<T: WebApp> server::Handler for Page<T> {
	fn handle(&self, req: server::Request, mut res: server::Response) {
		*res.status_mut() = StatusCode::NotFound;

		if let RequestUri::AbsolutePath(ref path) = req.uri {
			self.serve_file(path, res);
		}
	}
}
