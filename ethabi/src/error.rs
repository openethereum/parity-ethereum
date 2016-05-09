//! ABI error variants.

use std::string::FromUtf8Error;

/// ABI error variants.
#[derive(Debug)]
pub enum Error {
	/// Returned when encoded / decoded data does not match params.
	InvalidData,
	/// Returned when there is a problem with decoding utf8 string.
	Utf8(FromUtf8Error),
	/// Returned when constructor or function does not exists.
	InvalidName,
}

impl From<FromUtf8Error> for Error {
	fn from(err: FromUtf8Error) -> Self {
		Error::Utf8(err)
	}
}
