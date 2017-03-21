// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Unified interfaces for RLP bytes operations on basic types
//!

use std::{mem, fmt, cmp};
use std::error::Error as StdError;
use bigint::prelude::{U128, U256, H64, H128, H160, H256, H512, H520, H2048};

/// Error returned when `FromBytes` conversation goes wrong
#[derive(Debug, PartialEq, Eq)]
pub enum FromBytesError {
	/// Expected more RLP data
	DataIsTooShort,
	/// Extra bytes after the end of the last item
	DataIsTooLong,
	/// Integer-representation is non-canonically prefixed with zero byte(s).
	ZeroPrefixedInt,
	/// String representation is not utf-8
	InvalidUtf8,
}

impl StdError for FromBytesError {
	fn description(&self) -> &str { "from_bytes error" }
}

impl fmt::Display for FromBytesError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

/// Alias for the result of `FromBytes` trait
pub type FromBytesResult<T> = Result<T, FromBytesError>;

/// Converts to given type from its bytes representation
///
/// TODO: check size of bytes before conversation and return appropriate error
pub trait FromBytes: Sized {
	/// Create a value from bytes
	fn from_bytes(bytes: &[u8]) -> FromBytesResult<Self>;
}

impl FromBytes for String {
	fn from_bytes(bytes: &[u8]) -> FromBytesResult<String> {
		::std::str::from_utf8(bytes).map(|s| s.to_owned()).map_err(|_| FromBytesError::InvalidUtf8)
	}
}

macro_rules! impl_uint_from_bytes {
	($to: ident) => {
		impl FromBytes for $to {
			fn from_bytes(bytes: &[u8]) -> FromBytesResult<$to> {
				match bytes.len() {
					0 => Ok(0),
					l if l <= mem::size_of::<$to>() => {
						if bytes[0] == 0 {
							return Err(FromBytesError::ZeroPrefixedInt)
						}
						let mut res = 0 as $to;
						for i in 0..l {
							let shift = (l - 1 - i) * 8;
							res = res + ((bytes[i] as $to) << shift);
						}
						Ok(res)
					}
					_ => Err(FromBytesError::DataIsTooLong)
				}
			}
		}
	}
}

impl FromBytes for bool {
	fn from_bytes(bytes: &[u8]) -> FromBytesResult<bool> {
		match bytes.len() {
			0 => Ok(false),
			1 => Ok(bytes[0] != 0),
			_ => Err(FromBytesError::DataIsTooLong),
		}
	}
}

//impl_uint_from_bytes!(u8);
impl_uint_from_bytes!(u16);
impl_uint_from_bytes!(u32);
impl_uint_from_bytes!(u64);
impl_uint_from_bytes!(usize);

macro_rules! impl_uint_from_bytes {
	($name: ident, $size: expr) => {
		impl FromBytes for $name {
			fn from_bytes(bytes: &[u8]) -> FromBytesResult<$name> {
				if !bytes.is_empty() && bytes[0] == 0 {
					Err(FromBytesError::ZeroPrefixedInt)
				} else if bytes.len() <= $size {
					Ok($name::from(bytes))
				} else {
					Err(FromBytesError::DataIsTooLong)
				}
			}
		}
	}
}

impl_uint_from_bytes!(U256, 32);
impl_uint_from_bytes!(U128, 16);

macro_rules! impl_hash_from_bytes {
	($name: ident, $size: expr) => {
		impl FromBytes for $name {
			fn from_bytes(bytes: &[u8]) -> FromBytesResult<$name> {
				match bytes.len().cmp(&$size) {
					cmp::Ordering::Less => Err(FromBytesError::DataIsTooShort),
					cmp::Ordering::Greater => Err(FromBytesError::DataIsTooLong),
					cmp::Ordering::Equal => {
						let mut t = [0u8; $size];
						t.copy_from_slice(bytes);
						Ok($name(t))
					}
				}
			}
		}
	}
}

impl_hash_from_bytes!(H64, 8);
impl_hash_from_bytes!(H128, 16);
impl_hash_from_bytes!(H160, 20);
impl_hash_from_bytes!(H256, 32);
impl_hash_from_bytes!(H512, 64);
impl_hash_from_bytes!(H520, 65);
impl_hash_from_bytes!(H2048, 256);

