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
#[derive(Debug, Clone, PartialEq)]
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


/// Time provider.
pub trait TimeProvider: Clone + Send + 'static {
	/// Returns an instance of this provider.
	fn new() -> Self where Self: Sized;
	/// Returns current time in milliseconds.
	fn utc_timestamp_millis(&self) -> i64;
}
/// Default system time provider.
#[derive(Clone)]
pub struct StdTimeProvider;
impl TimeProvider for StdTimeProvider {
	fn new() -> Self where Self: Sized { StdTimeProvider }
	fn utc_timestamp_millis(&self) -> i64 {
		let time = ::time::now_utc().to_timespec();

		1_000 * time.sec + time.nsec as i64 / 1_000_000
	}
}

const UPDATE_TIMEOUT_OK_SECS: u64 = 30;
const UPDATE_TIMEOUT_ERR_SECS: u64 = 2;

#[derive(Debug, Clone)]
/// A time checker.
pub struct TimeChecker<F, T: TimeProvider = StdTimeProvider> {
	api_endpoint: String,
	fetch: F,
	last_result: Arc<RwLock<(time::Instant, Result<i64, Error>)>>,
	time: T,
}

impl<F: Fetch, T: TimeProvider> TimeChecker<F, T> {

	/// Creates new time checker given API endpoint URL and `Fetch` instance.
	pub fn new(api_endpoint: String, fetch: F) -> Self {
		let last_result = Arc::new(RwLock::new(
			(time::Instant::now(), Err(Error::Fetch("API unavailable.".into())))
		));

		TimeChecker {
			api_endpoint,
			fetch,
			last_result,
			time: T::new(),
		}
	}

	/// Updates the time
	pub fn update(&self) -> BoxFuture<i64, Error> {
		let last_result = self.last_result.clone();
		self.fetch_time().then(move |res| {
			let valid_till = time::Instant::now() + time::Duration::from_secs(
				if res.is_ok() { UPDATE_TIMEOUT_OK_SECS } else { UPDATE_TIMEOUT_ERR_SECS }
			);

			*last_result.write() = (valid_till, res.clone());
			res
		}).boxed()
	}

	fn fetch_time(&self) -> BoxFuture<i64, Error>{
		let start = self.time.utc_timestamp_millis();
		let time = self.time.clone();
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
				let end = time.utc_timestamp_millis();
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
			if res.0 > time::Instant::now() {
				return futures::done(res.1.clone()).boxed();
			}
		}
		// or update and return result
		self.update()
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::cell::RefCell;
	use fetch::{self, Fetch};
	use futures::{self, Future};
	use futures::future::FutureResult;
	use super::{TimeProvider, TimeChecker, Error};
	use util::Mutex;

	#[derive(Clone)]
	struct FakeFetch(bool, Arc<Mutex<u64>>);
	impl Fetch for FakeFetch {
		type Result = FutureResult<fetch::Response, fetch::Error>;
		fn new() -> Result<Self, fetch::Error> where Self: Sized { Ok(FakeFetch(false, Default::default())) }
		fn fetch_with_abort(&self, url: &str, _abort: fetch::Abort) -> Self::Result {
			assert_eq!(url, "https://time.parity.io/api");
			let mut val = self.1.lock();
			*val = *val + 1;
			if self.0 {
				futures::future::ok(fetch::Response::not_found())
			} else {
				let data = ::std::io::Cursor::new(b"1".to_vec());
				futures::future::ok(fetch::Response::from_reader(data))
			}
		}
	}
	#[derive(Clone)]
	struct FakeTime(RefCell<Vec<i64>>);
	impl TimeProvider for FakeTime {
		fn new() -> Self where Self: Sized { FakeTime(RefCell::new(vec![150, 0])) }
		fn utc_timestamp_millis(&self) -> i64 {
			self.0.borrow_mut().pop().expect("Expecting only two calls to utc_timestamp_millis.")
		}
	}

	fn time_checker() -> TimeChecker<FakeFetch, FakeTime> {
		TimeChecker::new("https://time.parity.io/api".into(), FakeFetch::new().unwrap())
	}

	#[test]
	fn should_fetch_time_on_start() {
		// given
		let time = time_checker();

		// when
		let diff = time.time_drift().wait().unwrap();

		// then
		assert_eq!(diff, 1 - 150 / 2);
		assert_eq!(*time.fetch.1.lock(), 1);
	}

	#[test]
	fn should_not_fetch_twice_if_timeout_has_not_passed() {
		// given
		let time = time_checker();

		// when
		let diff1 = time.time_drift().wait().unwrap();
		let diff2 = time.time_drift().wait().unwrap();

		// then
		assert_eq!(diff1, 1 - 150 / 2);
		assert_eq!(diff2, 1 - 150 / 2);
		assert_eq!(*time.fetch.1.lock(), 1);
	}

	#[test]
	fn should_return_error_if_response_is_invalid() {
		// given
		let mut time = time_checker();
		time.fetch.0 = true;

		// when
		let err = time.time_drift().wait();

		// then
		assert_eq!(err, Err(Error::UnexpectedResponse("Not Found", "".to_owned())));
		assert_eq!(*time.fetch.1.lock(), 1);
	}

}
