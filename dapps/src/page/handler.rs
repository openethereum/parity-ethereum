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

use std::io;
use std::time::{Duration, SystemTime};
use hyper::{self, header, StatusCode};
use hyper::mime::Mime;

use handlers::{Reader, ContentHandler, add_security_headers};
use {Embeddable};

/// Represents a file that can be sent to client.
/// Implementation should keep track of bytes already sent internally.
pub trait DappFile {
	/// A reader type returned by this file.
	type Reader: io::Read;

	/// Returns a content-type of this file.
	fn content_type(&self) -> &Mime;

	/// Convert this file into io::Read instance.
	fn into_reader(self) -> Self::Reader where Self: Sized;
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

impl<T: DappFile> PageHandler<T> {
	pub fn into_response(self) -> (Option<Reader<T::Reader>>, hyper::Response) {
		let file = match self.file {
			None => return (None, ContentHandler::error(
				StatusCode::NotFound,
				"File not found",
				"Requested file has not been found.",
				None,
				self.safe_to_embed_on,
			).into()),
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

			headers.set(header::ContentType(file.content_type().to_owned()));

			add_security_headers(&mut headers, self.safe_to_embed_on);
		}

		let (reader, body) = Reader::pair(file.into_reader(), Vec::new());
		res.set_body(body);
		(Some(reader), res)
	}
}
