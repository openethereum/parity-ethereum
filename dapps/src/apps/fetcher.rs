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

use std::sync::mpsc;
use rustc_serialize::hex::ToHex;
use util::{Address, FromHex};

use hyper::Control;
use hyper::client::{Client};

use endpoint::{EndpointPath, Handler};
use handlers::{Fetch, FetchResult, FetchError};

struct GithubApp {
	pub account: String,
	pub repo: String,
	pub commit: [u8;20],
	pub owner: Address,
}

impl GithubApp {
	pub fn url(&self) -> String {
		// format!("https://github.com/{}/{}/archive/{}.zip", self.account, self.repo, self.commit.to_hex())
		format!("http://files.todr.me/{}.zip", self.commit.to_hex())
	}

	fn commit(bytes: &[u8]) -> [u8;20] {
		let mut commit = [0; 20];
		for i in 0..20 {
			commit[i] = bytes[i];
		}
		commit
	}
}

pub struct AppFetcher;

impl AppFetcher {

	fn resolve(&self, app_id: &str) -> Option<GithubApp> {
		// TODO [todr] use GithubHint contract to check the details
		Some(GithubApp {
			account: "ethcore".into(),
			repo: "daoclaim".into(),
			commit: GithubApp::commit(&"ec4c1fe06c808fe3739858c347109b1f5f1ed4b5".from_hex().unwrap()),
			owner: Address::default(),
		})
	}

	pub fn can_resolve(&self, app_id: &str) -> bool {
		self.resolve(app_id).is_some()
	}

	pub fn to_handler(&self, path: EndpointPath, control: Control) -> Box<Handler> {
		let app = self.resolve(&path.app_id).expect("to_handler is called only when `can_resolve` returns true.");
		let client = Client::new().expect("Failed to create a Client");
		Box::new(AppFetcherHandler {
			control: Some(control),
			status: FetchStatus::NotStarted(app),
			client: Some(client),
		})
	}

}

// TODO [todr] https support
fn fetch_app(client: &mut Client<Fetch>, app: &GithubApp, control: Control) -> mpsc::Receiver<FetchResult> {
	let (tx, rx) = mpsc::channel();
	let x = client.request(app.url().parse().expect("ValidURL"), Fetch::new(tx, Box::new(move || {
		// Ignoring control errors
		let _ = control.ready(Next::read());
	})));
	if let Ok(_) = x {
		rx
	} else {
		rx
	}
}

// fn validate_and_install_app(app: PathBuf) {
//
// }

use std::io::Write;
use std::time::{Instant, Duration};
use hyper::{header, server, Decoder, Encoder, Next, Method};
use hyper::net::HttpStream;
use hyper::status::StatusCode;
use handlers::ContentHandler;

enum FetchStatus {
	NotStarted(GithubApp),
	Error(ContentHandler),
	InProgress {
		deadline: Instant,
		receiver: mpsc::Receiver<FetchResult>
	},
	Done,
}

const FETCH_TIMEOUT: u64 = 30;

struct AppFetcherHandler {
	control: Option<Control>,
	status: FetchStatus,
	client: Option<Client<Fetch>>,
}

impl server::Handler<HttpStream> for AppFetcherHandler {
	fn on_request(&mut self, request: server::Request<HttpStream>) -> Next {
		let status = if let FetchStatus::NotStarted(ref app) = self.status {
			Some(match *request.method() {
				// Start fetching content
				Method::Get => {
					let control = self.control.take().expect("on_request is called only once, thus control is always defined");
					FetchStatus::InProgress {
						deadline: Instant::now() + Duration::from_secs(FETCH_TIMEOUT),
						receiver: fetch_app(self.client.as_mut().expect("on_request is called before client is closed."), app, control),
					}
				},
				// or return error
				_ => FetchStatus::Error(ContentHandler::new(
					StatusCode::MethodNotAllowed,
					"<h1>Only <code>GET</code> requests are allowed.</h1>".into(),
					"text/html".into()
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
			FetchStatus::InProgress { ref deadline, ref receiver } if *deadline < Instant::now() => {
				let timeout = ContentHandler::new(
					StatusCode::GatewayTimeout,
					format!("<h1>Could not fetch app bundle within {} seconds.</h1>", FETCH_TIMEOUT),
					"text/html".into()
					);
				(Some(FetchStatus::Error(timeout)), Next::write())
			},
			FetchStatus::InProgress { ref deadline, ref receiver } => {
				// Check if there is an answer
				let rec = receiver.try_recv();
				match rec {
					// Unpack and validate
					Ok(Ok(path)) => {
						self.client.take()
							.expect("After client is closed we are going into write, hence we can never close it again")
							.close();
						(Some(FetchStatus::Done), Next::write())
					},
					Ok(Err(e)) => {
						warn!("Unable to fetch new Dapp: {:?}", e);
						let error = ContentHandler::new(
							StatusCode::BadGateway,
							"<h1>There was an error when fetching the Dapp.".into(),
							"text/html".into(),
						);
						(Some(FetchStatus::Error(error)), Next::write())
					},
					// wait some more
					_ => (None, Next::wait())
				}
			},
			FetchStatus::Error(ref mut handler) => (None, handler.on_request_readable(decoder)),
			_ => (None, Next::write()),
		};

		if let Some(status) = status {
			self.status = status;
		}

		next
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.status {
			FetchStatus::Done => {
				res.set_status(StatusCode::Found);
				res.headers_mut().set(header::Location("todo".into()));
				Next::write()
			},
			FetchStatus::Error(ref mut handler) => handler.on_response(res),
			_ => Next::end(),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		match self.status {
			FetchStatus::Error(ref mut handler) => handler.on_response_writable(encoder),
			_ => Next::end(),
		}
	}
}

// 1. Wait for response (with some timeout)
// 2. Validate
// 3. Unpack to ~/.parity/dapps
// 4. Display errors or refresh to load again from memory / FS
// 5. Mark as volatile?
//    Keep a list of "installed" apps?
//    Serve from memory?
//
// 6. Hosts validation?
// 7. Mutex on dapp
