//! General error types for use in ethcore.

use rustc_serialize::hex::FromHexError;
use network::NetworkError;
use rlp::DecoderError;
use io;

#[derive(Debug)]
/// Error in database subsystem.
pub enum BaseDataError {
	/// An entry was removed more times than inserted.
	NegativelyReferencedHash,
}

#[derive(Debug)]
/// General error type which should be capable of representing all errors in ethcore.
pub enum UtilError {
	/// Error concerning the crypto utility subsystem.
	Crypto(::crypto::CryptoError),
	/// Error concerning the Rust standard library's IO subsystem.
	StdIo(::std::io::Error),
	/// Error concerning our IO utility subsystem.
	Io(io::IoError),
	/// Error concerning the network address parsing subsystem.
	AddressParse(::std::net::AddrParseError),
	/// Error concerning the network address resolution subsystem.
	AddressResolve(Option<::std::io::Error>),
	/// Error concerning the hex conversion logic.
	FromHex(FromHexError),
	/// Error concerning the database abstraction logic.
	BaseData(BaseDataError),
	/// Error concerning the network subsystem.
	Network(NetworkError),
	/// Error concerning the RLP decoder.
	Decoder(DecoderError),
	/// Miscellaneous error described by a string.
	SimpleString(String),
	/// Error from a bad input size being given for the needed output.
	BadSize,
}

impl From<FromHexError> for UtilError {
	fn from(err: FromHexError) -> UtilError {
		UtilError::FromHex(err)
	}
}

impl From<BaseDataError> for UtilError {
	fn from(err: BaseDataError) -> UtilError {
		UtilError::BaseData(err)
	}
}

impl From<NetworkError> for UtilError {
	fn from(err: NetworkError) -> UtilError {
		UtilError::Network(err)
	}
}

impl From<::std::io::Error> for UtilError {
	fn from(err: ::std::io::Error) -> UtilError {
		UtilError::StdIo(err)
	}
}

impl From<io::IoError> for UtilError {
	fn from(err: io::IoError) -> UtilError {
		UtilError::Io(err)
	}
}

impl From<::crypto::CryptoError> for UtilError {
	fn from(err: ::crypto::CryptoError) -> UtilError {
		UtilError::Crypto(err)
	}
}

impl From<::std::net::AddrParseError> for UtilError {
	fn from(err: ::std::net::AddrParseError) -> UtilError {
		UtilError::AddressParse(err)
	}
}

impl From<::rlp::DecoderError> for UtilError {
	fn from(err: ::rlp::DecoderError) -> UtilError {
		UtilError::Decoder(err)
	}
}

impl From<String> for UtilError {
	fn from(err: String) -> UtilError {
		UtilError::SimpleString(err)
	}
}

// TODO: uncomment below once https://github.com/rust-lang/rust/issues/27336 sorted.
/*#![feature(concat_idents)]
macro_rules! assimilate {
    ($name:ident) => (
		impl From<concat_idents!($name, Error)> for Error {
			fn from(err: concat_idents!($name, Error)) -> Error {
				Error:: $name (err)
			}
		}
    )
}
assimilate!(FromHex);
assimilate!(BaseData);*/
