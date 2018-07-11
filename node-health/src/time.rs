// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
use std::{fmt, mem, time};
use std::collections::VecDeque;
use std::sync::atomic::{self, AtomicUsize};
use std::sync::Arc;

use futures::{self, Future};
use futures::future::{self, IntoFuture};
use futures_cpupool::{CpuPool, CpuFuture};
use ntp;
use parking_lot::RwLock;
use time_crate::{Duration, Timespec};

/// Time checker error.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
	/// No servers are currently available for a query.
	NoServersAvailable,
	/// There was an error when trying to reach the NTP server.
	Ntp(String),
	/// IO error when reading NTP response.
	Io(String),
}

impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::Error::*;

		match *self {
			NoServersAvailable => write!(fmt, "No NTP servers available"),
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

/// NTP time drift checker.
pub trait Ntp {
	/// Returned Future.
	type Future: IntoFuture<Item=Duration, Error=Error>;

	/// Returns the current time drift.
	fn drift(&self) -> Self::Future;
}

const SERVER_MAX_POLL_INTERVAL_SECS: u64 = 60;
#[derive(Debug)]
struct Server {
	pub address: String,
	next_call: RwLock<time::Instant>,
	failures: AtomicUsize,
}

impl Server {
	pub fn is_available(&self) -> bool {
		*self.next_call.read() < time::Instant::now()
	}

	pub fn report_success(&self) {
		self.failures.store(0, atomic::Ordering::SeqCst);
		self.update_next_call(1)
	}

	pub fn report_failure(&self) {
		let errors = self.failures.fetch_add(1, atomic::Ordering::SeqCst);
		self.update_next_call(1 << errors)
	}

	fn update_next_call(&self, delay: usize) {
		*self.next_call.write() = time::Instant::now() + time::Duration::from_secs(delay as u64 * SERVER_MAX_POLL_INTERVAL_SECS);
	}
}

impl<T: AsRef<str>> From<T> for Server {
	fn from(t: T) -> Self {
		Server {
			address: t.as_ref().to_owned(),
			next_call: RwLock::new(time::Instant::now()),
			failures: Default::default(),
		}
	}
}

/// NTP client using the SNTP algorithm for calculating drift.
#[derive(Clone)]
pub struct SimpleNtp {
	addresses: Vec<Arc<Server>>,
	pool: CpuPool,
}

impl fmt::Debug for SimpleNtp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f
			.debug_struct("SimpleNtp")
			.field("addresses", &self.addresses)
			.finish()
	}
}

impl SimpleNtp {
	fn new<T: AsRef<str>>(addresses: &[T], pool: CpuPool) -> SimpleNtp {
		SimpleNtp {
			addresses: addresses.iter().map(Server::from).map(Arc::new).collect(),
			pool: pool,
		}
	}
}

impl Ntp for SimpleNtp {
	type Future = future::Either<
		CpuFuture<Duration, Error>,
		future::FutureResult<Duration, Error>,
	>;

	fn drift(&self) -> Self::Future {
		use self::future::Either::{A, B};

		let server = self.addresses.iter().find(|server| server.is_available());
		server.map(|server| {
			let server = server.clone();
			A(self.pool.spawn_fn(move || {
				debug!(target: "dapps", "Fetching time from {}.", server.address);

				match ntp::request(&server.address) {
					Ok(packet) => {
						let dest_time = ::time_crate::now_utc().to_timespec();
						let orig_time = Timespec::from(packet.orig_time);
						let recv_time = Timespec::from(packet.recv_time);
						let transmit_time = Timespec::from(packet.transmit_time);

						let drift = ((recv_time - orig_time) + (transmit_time - dest_time)) / 2;

						server.report_success();
						Ok(drift)
					},
					Err(err) => {
						server.report_failure();
						Err(err.into())
					},
				}
			}))
		}).unwrap_or_else(|| B(future::err(Error::NoServersAvailable)))
	}
}

// NOTE In a positive scenario first results will be seen after:
// MAX_RESULTS * UPDATE_TIMEOUT_INCOMPLETE_SECS seconds.
const MAX_RESULTS: usize = 4;
const UPDATE_TIMEOUT_OK_SECS: u64 = 6 * 60 * 60;
const UPDATE_TIMEOUT_WARN_SECS: u64 = 15 * 60;
const UPDATE_TIMEOUT_ERR_SECS: u64 = 60;
const UPDATE_TIMEOUT_INCOMPLETE_SECS: u64 = 10;

/// Maximal valid time drift.
pub const MAX_DRIFT: i64 = 10_000;

type BoxFuture<A, B> = Box<Future<Item = A, Error = B> + Send>;

#[derive(Debug, Clone)]
/// A time checker.
pub struct TimeChecker<N: Ntp = SimpleNtp> {
	ntp: N,
	last_result: Arc<RwLock<(time::Instant, VecDeque<Result<i64, Error>>)>>,
}

impl TimeChecker<SimpleNtp> {
	/// Creates new time checker given the NTP server address.
	pub fn new<T: AsRef<str>>(ntp_addresses: &[T], pool: CpuPool) -> Self {
		let last_result = Arc::new(RwLock::new(
			// Assume everything is ok at the very beginning.
			(time::Instant::now(), vec![Ok(0)].into())
		));

		let ntp = SimpleNtp::new(ntp_addresses, pool);

		TimeChecker {
			ntp,
			last_result,
		}
	}
}

impl<N: Ntp> TimeChecker<N> where <N::Future as IntoFuture>::Future: Send + 'static {
	/// Updates the time
	pub fn update(&self) -> BoxFuture<i64, Error> {
		trace!(target: "dapps", "Updating time from NTP.");
		let last_result = self.last_result.clone();
		Box::new(self.ntp.drift().into_future().then(move |res| {
			let res = res.map(|d| d.num_milliseconds());

			if let Err(Error::NoServersAvailable) = res {
				debug!(target: "dapps", "No NTP servers available. Selecting an older result.");
				return select_result(last_result.read().1.iter());
			}

			// Update the results.
			let mut results = mem::replace(&mut last_result.write().1, VecDeque::new());
			let has_all_results = results.len() >= MAX_RESULTS;
			let valid_till = time::Instant::now() + time::Duration::from_secs(
				match res {
					Ok(time) if has_all_results && time < MAX_DRIFT => UPDATE_TIMEOUT_OK_SECS,
					Ok(_) if has_all_results => UPDATE_TIMEOUT_WARN_SECS,
					Err(_) if has_all_results => UPDATE_TIMEOUT_ERR_SECS,
					_ => UPDATE_TIMEOUT_INCOMPLETE_SECS,
				}
			);

			trace!(target: "dapps", "New time drift received: {:?}", res);
			// Push the result.
			results.push_back(res);
			while results.len() > MAX_RESULTS {
				results.pop_front();
			}

			// Select a response and update last result.
			let res = select_result(results.iter());
			*last_result.write() = (valid_till, results);
			res
		}))
	}

	/// Returns a current time drift or error if last request to NTP server failed.
	pub fn time_drift(&self) -> BoxFuture<i64, Error> {
		// return cached result
		{
			let res = self.last_result.read();
			if res.0 > time::Instant::now() {
				return Box::new(futures::done(select_result(res.1.iter())));
			}
		}
		// or update and return result
		self.update()
	}
}

fn select_result<'a, T: Iterator<Item=&'a Result<i64, Error>>>(results: T) -> Result<i64, Error> {
	let mut min = None;
	for res in results {
		min = Some(match (min.take(), res) {
			(Some(Ok(min)), &Ok(ref new)) => Ok(::std::cmp::min(min, *new)),
			(Some(Ok(old)), &Err(_)) => Ok(old),
			(_, ref new) => (*new).clone(),
		})
	}

	min.unwrap_or_else(|| Err(Error::Ntp("NTP server unavailable.".into())))
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::cell::{Cell, RefCell};
	use std::time::Instant;
	use time::Duration;
	use futures::{future, Future};
	use super::{Ntp, TimeChecker, Error};
	use parking_lot::RwLock;

	#[derive(Clone)]
	struct FakeNtp(RefCell<Vec<Duration>>, Cell<u64>);
	impl FakeNtp {
		fn new() -> FakeNtp {
			FakeNtp(
				RefCell::new(vec![Duration::milliseconds(150)]),
				Cell::new(0))
		}
	}

	impl Ntp for FakeNtp {
		type Future = future::FutureResult<Duration, Error>;

		fn drift(&self) -> Self::Future {
			self.1.set(self.1.get() + 1);
			future::ok(self.0.borrow_mut().pop().expect("Unexpected call to drift()."))
		}
	}

	fn time_checker() -> TimeChecker<FakeNtp> {
		let last_result = Arc::new(RwLock::new(
			(Instant::now(), vec![Err(Error::Ntp("NTP server unavailable".into()))].into())
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
		assert_eq!(time.ntp.1.get(), 1);
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
		assert_eq!(time.ntp.1.get(), 1);
	}
}
