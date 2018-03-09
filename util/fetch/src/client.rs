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

use futures::future::{self, Loop};
use futures::sync::{mpsc, oneshot};
use futures::{self, Future, Async, Sink, Stream};
use futures_timer::FutureExt;
use hyper::header::{UserAgent, Location, ContentLength, ContentType};
use hyper::mime::Mime;
use hyper::{self, Request, Method, StatusCode};
use hyper_rustls;
use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::{cmp, io, fmt, mem};
use tokio_core::reactor;
use url::{self, Url};

const MAX_SIZE: usize = 64 * 1024 * 1024;
const MAX_SECS: u64 = 5;
const MAX_REDR: usize = 5;

/// A handle to abort requests.
///
/// Requests are either aborted based on reaching thresholds such as
/// maximum response size, timeouts or too many redirects, or else
/// they can be aborted explicitly by the calling code.
#[derive(Clone, Debug)]
pub struct Abort {
	abort: Arc<AtomicBool>,
	size: usize,
	time: Duration,
	redir: usize,
}

impl Default for Abort {
	fn default() -> Abort {
		Abort {
			abort: Arc::new(AtomicBool::new(false)),
			size: MAX_SIZE,
			time: Duration::from_secs(MAX_SECS),
			redir: MAX_REDR
		}
	}
}

impl From<Arc<AtomicBool>> for Abort {
	fn from(a: Arc<AtomicBool>) -> Abort {
		Abort {
			abort: a,
			size: MAX_SIZE,
			time: Duration::from_secs(MAX_SECS),
			redir: MAX_REDR
		}
	}
}

impl Abort {
	/// True if `abort` has been invoked.
	pub fn is_aborted(&self) -> bool {
		self.abort.load(Ordering::SeqCst)
	}

	/// The maximum response body size.
	pub fn max_size(&self) -> usize {
		self.size
	}

	/// The maximum total time, including redirects.
	pub fn max_duration(&self) -> Duration {
		self.time
	}

	/// The maximum number of redirects to allow.
	pub fn max_redirects(&self) -> usize {
		self.redir
	}

	/// Mark as aborted.
	pub fn abort(&self) {
		self.abort.store(true, Ordering::SeqCst)
	}

	/// Set the maximum reponse body size.
	pub fn with_max_size(self, n: usize) -> Abort {
		Abort { size: n, .. self }
	}

	/// Set the maximum duration (including redirects).
	pub fn with_max_duration(self, d: Duration) -> Abort {
		Abort { time: d, .. self }
	}

	/// Set the maximum number of redirects to follow.
	pub fn with_max_redirects(self, n: usize) -> Abort {
		Abort { redir: n, .. self }
	}
}

/// Types which retrieve content from some URL.
pub trait Fetch: Clone + Send + Sync + 'static {
	/// The result future.
	type Result: Future<Item=Response, Error=Error> + Send + 'static;

	/// Get content from some URL.
	fn fetch(&self, url: &str, abort: Abort) -> Self::Result;
}

type TxResponse = oneshot::Sender<Result<Response, Error>>;
type StartupCond = Arc<(Mutex<Result<(), io::Error>>, Condvar)>;
type ChanItem = (Url, Abort, TxResponse);

/// An implementation of `Fetch` using a `hyper` client.
// Due to the `Send` bound of `Fetch` we spawn a background thread for
// actual request/response processing as `hyper::Client` itself does
// not implement `Send` currently.
#[derive(Debug, Clone)]
pub struct Client {
	core: mpsc::Sender<ChanItem>,
}

impl Client {
	/// Create a new fetch client.
	///
	/// This spawns off a background thread for request/response processing.
	pub fn new() -> Result<Self, Error> {
		let startup_done = Arc::new((Mutex::new(Ok(())), Condvar::new()));
		let (tx_proto, rx_proto) = mpsc::channel(64);

		Client::background_thread(startup_done.clone(), rx_proto)?;

		let mut guard = startup_done.0.lock();
		let startup_result = startup_done.1.wait_for(&mut guard, Duration::from_secs(3));

		if startup_result.timed_out() {
			error!(target: "fetch", "timeout starting background thread");
			return Err(Error::BackgroundThreadDead)
		}
		if let Err(e) = mem::replace(&mut *guard, Ok(())) {
			error!(target: "fetch", "error starting background thread: {}", e);
			return Err(e.into())
		}

		Ok(Client { core: tx_proto })
	}

	fn background_thread(start: StartupCond, rx_proto: mpsc::Receiver<ChanItem>) -> io::Result<thread::JoinHandle<()>> {
		thread::Builder::new().name("fetch".into()).spawn(move || {
			let mut core = match reactor::Core::new() {
				Ok(c)  => c,
				Err(e) => {
					*start.0.lock() = Err(e);
					start.1.notify_one();
					return ()
				}
			};
			let handle = core.handle();
			let hyper = hyper::Client::configure()
				.connector(hyper_rustls::HttpsConnector::new(4, &core.handle()))
				.build(&core.handle());

			start.1.notify_one();
			debug!(target: "fetch", "processing requests ...");

			let work = rx_proto.for_each(|(url, abort, sender)| {
				trace!(target: "fetch", "new request to {}", url);
				if abort.is_aborted() {
					return future::ok(sender.send(Err(Error::Aborted)).unwrap_or(()))
				}
				let ini = (hyper.clone(), url, abort, 0);
				let fut = future::loop_fn(ini, |(client, url, abort, redirects)| {
					let url2 = url.clone();
					let abort2 = abort.clone();
					client.request(get(&url))
						.map(move |resp| Response::new(url2, resp, abort2))
						.from_err()
						.and_then(move |resp| {
							if abort.is_aborted() {
								debug!(target: "fetch", "fetch of {} aborted", url);
								return Err(Error::Aborted)
							}
							if let Some(next_url) = redirect_location(url, &resp) {
								if redirects >= abort.max_redirects() {
									return Err(Error::TooManyRedirects)
								}
								Ok(Loop::Continue((client, next_url, abort, redirects + 1)))
							} else {
								let content_len = resp.headers.get::<ContentLength>().cloned();
								if content_len.map(|n| *n > abort.max_size() as u64).unwrap_or(false) {
									return Err(Error::SizeLimit)
								}
								Ok(Loop::Break(resp))
							}
						})
					})
					.then(|result| {
						future::ok(sender.send(result).unwrap_or(()))
					});
				handle.spawn(fut);
				trace!(target: "fetch", "waiting for next request ...");
				future::ok(())
			});
			if let Err(()) = core.run(work) {
				error!(target: "fetch", "error while executing future")
			}
			debug!(target: "fetch", "fetch background thread finished")
		})
	}
}

impl Fetch for Client {
	type Result = Box<Future<Item=Response, Error=Error> + Send>;

	fn fetch(&self, url: &str, abort: Abort) -> Self::Result {
		debug!(target: "fetch", "fetching: {:?}", url);
		if abort.is_aborted() {
			return Box::new(future::err(Error::Aborted))
		}
		let url: Url = match url.parse() {
			Ok(u) => u,
			Err(e) => return Box::new(future::err(e.into()))
		};
		let (tx_res, rx_res) = oneshot::channel();
		let maxdur = abort.max_duration();
		let sender = self.core.clone();
		let future = sender.send((url.clone(), abort, tx_res))
			.map_err(|e| {
				error!(target: "fetch", "failed to schedule request: {}", e);
				Error::BackgroundThreadDead
			})
			.and_then(|_| rx_res.map_err(|oneshot::Canceled| Error::BackgroundThreadDead))
			.and_then(future::result)
			.timeout(maxdur)
			.map_err(|err| {
				if let Error::Io(ref e) = err {
					if let io::ErrorKind::TimedOut = e.kind() {
						return Error::Timeout
					}
				}
				err.into()
			});
		Box::new(future)
	}
}

// Extract redirect location from response.
fn redirect_location(u: Url, r: &Response) -> Option<Url> {
	use hyper::StatusCode::*;
	match r.status() {
		MovedPermanently
		| PermanentRedirect
		| TemporaryRedirect
		| Found
		| SeeOther => {
			if let Some(loc) = r.headers.get::<Location>() {
				u.join(loc).ok()
			} else {
				None
			}
		}
		_ => None
	}
}

// Build a simple GET request for the given Url.
fn get(u: &Url) -> hyper::Request {
    let uri = u.as_ref().parse().expect("Every valid URL is aso a URI.");
	let mut rq = Request::new(Method::Get, uri);
	rq.headers_mut().set(UserAgent::new("Parity Fetch Neo"));
	rq
}

/// An HTTP response.
#[derive(Debug)]
pub struct Response {
	url: Url,
	status: StatusCode,
	headers: hyper::Headers,
	body: hyper::Body,
	abort: Abort,
	nread: usize
}

impl Response {
	/// Create a new response, wrapping a hyper response.
	pub fn new(u: Url, r: hyper::Response, a: Abort) -> Response {
		Response {
			url: u,
			status: r.status(),
			headers: r.headers().clone(),
			body: r.body(),
			abort: a,
			nread: 0
		}
	}

	/// The response status.
	pub fn status(&self) -> StatusCode {
		self.status
	}

	/// Status code == OK (200)?
	pub fn is_success(&self) -> bool {
		self.status() == StatusCode::Ok
	}

	/// Is the content-type text/html?
	pub fn is_html(&self) -> bool {
		if let Some(ref mime) = self.content_type() {
			mime.type_() == "text" && mime.subtype() == "html"
		} else {
			false
		}
	}

	/// The conten-type header value.
	pub fn content_type(&self) -> Option<Mime> {
		self.headers.get::<ContentType>().map(|ct| ct.0.clone())
	}
}

impl Stream for Response {
	type Item = hyper::Chunk;
	type Error = Error;

	fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
		if self.abort.is_aborted() {
			debug!(target: "fetch", "fetch of {} aborted", self.url);
			return Err(Error::Aborted)
		}
		match try_ready!(self.body.poll()) {
			None    => Ok(Async::Ready(None)),
			Some(c) => {
				if self.nread + c.len() > self.abort.max_size() {
					debug!(target: "fetch", "size limit {:?} for {} exceeded", self.abort.max_size(), self.url);
					return Err(Error::SizeLimit)
				}
				self.nread += c.len();
				Ok(Async::Ready(Some(c)))
			}
		}
	}
}

/// `BodyReader` serves as an adapter from async to sync I/O.
///
/// It implements `io::Read` by repedately waiting for the next `Chunk`
/// of hyper's response `Body` which blocks the current thread.
pub struct BodyReader {
	chunk:  hyper::Chunk,
	body:   Option<hyper::Body>,
	offset: usize,
}

impl BodyReader {
	/// Create a new body reader for the given response.
	pub fn new(r: Response) -> BodyReader {
		BodyReader {
			body: Some(r.body),
			chunk: Default::default(),
			offset: 0,
		}
	}
}

impl io::Read for BodyReader {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let mut m = 0;
		while self.body.is_some() {
			// Can we still read from the current chunk?
			if self.offset < self.chunk.len() {
				let k = cmp::min(self.chunk.len() - self.offset, buf.len() - m);
				let c = &self.chunk[self.offset .. self.offset + k];
				(&mut buf[m .. m + k]).copy_from_slice(c);
				self.offset += k;
				m += k;
				if m == buf.len() {
					break
				}
			} else {
				// While in this loop, `self.body` is always defined => wait for the next chunk.
				match self.body.take().unwrap().into_future().wait() {
					Err((e, _))   => {
						error!(target: "fetch", "failed to read chunk: {}", e);
						return Err(io::Error::new(io::ErrorKind::Other, "failed to read body chunk"))
					}
					Ok((None,    _)) => break, // body is exhausted, break out of the loop
					Ok((Some(c), b)) => {
						self.body = Some(b);
						self.chunk = c;
						self.offset = 0
					}
				}
			}
		}
		Ok(m)
	}
}

/// Fetch error cases.
#[derive(Debug)]
pub enum Error {
	/// Hyper gave us an error.
	Hyper(hyper::Error),
	/// Some I/O error occured.
	Io(io::Error),
	/// Invalid URLs where attempted to parse.
	Url(url::ParseError),
	/// Calling code invoked `Abort::abort`.
	Aborted,
	/// Too many redirects have been encountered.
	TooManyRedirects,
	/// The maximum duration was reached.
	Timeout,
	/// The response body is too large.
	SizeLimit,
	/// The background processing thread does not run.
	BackgroundThreadDead,
}

impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Aborted => write!(fmt, "The request has been aborted."),
			Error::Hyper(ref e) => write!(fmt, "{}", e),
			Error::Url(ref e) => write!(fmt, "{}", e),
			Error::Io(ref e) => write!(fmt, "{}", e),
			Error::BackgroundThreadDead => write!(fmt, "background thread gond"),
			Error::TooManyRedirects => write!(fmt, "too many redirects"),
			Error::Timeout => write!(fmt, "request timed out"),
			Error::SizeLimit => write!(fmt, "size limit reached"),
		}
	}
}

impl From<hyper::Error> for Error {
	fn from(e: hyper::Error) -> Self {
		Error::Hyper(e)
	}
}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Self {
		Error::Io(e)
	}
}

impl From<url::ParseError> for Error {
	fn from(e: url::ParseError) -> Self {
		Error::Url(e)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn it_should_fetch() {
		let client = Client::new().unwrap();
		let abort = Abort::default().with_max_duration(Duration::from_secs(10));
		let future = client.fetch("https://httpbin.org/drip?numbytes=3&duration=3&delay=1&code=200", abort);
		let resp = future.wait().unwrap();
		assert!(resp.is_success());
		let body = resp.concat2().wait().unwrap();
		assert_eq!(body.len(), 3)
	}

	#[test]
	fn it_should_timeout() {
		let client = Client::new().unwrap();
		let abort = Abort::default().with_max_duration(Duration::from_secs(5));
		match client.fetch("https://httpbin.org/delay/7", abort).wait() {
			Err(Error::Timeout) => {}
			other => panic!("expected timeout, got {:?}", other)
		}
	}

	#[test]
	fn it_should_follow_redirects() {
		let client = Client::new().unwrap();
		let abort = Abort::default().with_max_redirects(4).with_max_duration(Duration::from_secs(15));
		let future = client.fetch("https://httpbin.org/absolute-redirect/3", abort);
		assert!(future.wait().unwrap().is_success())
	}

	#[test]
	fn it_should_follow_relative_redirects() {
		let client = Client::new().unwrap();
		let abort = Abort::default().with_max_redirects(4).with_max_duration(Duration::from_secs(15));
		let future = client.fetch("https://httpbin.org/relative-redirect/3", abort);
		assert!(future.wait().unwrap().is_success())
	}

	#[test]
	fn it_should_not_follow_too_many_redirects() {
		let client = Client::new().unwrap();
		let abort = Abort::default().with_max_redirects(3);
		match client.fetch("https://httpbin.org/absolute-redirect/4", abort).wait() {
			Err(Error::TooManyRedirects) => {}
			other => panic!("expected too many redirects error, got {:?}", other)
		}
	}

	#[test]
	fn it_should_read_data() {
		let client = Client::new().unwrap();
		let abort = Abort::default();
		let future = client.fetch("https://httpbin.org/bytes/1024", abort);
		let resp = future.wait().unwrap();
		assert!(resp.is_success());
		assert_eq!(resp.concat2().wait().unwrap().len(), 1024)
	}

	#[test]
	fn it_should_not_read_too_much_data() {
		let client = Client::new().unwrap();
		let abort = Abort::default().with_max_size(3);
		match client.fetch("https://httpbin.org/bytes/4", abort).wait() {
			Err(Error::SizeLimit) => {}
			other => panic!("expected size limit error, got {:?}", other)
		}
	}
}
