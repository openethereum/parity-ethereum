// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! General error types for use in ethcore.

use rustc_serialize::hex::FromHexError;
use network::NetworkError;
use rlp::DecoderError;
use io;
use std::fmt;
use hash::H256;

#[derive(Debug)]
/// Error in database subsystem.
pub enum BaseDataError {
	/// An entry was removed more times than inserted.
	NegativelyReferencedHash(H256),
}

impl fmt::Display for BaseDataError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			BaseDataError::NegativelyReferencedHash(hash) =>
				f.write_fmt(format_args!("Entry {} removed from database more times \
					than it was added.", hash)),
		}
	}
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
	/// Error from snappy.
	Snappy(::snappy::Error),
}

impl fmt::Display for UtilError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			UtilError::Crypto(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::StdIo(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::Io(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::AddressParse(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::AddressResolve(Some(ref err)) => f.write_fmt(format_args!("{}", err)),
			UtilError::AddressResolve(_) => f.write_str("Failed to resolve network address."),
			UtilError::FromHex(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::BaseData(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::Network(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::Decoder(ref err) => f.write_fmt(format_args!("{}", err)),
			UtilError::SimpleString(ref msg) => f.write_str(&msg),
			UtilError::BadSize => f.write_str("Bad input size."),
			UtilError::Snappy(ref err) => f.write_fmt(format_args!("{}", err)),
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
/// Error indicating an expected value was not found.
pub struct Mismatch<T: fmt::Debug> {
	/// Value expected.
	pub expected: T,
	/// Value found.
	pub found: T,
}

impl<T: fmt::Debug + fmt::Display> fmt::Display for Mismatch<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_fmt(format_args!("Expected {}, found {}", self.expected, self.found))
	}
}

#[derive(Debug, PartialEq, Eq)]
/// Error indicating value found is outside of a valid range.
pub struct OutOfBounds<T: fmt::Debug> {
	/// Minimum allowed value.
	pub min: Option<T>,
	/// Maximum allowed value.
	pub max: Option<T>,
	/// Value found.
	pub found: T,
}

impl<T: fmt::Debug + fmt::Display> fmt::Display for OutOfBounds<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let msg = match (self.min.as_ref(), self.max.as_ref()) {
			(Some(min), Some(max)) => format!("Min={}, Max={}", min, max),
			(Some(min), _) => format!("Min={}", min),
			(_, Some(max)) => format!("Max={}", max),
			(None, None) => "".into(),
		};

		f.write_fmt(format_args!("Value {} out of bounds. {}", self.found, msg))
	}
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

impl From<::snappy::Error> for UtilError {
	fn from(err: ::snappy::Error) -> UtilError {
		UtilError::Snappy(err)
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
