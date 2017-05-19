use std::num::ParseIntError;
use hex::FromHexError;

/// Called when tokenizing fails.
#[derive(Debug)]
pub enum Error {
	/// Returned when string is expected to be hex, but it is not.
	FromHex(FromHexError),
	/// Returned in all other cases.
	InvalidValue,
	/// Returned when integer value cannot be parsed.
	ParseInt(ParseIntError),
}

impl From<FromHexError> for Error {
	fn from(err: FromHexError) -> Self {
		Error::FromHex(err)
	}
}

impl From<ParseIntError> for Error {
	fn from(err: ParseIntError) -> Self {
		Error::ParseInt(err)
	}
}
