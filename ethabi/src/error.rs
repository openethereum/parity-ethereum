use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
	FunctionNotFound,
	EventNotFound,
	InvalidData,
	InvalidFunctionParams,
	Utf8(FromUtf8Error),
}

impl From<FromUtf8Error> for Error {
	fn from(err: FromUtf8Error) -> Self {
		Error::Utf8(err)
	}
}
