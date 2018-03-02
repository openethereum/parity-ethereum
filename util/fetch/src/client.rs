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

use std::cmp::min;
use std::{io, error, fmt, mem};
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};
use std::thread;
use std::time::Duration;

use futures::{self, Future, Async, Sink, Stream};
use futures::future::{self, Either};
use futures_cpupool::CpuPool;
use futures::sync::{mpsc, oneshot};
use parking_lot::{Condvar, Mutex};

use hyper::{self, Request, Method, StatusCode};
use hyper::header::{UserAgent, Location, ContentType};
use hyper::mime::Mime;

use hyper_rustls;
use tokio_core::reactor;
use url::{self, Url};

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

	/// Spawn the future in context of this `Fetch` thread pool.
	fn process<F, I, E>(&self, f: F) -> BoxFuture<I, E>
		where F: Future<Item=I, Error=E> + Send + 'static,
			  I: Send + 'static,
			  E: Send + 'static;

	/// Spawn the future in context of this `Fetch` thread pool as
	/// "fire and forget", i.e. dropping this future without canceling
	/// the underlying future.
	fn process_and_forget<F, I, E>(&self, f: F)
		where F: Future<Item=I, Error=E> + Send + 'static,
			  I: Send + 'static,
			  E: Send + 'static;

	/// Fetch URL and get a future for the result.
	/// Supports aborting the request in the middle of execution.
	fn fetch_with_abort(&self, url: &str, abort: Abort) -> Self::Result;

	/// Fetch URL and get a future for the result.
	fn fetch(&self, url: &str) -> Self::Result {
		self.fetch_with_abort(url, Default::default())
	}
}

const THREAD_NAME: &str = "fetch";
const CLIENT_TIMEOUT_SECONDS: u64 = 5;
const MAX_REDIRECTS: usize = 5;

type TxResponse  = oneshot::Sender<Result<hyper::Response, Error>>;
type RxResponse  = oneshot::Receiver<Result<hyper::Response, Error>>;
type StartupCond = Arc<(Mutex<Result<(), io::Error>>, Condvar)>;

// `Proto`col values are sent over an mpsc channel from clients to
// their shared background thread with a tokio core and hyper cient inside.
enum Proto {
	Request(Url, hyper::Request, TxResponse, usize),
	Quit // terminates background thread
}

impl Proto {
	fn is_quit(&self) -> bool {
		if let Proto::Quit = *self { true } else { false }
	}
}

/// Fetch client
#[derive(Clone)]
pub struct Client {
	pool:     CpuPool,
	tx_proto: mpsc::Sender<Proto>,
	limit:    Option<usize>
}

impl Client {
	/// Create a new client which spins up a separate thread running a
	/// tokio `Core` and a `hyper::Client`.
	/// Clones of this client share the same background thread.
	pub fn new() -> Result<Self, Error> {
		let startup_done = Arc::new((Mutex::new(Ok(())), Condvar::new()));
		let (tx_proto, rx_proto) = mpsc::channel(64);

		Client::background_thread(startup_done.clone(), tx_proto.clone(), rx_proto)?;

		let mut guard = startup_done.0.lock();
		let startup_result = startup_done.1.wait_for(&mut guard, Duration::from_secs(3));

		if startup_result.timed_out() {
			error!(target: "fetch", "timeout starting {}", THREAD_NAME);
			return Err(Error::Other("timeout starting background thread".into()))
		}
		if let Err(e) = mem::replace(&mut *guard, Ok(())) {
			error!(target: "fetch", "error starting background thread: {}", e);
			return Err(e.into())
		}

		Ok(Client {
			pool:     CpuPool::new(4),
			tx_proto: tx_proto,
			limit:    Some(64 * 1024 * 1024)
		})
	}

	fn background_thread(start: StartupCond,
                         tx_proto: mpsc::Sender<Proto>,
                         rx_proto: mpsc::Receiver<Proto>) -> io::Result<thread::JoinHandle<()>>
	{
		thread::Builder::new().name(THREAD_NAME.into()).spawn(move || {
			let mut core = match reactor::Core::new() {
				Ok(c)  => c,
				Err(e) => {
					*start.0.lock() = Err(e);
					start.1.notify_one();
					return ()
				}
			};
			let handle = core.handle();
			let client = hyper::Client::configure()
				.connector(hyper_rustls::HttpsConnector::new(4, &core.handle()))
				.build(&core.handle());

			start.1.notify_one();
			debug!(target: "fetch", "processing requests ...");

			let maxdur = Duration::from_secs(CLIENT_TIMEOUT_SECONDS);
			let work = rx_proto.take_while(|item| Ok(!item.is_quit())).for_each(|item| {
				if let Proto::Request(url, rq, sender, redir) = item {
					trace!(target: "fetch", "new request to {}", url);
					let timeout = match reactor::Timeout::new(maxdur, &handle) {
						Ok(t)  => t,
						Err(e) => {
							error!(target: "fetch", "failed to create timeout: {}.", e);
							return future::err(())
						}
					};
					let reschedule = tx_proto.clone();
					let future = client.request(rq).select2(timeout).then(move |rs| {
						trace!(target: "fetch", "response received from {}", url);
						// When sending responses back over the oneshot channels, we treat
						// the possibility that the other end is gone as normal, hence we
						// use `unwrap_or(())` and do not error.
						match rs {
							Ok(Either::A((rs, _))) => {
								if let Some(next_url) = redirect_location(url, &rs) {
									if redir == 0 {
										Either::A(future::ok(sender.send(Err(Error::TooManyRedirects)).unwrap_or(())))
									} else {
										let next_req = get(&next_url);
										Either::B(reschedule.send(Proto::Request(next_url, next_req, sender, redir - 1)).then(|result| {
											if let Err(e) = result {
												error!(target: "fetch", "failed to reschedule request: {}", e);
											}
											// We can not recover from this error. Client code will
											// get a `oneshot::Canceled` error since we dropped the
											// `oneshot::Sender`. This should not happen as long as
											// this thread runs, as with `reschedule` we are
											// sending the `Proto` value back to ourselves.
											future::ok(())
										}))
									}
								} else {
									Either::A(future::ok(sender.send(Ok(rs)).unwrap_or(())))
								}
							}
							Ok(Either::B((_, _)))    => Either::A(future::ok(sender.send(Err(Error::Timeout)).unwrap_or(()))),
							Err(Either::A((err, _))) => Either::A(future::ok(sender.send(Err(err.into())).unwrap_or(()))),
							Err(Either::B((err, _))) => Either::A(future::ok(sender.send(Err(err.into())).unwrap_or(()))),
						}
					});
					handle.spawn(future);
					trace!(target: "fetch", "waiting for next request...")
				}
				future::ok(())
			});
			if let Err(()) = core.run(work) {
				error!(target: "fetch", "error while executing future")
			}
			debug!(target: "fetch", "{} background thread finished", THREAD_NAME)
		})
	}

	/// Close this client by shutting down the background thread.
	///
	/// Please note that this will affect all clones of this `Client` as they all
	/// share the same background thread.
	pub fn close(self) -> Result<(), Error> {
		self.tx_proto.clone().send(Proto::Quit).wait()
			.map_err(|e| {
				error!(target: "fetch", "failed to send quit to background thread: {}", e);
				// We can not put `e: SendError<Proto>` into `Other` as it is not `Send`.
				Error::Other("failed to terminate background thread".into())
			})?;
		Ok(())
	}

	/// (Un-)set size limit on response body.
	pub fn set_limit(&mut self, limit: Option<usize>) {
		self.limit = limit;
	}

	/// Returns a handle to underlying CpuPool of this client.
	pub fn pool(&self) -> CpuPool {
		self.pool.clone()
	}
}

impl Fetch for Client {
	type Result = BoxFuture<Response, Error>;

	fn fetch_with_abort(&self, url: &str, abort: Abort) -> Self::Result {
		debug!(target: "fetch", "fetching: {:?}", url);

		let url: Url = match url.parse() {
			Ok(u)  => u,
			Err(e) => return Box::new(futures::future::err(e.into()))
		};

		let req    = get(&url);
		let sender = self.tx_proto.clone();
		let limit  = self.limit.clone();
		let (tx_res, rx_res) = oneshot::channel();
		let future = sender.send(Proto::Request(url.clone(), req, tx_res, MAX_REDIRECTS))
			.map(|_| rx_res)
			.map_err(|e| {
				error!(target: "fetch", "failed to schedule request: {}", e);
				Error::Other("failed to schedule request".into())
			})
			.and_then(move |rx_res| {
				FetchTask {
					url: url,
					rx_res: rx_res,
					limit: limit,
					abort: abort
				}
			});
		Box::new(future)
	}

	fn process<F, I, E>(&self, f: F) -> BoxFuture<I, E>
		where F: Future<Item=I, Error=E> + Send + 'static,
			  I: Send + 'static,
			  E: Send + 'static
	{
		Box::new(self.pool.spawn(f))
	}

	fn process_and_forget<F, I, E>(&self, f: F)
		where F: Future<Item=I, Error=E> + Send + 'static,
			  I: Send + 'static,
			  E: Send + 'static
	{
		self.pool.spawn(f).forget()
	}
}

fn redirect_location(u: Url, r: &hyper::Response) -> Option<Url> {
	use hyper::StatusCode::*;
	match r.status() {
		MovedPermanently
		| PermanentRedirect
		| TemporaryRedirect
		| Found
		| SeeOther => {
			if let Some(loc) = r.headers().get::<Location>() {
				u.join(loc).ok()
			} else {
				None
			}
		}
		_ => None
	}
}

fn get(u: &Url) -> hyper::Request {
    let uri = u.as_ref().parse().expect("Every valid URL is aso a URI");
	let mut rq = Request::new(Method::Get, uri);
	rq.headers_mut().set(UserAgent::new("Parity Fetch Neo"));
	rq
}

struct FetchTask {
	url:    Url,
	rx_res: RxResponse,
	limit:  Option<usize>,
	abort:  Abort
}

impl Future for FetchTask {
	type Item = Response;
	type Error = Error;

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		if self.abort.is_aborted() {
			debug!(target: "fetch", "Fetch of {:?} aborted.", self.url);
			return Err(Error::Aborted);
		}
		match self.rx_res.poll()? {
			Async::Ready(Err(e)) => Err(e.into()),
			Async::Ready(Ok(r))  => {
				let ctype = r.headers().get::<ContentType>().cloned();
				Ok(Async::Ready(Response {
					inner: ResponseInner::Response(r.status(), ctype, BodyReader::new(r.body())),
					abort: self.abort.clone(),
					limit: self.limit.clone(),
					read:  0
				}))
			}
			Async::NotReady => Ok(Async::NotReady)
		}
	}
}

/// Fetch related error cases.
#[derive(Debug)]
pub enum Error {
	/// Error produced by hyper.
	Hyper(hyper::Error),
	/// I/O error
	Io(io::Error),
	/// URL parse error
	Url(url::ParseError),
	/// Request aborted
	Aborted,
	/// Followed too many redirects
	TooManyRedirects,
	/// Request took too long
	Timeout,
	/// The background request procesing was canceled
	Canceled,
	/// Some other error
	Other(Box<error::Error + Send + Sync + 'static>)
}

impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Aborted          => write!(fmt, "The request has been aborted."),
			Error::Hyper(ref e)     => write!(fmt, "{}", e),
			Error::Url(ref e)       => write!(fmt, "{}", e),
			Error::Io(ref e)        => write!(fmt, "{}", e),
			Error::Other(ref e)     => write!(fmt, "{}", e),
			Error::TooManyRedirects => write!(fmt, "too many redirects"),
			Error::Timeout          => write!(fmt, "request timed out"),
			Error::Canceled         => write!(fmt, "background thread canceled request processing"),
		}
	}
}

impl From<oneshot::Canceled> for Error {
	fn from(_: oneshot::Canceled) -> Self {
		Error::Canceled
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

enum ResponseInner {
	Response(StatusCode, Option<ContentType>, BodyReader),
	Reader(Box<io::Read + Send>),
	NotFound
}

impl fmt::Debug for ResponseInner {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ResponseInner::Response(s, ..) => write!(f, "hyper response (status={})", s),
			ResponseInner::NotFound        => write!(f, "not found"),
			ResponseInner::Reader(_)       => write!(f, "io reader"),
		}
	}
}

/// A fetch response type.
#[derive(Debug)]
pub struct Response {
	inner: ResponseInner,
	abort: Abort,
	limit: Option<usize>,
	read:  usize
}

impl Response {
	/// Creates new successfuly response reading from a file.
	pub fn from_reader<R: io::Read + Send + 'static>(reader: R) -> Self {
		Response {
			inner: ResponseInner::Reader(Box::new(reader)),
			abort: Abort::default(),
			limit: None,
			read:  0
		}
	}

	/// Creates 404 response (useful for tests)
	pub fn not_found() -> Self {
		Response {
			inner: ResponseInner::NotFound,
			abort: Abort::default(),
			limit: None,
			read:  0
		}
	}

	/// Returns status code of this response.
	pub fn status(&self) -> StatusCode {
		match self.inner {
			ResponseInner::Response(s, ..) => s,
			ResponseInner::NotFound        => StatusCode::NotFound,
			_                              => StatusCode::Ok
		}
	}

	/// Returns `true` if response status code is successful.
	pub fn is_success(&self) -> bool {
		self.status() == StatusCode::Ok
	}

	/// Returns `true` if content type of this response is `text/html`
	pub fn is_html(&self) -> bool {
		if let Some(ref mime) = self.content_type() {
			mime.type_() == "text" && mime.subtype() == "html"
		} else {
			false
		}
	}

	/// Returns content type of this response (if present)
	pub fn content_type(&self) -> Option<Mime> {
		if let ResponseInner::Response(_, ref c, _) = self.inner {
			c.as_ref().map(|mime| mime.0.clone())
		} else {
			None
		}
	}
}

impl io::Read for Response {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if self.abort.is_aborted() {
			return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Fetch aborted."));
		}

		let res = match self.inner {
			ResponseInner::Response(_, _, ref mut r) => r.read(buf),
			ResponseInner::NotFound                  => return Ok(0),
			ResponseInner::Reader(ref mut r)         => r.read(buf)
		};

		// increase bytes read
		if let Ok(read) = res {
			self.read += read
		}

		// check limit
		match self.limit {
			Some(limit) if limit < self.read => {
				return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Size limit reached."));
			}
			_ => {}
		}

		res
	}
}

// `BodyReader` serves as a bridge from async to sync I/O. It implements
// `io::Read` by repedately waiting for the next `Chunk` of hyper's response `Body`.
struct BodyReader {
	chunk:  hyper::Chunk,
	body:   Option<hyper::Body>,
	offset: usize
}

impl BodyReader {
	fn new(b: hyper::Body) -> BodyReader {
		BodyReader { body: Some(b), chunk: Default::default(), offset: 0 }
	}
}

impl io::Read for BodyReader {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let mut m = 0;
		while self.body.is_some() {
			// Can we still read from the current chunk?
			if self.offset < self.chunk.len() {
				let k = min(self.chunk.len() - self.offset, buf.len() - m);
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

#[cfg(test)]
mod test {
	use super::*;
	use std::io::Read;

	#[test]
	fn it_should_fetch() {
		let client = Client::new().unwrap();
		let future = client.fetch("https://httpbin.org/drip?numbytes=3&duration=3&delay=1&code=200");
		let mut resp = future.wait().unwrap();
		assert!(resp.is_success());
		let mut body = Vec::new();
		resp.read_to_end(&mut body).unwrap();
		assert_eq!(body.len(), 3)
	}

	#[test]
	fn it_should_timeout() {
		let client = Client::new().unwrap();
		let future = client.fetch("https://httpbin.org/delay/7");
		match future.wait() {
			Err(Error::Timeout) => {}
			other => panic!("expected timeout, got {:?}", other)
		}
	}

	#[test]
	fn it_should_follow_redirects() {
		let client = Client::new().unwrap();
		let future = client.fetch("https://httpbin.org/absolute-redirect/3");
		assert!(future.wait().unwrap().is_success())
	}

	#[test]
	fn it_should_follow_relative_redirects() {
		let client = Client::new().unwrap();
		let future = client.fetch("https://httpbin.org/relative-redirect/3");
		assert!(future.wait().unwrap().is_success())
	}

	#[test]
	fn it_should_not_follow_too_many_redirects() {
		let client = Client::new().unwrap();
		let future = client.fetch("https://httpbin.org/absolute-redirect/100");
		match future.wait() {
			Err(Error::TooManyRedirects) => {}
			other => panic!("expected too many redirects error, got {:?}", other)
		}
	}

	#[test]
	fn it_should_read_data() {
		let client = Client::new().unwrap();
		let future = client.fetch("https://httpbin.org/bytes/1024");
		let mut resp = future.wait().unwrap();
		assert!(resp.is_success());
		let mut body = Vec::new();
		resp.read_to_end(&mut body).unwrap();
		assert_eq!(body.len(), 1024)
	}

	#[test]
	fn it_should_read_chunked_data() {
		let client = Client::new().unwrap();
		let future = client.fetch("https://httpbin.org/stream-bytes/1024?chunk_size=19");
		let mut resp = future.wait().unwrap();
		assert!(resp.is_success());
		let mut body = Vec::new();
		resp.read_to_end(&mut body).unwrap();
		assert_eq!(body.len(), 1024)
	}
}
