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

use std::sync::Arc;
use std::io::Write;
use hyper::uri::RequestUri;
use hyper::server;
use hyper::header;
use hyper::status::StatusCode;
use hyper::net::HttpStream;
use hyper::{Decoder, Encoder, Next};
use endpoint::{Endpoint, EndpointInfo, EndpointPath};
use parity_webapp::{WebApp, Info};

pub struct PageEndpoint<T : WebApp + 'static> {
	/// Content of the files
	pub app: Arc<T>,
	/// Prefix to strip from the path (when `None` deducted from `app_id`)
	pub prefix: Option<String>,
}

impl<T: WebApp + 'static> PageEndpoint<T> {
	pub fn new(app: T) -> Self {
		PageEndpoint {
			app: Arc::new(app),
			prefix: None,
		}
	}

	pub fn with_prefix(app: T, prefix: String) -> Self {
		PageEndpoint {
			app: Arc::new(app),
			prefix: Some(prefix),
		}
	}
}

impl<T: WebApp> Endpoint for PageEndpoint<T> {

	fn info(&self) -> Option<EndpointInfo> {
		Some(EndpointInfo::from(self.app.info()))
	}

	fn to_handler(&self, path: EndpointPath) -> Box<server::Handler<HttpStream>> {
		Box::new(PageHandler {
			app: self.app.clone(),
			prefix: self.prefix.clone(),
			path: path,
			file: None,
			write_pos: 0,
		})
	}
}

impl From<Info> for EndpointInfo {
	fn from(info: Info) -> Self {
		EndpointInfo {
			name: info.name,
			description: info.description,
			author: info.author,
			icon_url: info.icon_url,
			version: info.version,
		}
	}
}

struct PageHandler<T: WebApp + 'static> {
	app: Arc<T>,
	prefix: Option<String>,
	path: EndpointPath,
	file: Option<String>,
	write_pos: usize,
}

impl<T: WebApp + 'static> PageHandler<T> {
	fn extract_path(&self, path: &str) -> String {
		let app_id = &self.path.app_id;
		let prefix = "/".to_owned() + self.prefix.as_ref().unwrap_or(app_id);
		let prefix_with_slash = prefix.clone() + "/";
		let query_pos = path.find('?').unwrap_or_else(|| path.len());

		// Index file support
		match path == "/" || path == &prefix || path == &prefix_with_slash {
			true => "index.html".to_owned(),
			false => if path.starts_with(&prefix_with_slash) {
				path[prefix_with_slash.len()..query_pos].to_owned()
			} else if path.starts_with("/") {
				path[1..query_pos].to_owned()
			} else {
				path[0..query_pos].to_owned()
			}
		}
	}
}

impl<T: WebApp + 'static> server::Handler<HttpStream> for PageHandler<T> {
	fn on_request(&mut self, req: server::Request) -> Next {
		self.file = match *req.uri() {
			RequestUri::AbsolutePath(ref path) => {
				Some(self.extract_path(path))
			},
			RequestUri::AbsoluteUri(ref url) => {
				Some(self.extract_path(url.path()))
			},
			_ => None,
		};
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		if let Some(f) = self.file.as_ref().and_then(|f| self.app.file(f)) {
			res.set_status(StatusCode::Ok);
			res.headers_mut().set(header::ContentType(f.content_type.parse().unwrap()));
			Next::write()
		} else {
			res.set_status(StatusCode::NotFound);
			Next::write()
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		let (wrote, res) = {
			let file = self.file.as_ref().and_then(|f| self.app.file(f));
			match file {
				None => (None, Next::end()),
				Some(f) if self.write_pos == f.content.len() => (None, Next::end()),
				Some(f) => match encoder.write(&f.content[self.write_pos..]) {
					Ok(bytes) => (Some(bytes), Next::write()),
					Err(e) => match e.kind() {
						::std::io::ErrorKind::WouldBlock => (None, Next::write()),
						_ => (None, Next::end())
					},
				}
			}
		};
		if let Some(bytes) = wrote {
			self.write_pos += bytes;
		}
		res
	}
}


#[cfg(test)]
use parity_webapp::File;

#[cfg(test)]
#[derive(Default)]
struct TestWebapp;

#[cfg(test)]
impl WebApp for TestWebapp {
	fn file(&self, _path: &str) -> Option<&File> {
		None
	}
	fn info(&self) -> Info {
		unimplemented!()
	}
}

#[test]
fn should_extract_path_with_appid() {
	// given
	let path1 = "/";
	let path2= "/test.css";
	let path3 = "/app/myfile.txt";
	let path4 = "/app/myfile.txt?query=123";
	let page_handler = PageHandler {
		app: Arc::new(TestWebapp),
		prefix: None,
		path: EndpointPath {
			app_id: "app".to_owned(),
			host: "".to_owned(),
			port: 8080
		},
		file: None,
		write_pos: 0,
	};

	// when
	let res1 = page_handler.extract_path(path1);
	let res2 = page_handler.extract_path(path2);
	let res3 = page_handler.extract_path(path3);
	let res4 = page_handler.extract_path(path4);

	// then
	assert_eq!(&res1, "index.html");
	assert_eq!(&res2, "test.css");
	assert_eq!(&res3, "myfile.txt");
	assert_eq!(&res4, "myfile.txt");
}
