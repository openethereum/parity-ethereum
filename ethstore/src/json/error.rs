use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
	UnsupportedCipher,
	InvalidCipherParams,
	UnsupportedKdf,
	InvalidUUID,
	UnsupportedVersion,
	InvalidCiphertext,
	InvalidH256,
	InvalidPrf,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::InvalidUUID => write!(f, "Invalid UUID"),
			Error::UnsupportedVersion => write!(f, "Unsupported version"),
			Error::UnsupportedKdf => write!(f, "Unsupported kdf"),
			Error::InvalidCiphertext => write!(f, "Invalid ciphertext"),
			Error::UnsupportedCipher => write!(f, "Unsupported cipher"),
			Error::InvalidCipherParams => write!(f, "Invalid cipher params"),
			Error::InvalidH256 => write!(f, "Invalid hash"),
			Error::InvalidPrf => write!(f, "Invalid prf"),
		}
	}
}

impl Into<String> for Error {
	fn into(self) -> String {
		format!("{}", self)
	}
}
