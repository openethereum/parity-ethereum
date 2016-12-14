// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! Helpers for decoding and verifying responses for headers.

use std::fmt;

use ethcore::header::Header;
use light::request::{HashOrNumber, Headers as HeadersRequest};
use rlp::{DecoderError, UntrustedRlp, View};
use util::H256;

/// Errors found when decoding headers and verifying with basic constraints.
#[derive(Debug, Clone)]
pub enum BasicError {
	/// Wrong skip value: expected, found (if any).
	WrongSkip(u64, Option<u64>),
	/// Wrong start number.
	WrongStartNumber(u64, u64),
	/// Wrong start hash.
	WrongStartHash(H256, H256),
	/// Too many headers.
	TooManyHeaders(usize, usize),
	/// Decoder error.
	Decoder(DecoderError),
}

impl From<DecoderError> for BasicError {
	fn from(err: DecoderError) -> Self {
		BasicError::Decoder(err)
	}
}

impl fmt::Display for BasicError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Header response verification error: ");

		match *self {
			BasicError::WrongSkip(ref exp, ref got)
				=> write!(f, "wrong skip (expected {}, got {:?})", exp, got),
			BasicError::WrongStartNumber(ref exp, ref got)
				=> write!(f, "wrong start number (expected {}, got {})", exp, got),
			BasicError::WrongStartHash(ref exp, ref got)
				=> write!(f, "wrong start hash (expected {}, got {})", exp, got),
			BasicError::TooManyHeaders(ref max, ref got)
				=> write!(f, "too many headers (max {}, got {})", max, got),
			BasicError::Decoder(ref err)
				=> write!(f, "invalid encoding ({})", err),
		}
	}
}

/// Request verification constraint.
pub trait Constraint {
	type Error;

	/// Verify headers against this.
	fn verify(&self, headers: &[Header], reverse: bool) -> Result<(), Self::Error>;
}

/// Decode a response and do basic verification against a request.
pub fn decode_and_verify(headers: &[Bytes], request: &HeadersRequest) -> Result<Vec<Header>, BasicError> {
	let headers: Vec<_> = try!!(headers.iter().map(|x| UntrustedRlp::new(&x).as_val()).collect());

	let reverse = request.reverse;

	try!(Max(request.max).verify(&headers, reverse));
	match request.start {
		HashOrNumber::Number(ref num) => try!(StartsAtNumber(*num).verify(&headers, reverse)),
		HashOrNumber::Hash(ref hash) => try!(StartsAtHash(*hash).verify(&headers, reverse)),
	}

	try!(SkipsBetween(request.skip).verify(&headers, reverse));
}

struct StartsAtNumber(u64);
struct StartsAtHash(H256);
struct SkipsBetween(u64);
struct Max(usize);

impl Constraint for StartsAtNumber {
	type Error = BasicError;

	fn verify(&self, headers: &[Header], reverse: bool) -> Result<(), BasicError> {
		let earliest = if reverse { headers.last() } else {	headers.first() };

		earliest.map_or(Ok(()), |h| {
			if h.number() == self.0 {
				Ok(())
			} else {
				Err(BasicError::WrongStartNumber(self.0, h.number()))
			}
		})
	}
}

impl Constraint for StartsAtHash {
	type Error = BasicError;

	fn verify(&self, headers: &[Header], reverse: bool) -> Result<(), BasicError> {
		let earliest = if reverse { headers.last() } else {	headers.first() };

		earliest.map_or(Ok(()), |h| {
			if h.hash() == self.0 {
				Ok(())
			} else {
				Err(BasicError::WrongStartHash(self.0, h.hash()))
			}
		})
	}
}

impl Constraint for SkipsBetween {
	type Error = BasicError;

	fn verify(&self, headers: &[Header], reverse: bool) -> Result<(), BasicError> {
		for pair in headers.windows(2) {
			let (low, high) = if reverse { (&pair[1], &pair[0]) } else { (&pair[0], &pair[1]) };
			if low.number() >= high.number() { return Err(BasicError::WrongSkip(self.0, None)) }

			let skip = (high.number() - low.number()) - 1;
			if skip != self.0 { return Err(BasicError::WrongSkip(self.0, Some(skip))) }
		}

		Ok(())
	}
}

impl Constraint for Max {
	type Error = BasicError;

	fn verify(&self, headers: &[Header], _reverse: bool) -> Result<(), BasicError> {
		match headers.len() > self.0 {
			true => Err(BasicError::TooManyHeaders(self.0, headers.len())),
			false => Ok(())
		}
	}
}

