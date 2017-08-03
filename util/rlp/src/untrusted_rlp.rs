// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::Cell;
use std::fmt;
use rustc_hex::ToHex;
use impls::decode_usize;
use {Decodable, DecoderError};

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

#[derive(Debug)]
/// RLP prototype
pub enum Prototype {
	/// Empty
	Null,
	/// Value
	Data(usize),
	/// List
	List(usize),
}

/// Stores basic information about item
pub struct PayloadInfo {
	/// Header length in bytes
	pub header_len: usize,
	/// Value length in bytes
	pub value_len: usize,
}

fn calculate_payload_info(header_bytes: &[u8], len_of_len: usize) -> Result<PayloadInfo, DecoderError> {
	let header_len = 1 + len_of_len;
	match header_bytes.get(1) {
		Some(&0) => return Err(DecoderError::RlpDataLenWithZeroPrefix),
		None => return Err(DecoderError::RlpIsTooShort),
		_ => (),
	}
	if header_bytes.len() < header_len { return Err(DecoderError::RlpIsTooShort); }
	let value_len = decode_usize(&header_bytes[1..header_len])?;
	Ok(PayloadInfo::new(header_len, value_len))
}

impl PayloadInfo {
	fn new(header_len: usize, value_len: usize) -> PayloadInfo {
		PayloadInfo {
			header_len: header_len,
			value_len: value_len,
		}
	}

	/// Total size of the RLP.
	pub fn total(&self) -> usize { self.header_len + self.value_len }

	/// Create a new object from the given bytes RLP. The bytes
	pub fn from(header_bytes: &[u8]) -> Result<PayloadInfo, DecoderError> {
		match header_bytes.first().cloned() {
			None => Err(DecoderError::RlpIsTooShort),
			Some(0...0x7f) => Ok(PayloadInfo::new(0, 1)),
			Some(l @ 0x80...0xb7) => Ok(PayloadInfo::new(1, l as usize - 0x80)),
			Some(l @ 0xb8...0xbf) => {
				let len_of_len = l as usize - 0xb7;
				calculate_payload_info(header_bytes, len_of_len)
			}
			Some(l @ 0xc0...0xf7) => Ok(PayloadInfo::new(1, l as usize - 0xc0)),
			Some(l @ 0xf8...0xff) => {
				let len_of_len = l as usize - 0xf7;
				calculate_payload_info(header_bytes, len_of_len)
			},
			// we cant reach this place, but rust requires _ to be implemented
			_ => { unreachable!(); }
		}
	}
}

/// Data-oriented view onto rlp-slice.
///
/// This is an immutable structure. No operations change it.
///
/// Should be used in places where, error handling is required,
/// eg. on input
#[derive(Debug)]
pub struct UntrustedRlp<'a> {
	bytes: &'a [u8],
	offset_cache: Cell<OffsetCache>,
	count_cache: Cell<Option<usize>>,
}

impl<'a> Clone for UntrustedRlp<'a> {
	fn clone(&self) -> UntrustedRlp<'a> {
		UntrustedRlp {
			bytes: self.bytes,
			offset_cache: self.offset_cache.clone(),
			count_cache: self.count_cache.clone(),
		}
	}
}

impl<'a> fmt::Display for UntrustedRlp<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self.prototype() {
			Ok(Prototype::Null) => write!(f, "null"),
			Ok(Prototype::Data(_)) => write!(f, "\"0x{}\"", self.data().unwrap().to_hex()),
			Ok(Prototype::List(len)) => {
				write!(f, "[")?;
				for i in 0..len-1 {
					write!(f, "{}, ", self.at(i).unwrap())?;
				}
				write!(f, "{}", self.at(len - 1).unwrap())?;
				write!(f, "]")
			},
			Err(err) => write!(f, "{:?}", err)
		}
	}
}

impl<'a, 'view> UntrustedRlp<'a> where 'a: 'view {
	pub fn new(bytes: &'a [u8]) -> UntrustedRlp<'a> {
		UntrustedRlp {
			bytes: bytes,
			offset_cache: Cell::new(OffsetCache::new(usize::max_value(), 0)),
			count_cache: Cell::new(None)
		}
	}

	pub fn as_raw(&'view self) -> &'a [u8] {
		self.bytes
	}

	pub fn prototype(&self) -> Result<Prototype, DecoderError> {
		// optimize? && return appropriate errors
		if self.is_data() {
			Ok(Prototype::Data(self.size()))
		} else if self.is_list() {
			self.item_count().map(Prototype::List)
		} else {
			Ok(Prototype::Null)
		}
	}

	pub fn payload_info(&self) -> Result<PayloadInfo, DecoderError> {
		BasicDecoder::payload_info(self.bytes)
	}

	pub fn data(&'view self) -> Result<&'a [u8], DecoderError> {
		let pi = BasicDecoder::payload_info(self.bytes)?;
		Ok(&self.bytes[pi.header_len..(pi.header_len + pi.value_len)])
	}

	pub fn item_count(&self) -> Result<usize, DecoderError> {
		match self.is_list() {
			true => match self.count_cache.get() {
				Some(c) => Ok(c),
				None => {
					let c = self.iter().count();
					self.count_cache.set(Some(c));
					Ok(c)
				}
			},
			false => Err(DecoderError::RlpExpectedToBeList),
		}
	}

	pub fn size(&self) -> usize {
		match self.is_data() {
			// TODO: No panic on malformed data, but ideally would Err on no PayloadInfo.
			true => BasicDecoder::payload_info(self.bytes).map(|b| b.value_len).unwrap_or(0),
			false => 0
		}
	}

	pub fn at(&'view self, index: usize) -> Result<UntrustedRlp<'a>, DecoderError> {
		if !self.is_list() {
			return Err(DecoderError::RlpExpectedToBeList);
		}

		// move to cached position if its index is less or equal to
		// current search index, otherwise move to beginning of list
		let c = self.offset_cache.get();
		let (mut bytes, to_skip) = match c.index <= index {
			true => (UntrustedRlp::consume(self.bytes, c.offset)?, index - c.index),
			false => (self.consume_list_payload()?, index),
		};

		// skip up to x items
		bytes = UntrustedRlp::consume_items(bytes, to_skip)?;

		// update the cache
		self.offset_cache.set(OffsetCache::new(index, self.bytes.len() - bytes.len()));

		// construct new rlp
		let found = BasicDecoder::payload_info(bytes)?;
		Ok(UntrustedRlp::new(&bytes[0..found.header_len + found.value_len]))
	}

	pub fn is_null(&self) -> bool {
		self.bytes.len() == 0
	}

	pub fn is_empty(&self) -> bool {
		!self.is_null() && (self.bytes[0] == 0xc0 || self.bytes[0] == 0x80)
	}

	pub fn is_list(&self) -> bool {
		!self.is_null() && self.bytes[0] >= 0xc0
	}

	pub fn is_data(&self) -> bool {
		!self.is_null() && self.bytes[0] < 0xc0
	}

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

	pub fn iter(&'view self) -> UntrustedRlpIterator<'a, 'view> {
		self.into_iter()
	}

	pub fn as_val<T>(&self) -> Result<T, DecoderError> where T: Decodable {
		T::decode(self)
	}

	pub fn as_list<T>(&self) -> Result<Vec<T>, DecoderError> where T: Decodable {
		self.iter().map(|rlp| rlp.as_val()).collect()
	}

	pub fn val_at<T>(&self, index: usize) -> Result<T, DecoderError> where T: Decodable {
		self.at(index)?.as_val()
	}

	pub fn list_at<T>(&self, index: usize) -> Result<Vec<T>, DecoderError> where T: Decodable {
		self.at(index)?.as_list()
	}

	pub fn decoder(&self) -> BasicDecoder {
		BasicDecoder::new(self.clone())
	}

	/// consumes first found prefix
	fn consume_list_payload(&self) -> Result<&'a [u8], DecoderError> {
		let item = BasicDecoder::payload_info(self.bytes)?;
		let bytes = UntrustedRlp::consume(self.bytes, item.header_len)?;
		Ok(bytes)
	}

	/// consumes fixed number of items
	fn consume_items(bytes: &'a [u8], items: usize) -> Result<&'a [u8], DecoderError> {
		let mut result = bytes;
		for _ in 0..items {
			let i = BasicDecoder::payload_info(result)?;
			result = UntrustedRlp::consume(result, (i.header_len + i.value_len))?;
		}
		Ok(result)
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
pub struct UntrustedRlpIterator<'a, 'view> where 'a: 'view {
	rlp: &'view UntrustedRlp<'a>,
	index: usize,
}

impl<'a, 'view> IntoIterator for &'view UntrustedRlp<'a> where 'a: 'view {
	type Item = UntrustedRlp<'a>;
	type IntoIter = UntrustedRlpIterator<'a, 'view>;

	fn into_iter(self) -> Self::IntoIter {
		UntrustedRlpIterator {
			rlp: self,
			index: 0,
		}
	}
}

impl<'a, 'view> Iterator for UntrustedRlpIterator<'a, 'view> {
	type Item = UntrustedRlp<'a>;

	fn next(&mut self) -> Option<UntrustedRlp<'a>> {
		let index = self.index;
		let result = self.rlp.at(index).ok();
		self.index += 1;
		result
	}
}

pub struct BasicDecoder<'a> {
	rlp: UntrustedRlp<'a>
}

impl<'a> BasicDecoder<'a> {
	pub fn new(rlp: UntrustedRlp<'a>) -> BasicDecoder<'a> {
		BasicDecoder {
			rlp: rlp
		}
	}

	/// Return first item info.
	fn payload_info(bytes: &[u8]) -> Result<PayloadInfo, DecoderError> {
		let item = PayloadInfo::from(bytes)?;
		match item.header_len.checked_add(item.value_len) {
			Some(x) if x <= bytes.len() => Ok(item),
			_ => Err(DecoderError::RlpIsTooShort),
		}
	}

	pub fn decode_value<T, F>(&self, f: F) -> Result<T, DecoderError>
		where F: Fn(&[u8]) -> Result<T, DecoderError> {

		let bytes = self.rlp.as_raw();

		match bytes.first().cloned() {
			// RLP is too short.
			None => Err(DecoderError::RlpIsTooShort),
			// Single byte value.
			Some(l @ 0...0x7f) => Ok(f(&[l])?),
			// 0-55 bytes
			Some(l @ 0x80...0xb7) => {
				let last_index_of = 1 + l as usize - 0x80;
				if bytes.len() < last_index_of {
					return Err(DecoderError::RlpInconsistentLengthAndData);
				}
				let d = &bytes[1..last_index_of];
				if l == 0x81 && d[0] < 0x80 {
					return Err(DecoderError::RlpInvalidIndirection);
				}
				Ok(f(d)?)
			},
			// Longer than 55 bytes.
			Some(l @ 0xb8...0xbf) => {
				let len_of_len = l as usize - 0xb7;
				let begin_of_value = 1 as usize + len_of_len;
				if bytes.len() < begin_of_value {
					return Err(DecoderError::RlpInconsistentLengthAndData);
				}
				let len = decode_usize(&bytes[1..begin_of_value])?;

				let last_index_of_value = begin_of_value.checked_add(len)
					.ok_or(DecoderError::RlpInvalidLength)?;
				if bytes.len() < last_index_of_value {
					return Err(DecoderError::RlpInconsistentLengthAndData);
				}
				Ok(f(&bytes[begin_of_value..last_index_of_value])?)
			}
			// We are reading value, not a list!
			_ => Err(DecoderError::RlpExpectedToBeData)
		}
	}
}

#[cfg(test)]
mod tests {
	use {UntrustedRlp, DecoderError};

	#[test]
	fn test_rlp_display() {
		use rustc_hex::FromHex;
		let data = "f84d0589010efbef67941f79b2a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470".from_hex().unwrap();
		let rlp = UntrustedRlp::new(&data);
		assert_eq!(format!("{}", rlp), "[\"0x05\", \"0x010efbef67941f79b2\", \"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421\", \"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470\"]");
	}

	#[test]
	fn length_overflow() {
		let bs = [0xbf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xe5];
		let rlp = UntrustedRlp::new(&bs);
		let res: Result<u8, DecoderError> = rlp.as_val();
		assert_eq!(Err(DecoderError::RlpInvalidLength), res);
	}
}
