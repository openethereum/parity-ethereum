use rustc_serialize::hex::FromHexError;

#[derive(Debug)]
pub enum Error {
	FromHex(FromHexError),
	InvalidValue,
}

impl From<FromHexError> for Error {
	fn from(err: FromHexError) -> Self {
		Error::FromHex(err)
	}
}
