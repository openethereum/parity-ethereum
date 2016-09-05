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

use std::cell::Cell;
use std::fmt;
use rustc_serialize::hex::ToHex;

use bytes::{FromBytes, FromBytesResult, FromBytesError};
use ::{View, Decoder, Decodable, DecoderError, RlpDecodable};

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
	let value_len = try!(usize::from_bytes(&header_bytes[1..header_len]));
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
/// This is immutable structere. No operations change it.
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
				try!(write!(f, "["));
				for i in 0..len-1 {
					try!(write!(f, "{}, ", self.at(i).unwrap()));
				}
				try!(write!(f, "{}", self.at(len - 1).unwrap()));
				write!(f, "]")
			},
			Err(err) => write!(f, "{:?}", err)
		}
	}
}

impl<'a, 'view> View<'a, 'view> for UntrustedRlp<'a> where 'a: 'view {
	type Prototype = Result<Prototype, DecoderError>;
	type PayloadInfo = Result<PayloadInfo, DecoderError>;
	type Data = Result<&'a [u8], DecoderError>;
	type Item = Result<UntrustedRlp<'a>, DecoderError>;
	type Iter = UntrustedRlpIterator<'a, 'view>;

	//returns new instance of `UntrustedRlp`
	fn new(bytes: &'a [u8]) -> UntrustedRlp<'a> {
		UntrustedRlp {
			bytes: bytes,
			offset_cache: Cell::new(OffsetCache::new(usize::max_value(), 0)),
			count_cache: Cell::new(None)
		}
	}

	fn as_raw(&'view self) -> &'a [u8] {
		self.bytes
	}

	fn prototype(&self) -> Self::Prototype {
		// optimize? && return appropriate errors
		if self.is_data() {
			Ok(Prototype::Data(self.size()))
		} else if self.is_list() {
			Ok(Prototype::List(self.item_count()))
		} else {
			Ok(Prototype::Null)
		}
	}

	fn payload_info(&self) -> Self::PayloadInfo {
		BasicDecoder::payload_info(self.bytes)
	}

	fn data(&'view self) -> Self::Data {
		let pi = try!(BasicDecoder::payload_info(self.bytes));
		Ok(&self.bytes[pi.header_len..(pi.header_len + pi.value_len)])
	}

	fn item_count(&self) -> usize {
		match self.is_list() {
			true => match self.count_cache.get() {
				Some(c) => c,
				None => {
					let c = self.iter().count();
					self.count_cache.set(Some(c));
					c
				}
			},
			false => 0
		}
	}

	fn size(&self) -> usize {
		match self.is_data() {
			// TODO: No panic on malformed data, but ideally would Err on no PayloadInfo.
			true => BasicDecoder::payload_info(self.bytes).map(|b| b.value_len).unwrap_or(0),
			false => 0
		}
	}

	fn at(&'view self, index: usize) -> Self::Item {
		if !self.is_list() {
			return Err(DecoderError::RlpExpectedToBeList);
		}

		// move to cached position if its index is less or equal to
		// current search index, otherwise move to beginning of list
		let c = self.offset_cache.get();
		let (mut bytes, to_skip) = match c.index <= index {
			true => (try!(UntrustedRlp::consume(self.bytes, c.offset)), index - c.index),
			false => (try!(self.consume_list_prefix()), index),
		};

		// skip up to x items
		bytes = try!(UntrustedRlp::consume_items(bytes, to_skip));

		// update the cache
		self.offset_cache.set(OffsetCache::new(index, self.bytes.len() - bytes.len()));

		// construct new rlp
		let found = try!(BasicDecoder::payload_info(bytes));
		Ok(UntrustedRlp::new(&bytes[0..found.header_len + found.value_len]))
	}

	fn is_null(&self) -> bool {
		self.bytes.len() == 0
	}

	fn is_empty(&self) -> bool {
		!self.is_null() && (self.bytes[0] == 0xc0 || self.bytes[0] == 0x80)
	}

	fn is_list(&self) -> bool {
		!self.is_null() && self.bytes[0] >= 0xc0
	}

	fn is_data(&self) -> bool {
		!self.is_null() && self.bytes[0] < 0xc0
	}

	fn is_int(&self) -> bool {
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

	fn iter(&'view self) -> Self::Iter {
		self.into_iter()
	}

	fn as_val<T>(&self) -> Result<T, DecoderError> where T: RlpDecodable {
		// optimize, so it doesn't use clone (although This clone is cheap)
		T::decode(&BasicDecoder::new(self.clone()))
	}

	fn val_at<T>(&self, index: usize) -> Result<T, DecoderError> where T: RlpDecodable {
		try!(self.at(index)).as_val()
	}
}

impl<'a> UntrustedRlp<'a> {
	/// consumes first found prefix
	fn consume_list_prefix(&self) -> Result<&'a [u8], DecoderError> {
		let item = try!(BasicDecoder::payload_info(self.bytes));
		let bytes = try!(UntrustedRlp::consume(self.bytes, item.header_len));
		Ok(bytes)
	}

	/// consumes fixed number of items
	fn consume_items(bytes: &'a [u8], items: usize) -> Result<&'a [u8], DecoderError> {
		let mut result = bytes;
		for _ in 0..items {
			let i = try!(BasicDecoder::payload_info(result));
			result = try!(UntrustedRlp::consume(result, (i.header_len + i.value_len)));
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

struct BasicDecoder<'a> {
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
		let item = try!(PayloadInfo::from(bytes));
		match item.header_len.checked_add(item.value_len) {
			Some(x) if x <= bytes.len() => Ok(item),
			_ => Err(DecoderError::RlpIsTooShort),
		}
	}
}

impl<'a> Decoder for BasicDecoder<'a> {
	fn read_value<T, F>(&self, f: &F) -> Result<T, DecoderError>
		where F: Fn(&[u8]) -> Result<T, DecoderError> {

		let bytes = self.rlp.as_raw();

		match bytes.first().cloned() {
			// RLP is too short.
			None => Err(DecoderError::RlpIsTooShort),
			// Single byte value.
			Some(l @ 0...0x7f) => Ok(try!(f(&[l]))),
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
				Ok(try!(f(d)))
			},
			// Longer than 55 bytes.
			Some(l @ 0xb8...0xbf) => {
				let len_of_len = l as usize - 0xb7;
				let begin_of_value = 1 as usize + len_of_len;
				if bytes.len() < begin_of_value {
					return Err(DecoderError::RlpInconsistentLengthAndData);
				}
				let len = try!(usize::from_bytes(&bytes[1..begin_of_value]));

				let last_index_of_value = begin_of_value + len;
				if bytes.len() < last_index_of_value {
					return Err(DecoderError::RlpInconsistentLengthAndData);
				}
				Ok(try!(f(&bytes[begin_of_value..last_index_of_value])))
			}
			// We are reading value, not a list!
			_ => Err(DecoderError::RlpExpectedToBeData)
		}
	}

	fn as_raw(&self) -> &[u8] {
		self.rlp.as_raw()
	}

	fn as_rlp(&self) -> &UntrustedRlp {
		&self.rlp
	}
}

impl<T> Decodable for T where T: FromBytes {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		decoder.read_value(&|bytes: &[u8]| Ok(try!(T::from_bytes(bytes))))
	}
}

impl<T> Decodable for Vec<T> where T: Decodable {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		decoder.as_rlp().iter().map(|d| T::decode(&BasicDecoder::new(d))).collect()
	}
}

impl<T> Decodable for Option<T> where T: Decodable {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		decoder.as_rlp().iter().map(|d| T::decode(&BasicDecoder::new(d))).collect::<Result<Vec<_>, DecoderError>>().map(|mut a| a.pop())
	}
}

impl Decodable for Vec<u8> {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		decoder.read_value(&|bytes: &[u8]| Ok(bytes.to_vec()))
	}
}

macro_rules! impl_array_decodable {
	($index_type:ty, $len:expr ) => (
		impl<T> Decodable for [T; $len] where T: Decodable {
			fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
				let decoders = decoder.as_rlp();

				let mut result: [T; $len] = unsafe { ::std::mem::uninitialized() };
				if decoders.item_count() != $len {
					return Err(DecoderError::RlpIncorrectListLen);
				}

				for i in 0..decoders.item_count() {
					result[i] = try!(T::decode(&BasicDecoder::new(try!(decoders.at(i)))));
				}

				Ok(result)
			}
		}
	)
}

macro_rules! impl_array_decodable_recursive {
	($index_type:ty, ) => ();
	($index_type:ty, $len:expr, $($more:expr,)*) => (
		impl_array_decodable!($index_type, $len);
		impl_array_decodable_recursive!($index_type, $($more,)*);
	);
}

impl_array_decodable_recursive!(
	u8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
	16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
	32, 40, 48, 56, 64, 72, 96, 128, 160, 192, 224,
);

impl<T> RlpDecodable for T where T: Decodable {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		Decodable::decode(decoder)
	}
}

struct DecodableU8 (u8);

impl FromBytes for DecodableU8 {
	fn from_bytes(bytes: &[u8]) -> FromBytesResult<DecodableU8> {
		match bytes.len() {
			0 => Ok(DecodableU8(0u8)),
			1 => {
				if bytes[0] == 0 {
					return Err(FromBytesError::ZeroPrefixedInt)
				}
				Ok(DecodableU8(bytes[0]))
			}
			_ => Err(FromBytesError::DataIsTooLong)
		}
	}
}

impl RlpDecodable for u8 {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let u: DecodableU8 = try!(Decodable::decode(decoder));
		Ok(u.0)
	}
}

#[cfg(test)]
mod tests {
	use ::{UntrustedRlp, View};
	#[test]
	fn test_rlp_display() {
		use rustc_serialize::hex::FromHex;
		let data = "f84d0589010efbef67941f79b2a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470".from_hex().unwrap();
		let rlp = UntrustedRlp::new(&data);
		assert_eq!(format!("{}", rlp), "[\"0x05\", \"0x010efbef67941f79b2\", \"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421\", \"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470\"]");
	}
}
