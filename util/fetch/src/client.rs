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

//! Fetching

use std::{env, io};
use std::sync::{mpsc, Arc};
use std::sync::atomic::AtomicBool;
use std::path::PathBuf;

use hyper;
use https_fetch as https;

use fetch_file::{FetchHandler, Error as HttpFetchError};

pub type FetchResult = Result<PathBuf, FetchError>;

#[derive(Debug)]
pub enum FetchError {
	InvalidUrl,
	Http(HttpFetchError),
	Https(https::FetchError),
	Io(io::Error),
	Other(String),
}

impl From<HttpFetchError> for FetchError {
	fn from(e: HttpFetchError) -> Self {
		FetchError::Http(e)
	}
}

impl From<io::Error> for FetchError {
	fn from(e: io::Error) -> Self {
		FetchError::Io(e)
	}
}

pub trait Fetch: Default + Send {
	/// Fetch URL and get the result in callback.
	fn request_async(&mut self, url: &str, abort: Arc<AtomicBool>, on_done: Box<Fn(FetchResult) + Send>) -> Result<(), FetchError>;

	/// Fetch URL and get a result Receiver. You will be notified when receiver is ready by `on_done` callback.
	fn request(&mut self, url: &str, abort: Arc<AtomicBool>, on_done: Box<Fn() + Send>) -> Result<mpsc::Receiver<FetchResult>, FetchError> {
		let (tx, rx) = mpsc::channel();
		try!(self.request_async(url, abort, Box::new(move |result| {
			let res = tx.send(result);
			if let Err(_) = res {
				warn!("Fetch finished, but no one was listening");
			}
			on_done();
		})));
		Ok(rx)
	}

	/// Closes this client
	fn close(self) {}

	/// Returns a random filename
	fn random_filename() -> String {
		use ::rand::Rng;
		let mut rng = ::rand::OsRng::new().unwrap();
		rng.gen_ascii_chars().take(12).collect()
	}
}

pub struct Client {
	http_client: hyper::Client<FetchHandler>,
	https_client: https::Client,
	limit: Option<usize>,
}

impl Default for Client {
	fn default() -> Self {
		// Max 15MB will be downloaded.
		Client::with_limit(Some(15*1024*1024))
	}
}

impl Client {
	fn with_limit(limit: Option<usize>) -> Self {
		Client {
			http_client: hyper::Client::new().expect("Unable to initialize http client."),
			https_client: https::Client::with_limit(limit).expect("Unable to initialize https client."),
			limit: limit,
		}
	}

	fn convert_url(url: hyper::Url) -> Result<https::Url, FetchError> {
		let host = format!("{}", try!(url.host().ok_or(FetchError::InvalidUrl)));
		let port = try!(url.port_or_known_default().ok_or(FetchError::InvalidUrl));
		https::Url::new(&host, port, url.path()).map_err(|_| FetchError::InvalidUrl)
	}

	fn temp_path() -> PathBuf {
		let mut dir = env::temp_dir();
		dir.push(Self::random_filename());
		dir
	}
}

impl Fetch for Client {
	fn close(self) {
		self.http_client.close();
		self.https_client.close();
	}

	fn request_async(&mut self, url: &str, abort: Arc<AtomicBool>, on_done: Box<Fn(FetchResult) + Send>) -> Result<(), FetchError> {
		let is_https = url.starts_with("https://");
		let url = try!(url.parse().map_err(|_| FetchError::InvalidUrl));
		let temp_path = Self::temp_path();

		trace!(target: "fetch", "Fetching from: {:?}", url);

		if is_https {
			let url = try!(Self::convert_url(url));
			try!(self.https_client.fetch_to_file(
				url,
				temp_path.clone(),
				abort,
				move |result| on_done(result.map(|_| temp_path).map_err(FetchError::Https)),
			).map_err(|e| FetchError::Other(format!("{:?}", e))));
		} else {
			try!(self.http_client.request(
				url,
				FetchHandler::new(temp_path, abort, Box::new(move |result| on_done(result)), self.limit.map(|v| v as u64).clone()),
			).map_err(|e| FetchError::Other(format!("{:?}", e))));
		}

		Ok(())
	}
}

