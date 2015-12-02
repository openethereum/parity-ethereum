//! Rlp serialization module 
//! 
//! Allows encoding, decoding, and view onto rlp-slice 
//!
//!# What should you use when?
//!
//!### Use `encode` function when: 
//! * You want to encode something inline.
//! * You do not work on big set of data.
//! * You want to encode whole data structure at once.
//!
//!### Use `decode` function when:
//! * You want to decode something inline.
//! * You do not work on big set of data.
//! * You want to decode whole rlp at once.
//!
//!### Use `RlpStream` when:
//! * You want to encode something in portions.
//! * You encode a big set of data.
//!
//!### Use `Rlp` when:
//! * You are working on trusted data (not corrupted).
//! * You want to get view onto rlp-slice.
//! * You don't want to decode whole rlp at once.
//!
//!### Use `UntrustedRlp` when: 
//! * You are working on untrusted data (~corrupted).
//! * You need to handle data corruption errors.
//! * You are working on input data.
//! * You want to get view onto rlp-slice.
//! * You don't want to decode whole rlp at once.

use std::fmt;
use std::cell::Cell;
use std::collections::LinkedList;
use std::error::Error as StdError;
use bytes::{ToBytes, FromBytes, FromBytesError};
use vector::InsertSlice;

/// Data-oriented view onto rlp-slice.
/// 
/// This is immutable structere. No operations change it.
/// 
/// Should be used in places where, error handling is required,
/// eg. on input
#[derive(Debug)]
pub struct UntrustedRlp<'a> {
	bytes: &'a [u8],
	cache: Cell<OffsetCache>,
}

/// rlp offset
#[derive(Copy, Clone, Debug)]
struct OffsetCache {
	index: usize,
	offset: usize,
}

impl OffsetCache {
	fn new(index: usize, offset: usize) -> OffsetCache {
		OffsetCache {
			index: index,
			offset: offset,
		}
	}
}

/// stores basic information about item
struct ItemInfo {
	prefix_len: usize,
	value_len: usize,
}

impl ItemInfo {
	fn new(prefix_len: usize, value_len: usize) -> ItemInfo {
		ItemInfo {
			prefix_len: prefix_len,
			value_len: value_len,
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
	FromBytesError(FromBytesError),
	RlpIsTooShort,
	RlpExpectedToBeList,
	RlpExpectedToBeData,
	BadRlp,
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

/// Data-oriented view onto trusted rlp-slice.
/// 
/// Unlikely to `UntrustedRlp` doesn't bother you with error
/// handling. It assumes that you know what you are doing.
pub struct Rlp<'a> {
	rlp: UntrustedRlp<'a>
}

impl<'a> From<UntrustedRlp<'a>> for Rlp<'a> {
	fn from(rlp: UntrustedRlp<'a>) -> Rlp<'a> {
		Rlp { rlp: rlp }
	}
}

impl<'a> From<Rlp<'a>> for UntrustedRlp<'a> {
	fn from(unsafe_rlp: Rlp<'a>) -> UntrustedRlp<'a> {
		unsafe_rlp.rlp
	}
}

pub enum Prototype {
	Null,
	Data(usize),
	List(usize),
}

impl<'a, 'view> Rlp<'a> where 'a: 'view {
	/// Create a new instance of `Rlp`
	pub fn new(bytes: &'a [u8]) -> Rlp<'a> {
		Rlp {
			rlp: UntrustedRlp::new(bytes)
		}
	}

	/// Get the prototype of the RLP.
	pub fn prototype(&self) -> Prototype {
		if self.is_data() {
			Prototype::Data(self.size())
		}
		else if self.is_list() {
			Prototype::List(self.item_count())
		}
		else {
			Prototype::Null
		}
	}

	/// The raw data of the RLP.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let dog = rlp.at(1).raw();
	/// 	assert_eq!(dog, &[0x83, b'd', b'o', b'g']);
	/// }
	/// ```
	pub fn raw(&'view self) -> &'a [u8] {
		self.rlp.raw()
	}

	pub fn data(&'view self) -> &'a [u8] {
		self.rlp.data()
	}

	/// Returns number of RLP items.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert_eq!(rlp.item_count(), 2);
	/// 	let view = rlp.at(1);
	/// 	assert_eq!(view.item_count(), 0);
	/// }
	/// ```
	pub fn item_count(&self) -> usize {
		self.rlp.item_count()
	}

	/// Returns the number of bytes in the data, or zero if it isn't data.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert_eq!(rlp.size(), 0);
	/// 	let view = rlp.at(1);
	/// 	assert_eq!(view.size(), 3);
	/// }
	/// ```
	pub fn size(&self) -> usize {
		self.rlp.size()
	}

	/// Get view onto RLP-slice at index.
	/// 
	/// Caches offset to given index, so access to successive
	/// slices is faster.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let dog = String::decode(&rlp.at(1));
	/// 	assert_eq!(dog, "dog".to_string());
	/// }
	/// ```
	pub fn at(&'view self, index: usize) -> Rlp<'a> {
		From::from(self.rlp.at(index).unwrap())
	}

	/// No value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.is_null());
	/// }
	/// ```
	pub fn is_null(&self) -> bool {
		self.rlp.is_null()
	}

	/// Contains a zero-length string or zero-length list.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc0];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.is_empty());
	/// }
	/// ```
	pub fn is_empty(&self) -> bool {
		self.rlp.is_empty()
	}

	/// List value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.is_list());
	/// }
	/// ```
	pub fn is_list(&self) -> bool {
		self.rlp.is_list()
	}

	/// String value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert!(rlp.at(1).is_data());
	/// }
	/// ```
	pub fn is_data(&self) -> bool {
		self.rlp.is_data()
	}

	/// Int value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc1, 0x10];
	/// 	let rlp = Rlp::new(&data);
	/// 	assert_eq!(rlp.is_int(), false);
	/// 	assert_eq!(rlp.at(0).is_int(), true);
	/// }
	/// ```
	pub fn is_int(&self) -> bool {
		self.rlp.is_int()
	}

	/// Get iterator over rlp-slices
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let strings: Vec<String> = rlp.iter().map(| i | String::decode(&i)).collect();
	/// }
	/// ```
	pub fn iter(&'a self) -> RlpIterator<'a> {
		self.into_iter()
	}
}

impl<'a, 'view> UntrustedRlp<'a> where 'a: 'view {
	/// returns new instance of `UntrustedRlp`
	pub fn new(bytes: &'a [u8]) -> UntrustedRlp<'a> {
		UntrustedRlp {
			bytes: bytes,
			cache: Cell::new(OffsetCache::new(usize::max_value(), 0)),
		}
	}

	/// The bare data of the RLP.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	let dog = rlp.at(1).unwrap().raw();
	/// 	assert_eq!(dog, &[0x83, b'd', b'o', b'g']);
	/// }
	/// ```
	pub fn raw(&'view self) -> &'a [u8] {
		self.bytes
	}

	pub fn data(&'view self) -> &'a [u8] {
		let ii = Self::item_info(self.bytes).unwrap();
		&self.bytes[ii.prefix_len..(ii.prefix_len + ii.value_len)]
	}

	/// Returns number of rlp items.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	assert_eq!(rlp.item_count(), 2);
	/// 	let view = rlp.at(1).unwrap();
	/// 	assert_eq!(view.item_count(), 0);
	/// }
	/// ```
	pub fn item_count(&self) -> usize {
		match self.is_list() {
			true => self.iter().count(),
			false => 0
		}
	}

	/// Returns the number of bytes in the data, or zero if it isn't data.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	assert_eq!(rlp.size(), 0);
	/// 	let view = rlp.at(1).unwrap();
	/// 	assert_eq!(view.size(), 3);
	/// }
	/// ```
	pub fn size(&self) -> usize {
		match self.is_data() {
			true => Self::item_info(self.bytes).unwrap().value_len,
			false => 0
		}
	}

	/// Get view onto rlp-slice at index
	/// 
	/// Caches offset to given index, so access to successive
	/// slices is faster.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	let dog = String::decode_untrusted(&rlp.at(1).unwrap()).unwrap();
	/// 	assert_eq!(dog, "dog".to_string());
	/// }
	/// ```
	pub fn at(&'view self, index: usize) -> Result<UntrustedRlp<'a>, DecoderError> {
		if !self.is_list() {
			return Err(DecoderError::RlpExpectedToBeList);
		}

		// move to cached position if it's index is less or equal to
		// current search index, otherwise move to beginning of list
		let c = self.cache.get();
		let (mut bytes, to_skip) = match c.index <= index {
			true => (try!(UntrustedRlp::consume(self.bytes, c.offset)), index - c.index),
			false => (try!(self.consume_list_prefix()), index),
		};

		// skip up to x items
		bytes = try!(UntrustedRlp::consume_items(bytes, to_skip));

		// update the cache
		self.cache.set(OffsetCache::new(index, self.bytes.len() - bytes.len()));

		// construct new rlp
		let found = try!(UntrustedRlp::item_info(bytes));
		Ok(UntrustedRlp::new(&bytes[0..found.prefix_len + found.value_len]))
	}

	/// No value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	assert!(rlp.is_null());
	/// }
	/// ```
	pub fn is_null(&self) -> bool {
		self.bytes.len() == 0
	}

	/// Contains a zero-length string or zero-length list.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc0];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	assert!(rlp.is_empty());
	/// }
	/// ```
	pub fn is_empty(&self) -> bool {
		!self.is_null() && (self.bytes[0] == 0xc0 || self.bytes[0] == 0x80)
	}

	/// List value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	assert!(rlp.is_list());
	/// }
	/// ```
	pub fn is_list(&self) -> bool {
		!self.is_null() && self.bytes[0] >= 0xc0
	}

	/// String value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	assert!(rlp.at(1).unwrap().is_data());
	/// }
	/// ```
	pub fn is_data(&self) -> bool {
		!self.is_null() && self.bytes[0] < 0xc0
	}

	/// Int value
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc1, 0x10];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	assert_eq!(rlp.is_int(), false);
	/// 	assert_eq!(rlp.at(0).unwrap().is_int(), true);
	/// }
	/// ```
	pub fn is_int(&self) -> bool {
		if self.is_null() {
			return false;
		}

		match self.bytes[0] {
			0...0x80 => true,
			0x81...0xb7 => self.bytes[1] != 0,
			b @ 0xb8...0xbf => self.bytes[1 + b as usize - 0xb7] != 0,
			_ => false
		}
	}

	/// Get iterator over rlp-slices
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = UntrustedRlp::new(&data);
	/// 	let strings: Vec<String> = rlp.iter()
	/// 		.map(| i | String::decode_untrusted(&i))
	/// 		.map(| s | s.unwrap())
	/// 		.collect();
	/// }
	/// ```
	pub fn iter(&'a self) -> UntrustedRlpIterator<'a> {
		self.into_iter()
	}

	/// consumes first found prefix
	fn consume_list_prefix(&self) -> Result<&'a [u8], DecoderError> {
		let item = try!(UntrustedRlp::item_info(self.bytes));
		let bytes = try!(UntrustedRlp::consume(self.bytes, item.prefix_len));
		Ok(bytes)
	}

	/// consumes fixed number of items
	fn consume_items(bytes: &'a [u8], items: usize) -> Result<&'a [u8], DecoderError> {
		let mut result = bytes;
		for _ in 0..items {
			let i = try!(UntrustedRlp::item_info(result));
			result = try!(UntrustedRlp::consume(result, (i.prefix_len + i.value_len)));
		}
		Ok(result)
	}

	/// return first item info
	///
	/// TODO: move this to decoder (?)
	fn item_info(bytes: &[u8]) -> Result<ItemInfo, DecoderError> {
		let item = match bytes.first().map(|&x| x) {
			None => return Err(DecoderError::RlpIsTooShort),
			Some(0...0x7f) => ItemInfo::new(0, 1),
			Some(l @ 0x80...0xb7) => ItemInfo::new(1, l as usize - 0x80),
			Some(l @ 0xb8...0xbf) => {
				let len_of_len = l as usize - 0xb7;
				let prefix_len = 1 + len_of_len;
				let value_len = try!(usize::from_bytes(&bytes[1..prefix_len]));
				ItemInfo::new(prefix_len, value_len)
			}
			Some(l @ 0xc0...0xf7) => ItemInfo::new(1, l as usize - 0xc0),
			Some(l @ 0xf8...0xff) => {
				let len_of_len = l as usize - 0xf7;
				let prefix_len = 1 + len_of_len;
				let value_len = try!(usize::from_bytes(&bytes[1..prefix_len]));
				ItemInfo::new(prefix_len, value_len)
			}
			_ => return Err(DecoderError::BadRlp),
		};

		match item.prefix_len + item.value_len <= bytes.len() {
			true => Ok(item),
			false => Err(DecoderError::RlpIsTooShort),
		}
	}

	/// consumes slice prefix of length `len`
	fn consume(bytes: &'a [u8], len: usize) -> Result<&'a [u8], DecoderError> {
		match bytes.len() >= len {
			true => Ok(&bytes[len..]),
			false => Err(DecoderError::RlpIsTooShort),
		}
	}
}

/// Iterator over rlp-slice list elements.
pub struct UntrustedRlpIterator<'a> {
	rlp: &'a UntrustedRlp<'a>,
	index: usize,
}

impl<'a> IntoIterator for &'a UntrustedRlp<'a> {
	type Item = UntrustedRlp<'a>;
	type IntoIter = UntrustedRlpIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		UntrustedRlpIterator {
			rlp: self,
			index: 0,
		}
	}
}

impl<'a> Iterator for UntrustedRlpIterator<'a> {
	type Item = UntrustedRlp<'a>;

	fn next(&mut self) -> Option<UntrustedRlp<'a>> {
		let index = self.index;
		let result = self.rlp.at(index).ok();
		self.index += 1;
		result
	}
}

/// Iterator over trusted rlp-slice list elements.
pub struct RlpIterator<'a> {
	rlp: &'a Rlp<'a>,
	index: usize
}

impl<'a> IntoIterator for &'a Rlp<'a> {
	type Item = Rlp<'a>;
	type IntoIter = RlpIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		RlpIterator {
			rlp: self,
			index: 0,
		}
	}
}

impl<'a> Iterator for RlpIterator<'a> {
	type Item = Rlp<'a>;

	fn next(&mut self) -> Option<Rlp<'a>> {
		let index = self.index;
		let result = self.rlp.rlp.at(index).ok().map(| iter | { From::from(iter) });
		self.index += 1;
		result
	}
}

/// Shortcut function to decode trusted rlp
/// 
/// ```rust
/// extern crate ethcore_util as util;
/// use util::rlp::*;
/// 
/// fn main () {
/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
/// 	let animals: Vec<String> = decode(&data);
/// 	assert_eq!(animals, vec!["cat".to_string(), "dog".to_string()]);
/// }
/// ```
pub fn decode<T>(bytes: &[u8]) -> T where T: Decodable {
	let rlp = Rlp::new(bytes);
	T::decode(&rlp)
}

pub trait Decodable: Sized {
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError>;
	fn decode(rlp: &Rlp) -> Self {
		Self::decode_untrusted(&rlp.rlp).unwrap()
	}
}

impl<T> Decodable for T where T: FromBytes {
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		match rlp.is_data() {
			true => BasicDecoder::read_value(rlp.bytes, | bytes | {
				Ok(try!(T::from_bytes(bytes)))
			}),
			false => Err(DecoderError::RlpExpectedToBeData),
		}
	}
}

impl<T> Decodable for Vec<T> where T: Decodable {
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		match rlp.is_list() {
			true => rlp.iter().map(|rlp| T::decode_untrusted(&rlp)).collect(),
			false => Err(DecoderError::RlpExpectedToBeList),
		}
	}
}

impl Decodable for Vec<u8> {
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		match rlp.is_data() {
			true =>	BasicDecoder::read_value(rlp.bytes, | bytes | {
				let mut res = vec![];
				res.extend(bytes);
				Ok(res)
			}),
			false => Err(DecoderError::RlpExpectedToBeData),
		}
	}
}

pub trait Decoder {
	fn read_value<T, F>(bytes: &[u8], f: F) -> Result<T, DecoderError> where F: FnOnce(&[u8]) -> Result<T, DecoderError>;
}

pub struct BasicDecoder;

impl Decoder for BasicDecoder {
	fn read_value<T, F>(bytes: &[u8], f: F) -> Result<T, DecoderError> where F: FnOnce(&[u8]) -> Result<T, DecoderError> {
		match bytes.first().map(|&x| x) {
			// rlp is too short
			None => Err(DecoderError::RlpIsTooShort),
			// single byt value
			Some(l @ 0...0x7f) => Ok(try!(f(&[l]))),
			// 0-55 bytes
			Some(l @ 0x80...0xb7) => Ok(try!(f(&bytes[1..(1 + l as usize - 0x80)]))),
			// longer than 55 bytes
			Some(l @ 0xb8...0xbf) => {
				let len_of_len = l as usize - 0xb7;
				let begin_of_value = 1 as usize + len_of_len;
				let len = try!(usize::from_bytes(&bytes[1..begin_of_value]));
				Ok(try!(f(&bytes[begin_of_value..begin_of_value + len])))
			}
			_ => Err(DecoderError::BadRlp),
		}
	}
}

#[derive(Debug)]
struct ListInfo {
	position: usize,
	current: usize,
	max: usize,
}

impl ListInfo {
	fn new(position: usize, max: usize) -> ListInfo {
		ListInfo {
			position: position,
			current: 0,
			max: max,
		}
	}
}

/// Appendable rlp encoder.
pub struct RlpStream {
	unfinished_lists: LinkedList<ListInfo>,
	encoder: BasicEncoder,
}

impl RlpStream {
	/// Initializes instance of empty `RlpStream`.
	pub fn new() -> RlpStream {
		RlpStream {
			unfinished_lists: LinkedList::new(),
			encoder: BasicEncoder::new(),
		}
	}

	/// Initializes the `RLPStream` as a list.
	pub fn new_list(len: usize) -> RlpStream {
		let mut stream = RlpStream::new();
		stream.append_list(len);
		stream
	}

	/// Apends value to the end of stream, chainable.
	///
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.append(&"cat").append(&"dog");
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
	/// }
	/// ```
	pub fn append<'a, E>(&'a mut self, object: &E) -> &'a mut RlpStream where E: Encodable + fmt::Debug {
		// encode given value and add it at the end of the stream
		object.encode(&mut self.encoder);

		// if list is finished, prepend the length
		self.note_appended(1);

		// return chainable self
		self
	}

	/// Declare appending the list of given size, chainable.
	///
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.append_list(2).append(&"cat").append(&"dog");
	/// 	stream.append(&"");	
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xca, 0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g', 0x80]);
	/// }
	/// ```
	pub fn append_list<'a>(&'a mut self, len: usize) -> &'a mut RlpStream {
		// push new list
		let position = self.encoder.bytes.len();
		match len {
			0 => {
				// we may finish, if the appended list len is equal 0
				self.encoder.bytes.push(0xc0u8);
				self.note_appended(1);
			}
			_ => self.unfinished_lists.push_back(ListInfo::new(position, len)),
		}

		// return chainable self
		self
	}

	/// Apends null to the end of stream, chainable.
	///
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.append_empty_data().append_empty_data();
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xc2, 0x80, 0x80]);
	/// }
	/// ```
	pub fn append_empty_data<'a>(&'a mut self) -> &'a mut RlpStream {
		// self push raw item
		self.encoder.bytes.push(0x80);

		// try to finish and prepend the length
		self.note_appended(1);

		// return chainable self
		self
	}

	/// Appends raw (pre-serialised) RLP data. Use with caution. Chainable.
	pub fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut RlpStream {
		// push raw items
		self.encoder.bytes.extend(bytes);	

		// try to finish and prepend the length
		self.note_appended(item_count);

		// return chainable self
		self
	}

	/// Clear the output stream so far.
	/// 
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(3);
	/// 	stream.append(&"cat");
	/// 	stream.clear();
	/// 	stream.append(&"dog");
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0x83, b'd', b'o', b'g']);
	/// }
	pub fn clear(&mut self) {
		// clear bytes
		self.encoder.bytes.clear();

		// clear lists
		self.unfinished_lists.clear();
	}

	/// Returns true if stream doesnt expect any more items.
	///
	/// ```rust
	/// extern crate ethcore_util as util;
	/// use util::rlp::*;
	/// 
	/// fn main () {
	/// 	let mut stream = RlpStream::new_list(2);
	/// 	stream.append(&"cat");
	/// 	assert_eq!(stream.is_finished(), false);
	/// 	stream.append(&"dog");
	/// 	assert_eq!(stream.is_finished(), true);
	/// 	let out = stream.out();
	/// 	assert_eq!(out, vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
	/// }
	pub fn is_finished(&self) -> bool {
		self.unfinished_lists.back().is_none()
	}

	/// Streams out encoded bytes.
	/// 
	/// panic! if stream is not finished.
	pub fn out(self) -> Vec<u8> {
		match self.is_finished() {
			true => self.encoder.out(),
			false => panic!()
		}
	}

	/// Try to finish lists
	fn note_appended(&mut self, inserted_items: usize) -> () {
		let should_finish = match self.unfinished_lists.back_mut() {
			None => false,
			Some(ref mut x) => {
				x.current += inserted_items;
				if x.current > x.max {
					panic!("You cannot append more items then you expect!");
				}
				x.current == x.max
			}
		};

		if should_finish {
			let x = self.unfinished_lists.pop_back().unwrap();
			let len = self.encoder.bytes.len() - x.position;
			self.encoder.insert_list_len_at_pos(len, x.position);
			self.note_appended(1);
		}
	}
}

/// Shortcut function to encode structure into rlp.
///
/// ```rust
/// extern crate ethcore_util as util;
/// use util::rlp::*;
/// 
/// fn main () {
/// 	let animals = vec!["cat", "dog"];
/// 	let out = encode(&animals);
/// 	assert_eq!(out, vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
/// }
/// ```
pub fn encode<E>(object: &E) -> Vec<u8> where E: Encodable
{
	let mut encoder = BasicEncoder::new();
	object.encode(&mut encoder);
	encoder.out()
}

pub trait Encodable {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder;
}

pub trait Encoder {
	fn emit_value(&mut self, bytes: &[u8]) -> ();
	fn emit_list<F>(&mut self, f: F) -> () where F: FnOnce(&mut Self) -> ();
}

impl<T> Encodable for T where T: ToBytes {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder {
		encoder.emit_value(&self.to_bytes())
	}
}

impl<'a, T> Encodable for &'a [T] where T: Encodable + 'a {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder {
		encoder.emit_list(|e| {
			// insert all list elements
			for el in self.iter() {
				el.encode(e);
			}
		})
	}
}

impl<T> Encodable for Vec<T> where T: Encodable {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder {
		let r: &[T] = self.as_ref();
		r.encode(encoder)
	}
}

/// lets treat bytes differently than other lists
/// they are a single value
impl<'a> Encodable for &'a [u8] {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder {
		encoder.emit_value(self)
	}
}

/// lets treat bytes differently than other lists
/// they are a single value
impl Encodable for Vec<u8> {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder {
		encoder.emit_value(self)
	}
}

struct BasicEncoder {
	bytes: Vec<u8>,
}

impl BasicEncoder {
	fn new() -> BasicEncoder {
		BasicEncoder { bytes: vec![] }
	}

	/// inserts list prefix at given position
	/// TODO: optimise it further?
	fn insert_list_len_at_pos(&mut self, len: usize, pos: usize) -> () {
		let mut res = vec![];
		match len {
			0...55 => res.push(0xc0u8 + len as u8),
			_ => {
				res.push(0xf7u8 + len.to_bytes_len() as u8);
				res.extend(len.to_bytes());
			}
		};

		self.bytes.insert_slice(pos, &res);
	}

	/// get encoded value
	fn out(self) -> Vec<u8> {
		self.bytes
	}
}

impl Encoder for BasicEncoder {
	fn emit_value(&mut self, bytes: &[u8]) -> () {
		match bytes.len() {
			// just 0
			0 => self.bytes.push(0x80u8),
			// byte is its own encoding
			1 if bytes[0] < 0x80 => self.bytes.extend(bytes),
			// (prefix + length), followed by the string
			len @ 1 ... 55 => {
				self.bytes.push(0x80u8 + len as u8);
				self.bytes.extend(bytes);
			}
			// (prefix + length of length), followed by the length, followd by the string
			len => {
				self.bytes.push(0xb7 + len.to_bytes_len() as u8);
				self.bytes.extend(len.to_bytes());
				self.bytes.extend(bytes);
			}
		}
	}

	fn emit_list<F>(&mut self, f: F) -> () where F: FnOnce(&mut Self) -> ()
	{
		// get len before inserting a list
		let before_len = self.bytes.len();

		// insert all list elements
		f(self);

		// get len after inserting a list
		let after_len = self.bytes.len();

		// diff is list len
		let list_len = after_len - before_len;
		self.insert_list_len_at_pos(list_len, before_len);
	}
}

#[cfg(test)]
mod tests {
	use std::{fmt, cmp};
	use std::str::FromStr;
	use rlp;
	use rlp::{UntrustedRlp, RlpStream, Decodable};
	use uint::U256;

	#[test]
	fn rlp_at() {
		let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
		{
			let rlp = UntrustedRlp::new(&data);
			assert!(rlp.is_list());
			let animals = <Vec<String> as rlp::Decodable>::decode_untrusted(&rlp).unwrap();
			assert_eq!(animals, vec!["cat".to_string(), "dog".to_string()]);

			let cat = rlp.at(0).unwrap();
			assert!(cat.is_data());
			assert_eq!(cat.bytes, &[0x83, b'c', b'a', b't']);
			assert_eq!(String::decode_untrusted(&cat).unwrap(), "cat".to_string());

			let dog = rlp.at(1).unwrap();
			assert!(dog.is_data());
			assert_eq!(dog.bytes, &[0x83, b'd', b'o', b'g']);
			assert_eq!(String::decode_untrusted(&dog).unwrap(), "dog".to_string());

			let cat_again = rlp.at(0).unwrap();
			assert!(cat_again.is_data());
			assert_eq!(cat_again.bytes, &[0x83, b'c', b'a', b't']);
			assert_eq!(String::decode_untrusted(&cat_again).unwrap(), "cat".to_string());
		}
	}

	#[test]
	fn rlp_at_err() {
		let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o'];
		{
			let rlp = UntrustedRlp::new(&data);
			assert!(rlp.is_list());

			let cat_err = rlp.at(0).unwrap_err();
			assert_eq!(cat_err, rlp::DecoderError::RlpIsTooShort);

			let dog_err = rlp.at(1).unwrap_err();
			assert_eq!(dog_err, rlp::DecoderError::RlpIsTooShort);
		}
	}

	#[test]
	fn rlp_iter() {
		let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
		{
			let rlp = UntrustedRlp::new(&data);
			let mut iter = rlp.iter();

			let cat = iter.next().unwrap();
			assert!(cat.is_data());
			assert_eq!(cat.bytes, &[0x83, b'c', b'a', b't']);

			let dog = iter.next().unwrap();
			assert!(dog.is_data());
			assert_eq!(dog.bytes, &[0x83, b'd', b'o', b'g']);

			let none = iter.next();
			assert!(none.is_none());

			let cat_again = rlp.at(0).unwrap();
			assert!(cat_again.is_data());
			assert_eq!(cat_again.bytes, &[0x83, b'c', b'a', b't']);
		}
	}

	struct ETestPair<T>(T, Vec<u8>) where T: rlp::Encodable;

	fn run_encode_tests<T>(tests: Vec<ETestPair<T>>)
		where T: rlp::Encodable
	{
		for t in &tests {
			let res = rlp::encode(&t.0);
			assert_eq!(res, &t.1[..]);
		}
	}
	
	#[test]
	fn encode_u16() {
		let tests = vec![
			ETestPair(0u16, vec![0x80u8]),
			ETestPair(0x100, vec![0x82, 0x01, 0x00]),
			ETestPair(0xffff, vec![0x82, 0xff, 0xff]),
		];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_u32() {
		let tests = vec![
			ETestPair(0u32, vec![0x80u8]),
			ETestPair(0x10000, vec![0x83, 0x01, 0x00, 0x00]),
			ETestPair(0xffffff, vec![0x83, 0xff, 0xff, 0xff]),
		];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_u64() {
		let tests = vec![
			ETestPair(0u64, vec![0x80u8]),
			ETestPair(0x1000000, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
			ETestPair(0xFFFFFFFF, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
		];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_u256() {
		let tests = vec![ETestPair(U256::from(0u64), vec![0x80u8]),
						 ETestPair(U256::from(0x1000000u64), vec![0x84, 0x01, 0x00, 0x00, 0x00]),
						 ETestPair(U256::from(0xffffffffu64),
								   vec![0x84, 0xff, 0xff, 0xff, 0xff]),
						 ETestPair(U256::from_str("8090a0b0c0d0e0f00910203040506077000000000000\
												   000100000000000012f0")
									   .unwrap(),
								   vec![0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
										0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77, 0x00,
										0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
										0x00, 0x00, 0x00, 0x00, 0x12, 0xf0])];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_str() {
		let tests = vec![ETestPair("cat", vec![0x83, b'c', b'a', b't']),
						 ETestPair("dog", vec![0x83, b'd', b'o', b'g']),
						 ETestPair("Marek", vec![0x85, b'M', b'a', b'r', b'e', b'k']),
						 ETestPair("", vec![0x80]),
						 ETestPair("Lorem ipsum dolor sit amet, consectetur adipisicing elit",
								   vec![0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i',
										b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l', b'o',
										b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e',
										b't', b',', b' ', b'c', b'o', b'n', b's', b'e', b'c',
										b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i',
										b'p', b'i', b's', b'i', b'c', b'i', b'n', b'g', b' ',
										b'e', b'l', b'i', b't'])];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_address() {
		use hash::*;

		let tests = vec![
			ETestPair(Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap(), 
					  vec![0x94, 0xef, 0x2d, 0x6d, 0x19, 0x40, 0x84, 0xc2, 0xde,
								 0x36, 0xe0, 0xda, 0xbf, 0xce, 0x45, 0xd0, 0x46,
								 0xb3, 0x7d, 0x11, 0x06])
		];
		run_encode_tests(tests);
	}

	/// Vec<u8> is treated as a single value
	#[test]
	fn encode_vector_u8() {
		let tests = vec![
			ETestPair(vec![], vec![0x80]),
			ETestPair(vec![0u8], vec![0]),
			ETestPair(vec![0x15], vec![0x15]),
			ETestPair(vec![0x40, 0x00], vec![0x82, 0x40, 0x00]),
		];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_vector_u64() {
		let tests = vec![
			ETestPair(vec![], vec![0xc0]),
			ETestPair(vec![15u64], vec![0xc1, 0x0f]),
			ETestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
			ETestPair(vec![0xffffffff, 1, 2, 3, 7, 0xff], vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff,  1, 2, 3, 7, 0x81, 0xff]),
		];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_vector_str() {
		let tests = vec![ETestPair(vec!["cat", "dog"],
								   vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_vector_of_vectors_str() {
		let tests = vec![ETestPair(vec![vec!["cat"]], vec![0xc5, 0xc4, 0x83, b'c', b'a', b't'])];
		run_encode_tests(tests);
	}

	#[test]
	fn encode_bytes() {
		let vec = vec![0u8];
		let slice: &[u8] = &vec;
		let res = rlp::encode(&slice);
		assert_eq!(res, vec![0u8]);
	}

	#[test]
	fn rlp_stream() {
		let mut stream = RlpStream::new_list(2);
		stream.append(&"cat").append(&"dog");
		let out = stream.out();
		assert_eq!(out,
				   vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
	}

	#[test]
	fn rlp_stream_list() {
		let mut stream = RlpStream::new_list(3);
		stream.append_list(0);
		stream.append_list(1).append_list(0);
		stream.append_list(2).append_list(0).append_list(1).append_list(0);
		let out = stream.out();
		assert_eq!(out, vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]);
	}

	#[test]
	fn rlp_stream_list2() {
		let mut stream = RlpStream::new();
		stream.append_list(17);
		for _ in 0..17 {
			stream.append(&"");
		}
		let out = stream.out();
		assert_eq!(out, vec![0xd1, 0x80, 0x80, 0x80, 0x80, 0x80,
							 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
							 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]);
	}

	#[test]
	fn rlp_stream_list3() {
		let mut stream = RlpStream::new();
		stream.append_list(17);

		let mut res = vec![0xf8, 0x44];
		for _ in 0..17 {
			stream.append(&"aaa");
			res.extend(vec![0x83, b'a', b'a', b'a']);
		}
		let out = stream.out();
		assert_eq!(out, res);
	}

	struct DTestPair<T>(T, Vec<u8>) where T: rlp::Decodable + fmt::Debug + cmp::Eq;

	fn run_decode_tests<T>(tests: Vec<DTestPair<T>>) where T: rlp::Decodable + fmt::Debug + cmp::Eq {
		for t in &tests {
			let res: T = rlp::decode(&t.1);
			assert_eq!(res, t.0);
		}
	}

	/// Vec<u8> is treated as a single value
	#[test]
	fn decode_vector_u8() {
		let tests = vec![
			DTestPair(vec![], vec![0x80]),
			DTestPair(vec![0u8], vec![0]),
			DTestPair(vec![0x15], vec![0x15]),
			DTestPair(vec![0x40, 0x00], vec![0x82, 0x40, 0x00]),
		];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_u16() {
		let tests = vec![
			DTestPair(0u16, vec![0u8]),
			DTestPair(0x100, vec![0x82, 0x01, 0x00]),
			DTestPair(0xffff, vec![0x82, 0xff, 0xff]),
		];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_u32() {
		let tests = vec![
			DTestPair(0u32, vec![0u8]),
			DTestPair(0x10000, vec![0x83, 0x01, 0x00, 0x00]),
			DTestPair(0xffffff, vec![0x83, 0xff, 0xff, 0xff]),
		];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_u64() {
		let tests = vec![
			DTestPair(0u64, vec![0u8]),
			DTestPair(0x1000000, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
			DTestPair(0xFFFFFFFF, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
		];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_u256() {
		let tests = vec![DTestPair(U256::from(0u64), vec![0x80u8]),
						 DTestPair(U256::from(0x1000000u64), vec![0x84, 0x01, 0x00, 0x00, 0x00]),
						 DTestPair(U256::from(0xffffffffu64),
								   vec![0x84, 0xff, 0xff, 0xff, 0xff]),
						 DTestPair(U256::from_str("8090a0b0c0d0e0f00910203040506077000000000000\
												   000100000000000012f0")
									   .unwrap(),
								   vec![0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
										0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77, 0x00,
										0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
										0x00, 0x00, 0x00, 0x00, 0x12, 0xf0])];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_str() {
		let tests = vec![DTestPair("cat".to_string(), vec![0x83, b'c', b'a', b't']),
						 DTestPair("dog".to_string(), vec![0x83, b'd', b'o', b'g']),
						 DTestPair("Marek".to_string(),
								   vec![0x85, b'M', b'a', b'r', b'e', b'k']),
						 DTestPair("".to_string(), vec![0x80]),
						 DTestPair("Lorem ipsum dolor sit amet, consectetur adipisicing elit"
									   .to_string(),
								   vec![0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i',
										b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l', b'o',
										b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e',
										b't', b',', b' ', b'c', b'o', b'n', b's', b'e', b'c',
										b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i',
										b'p', b'i', b's', b'i', b'c', b'i', b'n', b'g', b' ',
										b'e', b'l', b'i', b't'])];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_address() {
		use hash::*;

		let tests = vec![
			DTestPair(Address::from_str("ef2d6d194084c2de36e0dabfce45d046b37d1106").unwrap(), 
					  vec![0x94, 0xef, 0x2d, 0x6d, 0x19, 0x40, 0x84, 0xc2, 0xde,
								 0x36, 0xe0, 0xda, 0xbf, 0xce, 0x45, 0xd0, 0x46,
								 0xb3, 0x7d, 0x11, 0x06])
		];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_vector_u64() {
		let tests = vec![
			DTestPair(vec![], vec![0xc0]),
			DTestPair(vec![15u64], vec![0xc1, 0x0f]),
			DTestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
			DTestPair(vec![0xffffffff, 1, 2, 3, 7, 0xff], vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff,  1, 2, 3, 7, 0x81, 0xff]),
		];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_vector_str() {
		let tests = vec![DTestPair(vec!["cat".to_string(), "dog".to_string()],
								   vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])];
		run_decode_tests(tests);
	}

	#[test]
	fn decode_untrusted_vector_of_vectors_str() {
		let tests = vec![DTestPair(vec![vec!["cat".to_string()]],
								   vec![0xc5, 0xc4, 0x83, b'c', b'a', b't'])];
		run_decode_tests(tests);
	}

	#[test]
	fn test_view() {
		struct View<'a> {
			bytes: &'a [u8]
		}

		impl <'a, 'view> View<'a> where 'a: 'view {
			fn new(bytes: &'a [u8]) -> View<'a> {
				View {
					bytes: bytes
				}
			}

			fn offset(&'view self, len: usize) -> View<'a> {
				View::new(&self.bytes[len..])
			}

			fn data(&'view self) -> &'a [u8] {
				self.bytes
			}
		}

		let data = vec![0, 1, 2, 3];
		let view = View::new(&data);
		let _data_slice = view.offset(1).data();
	}
}
