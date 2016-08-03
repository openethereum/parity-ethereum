use trace::Error as TraceError;
use util::UtilError;
use std::fmt::{Display, Formatter, Error as FmtError};

/// Client configuration errors.
#[derive(Debug)]
pub enum Error {
	/// TraceDB configuration error.
	Trace(TraceError),
	/// Database error
	Database(String),
	/// Util error
	Util(UtilError),
}

impl From<TraceError> for Error {
	fn from(err: TraceError) -> Self {
		Error::Trace(err)
	}
}

impl From<UtilError> for Error {
	fn from(err: UtilError) -> Self {
		Error::Util(err)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			Error::Trace(ref err) => write!(f, "{}", err),
			Error::Util(ref err) => write!(f, "{}", err),
			Error::Database(ref s) => write!(f, "Database error: {}", s),
		}
	}
}
