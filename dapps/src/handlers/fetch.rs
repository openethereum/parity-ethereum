// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::{fs, fmt};
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, Duration};
use util::Mutex;
use url::Url;
use fetch::{Client, Fetch, FetchResult};

use hyper::{server, Decoder, Encoder, Next, Method, Control};
use hyper::net::HttpStream;
use hyper::status::StatusCode;

use handlers::{ContentHandler, Redirection, extract_url};
use page::LocalPageEndpoint;

const FETCH_TIMEOUT: u64 = 30;

enum FetchState {
	NotStarted(String),
	Error(ContentHandler),
	InProgress(mpsc::Receiver<FetchResult>),
	Done(String, LocalPageEndpoint, Redirection),
}

pub trait ContentValidator {
	type Error: fmt::Debug + fmt::Display;

	fn validate_and_install(&self, app: PathBuf) -> Result<(String, LocalPageEndpoint), Self::Error>;
	fn done(&self, Option<LocalPageEndpoint>);
}

pub struct FetchControl {
	abort: Arc<AtomicBool>,
	listeners: Mutex<Vec<(Control, mpsc::Sender<FetchState>)>>,
	deadline: Instant,
}

impl Default for FetchControl {
	fn default() -> Self {
		FetchControl {
			abort: Arc::new(AtomicBool::new(false)),
			listeners: Mutex::new(Vec::new()),
			deadline: Instant::now() + Duration::from_secs(FETCH_TIMEOUT),
		}
	}
}

impl FetchControl {
	fn notify<F: Fn() -> FetchState>(&self, status: F) {
		let mut listeners = self.listeners.lock();
		for (control, sender) in listeners.drain(..) {
			if let Err(e) = sender.send(status()) {
				trace!(target: "dapps", "Waiting listener notification failed: {:?}", e);
			} else {
				let _ = control.ready(Next::read());
			}
		}
	}

	fn set_status(&self, status: &FetchState) {
		match *status {
			FetchState::Error(ref handler) => self.notify(|| FetchState::Error(handler.clone())),
			FetchState::Done(ref id, ref endpoint, ref handler) => self.notify(|| FetchState::Done(id.clone(), endpoint.clone(), handler.clone())),
			FetchState::NotStarted(_) | FetchState::InProgress(_) => {},
		}
	}

	pub fn abort(&self) {
		self.abort.store(true, Ordering::SeqCst);
	}

	pub fn to_handler(&self, control: Control) -> Box<server::Handler<HttpStream> + Send> {
		let (tx, rx) = mpsc::channel();
		self.listeners.lock().push((control, tx));

		Box::new(WaitingHandler {
			receiver: rx,
			state: None,
		})
	}
}

pub struct WaitingHandler {
	receiver: mpsc::Receiver<FetchState>,
	state: Option<FetchState>,
}

impl server::Handler<HttpStream> for WaitingHandler {
	fn on_request(&mut self, _request: server::Request<HttpStream>) -> Next {
		Next::wait()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		self.state = self.receiver.try_recv().ok();
		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.state {
			Some(FetchState::Done(_, _, ref mut handler)) => handler.on_response(res),
			Some(FetchState::Error(ref mut handler)) => handler.on_response(res),
			_ => Next::end(),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		match self.state {
			Some(FetchState::Done(_, _, ref mut handler)) => handler.on_response_writable(encoder),
			Some(FetchState::Error(ref mut handler)) => handler.on_response_writable(encoder),
			_ => Next::end(),
		}
	}
}

pub struct ContentFetcherHandler<H: ContentValidator> {
	fetch_control: Arc<FetchControl>,
	control: Option<Control>,
	status: FetchState,
	client: Option<Client>,
	installer: H,
	request_url: Option<Url>,
	embeddable_on: Option<(String, u16)>,
}

impl<H: ContentValidator> Drop for ContentFetcherHandler<H> {
	fn drop(&mut self) {
		let result = match self.status {
			FetchState::Done(_, ref result, _) => Some(result.clone()),
			_ => None,
		};
		self.installer.done(result);
	}
}

impl<H: ContentValidator> ContentFetcherHandler<H> {

	pub fn new(
		url: String,
		control: Control,
		handler: H,
		embeddable_on: Option<(String, u16)>,
	) -> (Self, Arc<FetchControl>) {

		let fetch_control = Arc::new(FetchControl::default());
		let client = Client::default();
		let handler = ContentFetcherHandler {
			fetch_control: fetch_control.clone(),
			control: Some(control),
			client: Some(client),
			status: FetchState::NotStarted(url),
			installer: handler,
			request_url: None,
			embeddable_on: embeddable_on,
		};

		(handler, fetch_control)
	}

	fn close_client(client: &mut Option<Client>) {
		client.take()
			.expect("After client is closed we are going into write, hence we can never close it again")
			.close();
	}

	fn fetch_content(client: &mut Client, url: &str, abort: Arc<AtomicBool>, control: Control) -> Result<mpsc::Receiver<FetchResult>, String> {
		client.request(url, abort, Box::new(move || {
			trace!(target: "dapps", "Fetching finished.");
			// Ignoring control errors
			let _ = control.ready(Next::read());
		})).map_err(|e| format!("{:?}", e))
	}
}

impl<H: ContentValidator> server::Handler<HttpStream> for ContentFetcherHandler<H> {
	fn on_request(&mut self, request: server::Request<HttpStream>) -> Next {
		self.request_url = extract_url(&request);
		let status = if let FetchState::NotStarted(ref url) = self.status {
			Some(match *request.method() {
				// Start fetching content
				Method::Get => {
					trace!(target: "dapps", "Fetching content from: {:?}", url);
					let control = self.control.take().expect("on_request is called only once, thus control is always Some");
					let client = self.client.as_mut().expect("on_request is called before client is closed.");
					let fetch = Self::fetch_content(client, url, self.fetch_control.abort.clone(), control);
					match fetch {
						Ok(receiver) => FetchState::InProgress(receiver),
						Err(e) => FetchState::Error(ContentHandler::error(
							StatusCode::BadGateway,
							"Unable To Start Dapp Download",
							"Could not initialize download of the dapp. It might be a problem with the remote server.",
							Some(&format!("{}", e)),
							self.embeddable_on.clone(),
						)),
					}
				},
				// or return error
				_ => FetchState::Error(ContentHandler::error(
					StatusCode::MethodNotAllowed,
					"Method Not Allowed",
					"Only <code>GET</code> requests are allowed.",
					None,
					self.embeddable_on.clone(),
				)),
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
			FetchState::InProgress(_) if self.fetch_control.deadline < Instant::now() => {
				trace!(target: "dapps", "Fetching dapp failed because of timeout.");
				let timeout = ContentHandler::error(
					StatusCode::GatewayTimeout,
					"Download Timeout",
					&format!("Could not fetch content within {} seconds.", FETCH_TIMEOUT),
					None,
					self.embeddable_on.clone(),
				);
				Self::close_client(&mut self.client);
				(Some(FetchState::Error(timeout)), Next::write())
			},
			FetchState::InProgress(ref receiver) => {
				// Check if there is an answer
				let rec = receiver.try_recv();
				match rec {
					// Unpack and validate
					Ok(Ok(path)) => {
						trace!(target: "dapps", "Fetching content finished. Starting validation ({:?})", path);
						Self::close_client(&mut self.client);
						// Unpack and verify
						let state = match self.installer.validate_and_install(path.clone()) {
							Err(e) => {
								trace!(target: "dapps", "Error while validating content: {:?}", e);
								FetchState::Error(ContentHandler::error(
									StatusCode::BadGateway,
									"Invalid Dapp",
									"Downloaded bundle does not contain a valid content.",
									Some(&format!("{:?}", e)),
									self.embeddable_on.clone(),
								))
							},
							Ok((id, result)) => {
								let url: String = self.request_url.take()
									.map(|url| url.raw.into_string())
									.expect("Request URL always read in on_request; qed");
								FetchState::Done(id, result, Redirection::new(&url))
							},
						};
						// Remove temporary zip file
						let _ = fs::remove_file(path);
						(Some(state), Next::write())
					},
					Ok(Err(e)) => {
						warn!(target: "dapps", "Unable to fetch content: {:?}", e);
						let error = ContentHandler::error(
							StatusCode::BadGateway,
							"Download Error",
							"There was an error when fetching the content.",
							Some(&format!("{:?}", e)),
							self.embeddable_on.clone(),
						);
						(Some(FetchState::Error(error)), Next::write())
					},
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
			FetchState::Done(_, _, ref mut handler) => handler.on_response(res),
			FetchState::Error(ref mut handler) => handler.on_response(res),
			_ => Next::end(),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		match self.status {
			FetchState::Done(_, _, ref mut handler) => handler.on_response_writable(encoder),
			FetchState::Error(ref mut handler) => handler.on_response_writable(encoder),
			_ => Next::end(),
		}
	}
}
