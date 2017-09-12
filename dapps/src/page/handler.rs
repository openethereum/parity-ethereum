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

use std::thread;
use std::time::{Duration, SystemTime};
use futures::{Future, Sink};
use hyper::{self, header, StatusCode};

use endpoint::EndpointPath;
use handlers::{ContentHandler, add_security_headers};
use {Embeddable};

// TODO [ToDr] This should be a stream!
/// Represents a file that can be sent to client.
/// Implementation should keep track of bytes already sent internally.
pub trait DappFile: Send + 'static {
	// TODO [ToDr] Use Mime?
	/// Returns a content-type of this file.
	fn content_type(&self) -> &str;

	/// Checks if all bytes from that file were written.
	fn is_drained(&self) -> bool;

	/// Fetch next chunk to write to the client.
	fn next_chunk(&mut self) -> &[u8];

	/// How many bytes have been written to the client.
	fn bytes_written(&mut self, bytes: usize);
}

/// Dapp as a (dynamic) set of files.
pub trait Dapp: Send {
	/// File type
	type DappFile: DappFile;

	/// Returns file under given path.
	fn file(&self, path: &str) -> Option<Self::DappFile>;
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

/// A handler for a single webapp.
/// Resolves correct paths and serves as a plumbing code between
/// hyper server and dapp.
pub struct PageHandler<T: DappFile> {
	/// File currently being served
	pub file: Option<T>,
	/// Flag indicating if the file can be safely embeded (put in iframe).
	pub safe_to_embed_on: Embeddable,
	/// Cache settings for this page.
	pub cache: PageCache,
}

impl <T: DappFile> PageHandler<T> {
	pub fn file<D>(app: D, prefix: &Option<String>, path: EndpointPath) -> Option<T> where
		D: Dapp<DappFile = T>,
	{
		let url = "/".to_owned() + &path.app_params.join("/");
		trace!(target: "dapps", "Setting URL: {} ({:?})", url, path);
		app.file(Self::extract_file_path(&url, &path.app_id, prefix))
	}

	fn extract_file_path<'a>(url: &'a str, app_id: &str, prefix: &Option<String>) -> &'a str {
		let prefix_with_slash = format!("/{}/", prefix.as_ref().map(|s| s.as_str()).unwrap_or(app_id));
		let len = prefix_with_slash.len();
		let prefix = &prefix_with_slash[0 .. len - 1];
		let query_pos = url.find('?').unwrap_or_else(|| url.len());

		// Index file support
		match url == "/" || url == prefix || url == &prefix_with_slash {
			true => "index.html",
			false => if url.starts_with(&prefix_with_slash) {
				&url[len .. query_pos]
			} else if url.starts_with("/") {
				&url[1 .. query_pos]
			} else {
				&url[0 .. query_pos]
			}
		}
	}

}

// TODO [ToDr] Consider this async
impl<T: DappFile + 'static> Into<hyper::Response> for PageHandler<T> {
	fn into(self) -> hyper::Response {
		let mut file = match self.file {
			None => return ContentHandler::error(
				StatusCode::NotFound,
				"File not found",
				"Requested file has not been found.",
				None,
				self.safe_to_embed_on,
			).into(),
			Some(file) => file,
		};

		let mut res = hyper::Response::new()
			.with_status(StatusCode::Ok);

		// headers
		{
			let mut headers = res.headers_mut();

			if let PageCache::Enabled = self.cache {
				let validity_secs = 365u32 * 24 * 3600;
				let validity = Duration::from_secs(validity_secs as u64);
				headers.set(header::CacheControl(vec![
					header::CacheDirective::Public,
					header::CacheDirective::MaxAge(validity_secs),
				]));
				headers.set(header::Expires(header::HttpDate::from(SystemTime::now() + validity)));
			}

			match file.content_type().parse() {
				Ok(mime) => headers.set(header::ContentType(mime)),
				Err(_) => {
					warn!(target: "dapps", "invalid MIME type: {}", file.content_type());
					headers.set(header::ContentType::html())
				},
			}

			add_security_headers(&mut headers, self.safe_to_embed_on);
		}

		let (mut sender, body) = hyper::Body::pair();
		res.set_body(body);

		sender = match write_chunk(&mut file, sender) {
			Ok(s) => s,
			Err(_) => return res,
		};

		// TODO [ToDr] do it asynchronously
		if !file.is_drained() {
			thread::spawn(move || {
				let mut sender = sender;
				while !file.is_drained() {
					sender = match write_chunk(&mut file, sender) {
						Ok(s) => s,
						Err(_) => return,
					};
				}
			});
		}

		res
	}
}

fn write_chunk<T, S>(file: &mut T, sender: S) -> Result<S, ()> where
	T: DappFile,
	S: Sink<SinkItem = Result<hyper::Chunk, hyper::Error>>,
	S::SinkError: ::std::fmt::Debug,
{
	let (written, new_sender) = {
		let chunk = file.next_chunk();
		let written = chunk.len();

		(written, match sender.send(Ok(chunk.to_vec().into())).wait() {
			Ok(sender) => sender,
			Err(err) => {
				warn!(target: "dapps", "Cannot stream response: {:?}", err);
				return Err(());
			},
		})
	};

	file.bytes_written(written);
	Ok(new_sender)
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

	#[test]
	fn should_extract_path_with_appid() {

		// given
		let path1 = "/";
		let path2= "/test.css";
		let path3 = "/app/myfile.txt";
		let path4 = "/app/myfile.txt?query=123";

		// when
		let res1 = PageHandler::<TestWebAppFile>::extract_file_path(path1, "app", &None);
		let res2 = PageHandler::<TestWebAppFile>::extract_file_path(path2, "app", &None);
		let res3 = PageHandler::<TestWebAppFile>::extract_file_path(path3, "app", &None);
		let res4 = PageHandler::<TestWebAppFile>::extract_file_path(path4, "app", &None);

		// then
		assert_eq!(res1, "index.html");
		assert_eq!(res2, "test.css");
		assert_eq!(res3, "myfile.txt");
		assert_eq!(res4, "myfile.txt");
	}
}
