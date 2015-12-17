use rlp::DecoderError;

pub trait Decoder: Sized {
	fn read_value<T, F>(&self, f: F) -> Result<T, DecoderError>
		where F: FnOnce(&[u8]) -> Result<T, DecoderError>;

	fn as_list(&self) -> Result<Vec<Self>, DecoderError>;
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
	/// 	let dog: String = rlp.at(1).as_val();
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
	/// 	let strings: Vec<String> = rlp.iter().map(| i | i.as_val()).collect();
	/// }
	/// ```
	fn iter(&'view self) -> Self::Iter;

	fn as_val<T>(&self) -> Result<T, DecoderError> where T: Decodable;

	fn val_at<T>(&self, index: usize) -> Result<T, DecoderError> where T: Decodable;
}

pub trait Encoder {
	fn emit_value(&mut self, bytes: &[u8]) -> ();
	fn emit_list<F>(&mut self, f: F) -> () where F: FnOnce(&mut Self) -> ();
}

pub trait Encodable {
	fn encode<E>(&self, encoder: &mut E) -> () where E: Encoder;
}

pub trait Stream: Sized {

	/// Initializes instance of empty `Stream`.
	fn new() -> Self;

	/// Initializes the `Stream` as a list.
	fn new_list(len: usize) -> Self;

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
	fn append<'a, E>(&'a mut self, object: &E) -> &'a mut Self where E: Encodable;

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
	fn append_list<'a>(&'a mut self, len: usize) -> &'a mut Self;

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
	fn append_empty_data<'a>(&'a mut self) -> &'a mut Self;

	/// Appends raw (pre-serialised) RLP data. Use with caution. Chainable.
	fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut Self;

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
	fn clear(&mut self);

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
	fn is_finished(&self) -> bool;

	fn raw(&self) -> &[u8];

	/// Streams out encoded bytes.
	/// 
	/// panic! if stream is not finished.
	fn out(self) -> Vec<u8>;
}
