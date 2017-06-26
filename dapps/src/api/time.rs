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

//! Checks node's time drift.
//! Fires an API call to a service returning server's UTC time (in millis).
//! Then we compare the value of local clock setting with the server one (trying to account for network latency as
//! well).

use std::io::{self, Read};
use std::time;

use futures::{self, Future, BoxFuture};
use fetch::{self, Fetch};
use util::{Arc, RwLock};

/// Time checker error.
#[derive(Debug, Clone)]
pub enum Error {
	/// The API returned unexpected status code.
	UnexpectedResponse(&'static str, String),
	/// Invalid response has been returned by the API.
	InvalidTime(String),
	/// There was an error when trying to reach the API.
	Fetch(String),
	/// IO error when reading API response.
	Io(String),
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self { Error::Io(format!("{:?}", err)) }
}

impl From<fetch::Error> for Error {
	fn from(err: fetch::Error) -> Self { Error::Fetch(format!("{:?}", err)) }
}

fn utc_timestamp_millis() -> i64 {
	let time = ::time::now_utc().to_timespec();

	1_000 * time.sec + time.nsec as i64 / 1_000_000
}

const UPDATE_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
/// A time checker.
pub struct TimeChecker<F> {
	api_endpoint: String,
	fetch: F,
	last_result: Arc<RwLock<(time::Instant, Result<i64, Error>)>>,
}

impl<F: Fetch> TimeChecker<F> {

	/// Creates new time checker given API endpoint URL and `Fetch` instance.
	pub fn new(api_endpoint: String, fetch: F) -> Self {
		let last_result = Arc::new(RwLock::new(
			(time::Instant::now(), Err(Error::Fetch("API unavailable.".into())))
		));

		TimeChecker {
			api_endpoint,
			fetch,
			last_result,
		}
	}

	/// Updates the time
	pub fn update(&self) -> BoxFuture<i64, Error> {
		let last_result = self.last_result.clone();
		self.fetch_time().then(move |res| {
			*last_result.write() = (time::Instant::now(), res.clone());
			res
		}).boxed()
	}

	fn fetch_time(&self) -> BoxFuture<i64, Error>{
		let start = utc_timestamp_millis();

		self.fetch.process(self.fetch.fetch(&self.api_endpoint)
			.map_err(|err| Error::Fetch(format!("{:?}", err)))
			.and_then(move |mut response| {
				let mut result = String::new();
				response.read_to_string(&mut result)?;

				if !response.is_success() {
					let status = response.status().canonical_reason().unwrap_or("unknown");
					return Err(Error::UnexpectedResponse(status, result));
				}

				let server_time: i64 = match result.parse() {
					Ok(time) => time,
					Err(err) => {
						return Err(Error::InvalidTime(format!("{}", err)));
					}
				};
				let end = utc_timestamp_millis();
				let rough_latency = (end - start) / 2;

				let diff = server_time - start - rough_latency;

				Ok(diff)
			})
		)
	}

	/// Returns a current time drift or error if last request to time API failed.
	pub fn time_drift(&self) -> BoxFuture<i64, Error> {
		// return cached result
		{
			let res = self.last_result.read();
			if res.0.elapsed() < time::Duration::from_secs(UPDATE_TIMEOUT_SECS) {
				return futures::done(res.1.clone()).boxed();
			}
		}
		// or update and return result
		self.update()
	}
}
