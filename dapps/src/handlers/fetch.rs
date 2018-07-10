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

//! Hyper Server Handler that fetches a file during a request (proxy).

use std::{fmt, mem};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use fetch::{self, Fetch};
use futures::sync::oneshot;
use futures::{self, Future};
use futures_cpupool::CpuPool;
use hyper;
use parking_lot::Mutex;

use endpoint::{self, EndpointPath};
use handlers::{ContentHandler, StreamingHandler, FETCH_TIMEOUT, errors};
use page::local;

pub enum ValidatorResponse {
	Local(local::Dapp),
	Streaming(StreamingHandler<fetch::BodyReader>),
}

pub trait ContentValidator: Sized + Send + 'static {
	type Error: fmt::Debug + fmt::Display;

	fn validate_and_install(self, fetch::Response) -> Result<ValidatorResponse, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct FetchControl {
	abort: Arc<AtomicBool>,
	listeners: Arc<Mutex<Vec<oneshot::Sender<WaitResult>>>>,
	deadline: Instant,
}

impl Default for FetchControl {
	fn default() -> Self {
		FetchControl {
			abort: Arc::new(AtomicBool::new(false)),
			listeners: Arc::new(Mutex::new(Vec::new())),
			deadline: Instant::now() + FETCH_TIMEOUT,
		}
	}
}

impl FetchControl {
	pub fn is_deadline_reached(&self) -> bool {
		self.deadline < Instant::now()
	}

	pub fn abort(&self) {
		self.abort.store(true, Ordering::SeqCst);
	}

	pub fn to_response(&self, path: EndpointPath) -> endpoint::Response {
		let (tx, receiver) = oneshot::channel();
		self.listeners.lock().push(tx);

		Box::new(WaitingHandler {
			path,
			state: WaitState::Waiting(receiver),
		})
	}

	fn notify<F: Fn() -> WaitResult>(&self, status: F) {
		let mut listeners = self.listeners.lock();
		for sender in listeners.drain(..) {
			trace!(target: "dapps", "Resuming request waiting for content...");
			if let Err(_) = sender.send(status()) {
				trace!(target: "dapps", "Waiting listener notification failed.");
			}
		}
	}

	fn set_status(&self, status: &FetchState) {
		match *status {
			FetchState::Error(ref handler) => self.notify(|| WaitResult::Error(handler.clone())),
			FetchState::Done(ref endpoint, _) => self.notify(|| WaitResult::Done(endpoint.clone())),
			FetchState::Streaming(_) => self.notify(|| WaitResult::NonAwaitable),
			FetchState::InProgress(_) => {},
			FetchState::Empty => {},
		}
	}
}

enum WaitState {
	Waiting(oneshot::Receiver<WaitResult>),
	Done(endpoint::Response),
}

#[derive(Debug)]
enum WaitResult {
	Error(ContentHandler),
	Done(local::Dapp),
	NonAwaitable,
}

pub struct WaitingHandler {
	path: EndpointPath,
	state: WaitState,
}

impl Future for WaitingHandler {
	type Item = hyper::Response;
	type Error = hyper::Error;

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		loop {
			let new_state = match self.state {
				WaitState::Waiting(ref mut receiver) => {
					let result = try_ready!(receiver.poll().map_err(|_| hyper::Error::Timeout));

					match result {
						WaitResult::Error(handler) => {
							return Ok(futures::Async::Ready(handler.into()));
						},
						WaitResult::NonAwaitable => {
							return Ok(futures::Async::Ready(errors::streaming().into()));
						},
						WaitResult::Done(endpoint) => {
							WaitState::Done(endpoint.to_response(&self.path).into())
						},
					}
				},
				WaitState::Done(ref mut response) => {
					return response.poll()
				},
			};

			self.state = new_state;
		}
	}
}

enum FetchState {
	Error(ContentHandler),
	InProgress(Box<Future<Item=FetchState, Error=()> + Send>),
	Streaming(hyper::Response),
	Done(local::Dapp, endpoint::Response),
	Empty,
}

impl fmt::Debug for FetchState {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::FetchState::*;

		write!(fmt, "FetchState(")?;
		match *self {
			Error(ref error) => write!(fmt, "error: {:?}", error),
			InProgress(_) => write!(fmt, "in progress"),
			Streaming(ref res) => write!(fmt, "streaming: {:?}", res),
			Done(ref endpoint, _) => write!(fmt, "done: {:?}", endpoint),
			Empty => write!(fmt, "?"),
		}?;
		write!(fmt, ")")
	}
}

#[derive(Debug)]
pub struct ContentFetcherHandler {
	fetch_control: FetchControl,
	status: FetchState,
}

impl ContentFetcherHandler {
	pub fn fetch_control(&self) -> FetchControl {
		self.fetch_control.clone()
	}

	pub fn new<H: ContentValidator, F: Fetch>(
		method: &hyper::Method,
		url: &str,
		path: EndpointPath,
		installer: H,
		fetch: F,
		pool: CpuPool,
	) -> Self {
		let fetch_control = FetchControl::default();

		// Validation of method
		let status = match *method {
			// Start fetching content
			hyper::Method::Get => {
				trace!(target: "dapps", "Fetching content from: {:?}", url);
				FetchState::InProgress(Self::fetch_content(
						pool,
						fetch,
						url,
						fetch_control.abort.clone(),
						path,
						installer,
				))
			},
			// or return error
			_ => FetchState::Error(errors::method_not_allowed()),
		};

		ContentFetcherHandler {
			fetch_control,
			status,
		}
	}

	fn fetch_content<H: ContentValidator, F: Fetch>(
		pool: CpuPool,
		fetch: F,
		url: &str,
		abort: Arc<AtomicBool>,
		path: EndpointPath,
		installer: H,
	) -> Box<Future<Item=FetchState, Error=()> + Send> {
		// Start fetching the content
		let pool2 = pool.clone();
		let future = fetch.get(url, abort.into()).then(move |result| {
			trace!(target: "dapps", "Fetching content finished. Starting validation: {:?}", result);
			Ok(match result {
				Ok(response) => match installer.validate_and_install(response) {
					Ok(ValidatorResponse::Local(endpoint)) => {
						trace!(target: "dapps", "Validation OK. Returning response.");
						let response = endpoint.to_response(&path);
						FetchState::Done(endpoint, response)
					},
					Ok(ValidatorResponse::Streaming(stream)) => {
						trace!(target: "dapps", "Validation OK. Streaming response.");
						let (reading, response) = stream.into_response();
						pool.spawn(reading).forget();
						FetchState::Streaming(response)
					},
					Err(e) => {
						trace!(target: "dapps", "Error while validating content: {:?}", e);
						FetchState::Error(errors::invalid_content(e))
					},
				},
				Err(e) => {
					warn!(target: "dapps", "Unable to fetch content: {:?}", e);
					FetchState::Error(errors::download_error(e))
				},
			})
		});

		// make sure to run within fetch thread pool.
		Box::new(pool2.spawn(future))
	}
}

impl Future for ContentFetcherHandler {
	type Item = hyper::Response;
	type Error = hyper::Error;

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		loop {
			trace!(target: "dapps", "Polling status: {:?}", self.status);
			self.status = match mem::replace(&mut self.status, FetchState::Empty) {
				FetchState::Error(error) => {
					return Ok(futures::Async::Ready(error.into()));
				},
				FetchState::Streaming(response) => {
					return Ok(futures::Async::Ready(response));
				},
				any => any,
			};

			let status = match self.status {
				// Request may time out
				FetchState::InProgress(_) if self.fetch_control.is_deadline_reached() => {
					trace!(target: "dapps", "Fetching dapp failed because of timeout.");
					FetchState::Error(errors::timeout_error())
				},
				FetchState::InProgress(ref mut receiver) => {
					// Check if there is a response
					trace!(target: "dapps", "Polling streaming response.");
					try_ready!(receiver.poll().map_err(|err| {
						warn!(target: "dapps", "Error while fetching response: {:?}", err);
						hyper::Error::Timeout
					}))
				},
				FetchState::Done(_, ref mut response) => {
					return response.poll()
				},
				FetchState::Empty => panic!("Future polled twice."),
				_ => unreachable!(),
			};

			trace!(target: "dapps", "New status: {:?}", status);
			self.fetch_control.set_status(&status);
			self.status = status;
		}
	}
}
