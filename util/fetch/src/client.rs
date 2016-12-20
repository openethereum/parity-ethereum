// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! Fetching

use std::{fs, io};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};

use futures::{self, Future};
use futures_cpupool::{CpuPool, CpuFuture};
use reqwest;
pub use mime::Mime;

#[derive(Default, Debug, Clone)]
pub struct Abort(Arc<AtomicBool>);

impl Abort {
	pub fn is_aborted(&self) -> bool {
		self.0.load(atomic::Ordering::SeqCst)
	}
}

impl From<Arc<AtomicBool>> for Abort {
	fn from(a: Arc<AtomicBool>) -> Self {
		Abort(a)
	}
}

pub trait Fetch: Clone + Send + Sync + 'static {
	type Result: Future<Item=Response, Error=Error> + Send + 'static;
	type FileResult: Future<Item=(PathBuf, Option<Mime>), Error=Error> + Send + 'static;

	/// Fetch URL and get a future for the result.
	fn fetch(&self, url: &str) -> Self::Result;

	/// Fetch URL and get the result synchronously.
	fn fetch_sync(&self, url: &str) -> Result<Response, Error> {
		self.fetch(url).wait()
	}

	fn fetch_to_file(&self, url: &str, path: &Path, abort: Abort) -> Self::FileResult;

	/// Closes this client
	fn close(self) where Self: Sized {}

	/// Random filename
	fn temp_filename() -> PathBuf {
		use ::rand::Rng;
		use ::std::env;

		let mut rng = ::rand::OsRng::new().expect("Reliable random source is required to work.");
		let file: String = rng.gen_ascii_chars().take(12).collect();

		let mut path = env::temp_dir();
		path.push(file);
		path
	}
}

#[derive(Clone)]
pub struct Client {
	client: Arc<reqwest::Client>,
	pool: CpuPool,
	limit: Option<usize>,
}

impl Client {
	pub fn new() -> Result<Self, Error> {
		// Max 15MB will be downloaded.
		Self::with_limit(Some(15*1024*1024))
	}

	fn with_limit(limit: Option<usize>) -> Result<Self, Error> {
		let mut client = try!(reqwest::Client::new());
		client.redirect(reqwest::RedirectPolicy::limited(5));

		Ok(Client {
			client: Arc::new(client),
			pool: CpuPool::new(4),
			limit: limit,
		})
	}

	fn fetch_with_abort(&self, url: &str, abort: Abort) -> CpuFuture<Response, Error> {
		debug!(target: "fetch", "Fetching from: {:?}", url);

		self.pool.spawn(FetchTask {
			url: url.into(),
			client: self.client.clone(),
			limit: self.limit,
			abort: abort,
		})
	}
}

impl Fetch for Client {
	type Result = CpuFuture<Response, Error>;
	type FileResult = CpuFuture<(PathBuf, Option<Mime>), Error>;

	fn fetch(&self, url: &str) -> Self::Result {
		self.fetch_with_abort(url, Default::default())
	}

	fn fetch_to_file(&self, url: &str, path: &Path, abort: Abort) -> Self::FileResult {
		let path = path.to_path_buf();
		trace!(target: "fetch", "Fetching {:?} to file: {:?}", url, path);
		self.pool.spawn(self.fetch_with_abort(url, abort).and_then(move |mut result| {
			trace!(target: "fetch", "Got response: {:?}. Saving.", result);
			let mut file = try!(fs::File::create(&path));
			try!(io::copy(&mut result, &mut file));
			try!(file.flush());

			Ok((path, result.content_type()))
		}))
	}

}

struct FetchTask {
	url: String,
	client: Arc<reqwest::Client>,
	limit: Option<usize>,
	abort: Abort,
}

impl Future for FetchTask {
	// TODO [ToDr] timeouts handling?
	type Item = Response;
	type Error = Error;

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		if self.abort.is_aborted() {
			trace!(target: "fetch", "Fetch of {:?} aborted.", self.url);
			return Err(Error::Aborted);
		}

		trace!(target: "fetch", "Starting fetch task: {:?}", self.url);
		let result = try!(self.client.get(&self.url)
			.header(reqwest::header::UserAgent("Parity Fetch".into()))
			.send());

		Ok(futures::Async::Ready(Response {
			inner: result,
			abort: self.abort.clone(),
			limit: self.limit,
			read: 0,
		}))
	}
}

#[derive(Debug)]
pub enum Error {
	Fetch(reqwest::Error),
	Io(io::Error),
	Aborted,
}

impl From<reqwest::Error> for Error {
	fn from(error: reqwest::Error) -> Self {
		Error::Fetch(error)
	}
}

impl From<io::Error> for Error {
	fn from(error: io::Error) -> Self {
		Error::Io(error)
	}
}

#[derive(Debug)]
pub struct Response {
	inner: reqwest::Response,
	abort: Abort,
	limit: Option<usize>,
	read: usize,
}

impl Response {
	pub fn content_type(&self) -> Option<Mime> {
		let content_type = self.inner.headers().get::<reqwest::header::ContentType>();
		content_type.map(|mime| mime.0.clone())
	}
}

impl io::Read for Response {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if self.abort.is_aborted() {
			return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Fetch aborted."));
		}

		let res = self.inner.read(buf);

		// increase bytes read
		if let Ok(read) = res {
			self.read += read;
		}

		// check limit
		match self.limit {
			Some(limit) if limit < self.read => {
				return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Size limit reached."));
			},
			_ => {},
		}

		res
	}
}
