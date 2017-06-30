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

//! Periodically checks node's time drift using [SNTP](https://tools.ietf.org/html/rfc1769).
//!
//! An NTP packet is sent to the server with a local timestamp, the server then completes the packet, yielding the
//! following timestamps:
//!
//!    Timestamp Name          ID   When Generated
//!   ------------------------------------------------------------
//!    Originate Timestamp     T1   time request sent by client
//!    Receive Timestamp       T2   time request received at server
//!    Transmit Timestamp      T3   time reply sent by server
//!    Destination Timestamp   T4   time reply received at client
//!
//! The drift is defined as:
//!
//! drift = ((T2 - T1) + (T3 - T4)) / 2.
//!

use std::io;
use std::{fmt, time};

use futures::{self, Future, BoxFuture};
use futures_cpupool::CpuPool;
use ntp;
use time::{Duration, Timespec};
use util::{Arc, RwLock};

/// Time checker error.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
	/// There was an error when trying to reach the NTP server.
	Ntp(String),
	/// IO error when reading NTP response.
	Io(String),
}

impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::Error::*;

		match *self {
			Ntp(ref err) => write!(fmt, "NTP error: {}", err),
			Io(ref err) => write!(fmt, "Connection Error: {}", err),
		}
	}
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self { Error::Io(format!("{}", err)) }
}

impl From<ntp::errors::Error> for Error {
	fn from(err: ntp::errors::Error) -> Self { Error::Ntp(format!("{}", err)) }
}

/// Time provider.
pub trait TimeProvider: Clone + Send + 'static {
	/// Returns an instance of this provider.
	fn new() -> Self where Self: Sized;
	/// Returns current time.
	fn now(&self) -> Timespec;
}
/// Default system time provider.
#[derive(Clone)]
pub struct StdTimeProvider;
impl TimeProvider for StdTimeProvider {
	fn new() -> Self where Self: Sized { StdTimeProvider }
	fn now(&self) -> Timespec {
		::time::now_utc().to_timespec()
	}
}

/// NTP time drift checker.
pub trait Ntp {
	/// Returns the current time drift.
	fn drift(&self) -> BoxFuture<Duration, Error>;
}

/// NTP client using the SNTP algorithm for calculating drift.
#[derive(Clone)]
pub struct SimpleNtp<T> {
	address: Arc<String>,
	time_provider: T,
	pool: CpuPool,
}

impl<T> fmt::Debug for SimpleNtp<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Ntp {{ address: {} }}", self.address)
	}
}

impl<T: TimeProvider> SimpleNtp<T> {
	fn new(address: &str, time_provider: T) -> SimpleNtp<T> {
		SimpleNtp {
			address: Arc::new(address.to_owned()),
			time_provider: time_provider,
			pool: CpuPool::new(4),
		}
	}
}

impl<T: TimeProvider> Ntp for SimpleNtp<T> {
	fn drift(&self) -> BoxFuture<Duration, Error> {
		let address = self.address.clone();
		let time_provider = self.time_provider.clone();
		self.pool.spawn_fn(move || {
			let packet = ntp::request(&*address)?;
			let dest_time = time_provider.now();
			let orig_time = Timespec::from(packet.orig_time);
			let recv_time = Timespec::from(packet.recv_time);
			let transmit_time = Timespec::from(packet.transmit_time);

			let drift = ((recv_time - orig_time) + (transmit_time - dest_time)) / 2;

			Ok(drift)
		}).boxed()
	}
}

const UPDATE_TIMEOUT_OK_SECS: u64 = 30;
const UPDATE_TIMEOUT_ERR_SECS: u64 = 2;

#[derive(Debug, Clone)]
/// A time checker.
pub struct TimeChecker<N: Ntp = SimpleNtp<StdTimeProvider>> {
	ntp: N,
	last_result: Arc<RwLock<(time::Instant, Result<i64, Error>)>>,
}

impl TimeChecker<SimpleNtp<StdTimeProvider>> {
	/// Creates new time checker given the NTP server address.
	pub fn new(ntp_address: String) -> Self {
		let last_result = Arc::new(RwLock::new(
			(time::Instant::now(), Err(Error::Ntp("NTP server unavailable.".into())))
		));

		let ntp = SimpleNtp::new(&ntp_address, StdTimeProvider::new());

		TimeChecker {
			ntp,
			last_result,
		}
	}
}

impl<N: Ntp> TimeChecker<N> {
	/// Updates the time
	pub fn update(&self) -> BoxFuture<i64, Error> {
		let last_result = self.last_result.clone();
		self.ntp.drift().then(move |res| {
			let valid_till = time::Instant::now() + time::Duration::from_secs(
				if res.is_ok() { UPDATE_TIMEOUT_OK_SECS } else { UPDATE_TIMEOUT_ERR_SECS }
			);

			let res = res.map(|d| d.num_milliseconds());
			*last_result.write() = (valid_till, res.clone());
			res
		}).boxed()
	}

	/// Returns a current time drift or error if last request to NTP server failed.
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
	use std::time::Instant;
	use time::Duration;
	use futures::{self, BoxFuture, Future};
	use super::{Ntp, TimeChecker, Error};
	use util::{Mutex, RwLock};

	#[derive(Clone)]
	struct FakeNtp(RefCell<Vec<Duration>>, Arc<Mutex<u64>>);
	impl FakeNtp {
		fn new() -> FakeNtp {
			FakeNtp(
				RefCell::new(vec![Duration::milliseconds(150)]),
				Arc::new(Mutex::new(0)))
		}
	}

	impl Ntp for FakeNtp {
		fn drift(&self) -> BoxFuture<Duration, Error> {
			let mut val = self.1.lock();
			*val = *val + 1;
			futures::future::ok(self.0.borrow_mut().pop().expect("Expecting only one call to now().")).boxed()
		}
	}

	fn time_checker() -> TimeChecker<FakeNtp> {
		let last_result = Arc::new(RwLock::new(
			(Instant::now(), Err(Error::Ntp("NTP server unavailable.".into())))
		));

		TimeChecker {
			ntp: FakeNtp::new(),
			last_result: last_result,
		}
	}

	#[test]
	fn should_fetch_time_on_start() {
		// given
		let time = time_checker();

		// when
		let diff = time.time_drift().wait().unwrap();

		// then
		assert_eq!(diff, 150);
		assert_eq!(*time.ntp.1.lock(), 1);
	}

	#[test]
	fn should_not_fetch_twice_if_timeout_has_not_passed() {
		// given
		let time = time_checker();

		// when
		let diff1 = time.time_drift().wait().unwrap();
		let diff2 = time.time_drift().wait().unwrap();

		// then
		assert_eq!(diff1, 150);
		assert_eq!(diff2, 150);
		assert_eq!(*time.ntp.1.lock(), 1);
	}
}
