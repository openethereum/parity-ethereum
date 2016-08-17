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


use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Instant, Duration};

use hyper::{header, server, Decoder, Encoder, Next, Method, Control, Client};
use hyper::net::HttpStream;
use hyper::status::StatusCode;

use handlers::ContentHandler;
use handlers::client::{Fetch, FetchResult};
use apps::DAPPS_DOMAIN;
use apps::fetcher::{GithubApp, validate_and_install_app};

const FETCH_TIMEOUT: u64 = 30;

enum FetchState {
	NotStarted(GithubApp),
	Error(ContentHandler),
	InProgress {
		deadline: Instant,
		receiver: mpsc::Receiver<FetchResult>
	},
	Done(String),
}

pub struct AppFetcherHandler<F: Fn() -> ()> {
	dapps_path: PathBuf,
	control: Option<Control>,
	status: FetchState,
	client: Option<Client<Fetch>>,
	done: Option<F>,
}

impl<F: Fn() -> ()> Drop for AppFetcherHandler<F> {
	fn drop(&mut self) {
		self.done.take().unwrap()();
	}
}

impl<F: Fn() -> ()> AppFetcherHandler<F> {

	pub fn new(
		app: GithubApp,
		dapps_path: PathBuf,
		control: Control,
		done: F) -> Self {

		let client = Client::new().expect("Failed to create a Client");
		AppFetcherHandler {
			dapps_path: dapps_path,
			control: Some(control),
			client: Some(client),
			status: FetchState::NotStarted(app),
			done: Some(done),
		}
	}

	fn close_client(client: &mut Option<Client<Fetch>>) {
		client.take()
			.expect("After client is closed we are going into write, hence we can never close it again")
			.close();
	}


	// TODO [todr] https support
	fn fetch_app(client: &mut Client<Fetch>, app: &GithubApp, control: Control) -> Result<mpsc::Receiver<FetchResult>, String> {
		let url = try!(app.url().parse().map_err(|e| format!("{:?}", e)));
		trace!(target: "dapps", "Fetching from: {:?}", url);

		let (tx, rx) = mpsc::channel();
		let res = client.request(url, Fetch::new(tx, Box::new(move || {
			trace!(target: "dapps", "Fetching finished.");
			// Ignoring control errors
			let _ = control.ready(Next::read());
		})));
		match res {
			Ok(_) => Ok(rx),
			Err(e) => Err(format!("{:?}", e)),
		}
	}
}

impl<F: Fn() -> ()> server::Handler<HttpStream> for AppFetcherHandler<F> {
	fn on_request(&mut self, request: server::Request<HttpStream>) -> Next {
		let status = if let FetchState::NotStarted(ref app) = self.status {
			Some(match *request.method() {
				// Start fetching content
				Method::Get => {
					trace!(target: "dapps", "Fetching dapp: {:?}", app);
					let control = self.control.take().expect("on_request is called only once, thus control is always Some");
					let client = self.client.as_mut().expect("on_request is called before client is closed.");
					let fetch = Self::fetch_app(client, app, control);
					match fetch {
						Ok(receiver) => FetchState::InProgress {
							deadline: Instant::now() + Duration::from_secs(FETCH_TIMEOUT),
							receiver: receiver,
						},
						Err(e) => FetchState::Error(ContentHandler::html(
							StatusCode::BadGateway,
							format!("<h1>Error starting dapp download.</h1><pre>{}</pre>", e),
						)),
					}
				},
				// or return error
				_ => FetchState::Error(ContentHandler::html(
					StatusCode::MethodNotAllowed,
					"<h1>Only <code>GET</code> requests are allowed.</h1>".into(),
				)),
			})
		} else { None };

		if let Some(status) = status {
			self.status = status;
		}

		Next::read()
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<HttpStream>) -> Next {
		let (status, next) = match self.status {
			// Request may time out
			FetchState::InProgress { ref deadline, .. } if *deadline < Instant::now() => {
				trace!(target: "dapps", "Fetching dapp failed because of timeout.");
				let timeout = ContentHandler::html(
					StatusCode::GatewayTimeout,
					format!("<h1>Could not fetch app bundle within {} seconds.</h1>", FETCH_TIMEOUT),
					);
				Self::close_client(&mut self.client);
				(Some(FetchState::Error(timeout)), Next::write())
			},
			FetchState::InProgress { ref receiver, .. } => {
				// Check if there is an answer
				let rec = receiver.try_recv();
				match rec {
					// Unpack and validate
					Ok(Ok(path)) => {
						trace!(target: "dapps", "Fetching dapp finished. Starting validation.");
						Self::close_client(&mut self.client);
						// Unpack and verify
						let state = match validate_and_install_app(self.dapps_path.clone(), path.clone()) {
							Err(e) => {
								trace!(target: "dapps", "Error while validating dapp: {:?}", e);
								FetchState::Error(ContentHandler::html(
									StatusCode::BadGateway,
									format!("<h1>Downloaded bundle does not contain valid app.</h1><pre>{:?}</pre>", e),
								))
							},
							Ok(id) => FetchState::Done(id)
						};
						// Remove temporary zip file
						let _ = fs::remove_file(path);
						(Some(state), Next::write())
					},
					Ok(Err(e)) => {
						warn!(target: "dapps", "Unable to fetch new dapp: {:?}", e);
						let error = ContentHandler::html(
							StatusCode::BadGateway,
							"<h1>There was an error when fetching the dapp.</h1>".into(),
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
			self.status = status;
		}

		next
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.status {
			FetchState::Done(ref id) => {
				trace!(target: "dapps", "Fetching dapp finished. Redirecting to {}", id);
				res.set_status(StatusCode::Found);
				// TODO [todr] should detect if its using nice-urls
				res.headers_mut().set(header::Location(format!("http://{}{}", id, DAPPS_DOMAIN)));
				Next::write()
			},
			FetchState::Error(ref mut handler) => handler.on_response(res),
			_ => Next::end(),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		match self.status {
			FetchState::Error(ref mut handler) => handler.on_response_writable(encoder),
			_ => Next::end(),
		}
	}
}

