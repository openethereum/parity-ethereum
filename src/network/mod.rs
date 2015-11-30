extern crate mio;
pub mod host;
pub mod connection;
pub mod handshake;


#[derive(Debug)]
pub enum Error {
	Crypto(::crypto::CryptoError),
	Io(::std::io::Error),
}

impl From<::std::io::Error> for Error {
	fn from(err: ::std::io::Error) -> Error {
		Error::Io(err)
	}
}

impl From<::crypto::CryptoError> for Error {
	fn from(err: ::crypto::CryptoError) -> Error {
		Error::Crypto(err)
	}
}

