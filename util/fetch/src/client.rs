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

//! Fetching

use std::{io, fmt, time};
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};

use futures::{self, Future};
use futures_cpupool::{CpuPool, CpuFuture};
use parking_lot::RwLock;
use reqwest;
use reqwest::mime::Mime;

type BoxFuture<A, B> = Box<Future<Item = A, Error = B> + Send>;

/// Fetch abort control
#[derive(Default, Debug, Clone)]
pub struct Abort(Arc<AtomicBool>);

impl Abort {
	/// Returns `true` if request is aborted.
	pub fn is_aborted(&self) -> bool {
		self.0.load(atomic::Ordering::SeqCst)
	}
}

impl From<Arc<AtomicBool>> for Abort {
	fn from(a: Arc<AtomicBool>) -> Self {
		Abort(a)
	}
}

/// Fetch
pub trait Fetch: Clone + Send + Sync + 'static {
	/// Result type
	type Result: Future<Item=Response, Error=Error> + Send + 'static;

	/// Creates new Fetch object.
	fn new() -> Result<Self, Error> where Self: Sized;

	/// Spawn the future in context of this `Fetch` thread pool.
	/// Implementation is optional.
	fn process<F, I, E>(&self, f: F) -> BoxFuture<I, E> where
		F: Future<Item=I, Error=E> + Send + 'static,
		I: Send + 'static,
		E: Send + 'static,
	{
		Box::new(f)
	}

	/// Spawn the future in context of this `Fetch` thread pool as "fire and forget", i.e. dropping this future without
	/// canceling the underlying future.
	/// Implementation is optional.
	fn process_and_forget<F, I, E>(&self, _: F) where
		F: Future<Item=I, Error=E> + Send + 'static,
		I: Send + 'static,
		E: Send + 'static,
	{
		panic!("Attempting to process and forget future on unsupported Fetch.");
	}

	/// Fetch URL and get a future for the result.
	/// Supports aborting the request in the middle of execution.
	fn fetch_with_abort(&self, url: &str, abort: Abort) -> Self::Result;

	/// Fetch URL and get a future for the result.
	fn fetch(&self, url: &str) -> Self::Result {
		self.fetch_with_abort(url, Default::default())
	}

	/// Fetch URL and get the result synchronously.
	fn fetch_sync(&self, url: &str) -> Result<Response, Error> {
		self.fetch(url).wait()
	}

	/// Closes this client
	fn close(self) where Self: Sized {}
}

const CLIENT_TIMEOUT_SECONDS: u64 = 5;

/// Fetch client
pub struct Client {
	client: RwLock<(time::Instant, Arc<reqwest::Client>)>,
	pool: CpuPool,
	limit: Option<usize>,
}

impl Clone for Client {
	fn clone(&self) -> Self {
		let (ref time, ref client) = *self.client.read();
		Client {
			client: RwLock::new((time.clone(), client.clone())),
			pool: self.pool.clone(),
			limit: self.limit.clone(),
		}
	}
}

impl Client {
	fn new_client() -> Result<Arc<reqwest::Client>, Error> {
		let mut client = reqwest::ClientBuilder::new()?;
		client.redirect(reqwest::RedirectPolicy::limited(5));
		Ok(Arc::new(client.build()?))
	}

	fn with_limit(limit: Option<usize>) -> Result<Self, Error> {
		Ok(Client {
			client: RwLock::new((time::Instant::now(), Self::new_client()?)),
			pool: CpuPool::new(4),
			limit: limit,
		})
	}

	fn client(&self) -> Result<Arc<reqwest::Client>, Error> {
		{
			let (ref time, ref client) = *self.client.read();
			if time.elapsed() < time::Duration::from_secs(CLIENT_TIMEOUT_SECONDS) {
				return Ok(client.clone());
			}
		}

		let client = Self::new_client()?;
		*self.client.write() = (time::Instant::now(), client.clone());
		Ok(client)
	}

	/// Returns a handle to underlying CpuPool of this client.
	pub fn pool(&self) -> CpuPool {
		self.pool.clone()
	}
}

impl Fetch for Client {
	type Result = CpuFuture<Response, Error>;

	fn new() -> Result<Self, Error> {
		// Max 50MB will be downloaded.
		Self::with_limit(Some(50*1024*1024))
	}

	fn process<F, I, E>(&self, f: F) -> BoxFuture<I, E> where
		F: Future<Item=I, Error=E> + Send + 'static,
		I: Send + 'static,
		E: Send + 'static,
	{
		Box::new(self.pool.spawn(f))
	}

	fn process_and_forget<F, I, E>(&self, f: F) where
		F: Future<Item=I, Error=E> + Send + 'static,
		I: Send + 'static,
		E: Send + 'static,
	{
		self.pool.spawn(f).forget()
	}

	fn fetch_with_abort(&self, url: &str, abort: Abort) -> Self::Result {
		debug!(target: "fetch", "Fetching from: {:?}", url);

		match self.client() {
			Ok(client) => {
				self.pool.spawn(FetchTask {
					url: url.into(),
					client: client,
					limit: self.limit,
					abort: abort,
				})
			},
			Err(err) => {
				self.pool.spawn(futures::future::err(err))
			},
		}
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
		let result = self.client.get(&self.url)?
						  .header(reqwest::header::UserAgent::new("Parity Fetch"))
						  .send()?;

		Ok(futures::Async::Ready(Response {
			inner: ResponseInner::Response(result),
			abort: self.abort.clone(),
			limit: self.limit,
			read: 0,
		}))
	}
}

/// Fetch Error
#[derive(Debug)]
pub enum Error {
	/// Internal fetch error
	Fetch(reqwest::Error),
	/// Request aborted
	Aborted,
}

impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Aborted => write!(fmt, "The request has been aborted."),
			Error::Fetch(ref err) => write!(fmt, "{}", err),
		}
	}
}

impl From<reqwest::Error> for Error {
	fn from(error: reqwest::Error) -> Self {
		Error::Fetch(error)
	}
}

enum ResponseInner {
	Response(reqwest::Response),
	Reader(Box<io::Read + Send>),
	NotFound,
}

impl fmt::Debug for ResponseInner {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ResponseInner::Response(ref response) => response.fmt(f),
			ResponseInner::NotFound => write!(f, "Not found"),
			ResponseInner::Reader(_) => write!(f, "io Reader"),
		}
	}
}

/// A fetch response type.
#[derive(Debug)]
pub struct Response {
	inner: ResponseInner,
	abort: Abort,
	limit: Option<usize>,
	read: usize,
}

impl Response {
	/// Creates new successfuly response reading from a file.
	pub fn from_reader<R: io::Read + Send + 'static>(reader: R) -> Self {
		Response {
			inner: ResponseInner::Reader(Box::new(reader)),
			abort: Abort::default(),
			limit: None,
			read: 0,
		}
	}

	/// Creates 404 response (useful for tests)
	pub fn not_found() -> Self {
		Response {
			inner: ResponseInner::NotFound,
			abort: Abort::default(),
			limit: None,
			read: 0,
		}
	}

	/// Returns status code of this response.
	pub fn status(&self) -> reqwest::StatusCode {
		match self.inner {
			ResponseInner::Response(ref r) => r.status(),
			ResponseInner::NotFound => reqwest::StatusCode::NotFound,
			_ => reqwest::StatusCode::Ok,
		}
	}

	/// Returns `true` if response status code is successful.
	pub fn is_success(&self) -> bool {
		self.status() == reqwest::StatusCode::Ok
	}

	/// Returns `true` if content type of this response is `text/html`
	pub fn is_html(&self) -> bool {
		match self.content_type() {
			Some(ref mime) if mime.type_() == "text" && mime.subtype() == "html" => true,
			_ => false,
		}
	}

	/// Returns content type of this response (if present)
	pub fn content_type(&self) -> Option<Mime> {
		match self.inner {
			ResponseInner::Response(ref r) => {
				let content_type = r.headers().get::<reqwest::header::ContentType>();
				content_type.map(|mime| mime.0.clone())
			},
			_ => None,
		}
	}
}

impl io::Read for Response {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if self.abort.is_aborted() {
			return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Fetch aborted."));
		}

		let res = match self.inner {
			ResponseInner::Response(ref mut response) => response.read(buf),
			ResponseInner::NotFound => return Ok(0),
			ResponseInner::Reader(ref mut reader) => reader.read(buf),
		};

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
