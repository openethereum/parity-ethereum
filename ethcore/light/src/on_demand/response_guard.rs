// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! ResponseGuard implementation.
//! It is responsible for the receiving end of `Pending Request` (see `OnDemand` module docs for more information)
//! The major functionality is the following:
//!    1) Register non-successful responses which will reported back if it fails
//!    2) A timeout mechanism that will wait for successful response at most t seconds

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fmt;

use super::{ResponseError, ValidityError};

/// Response guard error type
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
	/// No majority, the error reason can't be determined
	NoMajority(usize),
	/// Majority, with the error reason
	Majority(Inner, usize, usize),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::Majority(err, majority, total) => {
				write!(f, "Error cause was {:?}, (majority count: {} / total: {})",
					err, majority, total)
			}
			Error::NoMajority(total) => {
				write!(f, "Error cause couldn't be determined, the total number of responses was {}", total)
			}
		}
	}
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
	request_start: Instant,
	time_to_live: Duration,
	responses: HashMap<Inner, usize>,
	number_responses: usize,
}

impl ResponseGuard {
	/// Constructor
	pub fn new(time_to_live: Duration) -> Self {
		Self {
			request_start: Instant::now(),
			time_to_live,
			responses: HashMap::new(),
			number_responses: 0,
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
		let err = self.into_reason(err);
		*self.responses.entry(err).or_insert(0) += 1;
		self.number_responses = self.number_responses.saturating_add(1);
		trace!(target: "circuit_breaker", "ResponseGuard: {:?}", self.responses);
		// The request has exceeded its timeout
		if self.request_start.elapsed() >= self.time_to_live {
			let (&err, &max_count) = self.responses.iter().max_by_key(|(_k, v)| *v).expect("got at least one element; qed");
			let majority = self.responses.values().filter(|v| **v == max_count).count() == 1;
			if majority {
				Err(Error::Majority(err, max_count, self.number_responses))
			} else {
				Err(Error::NoMajority(self.number_responses))
			}
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use std::thread;
	use super::*;

	#[test]
	fn test_basic_by_majority() {
		let mut guard = ResponseGuard::new(Duration::from_secs(5));
		guard.register_error(&ResponseError::Validity(ValidityError::Empty)).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		thread::sleep(Duration::from_secs(5));

		assert_eq!(guard.register_error(&ResponseError::Validity(ValidityError::WrongKind)), Err(Error::Majority(Inner::Unexpected, 3, 5)));
	}

	#[test]
	fn test_no_majority() {
		let mut guard = ResponseGuard::new(Duration::from_secs(5));
		guard.register_error(&ResponseError::Validity(ValidityError::Empty)).unwrap();
		guard.register_error(&ResponseError::Validity(ValidityError::Empty)).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		guard.register_error(&ResponseError::Unexpected).unwrap();
		thread::sleep(Duration::from_secs(5));

		assert_eq!(guard.register_error(&ResponseError::Validity(ValidityError::WrongKind)), Err(Error::NoMajority(5)));
	}
}
