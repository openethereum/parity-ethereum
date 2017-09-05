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

//! Hyper Server Handler that fetches a file during a request (proxy).

use std::fmt;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, Duration};
use fetch::{self, Fetch};
use futures::Future;
use parity_reactor::Remote;
use parking_lot::Mutex;

use hyper::{server, Decoder, Encoder, Next, Method, Control};
use hyper::net::HttpStream;
use hyper::uri::RequestUri;
use hyper::status::StatusCode;

use endpoint::EndpointPath;
use handlers::{ContentHandler, StreamingHandler};
use page::{LocalPageEndpoint, PageHandlerWaiting};
use {Embeddable};

const FETCH_TIMEOUT: u64 = 300;

pub enum ValidatorResponse {
	Local(LocalPageEndpoint),
	Streaming(StreamingHandler<fetch::Response>),
}

pub trait ContentValidator: Send + 'static {
	type Error: fmt::Debug + fmt::Display;

	fn validate_and_install(&self, fetch::Response) -> Result<ValidatorResponse, Self::Error>;
}

enum FetchState {
	Waiting,
	NotStarted(String),
	Error(ContentHandler),
	InProgress(mpsc::Receiver<FetchState>),
	Streaming(StreamingHandler<fetch::Response>),
	Done(LocalPageEndpoint, Box<PageHandlerWaiting>),
}

enum WaitResult {
	Error(ContentHandler),
	Done(LocalPageEndpoint),
	NonAwaitable,
}

#[derive(Clone)]
pub struct FetchControl {
	abort: Arc<AtomicBool>,
	listeners: Arc<Mutex<Vec<(Control, mpsc::Sender<WaitResult>)>>>,
	deadline: Instant,
}

impl Default for FetchControl {
	fn default() -> Self {
		FetchControl {
			abort: Arc::new(AtomicBool::new(false)),
			listeners: Arc::new(Mutex::new(Vec::new())),
			deadline: Instant::now() + Duration::from_secs(FETCH_TIMEOUT),
		}
	}
}

impl FetchControl {
	fn notify<F: Fn() -> WaitResult>(&self, status: F) {
		let mut listeners = self.listeners.lock();
		for (control, sender) in listeners.drain(..) {
			trace!(target: "dapps", "Resuming request waiting for content...");
			if let Err(e) = sender.send(status()) {
				trace!(target: "dapps", "Waiting listener notification failed: {:?}", e);
			} else {
				let _ = control.ready(Next::read());
			}
		}
	}

	fn set_status(&self, status: &FetchState) {
		match *status {
			FetchState::Error(ref handler) => self.notify(|| WaitResult::Error(handler.clone())),
			FetchState::Done(ref endpoint, _) => self.notify(|| WaitResult::Done(endpoint.clone())),
			FetchState::Streaming(_) => self.notify(|| WaitResult::NonAwaitable),
			FetchState::NotStarted(_) | FetchState::InProgress(_) | FetchState::Waiting => {},
		}
	}

	pub fn is_deadline_reached(&self) -> bool {
		self.deadline < Instant::now()
	}

	pub fn abort(&self) {
		self.abort.store(true, Ordering::SeqCst);
	}

	pub fn to_async_handler(&self, path: EndpointPath, control: Control) -> Box<server::Handler<HttpStream> + Send> {
		let (tx, rx) = mpsc::channel();
		self.listeners.lock().push((control, tx));

		Box::new(WaitingHandler {
			receiver: rx,
			state: FetchState::Waiting,
			uri: RequestUri::default(),
			path: path,
		})
	}
}

pub struct WaitingHandler {
	receiver: mpsc::Receiver<WaitResult>,
	state: FetchState,
	uri: RequestUri,
	path: EndpointPath,
}

impl server::Handler<HttpStream> for WaitingHandler {
	fn on_request(&mut self, request: server::Request<HttpStream>) -> Next {
		self.uri = request.uri().clone();
		Next::wait()
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<HttpStream>) -> Next {
		let result = self.receiver.try_recv().ok();
		self.state = match result {
			Some(WaitResult::Error(handler)) => FetchState::Error(handler),
			Some(WaitResult::Done(endpoint)) => {
				let mut page_handler = endpoint.to_page_handler(self.path.clone());
				page_handler.set_uri(&self.uri);
				FetchState::Done(endpoint, page_handler)
			},
			_ => {
				warn!("A result for waiting request was not received.");
				FetchState::Waiting
			},
		};

		match self.state {
			FetchState::Done(_, ref mut handler) => handler.on_request_readable(decoder),
			FetchState::Streaming(ref mut handler) => handler.on_request_readable(decoder),
			FetchState::Error(ref mut handler) => handler.on_request_readable(decoder),
			_ => Next::write(),
		}
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.state {
			FetchState::Done(_, ref mut handler) => handler.on_response(res),
			FetchState::Streaming(ref mut handler) => handler.on_response(res),
			FetchState::Error(ref mut handler) => handler.on_response(res),
			_ => Next::end(),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		match self.state {
			FetchState::Done(_, ref mut handler) => handler.on_response_writable(encoder),
			FetchState::Streaming(ref mut handler) => handler.on_response_writable(encoder),
			FetchState::Error(ref mut handler) => handler.on_response_writable(encoder),
			_ => Next::end(),
		}
	}
}

#[derive(Clone)]
struct Errors {
	embeddable_on: Embeddable,
}

impl Errors {
	fn download_error<E: fmt::Debug>(&self, e: E) -> ContentHandler {
		ContentHandler::error(
			StatusCode::BadGateway,
			"Download Error",
			"There was an error when fetching the content.",
			Some(&format!("{:?}", e)),
			self.embeddable_on.clone(),
		)
	}

	fn invalid_content<E: fmt::Debug>(&self, e: E) -> ContentHandler {
		ContentHandler::error(
			StatusCode::BadGateway,
			"Invalid Dapp",
			"Downloaded bundle does not contain a valid content.",
			Some(&format!("{:?}", e)),
			self.embeddable_on.clone(),
		)
	}

	fn timeout_error(&self) -> ContentHandler {
		ContentHandler::error(
			StatusCode::GatewayTimeout,
			"Download Timeout",
			&format!("Could not fetch content within {} seconds.", FETCH_TIMEOUT),
			None,
			self.embeddable_on.clone(),
		)
	}

	fn method_not_allowed(&self) -> ContentHandler {
		ContentHandler::error(
			StatusCode::MethodNotAllowed,
			"Method Not Allowed",
			"Only <code>GET</code> requests are allowed.",
			None,
			self.embeddable_on.clone(),
		)
	}
}

pub struct ContentFetcherHandler<H: ContentValidator, F: Fetch> {
	fetch_control: FetchControl,
	control: Control,
	remote: Remote,
	status: FetchState,
	fetch: F,
	installer: Option<H>,
	path: EndpointPath,
	errors: Errors,
}

impl<H: ContentValidator, F: Fetch> ContentFetcherHandler<H, F> {
	pub fn new(
		url: String,
		path: EndpointPath,
		control: Control,
		installer: H,
		embeddable_on: Embeddable,
		remote: Remote,
		fetch: F,
	) -> Self {
		ContentFetcherHandler {
			fetch_control: FetchControl::default(),
			control,
			remote,
			fetch,
			status: FetchState::NotStarted(url),
			installer: Some(installer),
			path,
			errors: Errors {
				embeddable_on,
			},
		}
	}

	pub fn fetch_control(&self) -> FetchControl {
		self.fetch_control.clone()
	}

	fn fetch_content(&self, uri: RequestUri, url: &str, installer: H) -> mpsc::Receiver<FetchState> {
		let (tx, rx) = mpsc::channel();
		let abort = self.fetch_control.abort.clone();

		let path = self.path.clone();
		let tx2 = tx.clone();
		let control = self.control.clone();
		let errors = self.errors.clone();

		let future = self.fetch.fetch_with_abort(url, abort.into()).then(move |result| {
			trace!(target: "dapps", "Fetching content finished. Starting validation: {:?}", result);
			let new_state = match result {
				Ok(response) => match installer.validate_and_install(response) {
					Ok(ValidatorResponse::Local(endpoint)) => {
						trace!(target: "dapps", "Validation OK. Returning response.");
						let mut handler = endpoint.to_page_handler(path);
						handler.set_uri(&uri);
						FetchState::Done(endpoint, handler)
					},
					Ok(ValidatorResponse::Streaming(handler)) => {
						trace!(target: "dapps", "Validation OK. Streaming response.");
						FetchState::Streaming(handler)
					},
					Err(e) => {
						trace!(target: "dapps", "Error while validating content: {:?}", e);
						FetchState::Error(errors.invalid_content(e))
					},
				},
				Err(e) => {
					warn!(target: "dapps", "Unable to fetch content: {:?}", e);
					FetchState::Error(errors.download_error(e))
				},
			};
			// Content may be resolved when the connection is already dropped.
			let _ = tx2.send(new_state);
			// Ignoring control errors
			let _ = control.ready(Next::read());
			Ok(()) as Result<(), ()>
		});

		// make sure to run within fetch thread pool.
		let future = self.fetch.process(future);
		// spawn to event loop
		let control = self.control.clone();
		let errors = self.errors.clone();
		self.remote.spawn_with_timeout(|| future, Duration::from_secs(FETCH_TIMEOUT), move || {
			// Notify about the timeout
			let _ = tx.send(FetchState::Error(errors.timeout_error()));
			// Ignoring control errors
			let _ = control.ready(Next::read());
		});

		rx
	}
}

impl<H: ContentValidator, F: Fetch> server::Handler<HttpStream> for ContentFetcherHandler<H, F> {
	fn on_request(&mut self, request: server::Request<HttpStream>) -> Next {
		let status = if let FetchState::NotStarted(ref url) = self.status {
			let uri = request.uri().clone();
			let installer = self.installer.take().expect("Installer always set initialy; installer used only in on_request; on_request invoked only once; qed");

			Some(match *request.method() {
				// Start fetching content
				Method::Get => {
					trace!(target: "dapps", "Fetching content from: {:?}", url);
					let receiver = self.fetch_content(uri, url, installer);
					FetchState::InProgress(receiver)
				},
				// or return error
				_ => FetchState::Error(self.errors.method_not_allowed()),
			})
		} else { None };

		if let Some(status) = status {
			self.fetch_control.set_status(&status);
			self.status = status;
		}

		Next::read()
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<HttpStream>) -> Next {
		let (status, next) = match self.status {
			// Request may time out
			FetchState::InProgress(_) if self.fetch_control.is_deadline_reached() => {
				trace!(target: "dapps", "Fetching dapp failed because of timeout.");
				(Some(FetchState::Error(self.errors.timeout_error())), Next::write())
			},
			FetchState::InProgress(ref receiver) => {
				// Check if there is an answer
				let rec = receiver.try_recv();
				match rec {
					// just return the new state
					Ok(state) => (Some(state), Next::write()),
					// wait some more
					_ => (None, Next::wait())
				}
			},
			FetchState::Error(ref mut handler) => (None, handler.on_request_readable(decoder)),
			_ => (None, Next::write()),
		};

		if let Some(status) = status {
			self.fetch_control.set_status(&status);
			self.status = status;
		}

		next
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.status {
			FetchState::Done(_, ref mut handler) => handler.on_response(res),
			FetchState::Streaming(ref mut handler) => handler.on_response(res),
			FetchState::Error(ref mut handler) => handler.on_response(res),
			_ => Next::end(),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		match self.status {
			FetchState::Done(_, ref mut handler) => handler.on_response_writable(encoder),
			FetchState::Streaming(ref mut handler) => handler.on_response_writable(encoder),
			FetchState::Error(ref mut handler) => handler.on_response_writable(encoder),
			_ => Next::end(),
		}
	}
}
