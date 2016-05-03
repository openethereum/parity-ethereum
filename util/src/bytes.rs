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

//! Unified interfaces for bytes operations on basic types
//!
//! # Examples
//! ```rust
//! extern crate ethcore_util as util;
//!
//! fn bytes_convertable() {
//! 	use util::bytes::BytesConvertable;
//!
//! 	let arr = [0; 5];
//! 	let slice: &[u8] = arr.bytes();
//! }
//!
//! fn main() {
//! 	bytes_convertable();
//! }
//! ```

use std::fmt;
use std::slice;
use std::ops::{Deref, DerefMut};
use hash::FixedHash;
use elastic_array::*;
use std::mem;
use std::cmp::Ordering;

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

/// Slie pretty print helper
pub struct PrettySlice<'a> (&'a [u8]);

impl<'a> fmt::Debug for PrettySlice<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for i in 0..self.0.len() {
			match i > 0 {
				true => { try!(write!(f, "·{:02x}", self.0[i])); },
				false => { try!(write!(f, "{:02x}", self.0[i])); },
			}
		}
		Ok(())
	}
}

impl<'a> fmt::Display for PrettySlice<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for i in 0..self.0.len() {
			try!(write!(f, "{:02x}", self.0[i]));
		}
		Ok(())
	}
}

/// Trait to allow a type to be pretty-printed in `format!`, where unoverridable
/// defaults cannot otherwise be avoided.
pub trait ToPretty {
	/// Convert a type into a derivative form in order to make `format!` print it prettily.
	fn pretty(&self) -> PrettySlice;
	/// Express the object as a hex string.
	fn to_hex(&self) -> String {
		format!("{}", self.pretty())
	}
}

impl<'a> ToPretty for &'a [u8] {
	fn pretty(&self) -> PrettySlice {
		PrettySlice(self)
	}
}

impl<'a> ToPretty for &'a Bytes {
	fn pretty(&self) -> PrettySlice {
		PrettySlice(self.bytes())
	}
}
impl ToPretty for Bytes {
	fn pretty(&self) -> PrettySlice {
		PrettySlice(self.bytes())
	}
}

/// A byte collection reference that can either be a slice or a vector
pub enum BytesRef<'a> {
	/// This is a reference to a vector
	Flexible(&'a mut Bytes),
	/// This is a reference to a slice
	Fixed(&'a mut [u8])
}

impl<'a> Deref for BytesRef<'a> {
	type Target = [u8];

	fn deref(&self) -> &[u8] {
		match *self {
			BytesRef::Flexible(ref bytes) => bytes,
			BytesRef::Fixed(ref bytes) => bytes,
		}
	}
}

impl <'a> DerefMut for BytesRef<'a> {
	fn deref_mut(&mut self) -> &mut [u8] {
		match *self {
			BytesRef::Flexible(ref mut bytes) => bytes,
			BytesRef::Fixed(ref mut bytes) => bytes,
		}
	}
}

/// Vector of bytes
pub type Bytes = Vec<u8>;

/// Slice of bytes to underlying memory
pub trait BytesConvertable {
	// TODO: rename to as_slice
	/// Get the underlying byte-wise representation of the value.
	/// Deprecated - use `as_slice` instead.
	fn bytes(&self) -> &[u8];
	/// Get the underlying byte-wise representation of the value.
	fn as_slice(&self) -> &[u8] { self.bytes() }
	/// Get a copy of the underlying byte-wise representation.
	fn to_bytes(&self) -> Bytes { self.as_slice().to_vec() }
}

impl<T> BytesConvertable for T where T: AsRef<[u8]> {
	fn bytes(&self) -> &[u8] { self.as_ref() }
}

#[test]
fn bytes_convertable() {
	assert_eq!(vec![0x12u8, 0x34].bytes(), &[0x12u8, 0x34]);
	assert!([0u8; 0].as_slice().is_empty());
}

/// Simple trait to allow for raw population of a Sized object from a byte slice.
pub trait Populatable {
	/// Copies a bunch of bytes `d` to `self`, overwriting as necessary.
	///
	/// If `d` is smaller, zero-out the remaining bytes.
	fn populate_raw(&mut self, d: &[u8]) {
		let mut s = self.as_slice_mut();
		for i in 0..s.len() {
			s[i] = if i < d.len() {d[i]} else {0};
		}
	}

	/// Copies a bunch of bytes `d` to `self`, overwriting as necessary.
	///
	/// If `d` is smaller, will leave some bytes untouched.
	fn copy_raw(&mut self, d: &[u8]) {
		use std::io::Write;
		self.as_slice_mut().write(&d).unwrap();
	}

	/// Copies the raw representation of an object `d` to `self`, overwriting as necessary.
	///
	/// If `d` is smaller, zero-out the remaining bytes.
	fn populate_raw_from(&mut self, d: &BytesConvertable) { self.populate_raw(d.as_slice()); }

	/// Copies the raw representation of an object `d` to `self`, overwriting as necessary.
	///
	/// If `d` is smaller, will leave some bytes untouched.
	fn copy_raw_from(&mut self, d: &BytesConvertable) { self.copy_raw(d.as_slice()); }

	/// Get the raw slice for this object.
	fn as_slice_mut(&mut self) -> &mut [u8];
}

impl<T> Populatable for T where T: Sized {
	fn as_slice_mut(&mut self) -> &mut [u8] {
		use std::mem;
		unsafe {
			slice::from_raw_parts_mut(self as *mut T as *mut u8, mem::size_of::<T>())
		}
	}
}

impl<T> Populatable for [T] where T: Sized {
	fn as_slice_mut(&mut self) -> &mut [u8] {
		use std::mem;
		unsafe {
			slice::from_raw_parts_mut(self.as_mut_ptr() as *mut u8, mem::size_of::<T>() * self.len())
		}
	}
}

#[derive(Debug)]
/// Bytes array deserialization error
pub enum FromBytesError {
	/// Not enough bytes for the requested type
	NotLongEnough,
	/// Too many bytes for the requested type
	TooLong,
	/// Invalid marker for (enums)
	UnknownMarker,
}

/// Value that can be serialized from bytes array
pub trait FromRawBytes : Sized {
	/// function that will instantiate and initialize object from slice
	fn from_bytes(d: &[u8]) -> Result<Self, FromBytesError>;
}

impl<T> FromRawBytes for T where T: FixedHash {
	fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
		match bytes.len().cmp(&mem::size_of::<T>()) {
			Ordering::Less => return Err(FromBytesError::NotLongEnough),
			Ordering::Greater => return Err(FromBytesError::TooLong),
			Ordering::Equal => ()
		};

		let mut res: Self = unsafe { mem::uninitialized() };
		res.copy_raw(bytes);
		Ok(res)
	}
}

#[macro_export]
macro_rules! sized_binary_map {
	($target_ty: ident) => {
		impl FromRawBytes for $target_ty {
			fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
				match bytes.len().cmp(&::std::mem::size_of::<$target_ty>()) {
					::std::cmp::Ordering::Less => return Err(FromBytesError::NotLongEnough),
					::std::cmp::Ordering::Greater => return Err(FromBytesError::TooLong),
					::std::cmp::Ordering::Equal => ()
				};
				let mut res: Self = unsafe { ::std::mem::uninitialized() };
				res.copy_raw(bytes);
				Ok(res)
			}
		}
		impl ToBytesWithMap for $target_ty {
			fn to_bytes_map(&self) -> Vec<u8> {
				let sz = ::std::mem::size_of::<$target_ty>();
				let mut res = Vec::<u8>::with_capacity(sz);

				let ip: *const $target_ty = self;
				let ptr: *const u8 = ip as *const _;
				unsafe {
					res.set_len(sz);
					::std::ptr::copy(ptr, res.as_mut_ptr(), sz);
				}
				res
			}
		}
	}
}

sized_binary_map!(u16);
sized_binary_map!(u32);
sized_binary_map!(u64);

/// Value that can be serialized from variable-length byte array
pub trait FromRawBytesVariable : Sized {
	/// Create value from slice
	fn from_bytes_variable(bytes: &[u8]) -> Result<Self, FromBytesError>;
}

impl<T> FromRawBytesVariable for T where T: FromRawBytes {
	fn from_bytes_variable(bytes: &[u8]) -> Result<Self, FromBytesError> {
		match bytes.len().cmp(&mem::size_of::<T>()) {
			Ordering::Less => return Err(FromBytesError::NotLongEnough),
			Ordering::Greater => return Err(FromBytesError::TooLong),
			Ordering::Equal => ()
		};

		T::from_bytes(bytes)
	}
}

impl FromRawBytesVariable for String {
	fn from_bytes_variable(bytes: &[u8]) -> Result<String, FromBytesError> {
		Ok(::std::str::from_utf8(bytes).unwrap().to_owned())
	}
}

impl<T> FromRawBytesVariable for Vec<T> where T: FromRawBytes {
	fn from_bytes_variable(bytes: &[u8]) -> Result<Self, FromBytesError> {
		let size_of_t = mem::size_of::<T>();
		let length_in_chunks = bytes.len() / size_of_t;

		let mut result = Vec::with_capacity(length_in_chunks );
		unsafe { result.set_len(length_in_chunks) };
		for i in 0..length_in_chunks {
			*result.get_mut(i).unwrap() = try!(T::from_bytes(
				&bytes[size_of_t * i..size_of_t * (i+1)]))
		}
		Ok(result)
	}
}

impl<V1, T2> FromRawBytes for (V1, T2) where V1: FromRawBytesVariable, T2: FromRawBytes {
	fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
		let header = 8usize;
		let mut map: (u64, ) = unsafe { mem::uninitialized() };

		if bytes.len() < header { return  Err(FromBytesError::NotLongEnough); }
		map.copy_raw(&bytes[0..header]);

		Ok((
			try!(V1::from_bytes_variable(&bytes[header..header + (map.0 as usize)])),
			try!(T2::from_bytes(&bytes[header + (map.0 as usize)..bytes.len()])),
		))
	}
}

impl<V1, V2, T3> FromRawBytes for (V1, V2, T3)
	where V1: FromRawBytesVariable,
		V2: FromRawBytesVariable,
		T3: FromRawBytes
{
	fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
		let header = 16usize;
		let mut map: (u64, u64, ) = unsafe { mem::uninitialized() };

		if bytes.len() < header { return  Err(FromBytesError::NotLongEnough); }
		map.copy_raw(&bytes[0..header]);

		let map_1 = (header, header + map.0 as usize);
		let map_2 = (map_1.1 as usize, map_1.1 as usize + map.1 as usize);
		Ok((
			try!(V1::from_bytes_variable(&bytes[map_1.0..map_1.1])),
			try!(V2::from_bytes_variable(&bytes[map_2.0..map_2.1])),
			try!(T3::from_bytes(&bytes[map_2.1..bytes.len()])),
		))
	}
}

impl<'a, V1, T2> ToBytesWithMap for (&'a Vec<V1>, &'a T2) where V1: ToBytesWithMap, T2: ToBytesWithMap {
	fn to_bytes_map(&self) -> Vec<u8> {
		let header = 8usize;
		let v1_size = mem::size_of::<V1>();
		let mut result = Vec::with_capacity(header + self.0.len() * v1_size + mem::size_of::<T2>());
		result.extend(((self.0.len() * v1_size) as u64).to_bytes_map());

		for i in 0..self.0.len() {
			result.extend(self.0[i].to_bytes_map());
		}
		result.extend(self.1.to_bytes_map());

		result
	}

}

impl<'a, V1, V2, T3> ToBytesWithMap for (&'a Vec<V1>, &'a Vec<V2>, &'a T3)
	where V1: ToBytesWithMap,
		V2: ToBytesWithMap,
		T3: ToBytesWithMap
{
	fn to_bytes_map(&self) -> Vec<u8> {
		let header = 16usize;
		let v1_size = mem::size_of::<V1>();
		let v2_size = mem::size_of::<V2>();
		let mut result = Vec::with_capacity(
			header +
			self.0.len() * v1_size +
			self.1.len() * v2_size +
			mem::size_of::<T3>()
		);
		result.extend(((self.0.len() * v1_size) as u64).to_bytes_map());
		result.extend(((self.1.len() * v2_size) as u64).to_bytes_map());
		for i in 0..self.0.len() {
			result.extend(self.0[i].to_bytes_map());
		}
		for i in 0..self.1.len() {
			result.extend(self.1[i].to_bytes_map());
		}
		result.extend(self.2.to_bytes_map());

		result
	}
}

impl FromRawBytesVariable for Vec<u8> {
	fn from_bytes_variable(bytes: &[u8]) -> Result<Vec<u8>, FromBytesError> {
		Ok(bytes.to_vec())
	}
}

/// Value that serializes directly to variable-sized byte array and stores map
pub trait ToBytesWithMap {
	/// serialize to variable-sized byte array and store map
	fn to_bytes_map(&self) -> Vec<u8>;
}

impl<T> ToBytesWithMap for T where T: FixedHash {
	fn to_bytes_map(&self) -> Vec<u8> {
		self.as_slice().to_vec()
	}
}

#[test]
fn fax_raw() {
	let mut x = [255u8; 4];
	x.copy_raw(&[1u8; 2][..]);
	assert_eq!(x, [1u8, 1, 255, 255]);
	let mut x = [255u8; 4];
	x.copy_raw(&[1u8; 6][..]);
	assert_eq!(x, [1u8, 1, 1, 1]);
}

#[test]
fn populate_raw() {
	let mut x = [255u8; 4];
	x.populate_raw(&[1u8; 2][..]);
	assert_eq!(x, [1u8, 1, 0, 0]);
	let mut x = [255u8; 4];
	x.populate_raw(&[1u8; 6][..]);
	assert_eq!(x, [1u8, 1, 1, 1]);
}

#[test]
fn populate_raw_dyn() {
	let mut x = [255u8; 4];
	x.populate_raw(&[1u8; 2][..]);
	assert_eq!(&x[..], [1u8, 1, 0, 0]);
	let mut x = [255u8; 4];
	x.populate_raw(&[1u8; 6][..]);
	assert_eq!(&x[..], [1u8, 1, 1, 1]);
}

#[test]
fn fax_raw_dyn() {
	let mut x = [255u8; 4];
	x.copy_raw(&[1u8; 2][..]);
	assert_eq!(&x[..], [1u8, 1, 255, 255]);
	let mut x = [255u8; 4];
	x.copy_raw(&[1u8; 6][..]);
	assert_eq!(&x[..], [1u8, 1, 1, 1]);
}

#[test]
fn populate_big_types() {
	use hash::*;
	let a = address_from_hex("ffffffffffffffffffffffffffffffffffffffff");
	let mut h = h256_from_u64(0x69);
	h.populate_raw_from(&a);
	assert_eq!(h, h256_from_hex("ffffffffffffffffffffffffffffffffffffffff000000000000000000000000"));
	let mut h = h256_from_u64(0x69);
	h.copy_raw_from(&a);
	assert_eq!(h, h256_from_hex("ffffffffffffffffffffffffffffffffffffffff000000000000000000000069"));
}

#[test]
fn raw_bytes_from_tuple() {
	type Tup = (Vec<u16>, u16);

	let tup = (vec![1u16, 1u16, 1u16, 1u16], 10u16);
	let bytes = vec![
		// map
		8u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		// four 1u16
		1u8, 0u8,
		1u8, 0u8,
		1u8, 0u8,
		1u8, 0u8,
		// 10u16
		10u8, 0u8];

	let tup_from = Tup::from_bytes(&bytes).unwrap();
	assert_eq!(tup, tup_from);

	let tup_to = (&tup_from.0, &tup_from.1);
	let bytes_to = tup_to.to_bytes_map();
	assert_eq!(bytes_to, bytes);
}

#[test]
fn bytes_map_from_triple() {
	let data = (vec![2u16; 6], vec![6u32; 3], 12u64);
	let bytes_map = (&data.0, &data.1, &data.2).to_bytes_map();
	assert_eq!(bytes_map, vec![
		// data map 2 x u64
		12, 0, 0, 0, 0, 0, 0, 0,
		12, 0, 0, 0, 0, 0, 0, 0,
		// vec![2u16; 6]
		2, 0, 2, 0, 2, 0, 2, 0, 2, 0, 2, 0,
		// vec![6u32; 3]
		6, 0, 0, 0, 6, 0, 0, 0, 6, 0, 0, 0,
		// 12u64
		12, 0, 0, 0, 0, 0, 0, 0]);
}
