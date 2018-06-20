// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Façade crate for `patricia_trie` for Ethereum specific impls

pub extern crate patricia_trie as trie; // `pub` because we need to import this crate for the tests in `patricia_trie` and there were issues: https://gist.github.com/dvdplm/869251ee557a1b4bd53adc7c971979aa
extern crate ethcore_bytes;
extern crate hashdb;
extern crate keccak_hasher;
extern crate rlp;
extern crate stream_encoder;
extern crate ethereum_types;

use ethcore_bytes::Bytes;
use ethereum_types::H256;
use hashdb::Hasher;
use keccak_hasher::KeccakHasher;
use rlp::{DecoderError, Decodable, RlpStream, Rlp, Prototype};
use std::marker::PhantomData;
use stream_encoder::Stream;
use trie::{NibbleSlice, NodeCodec, node::Node};

pub type RlpCodec = RlpNodeCodec<KeccakHasher>;
pub type TrieError = trie::TrieError<H256, DecoderError>;
pub type Result<T> = trie::Result<T, H256, DecoderError>;

#[derive(Default, Clone)]
pub struct RlpNodeCodec<H: Hasher> {mark: PhantomData<H>}

impl<H: Hasher> NodeCodec<H> for RlpNodeCodec<H>
where H::Out: Decodable
{
	type E = DecoderError;
	type S = RlpStream;
	fn encode(node: &Node) -> Bytes {
		match *node {
			Node::Leaf(ref slice, ref value) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&&*slice.encoded(true));
				stream.append(value);
				stream.out()
			},
			Node::Extension(ref slice, ref raw_rlp) => {
				let mut stream = RlpStream::new_list(2);
				stream.append(&&*slice.encoded(false));
				stream.append_raw(raw_rlp, 1);
				stream.out()
			},
			Node::Branch(ref nodes, ref value) => {
				let mut stream = RlpStream::new_list(17);
				for i in 0..16 {
					stream.append_raw(nodes[i], 1);
				}
				match *value {
					Some(ref n) => { stream.append(n); },
					None => { stream.append_empty_data(); },
				}
				stream.out()
			},
			Node::Empty => {
				let mut stream = RlpStream::new();
				stream.append_empty_data();
				stream.out()
			}
		}
	}
	fn decode(data: &[u8]) -> ::std::result::Result<Node, Self::E> {
		let r = Rlp::new(data);
		match r.prototype()? {
			// either leaf or extension - decode first item with NibbleSlice::???
			// and use is_leaf return to figure out which.
			// if leaf, second item is a value (is_data())
			// if extension, second item is a node (either SHA3 to be looked up and
			// fed back into this function or inline RLP which can be fed back into this function).
			Prototype::List(2) => match NibbleSlice::from_encoded(r.at(0)?.data()?) {
				(slice, true) => Ok(Node::Leaf(slice, r.at(1)?.data()?)),
				(slice, false) => Ok(Node::Extension(slice, r.at(1)?.as_raw())),
			},
			// branch - first 16 are nodes, 17th is a value (or empty).
			Prototype::List(17) => {
				let mut nodes = [&[] as &[u8]; 16];
				for i in 0..16 {
					nodes[i] = r.at(i)?.as_raw();
				}
				Ok(Node::Branch(nodes, if r.at(16)?.is_empty() { None } else { Some(r.at(16)?.data()?) }))
			},
			// an empty branch index.
			Prototype::Data(0) => Ok(Node::Empty),
			// something went wrong.
			_ => Err(DecoderError::Custom("Rlp is not valid."))
		}
	}
	fn try_decode_hash(data: &[u8]) -> Option<H::Out> {
		let r = Rlp::new(data);
		if r.is_data() && r.size() == H::LENGTH { 
			Some(r.as_val().expect("Hash is the correct size; qed"))
		} else {
			None
		}
	}

	fn is_empty_node(data: &[u8]) -> bool {
		// REVIEW: Could also be `Rlp::new(data).is_empty()` – better?
		data.len() != 0 && (data[0] == 0xC0 || data[0] == 0x80)
	}
}
