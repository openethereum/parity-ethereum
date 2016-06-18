use std::fmt;
use std::io::Error as IoError;
use ethkey::Error as EthKeyError;

#[derive(Debug)]
pub enum Error {
	Io(IoError),
	InvalidPassword,
	InvalidSecret,
	InvalidAccount,
	CreationFailed,
	EthKey(EthKeyError),
	Custom(String),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		let s = match *self {
			Error::Io(ref err) => format!("{}", err),
			Error::InvalidPassword => "Invalid password".into(),
			Error::InvalidSecret => "Invalid secret".into(),
			Error::InvalidAccount => "Invalid account".into(),
			Error::CreationFailed => "Account creation failed".into(),
			Error::EthKey(ref err) => format!("{}", err),
			Error::Custom(ref s) => s.clone(),
		};

		write!(f, "{}", s)
	}
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Self {
		Error::Io(err)
	}
}

impl From<EthKeyError> for Error {
	fn from(err: EthKeyError) -> Self {
		Error::EthKey(err)
	}
}
