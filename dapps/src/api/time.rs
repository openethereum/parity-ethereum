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

use time;
use fetch::{self, Fetch};

/// Time checker error.
#[derive(Debug)]
pub enum Error {
	/// The API returned unexpected status code.
	UnexpectedResponse(&'static str, String),
	/// Invalid response has been returned by the API.
	InvalidTime(String),
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

fn utc_timestamp_millis() -> i64 {
	let time = time::now_utc().to_timespec();

	1_000 * time.sec + time.nsec as i64 / 1_000_000
}

#[derive(Debug)]
/// A time checker.
pub struct TimeChecker<F> {
	api_endpoint: String,
	fetch: F,
}

impl<F: Fetch> TimeChecker<F> {

	pub fn new(api_endpoint: String, fetch: F) -> Self {
		TimeChecker {
			api_endpoint,
			fetch,
		}
	}

	pub fn time_drift(&self) -> Result<i64, Error> {
		let start = utc_timestamp_millis();
		let mut response = self.fetch.fetch_sync(&self.api_endpoint)?;
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

		return Ok(diff);
	}
}
