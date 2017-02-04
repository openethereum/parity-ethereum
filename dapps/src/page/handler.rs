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

use time::{self, Duration};

use hyper::header;
use hyper::server;
use hyper::uri::RequestUri;
use hyper::net::HttpStream;
use hyper::status::StatusCode;
use hyper::{Decoder, Encoder, Next};
use endpoint::EndpointPath;
use handlers::{ContentHandler, add_security_headers};

/// Represents a file that can be sent to client.
/// Implementation should keep track of bytes already sent internally.
pub trait DappFile: Send {
	/// Returns a content-type of this file.
	fn content_type(&self) -> &str;

	/// Checks if all bytes from that file were written.
	fn is_drained(&self) -> bool;

	/// Fetch next chunk to write to the client.
	fn next_chunk(&mut self) -> &[u8];

	/// How many files have been written to the client.
	fn bytes_written(&mut self, bytes: usize);
}

/// Dapp as a (dynamic) set of files.
pub trait Dapp: Send + 'static {
	/// File type
	type DappFile: DappFile;

	/// Returns file under given path.
	fn file(&self, path: &str) -> Option<Self::DappFile>;
}

/// Currently served by `PageHandler` file
pub enum ServedFile<T: Dapp> {
	/// File from dapp
	File(T::DappFile),
	/// Error (404)
	Error(ContentHandler),
}

impl<T: Dapp> ServedFile<T> {
	pub fn new(embeddable_on: Option<(String, u16)>) -> Self {
		ServedFile::Error(ContentHandler::error(
			StatusCode::NotFound,
			"404 Not Found",
			"Requested dapp resource was not found.",
			None,
			embeddable_on,
		))
	}
}

/// Defines what cache headers should be appended to returned resources.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PageCache {
	Enabled,
	Disabled,
}

impl Default for PageCache {
	fn default() -> Self {
		PageCache::Disabled
	}
}

/// A generic type for `PageHandler` allowing to set the URL.
/// Used by dapps fetching to set the URL after the content was downloaded.
pub trait PageHandlerWaiting: server::Handler<HttpStream> + Send {
	fn set_uri(&mut self, uri: &RequestUri);
}

/// A handler for a single webapp.
/// Resolves correct paths and serves as a plumbing code between
/// hyper server and dapp.
pub struct PageHandler<T: Dapp> {
	/// A Dapp.
	pub app: T,
	/// File currently being served
	pub file: ServedFile<T>,
	/// Optional prefix to strip from path.
	pub prefix: Option<String>,
	/// Requested path.
	pub path: EndpointPath,
	/// Flag indicating if the file can be safely embeded (put in iframe).
	pub safe_to_embed_on: Option<(String, u16)>,
	/// Cache settings for this page.
	pub cache: PageCache,
}

impl<T: Dapp> PageHandlerWaiting for PageHandler<T> {
	fn set_uri(&mut self, uri: &RequestUri) {
		trace!(target: "dapps", "Setting URI: {:?}", uri);
		self.file = match *uri {
			RequestUri::AbsolutePath { ref path, .. } => {
				self.app.file(&self.extract_path(path))
			},
			RequestUri::AbsoluteUri(ref url) => {
				self.app.file(&self.extract_path(url.path()))
			},
			_ => None,
		}.map_or_else(|| ServedFile::new(self.safe_to_embed_on.clone()), |f| ServedFile::File(f));
	}
}

impl<T: Dapp> PageHandler<T> {
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

impl<T: Dapp> server::Handler<HttpStream> for PageHandler<T> {
	fn on_request(&mut self, req: server::Request<HttpStream>) -> Next {
		self.set_uri(req.uri());
		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.file {
			ServedFile::File(ref f) => {
				res.set_status(StatusCode::Ok);

				if let PageCache::Enabled = self.cache {
					let mut headers = res.headers_mut();
					let validity = Duration::days(365);
					headers.set(header::CacheControl(vec![
						header::CacheDirective::Public,
						header::CacheDirective::MaxAge(validity.num_seconds() as u32),
					]));
					headers.set(header::Expires(header::HttpDate(time::now() + validity)));
				}

				match f.content_type().parse() {
					Ok(mime) => res.headers_mut().set(header::ContentType(mime)),
					Err(()) => debug!(target: "dapps", "invalid MIME type: {}", f.content_type()),
				}

				// Security headers:
				add_security_headers(&mut res.headers_mut(), self.safe_to_embed_on.clone());
				Next::write()
			},
			ServedFile::Error(ref mut handler) => {
				handler.on_response(res)
			}
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		match self.file {
			ServedFile::Error(ref mut handler) => handler.on_response_writable(encoder),
			ServedFile::File(ref f) if f.is_drained() => Next::end(),
			ServedFile::File(ref mut f) => match encoder.write(f.next_chunk()) {
				Ok(bytes) => {
					f.bytes_written(bytes);
					Next::write()
				},
				Err(e) => match e.kind() {
					::std::io::ErrorKind::WouldBlock => Next::write(),
					_ => Next::end(),
				},
			}
		}
	}
}



#[cfg(test)]
mod test {
	use super::*;

	pub struct TestWebAppFile;

	impl DappFile for TestWebAppFile {
		fn content_type(&self) -> &str {
			unimplemented!()
		}

		fn is_drained(&self) -> bool {
			unimplemented!()
		}

		fn next_chunk(&mut self) -> &[u8] {
			unimplemented!()
		}

		fn bytes_written(&mut self, _bytes: usize) {
			unimplemented!()
		}
	}

	#[derive(Default)]
	pub struct TestWebapp;

	impl Dapp for TestWebapp {
		type DappFile = TestWebAppFile;

		fn file(&self, _path: &str) -> Option<Self::DappFile> {
			None
		}
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
		app: test::TestWebapp,
		prefix: None,
		path: EndpointPath {
			app_id: "app".to_owned(),
			app_params: vec![],
			host: "".to_owned(),
			port: 8080,
			using_dapps_domains: true,
		},
		file: ServedFile::new(None),
		cache: Default::default(),
		safe_to_embed_on: None,
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
