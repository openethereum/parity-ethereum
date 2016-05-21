use trace::Error as TraceError;
use std::fmt::{Display, Formatter, Error as FmtError};

/// Client configuration errors.
#[derive(Debug)]
pub enum Error {
	/// TraceDB configuration error.
	Trace(TraceError),
}

impl From<TraceError> for Error {
	fn from(err: TraceError) -> Self {
		Error::Trace(err)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			Error::Trace(ref err) => write!(f, "{}", err)
		}
	}
}
