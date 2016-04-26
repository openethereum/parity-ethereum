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

use util::bytes::Populatable;
use util::numbers::{U256, H256, H2048, Address};
use std::mem;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BinaryConvertError;

pub trait BinaryConvertable : Sized {
	fn size(&self) -> usize {
		mem::size_of::<Self>()
	}

	fn to_bytes(&self, buffer: &mut [u8], length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError>;

	fn from_bytes(buffer: &[u8], length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError>;

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

	fn to_bytes(&self, buffer: &mut [u8], length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
		match *self { None => Err(BinaryConvertError), Some(ref val) => val.to_bytes(buffer, length_stack) }
	}

	fn from_bytes(buffer: &[u8], length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
		Ok(Some(try!(T::from_bytes(buffer, length_stack))))
	}

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Ok(None)
	}

	fn len_params() -> usize {
		1
	}
}

impl<E: BinaryConvertable> BinaryConvertable for Result<(), E> {
	fn size(&self) -> usize {
		1usize + match *self {
			Ok(_) => 0,
			Err(ref e) => e.size(),
		}
	}

	fn to_bytes(&self, buffer: &mut [u8], length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
		match *self {
			Ok(_) => Ok(()),
			Err(ref e) => Ok(try!(e.to_bytes(buffer, length_stack))),
		}
	}

	fn from_bytes(buffer: &[u8], length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
		match buffer[0] {
			0 => Ok(Ok(())),
			1 => Ok(Err(try!(E::from_bytes(&buffer[1..], length_stack)))),
			_ => Err(BinaryConvertError)
		}
	}

	fn len_params() -> usize {
		1
	}
}


impl<R: BinaryConvertable, E: BinaryConvertable> BinaryConvertable for Result<R, E> {
	fn size(&self) -> usize {
		1usize + match *self {
			Ok(ref r) => r.size(),
			Err(ref e) => e.size(),
		}
	}

	fn to_bytes(&self, buffer: &mut [u8], length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
		match *self {
			Ok(ref r) => Ok(try!(r.to_bytes(buffer, length_stack))),
			Err(ref e) => Ok(try!(e.to_bytes(buffer, length_stack))),
		}
	}

	fn from_bytes(buffer: &[u8], length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
		match buffer[0] {
			0 => Ok(Ok(try!(R::from_bytes(&buffer[1..], length_stack)))),
			1 => Ok(Err(try!(E::from_bytes(&buffer[1..], length_stack)))),
			_ => Err(BinaryConvertError)
		}
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

	fn to_bytes(&self, buffer: &mut [u8], length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
		let mut offset = 0usize;
		for item in self.iter() {
			let next_size = match T::len_params() {
				0 => mem::size_of::<T>(),
				_ => { let size = item.size(); length_stack.push_back(size); size },
			};
			if next_size > 0 {
				let item_end = offset + next_size;
				try!(item.to_bytes(&mut buffer[offset..item_end], length_stack));
				offset = item_end;
			}
		}
		Ok(())
	}

	fn from_bytes(buffer: &[u8], length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
		let mut index = 0;
		let mut result = Self::with_capacity(
			match T::len_params() {
				0 => buffer.len() /  mem::size_of::<T>(),
				_ => 128,
			});

		loop {
			let next_size = match T::len_params() {
				0 => mem::size_of::<T>(),
				_ => try!(length_stack.pop_front().ok_or(BinaryConvertError)),
			};
			let item = if next_size == 0 {
				try!(T::from_empty_bytes())
			}
			else {
				try!(T::from_bytes(&buffer[index..index+next_size], length_stack))
			};
			result.push(item);

			index = index + next_size;
			if index == buffer.len() { break; }
			if index > buffer.len() {
				return Err(BinaryConvertError)
			}

		}

		Ok(result)
	}

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Ok(Self::new())
	}

	fn len_params() -> usize {
		1
	}
}

impl BinaryConvertable for String {
	fn size(&self) -> usize {
		self.as_bytes().len()
	}

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Ok(String::new())
	}

	fn to_bytes(&self, buffer: &mut [u8], _length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
		buffer[..].clone_from_slice(self.as_bytes());
		Ok(())
	}

	fn from_bytes(buffer: &[u8], _length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
		Ok(::std::str::from_utf8(buffer).unwrap().to_owned())
	}

	fn len_params() -> usize {
		1
	}
}

impl<T> BinaryConvertable for ::std::cell::RefCell<T> where T: BinaryConvertable {
	fn size(&self) -> usize {
		self.borrow().size()
	}

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Ok(::std::cell::RefCell::new(try!(T::from_empty_bytes())))
	}

	fn from_bytes(buffer: &[u8], length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
		Ok(::std::cell::RefCell::new(try!(T::from_bytes(buffer, length_stack))))
	}

	fn to_bytes(&self, buffer: &mut [u8], length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
		try!(self.borrow().to_bytes(buffer, length_stack));
		Ok(())
	}

	fn len_params() -> usize {
		T::len_params()
	}
}

impl BinaryConvertable for Vec<u8> {
	fn size(&self) -> usize {
		self.len()
	}

	fn from_empty_bytes() -> Result<Self, BinaryConvertError> {
		Ok(Vec::new())
	}

	fn to_bytes(&self, buffer: &mut [u8], _length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
		buffer[..].clone_from_slice(&self[..]);
		Ok(())
	}

	fn from_bytes(buffer: &[u8], _length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
		let mut res = Self::with_capacity(buffer.len());
		unsafe { res.set_len(buffer.len()) }
		res[..].clone_from_slice(&buffer[..]);
		Ok(res)
	}

	fn len_params() -> usize {
		1
	}
}

pub fn deserialize_from<T, R>(r: &mut R) -> Result<T, BinaryConvertError>
	where R: ::std::io::Read,
		T: BinaryConvertable
{
	let mut fake_stack = VecDeque::new();

	match T::len_params() {
		0 => {
			let fixed_size = mem::size_of::<T>();
			let mut payload_buffer = Vec::with_capacity(fixed_size);
			unsafe { payload_buffer.set_len(fixed_size); }
			try!(r.read(&mut payload_buffer).map_err(|_| BinaryConvertError));
			T::from_bytes(&payload_buffer[..], &mut fake_stack)
		},
		_ => {
			let mut length_stack = VecDeque::<usize>::new();
			let mut size_buffer = [0u8; 8];
			try!(r.read(&mut size_buffer[..]).map_err(|_| BinaryConvertError));
			let stack_len = try!(u64::from_bytes(&mut size_buffer[..], &mut fake_stack)) as usize;
			if stack_len > 0 {
				let mut header_buffer = Vec::with_capacity(stack_len * 8);
				unsafe {  header_buffer.set_len(stack_len * 8); };

				try!(r.read(&mut header_buffer[..]).map_err(|_| BinaryConvertError));
				for idx in 0..stack_len {
					let stack_item = try!(u64::from_bytes(&header_buffer[idx*8..(idx+1)*8], &mut fake_stack));
					length_stack.push_back(stack_item as usize);
				}
			}

			try!(r.read(&mut size_buffer[..]).map_err(|_| BinaryConvertError));
			let size = try!(u64::from_bytes(&size_buffer[..], &mut fake_stack)) as usize;

			let mut data = Vec::with_capacity(size);
			unsafe { data.set_len(size) };
			try!(r.read(&mut data).map_err(|_| BinaryConvertError));

			T::from_bytes(&data[..], &mut length_stack)
		},
	}
}

pub fn deserialize<T: BinaryConvertable>(buffer: &[u8]) -> Result<T, BinaryConvertError> {
	use std::io::Cursor;
	let mut buff = Cursor::new(buffer);
	deserialize_from::<T, _>(&mut buff)
}

pub fn serialize_into<T, W>(t: &T, w: &mut W) -> Result<(), BinaryConvertError>
	where W: ::std::io::Write,
		T: BinaryConvertable
{
	let mut fake_stack = VecDeque::new();

	match T::len_params() {
		0 => {
			let fixed_size = mem::size_of::<T>();
			let mut buffer = Vec::with_capacity(fixed_size);
			unsafe { buffer.set_len(fixed_size); }
			try!(t.to_bytes(&mut buffer[..], &mut fake_stack));
			try!(w.write(&buffer[..]).map_err(|_| BinaryConvertError));
			Ok(())
		},
		_ => {
			let mut length_stack = VecDeque::<usize>::new();
			let mut size_buffer = [0u8; 8];

			let size = t.size();
			let mut buffer = Vec::with_capacity(size);
			unsafe { buffer.set_len(size); }
			try!(t.to_bytes(&mut buffer[..], &mut length_stack));

			let stack_len = length_stack.len();
			try!((stack_len as u64).to_bytes(&mut size_buffer[..], &mut fake_stack));
			try!(w.write(&size_buffer[..]).map_err(|_| BinaryConvertError));
			if stack_len > 0 {
				let mut header_buffer = Vec::with_capacity(stack_len * 8);
				unsafe {  header_buffer.set_len(stack_len * 8); };
				try!((stack_len as u64).to_bytes(&mut header_buffer[0..8], &mut fake_stack));
				let mut idx = 0;
				loop {
					match length_stack.pop_front() {
						Some(val) => try!((val as u64).to_bytes(&mut header_buffer[idx * 8..(idx+1) * 8], &mut fake_stack)),
						None => { break; }
					}
					idx = idx + 1;
				}
				try!(w.write(&header_buffer[..]).map_err(|_| BinaryConvertError));
			}

			try!((size as u64).to_bytes(&mut size_buffer[..], &mut fake_stack));
			try!(w.write(&size_buffer[..]).map_err(|_| BinaryConvertError));

			try!(w.write(&buffer[..]).map_err(|_| BinaryConvertError));

			Ok(())
		},
	}
}

pub fn serialize<T: BinaryConvertable>(t: &T) -> Result<Vec<u8>, BinaryConvertError> {
	use std::io::Cursor;
	let mut buff = Cursor::new(Vec::new());
	try!(serialize_into(t, &mut buff));
	Ok(buff.into_inner())
}

macro_rules! binary_fixed_size {
	($target_ty: ident) => {
		impl BinaryConvertable for $target_ty {
			fn from_bytes(bytes: &[u8], _length_stack: &mut VecDeque<usize>) -> Result<Self, BinaryConvertError> {
				match bytes.len().cmp(&::std::mem::size_of::<$target_ty>()) {
					::std::cmp::Ordering::Less => return Err(BinaryConvertError),
					::std::cmp::Ordering::Greater => return Err(BinaryConvertError),
					::std::cmp::Ordering::Equal => ()
				};
				let mut res: Self = unsafe { ::std::mem::uninitialized() };
				res.copy_raw(bytes);
				Ok(res)
			}

			fn to_bytes(&self, buffer: &mut [u8], _length_stack: &mut VecDeque<usize>) -> Result<(), BinaryConvertError> {
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
binary_fixed_size!(usize);
binary_fixed_size!(i32);
binary_fixed_size!(bool);
binary_fixed_size!(U256);
binary_fixed_size!(H256);
binary_fixed_size!(H2048);
binary_fixed_size!(Address);

#[test]
fn vec_serialize() {
	let mut v = Vec::new();
	v.push(5u64);
	v.push(10u64);
	let mut length_stack = VecDeque::new();
	let mut data = Vec::with_capacity(v.size());
	unsafe { data.set_len(v.size()); }
	let result = v.to_bytes(&mut data[..], &mut length_stack);

	assert!(result.is_ok());
	assert_eq!(5, data[0]);
	assert_eq!(0, data[1]);
	assert_eq!(10, data[8]);
	assert_eq!(0, data[12]);
}

#[test]
fn calculates_size() {
	let mut v = Vec::new();
	v.push(5u64);
	v.push(10u64);

	assert_eq!(16, v.size());
}

#[test]
fn vec_deserialize() {
	let data = [
		10u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		5u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
	];

	let mut length_stack = VecDeque::new();
	let vec = Vec::<u64>::from_bytes(&data[..], &mut length_stack).unwrap();

	assert_eq!(vec![10u64, 5u64], vec);
}

#[test]
fn vec_deserialize_chained() {
	let mut v = Vec::new();
	v.push(Some(5u64));
	v.push(Some(10u64));
	v.push(None);
	v.push(Some(12u64));

	let mut length_stack = VecDeque::new();
	let mut data = Vec::with_capacity(v.size());
	unsafe { data.set_len(v.size()); }
	let result = v.to_bytes(&mut data[..], &mut length_stack);

	assert!(result.is_ok());
	assert_eq!(4, length_stack.len());
}

#[test]
fn vec_serialize_deserialize() {
	let mut v = Vec::new();
	v.push(Some(5u64));
	v.push(None);
	v.push(Some(10u64));
	v.push(None);
	v.push(Some(12u64));


	let mut data = Vec::with_capacity(v.size());
	unsafe { data.set_len(v.size()); }
	let mut length_stack = VecDeque::new();

	v.to_bytes(&mut data[..], &mut length_stack).unwrap();
	let de_v = Vec::<Option<u64>>::from_bytes(&data[..], &mut length_stack).unwrap();

	assert_eq!(v, de_v);
}

#[test]
fn serialize_into_ok() {
	use std::io::Cursor;
    let mut buff = Cursor::new(vec![0; 128]);

	let mut v = Vec::new();
	v.push(Some(5u64));
	v.push(None);
	v.push(Some(10u64));
	v.push(None);
	v.push(Some(12u64));

	serialize_into(&v, &mut buff).unwrap();
	assert_eq!(5, buff.get_ref()[0]);
	assert_eq!(8, buff.get_ref()[8]);
	assert_eq!(0, buff.get_ref()[16]);
	assert_eq!(8, buff.get_ref()[24]);
}

#[test]
fn deserialize_from_ok() {
	use std::io::Cursor;
    let mut buff = Cursor::new(vec![
		0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		16u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		10u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		5u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
	]);

	let vec = deserialize_from::<Vec<u64>, _>(&mut buff).unwrap();

	assert_eq!(vec![10u64, 5u64], vec);
}

#[test]
fn serialize_into_deserialize_from() {
	use std::io::{Cursor, SeekFrom, Seek};

	let mut buff = Cursor::new(vec![0u8; 1024]);
	let mut v = Vec::new();
	v.push(Some(5u64));
	v.push(None);
	v.push(Some(10u64));
	v.push(None);
	v.push(Some(12u64));

	serialize_into(&v, &mut buff).unwrap();
	buff.seek(SeekFrom::Start(0)).unwrap();
	let de_v = deserialize_from::<Vec<Option<u64>>, _>(&mut buff).unwrap();
	assert_eq!(v, de_v);
}
