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

use failsafe;
use std::time::Duration;

type RequestPolicy = failsafe::failure_policy::SuccessRateOverTimeWindow<failsafe::backoff::Exponential>;

/// Error wrapped on-top of `FailsafeError`
#[derive(Debug)]
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
	num_failures: usize,
	max_failures: usize,
	state: failsafe::StateMachine<RequestPolicy, ()>,
}

impl RequestGuard {
	/// Constructor
	pub fn new(
		window_dur: Duration,
		min_backoff_dur: Duration,
		max_backoff_dur: Duration,
		max_backoff_rounds: usize
	) -> Self {
		let backoff = failsafe::backoff::exponential(min_backoff_dur, max_backoff_dur);
		// success_rate not used because only errors are registered
		let policy = failsafe::failure_policy::success_rate_over_time_window(1.00, 1, window_dur, backoff);

		Self {
			num_failures: 0,
			max_failures: max_backoff_rounds,
			state: failsafe::StateMachine::new(policy, ()),
		}
	}

	/// Update the state after a `faulty` call
	pub fn register_error(&mut self) -> Error {
		// Circuit breaker is `closed`, count as a failure
		if self.num_failures >= self.max_failures {
				trace!(target: "circuit_breaker", "RequestGuard: reached_limit, failures: {}/{}, state {:?}",
					self.num_failures, self.max_failures, self.state);
				Error::ReachedLimit
		}
		else if self.state.is_call_permitted() {
			self.state.on_error();
			self.num_failures += 1;

			if self.num_failures >= self.max_failures {
				trace!(target: "circuit_breaker", "RequestGuard: reached_limit, failures: {}/{}, state {:?}",
					self.num_failures, self.max_failures, self.state);
				Error::ReachedLimit
			} else {
				trace!(target: "circuit_breaker", "RequestGuard; failures: {}/{}, state {:?}",
					self.num_failures, self.max_failures, self.state);
				Error::LetThrough
			}
		}
		// Circuit breaker is `open`, don't count as a failure
		else {
			trace!(target: "circuit_breaker", "RequestGuard; failures: {}/{}, state {:?}",
				self.num_failures, self.max_failures, self.state);
			Error::Rejected
		}
	}

	/// Poll the circuit breaker, to check if the call is permitted
	pub fn is_call_permitted(&self) -> bool {
		self.state.is_call_permitted()
	}
}

