use std::fmt;
use std::error::Error as StdError;
use bytes::FromBytesError;

#[derive(Debug, PartialEq, Eq)]
/// TODO [debris] Please document me
pub enum DecoderError {
	/// TODO [debris] Please document me
	FromBytesError(FromBytesError),
	/// TODO [debris] Please document me
	RlpIsTooShort,
	/// TODO [debris] Please document me
	RlpExpectedToBeList,
	/// TODO [Gav Wood] Please document me
	RlpExpectedToBeData,
	/// TODO [Gav Wood] Please document me
	RlpIncorrectListLen,
	/// TODO [Gav Wood] Please document me
	RlpDataLenWithZeroPrefix,
	/// TODO [Gav Wood] Please document me
	RlpListLenWithZeroPrefix,
	/// TODO [debris] Please document me
	RlpInvalidIndirection,
	/// Returned when declared length is inconsistent with data specified after
	RlpInconsistentLengthAndData
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
