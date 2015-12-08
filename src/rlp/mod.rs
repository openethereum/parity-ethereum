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

pub mod errors;
pub mod traits;
pub mod rlp;
pub mod untrusted_rlp;
pub mod rlpstream;

#[cfg(test)]
mod tests;

pub use self::errors::DecoderError;
pub use self::traits::{Decoder, Decodable, View, Stream, Encodable, Encoder};
pub use self::rlp::{Rlp, RlpIterator};
pub use self::untrusted_rlp::{UntrustedRlp, UntrustedRlpIterator, Prototype, PayloadInfo};
pub use self::rlpstream::{RlpStream};

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
	rlp.as_val()
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
	let mut stream = RlpStream::new();
	stream.append(object);
	stream.out()
}
