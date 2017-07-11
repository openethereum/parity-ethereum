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

use rustc_serialize::json::Json;
use std::cmp;
use std::fmt;
use std::io;
use std::io::Read;
use std::str::FromStr;

use fetch;
use fetch::{Client as FetchClient, Fetch};
use futures::Future;

#[derive(Debug)]
pub struct PriceInfo {
	pub ethusd: f32,
}

/// Price info error.
#[derive(Debug)]
pub enum Error {
	/// The API returned an unexpected status code or content.
	UnexpectedResponse(&'static str, String),
	/// There was an error when trying to reach the API.
	Fetch(fetch::Error),
	/// IO error when reading API response.
	Io(io::Error),
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self { Error::Io(err) }
}

impl From<fetch::Error> for Error {
	fn from(err: fetch::Error) -> Self { Error::Fetch(err) }
}

pub struct Client {
	api_endpoint: String,
	fetch: FetchClient,
}

impl fmt::Debug for Client {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "price_info::Client {{ api_endpoint: {} }}", self.api_endpoint)
	}
}

impl cmp::PartialEq for Client {
	fn eq(&self, other: &Client) -> bool {
		self.api_endpoint == other.api_endpoint
	}
}

impl Client {
	pub fn new() -> Client {
		let api_endpoint = "http://api.etherscan.io/api?module=stats&action=ethprice".to_owned();
		let fetch = FetchClient::new().expect("Error starting fetch client");
		Client { api_endpoint, fetch }
	}

	pub fn get<F: Fn(PriceInfo) + Sync + Send + 'static>(&self, set_price: F) {
		self.fetch.forget(self.fetch.fetch(&self.api_endpoint)
			.map_err(|err| Error::Fetch(err))
			.and_then(move |mut response| {
				let mut result = String::new();
				response.read_to_string(&mut result)?;

				if response.is_success() {
					if let Ok(json) = Json::from_str(&result) {
						let obj = json.find_path(&["result", "ethusd"]).and_then(|obj| {
							match *obj {
								Json::String(ref s) => FromStr::from_str(s).ok(),
								_ => None,
							}
						});

						if let Some(ethusd) = obj {
							set_price(PriceInfo { ethusd });
							return Ok(());
						}
					}
				}

				let status = response.status().canonical_reason().unwrap_or("unknown");
				Err(Error::UnexpectedResponse(status, result))
			})
		   .map_err(|err| {
			   warn!("Failed to auto-update latest ETH price: {:?}", err);
			   err
		   })
		);
	}
}

#[test] #[ignore]
fn should_get_price_info() {
	use std::sync::Arc;
	use std::time::Duration;
	use ethcore_logger::init_log;
	use util::{Condvar, Mutex};

	init_log();
	let done = Arc::new((Mutex::new(PriceInfo { ethusd: 0f32 }), Condvar::new()));
	let rdone = done.clone();
	let price_info = Client::new();

	price_info.get(move |price| { let mut p = rdone.0.lock(); *p = price; rdone.1.notify_one(); });
	let mut p = done.0.lock();
	let t = done.1.wait_for(&mut p, Duration::from_millis(10000));
	assert!(!t.timed_out());
	assert!(p.ethusd != 0f32);
}
