// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
use hyper::header::{UserAgent, Location, ContentLength, ContentType};
use hyper::mime::Mime;
use hyper::{self, Method, StatusCode};
use hyper_rustls;
use std;
use std::cmp::min;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;
use std::{io, fmt};
use tokio_core::reactor;
use tokio_timer::{self, Timer};
use url::{self, Url};
use bytes::Bytes;

const MAX_SIZE: usize = 64 * 1024 * 1024;
const MAX_SECS: Duration = Duration::from_secs(5);
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
			time: MAX_SECS,
			redir: MAX_REDR,
		}
	}
}

impl From<Arc<AtomicBool>> for Abort {
	fn from(a: Arc<AtomicBool>) -> Abort {
		Abort {
			abort: a,
			size: MAX_SIZE,
			time: MAX_SECS,
			redir: MAX_REDR,
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

	/// Make a request to given URL
	fn fetch(&self, request: Request, abort: Abort) -> Self::Result;

	/// Get content from some URL.
	fn get(&self, url: &str, abort: Abort) -> Self::Result;

	/// Post content to some URL.
	fn post(&self, url: &str, abort: Abort) -> Self::Result;
}

type TxResponse = oneshot::Sender<Result<Response, Error>>;
type TxStartup = std::sync::mpsc::SyncSender<Result<(), io::Error>>;
type ChanItem = Option<(Request, Abort, TxResponse)>;

/// An implementation of `Fetch` using a `hyper` client.
// Due to the `Send` bound of `Fetch` we spawn a background thread for
// actual request/response processing as `hyper::Client` itself does
// not implement `Send` currently.
#[derive(Debug)]
pub struct Client {
	core: mpsc::Sender<ChanItem>,
	refs: Arc<AtomicUsize>,
	timer: Timer,
}

// When cloning a client we increment the internal reference counter.
impl Clone for Client {
	fn clone(&self) -> Client {
		self.refs.fetch_add(1, Ordering::SeqCst);
		Client {
			core: self.core.clone(),
			refs: self.refs.clone(),
			timer: self.timer.clone(),
		}
	}
}

// When dropping a client, we decrement the reference counter.
// Once it reaches 0 we terminate the background thread.
impl Drop for Client {
	fn drop(&mut self) {
		if self.refs.fetch_sub(1, Ordering::SeqCst) == 1 {
			// ignore send error as it means the background thread is gone already
			let _ = self.core.clone().send(None).wait();
		}
	}
}

impl Client {
	/// Create a new fetch client.
	pub fn new(num_dns_threads: usize) -> Result<Self, Error> {
		let (tx_start, rx_start) = std::sync::mpsc::sync_channel(1);
		let (tx_proto, rx_proto) = mpsc::channel(64);

		Client::background_thread(tx_start, rx_proto, num_dns_threads)?;

		match rx_start.recv_timeout(Duration::from_secs(10)) {
			Err(RecvTimeoutError::Timeout) => {
				error!(target: "fetch", "timeout starting background thread");
				return Err(Error::BackgroundThreadDead)
			}
			Err(RecvTimeoutError::Disconnected) => {
				error!(target: "fetch", "background thread gone");
				return Err(Error::BackgroundThreadDead)
			}
			Ok(Err(e)) => {
				error!(target: "fetch", "error starting background thread: {}", e);
				return Err(e.into())
			}
			Ok(Ok(())) => {}
		}

		Ok(Client {
			core: tx_proto,
			refs: Arc::new(AtomicUsize::new(1)),
			timer: Timer::default(),
		})
	}

	fn background_thread(tx_start: TxStartup, rx_proto: mpsc::Receiver<ChanItem>, num_dns_threads: usize) -> io::Result<thread::JoinHandle<()>> {
		thread::Builder::new().name("fetch".into()).spawn(move || {
			let mut core = match reactor::Core::new() {
				Ok(c) => c,
				Err(e) => return tx_start.send(Err(e)).unwrap_or(())
			};

			let handle = core.handle();
			let hyper = hyper::Client::configure()
				.connector(hyper_rustls::HttpsConnector::new(num_dns_threads, &core.handle()))
				.build(&core.handle());

			let future = rx_proto.take_while(|item| Ok(item.is_some()))
				.map(|item| item.expect("`take_while` is only passing on channel items != None; qed"))
				.for_each(|(request, abort, sender)|
			{
				trace!(target: "fetch", "new request to {}", request.url());
				if abort.is_aborted() {
					return future::ok(sender.send(Err(Error::Aborted)).unwrap_or(()))
				}
				let ini = (hyper.clone(), request, abort, 0);
				let fut = future::loop_fn(ini, |(client, request, abort, redirects)| {
					let request2 = request.clone();
					let url2 = request2.url().clone();
					let abort2 = abort.clone();
					client.request(request.into())
						.map(move |resp| Response::new(url2, resp, abort2))
						.from_err()
						.and_then(move |resp| {
							if abort.is_aborted() {
								debug!(target: "fetch", "fetch of {} aborted", request2.url());
								return Err(Error::Aborted)
							}
							if let Some((next_url, preserve_method)) = redirect_location(request2.url().clone(), &resp) {
								if redirects >= abort.max_redirects() {
									return Err(Error::TooManyRedirects)
								}
								let request = if preserve_method {
									let mut request2 = request2.clone();
									request2.set_url(next_url);
									request2
								} else {
									Request::new(next_url, Method::Get)
								};
								Ok(Loop::Continue((client, request, abort, redirects + 1)))
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

			tx_start.send(Ok(())).unwrap_or(());

			debug!(target: "fetch", "processing requests ...");
			if let Err(()) = core.run(future) {
				error!(target: "fetch", "error while executing future")
			}
			debug!(target: "fetch", "fetch background thread finished")
		})
	}
}

impl Fetch for Client {
	type Result = Box<Future<Item=Response, Error=Error> + Send>;

	fn fetch(&self, request: Request, abort: Abort) -> Self::Result {
		debug!(target: "fetch", "fetching: {:?}", request.url());
		if abort.is_aborted() {
			return Box::new(future::err(Error::Aborted))
		}
		let (tx_res, rx_res) = oneshot::channel();
		let maxdur = abort.max_duration();
		let sender = self.core.clone();
		let future = sender.send(Some((request, abort, tx_res)))
			.map_err(|e| {
				error!(target: "fetch", "failed to schedule request: {}", e);
				Error::BackgroundThreadDead
			})
			.and_then(|_| rx_res.map_err(|oneshot::Canceled| Error::BackgroundThreadDead))
			.and_then(future::result);

		Box::new(self.timer.timeout(future, maxdur))
	}

	/// Get content from some URL.
	fn get(&self, url: &str, abort: Abort) -> Self::Result {
		let url: Url = match url.parse() {
			Ok(u) => u,
			Err(e) => return Box::new(future::err(e.into()))
		};
		self.fetch(Request::get(url), abort)
	}

	/// Post content to some URL.
	fn post(&self, url: &str, abort: Abort) -> Self::Result {
		let url: Url = match url.parse() {
			Ok(u) => u,
			Err(e) => return Box::new(future::err(e.into()))
		};
		self.fetch(Request::post(url), abort)
	}
}

// Extract redirect location from response. The second return value indicate whether the original method should be preserved.
fn redirect_location(u: Url, r: &Response) -> Option<(Url, bool)> {
	use hyper::StatusCode::*;
	let preserve_method = match r.status() {
		TemporaryRedirect | PermanentRedirect => true,
		_ => false,
	};
	match r.status() {
		MovedPermanently
		| PermanentRedirect
		| TemporaryRedirect
		| Found
		| SeeOther => {
			if let Some(loc) = r.headers.get::<Location>() {
				u.join(loc).ok().map(|url| (url, preserve_method))
			} else {
				None
			}
		}
		_ => None
	}
}

/// A wrapper for hyper::Request using Url and with methods.
#[derive(Debug, Clone)]
pub struct Request {
	url: Url,
	method: Method,
	headers: hyper::Headers,
	body: Bytes,
}

impl Request {
	/// Create a new request, with given url and method.
	pub fn new(url: Url, method: Method) -> Request {
		Request {
			url, method,
			headers: hyper::Headers::new(),
			body: Default::default(),
		}
	}

	/// Create a new GET request.
	pub fn get(url: Url) -> Request {
		Request::new(url, Method::Get)
	}

	/// Create a new empty POST request.
	pub fn post(url: Url) -> Request {
		Request::new(url, Method::Post)
	}

	/// Read the url.
	pub fn url(&self) -> &Url {
		&self.url
	}

	/// Read the request headers.
	pub fn headers(&self) -> &hyper::Headers {
		&self.headers
	}

	/// Get a mutable reference to the headers.
	pub fn headers_mut(&mut self) -> &mut hyper::Headers {
		&mut self.headers
	}

	/// Set the body of the request.
	pub fn set_body<T: Into<Bytes>>(&mut self, body: T) {
		self.body = body.into();
	}

	/// Set the url of the request.
	pub fn set_url(&mut self, url: Url) {
		self.url = url;
	}

	/// Consume self, and return it with the added given header.
	pub fn with_header<H: hyper::header::Header>(mut self, value: H) -> Self {
		self.headers_mut().set(value);
		self
	}

	/// Consume self, and return it with the body.
	pub fn with_body<T: Into<Bytes>>(mut self, body: T) -> Self {
		self.set_body(body);
		self
	}
}

impl Into<hyper::Request> for Request {
	fn into(mut self) -> hyper::Request {
		let uri = self.url.as_ref().parse().expect("Every valid URLis also a URI.");
		let mut req = hyper::Request::new(self.method, uri);

		self.headers.set(UserAgent::new("Parity Fetch Neo"));
		*req.headers_mut() = self.headers;
		req.set_body(self.body);

		req
	}
}

/// An HTTP response.
#[derive(Debug)]
pub struct Response {
	url: Url,
	status: StatusCode,
	headers: hyper::Headers,
	body: hyper::Body,
	abort: Abort,
	nread: usize,
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
			nread: 0,
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

	/// Status code == 404.
	pub fn is_not_found(&self) -> bool {
		self.status() == StatusCode::NotFound
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
			None => Ok(Async::Ready(None)),
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
	chunk: hyper::Chunk,
	body: Option<hyper::Body>,
	abort: Abort,
	offset: usize,
	count: usize,
}

impl BodyReader {
	/// Create a new body reader for the given response.
	pub fn new(r: Response) -> BodyReader {
		BodyReader {
			body: Some(r.body),
			chunk: Default::default(),
			abort: r.abort,
			offset: 0,
			count: 0,
		}
	}
}

impl io::Read for BodyReader {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let mut n = 0;
		while self.body.is_some() {
			// Can we still read from the current chunk?
			if self.offset < self.chunk.len() {
				let k = min(self.chunk.len() - self.offset, buf.len() - n);
				if self.count + k > self.abort.max_size() {
					debug!(target: "fetch", "size limit {:?} exceeded", self.abort.max_size());
					return Err(io::Error::new(io::ErrorKind::PermissionDenied, "size limit exceeded"))
				}
				let c = &self.chunk[self.offset .. self.offset + k];
				(&mut buf[n .. n + k]).copy_from_slice(c);
				self.offset += k;
				self.count += k;
				n += k;
				if n == buf.len() {
					break
				}
			} else {
				let body = self.body.take().expect("loop condition ensures `self.body` is always defined; qed");
				match body.into_future().wait() { // wait for next chunk
					Err((e, _)) => {
						error!(target: "fetch", "failed to read chunk: {}", e);
						return Err(io::Error::new(io::ErrorKind::Other, "failed to read body chunk"))
					}
					Ok((None, _)) => break, // body is exhausted, break out of the loop
					Ok((Some(c), b)) => {
						self.body = Some(b);
						self.chunk = c;
						self.offset = 0
					}
				}
			}
		}
		Ok(n)
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
	/// tokio-timer gave us an error.
	Timer(tokio_timer::TimerError),
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
			Error::Timer(ref e) => write!(fmt, "{}", e),
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

impl<F> From<tokio_timer::TimeoutError<F>> for Error {
	fn from(e: tokio_timer::TimeoutError<F>) -> Self {
		match e {
			tokio_timer::TimeoutError::Timer(_, e) => Error::Timer(e),
			tokio_timer::TimeoutError::TimedOut(_) => Error::Timeout,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures::future;
	use futures::sync::mpsc;
	use hyper::StatusCode;
	use hyper::server::{Http, Request, Response, Service};
	use tokio_timer::Timer;
	use std;
	use std::io::Read;
	use std::net::SocketAddr;

	const ADDRESS: &str = "127.0.0.1:0";

	#[test]
	fn it_should_fetch() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let future = client.get(&format!("http://{}?123", server.addr()), Default::default());
		let resp = future.wait().unwrap();
		assert!(resp.is_success());
		let body = resp.concat2().wait().unwrap();
		assert_eq!(&body[..], b"123")
	}

	#[test]
	fn it_should_fetch_in_light_mode() {
		let server = TestServer::run();
		let client = Client::new(1).unwrap();
		let future = client.get(&format!("http://{}?123", server.addr()), Default::default());
		let resp = future.wait().unwrap();
		assert!(resp.is_success());
		let body = resp.concat2().wait().unwrap();
		assert_eq!(&body[..], b"123")
	}

	#[test]
	fn it_should_timeout() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let abort = Abort::default().with_max_duration(Duration::from_secs(1));
		match client.get(&format!("http://{}/delay?3", server.addr()), abort).wait() {
			Err(Error::Timeout) => {}
			other => panic!("expected timeout, got {:?}", other)
		}
	}

	#[test]
	fn it_should_follow_redirects() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let abort = Abort::default();
		let future = client.get(&format!("http://{}/redirect?http://{}/", server.addr(), server.addr()), abort);
		assert!(future.wait().unwrap().is_success())
	}

	#[test]
	fn it_should_follow_relative_redirects() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let abort = Abort::default().with_max_redirects(4);
		let future = client.get(&format!("http://{}/redirect?/", server.addr()), abort);
		assert!(future.wait().unwrap().is_success())
	}

	#[test]
	fn it_should_not_follow_too_many_redirects() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let abort = Abort::default().with_max_redirects(3);
		match client.get(&format!("http://{}/loop", server.addr()), abort).wait() {
			Err(Error::TooManyRedirects) => {}
			other => panic!("expected too many redirects error, got {:?}", other)
		}
	}

	#[test]
	fn it_should_read_data() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let abort = Abort::default();
		let future = client.get(&format!("http://{}?abcdefghijklmnopqrstuvwxyz", server.addr()), abort);
		let resp = future.wait().unwrap();
		assert!(resp.is_success());
		assert_eq!(&resp.concat2().wait().unwrap()[..], b"abcdefghijklmnopqrstuvwxyz")
	}

	#[test]
	fn it_should_not_read_too_much_data() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let abort = Abort::default().with_max_size(3);
		let resp = client.get(&format!("http://{}/?1234", server.addr()), abort).wait().unwrap();
		assert!(resp.is_success());
		match resp.concat2().wait() {
			Err(Error::SizeLimit) => {}
			other => panic!("expected size limit error, got {:?}", other)
		}
	}

	#[test]
	fn it_should_not_read_too_much_data_sync() {
		let server = TestServer::run();
		let client = Client::new(4).unwrap();
		let abort = Abort::default().with_max_size(3);
		let resp = client.get(&format!("http://{}/?1234", server.addr()), abort).wait().unwrap();
		assert!(resp.is_success());
		let mut buffer = Vec::new();
		let mut reader = BodyReader::new(resp);
		match reader.read_to_end(&mut buffer) {
			Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {}
			other => panic!("expected size limit error, got {:?}", other)
		}
	}

	struct TestServer(Timer);

	impl Service for TestServer {
		type Request = Request;
		type Response = Response;
		type Error = hyper::Error;
		type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

		fn call(&self, req: Request) -> Self::Future {
			match req.uri().path() {
				"/" => {
					let body = req.uri().query().unwrap_or("").to_string();
					let req = Response::new().with_body(body);
					Box::new(future::ok(req))
				}
				"/redirect" => {
					let loc = Location::new(req.uri().query().unwrap_or("/").to_string());
					let req = Response::new()
						.with_status(StatusCode::MovedPermanently)
						.with_header(loc);
					Box::new(future::ok(req))
				}
				"/loop" => {
					let req = Response::new()
						.with_status(StatusCode::MovedPermanently)
						.with_header(Location::new("/loop".to_string()));
					Box::new(future::ok(req))
				}
				"/delay" => {
					let d = Duration::from_secs(req.uri().query().unwrap_or("0").parse().unwrap());
					Box::new(self.0.sleep(d)
							 .map_err(|_| return io::Error::new(io::ErrorKind::Other, "timer error"))
							 .from_err()
							 .map(|_| Response::new()))
				}
				_ => Box::new(future::ok(Response::new().with_status(StatusCode::NotFound)))
			}
		}
	}

	impl TestServer {
		fn run() -> Handle {
			let (tx_start, rx_start) = std::sync::mpsc::sync_channel(1);
			let (tx_end, rx_end) = mpsc::channel(0);
			let rx_end_fut = rx_end.into_future().map(|_| ()).map_err(|_| ());
			thread::spawn(move || {
				let addr = ADDRESS.parse().unwrap();
				let server = Http::new().bind(&addr, || Ok(TestServer(Timer::default()))).unwrap();
				tx_start.send(server.local_addr().unwrap()).unwrap_or(());
				server.run_until(rx_end_fut).unwrap();
			});
			Handle(rx_start.recv().unwrap(), tx_end)
		}
	}

	struct Handle(SocketAddr, mpsc::Sender<()>);

	impl Handle {
		fn addr(&self) -> SocketAddr {
			self.0
		}
	}

	impl Drop for Handle {
		fn drop(&mut self) {
			self.1.clone().send(()).wait().unwrap();
		}
	}
}
