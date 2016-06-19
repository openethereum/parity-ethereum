use std::fmt;

#[derive(Debug)]
/// Crypto error
pub enum Error {
	/// Invalid secret key
	InvalidSecret,
	/// Invalid public key
	InvalidPublic,
	/// Invalid address
	InvalidAddress,
	/// Invalid EC signature
	InvalidSignature,
	/// Invalid AES message
	InvalidMessage,
	/// IO Error
	Io(::std::io::Error),
	/// Custom
	Custom(String),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let msg = match *self {
			Error::InvalidSecret => "Invalid secret key".into(),
			Error::InvalidPublic => "Invalid public key".into(),
			Error::InvalidAddress => "Invalid address".into(),
			Error::InvalidSignature => "Invalid EC signature".into(),
			Error::InvalidMessage => "Invalid AES message".into(),
			Error::Io(ref err) => format!("I/O error: {}", err),
			Error::Custom(ref s) => s.clone(),
		};

		f.write_fmt(format_args!("Crypto error ({})", msg))
	}
}

impl From<::secp256k1::Error> for Error {
	fn from(e: ::secp256k1::Error) -> Error {
		match e {
			::secp256k1::Error::InvalidMessage => Error::InvalidMessage,
			::secp256k1::Error::InvalidPublicKey => Error::InvalidPublic,
			::secp256k1::Error::InvalidSecretKey => Error::InvalidSecret,
			_ => Error::InvalidSignature,
		}
	}
}

impl From<::std::io::Error> for Error {
	fn from(err: ::std::io::Error) -> Error {
		Error::Io(err)
	}
}
