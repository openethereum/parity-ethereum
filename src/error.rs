//! General error types for use in ethcore.

use rustc_serialize::hex::FromHexError;
use network::NetworkError;
use rlp::DecoderError;

#[derive(Debug)]
pub enum BaseDataError {
	NegativelyReferencedHash,
}

#[derive(Debug)]
/// General error type which should be capable of representing all errors in ethcore.
pub enum UtilError {
	Crypto(::crypto::CryptoError),
	Io(::std::io::Error),
	AddressParse(::std::net::AddrParseError),
	AddressResolve(Option<::std::io::Error>),
	FromHex(FromHexError),
	BaseData(BaseDataError),
	Network(NetworkError),
	Decoder(DecoderError),
	BadSize,
	UnknownName,
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
