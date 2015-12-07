use std::fmt;
use std::error::Error as StdError;
use bytes::FromBytesError;

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

pub trait Decoder {
	fn read_value<T, F>(&self, f: F) -> Result<T, DecoderError>
		where F: FnOnce(&[u8]) -> Result<T, DecoderError>;

	fn read_list<T, F>(&self, f: F) -> Result<T, DecoderError>
		where F: FnOnce(&[Self]) -> Result<T, DecoderError>;
}

pub trait Decodable: Sized {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder;
}

pub trait View<'a, 'view>: Sized {
	type Prototype;
	type PayloadInfo;
	type Data;
	type Item;
	type Iter;

	/// Creates a new instance of `Rlp` reader
	fn new(bytes: &'a [u8]) -> Self;

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
	fn raw(&'view self) -> &'a [u8];	

	/// Get the prototype of the RLP.
	fn prototype(&self) -> Self::Prototype;

	fn payload_info(&self) -> Self::PayloadInfo;

	fn data(&'view self) -> Self::Data;

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
	fn item_count(&self) -> usize;

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
	fn size(&self) -> usize;

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
	fn at(&'view self, index: usize) -> Self::Item;

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
	fn is_null(&self) -> bool;

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
	fn is_empty(&self) -> bool;

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
	fn is_list(&self) -> bool;

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
	fn is_data(&self) -> bool;

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
	fn is_int(&self) -> bool;

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
	fn iter(&'view self) -> Self::Iter;

	fn as_val<T>(&self) -> Result<T, DecoderError> where T: Decodable;
}

pub trait Encoder {
	fn emit_value(&mut self, bytes: &[u8]) -> ();
	fn emit_list<F>(&mut self, f: F) -> () where F: FnOnce(&mut Self) -> ();
}

pub trait Encodable {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder;
}

pub trait Stream: Sized {
	fn new() -> Self;
	fn new_list(len: usize) -> Self;
	fn append<'a, E>(&'a mut self, object: &E) -> &'a mut Self;
}
