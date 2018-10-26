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

use std::time::Duration;
use std::collections::HashMap;

use failsafe;
use super::{ResponseError, ValidityError};

type ResponsePolicy = failsafe::failure_policy::SuccessRateOverTimeWindow<NoBackoff>;

/// Response guard error type
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
	/// No majority, the error reason can't be determined
	NoMajority,
	/// Majority, with the error reason
	Majority(Inner),
}


/// Dummy type to convert a generic type with no trait bounds
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Inner {
	/// Bad execution proof
	BadProof,
	/// RLP decoding
	Decoder,
	/// Empty response
	EmptyResponse,
	/// Wrong header sequence
	HeaderByNumber,
	/// Too few results
	TooFewResults,
	/// Too many results
	TooManyResults,
	/// Trie error
	Trie,
	/// Unresolved header
	UnresolvedHeader,
	/// No responses expected.
	Unexpected,
	/// Wrong hash
	WrongHash,
	/// Wrong Header sequence
	WrongHeaderSequence,
	/// Wrong response kind
	WrongKind,
	/// Wrong number
	WrongNumber,
	/// Wrong Trie Root
	WrongTrieRoot,
}

/// Handle and register responses that can fail
#[derive(Debug)]
pub struct ResponseGuard {
	state: failsafe::StateMachine<ResponsePolicy, ()>,
	responses: HashMap<Inner, usize>,
}

impl ResponseGuard {
	/// Constructor
	pub fn new(required_success_rate: f64, window_dur: Duration) -> Self {
		let policy = failsafe::failure_policy::success_rate_over_time_window(required_success_rate, 1, window_dur, NoBackoff);

		Self {
			state: failsafe::StateMachine::new(policy, ()),
			responses: HashMap::new(),
		}
	}

	fn into_reason(&self, err: &ResponseError<super::request::Error>) -> Inner {
		match err {
			ResponseError::Unexpected => Inner::Unexpected,
			ResponseError::Validity(ValidityError::BadProof) => Inner::BadProof,
			ResponseError::Validity(ValidityError::Decoder(_)) => Inner::Decoder,
			ResponseError::Validity(ValidityError::Empty) => Inner::EmptyResponse,
			ResponseError::Validity(ValidityError::HeaderByNumber) => Inner::HeaderByNumber,
			ResponseError::Validity(ValidityError::TooFewResults(_, _)) => Inner::TooFewResults,
			ResponseError::Validity(ValidityError::TooManyResults(_, _)) => Inner::TooManyResults,
			ResponseError::Validity(ValidityError::Trie(_)) => Inner::Trie,
			ResponseError::Validity(ValidityError::UnresolvedHeader(_)) => Inner::UnresolvedHeader,
			ResponseError::Validity(ValidityError::WrongHash(_, _)) => Inner::WrongHash,
			ResponseError::Validity(ValidityError::WrongHeaderSequence) => Inner::WrongHeaderSequence,
			ResponseError::Validity(ValidityError::WrongKind) => Inner::WrongKind,
			ResponseError::Validity(ValidityError::WrongNumber(_, _)) => Inner::WrongNumber,
			ResponseError::Validity(ValidityError::WrongTrieRoot(_, _)) => Inner::WrongTrieRoot,
		}
	}

	/// Update the state after a `faulty` call
	pub fn register_error(&mut self, err: &ResponseError<super::request::Error>) -> Result<(), Error> {
			self.state.on_error();
			let err = self.into_reason(err);
			*self.responses.entry(err).or_insert(0) += 1;
			if self.state.is_call_permitted() {
				Ok(())
			} else {
				let (&err, &max_count) = self.responses.iter().max_by_key(|(_k, v)| *v).expect("got at least one element; qed");
				let majority = self.responses.values().filter(|v| **v == max_count).count() == 1;

				if majority {
					Err(Error::Majority(err))
				} else {
					Err(Error::NoMajority)
				}
			}
	}
}

// Type to indicate that response should never back-off
#[derive(Clone, Debug)]
struct NoBackoff;

impl Iterator for NoBackoff {
	type Item = Duration;

	fn next(&mut self) -> Option<Self::Item> {
		None
	}
}

#[cfg(test)]
mod tests {
	use std::thread;
	use super::*;

	#[test]
	fn test_basic_by_majority() {
		let mut guard = ResponseGuard::new(0.8, Duration::from_secs(5));
		guard.register_error(&ResponseError::Validity(ValidityError::Empty)).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		// wait for the current time window to end
		thread::sleep(Duration::from_secs(5));

		assert_eq!(guard.register_error(&ResponseError::Validity(ValidityError::WrongKind)), Err(Error::Majority(Inner::Unexpected)));
	}

	#[test]
	fn test_no_majority() {
		let mut guard = ResponseGuard::new(0.8, Duration::from_secs(5));
		guard.register_error(&ResponseError::Validity(ValidityError::Empty)).unwrap();
		guard.register_error(&ResponseError::Validity(ValidityError::Empty)).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		// wait for the current time window to end
		thread::sleep(Duration::from_secs(5));

		assert_eq!(guard.register_error(&ResponseError::Validity(ValidityError::WrongKind)), Err(Error::NoMajority));
	}
}
