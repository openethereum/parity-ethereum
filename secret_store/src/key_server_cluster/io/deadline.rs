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

use std::io;
use std::time::Duration;
use futures::{Future, Select, BoxFuture, Poll, Async};
use tokio_core::reactor::{Handle, Timeout};

type DeadlineBox<F> = BoxFuture<DeadlineStatus<<F as Future>::Item>, <F as Future>::Error>;

/// Complete a passed future or fail if it is not completed within timeout.
pub fn deadline<F, T>(duration: Duration, handle: &Handle, future: F) -> Result<Deadline<F>, io::Error>
	where F: Future<Item = T, Error = io::Error> + Send + 'static, T: 'static {
	let timeout = Timeout::new(duration, handle)?.map(|_| DeadlineStatus::Timeout).boxed();
	let future = future.map(DeadlineStatus::Meet).boxed();
	let deadline = Deadline {
		future: timeout.select(future),
	};
	Ok(deadline)
}

/// Deadline future completion status.
#[derive(Debug, PartialEq)]
pub enum DeadlineStatus<T> {
	/// Completed a future.
	Meet(T),
	/// Faled with timeout.
	Timeout,
}

/// Future, which waits for passed future completion within given period, or fails with timeout.
pub struct Deadline<F> where F: Future {
	future: Select<DeadlineBox<F>, DeadlineBox<F>>,
}

impl<F, T> Future for Deadline<F> where F: Future<Item = T, Error = io::Error> {
	type Item = DeadlineStatus<T>;
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		match self.future.poll() {
			Ok(Async::Ready((result, _other))) => Ok(Async::Ready(result)),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err((err, _other)) => Err(err),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::io;
	use std::time::Duration;
	use futures::{Future, empty, done};
	use tokio_core::reactor::Core;
	use super::{deadline, DeadlineStatus};

	//#[test] TODO: not working
	fn _deadline_timeout_works() {
		let mut core = Core::new().unwrap();
		let deadline = deadline(Duration::from_millis(1), &core.handle(), empty::<(), io::Error>()).unwrap();
		core.turn(Some(Duration::from_millis(3)));
		assert_eq!(deadline.wait().unwrap(), DeadlineStatus::Timeout);
	}

	#[test]
	fn deadline_result_works() {
		let mut core = Core::new().unwrap();
		let deadline = deadline(Duration::from_millis(1000), &core.handle(), done(Ok(()))).unwrap();
		core.turn(Some(Duration::from_millis(3)));
		assert_eq!(deadline.wait().unwrap(), DeadlineStatus::Meet(()));
	}
}
