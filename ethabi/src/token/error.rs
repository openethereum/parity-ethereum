use rustc_serialize::hex::FromHexError;

/// Called when tokenizing fails.
#[derive(Debug)]
pub enum Error {
	/// Returned when string is expected to be hex, but it is not.
	FromHex(FromHexError),
	/// Returned in all other cases.
	InvalidValue,
}

impl From<FromHexError> for Error {
	fn from(err: FromHexError) -> Self {
		Error::FromHex(err)
	}
}
