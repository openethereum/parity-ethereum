use std::cell::Cell;
use bytes::{FromBytes};
use super::faces::{View, Decoder, Decodable, DecoderError};

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
pub enum Prototype {
	Null,
	Data(usize),
	List(usize),
}

/// Stores basic information about item
pub struct PayloadInfo {
	pub header_len: usize,
	pub value_len: usize,
}

impl PayloadInfo {
	fn new(header_len: usize, value_len: usize) -> PayloadInfo {
		PayloadInfo {
			header_len: header_len,
			value_len: value_len,
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
	cache: Cell<OffsetCache>,
}

impl<'a> Clone for UntrustedRlp<'a> {
	fn clone(&self) -> UntrustedRlp<'a> {
		UntrustedRlp {
			bytes: self.bytes,
			cache: Cell::new(OffsetCache::new(usize::max_value(), 0))
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
			cache: Cell::new(OffsetCache::new(usize::max_value(), 0)),
		}
	}
	
	fn raw(&'view self) -> &'a [u8] {
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
			true => self.iter().count(),
			false => 0
		}
	}

	fn size(&self) -> usize {
		match self.is_data() {
			// we can safely unwrap (?) cause its data
			true => BasicDecoder::payload_info(self.bytes).unwrap().value_len,
			false => 0
		}
	}

	fn at(&'view self, index: usize) -> Self::Item {
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

	fn as_val<T>(&self) -> Result<T, DecoderError> where T: Decodable {
		// optimize, so it doesn't use clone (although This clone is cheap)
		T::decode(&BasicDecoder::new(self.clone()))
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

	/// Return first item info
	fn payload_info(bytes: &[u8]) -> Result<PayloadInfo, DecoderError> {
		let item = match bytes.first().map(|&x| x) {
			None => return Err(DecoderError::RlpIsTooShort),
			Some(0...0x7f) => PayloadInfo::new(0, 1),
			Some(l @ 0x80...0xb7) => PayloadInfo::new(1, l as usize - 0x80),
			Some(l @ 0xb8...0xbf) => {
				let len_of_len = l as usize - 0xb7;
				let header_len = 1 + len_of_len;
				let value_len = try!(usize::from_bytes(&bytes[1..header_len]));
				PayloadInfo::new(header_len, value_len)
			}
			Some(l @ 0xc0...0xf7) => PayloadInfo::new(1, l as usize - 0xc0),
			Some(l @ 0xf8...0xff) => {
				let len_of_len = l as usize - 0xf7;
				let header_len = 1 + len_of_len;
				let value_len = try!(usize::from_bytes(&bytes[1..header_len]));
				PayloadInfo::new(header_len, value_len)
			},
			// we cant reach this place, but rust requires _ to be implemented
			_ => { unreachable!(); }
		};

		match item.header_len + item.value_len <= bytes.len() {
			true => Ok(item),
			false => Err(DecoderError::RlpIsTooShort),
		}
	}
}

impl<'a> Decoder for BasicDecoder<'a> {
	fn read_value<T, F>(&self, f: F) -> Result<T, DecoderError>
		where F: FnOnce(&[u8]) -> Result<T, DecoderError> {

		let bytes = self.rlp.raw();

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
			// we are reading value, not a list!
			_ => Err(DecoderError::RlpExpectedToBeData)
		}
	}

	fn read_list<T, F>(&self, f: F) -> Result<T, DecoderError>
		where F: FnOnce(&[Self]) -> Result<T, DecoderError> {

		let v: Vec<BasicDecoder<'a>> = self.rlp.iter()
			.map(| i | BasicDecoder::new(i))
			.collect();
		f(&v)
	}
}

impl<T> Decodable for T where T: FromBytes {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder {
		decoder.read_value(| bytes | {
			Ok(try!(T::from_bytes(bytes)))
		})
	}
}

impl<T> Decodable for Vec<T> where T: Decodable {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder {
		decoder.read_list(| decoders | {
			decoders.iter().map(|d| T::decode(d)).collect()
		})
	}
}

impl Decodable for Vec<u8> {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder {
		decoder.read_value(| bytes | {
			let mut res = vec![];
			res.extend(bytes);
			Ok(res)
		})
	}
}
