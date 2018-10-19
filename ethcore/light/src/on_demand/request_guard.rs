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
	/// The request reached the maximum number of attempts
	ReachedLimit(usize),
}

/// Handle and register calls that can fail
#[derive(Debug)]
pub struct RequestGuard {
	num_failures: usize,
	max_failures: usize,
	state: failsafe::StateMachine<RequestPolicy, failsafe::NoopInstrument>,
}

impl RequestGuard {
	/// Constructor
	pub fn new(
			required_success_rate: f64,
			min_backoff_dur: Duration,
			max_backoff_dur: Duration,
			window_dur: Duration,
			max_backoff_rounds: usize
		) -> Self {
		let backoff = failsafe::backoff::exponential(min_backoff_dur, max_backoff_dur);
		let policy = failsafe::failure_policy::success_rate_over_time_window(required_success_rate, 1, window_dur, backoff);

		Self {
			num_failures: 0,
			max_failures: max_backoff_rounds,
			state: failsafe::StateMachine::new(policy, failsafe::NoopInstrument)
		}
	}

	/// Update the state after a `faulty` call
	pub fn register_error(&mut self) -> Error {
		if self.state.is_call_permitted() {
			// register as a `failure`
			self.state.on_error();
			self.num_failures += 1;

			// max number of failures received
			if self.num_failures >= self.max_failures {
				Error::ReachedLimit(self.max_failures)
			} else {
				Error::LetThrough
			}
		} else {
			Error::Rejected
		}
	}

	/// Wrapper
	pub fn is_call_permitted(&self) -> bool {
		self.state.is_call_permitted()
	}
}
