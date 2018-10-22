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

use std::io;
use std::time::Duration;
use futures::{Future, Poll};
use tokio::timer::timeout::{Timeout, Error as TimeoutError};

type DeadlineBox<F> = Box<Future<
	Item = DeadlineStatus<<F as Future>::Item>,
	Error = TimeoutError<<F as Future>::Error>
> + Send>;

/// Complete a passed future or fail if it is not completed within timeout.
pub fn deadline<F, T>(duration: Duration, future: F) -> Result<Deadline<F>, io::Error>
	where F: Future<Item = T, Error = io::Error> + Send + 'static, T: Send + 'static
{
	let timeout = Box::new(Timeout::new(future, duration)
		.then(|res| {
			match res {
				Ok(fut) => Ok(DeadlineStatus::Meet(fut)),
				Err(err) => {
					if err.is_elapsed() {
						Ok(DeadlineStatus::Timeout)
					} else {
						Err(err)
					}
				},
			}
		})
	);
	let deadline = Deadline {
		future: timeout,
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
	future: DeadlineBox<F>,
}

impl<F, T> Future for Deadline<F> where F: Future<Item = T, Error = io::Error> {
	type Item = DeadlineStatus<T>;
	type Error = TimeoutError<io::Error>;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		self.future.poll()
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;
	use futures::{Future, done};
	use tokio::reactor::Reactor;
	use super::{deadline, DeadlineStatus};

	#[test]
	fn deadline_result_works() {
		let mut reactor = Reactor::new().unwrap();
		let deadline = deadline(Duration::from_millis(1000), done(Ok(()))).unwrap();
		reactor.turn(Some(Duration::from_millis(3))).unwrap();
		assert_eq!(deadline.wait().unwrap(), DeadlineStatus::Meet(()));
	}
}
