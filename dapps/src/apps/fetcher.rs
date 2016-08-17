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

use zip;
use zip::result::ZipError;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::collections::HashSet;
use rustc_serialize::hex::ToHex;
use util::{Address, FromHex, Mutex};

use hyper::Control;
use hyper::client::{Client};

use apps::manifest::{MANIFEST_FILENAME, deserialize_manifest, Manifest};
use endpoint::{EndpointPath, Handler};
use handlers::{Fetch, FetchResult};

#[derive(Debug)]
struct GithubApp {
	pub account: String,
	pub repo: String,
	pub commit: [u8;20],
	pub owner: Address,
}

impl GithubApp {
	pub fn url(&self) -> String {
		// format!("https://github.com/{}/{}/archive/{}.zip", self.account, self.repo, self.commit.to_hex())
		format!("http://github.todr.me/{}/{}/zip/{}", self.account, self.repo, self.commit.to_hex())
	}

	fn commit(bytes: &[u8]) -> [u8;20] {
		let mut commit = [0; 20];
		for i in 0..20 {
			commit[i] = bytes[i];
		}
		commit
	}
}

pub struct AppFetcher {
	dapps_path: PathBuf,
	in_progress: Arc<Mutex<HashSet<String>>>,
}

impl AppFetcher {

	pub fn new(dapps_path: &str) -> Self {
		AppFetcher {
			dapps_path: PathBuf::from(dapps_path),
			in_progress: Arc::new(Mutex::new(HashSet::new())),
		}
	}

	fn resolve(&self, app_id: &str) -> Option<GithubApp> {
		// TODO [todr] use GithubHint contract to check the details
		// For now we are just accepting patterns: <commithash>.<repo>.<account>.parity

		let mut app_parts = app_id.split('.');
		let hash = app_parts.next().and_then(|h| h.from_hex().ok());
		let repo = app_parts.next();
		let account = app_parts.next();

		match (hash, repo, account) {
			(Some(hash), Some(repo), Some(account)) => {
				Some(GithubApp {
					account: account.into(),
					repo: repo.into(),
					commit: GithubApp::commit(&hash),
					owner: Address::default(),
				})
			},
			_ => None,
		}
	}

	pub fn can_resolve(&self, app_id: &str) -> bool {
		self.resolve(app_id).is_some()
	}

	pub fn to_handler(&self, path: EndpointPath, control: Control) -> Box<Handler> {
		{
			let mut in_progress = self.in_progress.lock();
			if in_progress.contains(&path.app_id) {
				return Box::new(ContentHandler::html(
					StatusCode::ServiceUnavailable,
					"<h1>This dapp is already being downloaded.</h1>".into()
				));
			}
			in_progress.insert(path.app_id.clone());
		}

		let app = self.resolve(&path.app_id).expect("to_handler is called only when `can_resolve` returns true.");
		let client = Client::new().expect("Failed to create a Client");
		let in_progress = self.in_progress.clone();
		let app_id = path.app_id.clone();
		Box::new(AppFetcherHandler {
			dapps_path: self.dapps_path.clone(),
			control: Some(control),
			status: FetchState::NotStarted(app),
			client: Some(client),
			done: Some(move || {
				in_progress.lock().remove(&app_id);
			})
		})
	}

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

#[derive(Debug)]
enum ValidationError {
	ManifestNotFound,
	Io(io::Error),
	Zip(ZipError),
}

impl From<io::Error> for ValidationError {
	fn from(err: io::Error) -> Self {
		ValidationError::Io(err)
	}
}

impl From<ZipError> for ValidationError {
	fn from(err: ZipError) -> Self {
		ValidationError::Zip(err)
	}
}

fn find_manifest(zip: &mut zip::ZipArchive<fs::File>) -> Result<(Manifest, PathBuf), ValidationError> {
	for i in 0..zip.len() {
		let mut file = try!(zip.by_index(i));

		if !file.name().ends_with(MANIFEST_FILENAME) {
			continue;
		}

		// try to read manifest
		let mut manifest = String::new();
		let manifest = file
				.read_to_string(&mut manifest).ok()
				.and_then(|_| deserialize_manifest(manifest).ok());
		if let Some(manifest) = manifest {
			let mut manifest_location = PathBuf::from(file.name());
			manifest_location.pop(); // get rid of filename
			return Ok((manifest, manifest_location));
		}
	}
	return Err(ValidationError::ManifestNotFound);
}

fn validate_and_install_app(mut target: PathBuf, app_path: PathBuf) -> Result<String, ValidationError> {
	trace!(target: "dapps", "Opening dapp bundle at {:?}", app_path);
	let file = try!(fs::File::open(app_path));
	// Unpack archive
	let mut zip = try!(zip::ZipArchive::new(file));
	// First find manifest file
	let (manifest, manifest_dir) = try!(find_manifest(&mut zip));
	target.push(&manifest.id);

	// Remove old directory
	if target.exists() {
		warn!(target: "dapps", "Overwriting existing dapp: {}", manifest.id);
		try!(fs::remove_dir_all(target.clone()));
	}

	// Unpack zip
	for i in 0..zip.len() {
		let mut file = try!(zip.by_index(i));
		// TODO [todr] Check if it's consistent on windows.
		let is_dir = file.name().chars().rev().next() == Some('/');

		let file_path = PathBuf::from(file.name());
		let location_in_manifest_base = file_path.strip_prefix(&manifest_dir);
		// Create files that are inside manifest directory
		if let Ok(location_in_manifest_base) = location_in_manifest_base {
			let p = target.join(location_in_manifest_base);
			// Check if it's a directory
			if is_dir {
				try!(fs::create_dir_all(p));
			} else {
				let mut target = try!(fs::File::create(p));
				try!(io::copy(&mut file, &mut target));
			}
		}
	}

	Ok(manifest.id)
}

use std::time::{Instant, Duration};

use hyper::{header, server, Decoder, Encoder, Next, Method};
use hyper::net::HttpStream;
use hyper::status::StatusCode;

use handlers::ContentHandler;
use apps::DAPPS_DOMAIN;

enum FetchState {
	NotStarted(GithubApp),
	Error(ContentHandler),
	InProgress {
		deadline: Instant,
		receiver: mpsc::Receiver<FetchResult>
	},
	Done(String),
}

const FETCH_TIMEOUT: u64 = 30;

struct AppFetcherHandler<F: Fn() -> ()> {
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
	fn close_client(client: &mut Option<Client<Fetch>>) {
		client.take()
			.expect("After client is closed we are going into write, hence we can never close it again")
			.close();
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
					let fetch = fetch_app(self.client.as_mut().expect("on_request is called before client is closed."), app, control);
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
						let state = match validate_and_install_app(self.dapps_path.clone(), path) {
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

// 1. [x] Wait for response (with some timeout)
// 2. Validate (Check hash)
// 3. [x] Unpack to ~/.parity/dapps
// 4. [x] Display errors or refresh to load again from memory / FS
// 5. Mark as volatile?
//    Keep a list of "installed" apps?
//    Serve from memory?
//
// 6. Hosts validation?
// 7. [x] Mutex on dapp
