use trace::Error as TraceError;
use std::fmt::{Display, Formatter, Error as FmtError};

use util::trie::TrieError;

/// Client configuration errors.
#[derive(Debug)]
pub enum Error {
	/// TraceDB configuration error.
	Trace(TraceError),
	/// TrieDB-related error.
	Trie(TrieError),
}

impl From<TraceError> for Error {
	fn from(err: TraceError) -> Self {
		Error::Trace(err)
	}
}

impl From<TrieError> for Error {
	fn from(err: TrieError) -> Self {
		Error::Trie(err)
	}
}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Self {
		Error::from(*err)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			Error::Trace(ref err) => write!(f, "{}", err),
			Error::Trie(ref err) => write!(f, "{}", err),
		}
	}
}
