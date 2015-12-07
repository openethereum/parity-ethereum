use std::fmt;
use std::cell::Cell;
use std::error::Error as StdError;
use bytes::{FromBytesError};
use super::faces::Reader;

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

#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
	FromBytesError(FromBytesError),
	RlpIsTooShort,
	RlpExpectedToBeList,
	RlpExpectedToBeData,
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

impl<'a, 'view> Reader<'a, 'view> for UntrustedRlp<'a> where 'a: 'view {
	type Prototype = Result<Prototype, DecoderError>;
	type PayloadInfo = Result<PayloadInfo, DecoderError>;
	type Data = Result<&'a [u8], DecoderError>;
	type Item = Result<UntrustedRlp<'a>, DecoderError>;

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
		unimplemented!()
	}

	fn payload_info(&self) -> Self::PayloadInfo {
		unimplemented!()
	}

	fn data(&'view self) -> Self::Data {
		unimplemented!()
	}

	fn item_count(&self) -> usize {
		unimplemented!()
	}

	fn size(&self) -> usize {
		unimplemented!()
	}

	fn at(&'view self, index: usize) -> Self::Item {
		unimplemented!()
	}

	fn is_null(&self) -> bool {
		unimplemented!()
	}

	fn is_empty(&self) -> bool {
		unimplemented!()
	}

	fn is_list(&self) -> bool {
		unimplemented!()
	}

	fn is_data(&self) -> bool {
		unimplemented!()
	}

	fn is_int(&self) -> bool {
		unimplemented!()
	}
}
