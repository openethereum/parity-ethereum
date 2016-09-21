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

//! Hyper Client Handlers

pub mod fetch_file;

use std::env;
use std::sync::{mpsc, Arc};
use std::sync::atomic::AtomicBool;
use std::path::PathBuf;

use hyper;
use https_fetch as https;

use random_filename;
use self::fetch_file::{Fetch, Error as HttpFetchError};

pub type FetchResult = Result<PathBuf, FetchError>;

#[derive(Debug)]
pub enum FetchError {
	InvalidUrl,
	Http(HttpFetchError),
	Https(https::FetchError),
	Other(String),
}

impl From<HttpFetchError> for FetchError {
	fn from(e: HttpFetchError) -> Self {
		FetchError::Http(e)
	}
}

pub struct Client {
	http_client: hyper::Client<Fetch>,
	https_client: https::Client,
}

impl Client {
	pub fn new() -> Self {
		Client {
			http_client: hyper::Client::new().expect("Unable to initialize http client."),
			https_client: https::Client::new().expect("Unable to initialize https client."),
		}
	}

	pub fn close(self) {
		self.http_client.close();
		self.https_client.close();
	}

	pub fn request(&mut self, url: &str, abort: Arc<AtomicBool>, on_done: Box<Fn() + Send>) -> Result<mpsc::Receiver<FetchResult>, FetchError> {
		let is_https = url.starts_with("https://");
		let url = try!(url.parse().map_err(|_| FetchError::InvalidUrl));
		trace!(target: "dapps", "Fetching from: {:?}", url);
		if is_https {
			let url = try!(Self::convert_url(url));

			let (tx, rx) = mpsc::channel();
			let temp_path = Self::temp_path();
			let res = self.https_client.fetch_to_file(url, temp_path.clone(), abort, move |result| {
				let res = tx.send(
					result.map(|_| temp_path).map_err(FetchError::Https)
				);
				if let Err(_) = res {
					warn!("Fetch finished, but no one was listening");
				}
				on_done();
			});

			match res {
				Ok(_) => Ok(rx),
				Err(e) => Err(FetchError::Other(format!("{:?}", e))),
			}
		} else {
			let (tx, rx) = mpsc::channel();
			let res = self.http_client.request(url, Fetch::new(tx, abort, on_done));

			match res {
				Ok(_) => Ok(rx),
				Err(e) => Err(FetchError::Other(format!("{:?}", e))),
			}
		}
	}

	fn convert_url(url: hyper::Url) -> Result<https::Url, FetchError> {
		let host = format!("{}", try!(url.host().ok_or(FetchError::InvalidUrl)));
		let port = try!(url.port_or_known_default().ok_or(FetchError::InvalidUrl));
		https::Url::new(&host, port, url.path()).map_err(|_| FetchError::InvalidUrl)
	}

	fn temp_path() -> PathBuf {
		let mut dir = env::temp_dir();
		dir.push(random_filename());
		dir
	}
}


