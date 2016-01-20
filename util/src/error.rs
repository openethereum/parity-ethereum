//! General error types for use in ethcore.

use rustc_serialize::hex::FromHexError;
use network::NetworkError;
use rlp::DecoderError;
use io;

#[derive(Debug)]
/// TODO [Gav Wood] Please document me
pub enum BaseDataError {
	/// TODO [Gav Wood] Please document me
	NegativelyReferencedHash,
}

#[derive(Debug)]
/// General error type which should be capable of representing all errors in ethcore.
pub enum UtilError {
	/// TODO [Gav Wood] Please document me
	Crypto(::crypto::CryptoError),
	/// TODO [Gav Wood] Please document me
	StdIo(::std::io::Error),
	/// TODO [Gav Wood] Please document me
	Io(io::IoError),
	/// TODO [Gav Wood] Please document me
	AddressParse(::std::net::AddrParseError),
	/// TODO [Gav Wood] Please document me
	AddressResolve(Option<::std::io::Error>),
	/// TODO [Gav Wood] Please document me
	FromHex(FromHexError),
	/// TODO [Gav Wood] Please document me
	BaseData(BaseDataError),
	/// TODO [Gav Wood] Please document me
	Network(NetworkError),
	/// TODO [Gav Wood] Please document me
	Decoder(DecoderError),
	/// TODO [Gav Wood] Please document me
	SimpleString(String),
	/// TODO [Gav Wood] Please document me
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
