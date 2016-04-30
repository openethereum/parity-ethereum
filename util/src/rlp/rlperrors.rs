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

use std::fmt;
use std::error::Error as StdError;
use rlp::bytes::FromBytesError;

#[derive(Debug, PartialEq, Eq)]
/// Error concerning the RLP decoder.
pub enum DecoderError {
	/// Couldn't convert given bytes to an instance of required type.
	FromBytesError(FromBytesError),
	/// Data has additional bytes at the end of the valid RLP fragment.
	RlpIsTooBig,
	/// Data has too few bytes for valid RLP.
	RlpIsTooShort,
	/// Expect an encoded list, RLP was something else.
	RlpExpectedToBeList,
	/// Expect encoded data, RLP was something else.
	RlpExpectedToBeData,
	/// Expected a different size list.
	RlpIncorrectListLen,
	/// Data length number has a prefixed zero byte, invalid for numbers.
	RlpDataLenWithZeroPrefix,
	/// List length number has a prefixed zero byte, invalid for numbers.
	RlpListLenWithZeroPrefix,
	/// Non-canonical (longer than necessary) representation used for data or list.
	RlpInvalidIndirection,
	/// Declared length is inconsistent with data specified after.
	RlpInconsistentLengthAndData,
	/// Custom rlp decoding error.
	Custom(&'static str),
}

impl StdError for DecoderError {
	fn description(&self) -> &str {
		"builder error"
	}
}

impl fmt::Display for DecoderError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

impl From<FromBytesError> for DecoderError {
	fn from(err: FromBytesError) -> DecoderError {
		DecoderError::FromBytesError(err)
	}
}
