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

pub mod old;

pub mod faces;
pub mod rlp;
pub mod untrusted_rlp;
pub mod rlpstream;

pub use self::faces::{DecoderError, Decoder, Decodable, View};
pub use self::rlp::*;
pub use self::untrusted_rlp::*;

pub use self::old::{encode, RlpStream, Encodable};
//pub use self::old::*;

pub fn decode<T>(bytes: &[u8]) -> T where T: Decodable {
	let rlp = Rlp::new(bytes);
	rlp.as_val()
}
