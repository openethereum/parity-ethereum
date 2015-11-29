//! UntrustedRlp serialization module
//!
//! Types implementing `Endocable` and `Decodable` traits
//! can be easily coverted to and from rlp
//!
//! # Examples:
//!
//! ```rust
//! extern crate ethcore_util;
//! use ethcore_util::rlp::{UntrustedRlp, RlpStream, Decodable};
//!
//! fn encode_value() {
//!		// 1029
//!		let mut stream = RlpStream::new();
//!		stream.append(&1029u32);
//!		let out = stream.out().unwrap();
//!		assert_eq!(out, vec![0x82, 0x04, 0x05]);
//! }
//!
//! fn encode_list() {
//!		// [ "cat", "dog" ]
//!		let mut stream = RlpStream::new_list(2);
//!		stream.append(&"cat").append(&"dog");
//!		let out = stream.out().unwrap();
//!		assert_eq!(out, vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
//! }
//!
//! fn encode_list2() {
//!		// [ [], [[]], [ [], [[]] ] ]
//!		let mut stream = RlpStream::new_list(3);
//!		stream.append_list(0);
//!		stream.append_list(1).append_list(0);
//!		stream.append_list(2).append_list(0).append_list(1).append_list(0);
//!		let out = stream.out().unwrap();
//!		assert_eq!(out, vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]);
//! }
//!
//! fn decode_untrusted_value() {
//!		//  0x102456789abcdef
//!		let data = vec![0x88, 0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
//!		let rlp = UntrustedRlp::new(&data);
//!		let _ = u64::decode_untrusted(&rlp).unwrap();
//! }
//! 
//! fn decode_untrusted_string() {
//!		// "cat"
//!		let data = vec![0x83, b'c', b'a', b't'];
//!		let rlp = UntrustedRlp::new(&data);
//!		let _ = String::decode_untrusted(&rlp).unwrap();
//! }
//! 
//! fn decode_untrusted_list() {
//!     // ["cat", "dog"]
//!     let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
//!     let rlp = UntrustedRlp::new(&data);
//!     let _ : Vec<String> = Decodable::decode_untrusted(&rlp).unwrap();
//! }
//! 
//! fn decode_untrusted_list2() {
//!		// [ [], [[]], [ [], [[]] ] ]
//!		let data = vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0];
//!		let rlp = UntrustedRlp::new(&data);
//!		let _v0: Vec<u8> = Decodable::decode_untrusted(&rlp.at(0).unwrap()).unwrap();
//!		let _v1: Vec<Vec<u8>> = Decodable::decode_untrusted(&rlp.at(1).unwrap()).unwrap();
//!		let nested_rlp = rlp.at(2).unwrap();
//!		let _v2a: Vec<u8> = Decodable::decode_untrusted(&nested_rlp.at(0).unwrap()).unwrap();
//!		let _v2b: Vec<Vec<u8>> = Decodable::decode_untrusted(&nested_rlp.at(1).unwrap()).unwrap();
//! }
//!
//! fn main() {
//!		encode_value();
//!		encode_list();
//!		encode_list2();
//!
//!		decode_untrusted_value();
//!		decode_untrusted_string();
//!		decode_untrusted_list();
//!		decode_untrusted_list2();
//! }
//! ```
//!

use std::fmt;
use std::cell::Cell;
use std::collections::LinkedList;
use std::error::Error as StdError;
use bytes::{ToBytes, FromBytes, FromBytesError};
use vector::InsertSlice;

/// rlp container
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
	UntrustedRlpIsTooShort,
	UntrustedRlpExpectedToBeList,
	UntrustedRlpExpectedToBeValue,
	BadUntrustedRlp,
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

/// Unsafe wrapper for rlp decode_untrustedr.
/// 
/// It assumes that you know what you are doing. Doesn't bother
/// you with error handling.
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

impl<'a> Rlp<'a> {
	/// returns new instance of `Rlp`
	pub fn new(bytes: &'a [u8]) -> Rlp<'a> {
		Rlp {
			rlp: UntrustedRlp::new(bytes)
		}
	}

	pub fn at(&self, index: usize) -> Rlp<'a> {
		From::from(self.rlp.at(index).unwrap())
	}

	/// returns true if rlp is a list
	pub fn is_list(&self) -> bool {
		self.rlp.is_list()
	}

	/// returns true if rlp is a value
	pub fn is_value(&self) -> bool {
		self.rlp.is_value()
	}

	/// returns rlp iterator
	pub fn iter(&'a self) -> UntrustedRlpIterator<'a> {
		self.rlp.into_iter()
	}
}

impl<'a> UntrustedRlp<'a> {
	/// returns new instance of `UntrustedRlp`
	pub fn new(bytes: &'a [u8]) -> UntrustedRlp<'a> {
		UntrustedRlp {
			bytes: bytes,
			cache: Cell::new(OffsetCache::new(usize::max_value(), 0)),
		}
	}

	/// get container subset at given index
	///
	/// paren container caches searched position
	pub fn at(&self, index: usize) -> Result<UntrustedRlp<'a>, DecoderError> {
		if !self.is_list() {
			return Err(DecoderError::UntrustedRlpExpectedToBeList);
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

	/// returns true if rlp is a list
	pub fn is_list(&self) -> bool {
		self.bytes.len() > 0 && self.bytes[0] >= 0xc0
	}

	/// returns true if rlp is a value
	pub fn is_value(&self) -> bool {
		self.bytes.len() > 0 && self.bytes[0] <= 0xbf
	}

	/// returns rlp iterator
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
	/// TODO: move this to decode_untrustedr?
	fn item_info(bytes: &[u8]) -> Result<ItemInfo, DecoderError> {
		let item = match bytes.first().map(|&x| x) {
			None => return Err(DecoderError::UntrustedRlpIsTooShort),
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
			_ => return Err(DecoderError::BadUntrustedRlp),
		};

		match item.prefix_len + item.value_len <= bytes.len() {
			true => Ok(item),
			false => Err(DecoderError::UntrustedRlpIsTooShort),
		}
	}

	/// consumes slice prefix of length `len`
	fn consume(bytes: &'a [u8], len: usize) -> Result<&'a [u8], DecoderError> {
		match bytes.len() >= len {
			true => Ok(&bytes[len..]),
			false => Err(DecoderError::UntrustedRlpIsTooShort),
		}
	}
}

/// non-consuming rlp iterator
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

/// shortcut function to decoded Trusted Rlp `&[u8]` into an object
pub fn decode<T>(bytes: &[u8]) -> T where T: Decodable {
	let rlp = Rlp::new(bytes);
	T::decode(&rlp)
}

/// shortcut function to decode UntrustedRlp `&[u8]` into an object
pub fn decode_untrusted<T>(bytes: &[u8]) -> Result<T, DecoderError>
	where T: Decodable
{
	let rlp = UntrustedRlp::new(bytes);
	T::decode_untrusted(&rlp)
}

pub trait Decodable: Sized {
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError>;
	fn decode(rlp: &Rlp) -> Self {
		Self::decode_untrusted(&rlp.rlp).unwrap()
	}
}

impl<T> Decodable for T where T: FromBytes
{
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		match rlp.is_value() {
			true => BasicDecoder::read_value(rlp.bytes),
			false => Err(DecoderError::UntrustedRlpExpectedToBeValue),
		}
	}
}

impl<T> Decodable for Vec<T> where T: Decodable
{
	fn decode_untrusted(rlp: &UntrustedRlp) -> Result<Self, DecoderError> {
		match rlp.is_list() {
			true => rlp.iter().map(|rlp| T::decode_untrusted(&rlp)).collect(),
			false => Err(DecoderError::UntrustedRlpExpectedToBeList),
		}
	}
}

pub trait Decoder {
	fn read_value<T>(bytes: &[u8]) -> Result<T, DecoderError> where T: FromBytes;
}

struct BasicDecoder;

impl Decoder for BasicDecoder {
	fn read_value<T>(bytes: &[u8]) -> Result<T, DecoderError>
		where T: FromBytes
	{
		match bytes.first().map(|&x| x) {
			// rlp is too short
			None => Err(DecoderError::UntrustedRlpIsTooShort),
			// single byt value
			Some(l @ 0...0x7f) => Ok(try!(T::from_bytes(&[l]))),
			// 0-55 bytes
			Some(l @ 0x80...0xb7) => Ok(try!(T::from_bytes(&bytes[1..(1 + l as usize - 0x80)]))),
			// longer than 55 bytes
			Some(l @ 0xb8...0xbf) => {
				let len_of_len = l as usize - 0xb7;
				let begin_of_value = 1 as usize + len_of_len;
				let len = try!(usize::from_bytes(&bytes[1..begin_of_value]));
				Ok(try!(T::from_bytes(&bytes[begin_of_value..begin_of_value + len])))
			}
			_ => Err(DecoderError::BadUntrustedRlp),
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

/// container that should be used to encode rlp
pub struct RlpStream {
	unfinished_lists: LinkedList<ListInfo>,
	encoder: BasicEncoder,
}

impl RlpStream {
	/// create new container for values appended one after another,
	/// but not being part of the same list
	pub fn new() -> RlpStream {
		RlpStream {
			unfinished_lists: LinkedList::new(),
			encoder: BasicEncoder::new(),
		}
	}

	/// create new container for list of size `max_len`
	pub fn new_list(len: usize) -> RlpStream {
		let mut stream = RlpStream::new();
		stream.append_list(len);
		stream
	}

	/// apends value to the end of stream, chainable
	pub fn append<'a, E>(&'a mut self, object: &E) -> &'a mut RlpStream
		where E: Encodable
	{
		// encode given value and add it at the end of the stream
		object.encode(&mut self.encoder);

		// if list is finished, prepend the length
		self.try_to_finish();

		// return chainable self
		self
	}

	/// declare appending the list of given size
	pub fn append_list<'a>(&'a mut self, len: usize) -> &'a mut RlpStream {
		// push new list
		let position = self.encoder.bytes.len();
		match len {
			0 => {
				// we may finish, if the appended list len is equal 0
				self.encoder.bytes.push(0xc0u8);
				self.try_to_finish();
			}
			_ => self.unfinished_lists.push_back(ListInfo::new(position, len)),
		}

		// return chainable self
		self
	}

	/// return true if stream is ready
	pub fn is_finished(&self) -> bool {
		self.unfinished_lists.back().is_none()
	}

	/// streams out encoded bytes
	pub fn out(self) -> Result<Vec<u8>, EncoderError> {
		match self.is_finished() {
			true => Ok(self.encoder.out()),
			false => Err(EncoderError::StreamIsUnfinished),
		}
	}

	/// try to finish lists
	fn try_to_finish(&mut self) -> () {
		let should_finish = match self.unfinished_lists.back_mut() {
			None => false,
			Some(ref mut x) => {
				x.current += 1;
				x.current == x.max
			}
		};

		if should_finish {
			let x = self.unfinished_lists.pop_back().unwrap();
			let len = self.encoder.bytes.len() - x.position;
			self.encoder.insert_list_len_at_pos(len, x.position);
			self.try_to_finish();
		}
	}
}

/// shortcut function to encode a `T: Encodable` into a UntrustedRlp `Vec<u8>`
pub fn encode<E>(object: &E) -> Vec<u8>
	where E: Encodable
{
	let mut encoder = BasicEncoder::new();
	object.encode(&mut encoder);
	encoder.out()
}

#[derive(Debug)]
pub enum EncoderError {
	StreamIsUnfinished,
}

impl StdError for EncoderError {
	fn description(&self) -> &str {
		"encoder error"
	}
}

impl fmt::Display for EncoderError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

pub trait Encodable {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder;
}

pub trait Encoder {
	fn emit_value(&mut self, bytes: &[u8]) -> ();
	fn emit_list<F>(&mut self, f: F) -> () where F: FnOnce(&mut Self) -> ();
}

impl<T> Encodable for T where T: ToBytes
{
	fn encode<E>(&self, encoder: &mut E) -> ()
		where E: Encoder
	{
		encoder.emit_value(&self.to_bytes())
	}
}

impl<'a, T> Encodable for &'a [T] where T: Encodable + 'a
{
	fn encode<E>(&self, encoder: &mut E) -> ()
		where E: Encoder
	{
		encoder.emit_list(|e| {
			// insert all list elements
			for el in self.iter() {
				el.encode(e);
			}
		})
	}
}

impl<T> Encodable for Vec<T> where T: Encodable
{
	fn encode<E>(&self, encoder: &mut E) -> ()
		where E: Encoder
	{
		let r: &[T] = self.as_ref();
		r.encode(encoder)
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
				res.push(0x7fu8 + len.to_bytes_len() as u8);
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

	fn emit_list<F>(&mut self, f: F) -> ()
		where F: FnOnce(&mut Self) -> ()
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
			assert!(cat.is_value());
			assert_eq!(cat.bytes, &[0x83, b'c', b'a', b't']);
			assert_eq!(String::decode_untrusted(&cat).unwrap(), "cat".to_string());

			let dog = rlp.at(1).unwrap();
			assert!(dog.is_value());
			assert_eq!(dog.bytes, &[0x83, b'd', b'o', b'g']);
			assert_eq!(String::decode_untrusted(&dog).unwrap(), "dog".to_string());

			let cat_again = rlp.at(0).unwrap();
			assert!(cat_again.is_value());
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
			assert_eq!(cat_err, rlp::DecoderError::UntrustedRlpIsTooShort);

			let dog_err = rlp.at(1).unwrap_err();
			assert_eq!(dog_err, rlp::DecoderError::UntrustedRlpIsTooShort);
		}
	}

	#[test]
	fn rlp_iter() {
		let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
		{
			let rlp = UntrustedRlp::new(&data);
			let mut iter = rlp.iter();

			let cat = iter.next().unwrap();
			assert!(cat.is_value());
			assert_eq!(cat.bytes, &[0x83, b'c', b'a', b't']);

			let dog = iter.next().unwrap();
			assert!(dog.is_value());
			assert_eq!(dog.bytes, &[0x83, b'd', b'o', b'g']);

			let none = iter.next();
			assert!(none.is_none());

			let cat_again = rlp.at(0).unwrap();
			assert!(cat_again.is_value());
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
	fn encode_u8() {
		let tests = vec![
            ETestPair(0u8, vec![0x80u8]),
            ETestPair(15, vec![15]),
            ETestPair(55, vec![55]),
            ETestPair(56, vec![56]),
            ETestPair(0x7f, vec![0x7f]),
            ETestPair(0x80, vec![0x81, 0x80]),
            ETestPair(0xff, vec![0x81, 0xff]),
        ];
		run_encode_tests(tests);
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

	#[test]
	fn encode_vector_u8() {
		let tests = vec![
            ETestPair(vec![], vec![0xc0]),
            ETestPair(vec![15u8], vec![0xc1, 0x0f]),
            ETestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
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
	fn rlp_stream() {
		let mut stream = RlpStream::new_list(2);
		stream.append(&"cat").append(&"dog");
		let out = stream.out().unwrap();
		assert_eq!(out,
		           vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
	}

	#[test]
	fn rlp_stream_list() {
		let mut stream = RlpStream::new_list(3);
		stream.append_list(0);
		stream.append_list(1).append_list(0);
		stream.append_list(2).append_list(0).append_list(1).append_list(0);
		let out = stream.out().unwrap();
		assert_eq!(out, vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]);
	}

	struct DTestPair<T>(T, Vec<u8>) where T: rlp::Decodable + fmt::Debug + cmp::Eq;

	fn run_decode_untrusted_tests<T>(tests: Vec<DTestPair<T>>)
		where T: rlp::Decodable + fmt::Debug + cmp::Eq
	{
		for t in &tests {
			let res: T = rlp::decode_untrusted(&t.1).unwrap();
			assert_eq!(res, t.0);
		}
	}

	#[test]
	fn decode_untrusted_u8() {
		let tests = vec![
            DTestPair(0u8, vec![0u8]),
            DTestPair(15, vec![15]),
            DTestPair(55, vec![55]),
            DTestPair(56, vec![56]),
            DTestPair(0x7f, vec![0x7f]),
            DTestPair(0x80, vec![0x81, 0x80]),
            DTestPair(0xff, vec![0x81, 0xff]),
        ];
		run_decode_untrusted_tests(tests);
	}

	#[test]
	fn decode_untrusted_u16() {
		let tests = vec![
            DTestPair(0u16, vec![0u8]),
            DTestPair(0x100, vec![0x82, 0x01, 0x00]),
            DTestPair(0xffff, vec![0x82, 0xff, 0xff]),
        ];
		run_decode_untrusted_tests(tests);
	}

	#[test]
	fn decode_untrusted_u32() {
		let tests = vec![
            DTestPair(0u32, vec![0u8]),
            DTestPair(0x10000, vec![0x83, 0x01, 0x00, 0x00]),
            DTestPair(0xffffff, vec![0x83, 0xff, 0xff, 0xff]),
        ];
		run_decode_untrusted_tests(tests);
	}

	#[test]
	fn decode_untrusted_u64() {
		let tests = vec![
            DTestPair(0u64, vec![0u8]),
            DTestPair(0x1000000, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
            DTestPair(0xFFFFFFFF, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
        ];
		run_decode_untrusted_tests(tests);
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
		run_decode_untrusted_tests(tests);
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
		run_decode_untrusted_tests(tests);
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
		run_decode_untrusted_tests(tests);
	}

	#[test]
	fn decode_untrusted_vector_u8() {
		let tests = vec![
            DTestPair(vec![] as Vec<u8>, vec![0xc0]),
            DTestPair(vec![15u8], vec![0xc1, 0x0f]),
            DTestPair(vec![1u8, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
        ];
		run_decode_untrusted_tests(tests);
	}

	#[test]
	fn decode_untrusted_vector_u64() {
		let tests = vec![
            DTestPair(vec![], vec![0xc0]),
            DTestPair(vec![15u64], vec![0xc1, 0x0f]),
            DTestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
            DTestPair(vec![0xffffffff, 1, 2, 3, 7, 0xff], vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff,  1, 2, 3, 7, 0x81, 0xff]),
        ];
		run_decode_untrusted_tests(tests);
	}

	#[test]
	fn decode_untrusted_vector_str() {
		let tests = vec![DTestPair(vec!["cat".to_string(), "dog".to_string()],
		                           vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])];
		run_decode_untrusted_tests(tests);
	}

	#[test]
	fn decode_untrusted_vector_of_vectors_str() {
		let tests = vec![DTestPair(vec![vec!["cat".to_string()]],
		                           vec![0xc5, 0xc4, 0x83, b'c', b'a', b't'])];
		run_decode_untrusted_tests(tests);
	}
}
