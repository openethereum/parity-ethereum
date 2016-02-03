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

pub mod rlptraits;
mod rlperrors;
mod rlpin;
mod untrusted_rlp;
mod rlpstream;
mod bytes;

#[cfg(test)]
mod tests;

pub use self::rlperrors::DecoderError;
pub use self::rlptraits::{Decoder, Decodable, View, Stream, Encodable, Encoder, RlpEncodable, RlpDecodable};
pub use self::untrusted_rlp::{UntrustedRlp, UntrustedRlpIterator, PayloadInfo, Prototype};
pub use self::rlpin::{Rlp, RlpIterator};
pub use self::rlpstream::{RlpStream};
pub use elastic_array::ElasticArray1024;
use super::hash::H256;

/// The RLP encoded empty data (used to mean "null value").
pub const NULL_RLP: [u8; 1] = [0x80; 1];
/// The RLP encoded empty list.
pub const EMPTY_LIST_RLP: [u8; 1] = [0xC0; 1];
/// The SHA3 of the RLP encoding of empty data.
pub const SHA3_NULL_RLP: H256 = H256( [0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21] );
/// The SHA3 of the RLP encoding of empty list.
pub const SHA3_EMPTY_LIST_RLP: H256 = H256( [0x1d, 0xcc, 0x4d, 0xe8, 0xde, 0xc7, 0x5d, 0x7a, 0xab, 0x85, 0xb5, 0x67, 0xb6, 0xcc, 0xd4, 0x1a, 0xd3, 0x12, 0x45, 0x1b, 0x94, 0x8a, 0x74, 0x13, 0xf0, 0xa1, 0x42, 0xfd, 0x40, 0xd4, 0x93, 0x47] );

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
pub fn decode<T>(bytes: &[u8]) -> T where T: RlpDecodable {
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
/// 	let animal = "cat";
/// 	let out = encode(&animal).to_vec();
/// 	assert_eq!(out, vec![0x83, b'c', b'a', b't']);
/// }
/// ```
pub fn encode<E>(object: &E) -> ElasticArray1024<u8> where E: RlpEncodable {
	let mut stream = RlpStream::new();
	stream.append(object);
	stream.drain()
}
