// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Recursive Length Prefix serialization crate.
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

extern crate byteorder;
extern crate ethereum_types as bigint;
extern crate elastic_array;
extern crate rustc_hex;

mod traits;
mod error;
mod rlpin;
mod untrusted_rlp;
mod stream;
mod impls;

use std::borrow::Borrow;
use elastic_array::ElasticArray1024;

pub use error::DecoderError;
pub use traits::{Decodable, Encodable};
pub use untrusted_rlp::{UntrustedRlp, UntrustedRlpIterator, PayloadInfo, Prototype};
pub use rlpin::{Rlp, RlpIterator};
pub use stream::RlpStream;

/// The RLP encoded empty data (used to mean "null value").
pub const NULL_RLP: [u8; 1] = [0x80; 1];
/// The RLP encoded empty list.
pub const EMPTY_LIST_RLP: [u8; 1] = [0xC0; 1];

/// Shortcut function to decode trusted rlp
///
/// ```rust
/// extern crate rlp;
///
/// fn main () {
/// 	let data = vec![0x83, b'c', b'a', b't'];
/// 	let animal: String = rlp::decode(&data);
/// 	assert_eq!(animal, "cat".to_owned());
/// }
/// ```
pub fn decode<T>(bytes: &[u8]) -> T where T: Decodable {
	let rlp = Rlp::new(bytes);
	rlp.as_val()
}

pub fn decode_list<T>(bytes: &[u8]) -> Vec<T> where T: Decodable {
	let rlp = Rlp::new(bytes);
	rlp.as_list()
}

/// Shortcut function to encode structure into rlp.
///
/// ```rust
/// extern crate rlp;
///
/// fn main () {
/// 	let animal = "cat";
/// 	let out = rlp::encode(&animal).into_vec();
/// 	assert_eq!(out, vec![0x83, b'c', b'a', b't']);
/// }
/// ```
pub fn encode<E>(object: &E) -> ElasticArray1024<u8> where E: Encodable {
	let mut stream = RlpStream::new();
	stream.append(object);
	stream.drain()
}

pub fn encode_list<E, K>(object: &[K]) -> ElasticArray1024<u8> where E: Encodable, K: Borrow<E> {
	let mut stream = RlpStream::new();
	stream.append_list(object);
	stream.drain()
}
