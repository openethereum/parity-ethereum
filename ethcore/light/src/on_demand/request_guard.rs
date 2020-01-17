// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use failsafe;
use std::time::Duration;

type RequestPolicy = failsafe::failure_policy::ConsecutiveFailures<failsafe::backoff::Exponential>;

/// Error wrapped on-top of `FailsafeError`
#[derive(Debug, PartialEq)]
pub enum Error {
	/// The call is let through
	LetThrough,
	/// The call rejected by the guard
	Rejected,
	/// The request reached the maximum of backoff iterations
	ReachedLimit,
}

/// Handle and register requests that can fail
#[derive(Debug)]
pub struct RequestGuard {
	backoff_round: usize,
	max_backoff_rounds: usize,
	state: failsafe::StateMachine<RequestPolicy, ()>,
}

impl RequestGuard {
	/// Constructor
	pub fn new(
		consecutive_failures: u32,
		max_backoff_rounds: usize,
		start_backoff: Duration,
		max_backoff: Duration,
	) -> Self {
		let backoff = failsafe::backoff::exponential(start_backoff, max_backoff);
		// success_rate not used because only errors are registered
		let policy = failsafe::failure_policy::consecutive_failures(consecutive_failures as u32, backoff);

		Self {
			backoff_round: 0,
			max_backoff_rounds,
			state: failsafe::StateMachine::new(policy, ()),
		}
	}

	/// Update the state after a `faulty` call
	pub fn register_error(&mut self) -> Error {
		trace!(target: "circuit_breaker", "RequestGuard; backoff_round: {}/{}, state {:?}",
			   self.backoff_round, self.max_backoff_rounds, self.state);

		if self.backoff_round >= self.max_backoff_rounds {
			Error::ReachedLimit
		} else if self.state.is_call_permitted() {
			self.state.on_error();
			if self.state.is_call_permitted() {
				Error::LetThrough
			} else {
				self.backoff_round += 1;
				Error::Rejected
			}
		} else {
			Error::Rejected
		}
	}

	/// Poll the circuit breaker, to check if the call is permitted
	pub fn is_call_permitted(&self) -> bool {
		self.state.is_call_permitted()
	}
}

#[cfg(test)]
mod tests {
	use std::iter;
	use std::time::Instant;
	use super::*;

	#[test]
	fn one_consecutive_failure_with_10_backoffs() {
		// 1, 2, 4, 5, 5 .... 5
		let binary_exp_backoff = vec![1_u64, 2, 4].into_iter().chain(iter::repeat(5_u64).take(7));
		let mut guard = RequestGuard::new(1, 10, Duration::from_secs(1), Duration::from_secs(5));
		for backoff in binary_exp_backoff {
			assert_eq!(guard.register_error(), Error::Rejected);
			let now = Instant::now();
			while now.elapsed() <= Duration::from_secs(backoff) {}
		}
		assert_eq!(guard.register_error(), Error::ReachedLimit, "10 backoffs should be error");
	}

	#[test]
	fn five_consecutive_failures_with_3_backoffs() {
		let mut guard = RequestGuard::new(5, 3, Duration::from_secs(1), Duration::from_secs(30));

		// register five errors
		for _ in 0..4 {
			assert_eq!(guard.register_error(), Error::LetThrough);
		}

		let binary_exp_backoff = [1, 2, 4];
		for backoff in &binary_exp_backoff {
			assert_eq!(guard.register_error(), Error::Rejected);
			let now = Instant::now();
			while now.elapsed() <= Duration::from_secs(*backoff) {}
		}

		assert_eq!(guard.register_error(), Error::ReachedLimit, "3 backoffs should be an error");
	}
}
