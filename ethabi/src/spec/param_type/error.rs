use std::num::ParseIntError;

/// Param parsing error.
#[derive(Debug)]
pub enum Error {
	/// Returned when part of the type name is expected to be a number, but it is not.
	ParseInt(ParseIntError),
	/// Returned in all other cases when param type is invalid.
	InvalidType,
}

impl From<ParseIntError> for Error {
	fn from(err: ParseIntError) -> Self {
		Error::ParseInt(err)
	}
}

