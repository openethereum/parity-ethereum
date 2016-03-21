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
use std::default::Default;

struct File {
	path: &'static str,
	content: &'static str,
	content_type: &'static str,
}

pub struct AdminPage {
	index: File,
	css: File,
	js: File,
}

impl Default for AdminPage {
	fn default() -> Self {
		AdminPage {
			index: File { path: "index.html", content_type: "text/html", content: include_str!("./web/index.html") },
			css: File { path: "app.css", content_type: "text/css", content: include_str!("./web/app.css") },
			js: File { path: "app.js", content_type: "application/javascript", content: include_str!("./web/app.js") },
		}
	}
}

impl AdminPage {
	fn serve_file(&self, path: &str, mut res: server::Response) {
		let files = vec![&self.index, &self.css, &self.js];

		for f in files {
			if path.ends_with(f.path) {
				*res.status_mut() = StatusCode::Ok;
				res.headers_mut().set(header::ContentType(f.content_type.parse().unwrap()));
				res.send(f.content.as_bytes()).expect("Error while writing response");
				return;
			}
		}
	}
}

impl server::Handler for AdminPage {
	fn handle(&self, req: server::Request, mut res: server::Response) {
		*res.status_mut() = StatusCode::NotFound;

		if let RequestUri::AbsolutePath(ref path) = req.uri {
			self.serve_file(path, res);
		}
	}
}
