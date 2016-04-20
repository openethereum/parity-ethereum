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

//! Binary representation of types

use util::bytes::*;
use std::mem;

pub struct BinaryConvertError;

pub trait BinaryConvertable : Sized {
	fn size(&self) -> usize;

	fn to_bytes(&self, buffer: &mut [u8]) -> Result<(), BinaryConvertError>;

	fn from_bytes(buffer: &[u8]) -> Result<Self, BinaryConvertError>;

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Err(BinaryConvertError)
	}

	fn len_params() -> usize {
		0
	}
}

impl<T> BinaryConvertable for Option<T> where T: BinaryConvertable {
	fn size(&self) -> usize {
		match * self { None => 0, Some(ref val) => val.size() }
	}

	fn to_bytes(&self, buffer: &mut [u8]) -> Result<(), BinaryConvertError> {
		match *self { None => Err(BinaryConvertError), Some(ref val) => val.to_bytes(buffer) }
	}

	fn from_bytes(buffer: &[u8]) -> Result<Self, BinaryConvertError> {
		Ok(Some(try!(T::from_bytes(buffer))))
	}

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Ok(None)
	}

	fn len_params() -> usize {
		1
	}
}

impl<T> BinaryConvertable for Vec<T> where T: BinaryConvertable {
	fn size(&self) -> usize {
		match T::len_params() {
			0 => mem::size_of::<T>() * self.len(),
			_ => self.iter().fold(0usize, |acc, t| acc + t.size()),
		}
	}

	fn to_bytes(&self, buffer: &mut [u8]) -> Result<(), BinaryConvertError> {
		let mut offset = 0usize;
		for (index, item) in self.iter().enumerate() {
			let item_end = offset + item.size();
			try!(item.to_bytes(&mut buffer[offset..item_end]));
			offset = item_end;
		}
		Ok(())
	}

	fn from_bytes(buffer: &[u8]) -> Result<Self, BinaryConvertError> {
		Err(BinaryConvertError)
	}

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Ok(Self::new())
	}

	fn len_params() -> usize {
		1
	}
}

macro_rules! binary_fixed_size {
	($target_ty: ident) => {
		impl BinaryConvertable for $target_ty {
			fn size(&self) -> usize {
				mem::size_of::<$target_ty>()
			}

			fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryConvertError> {
				match bytes.len().cmp(&::std::mem::size_of::<$target_ty>()) {
					::std::cmp::Ordering::Less => return Err(BinaryConvertError),
					::std::cmp::Ordering::Greater => return Err(BinaryConvertError),
					::std::cmp::Ordering::Equal => ()
				};
				let mut res: Self = unsafe { ::std::mem::uninitialized() };
				res.copy_raw(bytes);
				Ok(res)
			}

			fn to_bytes(&self, buffer: &mut [u8]) -> Result<(), BinaryConvertError> {
				let sz = ::std::mem::size_of::<$target_ty>();
				let ip: *const $target_ty = self;
				let ptr: *const u8 = ip as *const _;
				unsafe {
					::std::ptr::copy(ptr, buffer.as_mut_ptr(), sz);
				}
				Ok(())
			}
		}
	}
}

binary_fixed_size!(u64);
binary_fixed_size!(u32);
binary_fixed_size!(bool);
