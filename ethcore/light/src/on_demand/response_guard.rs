use std::time::Duration;
use std::collections::HashMap;

use failsafe;

type ResponsePolicy = failsafe::failure_policy::SuccessRateOverTimeWindow<failsafe::backoff::Exponential>;

use super::ValidityError;
use super::ResponseError;

/// Dummy type to convert a generic type with no trait bounds
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum IncompleteError {
	/// Bad execution proof
	BadProof,
	/// RLP decoding
	Decoder,
	/// Empty response
	EmptyResonse,
	/// Header by number when expecting something else
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

/// Handle and register calls that can fail
#[derive(Debug)]
pub struct ResponseGuard {
	state: failsafe::StateMachine<ResponsePolicy, failsafe::NoopInstrument>,
	responses: HashMap<IncompleteError, usize>,
}

impl ResponseGuard {
	/// Constructor
	pub fn new(
		required_success_rate: f64,
		min_backoff_dur: Duration,
		max_backoff_dur: Duration,
		window_dur: Duration
	) -> Self {
		let backoff = failsafe::backoff::exponential(min_backoff_dur, max_backoff_dur);
		let policy = failsafe::failure_policy::success_rate_over_time_window(required_success_rate, 1, window_dur, backoff);

		Self {
			state: failsafe::StateMachine::new(policy, failsafe::NoopInstrument),
			responses: HashMap::new(),
		}
	}

	fn into_incomplete(&self, err: &ResponseError<super::request::Error>) -> IncompleteError {
		match err {
			ResponseError::Unexpected => IncompleteError::Unexpected,
			ResponseError::EmptyResponse => IncompleteError::EmptyResonse,
			ResponseError::Validity(ValidityError::BadProof) => IncompleteError::BadProof,
			ResponseError::Validity(ValidityError::Decoder(_)) => IncompleteError::Decoder,
			ResponseError::Validity(ValidityError::Empty) => IncompleteError::EmptyResonse,
			ResponseError::Validity(ValidityError::HeaderByNumber) => IncompleteError::HeaderByNumber,
			ResponseError::Validity(ValidityError::TooFewResults(_, _)) => IncompleteError::TooFewResults,
			ResponseError::Validity(ValidityError::TooManyResults(_, _)) => IncompleteError::TooManyResults,
			ResponseError::Validity(ValidityError::Trie(_)) => IncompleteError::Trie,
			ResponseError::Validity(ValidityError::UnresolvedHeader(_)) => IncompleteError::UnresolvedHeader,
			ResponseError::Validity(ValidityError::WrongHash(_, _)) => IncompleteError::WrongHash,
			ResponseError::Validity(ValidityError::WrongHeaderSequence) => IncompleteError::WrongHeaderSequence,
			ResponseError::Validity(ValidityError::WrongKind) => IncompleteError::WrongKind,
			ResponseError::Validity(ValidityError::WrongNumber(_, _)) => IncompleteError::WrongNumber,
			ResponseError::Validity(ValidityError::WrongTrieRoot(_, _)) => IncompleteError::WrongTrieRoot,
		}
	}

	/// Update the state after a `faulty` call
	pub fn register_error(&mut self, err: &ResponseError<super::request::Error>) -> Result<(), IncompleteError> {
			self.state.on_error();
			let err = self.into_incomplete(err);
			*self.responses.entry(err).or_insert(0) += 1;
			if self.state.is_call_permitted() {
				Ok(())
			} else {
				// O(n)
				Err(*self.responses.iter().max().map(|(k, _v)| k).expect("got at least one element; qed"))
			}
	}
}
