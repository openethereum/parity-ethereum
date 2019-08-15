// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! `NodeCodec` implementation for Rlp

use ethereum_types::H256;
use hash_db::Hasher;
use keccak_hasher::KeccakHasher;
use rlp::{DecoderError, RlpStream, Rlp, Prototype};
use std::marker::PhantomData;
use std::borrow::Borrow;
use trie::{NibbleSlice, NodeCodec, node::Node, ChildReference, Partial};



/// Concrete implementation of a `NodeCodec` with Rlp encoding, generic over the `Hasher`
#[derive(Default, Clone)]
pub struct RlpNodeCodec<H: Hasher> {mark: PhantomData<H>}

const HASHED_NULL_NODE_BYTES : [u8;32] = [0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21];
const HASHED_NULL_NODE : H256 = H256( HASHED_NULL_NODE_BYTES );

/// Encode a partial value with a partial tuple as input.
fn encode_partial_iter<'a>(partial: Partial<'a>, is_leaf: bool) -> impl Iterator<Item = u8> + 'a {
	encode_partial_inner_iter((partial.0).1, partial.1.iter().map(|v| *v), (partial.0).0 > 0, is_leaf)
}

/// Encode a partial value with an iterator as input.
fn encode_partial_from_iterator_iter<'a>(
	mut partial: impl Iterator<Item = u8> + 'a,
	odd: bool,
	is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
	let first = if odd { partial.next().unwrap_or(0) } else { 0 }; 
	encode_partial_inner_iter(first, partial, odd, is_leaf)
}

/// Encode a partial value with an iterator as input.
fn encode_partial_inner_iter<'a>(
	first_byte: u8,
	partial_remaining: impl Iterator<Item = u8> + 'a,
	odd: bool,
	is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
	let encoded_type = if is_leaf {0x20} else {0};
	let first = if odd {
		0x10 + encoded_type + first_byte
	} else {
		encoded_type
	};
	std::iter::once(first).chain(partial_remaining)
}

// NOTE: what we'd really like here is:
// `impl<H: Hasher> NodeCodec<H> for RlpNodeCodec<H> where H::Out: Decodable`
// but due to the current limitations of Rust const evaluation we can't
// do `const HASHED_NULL_NODE: H::Out = H::Out( … … )`. Perhaps one day soon?
impl NodeCodec<KeccakHasher> for RlpNodeCodec<KeccakHasher> {

	type Error = DecoderError;

	fn hashed_null_node() -> <KeccakHasher as Hasher>::Out {
		HASHED_NULL_NODE
	}

	fn decode(data: &[u8]) -> ::std::result::Result<Node, Self::Error> {
		let r = Rlp::new(data);
		match r.prototype()? {
			// either leaf or extension - decode first item with NibbleSlice::???
			// and use is_leaf return to figure out which.
			// if leaf, second item is a value (is_data())
			// if extension, second item is a node (either SHA3 to be looked up and
			// fed back into this function or inline RLP which can be fed back into this function).
			Prototype::List(2) => {
				let enc_nibble = r.at(0)?.data()?;
				let from_encoded = if enc_nibble.is_empty() {
					(NibbleSlice::new(&[]), false)
				} else {
					// check leaf bit from header.
					let is_leaf = enc_nibble[0] & 32 == 32;
					// Check the header bit to see if we're dealing with an odd partial (only a nibble of header info)
					// or an even partial (skip a full byte).
					let (start, byte_offset) = if enc_nibble[0] & 16 == 16 { (0, 1) } else { (1, 0) };
					(NibbleSlice::new_offset(&enc_nibble[start..], byte_offset), is_leaf)
				};
				match from_encoded {
					(slice, true) => Ok(Node::Leaf(slice, r.at(1)?.data()?)),
					(slice, false) => Ok(Node::Extension(slice, r.at(1)?.data()?)),
				}
			},
			// branch - first 16 are nodes, 17th is a value (or empty).
			Prototype::List(17) => {
				let mut nodes = [None as Option<&[u8]>; 16];
				for i in 0..16 {
					let value = r.at(i)?;
					if value.is_empty() {
						nodes[i] = None;
					} else {
						if value.is_data() && value.size() == KeccakHasher::LENGTH {
							nodes[i] = Some(value.data()?);
						} else {
							return Err(DecoderError::Custom("Rlp is not valid."));
						}
					}
				}
				Ok(Node::Branch(nodes, if r.at(16)?.is_empty() { None } else { Some(r.at(16)?.data()?) }))
			},
			// an empty branch index.
			Prototype::Data(0) => Ok(Node::Empty),
			// something went wrong.
			_ => Err(DecoderError::Custom("Rlp is not valid."))
		}
	}

	fn try_decode_hash(data: &[u8]) -> Option<<KeccakHasher as Hasher>::Out> {

		if data.len() == KeccakHasher::LENGTH {
			let mut r = <KeccakHasher as Hasher>::Out::default();
			r.as_mut().copy_from_slice(data);
			Some(r)
		} else {
			None
		}
	}

	fn is_empty_node(data: &[u8]) -> bool {
		Rlp::new(data).is_empty()
	}

	fn empty_node() -> &'static[u8] {
		&[0x80]
	}

	fn leaf_node(partial: Partial, value: &[u8]) -> Vec<u8> {
		let mut stream = RlpStream::new_list(2);
		stream.append_iter(encode_partial_iter(partial, true));
		stream.append(&value);
		stream.drain()
	}

	fn extension_node(
		partial: impl Iterator<Item = u8>,
		number_nibble: usize,
		child_ref: ChildReference<<KeccakHasher as Hasher>::Out>,
	) -> Vec<u8> {
		let mut stream = RlpStream::new_list(2);
		stream.append_iter(encode_partial_from_iterator_iter(partial, number_nibble % 2 > 0, false));
		match child_ref {
			ChildReference::Hash(hash) => stream.append(&hash),
			ChildReference::Inline(inline_data, length) => {
				let bytes = &AsRef::<[u8]>::as_ref(&inline_data)[..length];
				stream.append_raw(bytes, 1)
			},
		};
		stream.drain()
	}

	fn branch_node(
		children: impl Iterator<Item = impl Borrow<Option<ChildReference<<KeccakHasher as Hasher>::Out>>>>,
		maybe_value: Option<&[u8]>,
	) -> Vec<u8> {
		let mut stream = RlpStream::new_list(17);
		for child_ref in children {
			match child_ref.borrow() {
				Some(c) => match c {
					ChildReference::Hash(h) => stream.append(h),
					ChildReference::Inline(inline_data, length) => {
						let bytes = &AsRef::<[u8]>::as_ref(inline_data)[..*length];
						stream.append_raw(bytes, 1)
					},
				},
				None => stream.append_empty_data()
			};
		}
		if let Some(value) = maybe_value {
			stream.append(&&*value);
		} else {
			stream.append_empty_data();
		}
		stream.drain()
	}

	fn branch_node_nibbled(
		_partial: impl Iterator<Item = u8>,
		_number_nibble: usize,
		_children: impl Iterator<Item = impl Borrow<Option<ChildReference<<KeccakHasher as Hasher>::Out>>>>,
		_maybe_value: Option<&[u8]>) -> Vec<u8> {
		unreachable!("This codec is only used with a trie Layout that uses extension node.")
	}

}
