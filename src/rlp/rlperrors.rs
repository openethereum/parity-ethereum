use std::fmt;
use std::error::Error as StdError;
use bytes::FromBytesError;

#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
	FromBytesError(FromBytesError),
	RlpIsTooShort,
	RlpExpectedToBeList,
	RlpExpectedToBeData,
	RlpIncorrectListLen,
}

impl StdError for DecoderError {
	fn description(&self) -> &str {
		"builder error"
	}
}

impl fmt::Display for DecoderError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

impl From<FromBytesError> for DecoderError {
	fn from(err: FromBytesError) -> DecoderError {
		DecoderError::FromBytesError(err)
	}
}
