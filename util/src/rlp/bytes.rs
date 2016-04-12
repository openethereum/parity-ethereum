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

//! Unified interfaces for RLP bytes operations on basic types
//!

use std::mem;
use std::fmt;
use std::cmp::Ordering;
use std::error::Error as StdError;
use bigint::uint::{Uint, U128, U256};
use hash::FixedHash;
use elastic_array::*;

/// Vector like object
pub trait VecLike<T> {
	/// Add an element to the collection
    fn vec_push(&mut self, value: T);

	/// Add a slice to the collection
    fn vec_extend(&mut self, slice: &[T]);
}

impl<T> VecLike<T> for Vec<T> where T: Copy {
	fn vec_push(&mut self, value: T) {
		Vec::<T>::push(self, value)
	}

	fn vec_extend(&mut self, slice: &[T]) {
		Vec::<T>::extend_from_slice(self, slice)
	}
}

macro_rules! impl_veclike_for_elastic_array {
	($from: ident) => {
		impl<T> VecLike<T> for $from<T> where T: Copy {
			fn vec_push(&mut self, value: T) {
				$from::<T>::push(self, value)
			}
			fn vec_extend(&mut self, slice: &[T]) {
				$from::<T>::append_slice(self, slice)

			}
		}
	}
}

impl_veclike_for_elastic_array!(ElasticArray16);
impl_veclike_for_elastic_array!(ElasticArray32);
impl_veclike_for_elastic_array!(ElasticArray1024);

/// Converts given type to its shortest representation in bytes
///
/// TODO: optimise some conversations
pub trait ToBytes {
	/// Serialize self to byte array
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V);
	/// Get length of serialized data in bytes
	fn to_bytes_len(&self) -> usize;
}

impl <'a> ToBytes for &'a str {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		out.vec_extend(self.as_bytes());
	}

	fn to_bytes_len(&self) -> usize {
		self.as_bytes().len()
	}
}

impl ToBytes for String {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		out.vec_extend(self.as_bytes());
	}

	fn to_bytes_len(&self) -> usize {
		self.len()
	}
}

impl ToBytes for u64 {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		let count = self.to_bytes_len();
		for i in 0..count {
			let j = count - 1 - i;
			out.vec_push((*self >> (j * 8)) as u8);
		}
	}

	fn to_bytes_len(&self) -> usize { 8 - self.leading_zeros() as usize / 8 }
}

impl ToBytes for bool {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		out.vec_push(if *self { 1u8 } else { 0u8 })
	}

	fn to_bytes_len(&self) -> usize { 1 }
}

macro_rules! impl_map_to_bytes {
	($from: ident, $to: ty) => {
		impl ToBytes for $from {
			fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
				(*self as $to).to_bytes(out)
			}

			fn to_bytes_len(&self) -> usize { (*self as $to).to_bytes_len() }
		}
	}
}

impl_map_to_bytes!(usize, u64);
impl_map_to_bytes!(u16, u64);
impl_map_to_bytes!(u32, u64);

macro_rules! impl_uint_to_bytes {
	($name: ident) => {
		impl ToBytes for $name {
			fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
				let count = self.to_bytes_len();
				for i in 0..count {
					let j = count - 1 - i;
					out.vec_push(self.byte(j));
				}
			}
			fn to_bytes_len(&self) -> usize { (self.bits() + 7) / 8 }
		}
	}
}

impl_uint_to_bytes!(U256);
impl_uint_to_bytes!(U128);

impl <T>ToBytes for T where T: FixedHash {
	fn to_bytes<V: VecLike<u8>>(&self, out: &mut V) {
		out.vec_extend(self.bytes());
	}
	fn to_bytes_len(&self) -> usize { self.bytes().len() }
}

/// Error returned when `FromBytes` conversation goes wrong
#[derive(Debug, PartialEq, Eq)]
pub enum FromBytesError {
	/// Expected more RLP data
	DataIsTooShort,
	/// Extra bytes after the end of the last item
	DataIsTooLong,
	/// Integer-representation is non-canonically prefixed with zero byte(s).
	ZeroPrefixedInt,
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
		Ok(::std::str::from_utf8(bytes).unwrap().to_owned())
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

impl <T>FromBytes for T where T: FixedHash {
	fn from_bytes(bytes: &[u8]) -> FromBytesResult<T> {
		match bytes.len().cmp(&T::len()) {
			Ordering::Less => return Err(FromBytesError::DataIsTooShort),
			Ordering::Greater => return Err(FromBytesError::DataIsTooLong),
			Ordering::Equal => ()
		};

		unsafe {
			use std::{mem, ptr};

			let mut res: T = mem::uninitialized();
			ptr::copy(bytes.as_ptr(), res.as_slice_mut().as_mut_ptr(), T::len());

			Ok(res)
		}
	}
}

