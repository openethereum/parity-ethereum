// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Common RLP traits
use elastic_array::ElasticArray1024;
use stream::RlpStream;
use {DecoderError, UntrustedRlp};

/// Type is able to decode RLP.
pub trait Decoder: Sized {
	/// Read a value from the RLP into a given type.
	fn read_value<T, F>(&self, f: &F) -> Result<T, DecoderError>
		where F: Fn(&[u8]) -> Result<T, DecoderError>;

	/// Get underlying `UntrustedRLP` object.
	fn as_rlp(&self) -> &UntrustedRlp;
	/// Get underlying raw bytes slice.
	fn as_raw(&self) -> &[u8];
}

/// RLP decodable trait
pub trait Decodable: Sized {
	/// Decode a value from RLP bytes
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder;
}

/// Internal helper trait. Implement `Decodable` for custom types.
pub trait RlpDecodable: Sized {
	/// Decode a value from RLP bytes
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder;
}

/// A view into RLP encoded data
pub trait View<'a, 'view>: Sized {
	/// RLP prototype type
	type Prototype;
	/// Payload info type
	type PayloadInfo;
	/// Data type
	type Data;
	/// Item type
	type Item;
	/// Iterator type
	type Iter;

	/// Creates a new instance of `Rlp` reader
	fn new(bytes: &'a [u8]) -> Self;

	/// The raw data of the RLP as slice.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let dog = rlp.at(1).as_raw();
	/// 	assert_eq!(dog, &[0x83, b'd', b'o', b'g']);
	/// }
	/// ```
	fn as_raw(&'view self) -> &'a [u8];

	/// Get the prototype of the RLP.
	fn prototype(&self) -> Self::Prototype;

	/// Get payload info.
	fn payload_info(&self) -> Self::PayloadInfo;

	/// Get underlieing data.
	fn data(&'view self) -> Self::Data;

	/// Returns number of RLP items.
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
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
	/// extern crate rlp;
	/// use rlp::*;
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
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let dog: String = rlp.at(1).as_val();
	/// 	assert_eq!(dog, "dog".to_string());
	/// }
	fn at(&'view self, index: usize) -> Self::Item;

	/// No value
	///
	/// ```rust
	/// extern crate rlp;
	/// use rlp::*;
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
	/// extern crate rlp;
	/// use rlp::*;
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
	/// extern crate rlp;
	/// use rlp::*;
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
	/// extern crate rlp;
	/// use rlp::*;
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
	/// extern crate rlp;
	/// use rlp::*;
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
	/// extern crate rlp;
	/// use rlp::*;
	///
	/// fn main () {
	/// 	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	/// 	let rlp = Rlp::new(&data);
	/// 	let strings: Vec<String> = rlp.iter().map(| i | i.as_val()).collect();
	/// }
	/// ```
	fn iter(&'view self) -> Self::Iter;

	/// Decode data into an object
	fn as_val<T>(&self) -> Result<T, DecoderError> where T: RlpDecodable;

	/// Decode data at given list index into an object
	fn val_at<T>(&self, index: usize) -> Result<T, DecoderError> where T: RlpDecodable;
}

/// Structure encodable to RLP
pub trait Encodable {
	/// Append a value to the stream
	fn rlp_append(&self, s: &mut RlpStream);

	/// Get rlp-encoded bytes for this instance
	fn rlp_bytes(&self) -> ElasticArray1024<u8> {
		let mut s = RlpStream::new();
		self.rlp_append(&mut s);
		s.drain()
	}
}

/// Trait for compressing and decompressing RLP by replacement of common terms.
pub trait Compressible: Sized {
	/// Indicates the origin of RLP to be compressed.
	type DataType;

	/// Compress given RLP type using appropriate methods.
	fn compress(&self, t: Self::DataType) -> ElasticArray1024<u8>;
	/// Decompress given RLP type using appropriate methods.
	fn decompress(&self, t: Self::DataType) -> ElasticArray1024<u8>;
}
